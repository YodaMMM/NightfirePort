## Your task: `los-mask-attrib`, the 09, 13 and 03 failures after the line-of-sight masks are traced and fixed

Branch from origin/s/los-mask-01-walk-2, merge current origin/master, and read its docs/reports/los-mask-01-walk-2.md, items 3 and 4 of "What's left". For each failure, first find what caused it (the mask change or master's own merge) by running it on both. 09's alarm-switch nav route: check NodesForPosition's cel search for the switch's route against the game [M 0x1586a8]. 13's plain-walk doorway: door swing against the merged solid bodies. guard_rounds_glass's 07000003 glass stop. Where the port differs from the game, fix it in the game's order and test it red on the old code; where it is a walk, repair it gated to its level. Real input only, never lower damage or health. Run walkreal_07000009_live, walkreal_07000013_live, walkreal_07000003_live, guard_rounds_glass, npc_doors_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-attrib`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

