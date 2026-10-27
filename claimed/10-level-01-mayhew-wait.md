## Your task: `level-01-mayhew-wait`, Mayhew waits for Bond instead of running ahead

On Double Cross Part 1 (07000001), during the escort Mayhew runs far ahead and through the door before Bond: at Bond (4.63, 6.47, 6.80) facing the hall corridor (frame ~4255 of the level) Mayhew is already past the far doorway. In the game the escort waits for the player at its hold points and does not go through a door ahead of him. Find the escort's wait/hold rule [M the escort follow states and their player-distance gates, the door-wait logic — escort_follow_07000001_live and fix_npc_doors_live pin parts of it] and the route points on this corridor [D], fix what lets him run ahead, and pin it with a real-input walk: Bond stays back at the corridor start, and Mayhew holds at his wait point instead of passing the doorway.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-mayhew-wait`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-27T00:47Z
