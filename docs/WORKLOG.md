## level-0c-remaining-teleports-2 (2026-10-31, s/level-0c-remaining-teleports-2)

Branch from origin/master (21bfe192). Task: 0c the walk's last placements — walkreal_0700000c_live
and walkreal_0700000c_eastroute_live still carry bond.pos writes (the trench exit, the east climb,
the ladder495 measurement spot, the eastroute start/jump spots). For each remaining write: if it is
a route stand-in, find the game's route and walk it on real input; if it is a settled no-route or a
measurement spot, make sure the reason and a sister assert sit next to it. Delete every write that
can go. Run walkreal_0700000c_live, walkreal_0700000c_eastroute_live and interactables_0700000c_live;
quote the totals.

