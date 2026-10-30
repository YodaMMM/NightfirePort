
### level-0c-plates-reach (start, 10:38 UTC, 2026-10-30)

Branch s/level-0c-plates-reach = origin/master (06b95819, batch-1030c — carries the merged
level-0c-vent-pulley-class whose walk keeps the plates-reach stand-in). Task: find the game's
own route to the plates (the slab's south face z -155.77 closed every probed approach from the
2.37 deck — see that report's closure evidence) with [D] placements + collision faces (doors,
SS movers, ladders, crouch gaps, nav web) and [M] any class relied on; fix any port gap;
replace the teleport in walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_
0d_handoff with a real-input leg; if the data shows no route, pin it with sister asserts and
name the live check. The walk must stay green; quote the totals.

The window's findings (10:38-12:00 UTC): the plates' spot fully characterised [D #558/#559 box
[52.24,0.90,−174.45]..[53.29,3.20,−172.73] over the warehouse nook; the E-use sphere reaches it
from the west strip; the body touch needs the capsule inside]. The route WALKED for real in
probe0c_plates_route (the sim): display ground → the crouch walk-off at x 10.5 (z −116.5) → the
2.18 lane (lands (10.57,3.25,−121.34)) → the storage room west/south (one 2.18 floor,
room_storage01 #143, x −1.72..12.2) → the 2.37 pocket (z −150.3..−155.77). THE ONE LINK NOT
CLOSED: the pocket→yard descent — the slab face z −155.77 (y 2.37..6.15) spans x −4.5..15.25
[D — this run's scans extend the prior x 2.5..15.25 west to −4.5], the pocket's west edge hangs
over void, and the yard side is fully mapped (the y≈0 floor → the bridge x 7.57..16.39 → the
corridor 0.00→0.38→0.63→0.69 → the east arm x 50.63..57.61 z −176.08..−178.27 into the
warehouse → the pit floor 0.76..1.02 z −170.8..−171.1 → the plates' box graze from the pit's
south edge z ~−172.2). The nav web [D]: NO AI link crosses between the G2 south-strip patrol
(y 6.7, x −2.4..13.7) and the yard patrol (y 0.4) — the connection is a player-only traversal;
ladder #495 tops at y 2.0 (the shelf only [M the ladder495 test]). The route probe is
#[ignore]d (a failing probe must not join the suite); the nav dump test runs green. The
walkreal_0700000c stand-in is UNCHANGED this window.
