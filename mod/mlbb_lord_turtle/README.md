# MLBB Lord & Turtle

An original **Teamfight Manager 2** mod by **Bl0rryFac3** that brings MLBB-inspired objective fights and lane pushes to the standard map. It replaces Morgard’s presentation with the Elemental Lord and Serpen’s with the Turtle, using custom sprites, animations, portraits, and English objective text. Securing Lord also earns your team an allied siege unit.

## Compatibility

| | |
| --- | --- |
| Game | Teamfight Manager 2 v0.7.2 |
| Platform | Windows x64 |
| Mod version | v0.1.16 |
| Native API | Stable ABI 10 |
| Map | Standard map, with its stock wave timing and lane layout |

## Elemental Lord

**Neutral fight.** While alive and injured, the neutral Lord can slam nearby champions from either team on positive eight-second match-clock boundaries. The slam deals 100 fixed damage and applies one second of airborne crowd control. The slam has a circular radius of 90,000 game coordinate units. Dead, untargetable, and out-of-range champions are excluded, as are non-champion entities.

**Securing Lord.** The mod tracks the player’s epic-objective secure counter to identify the securing team. Each new secure schedules one team-owned Lord for the next stock minion wave. Historical scores at initialization do not generate summons.

**Lane selection.** At spawn, Lord chooses the lane with the fewest surviving enemy towers. If the counts tie, he favors the lane with the weaker front tower. The final tie order is mid, top, then bottom. Routes follow the standard map and work for both sides.

**Siege behavior.** Lord advances through the chosen lane and attacks the outer tower before the inner tower. When nearby base guards protect the nexus, he can leave the original lane to clear them before hitting the core. A living required tower can block progress while temporarily untargetable. The allied Lord has 6,000 HP, 220 physical attack, 60 armor, 60 magic resistance, and a 90-second lifetime. Attacks use a one-second cooldown. Within range, Lord prioritizes the required structure, then an exposed nexus, minions, champions, and summons. Attack animations and crowd control can pause movement. The allied summon uses its own siege attacks.

**Animation.** The summoned Lord has nine glide frames and three trident swing frames, with left and right facing. Visual effects follow the underlying combat unit and its health bar. Body updates run at 20 events per simulated second while the Lord is active. The current moving minimap overlay is disabled. Static portraits remain available.

## Turtle

Turtle replaces Serpen’s visuals and English name across objective UI, with its own idle, attack, and collapse sprites. It keeps the game’s native Serpen gold, experience, and permanent team-stat rewards. Classic Turtle shields and a custom gold or EXP payout are planned features.

## Install or roll back

1. Close Teamfight Manager 2.
2. If updating, move the current `mods/mlbb_lord_turtle` folder outside `mods` as a backup.
3. Extract the Windows package and place its `mlbb_lord_turtle` folder in the game’s `mods` directory. Keep one installed copy.
4. Enable the mod and start a fresh match.

To roll back, close the game and restore the backed-up folder. Sanctuary is a separate map mod. Custom layouts and wave schedules are outside this version’s supported scope. The objective name changes target English.

## Build and test

The native module is written in Rust. From the full-source bundle root, run `VERIFY_AND_BUILD_WINDOWS.bat` with a Windows Rust toolchain installed, or run:

```sh
cargo test --locked --offline --manifest-path mod/mlbb_lord_turtle/native/Cargo.toml
cargo build --release --locked --offline --manifest-path mod/mlbb_lord_turtle/native/Cargo.toml
```

The source bundle includes the stable SDK and Cargo lockfile. Python and Pillow are needed only to regenerate sprites. The shipped Windows DLL was built for the Windows GNU target.

The v0.1.16 test report records 57 passing tests. Coverage includes both securing teams, wave timing, tower order, base defenses, crowd control, neutral slam, animation cadence, and independent simulations. The author confirmed in game that the slowdown while the summoned Lord was alive was resolved in v0.1.16.
