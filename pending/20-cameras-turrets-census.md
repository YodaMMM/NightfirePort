## Your task: `cameras-turrets-census`, every security camera and turret on the single-player levels: what the game creates, what the port has

Security cameras and turrets are missing or inert in the port's single-player levels. `GunImp` (placement kind 0x34, `GunImp_Create` 0x131f20, `crates/nf-game/src/gunimp.rs`) is simulated and its base is drawn, but its gun child (the `TurretGuns` table hash, `GunImp_Update` 0x1320d8:162-172) is not. There is no security-camera object in the port at all.

1. **Census [D + M].** For every on-foot SP level (the CAMPAIGN table in `crates/nf-view/src/menu.rs`): list every placement whose creator makes a security camera, a wall/ceiling turret, a gun emplacement or an auto-gun. Name each creator (follow `parsemap_create_dynamic_objects` 0x1d04f0's cases, and the drone templates that are cameras or turrets — `Drone_Create` 0x136be0 / the NDrone2 classes), its update function, what it does (sweep, alert/alarm channel, firing, damage, destruction) and what it draws (the glist, child meshes, the beam/cone). Put the table in `docs/systems/cameras-turrets.md` with every row tagged.
2. **The port side.** For each row: is the object created in the port, does it update, does it alert or fire, can it be destroyed, does its mesh reach the draw lists? Pin each answer with a test (one per class, real level data).
3. **Port the missing simulation** (update, alarm channel, firing, damage, destruction) [M], with tests that fail before the fix. Draw-side work (meshes, child parts, sweep beams) is NOT yours: list it as hand-off rows in your report (what to draw, which hash, which datum or matrix) and leave a `// draw hand-off:` comment at the spot.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cameras-turrets-census`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


