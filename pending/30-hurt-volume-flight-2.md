## Your task: `hurt-volume-flight-2`, the Icicles on 07000046 fall and hurt through a real shot

Branch from origin/s/hurt-volume-flight, merge current origin/master, and read the report in the queue's blocked/30-hurt-volume-flight.md. The state-1 flight is implemented and unit-pinned. Only the live pin's aim harness failed to converge, so the test's shooting is what is left. Fix the aim the way the report's harness notes say, using the real input path, then pin through the real level: an Icicle set off by a shot follows its path at the game's pace, Bond under it takes the damage, and clear of it he does not. Remove the ignore. Keep the rate-90 wrap question as a live check for the main agent. Run the hurt binaries, the 07000046 binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/hurt-volume-flight-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

