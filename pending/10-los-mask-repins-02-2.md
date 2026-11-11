## Your task: `los-mask-repins-02-2`, 07000002 can be walked with every objective on 00 Agent without the civilian being shot

Branch from origin/s/los-mask-repins-02, merge current origin/master, and read its docs/reports/los-mask-repins-02.md, "The traced cause" and "What is left (the fix)". The d3 castle walk fails because west-end rifle guards #10 and #11 (placements 452/453) shoot the scared civilian #22 while Bond crosses the upper-east stretch. First settle item 1: are placements 452/453 d3-only (Drone_Create's minimum-difficulty gate [M 0x136be0:49]), or alerted late on the d3 timeline? Read the placement bytes [D]. Then take its levers in order: beat the alert, a lane-side clear before the descent (probe the route first), then the silenced west-wing kills. The stair-top stand is a measured dead end. Helpers stay gated to 07000002 and the difficulty, so other walks stay byte-identical. Remove the temporary diagnostics before the final commit. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000002_live binary and interactables_07000002_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-repins-02-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

