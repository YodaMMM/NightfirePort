## Your task: `level-07-party-door-leaf-2`, Bond can reach the 07000007 party plate through the door as in the game

Branch from origin/s/level-07-party-door-leaf, merge current origin/master, and read its docs/reports/level-07-party-door-leaf.md. The 2-tick moved latch landed (the anti-crush bounce holds the leaf still [M 0x135980:109-111]). Take "What's left" in order: thread the swing window past the sill pit the report describes and remove both ignores in `party_door_lock_07000007_live`, then run `walkreal_07000007_live` end to end with the east reroute, which compiled but did not fit the last window. Never lower damage or health. Real input only. Run party_door_lock_07000007_live, door_guard_live, level07_library_door_guard_live, npc_doors_live, interact_07000007_live, walkreal_07000007_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-party-door-leaf-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T07:23Z
