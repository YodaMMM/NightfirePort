## Your task: `level-05-every-objective-3`, The Exchange castle: from the crawl exit to the 07000006 exit

Read the level-05-every-objective-2 report (on master). The 05 walk is real through the cable and the window-ledge lane to the crawl exit. Continue on real input, in order: the roofed corridor south of the crawl exit (x -36, z -38..-44; decode its mouth), the second crack (z -46..-50, jumped), the south gallery, the SW loop, the SWITCH plate #269, the Q-Claw grapple over the moat inlet to pulley #281, the east stairs and door #112's E -> ch 0x1E -> the 07000006 exit. Fix any port gap the real legs hit [M/D]. The walk must stay green; un-ignore what turns green. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-every-objective-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T23:48Z
