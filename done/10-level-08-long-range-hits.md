# level-08-long-range-hits — drone #20 lands hits from 60 units

Branch from origin/master.

walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage is ignored: Bond dies at tick 26696 at (111.0, 13.5, -179.7); the last hits are -3.60 from drone #20 (skin 0x500002f, weapon 0x0) at (100.9, 28.2, -121.9), 60.5 units away, every ~4 frames.

Do:
1. Name drone #20 (placement, class, what weapon 0x0 means for it — Hind/turret/gunner?) [D] and find the game's fire law for it at range [M: range cap, accuracy with distance, burst/fire rate, damage].
2. Fix the port where it differs; cite it. If the port matches, give the walk the cover the level offers (no health writes).
3. Un-ignore the walk; run walkreal_07000008_live, the copter tests, every real-damage walk, then the full suite.

---
claimed-by: sandbox1 at 2026-11-02T16:33Z

---
## Report (done, 2026-11-02T17:01Z)

Closed by the driver: the report is on `s/level-08-long-range-hits` (docs/reports/level-08-long-range-hits.md); the worker didn't run finish.sh.
