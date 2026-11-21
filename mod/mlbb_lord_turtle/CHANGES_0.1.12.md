# 0.1.12 — nexus defenders before core

Corrects the earlier interpretation of the user's screenshot: the remaining two towers guard the nexus. Adds a separate guard phase independent of lane labels. Uses nexus targetability plus a no-effective-damage fallback, records close defender IDs, fights both before returning to core, and holds if the protected core has no discoverable defenders.

No sprite, ownership, spawn-clock, damage or balance change. 46 automated tests passed; actual guard identification and renderer behavior need an in-game test.
