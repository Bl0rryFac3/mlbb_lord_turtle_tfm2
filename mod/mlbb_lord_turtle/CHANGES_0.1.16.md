# v0.1.16 — overlay and probe isolation candidate

TFM2 0.7.2 Windows x64. The user reports continuous slowdown while the summoned Lord is alive, including with v0.1.15, which stops when he dies. This build isolates the custom minimap overlay and diagnostics as possible causes; no in-game performance result is claimed.

The custom MinimapOverlay extension is not registered. presentation.rs is compiled only for unit tests, so the runtime DLL has no presentation capture hook, custom marker draw calls, UI-tree search, Turtle reward-stat polling or presentation-state mutex. Diagnostic write_log is a no-op, with no environment lookup, directory creation, file open or disk append. Static icon assets stay in the pack because they also serve ordinary portraits/other UI.

The complete push.rs source is byte-identical to v0.1.15: ownership, lane selection, base-guard targeting, movement, attack damage/cooldown, attack/glide body effects and their 20-events-per-second cadence remain unchanged. Neutral slam and English Lord/Turtle names are retained. This build adds no Turtle shield or new gold/EXP reward. The reward probe removed here only read native rewards; it did not grant shields. Classic Turtle rewards remain planned.

Validation: 57 unit tests pass, including the unchanged ownership, route, tower/nexus, crowd-control and animation tests. Presentation helper tests still compile under cfg(test) for future work but that module is excluded from the runtime. Windows GNU release DLL built against bundled stable ABI10 SDK. All game assets and objective definitions match v0.1.15; only metadata and native binary change. No actual game FPS measurements are available.

Install: close the game, back up the old mods/mlbb_lord_turtle folder outside mods, and replace it with the runtime ZIP's folder. Avoid duplicate copies under mods. Restart a fresh match with the same other mods, map and playback speed. Check slowdown while Lord walks/attacks and after death. The custom moving minimap marker is intentionally absent in this test. Report whether the continuous lag disappears, improves or remains. The objective probe log will not receive new entries in this build.

If the lag is gone, the removed presentation/diagnostics paths are implicated and can later be reintroduced one at a time. If it remains, the next isolated check is Lord's body-effect rendering and summon control. That follow-up is not enabled here, so this test changes one group of suspected systems. Do not treat the candidate as a verified final fix.

Windows candidate contains only runtime DLL, mod definitions, thumbnail and game assets. All Markdown, plans, reports, build scripts and debug helpers stay in this full-source bundle. Roll back by closing the game and restoring the backed-up folder. Earlier releases are preserved in Dropbox.

Source: mod/mlbb_lord_turtle/native/src; full stable SDK: native/sdk/mod-api-stable. On Windows run VERIFY_AND_BUILD_WINDOWS.bat, or cargo test --locked --offline --manifest-path mod/mlbb_lord_turtle/native/Cargo.toml then cargo build --release --locked --offline --manifest-path mod/mlbb_lord_turtle/native/Cargo.toml. Tests and source were built with the bundled SDK; no external dependencies are required beyond an installed Rust toolchain. Asset compilers are included; no asset rebuild is necessary for this diagnostic update.
