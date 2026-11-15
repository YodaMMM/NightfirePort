## Your task: `real-health-00agent-08-bleed-6`, 07000008's 00 Agent walk wins the Hind fight

Branch from current origin/master.

Master 4f3802517: the d3 Hind fight lands three rockets in its opening passes (the Hind 287.7 of 598) and dies to the Hind missile blasts at t30165. Measured this round: after fight t1400 no floor spot in the car has a clear flight line to the Hind (21x21 grid, 44 samples) while its missiles still hit (Explode_Propagate is a sphere sweep, no line of sight - the game hits through walls too); the car's 0x2c boxes at (112.0, 18.0, -179.4) refill the rocket pool after a reload, so ammo is not the limit; wider/narrower/no missile dodges, standing, the sisters anchor (111.09, -179.72) and step-outs on cannon hits were all worse. Goal: survive the out-of-view stretch until the Hind circuit returns and land three more rockets (the walk's 00 Agent test must pass). Start from the per-gate tally and the visibility probe described in the ignore text.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-08-bleed-6`.
