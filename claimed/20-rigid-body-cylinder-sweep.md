## Your task: `rigid-body-cylinder-sweep`, the unported rigid-body contact step

The rigid-body collision handler's cylinder contact check and its torque step are unported (origin/s/lamp-calib's notes, rigid_body.rs). Port them from the game [M RB_CollisionHandler 0x1d7a80 and the cylinder sweep it calls; RB_RestingContactRot 0x1d73a0's non-bullet arms], so bodies react to Bond/guards/doors brushing them as in the game (07000004 lamps, 07000006 barrels, 07000001 lamps). Pin with tests (a walk-through contact moves the body the game's way; sister), keeping the bullet-hit curve pinned on lamp-calib unchanged. Continue from origin/s/lamp-calib (merge origin/master first). Run rigid_body_draw_live, rigid_body_wiring_live, interactables_07000001/4/6_live and nf-game. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/rigid-body-cylinder-sweep`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-01T14:14Z
