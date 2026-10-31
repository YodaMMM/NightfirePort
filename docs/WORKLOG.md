# WORKLOG

## level-05-terrace-walk (branch s/level-05-terrace-walk)

- start (master d1b31fd3). Task: 07000005 — `walkreal_07000005_live` still places Bond at
  [22.0, 50.1, -68.0] on the terrace (floor 49.001 [D probe_terrace05 col (21.8, -68.1)]) as a
  stand-in (the comment above that write, ~line 1365); find the game's route from where the
  test's walk stands to that terrace spot [D collision/placements incl. doors, ladders, wires,
  grapple points; M the classes the route uses], walk it on real input, delete the stand-in
  write; if the data shows no route, pin that with a sister assert and name the live check
  that settles it. Run walkreal_07000005_live and interactables_07000005_live; quote totals.
