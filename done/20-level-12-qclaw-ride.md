## Your task: `level-12-qclaw-ride`, the 07000012 SkyBridge Q-Claw crossing through the real input path

The 07000012 audit marks the Q-Claw ride as untested: GRAPPLEPOINT_SkyBridge1 #130 at (11.49, 9.23, 30.14) (`Grapple_Create` 0x198298, class 0x51; the port's gadget.rs pick_grapple_point). Drive it the way a player does — select the Q-Claw, aim at the point from where the level's route reaches it, fire, ride to the end, land — with real input through the Walker, and compare every stage with the game's rule [M Player_SSGrapple*, Bullet_Delete 0x12e660:50-55's grapple arm, the ride's speed/end/drop]. Fix whatever the port gets wrong on that path (aim pick, the attach, the pull speed, the landing pose, the exit) and pin it: the ride starts from the reachable spot, Bond arrives on the far side at the game's landing point, and he can walk on from there.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-12-qclaw-ride`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-26T22:23Z

---
## Report (done, 2026-10-26T23:18Z)

Closed by the driver: the report is on `s/level-12-qclaw-ride` (docs/reports/level-12-qclaw-ride.md); the worker didn't run finish.sh.
