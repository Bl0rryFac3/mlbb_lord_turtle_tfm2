# 0.1.10 — allied Lord attack animation

The lane controller previously applied custom damage but did not request the character animation, leaving the lane Lord in its glide loop.

Now each custom hit applies CcKindV1::Animation with name attack for 35 ticks. The custom movement lock respects that animation until it expires, including after a lethal hit. The existing 3-frame trident swing is reused.

The stock damage amount, cooldown, ownership, spawn scheduling, targeting and routes from the user-confirmed 0.1.9 are retained. No artwork, sheet layout, remappings or visual bindings changed. Damage timing is still the beginning of the cycle; this change addresses the missing visible attack.

34 automated tests pass. The actual renderer response still needs an in-game check.
