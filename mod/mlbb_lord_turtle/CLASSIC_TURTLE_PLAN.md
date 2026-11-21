# Accepted next gameplay direction: classic MLBB Turtle

User chose classic team gold/EXP and shields over native TFM2 stat stacks or the newer Healing Turtle lane-support variant.

Current v0.1.14 preserves the existing native rewards. Supplied TFM2 0.7.2 game_setting lists serpen_jungle gold=50, growth_gold=10, exp=60. Those are configuration values, not a verified statement of the amount/distribution each teammate receives. Native object text explicitly says the permanent Serpen reward applies to the entire team. Its serpen_permanent_buff definition has +3 attack_mult, magic_power_mult, defence_mult, magic_resistance_mult and +1 move_speed_mult. No custom team-gold payout, shield or EXP adjustment is implemented in v0.1.14.

Classic source direction: shared team gold/EXP, one-time team shields, stronger last-hitter shielding. Native API supports player gold changes and entity_add_shield. The supplied ABI10 SDK exposes no direct player/hero EXP grant. Before adding rewards or removing native ones, use TURTLE_NATIVE_REWARD / TURTLE_NATIVE_PLAYER in the local log to identify existing per-player gold/EXP delivery. These log adjacent-frame deltas; other income occurring in the same tick can be included. More than one capture may be needed to distinguish distribution.

Once verified, retain/rebalance native EXP if it already supplies the required shared reward; otherwise a supported engine EXP hook is needed for a faithful team-EXP implementation. Do not fake XP by editing statistics or grant repeated native-plus-custom gold. Replace the permanent stat reward deliberately when switching to shield rewards. Do not clear unrelated item shields when implementing stronger or regenerating Turtle shields.

References inspected:
- https://play.google.com/store/apps/editorial?hl=en_US&id=mc_editorialevergreen_post_install_mobile_mls_campaign_fcp — classic jungle/Turtle overview.
- https://mobile-legends.fandom.com/wiki/Turtle — classic gold/EXP and team/killer shielding descriptions; exact patch/version may differ.
- https://liquipedia.net/mobilelegends/Patch_2.2.16 — newer Healing Turtle battlefield variant differs; user selected the classic variant.

No additional gameplay rewards have been silently enabled in this presentation-polish release.
