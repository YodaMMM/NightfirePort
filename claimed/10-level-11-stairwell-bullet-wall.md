## Your task: `level-11-stairwell-bullet-wall`, 07000011: something invisible stops the rounds aimed at the rope guards

In the 07000011 stairwell, rounds fired at the abseiling guards stop on something invisible in the well. Three spots where it happens (Bond root x, y, z, yaw, pitch): (15.91, 24.01, -2.45, -5.907, -0.107), (20.35, 24.01, -2.44, -6.757, -0.102), (15.33, 26.69, 1.33, -4.900, -0.559); the guards are on the ropes at (18.895, 46.516, 1.434) / (19.745, 46.580, 0.363) [D]. The bullet decals land on the far stairwell wall, not on the guard.

1. From each spot, cast the bullet path toward the rope guard and list every triangle/object the port's bullet collision meets (owner object, placement kind, glist flags, material/surface byte, collision class). Find which one the game's bullet passes and the port's does not [M — `Bullet_update` 0x12e7b0 / `Bullet_CollisionHandler` 0x12ce80, the pick mask the round uses, `Collide_Pick` 0x1eb320's node/surface drops, the pass flags (surface 0x40/0x80, bit 0x10 render-only objects, glass class 0x10)].
2. Fix it [M], with a real-input test from one of the spots: the round reaches the guard and damages him; sister: a round into a real wall still stops there.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-wall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-25T06:37Z
