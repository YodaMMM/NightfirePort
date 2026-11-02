# level-14-tripbomb-real-route — pick up, place and shoot a tripbomb on real input

Branch from origin/master.

07000014 carries two PICKUP_Tripbomb placements (#1341/#1342, listed at -32.19,1.41,31.12 and -32.19,1.41,35.86 [D docs/levels/07000014-audit.md]). tripwire_shot_live proves a placed tripbomb can be shot from range, but it gives Bond the 0x3a gadget through the dev panel. No test reaches it the way a player does. Note: those coordinates may be map-local (the pickups did not show at that spot on the main map) — resolve the real world position from the placement's map first [D].

Do:
1. From the level start, walk a real route (real input only; no teleport after the load, no health/ammo/weapon writes) to a tripbomb pickup and take it through the game's pickup path.
2. Switch to it with the real switch request, place it on a wall with the real fire, walk out of its beam to ≥ 12 units, switch back to a gun and shoot it: it must explode through the game's bullet-vs-tripbomb rule (the one tripwire_shot_live pins).
3. Log Bond's final position and the shot line so the run can be replayed in the app. Add the live test; run tripwire_shot_live, interactables_07000014_live, walkreal_07000014_live, then the full suite.
