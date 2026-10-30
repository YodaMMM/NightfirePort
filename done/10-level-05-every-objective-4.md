## Your task: `level-05-every-objective-4`, land the 05 walk extension

`origin/s/level-05-every-objective-3` (its report is on the branch) extends the 05 walk past the crawl exit; merged with origin/master its new `walkreal_07000005_live::the_door_band_exit_chains_to_07000006_climb_parked` FAILS: "the terrace placement stands on the y-49 band [D the band maps], got [21.805223, 50.035336, -68.09826]" (walkreal_07000005_live.rs:1361). Merge origin/master into the branch, find what Bond stands on at y 50.04 there [D the collision at (21.8, -68.1)], and fix the test's spot or the port with evidence. Then continue the remaining legs from its report (the east stairs, door #112's E -> 07000006). Every test in the file green. Run the touched binaries and quote their totals (no full suite needed).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-every-objective-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-30T03:55Z

---
## Report (done, 2026-10-30T04:41Z)

