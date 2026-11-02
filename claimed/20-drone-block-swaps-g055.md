## Your task: `drone-block-swaps-g055`, the other behaviour-block swaps (GAPS G-055)

origin/master now swaps a drone's active property block on GoToGoal mode-7 arrival (`NDrone2_ChangeToAttackMode` 0x14a648 -> `set_2nd_behaviour`). Three game swaps are still unwired (docs/GAPS.md G-055): `FirstAttack`'s alternate-state branch [M 0x148010:100-131], `PreDroneControl`'s second-switch flush [M 0x148db0:58-82], and `HandleImpact`'s 09/0A `Set1st` [M 0x1469b8:114]. Port each with the address, and pin each with a scenario on a level that uses it (sister: the swap does not happen when its condition is false). Run ai_live, stealth_ai_live, every walkreal_* and nf-game; fix walks only where fragile; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-block-swaps-g055`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-02T13:58Z
