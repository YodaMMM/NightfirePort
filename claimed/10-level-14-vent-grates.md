## Your task: `level-14-vent-grates`, Countdown's vent grates sit in their frames

On Countdown Part 1 (07000014) the vent grates at the duct mouths draw hanging inside the duct, off their frames (seen from Bond at (32.61, 2.80, 45.27) and at (56.03, 0.65, 46.89) looking at the mouth). Find which objects these are [D placements, their object/glist and kind] and the game's rule for their pose: placement matrix, an open/kicked state, a door-like or SimpleScript mover, a hinge pivot [M the object's create/update and the draw's matrix]. Port it so each grate draws in its frame (and moves only the way the game moves it). Test against the [D]/[M] pose; the main agent renders.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-vent-grates`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-27T08:43Z
