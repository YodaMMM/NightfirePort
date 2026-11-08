## Your task: `guard-pane-drain`, a guard's round through a glass sheet breaks it as in the game

Branch from origin/s/glass-pierce-2, merge current origin/master, and read its docs/reports/glass-pierce-2.md, item 4 of "What's left". Guard rounds now pass material-0x10 sheets, but only the player's flight runs the contact drain on the breakable boxes. In the game a guard round through a health-0 sheet kills the sheet; the 08 booth sheets are the example [D param 1]. Read the guard round's flight in the decompile and where it reaches the breakable's damage and kill (`Break_Kill` 0x12b970), with addresses, and wire the same drain into the port's guard flights in the game's order. Rand draws must keep the game's count. Pin through real play on 07000008: a guard firing at Bond through a booth sheet breaks it, and Bond can then walk through. Run the bullet binaries, glass_pass_every_level_live, dead_pane_live, walkreal_07000008_live, walkreal_07000001_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-08T05:13Z
