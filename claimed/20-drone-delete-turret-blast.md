## Your task: `drone-delete-turret-blast`, a guard's deployed minigun blows up when the guard is deleted

`Drone_Delete` 0x136ce8 (decomp lines ~90-96) walks the object list: for each class-0x36 turret whose ext+0xb8 owner is the drone being deleted, it sets obj+0xfe |= 1 and calls `Explode_Create(5.0, 5.0, 50.0, obj, obj+0x30, 0x2d6210, 0x6000052, 0xff, 0xff, 100, 0)`. Guards own such a turret through `GT_DeployMiniGun` 0x181560 from `DroneWeap_FireWeapon` 0x178f68:123 (the weapon-datum char 'R' deploy). crates/nf-game/src/gt.rs has the MP wreck and sibling sweep with the same blast and `GT_PlayerHasDied`, but no port code cites this arm of `Drone_Delete`. Read the whole function and the deploy path. First establish whether the port's guards deploy the turret at all, with the owner stored as the drone, and which placements carry an 'R' weapon [D]; if none does on disc, say so in the report and keep a unit pin. Otherwise add the arm at the point the port deletes a drone (delete, not death: check what the game's deletion timing is). The blast goes through the shared pending-blast path so it hurts bodies. Pin it through the real path: a guard deploys, is deleted, the turret goes with the blast at its position; a turret owned by another object stays. Run the gt tests, every binary that uses `GTurret`, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-delete-turret-blast`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-06T00:18Z
