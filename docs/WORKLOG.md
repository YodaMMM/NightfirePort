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
