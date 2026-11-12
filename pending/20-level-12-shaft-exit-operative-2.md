## Your task: `level-12-shaft-exit-operative-2`, Phoenix Base reaches the shaft exit on Operative with real damage

Branch from origin/s/los-mask-repins-12 and merge origin/master; read docs/reports/los-mask-repins-12.md on that branch. Door plc 543 is resolved there (the port opens it like the game; the west-stand route is gated to difficulty 1). walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_operative now crosses 543 and dies at t7167 to drone #2 (skin 0x5000008, weapon 0x19) at (-9.1, -4.1, -21.0), 20.8 u away, 57 hits, in the south hall before door plc 545. Re-tune that fight on real input (cover, distance, the level's head-hunting and unseen-cover levers), and check the 57-hit count against DroneWeap_DoBulletAccuracy 0x179740 at 20.8 u - if the port lands rounds the game would miss, fix the port with the address. Never lower damage or health, no top-ups. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000012_live, walkreal_07000013_live and interactables_07000012_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-12-shaft-exit-operative-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
