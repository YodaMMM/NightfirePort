## Your task: `level-14-walk-to-the-level-end`, Countdown Part 1 to the level's end

`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` is green and un-ignored (level-14-mezzanine-descent-2, merged). Extend 14's coverage on real input from where that walk ends to the level's end (the next level's load), every objective set by the game's own writers. Fix any port gap it hits with evidence [D/M]. Also correct the stale "STILL #[ignore]" comment above walk_infiltration_to_handback (walkreal_07000014_live.rs ~323). Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-walk-to-the-level-end`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-30T06:47Z
