## Your task: `bot-push-per-bot-gates`, Bond is pushed off guards every tick like the game, so he can't be shoved through the 03 gatehouse wall

`crates/nf-view/tests/flank_fall_repro.rs` `the_old_courtyard_flank_stays_on_the_floor` is #[ignore]d: on 07000003 Bond rides the gatehouse wall's south end-edge, goes airborne, and on that first airborne tick the port's bot-capsule push discharges the whole overlap built up while grounded (two guards, ~0.74 u in one tick), carrying him across the wall plane x 10.915 into a box with no floor; he falls out of the world. The game pushes off bots EVERY tick [M Collide_Jointy 0x1ec180:229-281, no ground gate], gated per bot by `NDrone2_DoCollision` 0x149ec8 and the drone's +0x4f8 0x20/0x200 bits. The port's airborne-only gate is a port choice; lifting it alone broke the 03/01 walks and exit_aim_01 (measured, see the file's comment). Port the per-bot +0x4f8 gates [M: every writer of 0x20/0x200 and the DoCollision reads], then make the push run every tick as the game's does, and un-ignore the test. Run flank_fall_repro, walkreal_07000003_live, walkreal_07000001_live, the exit-aim 01 test and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/bot-push-per-bot-gates`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-02T06:10Z
