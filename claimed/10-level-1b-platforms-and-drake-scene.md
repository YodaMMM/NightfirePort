## Your task: `level-1b-platforms-and-drake-scene`, 1b: platforms break at the slightest touch; Drake missing from the cut-scene

On 0700001b (1) the breakable platforms break at the slightest touch from Bond, and (2) Drake is missing from the cut-scene. For (1) find the platforms' break rule [M the class's damage/contact threshold, what counts as a hit; D placements] and fix the port so only what the game counts breaks them. For (2) find Drake's cut-scene entity [D the scene's streams and skin; M how the scene binds the actor] and why the port does not draw him. Pin both (sister checks). Run every *0700001b* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-platforms-and-drake-scene`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-01T04:25Z
