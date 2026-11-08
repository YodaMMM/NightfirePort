## Your task: `level-08-hind-fight`, the 07000008 Hind fight can be won on real input on every difficulty

Three branches each fixed part of the 08 Hind fight, and none has it green on 00 Agent. Branch from origin/s/glass-pierce-3, then merge origin/s/level-08-laser-hinges-2, origin/s/homing-copter-4 and current origin/master, and read the three reports (docs/reports/glass-pierce-3.md, level-08-laser-hinges-2.md, homing-copter-4.md). On 00 Agent the machine has 598 hp [M Copter_Create 0x17d340], so six full rockets of the eight carried must land. glass-pierce-3 measured four of eight rockets dying 2-7 ticks after launch, leaving 0.46 rad wide because the crosshair was still at its clamp, into the west doorway's south frame. Gate the launch on the aim having converged. laser-hinges-2 measured the Hind's missiles flying straight at Bond's launch-time position, so a 2-3 u side step off each missile's line is the dodge. homing-copter-4 says the Hind now attacks after its spot fix, so the walk's counter-tactic needs re-tuning, and its spot assert un-gates from the ignored test. Helpers stay gated to 07000008 and the difficulty, never lower damage or health, and keep the rocket count the walk really carries. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000008_live binary, the copter and homing binaries, glass_pass_every_level_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-hind-fight`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

