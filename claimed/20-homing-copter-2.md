# Homing rounds and the copter body: finish the arm

`origin/s/port-map-bullets` adds the class-0 arm of `Bullet_heatseekable` 0x12f248, but it is not mergeable yet. Branch from it, push to `s/homing-copter-2`.

Two gaps:

1. The game seeks a copter body only while its obj+0xfc byte is non-zero. The branch treats every created copter as a candidate, and a copter still waiting on its gate channel has a matrix of (0,0,0), so a homing round can turn toward the world origin. Trace what writes the body's +0xfc (`SP_HideObj`, the gate's first ON tick, `Copter_Update` 0x17dbb8) and carry that state. A hidden or gated copter must never be a candidate. Add a red test for the dormant case.
2. `homing_copter_07000012_live` is ignored: the give path refuses 0x2e on 07000012, and the copter never poses. Remove the ignore. Reach the real state by driving the gate channel on the real level and equipping through the path the level allows (07000012 carries pickups #2aa and #2f0). The test must show a fired round turning onto the body, and it must fail without the arm.

Also read `Bullet_homing` 0x12f358 past the target pick: check the re-validation and the steering against the branch. Report in docs/reports/homing-copter-2.md.

---
claimed-by: sandbox1 at 2026-11-07T09:32Z
