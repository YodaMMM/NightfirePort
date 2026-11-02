# level-14-tripbomb-real-route-2 — finish the tripbomb route

Continue origin/s/level-14-tripbomb-real-route (branch from it; merge origin/master in). The previous session mapped the route from the level start and proved the pickup take; the last ~4 units into the rack lane and the place/shoot leg remain. Its report is on that branch.


Do:
1. Real input only from the level start (no teleport after load, no health/ammo/weapon writes): reach the rack, take the tripbomb through the game's pickup path.
2. Switch to it with the real switch request, place it on a wall with the real fire, walk out of its beam to >= 12 units, switch back to a gun and shoot it: it must explode through the bullet-vs-tripbomb rule tripwire_shot_live pins.
3. Log Bond's final position and the shot line. Add the live test; run tripwire_shot_live, interactables_07000014_live, walkreal_07000014_live, then the full suite.

---
claimed-by: sandbox1 at 2026-11-02T13:24Z
