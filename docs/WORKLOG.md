## level-14-hall-walk-in (2026-10-31, s/level-14-hall-walk-in)

Branch from origin/master (15bf41af, batch-1031i). Task: the last placement in the walk —
walkreal_07000014_live still places Bond at [-60.3, 0.9, 38.4] (around line 1176, after seeding
the fusebox channels 0x87/0x88 that the walk's own laser cut and the panel tests prove). Replace
the placement with a real walked route from where the test's walk stands to that spot
[D collision/nav incl. doors and their lock channels; M the classes the route uses], and if the
seeded channels can be raised by the walk's own real fusebox cuts in the same run, do that too.
Fix any port gap with evidence; if the data shows no route, pin that with a sister assert and
name the live check. Run walkreal_07000014_live, door_guards_locked_door_07000014_live and
interactables_07000014_live; quote the totals.

## level-14-hall-walk-in, progress (2026-10-31)

The placement is GONE. The shared `walk_the_west_route_to_the_switch_band` (the descent + the
south loop + the Kiko west delivery + the generator room + the two REAL fusebox cuts + the
switch-band arrival) is extracted from the main walk verbatim; the cascade test
(the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door) now loads, pins the disc
checks (the 0xA0/4 touch, the locked 0x85 pair), then WALKS: walk_infiltration_to_handback +
the shared route — 0x87/0x88 raised by the walk's own laser cuts [M 0x1adfe8], 0x8C by the AND,
no seed writes, no teleport-aim (turn/look steering). Runs: walkreal_07000014_live 6/0,
door_guards_locked_door_07000014_live 4/0/1 ignored (the pre-existing named-blocker pin),
interactables_07000014_live 9/0.
