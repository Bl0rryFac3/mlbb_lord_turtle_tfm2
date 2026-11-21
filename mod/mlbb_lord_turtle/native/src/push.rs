//! Allied Lord prototype. All mutable gameplay state lives in zero-stat named
//! buffs on an in-simulation controller and the summoned Lord itself. No global
//! match-state maps: parallel pre-sims and watched matches cannot share timers.
use crate::probe::write_log;
use mod_api_stable::{
    AttackTypeV1, BuffV1, CcKindV1, CcV1, DamageTypeV1, EntityKindV1, InputTargetKindV1,
    InputTargetV1, StableSim, StatV1, UnitAttackV1,
};
type Point = (u64, u64);
const INIT: &str = "mlbb_lord_controller_ready";
const CONTROLLER: &str = "mlbb_lord_controller";
const PUSH_UNIT: &str = "mlbb_lord_push";
const STEP: &str = "mlbb_lord_step:";
const PATH: &str = "mlbb_lord_path:";
const CD: &str = "mlbb_lord_attack_cd";
const BODY_SWING: &str = "mlbb_lord_body_swing:";
const BODY_FACE: &str = "mlbb_lord_body_face:";
const BASE_GUARD: &str = "mlbb_lord_base_guard:";
// Existing attack frames total 0.58s; 35 ticks covers the full swing.
const ATTACK_ANIMATION_TICKS: u64 = 35;
const TTL: u64 = 90 * 60;
const SPEED: u64 = 1100;
const RANGE: u64 = 55_000;
const WAVE_START: usize = 10;
const WAVE_PERIOD: usize = 660;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lane {
    Top,
    Mid,
    Bottom,
}
impl Lane {
    fn code(self) -> usize {
        match self {
            Self::Top => 0,
            Self::Mid => 1,
            Self::Bottom => 2,
        }
    }
    fn from_code(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Top),
            1 => Some(Self::Mid),
            2 => Some(Self::Bottom),
            _ => None,
        }
    }
}
#[derive(Clone, Copy)]
struct Tower {
    id: usize,
    team: usize,
    pos: Point,
    hp: usize,
    alive: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pending {
    team: usize,
    player: usize,
    score: usize,
    due: usize,
}
fn log(sim: &StableSim<'_>, s: &str) {
    write_log(&format!(
        "PUSH tick={} seed={} origin={:?} {s}\n",
        sim.tick(),
        sim.seed(),
        sim.sim_origin()
    ));
}
fn names(sim: &StableSim<'_>, id: usize) -> Vec<String> {
    sim.get_entity(id)
        .map(|e| {
            (0..e.buff_count())
                .filter_map(|i| e.buff_at(i))
                .map(|b| b.name().to_owned())
                .collect()
        })
        .unwrap_or_default()
}
fn add_marker(sim: &mut StableSim<'_>, id: usize, name: &str) {
    debug_assert!(name.len() <= 64);
    sim.add_buff(id, &BuffV1::named(name));
}
fn replace_marker(sim: &mut StableSim<'_>, id: usize, prefix: &str, name: &str) {
    for old in names(sim, id) {
        if old.starts_with(prefix) {
            sim.entity_remove_buff(id, &old);
        }
    }
    add_marker(sim, id, name);
}
fn find_named(sim: &StableSim<'_>, name: &str) -> Option<usize> {
    (0..sim.entity_count()).find_map(|i| {
        let e = sim.entity_at(i)?;
        (e.is_alive() && e.name().as_deref() == Some(name)).then(|| e.id())
    })
}
fn nexuses(sim: &StableSim<'_>) -> Option<[(usize, Point); 2]> {
    let mut out = [None, None];
    for i in 0..sim.entity_count() {
        let Some(e) = sim.entity_at(i) else { continue };
        if e.kind() == Some(EntityKindV1::Nexus) && e.team() < 2 {
            out[e.team()] = Some((e.id(), e.pos()));
        }
    }
    Some([out[0]?, out[1]?])
}
fn towers(sim: &StableSim<'_>) -> Vec<Tower> {
    // The live entity list is authoritative. Also read the indexed helper for
    // compatibility, but never mistake a nexus for a tower or duplicate IDs.
    let mut out = Vec::new();
    let ids = (0..sim.entity_count())
        .filter_map(|i| sim.entity_at(i).map(|e| e.id()))
        .chain((0..sim.tower_count()).map(|i| sim.tower_id_at(i)));
    for id in ids {
        if out.iter().any(|t: &Tower| t.id == id) {
            continue;
        }
        let Some(e) = sim.get_entity(id) else {
            continue;
        };
        if e.kind() != Some(EntityKindV1::Tower) && !(e.kind().is_none() && e.is_tower()) {
            continue;
        }
        out.push(Tower {
            id,
            team: e.team(),
            pos: e.pos(),
            hp: e.hp().0,
            alive: e.is_alive() && e.hp().0 > 0,
        });
    }
    out
}
fn next_lane_tower(a: Point, b: Point, all: &[Tower], enemy: usize, lane: Lane) -> Option<Tower> {
    let (start, end) = if enemy == 1 { (a, b) } else { (b, a) };
    all.iter()
        .filter(|t| t.team == enemy && t.alive && lane_of(a, b, t.pos) == lane)
        .min_by_key(|t| (projection(start, end, t.pos), t.id))
        .copied()
}
fn registered_guards(markers: &[String], all: &[Tower], enemy: usize) -> Vec<Tower> {
    all.iter()
        .filter(|t| {
            t.team == enemy
                && t.alive
                && markers.iter().any(|m| {
                    m.strip_prefix(BASE_GUARD)
                        .and_then(|v| v.parse::<usize>().ok())
                        == Some(t.id)
                })
        })
        .copied()
        .collect()
}
fn register_nexus_guards(
    sim: &mut StableSim<'_>,
    lord: usize,
    all: &[Tower],
    enemy: usize,
    nexus: Point,
    reason: &str,
) -> Vec<Tower> {
    // The base has two core defenders. Select the surviving towers closest
    // to the protected nexus, not by their inferred ordinary lane.
    let mut guards: Vec<_> = all
        .iter()
        .filter(|t| t.team == enemy && t.alive)
        .copied()
        .collect();
    guards.sort_by_key(|t| (dist_sq(t.pos, nexus), t.id));
    // If one defender is already dead, don't pair the last close guard with
    // a distant lane tower. Additional guards can be discovered if the core
    // remains protected after this close group has fallen.
    if let Some(first) = guards.first() {
        let near = dist_sq(first.pos, nexus);
        guards.retain(|t| dist_sq(t.pos, nexus) <= near.saturating_mul(9) / 4);
    }
    guards.truncate(2);
    for t in &guards {
        let marker = format!("{BASE_GUARD}{}", t.id);
        if !names(sim, lord).contains(&marker) {
            add_marker(sim, lord, &marker);
        }
    }
    if !guards.is_empty() {
        log(
            sim,
            &format!(
                "NEXUS_GUARD_PHASE lord={lord} reason={reason} guards={:?}",
                guards.iter().map(|t| (t.id, t.pos)).collect::<Vec<_>>()
            ),
        );
    }
    guards
}
fn projection(a: Point, b: Point, p: Point) -> i128 {
    (p.0 as i128 - a.0 as i128) * (b.0 as i128 - a.0 as i128)
        + (p.1 as i128 - a.1 as i128) * (b.1 as i128 - a.1 as i128)
}
fn lane_of(a: Point, b: Point, p: Point) -> Lane {
    let dx = b.0 as i128 - a.0 as i128;
    let dy = b.1 as i128 - a.1 as i128;
    let cross = dx * (p.1 as i128 - a.1 as i128) - dy * (p.0 as i128 - a.0 as i128);
    // Perpendicular distance within 48,000 units counts as the mid lane.
    if cross * cross <= 48_000i128.pow(2) * (dx * dx + dy * dy) {
        Lane::Mid
    } else if cross < 0 {
        Lane::Top
    } else {
        Lane::Bottom
    }
}
fn route(a: Point, b: Point, all: &[Tower], lane: Lane) -> Vec<Point> {
    let mut points: Vec<_> = all
        .iter()
        .filter(|t| t.team < 2 && lane_of(a, b, t.pos) == lane)
        .map(|t| t.pos)
        .collect();
    // Standard 0.7.2 three-lane map: blue bottom-left, red top-right. Side
    // lanes turn at the corresponding map corner, rather than cutting jungle.
    if lane == Lane::Top {
        points.push((a.0.min(b.0), a.1.min(b.1)));
    }
    if lane == Lane::Bottom {
        points.push((a.0.max(b.0), a.1.max(b.1)));
    }
    points.sort_by_key(|p| projection(a, b, *p));
    points.dedup();
    let mut path = vec![a];
    path.extend(points);
    path.push(b);
    path
}
fn weakest(a: Point, b: Point, all: &[Tower], enemy: usize) -> Lane {
    [Lane::Mid, Lane::Top, Lane::Bottom]
        .into_iter()
        .min_by_key(|lane| {
            let ts: Vec<_> = all
                .iter()
                .filter(|t| t.team == enemy && t.alive && lane_of(a, b, t.pos) == *lane)
                .collect();
            let front = ts.iter().min_by_key(|t| {
                if enemy == 1 {
                    projection(a, b, t.pos)
                } else {
                    -projection(a, b, t.pos)
                }
            });
            (
                ts.len(),
                front.map_or(0, |t| t.hp),
                if *lane == Lane::Mid { 0 } else { 1 },
                lane.code(),
            )
        })
        .unwrap_or(Lane::Mid)
}
fn next_wave(tick: usize) -> usize {
    if tick < WAVE_START {
        WAVE_START
    } else {
        WAVE_START + ((tick - WAVE_START) / WAVE_PERIOD + 1) * WAVE_PERIOD
    }
}
fn scores(sim: &StableSim<'_>) -> Vec<(usize, usize, usize)> {
    (0..sim.player_count())
        .filter_map(|i| {
            let p = sim.player_at(i)?;
            if p.team() > 1 {
                return None;
            }
            let value = p
                .statistics_json("epic_secure")?
                .trim()
                .parse::<usize>()
                .ok()?;
            Some((p.id(), p.team(), value))
        })
        .collect()
}
fn pending_name(p: Pending) -> String {
    format!(
        "mlbb_lord_pending:{}:{}:{}:{}",
        p.team, p.player, p.score, p.due
    )
}
fn parse_pending(name: &str) -> Option<Pending> {
    let mut s = name.strip_prefix("mlbb_lord_pending:")?.split(':');
    let p = Pending {
        team: s.next()?.parse().ok()?,
        player: s.next()?.parse().ok()?,
        score: s.next()?.parse().ok()?,
        due: s.next()?.parse().ok()?,
    };
    (p.team < 2 && s.next().is_none()).then_some(p)
}
fn store_paths(sim: &mut StableSim<'_>, holder: usize, ns: [(usize, Point); 2], all: &[Tower]) {
    for lane in [Lane::Top, Lane::Mid, Lane::Bottom] {
        for (i, (x, y)) in route(ns[0].1, ns[1].1, all, lane).into_iter().enumerate() {
            add_marker(sim, holder, &format!("{PATH}{}:{i}:{x}:{y}", lane.code()));
        }
    }
}
fn read_path(markers: &[String], lane: Lane) -> Vec<Point> {
    let prefix = format!("{PATH}{}:", lane.code());
    let mut path: Vec<(usize, Point)> = Vec::new();
    for n in markers {
        let Some(s) = n.strip_prefix(&prefix) else {
            continue;
        };
        let parts: Vec<_> = s.split(':').collect();
        if parts.len() != 3 {
            continue;
        }
        if let (Ok(i), Ok(x), Ok(y)) = (parts[0].parse(), parts[1].parse(), parts[2].parse()) {
            path.push((i, (x, y)));
        }
    }
    path.sort_by_key(|v| v.0);
    path.into_iter().map(|v| v.1).collect()
}
fn holder(sim: &StableSim<'_>) -> Option<usize> {
    if let Some(ns) = nexuses(sim) {
        if names(sim, ns[0].0).iter().any(|s| s == INIT) {
            return Some(ns[0].0);
        }
    }
    find_named(sim, CONTROLLER)
}
pub fn start(sim: &mut StableSim<'_>) {
    if holder(sim).is_some() {
        return;
    }
    let Some(ns) = nexuses(sim) else { return };
    let all = towers(sim);
    // This prototype targets the standard three-lane orientation only.
    if ns[0].1 .0 >= ns[1].1 .0 || ns[0].1 .1 <= ns[1].1 .1 || all.len() < 6 {
        return;
    }
    add_marker(sim, ns[0].0, INIT);
    let carrier = if names(sim, ns[0].0).iter().any(|s| s == INIT) {
        ns[0].0
    } else {
        // Some host entity kinds may not accept buffs. Fall back to a hidden,
        // untargetable controller summon instead of storing state across sims.
        let stat = StatV1 {
            hp: 1,
            ..Default::default()
        };
        let Some(id) = sim.spawn_unit(
            CONTROLLER,
            usize::MAX,
            0,
            ns[0].1 .0,
            ns[0].1 .1,
            216000,
            &stat,
            &zero_attack(),
        ) else {
            log(sim, "controller spawn failed");
            return;
        };
        sim.add_buff(
            id,
            &BuffV1 {
                undying: true,
                ignore_wall: true,
                ..BuffV1::named("mlbb_lord_controller_protection")
            },
        );
        sim.entity_banish(id, id, 240, "", "");
        add_marker(sim, id, INIT);
        id
    };
    for (player, _, score) in scores(sim) {
        add_marker(sim, carrier, &format!("mlbb_lord_seen:{player}:{score}"));
    }
    store_paths(sim, carrier, ns, &all);
    log(
        sim,
        &format!("controller_ready id={carrier} wave_start={WAVE_START} wave_period={WAVE_PERIOD}"),
    );
}
fn zero_attack() -> UnitAttackV1 {
    UnitAttackV1 {
        attack_ratio: 0,
        attack: 0,
        range: 0,
        cooltime: 60,
        duration: 0,
        start_timing: 0,
        cancelable: false,
        attack_type: AttackTypeV1::BaseAttack.code(),
    }
}
fn spawn(sim: &mut StableSim<'_>, carrier: usize, p: Pending) -> Option<usize> {
    let Some(ns) = nexuses(sim) else { return None };
    let all = towers(sim);
    let lane = weakest(ns[0].1, ns[1].1, &all, 1 - p.team);
    let mut path = read_path(&names(sim, carrier), lane);
    if path.len() < 2 {
        log(sim, "missing route: summon skipped");
        return None;
    }
    if p.team == 1 {
        path.reverse();
    }
    let summoner = sim
        .get_player(p.player)
        .and_then(|p| p.champion())
        .map(|e| e.id())
        .unwrap_or(usize::MAX);
    let stat = StatV1 {
        hp: 6000,
        attack: 220,
        defence: 60,
        magic_resistance: 60,
        move_speed: 0,
        ..Default::default()
    };
    let Some(id) = sim.spawn_unit(
        PUSH_UNIT,
        summoner,
        p.team,
        path[0].0,
        path[0].1,
        TTL,
        &stat,
        &zero_attack(),
    ) else {
        log(sim, "allied Lord spawn failed");
        return None;
    };
    add_marker(sim, id, &format!("mlbb_lord_lane:{}", lane.code()));
    add_marker(sim, id, &format!("{STEP}1"));
    for (i, (x, y)) in path.iter().enumerate() {
        add_marker(sim, id, &format!("{PATH}{}:{i}:{x}:{y}", lane.code()));
    }
    // Suppress ghoul-style automatic attacks. This unit's attacks and movement
    // are controlled below; enemies can still damage and kill it normally.
    sim.apply_cc(id, &CcV1::of_kind(CcKindV1::BlockAttack, TTL));
    log(
        sim,
        &format!(
            "SPAWN team={} secured_by_player={} score={} lane={lane:?} id={id} path={path:?}",
            p.team, p.player, p.score
        ),
    );
    for t in &all {
        log(
            sim,
            &format!(
                "TOWER id={} team={} lane={:?} hp={} alive={} pos={:?}",
                t.id,
                t.team,
                lane_of(ns[0].1, ns[1].1, t.pos),
                t.hp,
                t.alive,
                t.pos
            ),
        );
    }
    Some(id)
}
pub fn tick(sim: &mut StableSim<'_>) {
    if sim.is_end() {
        return;
    }
    let Some(carrier) = holder(sim) else {
        if sim.tick() % 60 == 0 {
            start(sim);
        }
        return;
    };
    if sim.get_entity(carrier).and_then(|e| e.name()).as_deref() == Some(CONTROLLER) {
        sim.entity_banish(carrier, carrier, 240, "", "");
    }
    let current = scores(sim);
    let carrier_markers = names(sim, carrier);
    for (player, team, score) in current {
        let prefix = format!("mlbb_lord_seen:{player}:");
        let previous = carrier_markers
            .iter()
            .find_map(|s| s.strip_prefix(&prefix)?.parse::<usize>().ok());
        if previous == Some(score) {
            continue;
        }
        replace_marker(sim, carrier, &prefix, &format!("{prefix}{score}"));
        // No catch-up spawning when attaching to an already-running match.
        if previous.is_some_and(|v| score > v) {
            let pending = Pending {
                team,
                player,
                score,
                due: next_wave(sim.tick()),
            };
            add_marker(sim, carrier, &pending_name(pending));
            log(
                sim,
                &format!(
                    "SECURE team={team} player={player} counter={score} summon_due={}",
                    pending.due
                ),
            );
        }
    }
    let mut spawned = Vec::new();
    for name in names(sim, carrier) {
        let Some(p) = parse_pending(&name) else {
            continue;
        };
        if sim.tick() >= p.due {
            // Consume first: a failed spawn cannot create a duplicate each tick.
            sim.entity_remove_buff(carrier, &name);
            if let Some(id) = spawn(sim, carrier, p) {
                spawned.push(id);
            }
        }
    }
    let units: Vec<_> = (0..sim.entity_count())
        .filter_map(|i| {
            let e = sim.entity_at(i)?;
            (e.kind() == Some(EntityKindV1::Summon)
                && e.is_alive()
                && e.name().as_deref() == Some(PUSH_UNIT))
            .then(|| e.id())
        })
        .collect();
    for id in units {
        let before = sim.get_entity(id).map(|e| e.pos());
        step_unit(sim, id);
        if let Some((old, new)) = before.zip(sim.get_entity(id).map(|e| e.pos())) {
            if old.0 != new.0 {
                set_facing(sim, id, new.0 > old.0);
            }
        }
        draw_body(sim, id, spawned.contains(&id));
    }
}

// Generic summons did not display the Animation CC in the user's game.
// Keep their combat entity and health bar; draw their body using the same
// replay-recorded view-event pipeline that already displays Lord's hit VFX.
// Three-tick followed frames reduce event creation to 20/second. Movement and
// damage still run every tick. A partial first frame aligns off-cadence spawns.
const BODY_PERIOD: usize = 3;
fn body_hold(tick: usize, spawned: bool) -> Option<u64> {
    let phase = tick % BODY_PERIOD;
    (phase == 0 || spawned).then_some((BODY_PERIOD - phase) as u64)
}
fn set_facing(sim: &mut StableSim<'_>, id: usize, right: bool) {
    let name = format!("{BODY_FACE}{}", if right { "right" } else { "left" });
    if !names(sim, id).contains(&name) {
        replace_marker(sim, id, BODY_FACE, &name);
    }
}
fn body_frame(now: usize, started: Option<usize>) -> (&'static str, usize) {
    if let Some(age) = started.and_then(|s| now.checked_sub(s)).filter(|a| *a < 35) {
        return (
            "attack",
            if age < 9 {
                0
            } else if age < 21 {
                1
            } else {
                2
            },
        );
    }
    ("glide", ((now as u128 * 25 / 120) % 9) as usize)
}
fn draw_body(sim: &mut StableSim<'_>, id: usize, spawned: bool) {
    let Some(hold) = body_hold(sim.tick(), spawned) else {
        return;
    };
    let Some(team) = sim
        .get_entity(id)
        .filter(|e| e.is_alive())
        .map(|e| e.team())
    else {
        return;
    };
    let markers = names(sim, id);
    let started = markers
        .iter()
        .find_map(|n| n.strip_prefix(BODY_SWING)?.parse().ok());
    let face = markers
        .iter()
        .find_map(|n| n.strip_prefix(BODY_FACE))
        .unwrap_or(if team == 0 { "right" } else { "left" });
    let (action, frame) = body_frame(sim.tick(), started);
    let name = format!("mlbb_lord_turtle:body_{action}_{frame}_{face}_h{hold}");
    let input = InputTargetV1::target(id);
    sim.play_view_effect(&name, id, &input, 0, 0, hold);
}

fn dist_sq(a: Point, b: Point) -> u128 {
    let dx = a.0.abs_diff(b.0) as u128;
    let dy = a.1.abs_diff(b.1) as u128;
    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
}
fn move_toward(a: Point, b: Point, speed: u64) -> Point {
    let dx = b.0 as f64 - a.0 as f64;
    let dy = b.1 as f64 - a.1 as f64;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= speed as f64 {
        return b;
    }
    (
        (a.0 as f64 + dx / len * speed as f64).round() as u64,
        (a.1 as f64 + dy / len * speed as f64).round() as u64,
    )
}
fn movement_locked(sim: &StableSim<'_>, id: usize) -> bool {
    let Some(e) = sim.get_entity(id) else {
        return true;
    };
    (0..e.cc_count()).filter_map(|i| e.cc_at(i)).any(|c| {
        matches!(
            CcKindV1::from_code(c.kind),
            Some(
                CcKindV1::Animation
                    | CcKindV1::Airborne
                    | CcKindV1::Stun
                    | CcKindV1::Bind
                    | CcKindV1::ForceMove
                    | CcKindV1::Fear
                    | CcKindV1::Charm
                    | CcKindV1::Taunt
            )
        )
    })
}
fn step_unit(sim: &mut StableSim<'_>, id: usize) {
    let Some((team, pos, attack)) = sim
        .get_entity(id)
        .map(|e| (e.team(), e.pos(), e.stat().attack))
    else {
        return;
    };
    if team > 1 || movement_locked(sim, id) {
        return;
    }
    let markers = names(sim, id);
    let Some(lane) = markers
        .iter()
        .find_map(|s| Lane::from_code(s.strip_prefix("mlbb_lord_lane:")?.parse().ok()?))
    else {
        return;
    };
    let path = read_path(&markers, lane);
    if path.len() < 2 {
        return;
    }
    let Some(ns) = nexuses(sim) else { return };
    let all = towers(sim);
    let lane_tower = next_lane_tower(ns[0].1, ns[1].1, &all, 1 - team, lane);
    let lane_open = lane_tower.is_none();
    let nexus_targetable = sim
        .get_entity(ns[1 - team].0)
        .is_some_and(|e| e.is_targetable());
    let mut guards = registered_guards(&markers, &all, 1 - team);
    if lane_open && guards.is_empty() && !nexus_targetable {
        guards = register_nexus_guards(
            sim,
            id,
            &all,
            1 - team,
            ns[1 - team].1,
            "nexus_not_targetable",
        );
    }
    let guard_tower = guards
        .iter()
        .min_by_key(|t| (dist_sq(pos, t.pos), t.id))
        .copied();
    let required_tower = lane_tower.or(guard_tower);
    let guard_goal = lane_tower.is_none() && guard_tower.is_some();
    let nexus_allowed = lane_open && guards.is_empty() && nexus_targetable;
    let structure_id = required_tower.map(|t| t.id).unwrap_or(ns[1 - team].0);
    let goal_marker = format!("mlbb_lord_goal:{structure_id}");
    if !markers.iter().any(|n| n == &goal_marker) {
        replace_marker(sim, id, "mlbb_lord_goal:", &goal_marker);
        log(sim,&format!("STRUCTURE_GOAL lord={id} team={team} lane={lane:?} target={structure_id} kind={} lane_towers_alive={} remaining_enemy_towers={} guards_alive={} nexus_targetable={nexus_targetable}",
            if guard_goal {"nexus_guard"} else if lane_open {"nexus"} else {"lane_tower"},
            all.iter().filter(|t|t.team==1-team && t.alive && lane_of(ns[0].1,ns[1].1,t.pos)==lane).count(),
            all.iter().filter(|t|t.team==1-team && t.alive).count(),guards.len()));
    }
    let target = (0..sim.entity_count())
        .filter_map(|i| {
            let e = sim.entity_at(i)?;
            if e.team() != 1 - team || !e.is_alive() || !e.is_targetable() {
                return None;
            }
            let kind = e.kind()?;
            let priority = match kind {
                EntityKindV1::Tower => {
                    // A nearer inner tower cannot take priority over a still
                    // living outer tower in this lane.
                    if Some(e.id()) != required_tower.map(|t| t.id) {
                        return None;
                    }
                    0
                }
                EntityKindV1::Nexus => {
                    if !nexus_allowed {
                        return None;
                    }
                    1
                }
                EntityKindV1::Minion => 2,
                EntityKindV1::Champion => 3,
                EntityKindV1::Summon => 4,
                _ => return None,
            };
            let d = dist_sq(pos, e.pos());
            (d <= (RANGE as u128).pow(2)).then(|| (priority, d, e.id(), e.pos()))
        })
        .min_by_key(|t| (t.0, t.1, t.2));
    if let Some((_, _, target, hit)) = target {
        if !markers.iter().any(|n| n == CD) && !movement_locked(sim, id) {
            let mut animation = CcV1::of_kind(CcKindV1::Animation, ATTACK_ANIMATION_TICKS);
            animation.set_name("attack");
            sim.apply_cc(id, &animation);
            let now = sim.tick();
            replace_marker(sim, id, BODY_SWING, &format!("{BODY_SWING}{now}"));
            if hit.0 != pos.0 {
                set_facing(sim, id, hit.0 > pos.0);
            }
            let nexus_before = if target == ns[1 - team].0 {
                sim.get_entity(target).map(|e| (e.hp().0, e.shield()))
            } else {
                None
            };
            sim.deal_damage_typed(
                id,
                target,
                attack,
                DamageTypeV1::Ad,
                AttackTypeV1::BaseAttack,
            );
            if let Some((before, before_shield)) = nexus_before {
                let (after, after_shield) = sim
                    .get_entity(target)
                    .map(|e| (e.hp().0, e.shield()))
                    .unwrap_or((0, 0));
                if before > 0 && after >= before && after_shield >= before_shield {
                    // Some hosts expose the nexus as targetable while its
                    // guard protection rejects damage. Respect that outcome
                    // too, instead of repeatedly hitting an invulnerable core.
                    register_nexus_guards(
                        sim,
                        id,
                        &all,
                        1 - team,
                        ns[1 - team].1,
                        "nexus_rejected_damage",
                    );
                }
            }
            sim.add_buff(id, &BuffV1::timed(CD, 60));
            let input = InputTargetV1 {
                kind: InputTargetKindV1::Pos.code(),
                x: hit.0,
                y: hit.1,
                ..Default::default()
            };
            sim.play_view_effect(crate::HIT_EFFECT, id, &input, 0, 0, 26);
        }
        return;
    }
    if movement_locked(sim, id) {
        return;
    }
    if required_tower.is_none() && !nexus_allowed {
        // Do not march into a protected core if no defender is currently
        // exposed through the API. The log records the protected state.
        return;
    }
    if let Some(tower) = required_tower {
        let (start, end) = (ns[team].1, ns[1 - team].1);
        let passed = projection(start, end, pos) >= projection(start, end, tower.pos);
        let nearby = dist_sq(pos, tower.pos) <= 150_000u128.pow(2);
        let entering_base = guard_goal && dist_sq(pos, ns[1 - team].1) <= 350_000u128.pow(2);
        if passed || nearby || entering_base {
            // Do not follow a stale waypoint past a living inner tower. Go
            // back to it if displaced past it; wait if it is untargetable.
            if dist_sq(pos, tower.pos) > (RANGE as u128).pow(2) {
                let next = move_toward(pos, tower.pos, SPEED);
                sim.entity_set_pos(id, next.0, next.1);
            }
            return;
        }
    }
    let mut index = markers
        .iter()
        .find_map(|s| s.strip_prefix(STEP)?.parse::<usize>().ok())
        .unwrap_or(1);
    let (start, end) = (ns[team].1, ns[1 - team].1);
    let progress = projection(start, end, pos);
    if index >= path.len()
        || (progress > projection(start, end, path[index])
            && dist_sq(pos, path[index]) > (SPEED as u128).pow(2))
    {
        // Recover an invalid/stale route after displacement or returning to a
        // required tower. Structure gating above still prevents skipping one.
        index = path
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, p)| projection(start, end, **p) > progress)
            .map(|(i, _)| i)
            .unwrap_or(path.len() - 1);
        replace_marker(sim, id, STEP, &format!("{STEP}{index}"));
    }
    // Never bypass a living selected-lane tower, even if it has temporarily
    // become untargetable: stop at its route waypoint and wait.
    if dist_sq(pos, path[index]) <= (SPEED as u128).pow(2) {
        let blocking = all
            .iter()
            .any(|t| t.team != team && t.alive && dist_sq(t.pos, path[index]) < 10_000u128.pow(2));
        if blocking {
            return;
        }
        index += 1;
        replace_marker(sim, id, STEP, &format!("{STEP}{index}"));
        if index >= path.len() {
            return;
        }
    }
    let next = move_toward(pos, path[index], SPEED);
    sim.entity_set_pos(id, next.0, next.1);
}

#[cfg(test)]
mod tests;
