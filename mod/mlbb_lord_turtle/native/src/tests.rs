use super::*;
use mod_api_stable::{EntityHandleV1, SimCtxV1, SimVtableV1};
use std::{ffi::c_void, mem::size_of};

#[derive(Clone)]
struct Entity {
    kind: EntityKindV1,
    hp: usize,
    max_hp: usize,
    pos: (u64, u64),
    alive: bool,
    targetable: bool,
}
#[derive(Default, Debug, PartialEq)]
struct Calls {
    cc: Vec<(usize, u32, u64)>,
    damage: Vec<(usize, usize, usize, u32, u32)>,
    effects: Vec<(String, usize, u32, u64, u64)>,
}
struct World {
    tick: usize,
    end: bool,
    entities: Vec<Entity>,
    calls: Calls,
}
unsafe fn world<'a>(s: *const c_void) -> &'a World {
    &*(s as *const World)
}
unsafe fn world_mut<'a>(s: *mut c_void) -> &'a mut World {
    &mut *(s as *mut World)
}
unsafe extern "C" fn tick(s: *const c_void) -> usize {
    world(s).tick
}
unsafe extern "C" fn end(s: *const c_void) -> bool {
    world(s).end
}
unsafe extern "C" fn count(s: *const c_void) -> usize {
    world(s).entities.len()
}
unsafe extern "C" fn at(s: *const c_void, i: usize) -> EntityHandleV1 {
    if i < world(s).entities.len() {
        EntityHandleV1::from_id(i)
    } else {
        EntityHandleV1::NULL
    }
}
unsafe extern "C" fn valid(s: *const c_void, h: EntityHandleV1) -> bool {
    h.id().is_some_and(|i| i < world(s).entities.len())
}
unsafe fn ent<'a>(s: *const c_void, h: EntityHandleV1) -> &'a Entity {
    &world(s).entities[h.id().unwrap()]
}
unsafe extern "C" fn kind(s: *const c_void, h: EntityHandleV1) -> u32 {
    ent(s, h).kind.code()
}
unsafe extern "C" fn alive(s: *const c_void, h: EntityHandleV1) -> bool {
    ent(s, h).alive
}
unsafe extern "C" fn targetable(s: *const c_void, h: EntityHandleV1) -> bool {
    ent(s, h).targetable
}
unsafe extern "C" fn hp(
    s: *const c_void,
    h: EntityHandleV1,
    current: *mut usize,
    max: *mut usize,
) -> bool {
    let e = ent(s, h);
    *current = e.hp;
    *max = e.max_hp;
    true
}
unsafe extern "C" fn pos(s: *const c_void, h: EntityHandleV1, x: *mut u64, y: *mut u64) -> bool {
    let e = ent(s, h);
    *x = e.pos.0;
    *y = e.pos.1;
    true
}
unsafe extern "C" fn cc(s: *mut c_void, id: usize, c: *const CcV1) {
    world_mut(s).calls.cc.push((id, (*c).kind, (*c).tick));
}
unsafe extern "C" fn damage(s: *mut c_void, a: usize, t: usize, n: usize, d: u32, k: u32) -> bool {
    let w = world_mut(s);
    w.calls.damage.push((a, t, n, d, k));
    w.entities[t].hp = w.entities[t].hp.saturating_sub(n);
    if w.entities[t].hp == 0 {
        w.entities[t].alive = false;
    }
    true
}
unsafe extern "C" fn effect(
    s: *mut c_void,
    n: *const u8,
    len: usize,
    c: usize,
    i: *const InputTargetV1,
    _r: u64,
    _a: u64,
    _t: u64,
) -> bool {
    let name = String::from_utf8_lossy(std::slice::from_raw_parts(n, len)).into_owned();
    world_mut(s)
        .calls
        .effects
        .push((name, c, (*i).kind, (*i).x, (*i).y));
    true
}
fn run(w: &mut World) {
    // ABI table contains only integer size and nullable function pointers;
    // zero is a valid missing-slot value for all those fields.
    let mut v: SimVtableV1 = unsafe { std::mem::zeroed() };
    v.size = size_of::<SimVtableV1>();
    v.tick = Some(tick);
    v.is_end = Some(end);
    v.entity_count = Some(count);
    v.entity_at = Some(at);
    v.entity_is_valid = Some(valid);
    v.entity_kind = Some(kind);
    v.entity_is_alive = Some(alive);
    v.entity_is_targetable = Some(targetable);
    v.entity_hp = Some(hp);
    v.entity_pos = Some(pos);
    v.apply_cc = Some(cc);
    v.deal_damage_typed = Some(damage);
    v.play_view_effect = Some(effect);
    let mut raw = SimCtxV1 {
        size: size_of::<SimCtxV1>(),
        sim: &v,
        frame: std::ptr::null(),
        state: w as *mut _ as *mut c_void,
    };
    let mut sim = unsafe { StableSim::from_raw(&mut raw).unwrap() };
    LordSlam.on_match_tick(&mut sim, 123);
}
fn entity(kind: EntityKindV1, pos: (u64, u64)) -> Entity {
    Entity {
        kind,
        pos,
        hp: 900,
        max_hp: 1000,
        alive: true,
        targetable: true,
    }
}
fn fixture() -> World {
    World {
        tick: 480,
        end: false,
        entities: vec![
            entity(EntityKindV1::Epic, (300000, 300000)),
            entity(EntityKindV1::Champion, (320000, 300000)),
        ],
        calls: Calls::default(),
    }
}
#[test]
fn hits_champions_with_correct_cc_damage_and_registered_effect() {
    let mut w = fixture();
    w.entities
        .push(entity(EntityKindV1::Champion, (300000, 340000)));
    run(&mut w);
    assert_eq!(
        w.calls.cc,
        vec![
            (1, CcKindV1::Airborne.code(), 60),
            (2, CcKindV1::Airborne.code(), 60)
        ]
    );
    assert_eq!(
        w.calls.damage,
        vec![
            (
                0,
                1,
                100,
                DamageTypeV1::Fixed.code(),
                AttackTypeV1::Skill.code()
            ),
            (
                0,
                2,
                100,
                DamageTypeV1::Fixed.code(),
                AttackTypeV1::Skill.code()
            )
        ]
    );
    assert_eq!(
        w.calls.effects[0],
        (
            HIT_EFFECT.into(),
            0,
            InputTargetKindV1::Pos.code(),
            320000,
            300000
        )
    );
}
#[test]
fn excludes_distant_dead_untargetable_and_nonchampions() {
    let mut w = fixture();
    w.entities[1].pos = (390001, 300000);
    let mut dead = entity(EntityKindV1::Champion, (300000, 300000));
    dead.alive = false;
    w.entities.push(dead);
    let mut hidden = entity(EntityKindV1::Champion, (300000, 300000));
    hidden.targetable = false;
    w.entities.push(hidden);
    for kind in [
        EntityKindV1::Minion,
        EntityKindV1::Summon,
        EntityKindV1::Serpen,
        EntityKindV1::Tower,
    ] {
        w.entities.push(entity(kind, (300000, 300000)));
    }
    run(&mut w);
    assert_eq!(w.calls, Calls::default());
}
#[test]
fn idle_dead_missing_lord_and_finished_match_do_nothing() {
    for case in 0..5 {
        let mut w = fixture();
        match case {
            0 => w.entities[0].hp = 1000,
            1 => w.entities[0].alive = false,
            2 => w.entities[0].hp = 0,
            3 => w.entities[0].kind = EntityKindV1::Serpen,
            _ => w.end = true,
        }
        run(&mut w);
        assert_eq!(w.calls, Calls::default(), "case {case}");
    }
}
#[test]
fn only_positive_eight_second_clock_ticks_fire() {
    for t in [0, 1, 479, 481, 959] {
        let mut w = fixture();
        w.tick = t;
        run(&mut w);
        assert!(w.calls.cc.is_empty());
    }
    for t in [480, 960, 1440] {
        let mut w = fixture();
        w.tick = t;
        run(&mut w);
        assert_eq!(w.calls.cc.len(), 1);
    }
}
#[test]
fn radius_is_circular_inclusive_and_overflow_safe() {
    assert!(in_range((0, 0), (90000, 0)));
    assert!(!in_range((0, 0), (90000, 1)));
    assert!(!in_range((0, 0), (90001, 0)));
    assert!(!in_range((0, 0), (u64::MAX, u64::MAX)));
    assert!(in_range((u64::MAX, u64::MAX), (u64::MAX - 90000, u64::MAX)));
}
#[test]
fn lethal_damage_keeps_position_effect_valid() {
    let mut w = fixture();
    w.entities[1].hp = 50;
    run(&mut w);
    assert!(!w.entities[1].alive);
    assert_eq!(w.calls.effects.len(), 1);
    assert_eq!(w.calls.effects[0].3, 320000);
}
#[test]
fn independent_simulations_produce_identical_results() {
    let mut a = fixture();
    let mut b = fixture();
    run(&mut a);
    run(&mut b);
    assert_eq!(a.calls, b.calls);
    // A previous simulation does not alter the next match's clock behavior.
    let mut c = fixture();
    c.tick = 1;
    run(&mut c);
    assert_eq!(c.calls, Calls::default());
}
