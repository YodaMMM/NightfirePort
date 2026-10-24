## Your task: `level-15-ladder-3`, 07000015 ladder #382: settle whether the game's pick ever admits it

Two readings of the same decompile disagree:
- Master (`interactables_07000015_live.rs`, the ladder test and its doc): the ladder is reached by crouch-walking off the east room's south lip at x ~83.4 under the railing panels, falling, and strafing east; once Bond's straddle set empties below the room cel, `Collide_Pick` 0x1eb320:125-134 admits the unlinked ladder and `Player_CollLadder` 0x196118 grabs.
- Branch `s/level-15-ladder-2` (not merged): the ladder links to no cel (`build_FindCel` 0x1c8cb0 misses: its centre y 31.73 is below the east room's box, every shaft cel's box ends at x 81.87), `Controls_StraddleTest` 0x134778 does not force it, and `Collide_Pick`'s candidates are only the straddled cels' chains plus the ForcedList, so the ladder can never be picked.

Live [L fresh boot, PCSX2]: walking into #382 along the south walkway at y ~40 (both directions, one inside the angle gate) never engages a climb. The falling approach was not tested live.

1. Read `Collide_Pick` 0x1eb320 in full (disassembly where the decompile is unclear) and decide which reading is right; quote the lines.
2. If the ladder can be grabbed, make master's route test pass with real input. If it cannot, replace master's test with the branch's pin and its sister.
3. Name the PCSX2 read that would settle it if the decompile does not: a fresh-boot fall past x 84.53 from the south lip, watching the player substate for the climb.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-15-ladder-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T04:59Z
