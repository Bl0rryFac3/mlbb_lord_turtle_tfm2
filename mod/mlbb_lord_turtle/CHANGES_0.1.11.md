# 0.1.11 — same-lane structure targeting

Adds authoritative live tower inventory, an explicit next required tower, outer-to-inner attack order and recovery when displaced past a live tower. Keeps the MLBB-style rule that other-lane towers can survive when Lord attacks the nexus through a cleared lane.

Adds TOWER and STRUCTURE_GOAL log records. Includes all 0.1.10 animation work, still awaiting explicit visual confirmation. No sprite pixels, attack damage, spawn scheduling or ownership rules changed.

40 automated tests passed; real in-game validation is still required. The user's screenshot has not been proven to show a same-lane skip.
