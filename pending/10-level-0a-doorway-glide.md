## Your task: `level-0a-doorway-glide`, Night Shift's west-hall doorway crossing matches the game

Branch from origin/master. Two walkreal_0700000a_live tests are ignored: the_west_hall_glide_height_matches_the_game (the settled y never reaches the game's 1.1285 [L 2026-10-26]: 1.0905 on master) and the_corner_crosses_like_the_game (the crossing mechanism is not the port's move clip; read the full ignore message). These are how Bond slides through a doorway corner, which a player feels. Find the game's mechanism [M the player collision handler and the doorway contact push], port it, and un-ignore both, with a test red on the old code. If the answer needs a live capture, write the exact question in the report. Run walkreal_0700000a_live, interactables_0700000a_live and every walkreal_*_live binary one level per tool call if the player collision changes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0a-doorway-glide`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
