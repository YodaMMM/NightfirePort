## Your task: `level-0a-office-hatch-5`, 0700000a: the post-spawn settle (1.1000 -> 1.0891), then the hall and the corner

Live [L fresh boot; Collide_RayIntersect breakpointed from build_PointOnFloor]: the game's spawn ray from (-3.9319, 0.1, 10.3978) down to y -2.9 hits **y 0.0000** in the spawn cel (surface 0x15) — the SAME floor the port finds. So Player_StandAtNewPosition places the root at 1.1000 in both, and the snapshot's 1.0891 [L ee_start_0700000a] comes from what happens AFTER the spawn: a -0.0109 settle (the collision push against the floor at the capsule's cc 1.0347 band, the feet-on-point settle, the anim-driven cc [M Player_Collision 0x194df0 reads AnimObject+0xcc every tick]). The previous round's hunt for a -0.0109 surface (branch `s/level-0a-office-hatch-4`, not merged) is therefore moot.

1. Find the settle: run the port from the spawn with no input and log the root each tick; compare with the game's 1.0891 rest. Find which step (the player collision's floor push, Player_FeetOnPoint 0x194188's band, the animated cc) lowers the game's root by 0.0109, and port it [M].
2. Then the hall glide (game 1.1147 after the doorway-base climb at x -8.304) and the corner at (-17.87, 4.87) (the game walks through at y 1.127 -> 1.120), and un-ignore the three sisters and the main walk in `walkreal_0700000a_live`.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-24T01:37Z
