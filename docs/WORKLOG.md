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

## Progress (2026-11-01, ~01:55 UTC)

- THE SUSPECT REFUTED AT THE DATA LEVEL: the port's walk rides the cab's roof slab, and that IS
  the surface the game's own feet ray lands on [M Player_FeetOnPoint 0x194188 — a ray down from
  HITTEST+0x30; the first hit from above on obj 'lift' 0x02000367 is the keyed+3.76 slab]. But
  the cab is a SEALED CAGE: the parented glass 0x2000421 ('lift_glass', 56 tris, local y
  −1.26..+2.46) closes every face at body heights (36/36 face rays hit at floor+0.5/+0.9/+1.5),
  the collide node's kind-exclusion word cel+0x98 is written only for ROOM cels
  (parseentity_fixup 0x1d00c0:115/142) and the static parser writes only cel+0x94
  (0x1d1998:101-103) — so Collide_Intersect 0x1ebaa8:301-317's kind filters never open it — and
  the interior platform floor (local −0.15) has no walkable opening. Every exit from the ride
  surface is walled or a floor hole (east x 2..6 z 24.5..27.5 = no floor to y 39; south = the
  shaft's south wall, top 55.5); the AI nav has no balcony→exec route (nf-extract route: code 5).
- NEW PIN: walkreal_0700000b_live::the_paternoster_cab_is_a_sealed_cage (4/4 green in the
  binary) — the face-ray grid + the interior emptiness, with the [M] citations inline.
- The two south-step-off rewrites I built this session (east-first-then-south through the shaft
  wall, then the walked lane to n41) are NOT committed as route changes: the walker wedges on
  the shaft's south wall exactly as the ray grid predicts — the geometry refuses the transit.
- The teleport stand-in REMAINS in walkreal_0700000b_live (reverted to the green master route);
  the route up is [S] unpinned per docs/levels/0700000b.md:65 — three live checks for the main
  agent are listed in lifts.md §7 item 5.
- Totals so far: walkreal_0700000b_live 4/4; walk_0700000b_live 4/4; interactables_0700000b_live
  9/9; models_sweep_0700000b_live 2/2; objectives_0b_start_live 3/3; nf-game 98 binaries
  959 passed / 0 failed / 9 ignored. Full suite running (slow: 1013 binaries; partial at push).
