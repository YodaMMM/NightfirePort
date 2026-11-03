# level-12-copter-sound-distance — the helicopter sounds too close from behind

Branch from origin/master.

On 07000012's helicopter fight the copter's engine and gun are heard almost at full strength when the copter is behind Bond and away from him; the position cue is weak.

Do:
1. Find how the game positions the copter's looping sounds [M the copter object's update + the SFX 3D call it uses (volume falloff by distance, pan by angle, any rear attenuation)] and what radii/curve the copter's sound records carry [D].
2. Compare with the port's 3D sound path for that emitter (nf-audio / sfx.rs): falloff curve, min/max distance, pan law, whether the emitter follows the copter's live position each tick.
3. Fix the differences; cite each number. Add a test that pins the copter emitter's volume and pan at a few distances/angles to the game's law.
4. Run the nf-audio tests, the copter tests, then the full suite.

---
claimed-by: sandbox1 at 2026-11-03T05:13Z

---
## Report (done, 2026-11-03T05:55Z)

Closed by the driver: the report is on `s/level-12-copter-sound-distance` (docs/reports/level-12-copter-sound-distance.md); the worker didn't run finish.sh.
