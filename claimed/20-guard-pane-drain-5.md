## Your task: `guard-pane-drain-5`, the 07000001 d3 walk passes with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain-4, merge current origin/master, and read its docs/reports/guard-pane-drain-4.md plus the 01 d3 ignore message. Take its SPEND lever: trim the d3 pre-clear and stock passes and keep the wheel armed through the named loop. The phase after the AND gate has never run, so expect escort-phase work once the gate fires, and keep going through it to the exit. Real input only, no top-ups, never lower damage or health, helpers gated to 07000001 and difficulty 3. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary, the bullet binaries, glass_pass_every_level_live and dead_pane_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-09T05:55Z
