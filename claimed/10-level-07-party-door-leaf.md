## Your task: `level-07-party-door-leaf`, the 07000007 party door's swinging leaf moves Bond the way the game's does

Branch from origin/s/level-07-door-condition, merge current origin/master, and read its docs/reports/level-07-door-condition.md. With the game's branch restored, `party_door_lock_07000007_live`'s two tests are ignored: the rerouted walk-in (E at (95.1, -47.6), wait for OPEN, cross at x 95.0) is pushed out of the east sliver mid-swing, from (95.02, -48.14) to (95.48, -48.62), with the door state alternating 0x08/0x10, and Bond ends at x 96.18 outside the plate box. Read how the game's door leaf pushes a body: the door update's swing arc, the leaf's collision shape and its contact response, with addresses, and compare the port's sweep and push strength. If the port pushes harder or along a different arc, fix it and say which. If the game pushes the same way, the walk changes: time the crossing to the swing. Remove both ignores. Never lower damage or health. Run party_door_lock_07000007_live, door_guard_live, level07_library_door_guard_live, npc_doors_live, interact_07000007_live, walkreal_07000007_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-party-door-leaf`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-08T03:46Z
