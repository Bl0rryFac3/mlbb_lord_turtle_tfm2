use super::*;
use mod_api_stable::{EntityHandleV1, PlayerHandleV1, SimCtxV1, SimVtableV1};
use std::{ffi::c_void, mem::size_of};
const BLUE: Point = (96000, 864000);
const RED: Point = (864000, 96000);
fn tower(team: usize, pos: Point) -> Tower {
    Tower {
        id: 0,
        team,
        pos,
        hp: 2000,
        alive: true,
    }
}
fn map_towers() -> Vec<Tower> {
    let mut towers = vec![
        tower(0, (96000, 650000)),
        tower(0, (96000, 450000)),
        tower(1, (450000, 96000)),
        tower(1, (650000, 96000)),
        tower(0, (250000, 710000)),
        tower(0, (370000, 590000)),
        tower(1, (590000, 370000)),
        tower(1, (710000, 250000)),
        tower(0, (310000, 864000)),
        tower(0, (510000, 864000)),
        tower(1, (864000, 510000)),
        tower(1, (864000, 310000)),
    ];
    for (i, t) in towers.iter_mut().enumerate() {
        t.id = i + 4;
    }
    towers
}
#[test]
fn lane_classification_and_mid_band() {
    assert_eq!(lane_of(BLUE, RED, (96000, 450000)), Lane::Top);
    assert_eq!(lane_of(BLUE, RED, (500000, 500000)), Lane::Mid);
    assert_eq!(lane_of(BLUE, RED, (864000, 510000)), Lane::Bottom);
}
#[test]
fn fewer_enemy_towers_wins_over_lower_hp() {
    let mut t = map_towers();
    t[2].alive = false;
    t[6].hp = 1;
    assert_eq!(weakest(BLUE, RED, &t, 1), Lane::Top);
}
#[test]
fn front_tower_hp_breaks_ties_then_mid() {
    let mut t = map_towers();
    assert_eq!(weakest(BLUE, RED, &t, 1), Lane::Mid);
    t[10].hp = 500;
    assert_eq!(weakest(BLUE, RED, &t, 1), Lane::Bottom);
}
#[test]
fn enemy_team_filter_works_for_red() {
    let mut t = map_towers();
    t[0].alive = false;
    t[1].alive = false;
    assert_eq!(weakest(BLUE, RED, &t, 0), Lane::Top);
    assert_eq!(weakest(BLUE, RED, &t, 1), Lane::Mid);
}
#[test]
fn empty_lanes_tie_to_mid() {
    assert_eq!(weakest(BLUE, RED, &[], 1), Lane::Mid);
}
#[test]
fn route_turns_at_corner_and_red_reverses() {
    let t = map_towers();
    let r = route(BLUE, RED, &t, Lane::Top);
    assert_eq!(r[0], BLUE);
    assert_eq!(*r.last().unwrap(), RED);
    assert!(r.contains(&(96000, 96000)));
    let mut rr = r.clone();
    rr.reverse();
    assert_eq!(rr[0], RED);
    assert_eq!(*rr.last().unwrap(), BLUE);
    assert!(!route(BLUE, RED, &t, Lane::Mid).contains(&(96000, 96000)));
}
#[test]
fn wave_schedule_matches_bundle_and_is_strictly_next() {
    for (t, e) in [
        (0, 10),
        (9, 10),
        (10, 670),
        (669, 670),
        (670, 1330),
        (20316, 20470),
    ] {
        assert_eq!(next_wave(t), e);
    }
}
#[test]
fn pending_marker_roundtrip_and_validation() {
    let p = Pending {
        team: 1,
        player: 6,
        score: 1,
        due: 20470,
    };
    assert_eq!(parse_pending(&pending_name(p)), Some(p));
    assert!(parse_pending("mlbb_lord_pending:2:1:1:5").is_none());
    assert!(parse_pending("mlbb_lord_pending:0:1:1:5:6").is_none());
}
#[test]
fn movement_step_is_bounded_and_arrives_exactly() {
    assert_eq!(move_toward((0, 0), (3000, 4000), 1000), (600, 800));
    assert_eq!(move_toward((0, 0), (3, 4), 10), (3, 4));
    assert_eq!(move_toward((100, 100), (100, 100), 10), (100, 100));
}

#[derive(Clone)]
struct Entity {
    kind: EntityKindV1,
    name: String,
    team: usize,
    pos: Point,
    hp: usize,
    max: usize,
    alive: bool,
    targetable: bool,
    buffs: Vec<BuffV1>,
    cc: Vec<CcV1>,
    stat: StatV1,
}
struct Player {
    team: usize,
    score: usize,
    champion: Option<usize>,
}
struct World {
    entity_reads: std::cell::Cell<usize>,
    name_reads: std::cell::Cell<usize>,
    buff_reads: std::cell::Cell<usize>,
    tick: usize,
    entities: Vec<Entity>,
    players: Vec<Player>,
    tower_ids: Vec<usize>,
    nexus_buffs: bool,
    reject_nexus_damage: bool,
    spawns: Vec<(String, usize, usize)>,
    damage: Vec<(usize, usize, usize)>,
    effects: Vec<(String, usize, u64, u64, u64)>,
    effect_inputs: Vec<InputTargetV1>,
}
fn entity(kind: EntityKindV1, name: &str, team: usize, pos: Point) -> Entity {
    Entity {
        kind,
        name: name.into(),
        team,
        pos,
        hp: 2000,
        max: 2000,
        alive: true,
        targetable: true,
        buffs: vec![],
        cc: vec![],
        stat: StatV1 {
            attack: 220,
            hp: 2000,
            ..Default::default()
        },
    }
}
fn fixture() -> World {
    let mut w = World {
        entity_reads: Default::default(),
        name_reads: Default::default(),
        buff_reads: Default::default(),
        tick: 0,
        entities: vec![
            entity(EntityKindV1::Nexus, "nexus", 0, BLUE),
            entity(EntityKindV1::Nexus, "nexus", 1, RED),
            entity(EntityKindV1::Champion, "blue", 0, BLUE),
            entity(EntityKindV1::Champion, "red", 1, RED),
        ],
        players: vec![
            Player {
                team: 0,
                score: 0,
                champion: Some(2),
            },
            Player {
                team: 1,
                score: 0,
                champion: Some(3),
            },
        ],
        tower_ids: vec![],
        nexus_buffs: true,
        reject_nexus_damage: false,
        spawns: vec![],
        damage: vec![],
        effects: vec![],
        effect_inputs: vec![],
    };
    for t in map_towers() {
        w.tower_ids.push(w.entities.len());
        w.entities
            .push(entity(EntityKindV1::Tower, "tower", t.team, t.pos));
    }
    w
}
unsafe fn w<'a>(s: *const c_void) -> &'a World {
    &*(s as *const World)
}
unsafe fn wm<'a>(s: *mut c_void) -> &'a mut World {
    &mut *(s as *mut World)
}
unsafe fn ent<'a>(s: *const c_void, h: EntityHandleV1) -> &'a Entity {
    &w(s).entities[h.id().unwrap()]
}
unsafe extern "C" fn tick_fn(s: *const c_void) -> usize {
    w(s).tick
}
unsafe extern "C" fn count(s: *const c_void) -> usize {
    w(s).entities.len()
}
unsafe extern "C" fn at(s: *const c_void, i: usize) -> EntityHandleV1 {
    w(s).entity_reads.set(w(s).entity_reads.get() + 1);
    if i < w(s).entities.len() {
        EntityHandleV1::from_id(i)
    } else {
        EntityHandleV1::NULL
    }
}
unsafe extern "C" fn valid(s: *const c_void, h: EntityHandleV1) -> bool {
    h.id().is_some_and(|i| i < w(s).entities.len())
}
unsafe extern "C" fn kind(s: *const c_void, h: EntityHandleV1) -> u32 {
    ent(s, h).kind.code()
}
unsafe extern "C" fn team(s: *const c_void, h: EntityHandleV1) -> usize {
    ent(s, h).team
}
unsafe extern "C" fn alive(s: *const c_void, h: EntityHandleV1) -> bool {
    ent(s, h).alive
}
unsafe extern "C" fn targetable(s: *const c_void, h: EntityHandleV1) -> bool {
    ent(s, h).targetable
}
unsafe extern "C" fn pos(s: *const c_void, h: EntityHandleV1, x: *mut u64, y: *mut u64) -> bool {
    *x = ent(s, h).pos.0;
    *y = ent(s, h).pos.1;
    true
}
unsafe extern "C" fn hp(s: *const c_void, h: EntityHandleV1, a: *mut usize, b: *mut usize) -> bool {
    *a = ent(s, h).hp;
    *b = ent(s, h).max;
    true
}
unsafe extern "C" fn stat(s: *const c_void, h: EntityHandleV1, out: *mut StatV1) -> bool {
    *out = ent(s, h).stat;
    true
}
unsafe fn text_out(text: &str, out: *mut u8, cap: usize, len: *mut usize) -> bool {
    *len = text.len();
    if !out.is_null() && cap >= text.len() {
        std::ptr::copy_nonoverlapping(text.as_ptr(), out, text.len());
    }
    true
}
unsafe extern "C" fn name(
    s: *const c_void,
    h: EntityHandleV1,
    out: *mut u8,
    cap: usize,
    len: *mut usize,
) -> bool {
    w(s).name_reads.set(w(s).name_reads.get() + 1);
    text_out(&ent(s, h).name, out, cap, len)
}
unsafe extern "C" fn buff_count(s: *const c_void, h: EntityHandleV1) -> usize {
    w(s).buff_reads.set(w(s).buff_reads.get() + 1);
    ent(s, h).buffs.len()
}
unsafe extern "C" fn buff_at(
    s: *const c_void,
    h: EntityHandleV1,
    i: usize,
    out: *mut BuffV1,
) -> bool {
    if let Some(b) = ent(s, h).buffs.get(i) {
        *out = *b;
        true
    } else {
        false
    }
}
unsafe extern "C" fn add_buff(s: *mut c_void, id: usize, b: *const BuffV1) {
    let world = wm(s);
    if !world.nexus_buffs && world.entities[id].kind == EntityKindV1::Nexus {
        return;
    }
    world.entities[id].buffs.push(*b);
}
unsafe extern "C" fn remove_buff(
    s: *mut c_void,
    h: EntityHandleV1,
    n: *const u8,
    l: usize,
) -> usize {
    let n = std::str::from_utf8(std::slice::from_raw_parts(n, l)).unwrap();
    let e = &mut wm(s).entities[h.id().unwrap()];
    let old = e.buffs.len();
    e.buffs.retain(|b| b.name() != n);
    old - e.buffs.len()
}
unsafe extern "C" fn cc_count(s: *const c_void, h: EntityHandleV1) -> usize {
    ent(s, h).cc.len()
}
unsafe extern "C" fn cc_at(s: *const c_void, h: EntityHandleV1, i: usize, out: *mut CcV1) -> bool {
    if let Some(c) = ent(s, h).cc.get(i) {
        *out = *c;
        true
    } else {
        false
    }
}
unsafe extern "C" fn cc(s: *mut c_void, id: usize, c: *const CcV1) {
    wm(s).entities[id].cc.push(*c);
}
unsafe extern "C" fn set_pos(s: *mut c_void, h: EntityHandleV1, x: u64, y: u64) -> bool {
    wm(s).entities[h.id().unwrap()].pos = (x, y);
    true
}
unsafe extern "C" fn banish(
    s: *mut c_void,
    _caster: usize,
    h: EntityHandleV1,
    _t: usize,
    _a: *const u8,
    _al: usize,
    _b: *const u8,
    _bl: usize,
) -> bool {
    wm(s).entities[h.id().unwrap()].targetable = false;
    true
}
unsafe extern "C" fn t_count(s: *const c_void) -> usize {
    w(s).tower_ids.len()
}
unsafe extern "C" fn t_id(s: *const c_void, i: usize) -> usize {
    w(s).tower_ids[i]
}
unsafe extern "C" fn p_count(s: *const c_void) -> usize {
    w(s).players.len()
}
unsafe extern "C" fn p_at(s: *const c_void, i: usize) -> PlayerHandleV1 {
    if i < w(s).players.len() {
        PlayerHandleV1::from_id(i)
    } else {
        PlayerHandleV1::NULL
    }
}
unsafe extern "C" fn p_valid(s: *const c_void, h: PlayerHandleV1) -> bool {
    h.id().is_some_and(|i| i < w(s).players.len())
}
unsafe extern "C" fn p_team(s: *const c_void, h: PlayerHandleV1) -> usize {
    w(s).players[h.id().unwrap()].team
}
unsafe extern "C" fn p_champ(s: *const c_void, h: PlayerHandleV1) -> EntityHandleV1 {
    w(s).players[h.id().unwrap()]
        .champion
        .map(EntityHandleV1::from_id)
        .unwrap_or(EntityHandleV1::NULL)
}
unsafe extern "C" fn p_stats(
    s: *const c_void,
    h: PlayerHandleV1,
    _path: *const u8,
    _pl: usize,
    out: *mut u8,
    cap: usize,
    len: *mut usize,
) -> bool {
    text_out(
        &w(s).players[h.id().unwrap()].score.to_string(),
        out,
        cap,
        len,
    )
}
unsafe extern "C" fn spawn_fn(
    s: *mut c_void,
    n: *const u8,
    l: usize,
    summoner: usize,
    team: usize,
    x: u64,
    y: u64,
    _ttl: u64,
    st: *const StatV1,
    _a: *const UnitAttackV1,
) -> usize {
    let name = std::str::from_utf8(std::slice::from_raw_parts(n, l)).unwrap();
    let world = wm(s);
    let id = world.entities.len();
    let mut e = entity(EntityKindV1::Summon, name, team, (x, y));
    e.stat = *st;
    e.hp = (*st).hp;
    e.max = (*st).hp;
    world.entities.push(e);
    world.spawns.push((name.into(), team, summoner));
    id
}
unsafe extern "C" fn damage_fn(
    s: *mut c_void,
    a: usize,
    t: usize,
    n: usize,
    _dt: u32,
    _at: u32,
) -> bool {
    let world = wm(s);
    world.damage.push((a, t, n));
    if world.reject_nexus_damage && world.entities[t].kind == EntityKindV1::Nexus {
        return false;
    }
    world.entities[t].hp = world.entities[t].hp.saturating_sub(n);
    world.entities[t].alive = world.entities[t].hp > 0;
    true
}
unsafe extern "C" fn view_effect(
    s: *mut c_void,
    n: *const u8,
    len: usize,
    caster: usize,
    input: *const InputTargetV1,
    _range: u64,
    _radius: u64,
    ticks: u64,
) -> bool {
    let name = String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into_owned();
    wm(s).effect_inputs.push(*input);
    wm(s)
        .effects
        .push((name, caster, (*input).x, (*input).y, ticks));
    true
}
fn run(world: &mut World, initial: bool) {
    unsafe {
        let mut v: SimVtableV1 = std::mem::zeroed();
        v.size = size_of::<SimVtableV1>();
        v.tick = Some(tick_fn);
        v.entity_count = Some(count);
        v.entity_at = Some(at);
        v.entity_is_valid = Some(valid);
        v.entity_kind = Some(kind);
        v.entity_team = Some(team);
        v.entity_is_alive = Some(alive);
        v.entity_is_targetable = Some(targetable);
        v.entity_pos = Some(pos);
        v.entity_hp = Some(hp);
        v.entity_stat = Some(stat);
        v.entity_name = Some(name);
        v.entity_buff_count = Some(buff_count);
        v.entity_buff_at = Some(buff_at);
        v.add_buff = Some(add_buff);
        v.entity_remove_buff = Some(remove_buff);
        v.entity_cc_count = Some(cc_count);
        v.entity_cc_at = Some(cc_at);
        v.apply_cc = Some(cc);
        v.entity_set_pos = Some(set_pos);
        v.entity_banish = Some(banish);
        v.tower_count = Some(t_count);
        v.tower_id_at = Some(t_id);
        v.player_count = Some(p_count);
        v.player_at = Some(p_at);
        v.player_is_valid = Some(p_valid);
        v.player_team = Some(p_team);
        v.player_champion = Some(p_champ);
        v.player_statistics_json = Some(p_stats);
        v.spawn_unit = Some(spawn_fn);
        v.deal_damage_typed = Some(damage_fn);
        v.play_view_effect = Some(view_effect);
        let mut raw = SimCtxV1 {
            size: size_of::<SimCtxV1>(),
            sim: &v,
            frame: std::ptr::null(),
            state: world as *mut _ as *mut c_void,
        };
        let mut sim = StableSim::from_raw(&mut raw).unwrap();
        if initial {
            start(&mut sim)
        } else {
            tick(&mut sim)
        }
    }
}
fn pushes(world: &World) -> usize {
    world.spawns.iter().filter(|p| p.0 == PUSH_UNIT).count()
}
fn kill_lord_for(world: &mut World, player: usize) {
    world.tick = 100;
    world.players[player].score += 1;
    run(world, false);
    world.tick = next_wave(100);
    run(world, false);
}
#[test]
fn red_secure_spawns_red_once_at_next_wave() {
    let mut w = fixture();
    run(&mut w, true);
    w.tick = 100;
    w.players[1].score = 1;
    run(&mut w, false);
    assert_eq!(pushes(&w), 0);
    w.tick = 669;
    run(&mut w, false);
    assert_eq!(pushes(&w), 0);
    w.tick = 670;
    run(&mut w, false);
    assert_eq!(pushes(&w), 1);
    let s = w.spawns.iter().find(|p| p.0 == PUSH_UNIT).unwrap();
    assert_eq!((s.1, s.2), (1, 3));
    run(&mut w, false);
    w.tick = 671;
    run(&mut w, false);
    assert_eq!(pushes(&w), 1);
}
#[test]
fn blue_secure_spawns_blue() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let s = w.spawns.iter().find(|p| p.0 == PUSH_UNIT).unwrap();
    assert_eq!((s.1, s.2), (0, 2));
}
#[test]
fn historical_score_does_not_spawn_when_initializing() {
    let mut w = fixture();
    w.players[0].score = 3;
    run(&mut w, true);
    w.tick = 670;
    run(&mut w, false);
    assert_eq!(pushes(&w), 0);
}
#[test]
fn hidden_controller_fallback_stays_unique_and_untargetable() {
    let mut w = fixture();
    w.nexus_buffs = false;
    run(&mut w, true);
    run(&mut w, true);
    let controllers: Vec<_> = w.entities.iter().filter(|e| e.name == CONTROLLER).collect();
    assert_eq!(controllers.len(), 1);
    assert!(!controllers[0].targetable);
    kill_lord_for(&mut w, 1);
    assert_eq!(pushes(&w), 1);
}
#[test]
fn controller_pending_state_does_not_leak_between_simulations() {
    let mut a = fixture();
    let mut b = fixture();
    run(&mut a, true);
    run(&mut b, true);
    kill_lord_for(&mut a, 1);
    b.tick = 670;
    run(&mut b, false);
    assert_eq!(pushes(&a), 1);
    assert_eq!(pushes(&b), 0);
}
#[test]
fn dead_secure_champion_still_gives_correct_team() {
    let mut w = fixture();
    run(&mut w, true);
    w.players[1].champion = None;
    kill_lord_for(&mut w, 1);
    let s = w.spawns.iter().find(|p| p.0 == PUSH_UNIT).unwrap();
    assert_eq!((s.1, s.2), (1, usize::MAX));
}
#[test]
fn lord_attacks_selected_lane_tower_and_respects_cooldown() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let lord = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    let tower = w.tower_ids[6];
    w.entities[lord].pos = (560000, 400000);
    w.tick += 1;
    run(&mut w, false);
    assert!(w.damage.iter().any(|&(a, t, _)| a == lord && t == tower));
    let n = w.damage.len();
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.damage.len(), n);
}
#[test]
fn dead_lord_stops_moving_and_attacking() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[id].alive = false;
    let pos = w.entities[id].pos;
    let n = w.damage.len();
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.entities[id].pos, pos);
    assert_eq!(w.damage.len(), n);
}
#[test]
fn control_movement_follows_blue_and_red_route() {
    for team in 0..2 {
        let mut w = fixture();
        run(&mut w, true);
        kill_lord_for(&mut w, team);
        let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
        let initial = w.entities[id].pos;
        w.tick += 1;
        run(&mut w, false);
        let end = w.entities[id].pos;
        if team == 0 {
            assert!(end.0 > initial.0 && end.1 < initial.1)
        } else {
            assert!(end.0 < initial.0 && end.1 > initial.1)
        }
    }
}
#[test]
fn lane_is_chosen_from_towers_still_alive_at_spawn() {
    let mut w = fixture();
    run(&mut w, true);
    w.tick = 100;
    w.players[0].score = 1;
    run(&mut w, false);
    for i in [2, 3] {
        let id = w.tower_ids[i];
        w.entities[id].alive = false;
        w.entities[id].hp = 0;
    }
    w.tick = 670;
    run(&mut w, false);
    let lord = w.entities.iter().find(|e| e.name == PUSH_UNIT).unwrap();
    assert!(lord.buffs.iter().any(|b| b.name() == "mlbb_lord_lane:0"));
}
#[test]
fn nexus_is_protected_until_selected_lane_towers_are_down() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[id].pos = RED;
    w.tick += 1;
    run(&mut w, false);
    assert!(!w.damage.iter().any(|d| d.1 == 1));
    for i in [6, 7] {
        let t = w.tower_ids[i];
        w.entities[t].alive = false;
        w.entities[t].hp = 0;
    }
    advance(&mut w, 60);
    run(&mut w, false);
    assert!(w.damage.iter().any(|d| d.0 == id && d.1 == 1));
}
#[test]
fn temporarily_untargetable_tower_blocks_route_progress() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    let t = w.tower_ids[6];
    w.entities[t].targetable = false;
    w.entities[id].pos = w.entities[t].pos;
    w.entities[id].buffs.retain(|b| !b.name().starts_with(STEP));
    w.entities[id].buffs.push(BuffV1::named("mlbb_lord_step:3"));
    let pos = w.entities[id].pos;
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.entities[id].pos, pos);
    assert!(w.entities[id]
        .buffs
        .iter()
        .any(|b| b.name() == "mlbb_lord_step:3"));
}
#[test]
fn a_second_actual_secure_creates_one_more_lord() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    assert_eq!(pushes(&w), 1);
    w.tick = 1000;
    w.players[1].score = 1;
    run(&mut w, false);
    w.tick = 1330;
    run(&mut w, false);
    assert_eq!(pushes(&w), 2);
    run(&mut w, false);
    assert_eq!(pushes(&w), 2);
}
#[test]
fn stun_stops_custom_movement() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[id].cc.push(CcV1::of_kind(CcKindV1::Stun, 60));
    let pos = w.entities[id].pos;
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.entities[id].pos, pos);
}
#[test]
fn uploaded_log_regression_red_player6_then_blue_player3() {
    let mut w = fixture();
    w.players = (0..10)
        .map(|i| Player {
            team: if i < 5 { 0 } else { 1 },
            score: 0,
            champion: Some(if i < 5 { 2 } else { 3 }),
        })
        .collect();
    run(&mut w, true);
    w.tick = 20316;
    w.players[6].score = 1;
    run(&mut w, false);
    w.tick = 20470;
    run(&mut w, false);
    assert_eq!(
        w.spawns
            .iter()
            .filter(|s| s.0 == PUSH_UNIT)
            .map(|s| s.1)
            .collect::<Vec<_>>(),
        vec![1]
    );
    w.tick = 29560;
    w.players[3].score = 1;
    run(&mut w, false);
    w.tick = 29710;
    run(&mut w, false);
    assert_eq!(
        w.spawns
            .iter()
            .filter(|s| s.0 == PUSH_UNIT)
            .map(|s| s.1)
            .collect::<Vec<_>>(),
        vec![1, 0]
    );
}

#[test]
fn every_custom_attack_requests_the_existing_attack_animation_once() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[id].pos = (560000, 400000);
    w.tick += 1;
    run(&mut w, false);
    let cues: Vec<_> = w.entities[id]
        .cc
        .iter()
        .filter(|c| c.kind == CcKindV1::Animation.code())
        .collect();
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].name(), "attack");
    assert_eq!(cues[0].tick, 35);
    assert!(w.damage.iter().any(|d| d.0 == id));
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(
        w.entities[id]
            .cc
            .iter()
            .filter(|c| c.kind == CcKindV1::Animation.code())
            .count(),
        1
    );
}

#[test]
fn finishes_swing_before_gliding_after_a_lethal_hit() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    let tower = w.tower_ids[6];
    w.entities[tower].hp = 1;
    w.entities[id].pos = (560000, 400000);
    w.tick += 1;
    run(&mut w, false);
    assert!(!w.entities[tower].alive);
    let held = w.entities[id].pos;
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.entities[id].pos, held);
    // Simulate the host expiring the animation at the end of the swing.
    advance(&mut w, 35);
    run(&mut w, false);
    assert_ne!(w.entities[id].pos, held);
}

// Model the engine decrementing timed buffs and CC between callback samples.
fn advance(w: &mut World, ticks: usize) {
    w.tick += ticks;
    for e in &mut w.entities {
        for c in &mut e.cc {
            c.tick = c.tick.saturating_sub(ticks as u64);
        }
        e.cc.retain(|c| c.tick > 0);
        for b in &mut e.buffs {
            if b.duration_kind == mod_api_stable::BuffDurationV1::Time.code() {
                b.duration_tick = b.duration_tick.saturating_sub(ticks);
            }
        }
        e.buffs.retain(|b| {
            b.duration_kind != mod_api_stable::BuffDurationV1::Time.code() || b.duration_tick > 0
        });
    }
}

#[test]
fn new_swing_starts_only_after_existing_one_second_cooldown() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[id].pos = (560000, 400000);
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.damage.len(), 1);
    advance(&mut w, 35);
    run(&mut w, false);
    assert_eq!(w.damage.len(), 1);
    assert!(!w.entities[id]
        .cc
        .iter()
        .any(|c| c.kind == CcKindV1::Animation.code()));
    advance(&mut w, 25);
    run(&mut w, false);
    assert_eq!(w.damage.len(), 2);
    let cue = w.entities[id]
        .cc
        .iter()
        .find(|c| c.kind == CcKindV1::Animation.code())
        .unwrap();
    assert_eq!((cue.name(), cue.tick), ("attack", 35));
}

#[test]
fn tower_order_is_outer_then_inner_for_both_teams() {
    let mut all = map_towers();
    assert_eq!(
        next_lane_tower(BLUE, RED, &all, 1, Lane::Mid).unwrap().id,
        all[6].id
    );
    assert_eq!(
        next_lane_tower(BLUE, RED, &all, 0, Lane::Mid).unwrap().id,
        all[5].id
    );
    all[6].alive = false;
    all[5].alive = false;
    assert_eq!(
        next_lane_tower(BLUE, RED, &all, 1, Lane::Mid).unwrap().id,
        all[7].id
    );
    assert_eq!(
        next_lane_tower(BLUE, RED, &all, 0, Lane::Mid).unwrap().id,
        all[4].id
    );
}

#[test]
fn live_inner_tower_missing_from_index_still_blocks_nexus() {
    let mut w = fixture();
    let outer = w.tower_ids[6];
    let inner = w.tower_ids[7];
    // Simulate an incomplete/stale indexed tower helper.
    w.tower_ids.retain(|id| *id != inner);
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    w.entities[outer].alive = false;
    w.entities[outer].hp = 0;
    w.entities[3].targetable = false;
    let lord = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[lord].pos = RED;
    w.tick += 1;
    run(&mut w, false);
    assert!(!w.damage.iter().any(|d| d.1 == 1));
    assert_ne!(w.entities[lord].pos, RED);
    assert!(w.entities[lord]
        .buffs
        .iter()
        .any(|b| b.name() == format!("mlbb_lord_goal:{inner}")));
}

#[test]
fn reaching_inner_first_does_not_bypass_living_outer_tower() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let lord = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    let outer = w.tower_ids[6];
    let inner = w.tower_ids[7];
    let pos = w.entities[inner].pos;
    w.entities[lord].pos = pos;
    w.tick += 1;
    run(&mut w, false);
    assert!(!w.damage.iter().any(|d| d.1 == inner));
    assert_ne!(w.entities[lord].pos, pos);
    w.entities[outer].alive = false;
    w.entities[outer].hp = 0;
    w.tick += 1;
    run(&mut w, false);
    assert!(w.damage.iter().any(|d| d.0 == lord && d.1 == inner));
}

#[test]
fn stale_route_index_cannot_advance_past_untargetable_inner() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let outer = w.tower_ids[6];
    let inner = w.tower_ids[7];
    w.entities[outer].alive = false;
    w.entities[outer].hp = 0;
    w.entities[inner].targetable = false;
    w.entities[3].targetable = false;
    let lord = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[lord].pos = RED;
    w.entities[lord]
        .buffs
        .retain(|b| !b.name().starts_with(STEP));
    w.entities[lord]
        .buffs
        .push(BuffV1::named("mlbb_lord_step:999"));
    w.tick += 1;
    run(&mut w, false);
    assert_ne!(w.entities[lord].pos, RED);
    assert!(!w.damage.iter().any(|d| d.1 == 1));
}

#[test]
fn two_other_lane_inner_towers_do_not_block_a_cleared_mid_lane() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let keep = [w.tower_ids[3], w.tower_ids[11]];
    for id in w.tower_ids.clone() {
        if w.entities[id].team == 1 && !keep.contains(&id) {
            w.entities[id].alive = false;
            w.entities[id].hp = 0;
        }
    }
    let lord = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[lord].pos = RED;
    w.tick += 1;
    run(&mut w, false);
    assert!(w.damage.iter().any(|d| d.0 == lord && d.1 == 1));
    assert!(keep.iter().all(|id| w.entities[*id].alive));
}

#[test]
fn route_recovers_after_the_required_inner_tower_is_destroyed() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let outer = w.tower_ids[6];
    let inner = w.tower_ids[7];
    w.entities[outer].alive = false;
    w.entities[outer].hp = 0;
    w.entities[inner].targetable = false;
    let lord = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    w.entities[lord].pos = w.entities[inner].pos;
    w.entities[lord]
        .buffs
        .retain(|b| !b.name().starts_with(STEP));
    w.entities[lord]
        .buffs
        .push(BuffV1::named("mlbb_lord_step:999"));
    let held = w.entities[lord].pos;
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.entities[lord].pos, held);
    w.entities[inner].alive = false;
    w.entities[inner].hp = 0;
    w.tick += 1;
    run(&mut w, false);
    assert_ne!(w.entities[lord].pos, held);
    assert!(dist_sq(w.entities[lord].pos, RED) < dist_sq(held, RED));
}

fn add_core_guards(w: &mut World, team: usize) -> [usize; 2] {
    let points = if team == 1 {
        [(800000, 64000), (896000, 160000)]
    } else {
        [(64000, 800000), (160000, 896000)]
    };
    let mut ids = [0, 0];
    for (i, pos) in points.into_iter().enumerate() {
        ids[i] = w.entities.len();
        w.tower_ids.push(ids[i]);
        w.entities
            .push(entity(EntityKindV1::Tower, "base_guard", team, pos));
    }
    ids
}
fn clear_mid_lane(w: &mut World, enemy: usize) {
    for id in w.tower_ids.clone() {
        if w.entities[id].team == enemy && lane_of(BLUE, RED, w.entities[id].pos) == Lane::Mid {
            w.entities[id].alive = false;
            w.entities[id].hp = 0;
        }
    }
}
fn own_lord(w: &World) -> usize {
    w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap()
}
fn marked_guards(w: &World, lord: usize) -> Vec<usize> {
    w.entities[lord]
        .buffs
        .iter()
        .filter_map(|b| b.name().strip_prefix(BASE_GUARD)?.parse().ok())
        .collect()
}

#[test]
fn protected_core_registers_both_off_lane_base_guards() {
    for team in 0..2 {
        let mut w = fixture();
        let enemy = 1 - team;
        let guard = add_core_guards(&mut w, enemy);
        run(&mut w, true);
        kill_lord_for(&mut w, team);
        clear_mid_lane(&mut w, enemy);
        w.entities[enemy].targetable = false;
        let lord = own_lord(&w);
        w.entities[lord].pos = if enemy == 1 { RED } else { BLUE };
        w.entities[2 + enemy].targetable = false;
        let before = w.entities[lord].pos;
        w.tick += 1;
        run(&mut w, false);
        assert!(guard.iter().all(|id| marked_guards(&w, lord).contains(id)));
        assert!(!w.damage.iter().any(|d| d.1 == enemy));
        assert_ne!(w.entities[lord].pos, before);
    }
}

#[test]
fn both_base_guards_die_before_core_even_if_targetable_turns_true_early() {
    let mut w = fixture();
    let guard = add_core_guards(&mut w, 1);
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    clear_mid_lane(&mut w, 1);
    w.entities[1].targetable = false;
    let lord = own_lord(&w);
    w.entities[lord].pos = RED;
    w.entities[3].targetable = false;
    w.tick += 1;
    run(&mut w, false);
    w.entities[guard[0]].alive = false;
    w.entities[guard[0]].hp = 0;
    w.entities[1].targetable = true;
    w.entities[lord].pos = RED;
    w.tick += 1;
    run(&mut w, false);
    assert!(!w.damage.iter().any(|d| d.1 == 1));
    assert!(w.entities[lord]
        .buffs
        .iter()
        .any(|b| b.name() == format!("mlbb_lord_goal:{}", guard[1])));
    w.entities[guard[1]].alive = false;
    w.entities[guard[1]].hp = 0;
    w.entities[lord].pos = RED;
    w.tick += 1;
    run(&mut w, false);
    assert!(w.damage.iter().any(|d| d.0 == lord && d.1 == 1));
}

#[test]
fn base_guard_is_actually_attackable_outside_the_selected_lane() {
    let mut w = fixture();
    let guard = add_core_guards(&mut w, 1);
    w.entities[3].targetable = false; // Isolate structure targeting from defending heroes.
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    clear_mid_lane(&mut w, 1);
    w.entities[1].targetable = false;
    let lord = own_lord(&w);
    w.entities[lord].pos = RED;
    w.tick += 1;
    run(&mut w, false);
    w.entities[lord].pos = w.entities[guard[0]].pos;
    w.tick += 1;
    run(&mut w, false);
    assert!(w.damage.iter().any(|d| d.0 == lord && d.1 == guard[0]));
    assert!(!w.damage.iter().any(|d| d.1 == 1));
}

#[test]
fn rejected_core_damage_triggers_guard_phase_even_with_targetable_flag_true() {
    let mut w = fixture();
    let guard = add_core_guards(&mut w, 1);
    w.entities[3].targetable = false; // Isolate structure targeting from defending heroes.
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    clear_mid_lane(&mut w, 1);
    w.reject_nexus_damage = true;
    let lord = own_lord(&w);
    w.entities[lord].pos = RED;
    let hp = w.entities[1].hp;
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.entities[1].hp, hp);
    assert!(guard.iter().all(|id| marked_guards(&w, lord).contains(id)));
    advance(&mut w, 60);
    run(&mut w, false);
    assert_ne!(w.entities[lord].pos, RED);
    assert_eq!(w.damage.iter().filter(|d| d.1 == 1).count(), 1);
}

#[test]
fn surviving_single_close_guard_does_not_enroll_distant_lane_tower() {
    let mut w = fixture();
    let guard = add_core_guards(&mut w, 1);
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    clear_mid_lane(&mut w, 1);
    w.entities[guard[0]].alive = false;
    w.entities[guard[0]].hp = 0;
    w.entities[1].targetable = false;
    let lord = own_lord(&w);
    w.entities[lord].pos = RED;
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(marked_guards(&w, lord), vec![guard[1]]);
}

#[test]
fn protected_nexus_without_visible_towers_does_not_get_rushed() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    for id in w.tower_ids.clone() {
        if w.entities[id].team == 1 {
            w.entities[id].alive = false;
            w.entities[id].hp = 0;
        }
    }
    w.entities[1].targetable = false;
    let lord = own_lord(&w);
    let pos = (710000, 250000);
    w.entities[lord].pos = pos;
    w.tick += 1;
    run(&mut w, false);
    assert_eq!(w.entities[lord].pos, pos);
    assert!(!w.damage.iter().any(|d| d.1 == 1));
}

#[test]
fn body_swing_frames_cover_all_35_ticks_then_return_to_glide() {
    for age in 0..35 {
        assert_eq!(
            body_frame(100 + age, Some(100)),
            (
                "attack",
                if age < 9 {
                    0
                } else if age < 21 {
                    1
                } else {
                    2
                }
            )
        );
    }
    assert_eq!(body_frame(135, Some(100)).0, "glide");
    assert_eq!(body_frame(99, Some(100)).0, "glide");
    assert_eq!(body_frame(usize::MAX, None).0, "glide");
}
#[test]
fn body_frame_events_show_swing_after_lethal_hit_without_hiding_combat_entity() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 0);
    let id = w.entities.iter().position(|e| e.name == PUSH_UNIT).unwrap();
    let tower = w.tower_ids[6];
    w.entities[tower].hp = 1;
    w.entities[id].pos = (560000, 400000);
    w.tick += 1;
    run(&mut w, false);
    advance(&mut w, 1); // Next presentation boundary; attack already dealt damage.
    run(&mut w, false);
    let e = w
        .effects
        .iter()
        .rev()
        .find(|e| e.1 == id && e.0.contains(":body_"))
        .unwrap();
    assert!(e.0.contains("body_attack_0_right"));
    assert_eq!(e.4, 3);
    assert!(w.entities[id].targetable);
    assert!(!w.entities[tower].alive);
    for (ticks, frame) in [(9, 1), (12, 2)] {
        advance(&mut w, ticks);
        run(&mut w, false);
        let e = w
            .effects
            .iter()
            .rev()
            .find(|e| e.0.contains(":body_"))
            .unwrap();
        assert!(e.0.contains(&format!("body_attack_{frame}_right")));
    }
    advance(&mut w, 15);
    run(&mut w, false);
    assert!(w.effects.last().unwrap().0.contains("body_glide_"));
}
#[test]
fn body_frames_use_twenty_followed_events_per_second_and_stop_when_dead() {
    let mut w = fixture();
    run(&mut w, true);
    kill_lord_for(&mut w, 1);
    let id = own_lord(&w);
    // Even an off-cadence spawn gets an immediate, shortened body frame.
    let initial = w.effects.iter().find(|e| e.0.contains(":body_")).unwrap();
    assert_eq!(initial.4, 2); // Spawn at tick 670, next boundary 672.
    let first = w.effect_inputs[w
        .effects
        .iter()
        .position(|e| e.0.contains(":body_"))
        .unwrap()];
    assert_eq!(first, InputTargetV1::target(id));
    w.effects.clear();
    w.effect_inputs.clear();
    for _ in 0..60 {
        advance(&mut w, 1);
        run(&mut w, false);
    }
    let events: Vec<_> = w
        .effects
        .iter()
        .enumerate()
        .filter(|(_, e)| e.0.contains(":body_"))
        .collect();
    assert_eq!(events.len(), 20);
    for (i, e) in events {
        assert_eq!(e.4, 3);
        assert!(e.0.ends_with("left_h3"));
        assert_eq!(w.effect_inputs[i], InputTargetV1::target(id));
    }
    assert!(w.entities[id].targetable);
    w.entities[id].alive = false;
    w.effects.clear();
    for _ in 0..3 {
        advance(&mut w, 1);
        run(&mut w, false);
    }
    assert!(!w.effects.iter().any(|e| e.0.contains(":body_")));
}
#[test]
fn partial_spawn_frame_and_cadence_have_no_overlapping_lifetimes() {
    for start in 0..3 {
        let mut next = start;
        let mut events = 0;
        for tick in start..start + 60 {
            if let Some(hold) = body_hold(tick, tick == start) {
                assert_eq!(tick, next);
                next = tick + hold as usize;
                events += 1;
            }
        }
        assert!(events == 20 || events == 21);
        assert!(next >= start + 60);
    }
}

#[test]
fn performance_workload_30_seconds() {
    for player in 0..2 {
        let mut w = fixture();
        for _ in 0..100 {
            w.entities
                .push(entity(EntityKindV1::Minion, "minion", 1 - player, (0, 0)));
        }
        run(&mut w, true);
        kill_lord_for(&mut w, player);
        let id = own_lord(&w);
        w.effects.clear();
        w.entity_reads.set(0);
        w.name_reads.set(0);
        w.buff_reads.set(0);
        for _ in 0..1800 {
            advance(&mut w, 1);
            run(&mut w, false);
        }
        assert_eq!(
            w.effects.iter().filter(|e| e.0.contains(":body_")).count(),
            600
        );
        assert!(w.name_reads.get() < 10000);
        assert_eq!(w.damage.len(), 19);
        println!("PERF player={} body_events={} entity_reads={} name_reads={} buff_reads={} pos={:?} damage={:?} tower_hp={:?}",player,
            w.effects.iter().filter(|e| e.0.contains(":body_")).count(),w.entity_reads.get(),w.name_reads.get(),w.buff_reads.get(),
            w.entities[id].pos,w.damage,w.tower_ids.iter().map(|i|w.entities[*i].hp).collect::<Vec<_>>());
    }
}
