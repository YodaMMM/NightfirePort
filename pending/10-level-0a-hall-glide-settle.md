## Your task: `level-0a-hall-glide-settle`, the feet-band climb settle and the 0a hall glide

`walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game` is ignored: the route crosses the doorway at the game's rate, but the settled y reads 1.1201 vs the game's 1.1285 [L 2026-10-26]. Its ignore names the missing mechanism: the hall glide's +0.0256 climb settle (the feet-band walked-dy growth, "band-settle finding 1"). Port that mechanism from the decompile [M Player_FeetOnPoint 0x194188 and its callers; cite lines], un-ignore the test, and keep every other test in the file and every walkreal binary that walks stairs/ramps green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-hall-glide-settle`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

