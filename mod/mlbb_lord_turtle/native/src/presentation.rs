//! Presentation-only state: watched simulations/replays only. No gameplay writes.
use crate::probe::write_log;
use mod_api_stable::*;
use std::collections::VecDeque;
use std::sync::Mutex;
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
#[derive(Clone)]
struct Lord {
    team: usize,
    pos: (u64, u64),
}
#[derive(Clone)]
struct Totals {
    id: usize,
    team: usize,
    secure: usize,
    gold: usize,
    exp: usize,
}
#[derive(Clone)]
struct Snapshot {
    key: (u32, u64, u64, u64, u64),
    tick: usize,
    track_until: usize,
    lords: Vec<Lord>,
    players: Vec<Totals>,
}
static VIEW: Mutex<Option<Snapshot>> = Mutex::new(None);
fn watched(kind: u32) -> bool {
    matches!(
        SimOriginKindV1::from_code(kind),
        Some(
            SimOriginKindV1::ClientMatchView
                | SimOriginKindV1::ClientSpectate
                | SimOriginKindV1::ClientReplay
        )
    )
}
fn events<'a>(before: &[Totals], after: &'a [Totals]) -> Vec<&'a Totals> {
    after
        .iter()
        .filter(|p| {
            before
                .iter()
                .any(|old| old.id == p.id && old.team == p.team && p.secure > old.secure)
        })
        .collect()
}
pub fn capture(sim: &StableSim<'_>, reset: bool) {
    let Some(origin) = sim.sim_origin() else {
        return;
    };
    if !watched(origin.kind) {
        return;
    }
    let key = (
        origin.kind,
        origin.match_id,
        origin.replay_id,
        origin.set_index,
        sim.seed(),
    );
    let tick = sim.tick();
    let old = VIEW
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .filter(|p| !reset && p.key == key && tick >= p.tick);
    let mut lords = Vec::new();
    let mut turtle_active = false;
    if !sim.is_end() {
        for i in 0..sim.entity_count() {
            let Some(e) = sim.entity_at(i) else { continue };
            if e.kind() == Some(EntityKindV1::Summon)
                && e.is_alive()
                && e.name().as_deref() == Some("mlbb_lord_push")
                && e.team() < 2
            {
                lords.push(Lord {
                    team: e.team(),
                    pos: e.pos(),
                });
            }
            if e.kind() == Some(EntityKindV1::Serpen) && e.is_alive() {
                let (h, m) = e.hp();
                turtle_active = h > 0 && h < m;
            }
        }
    }
    let track_until = if turtle_active {
        tick.saturating_add(120)
    } else {
        old.as_ref().map_or(0, |p| p.track_until)
    };
    let mut players = Vec::new();
    if tick <= track_until {
        for i in 0..sim.player_count() {
            let Some(p) = sim.player_at(i) else { continue };
            let read = |field: &str| {
                p.statistics_json(field)
                    .and_then(|s| s.trim().parse::<usize>().ok())
            };
            if let (Some(secure), Some(gold), Some(exp)) =
                (read("serpen_secure"), read("gold"), read("exp"))
            {
                players.push(Totals {
                    id: p.id(),
                    team: p.team(),
                    secure,
                    gold,
                    exp,
                });
            }
        }
    }
    if let Some(prev) = &old {
        for owner in events(&prev.players, &players) {
            write_log(&format!("TURTLE_NATIVE_REWARD tick={tick} owner={} team={} counter={} (native rules; other same-tick income may be included)\n",owner.id,owner.team,owner.secure));
            for p in &players {
                if let Some(b) = prev.players.iter().find(|b| b.id == p.id) {
                    write_log(&format!(
                        "TURTLE_NATIVE_PLAYER id={} team={} gold_delta={} exp_delta={}\n",
                        p.id,
                        p.team,
                        p.gold.saturating_sub(b.gold),
                        p.exp.saturating_sub(b.exp)
                    ));
                }
            }
        }
    }
    *VIEW.lock().unwrap_or_else(|e| e.into_inner()) = Some(Snapshot {
        key,
        tick,
        track_until,
        lords,
        players,
    });
}
fn project(r: Rect, pos: (u64, u64)) -> (f32, f32) {
    (
        r.x + (pos.0 as f64 / 960000.0).clamp(0., 1.) as f32 * r.w,
        r.y + (pos.1 as f64 / 960000.0).clamp(0., 1.) as f32 * r.h,
    )
}
fn valid_rect(r: Rect) -> bool {
    r.x.is_finite()
        && r.y.is_finite()
        && r.w.is_finite()
        && r.h.is_finite()
        && r.w >= 80.
        && r.h >= 80.
        && r.w <= 800.
        && r.h <= 800.
        && (r.w - r.h).abs() <= r.w * 0.2
}
fn reference_rect(size: (f32, f32)) -> Option<Rect> {
    ((size.0 - 1920.).abs() < 1. && (size.1 - 1080.).abs() < 1.).then_some(Rect {
        x: 1014.,
        y: 677.,
        w: 320.,
        h: 320.,
    })
}
#[derive(Default)]
struct Cache {
    path: Option<String>,
    frames: usize,
    reported: bool,
    queue: VecDeque<(String, usize)>,
    seen: usize,
}
#[derive(Default)]
pub struct MinimapOverlay {
    cache: Mutex<Cache>,
}
impl StableExtension for MinimapOverlay {
    fn post_render(&self, ctx: &mut StableClient<'_>) {
        if !ctx.can_draw() || ctx.draw_map_size("Game").is_none() {
            return;
        }
        let active = matches!(
            ctx.client_scene_kind(),
            Some(
                ClientSceneKindV1::Match
                    | ClientSceneKindV1::InGame
                    | ClientSceneKindV1::Tutorial5v5
                    | ClientSceneKindV1::TutorialMorgad
                    | ClientSceneKindV1::TutorialSerpen
            )
        ) || ctx.scene_kind() == Some(SceneKindV1::GameTest);
        if !active {
            *self.cache.lock().unwrap_or_else(|e| e.into_inner()) = Cache::default();
            return;
        }
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.frames += 1;
        if cache.path.as_ref().is_some_and(|p| !ctx.ui_exists(p)) {
            cache.path = None;
            cache.reported = false;
        }
        if cache.path.is_none() && (cache.frames == 1 || cache.frames % 120 == 0) {
            cache.queue = VecDeque::from([(String::new(), 0usize)]);
            cache.seen = 0;
        }
        // Discover before Lord spawns and amortize the bounded UI traversal.
        if cache.path.is_none() {
            for _ in 0..32 {
                let Some((path, depth)) = cache.queue.pop_front() else {
                    break;
                };
                cache.seen += 1;
                if cache.seen > 512 {
                    cache.queue.clear();
                    break;
                }
                let name = format!("{} {}", path, ctx.ui_runner_name(&path).unwrap_or_default())
                    .to_lowercase();
                if name.contains("minimap") || name.contains("mini_map") {
                    if let Some((x, y, w, h)) = ctx
                        .ui_contents_rect(&path)
                        .or_else(|| ctx.ui_node_rect(&path))
                    {
                        let r = Rect { x, y, w, h };
                        if valid_rect(r) {
                            cache.path = Some(path.clone());
                            cache.queue.clear();
                            break;
                        }
                    }
                }
                if depth < 8 {
                    for child in ctx.ui_child_names(&path) {
                        if cache.seen + cache.queue.len() >= 512 {
                            break;
                        }
                        cache.queue.push_back((
                            if path.is_empty() {
                                child
                            } else {
                                format!("{path}.{child}")
                            },
                            depth + 1,
                        ));
                    }
                }
            }
        }
        let Some(snap) = VIEW.lock().unwrap_or_else(|e| e.into_inner()).clone() else {
            return;
        };
        if snap.lords.is_empty() {
            return;
        }
        let found = cache.path.as_ref().and_then(|p| {
            if ctx.ui_visible(p) == Some(false) {
                return None;
            }
            ctx.ui_contents_rect(p)
                .or_else(|| ctx.ui_node_rect(p))
                .map(|(x, y, w, h)| Rect { x, y, w, h })
                .filter(|r| valid_rect(*r))
        });
        let fallback = ctx.draw_map_size("UI").and_then(reference_rect);
        let Some(rect) = found.or(fallback) else {
            if !cache.reported {
                write_log("MINIMAP_OVERLAY no usable minimap anchor; supported screenshot fallback is 1920x1080 UI space\n");
                cache.reported = true;
            }
            return;
        };
        if !cache.reported {
            write_log(&format!(
                "MINIMAP_OVERLAY anchor={} rect={rect:?} tick={}\n",
                cache
                    .path
                    .as_deref()
                    .unwrap_or("reference_1920x1080_screenshot"),
                snap.tick
            ));
            cache.reported = true;
        }
        for lord in snap.lords {
            let (x, y) = project(rect, lord.pos);
            ctx.draw_circle(
                "UI",
                x,
                y,
                8.,
                10000,
                if lord.team == 0 {
                    0x5b85ffff
                } else {
                    0xff6677ff
                },
            );
            let asset = if lord.team == 0 {
                "asset/mlbb_lord_turtle/sprites/lord_minimap_blue"
            } else {
                "asset/mlbb_lord_turtle/sprites/lord_minimap_red"
            };
            let params = StableSpriteParams {
                x: x - 7.,
                y: y - 7.,
                z: 10001,
                ..Default::default()
            };
            if !ctx.draw_sprite("UI", asset, &params) {
                ctx.draw_text(
                    "UI",
                    "L",
                    "asset/base/font/set/bold",
                    (x - 7., y - 7., 14., 14.),
                    10002,
                    12.,
                    0xffffffff,
                    TextAlignXV1::Center,
                    TextAlignYV1::Center,
                );
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excludes_presims_and_unknown_origins() {
        assert!(!watched(0));
        assert!(!watched(1));
        assert!(!watched(5));
        for k in [2, 3, 4] {
            assert!(watched(k));
        }
    }
    #[test]
    fn projects_world_positions_and_clamps() {
        let r = Rect {
            x: 1000.,
            y: 500.,
            w: 320.,
            h: 320.,
        };
        assert_eq!(project(r, (96000, 864000)), (1032., 788.));
        assert_eq!(project(r, (864000, 96000)), (1288., 532.));
        assert_eq!(project(r, (u64::MAX, 0)), (1320., 500.));
    }
    #[test]
    fn fallback_only_for_documented_ui_canvas() {
        assert!(reference_rect((1920., 1080.)).is_some());
        assert!(reference_rect((1280., 720.)).is_none());
        assert!(reference_rect((2560., 1440.)).is_none());
    }
    #[test]
    fn rejects_non_map_panels() {
        assert!(valid_rect(Rect {
            x: 10.,
            y: 20.,
            w: 320.,
            h: 320.
        }));
        assert!(!valid_rect(Rect {
            x: 0.,
            y: 0.,
            w: 900.,
            h: 300.
        }));
        assert!(!valid_rect(Rect {
            x: f32::NAN,
            y: 0.,
            w: 320.,
            h: 320.
        }));
    }
    fn t(id: usize, team: usize, secure: usize) -> Totals {
        Totals {
            id,
            team,
            secure,
            gold: 0,
            exp: 0,
        }
    }
    #[test]
    fn detects_both_teams_reward_events_once() {
        let before = vec![t(0, 0, 0), t(6, 1, 0)];
        let after = vec![t(0, 0, 0), t(6, 1, 1)];
        assert_eq!(events(&before, &after)[0].id, 6);
        assert!(events(&after, &after).is_empty());
        let next = vec![t(0, 0, 1), t(6, 1, 1)];
        assert_eq!(events(&after, &next)[0].id, 0);
    }
    #[test]
    fn counter_reset_and_missing_players_are_not_new_kills() {
        assert!(events(&[t(0, 0, 3)], &[t(0, 0, 0)]).is_empty());
        assert!(events(&[], &[t(0, 0, 1)]).is_empty());
    }
}
