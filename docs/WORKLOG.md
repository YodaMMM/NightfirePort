## level-0b-lift-exit (2026-11-01, s/level-0b-lift-exit)

Branch from origin/master (e79794da). Task: 0b — off the lift without a teleport.
walkreal_0700000b_live's full walk teleports Bond about 6 m after the lift: stepping off lands
him in a pit whose rim is 1.35 m high while his jump rises about 0.77 m, and the dock's south
edge drops him down the shaft. Suspect: the walk rides the lift's ROOF while the game carries
the player INSIDE the car. Find the game's own route off the lift [D the lift car's collision
and the shaft geometry; M the mover/lift class and how it carries the player — Player_FeetOnPoint
0x194188, the platform ride], fix the port gap with the address cited, and delete the teleport.
Run walkreal_0700000b_live, interactables_0700000b_live, every *0700000b* binary and nf-game;
quote the totals. Full suite with the fast runner on the final tree (merged with origin/master).

## Progress (2026-11-01, ~01:10 UTC)

- Branch created from origin/master (e79794da) and pushed immediately; setup reused from the
  previous sandbox (work/, ISO, rust toolchain all present; freed 1.2G of browser caches).
