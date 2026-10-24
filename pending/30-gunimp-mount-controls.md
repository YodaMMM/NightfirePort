## Your task: `gunimp-mount-controls`, the mounted laser gun: controls, beam, lighting

0700000d's mounted laser (gun emplacement #225, `GunImp_*`, `crates/nf-game/src/gunimp.rs`, drawn from posed slots in scene.rs): when mounted the aim controls are reversed, the laser fires with no visible beam, and the gun has no lighting effect.

1. Controls: compare the port's mounted aim with `GunImp_Update` 0x1320d8:124-161 [M — which stick axis drives ext+0x70 pitch / ext+0x74 yaw, the signs, the clamps [-pi/2, +pi/4], the 0.15 easing] and the player's invert-look setting; fix the signs so up/down/left/right turn the gun the way the game does.
2. The beam: the round is weapon 0x67 through `TurretGuns[0xb]` [D]; find what the game draws for it (the laser bolt/beam mesh or sprite, `Bullet_DoTrails` trails, the weapon record's +0x5C/+0x4C meshes) and draw it from the muzzle along the barrel.
3. The lighting: the heat/hot byte (ext+0x90 / +0x9c) and any `Light_Create` / glow the game makes while firing [M] — port it.
Tests per item (draw lists carry the beam at the muzzle, the aim signs), plus headless `--cam` renders named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gunimp-mount-controls`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
