## Your task: `level-4a-every-objective-4`, the scoped drone bullet flight with the 08 fusebox walk green

Read the level-4a-every-objective-3 report (`origin/s/level-4a-every-objective-3`, not merged). With the game's cel-scoped drone bullet flight [M 0x12e7b0:343 + 0x1eb320:125], `walkreal_07000008_live::walk_the_whole_mission_on_real_input` stops at the fusebox shot (ch 137 never rises); the flat flight keeps it green. The report narrowed it to the station's guards' rounds reaching the player on the stair climbs under the scoped flight. Find exactly what changes for the walk (which guard, which round, what it hits), and what the game does there [M/D; L if needed]; fix the port side that is wrong (AI, walk route, or the flight) so the scoped flight lands AND the 08 walk is green. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-every-objective-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T05:32Z
