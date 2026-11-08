## Your task: `homing-copter-4`, the copter's own spot check sees past its own body

Branch from origin/s/homing-copter-3, merge current origin/master, and read its docs/reports/homing-copter-3.md. The subject exclusion landed for the homing lock. What's left is the copter's spot check: `Copter` at 0x17dbb8:171-172 passes the copter and Bond as the ray's subjects, so its line of sight must not report either one's own collision. Use the same exclusion there, and check whether any other caller of `Collide_LineOfSight` in the decompile passes subjects that the port ignores, listing them in the report. Pin through real play on 07000008: the copter spots Bond with its own mesh on the ray, and a wall still blocks it. Run the homing binaries, the copter binaries, walkreal_07000008_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/homing-copter-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

