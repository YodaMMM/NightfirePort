## Your task: `level-4a-every-objective-2`, Phoenix Fire lift ride: firing across the shaft at the group-2 garrison

Read the level-4a-every-objective report (on master). The garrison now releases for the rider (the 3-D spawner screen), but from the ride line the port has 0 of 2400 ticks with a clear eye-to-drone bullet ray to the group-2 spots (behind the shaft core at x about -4.3). Find whether the game's rider can hit them [M Collide_LineOfSight / the bullet pick's cel walk - the shaft core may be portal-crossable or not in the bullet's candidate set; L a fresh-boot PCSX2 ride if needed]. Fix the port's bullet/LOS path if it differs; then the ride's fire leg on real input, un-ignoring the parked test. If the game also cannot hit them, pin that with the evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T18:39Z
