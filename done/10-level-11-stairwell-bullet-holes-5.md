## Your task: `level-11-stairwell-bullet-holes-5`, Phoenix Fire Part 1's stairwell bullet impacts - current state first

Reports on 07000011's stairwell: (10-25) bullet holes hanging in mid-air along the railing level, on a plane in the well and looking up, and invisible walls eating shots; (10-26, after stairwell round 4) "maybe progress?" - not resolved either way. The per-caller bullet/LOS masks [M Collide_Intersect 0x1ebaa8:300-319] are being finished on `s/capsule-cull-and-los-masks-11`. FIRST measure the current state on master AND on that branch: fire from the stairwell spots across, up and down the well and record every impact whose point is not on a drawn surface (the impact's collision face vs the render geometry there). Only if mid-air impacts remain: find which collision faces stop the port's bullet that the game's bullet passes [M Bullet ray mask, Collide_FilterBullets 0x1ece30, the surface byte drops] and fix the bullet path. Report the measurement either way. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-holes-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T00:10Z

---
## Report (done, 2026-10-29T01:35Z)

Closed by the driver: the report is on `s/level-11-stairwell-bullet-holes-5` (docs/reports/level-11-stairwell-bullet-holes-5.md); the worker didn't run finish.sh.
