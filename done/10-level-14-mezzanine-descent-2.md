## Your task: `level-14-mezzanine-descent-2`, Countdown Part 1: walk the mezzanine descent and un-ignore the delta walk

Read the level-14-mezzanine-descent report (merging now): the hand-back pad and the landing below it are mapped, with two leads for the rest of the descent. Walk the descent into the facility on real input, fixing any port collision gap [D/M], and un-ignore `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` with the rest of the walk green. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-mezzanine-descent-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T03:37Z

---
## Report (done, 2026-10-30T04:52Z)

Closed by the driver: the report is on `s/level-14-mezzanine-descent-2` (docs/reports/level-14-mezzanine-descent-2.md); the worker didn't run finish.sh.
