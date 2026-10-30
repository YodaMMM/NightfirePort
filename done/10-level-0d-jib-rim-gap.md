## Your task: `level-0d-jib-rim-gap`, 0d: from the crane deck to the yard on real input

level-0d-crane-and-to-the-end (merged; read its report and the `probe_crane*` records) made the dock -> crane deck crossing real input (ladder #435, the wire, the rotate), and mapped the deck -> yard remainder, blocked only on the jib rim gap. Find how the game gets Bond from the deck to the yard [D the crane/jib placements, keys and collision; M the classes the route uses — wire, rotate, any script player], fix any port gap, and replace the remaining teleport in `walkreal_0700000d_live::walk_arrival_to_the_c5_door` with real input. Every test in the file green. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-jib-rim-gap`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-30T22:51Z

---
## Report (done, 2026-10-30T23:34Z)

Closed by the driver: the report is on `s/level-0d-jib-rim-gap` (docs/reports/level-0d-jib-rim-gap.md); the worker didn't run finish.sh.
