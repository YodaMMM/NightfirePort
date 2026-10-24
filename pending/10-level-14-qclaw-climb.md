## Your task: `level-14-qclaw-climb`, 07000014: the Q-Claw climb at the vent mouths, then the infiltration walk

Master walks the real east duct route green (`walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot`). Read `docs/reports/level-14-trench-landing.md` on `origin/s/level-14-trench-landing` (merged; the report is not on master). Next on the route: the vent mouths at z 42.16 with the grapple points #626/#657 at (45.30 / 40.78, 4.79, 42.86).

1. The climb leg with real input: aim the Q-Claw at the grapple point, fire, reel [M the grapple path the port already has, `Grapple_Create` 0x198298 / the wire ride], E-open the mask-4 lid, out onto the mezzanine.
2. Rewrite leg 1a of `walk_the_infiltration_to_the_delta_exit` onto this east route; run its legs 2+ unchanged and un-ignore it when it walks to the delta exit. Fix any port divergence found on the way [M].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-qclaw-climb`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


