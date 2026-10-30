## Your task: `level-0d-crane-and-to-the-end`, 0d: the crane on real input, then to the level's end

`walkreal_0700000d_live::walk_arrival_to_the_c5_door` walks to the dock's south edge, then TELEPORTS Bond to the yard (140.0, 5.5, -276.0) as the documented crane stand-in (walkreal_0700000d_live.rs ~55-66), and stops at the C5 door. The level-0d-crane-transit probes (merged; `probe_crane*`, the census) mapped the crane ride with the boarding leg unproven. Make the crossing real: how the game gets Bond from the dock to the yard [D the crane placements/keys, the script player and its channels; M the classes], fix any port gap, and replace the teleport with real input. Then extend the walk past the C5 door to the level's end (the next level's load), every objective by the game's own writers. Every test in the file green. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-crane-and-to-the-end`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T19:28Z
