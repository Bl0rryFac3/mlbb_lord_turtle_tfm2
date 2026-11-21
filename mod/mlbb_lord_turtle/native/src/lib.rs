//! TFM2 stable-API Lord slam. No shared match state: identical behavior in
//! watched matches, server pre-simulation, and deterministic re-simulation.
use mod_api_stable::{
    declare_stable_mod, AttackTypeV1, CcKindV1, CcV1, DamageTypeV1, EntityKindV1,
    InputTargetKindV1, InputTargetV1, LogLevel, StableHost, StableMatchHook, StableMod, StableSim,
};

#[cfg(test)]
mod presentation;
mod probe;
mod push;

const PERIOD_TICKS: usize = 8 * 60;
const RADIUS: u64 = 90_000; // base Morgard attack range is 80_000 in 0.7.2
const DAMAGE: usize = 100;
const AIRBORNE_TICKS: u64 = 60;
const HIT_EFFECT: &str = "mlbb_lord_turtle:ocean_force_hit";

fn init(host: &StableHost) -> StableMod {
    host.log(LogLevel::Info,
        "mlbb_lord_turtle 0.1.16: Lord slam + allied lane-pushing summon loaded (8s match clock, 90000 radius, 100 fixed damage, 1s airborne)");
    probe::write_log("\nLOADED mlbb_lord_turtle 0.1.16 allied Lord prototype\n");
    let mut module = StableMod::new("mlbb_lord_turtle");
    module.set_match_hook(LordSlam);
    module
}

struct LordSlam;

fn is_slam_tick(tick: usize) -> bool {
    tick > 0 && tick % PERIOD_TICKS == 0
}

fn in_range(a: (u64, u64), b: (u64, u64)) -> bool {
    // Wide arithmetic prevents overflow when subtracting unsigned coordinates.
    let dx = a.0.abs_diff(b.0) as u128;
    let dy = a.1.abs_diff(b.1) as u128;
    // Early reject also makes the subsequent sum safe for arbitrary input.
    dx <= RADIUS as u128
        && dy <= RADIUS as u128
        && dx * dx + dy * dy <= (RADIUS as u128) * (RADIUS as u128)
}

impl StableMatchHook for LordSlam {
    fn on_match_start(&self, sim: &mut StableSim<'_>) {
        push::start(sim);
    }
    fn on_match_tick(&self, sim: &mut StableSim<'_>, _rng_seed: u64) {
        push::tick(sim);
        if sim.is_end() || !is_slam_tick(sim.tick()) {
            return;
        }
        let lord = (0..sim.entity_count()).find_map(|i| {
            let e = sim.entity_at(i)?;
            if e.kind() != Some(EntityKindV1::Epic) || !e.is_alive() {
                return None;
            }
            let (hp, max_hp) = e.hp();
            // Full-health idle Lords do not slam; neither do dead/invalid ones.
            (hp > 0 && max_hp > 0 && hp < max_hp).then(|| (e.id(), e.pos()))
        });
        let Some((lord_id, lord_pos)) = lord else {
            return;
        };
        // Snapshot IDs/positions before mutating the simulation. Both teams
        // contesting the neutral Lord can be hit; minions/summons are excluded.
        let targets: Vec<_> = (0..sim.entity_count())
            .filter_map(|i| {
                let e = sim.entity_at(i)?;
                if e.kind() != Some(EntityKindV1::Champion)
                    || !e.is_alive()
                    || !e.is_targetable()
                    || !in_range(lord_pos, e.pos())
                {
                    return None;
                }
                Some((e.id(), e.pos()))
            })
            .collect();
        for (id, (x, y)) in targets {
            sim.apply_cc(id, &CcV1::of_kind(CcKindV1::Airborne, AIRBORNE_TICKS));
            sim.deal_damage_typed(
                lord_id,
                id,
                DAMAGE,
                DamageTypeV1::Fixed,
                AttackTypeV1::Skill,
            );
            // Named binding ships in view/lord_slam.view_effects. Position
            // targeting remains valid even if the damage just killed the victim.
            let input = InputTargetV1 {
                kind: InputTargetKindV1::Pos.code(),
                x,
                y,
                ..Default::default()
            };
            sim.play_view_effect(HIT_EFFECT, lord_id, &input, 0, 0, 26);
        }
    }
}

declare_stable_mod!(init, requires = 10);

#[cfg(test)]
mod tests;
