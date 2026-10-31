## Your task: `level-41-walk-to-the-end`, 07000041 from the plate to the level's end

`walkreal_07000041_live::walk_from_the_airlock_to_the_player_plate` stops at the player plate. Extend the walk on real input from there to the level's end (the next level's load or the mission end), every objective set by the game's own writers (no channel pokes); read the 41 docs and the existing 41 tests (the tether wires, the drone plate, the movie trigger). Fix any port gap it hits with evidence [M/D]. Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-41-walk-to-the-end`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T11:03Z
