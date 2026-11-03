## Your task: `level-0a-glide-corner`, 0a: Bond's walking height through the west-hall doorway and the corner

Continue `origin/s/level-0a-ignored-checks` (merge `origin/master` first; read `docs/reports/level-0a-ignored-checks.md` on that branch). Two tests in `walkreal_0700000a_live` stay ignored: `the_west_hall_glide_height_matches_the_game` (settled y 1.1285 ± 0.003) and `the_corner_crosses_like_the_game` (exit y 1.1159 ± 0.005). The report narrows the missing +0.0256 settle to two remaining y writers on a walked line: the `Player_FeetOnPoint` band [M 0x194188:106] and the walk-anim root ride (AnimObject+0xcc, read fresh per tick [M 0x194df0:86]). Find which one the game uses on these lines [D the walk clip root keys; M the band math], port it with the address cited, and un-ignore both. Run walkreal_0700000a_live, every *0700000a* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-glide-corner`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

