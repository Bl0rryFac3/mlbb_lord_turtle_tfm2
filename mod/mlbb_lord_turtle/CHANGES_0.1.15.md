# v0.1.15 — summoned Lord performance candidate

Target: Teamfight Manager 2 v0.7.2, Windows x64. This candidate addresses feedback that the latest Lord/Turtle mod slows the game while the summoned Lord is alive and that the slowdown stops when he dies. The report is credible, but the engine has not been profiled here. An in-game comparison is still required; this is not a measured FPS fix.

Changes:
- Reduce Lord body-view events from 60 to 20 per simulated second per living Lord. Each frame follows the combat entity for three ticks; an off-cadence spawn gets an immediate one- or two-tick first frame. The original nine glide frames and three attack frames are retained. Frame/facing transitions can appear up to two simulation ticks later (about 33 ms at normal speed).
- Filter entity kind before allocating names when finding Lord summons, in both gameplay and minimap capture.
- Read controller markers once for the secure-score pass, rather than once per player.
- Skip route, tower and combat-target scans while Lord is movement-locked by an attack animation or crowd control. They resume on the first unlocked tick.
- Search for the minimap before Lord appears, spreading the search across render frames with a 32-node/frame budget, 512-node total limit and depth cap of eight. Existing detected-anchor/screenshot-fallback rules are retained.

Lord combat/movement still uses the 60-tick simulation. Ownership, next-wave spawn timing, lane selection, structure ordering, damage, cooldown, slam, TTL and base-guard protection remain the same. v0.1.14 English Lord/Turtle naming and marker assets are included. Turtle still uses native Serpen rewards; the proposed classic gold/EXP-and-shields update is not enabled.

Validation:
- 57 automated tests pass, including blue/red ownership, attack/CC timing, structure protection, independent simulations, followed-effect targets, body cadence and immediate partial spawn frames.
- Same mocked 30-second workload on v0.1.14 and v0.1.15: one Lord, 100 additional minions, run separately for both securing teams. Body events 1800 -> 600 (-66.67%); entity-at reads 1053000 -> 836433 (-20.57%); entity-name API calls 423450 -> 7200 (-98.30%); buff-count reads 11581 -> 7964 (-31.23%). Final Lord position, full damage event sequence and tower HP match exactly for both teams.
- Those counts measure the push hook through a mock stable host. They exclude the real renderer, UI hook and presentation capture and are not FPS or wall-clock measurements.
- Windows GNU release DLL built against the supplied stable ABI10 SDK. All 72 body bindings resolve to in-bounds frames with the matching one/two/three-tick duration. Neutral sprite atlases and objective definitions match v0.1.14.

Install and compare:
1. Close the game. Move the current mods/mlbb_lord_turtle folder outside mods as a backup, then replace it with this ZIP's mlbb_lord_turtle folder. Avoid duplicate copies under mods.
2. Restart and use a fresh match. Keep the same playback speed, map and other mods for the comparison. Watch before spawn, while Lord walks, while he attacks, and after death. Check for body flicker/trails, attack frames, health bar and minimap alignment.
3. Report whether the continuous slowdown is gone, improved or unchanged. If it remains, provide game speed and whether it happens in live matches, replays or both, plus the objective log opened by open_objective_log.bat.

Rollback: close the game and restore the backed-up folder. v0.1.13 remains the prior confirmed-working release in the catalog; no existing release ZIPs are replaced. Keep the Sanctuary map mod separate.

Source/SDK: native/src contains all Rust source and regression tests; native/sdk/mod-api-stable contains the full stable SDK; Cargo.lock is included. Run cargo test --locked --offline --manifest-path native/Cargo.toml and build_windows.bat on Windows with the Rust toolchain installed. To regenerate assets, run the existing compilers, ending with compile_body_frames.py then compile_presentation_assets.py; Python/Pillow is only required for asset rebuilding. Full-source bundle includes test output and raw baseline/candidate work counts.
