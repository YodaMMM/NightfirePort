
## level-03-real-health-servant (started 2026-11-02 14:59 UTC)

Branch cut from origin/master 84e64088. Task: un-ignore
walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage — on the
real-damage run the route's "the servant carrying 0x4f [D p6]" lookup (~line 231) panics (not a
Bond death; 98.5 hp at tick 22200), while the top-up walk passes the same assertion. Find why the
runs differ (return fire moving/killing the servant, the kill order, the damage log's return fire)
and whether the game would let that happen; fix the walk or the port (cited); real input only;
then walkreal_07000003_live, every real-damage walk, full suite.
