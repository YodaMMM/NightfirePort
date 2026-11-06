# Guard radii and glass: five reds on current master

Branch from `origin/s/guard-on-master`, push to `s/guard-repin-master`. That branch is the guard radii and glass work on current master, reviewed. Full suite on it: 4417 passed, 5 failed. Merge this before the gas work: five reds against its fourteen.

The reds:
1. `walkreal_07000003_live` real_damage_agent: Bond died tick 51805 at (23.09, 6.37, 10.54).
2. `walkreal_07000013_live` warm chain real_damage: walk_to "the lobby west mid" hit the 6000-tick cap. Also a death at tick 2627 at (208.62, 1.04, -229.64); find which test that belongs to.
3. `walkreal_0700000d_live` c5 door real_damage: "the blind strip east 4 must walk".
4. `walkreal_07000006_live` castle interior real_damage: Bond died tick 5217 at (20.27, 41.13, -37.97).
5. `walkreal_07000008_live` whole mission real_damage_00agent: "the shoot-out must kill every end pane", 1 left.

For each say whether the walk is tuned to old guard positions or a guard now behaves wrongly. The cap and the "must walk" failures may be a guard or Bond standing somewhere new, so look before re-routing. Use route and timing levers for stream-tuned walks. Do not loosen pins tagged [L], do not add ignores to pass, and do not change game code to suit a walk.

Finish with the five binaries whole plus walkreal 01, 16, 1b and enemy_accuracy_live.

---
claimed-by: sandbox4 at 2026-11-06T09:59Z

---
## Report (done, 2026-11-06T10:41Z)

Closed by the driver: the report is on `s/guard-repin-master` (docs/reports/guard-repin-master.md); the worker didn't run finish.sh.
