## Your task: `level-13-chain-difficulty`, 13: the 12 -> 13 chain walk mixes difficulties

The 07000013 walk chain plays 07000012 at difficulty 3 (007) and 07000013 at difficulty 1, so Bond arrives at the 13 lobby with 43-50 health and dies on real damage. A player keeps one difficulty through a mission. Make the chain carry one difficulty end to end (the warm save carries GameState+0x40 [M the save/restore path]), check the 13 lobby fight on real damage at the default difficulty, fix the walker if Bond dies to tactics (never game code without a cited difference), and add a real-damage variant for 13. Run walkreal_07000012_live, walkreal_07000013_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-chain-difficulty`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-01T10:33Z

---
## Report (done, 2026-11-01T12:14Z)

