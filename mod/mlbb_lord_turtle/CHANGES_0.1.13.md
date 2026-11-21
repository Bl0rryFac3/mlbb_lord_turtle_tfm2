# v0.1.13 — summoned Lord body animation candidate

v0.1.12's targeting is user-confirmed. Its generic summon still ignored the Animation CC visually. This candidate retains the combat entity and health bar but makes its built-in sprite transparent. Tick-sized named body-frame effects now draw the supplied glide and existing three-frame attack directly. It uses the view-event pipeline already used for Lord hit effects and records frame selection in simulation time for replays. Facing follows horizontal movement/attack direction. The neutral Lord/Turtle art, damage timing, ownership, spawn timing and target selection are unchanged.

49 automated tests pass. These verify frame selection, swing completion after a lethal hit, following the live position, both facing directions and the existing targeting regressions. They cannot prove the renderer displays/alines these effects correctly. This is an in-game candidate. Check body visibility, anchoring to the health bar, no doubled body/flicker at x1/x3, pause/replay behavior and trident swing against a tower. Existing attack art is reused; no new attack artwork is claimed.

Close the game. Back up mods/mlbb_lord_turtle outside mods; replace it with this package's mlbb_lord_turtle folder. Restart a fresh match. To revert, replace it with the v0.1.12 backup. If the body disappears or floats away from its health bar, revert and report that behavior.

Build: cargo test --locked --offline, then cargo build --release (Windows Rust toolchain). native/sdk/mod-api-stable is included. Asset rebuild order: compile_sprites.py, repair_runtime_tags.py, compile_glide.py, compile_push_sprites.py, compile_body_frames.py. Python Pillow is needed only to rebuild assets. Runtime needs no Python or compiler.
