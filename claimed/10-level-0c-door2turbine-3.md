## Your task: `level-0c-door2turbine-3`, Chain Reaction Part 1: the band crossing and the rest of the walk

Continue `origin/s/level-0c-door2turbine-2` (merge `origin/master` first; read its report). Its walk is real up to the band east, then RED: the band floor has a void at x 55..65 (z -209.5..-206.0) [D its probe]. (1) Find how the game gets from the band (x 44..53, floor 10.34) to x 67+ [D the disc around x 53..69, z -209..-195: the 10.85 strip at z -206..-205.2, the descending east walkway x 61..69, z -204..-202; any trigger, door, lift or script that bridges it]. If the port's world lacks a connection the disc has, fix it [M/D]. (2) Make the walk green through that crossing on real input. (3) Then blockers 2-5 of the level-0c-every-objective report in order. The walkreal must be green before you push; keep any stand-in only with its exact blocker and evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-door2turbine-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-29T06:52Z
