## level-0c-remaining-teleports-2 (2026-10-31, s/level-0c-remaining-teleports-2)

Branch from origin/master (21bfe192). Task: 0c the walk's last placements — walkreal_0700000c_live
and walkreal_0700000c_eastroute_live still carry bond.pos writes (the trench exit, the east climb,
the ladder495 measurement spot, the eastroute start/jump spots). For each remaining write: if it is
a route stand-in, find the game's route and walk it on real input; if it is a settled no-route or a
measurement spot, make sure the reason and a sister assert sit next to it. Delete every write that
can go. Run walkreal_0700000c_live, walkreal_0700000c_eastroute_live and interactables_0700000c_live;
quote the totals.


## level-0c-remaining-teleports-2, outcome (2026-10-31)

The eastroute test's two bond.pos writes are GONE, walked on real input: the start (the band
west end) now REPLAYS the main walk's own route — legs 1-9c moved verbatim from
walkreal_0700000c_live into common/walkreal0c.rs (`walk_spawn_to_the_band`, same load, same
ticks; the main test re-run green proves the move preserved the timeline) — then the band-east
leg to (52.0, −207.6); the jump start (79.6, 1.1, −210.2) is the main walk's own due-east
run-up legs, the yaw faced east on real turn input, and the old single jump became the main
walk's proven hop idiom (the under-steps 0.75..1.55 then the 1.75 first step [D the step
scan]). The ladder495 measurement spot keeps its write (the subject is the grab/climb classes,
not the route) and gains a floor sister (~0.0 [D probes]). The four writes that remain are the
documented stand-ins, each with reason + sisters inline: the trench exit (the vent shaft is a
ONE-WAY DROP [D block dump + M 0x194df0 case 3 + L 2026-10-26 hatch-3]), the mezzanine reach
(the filed live check; [L 2026-10-31] closed every other entrance), the east climb (the band
deck is a collision ISLAND [D eastclimb3-7], sisters 5/5b/5c), the ladder495 spot. Totals:
walkreal_0700000c_live 2/0, walkreal_0700000c_eastroute_live 1/0,
interactables_0700000c_live 5/0. Full suite on the final tree: see the report.
