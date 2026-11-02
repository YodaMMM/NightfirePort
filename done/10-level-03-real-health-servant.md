# level-03-real-health-servant — the 03 real-health walk's servant assertion

Branch from origin/master.

walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage is ignored: on the real-damage run the route's "the servant carrying 0x4f [D p6]" assertion (walkreal_07000003_live.rs ~line 231) fails — not a death (Bond is at 98.5 health at tick 22200). The top-up walk passes the same assertion.

Do:
1. Find why the real-damage run differs (the return-fire fights kill or move the servant, a different kill order, the damage log's return fire shooting him) and whether the game would let that happen.
2. Fix the walk (or the port, if the port lets something die that the game protects — cite it). Real input only.
3. Un-ignore the test; run walkreal_07000003_live, every real-damage walk, then the full suite.

---
claimed-by: sandbox5 at 2026-11-02T14:53Z

---
## Report (done, 2026-11-02T15:39Z)

Closed by the driver: the report is on `s/level-03-real-health-servant` (docs/reports/level-03-real-health-servant.md); the worker didn't run finish.sh.
