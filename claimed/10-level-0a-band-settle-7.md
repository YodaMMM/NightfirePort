## Your task: `level-0a-band-settle-7`, the last ignored 0a walk test

Start from `origin/s/level-0a-band-settle-6-b` (merged to master: the radial tail with the flags-99 gate) and read its report. `walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game` stays ignored (its note says why). Port the remaining piece (the feet-band walked-dy growth the game shows: the hall rest 1.1285 [L 2026-10-26] vs the port's 1.1201) from the game's ground/step rule [M Player_FeetOnPoint 0x194188, the walk's vertical], un-ignore it, and keep every walkreal test green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T06:20Z
