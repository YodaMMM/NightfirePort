## Your task: `guard-glass-sight-2`, every level's walks stay green with guards seeing through glass as in the game

Branch from origin/s/guard-glass-sight, merge current origin/master, and read its docs/reports/guard-glass-sight.md. Guards' sight now follows the game's per-material glass rule, and the 08 walk is the one red. Repair it first, gated to 07000008 and the difficulty: plan its fights with the booth guards' new sight. The report notes a fight-filter repair that was tried and reverted, so do not repeat it. Then run the rest of the walkreal battery (02, 03, 06 and every other level) one level per tool call, and repair any walk that shifts, gated to its level. Carry the report's three live checks forward. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the ai binaries, glass_pass_every_level_live, dead_pane_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-glass-sight-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

