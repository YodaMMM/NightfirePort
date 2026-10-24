## Your task: `camera-beam-satchel-draw`, the security cameras' sight beam, the satchel charges, the turret wreck

Master draws the security cameras' body and mount and the gun emplacement's gun from posed slots (`crates/nf-view/src/scene.rs`, `lock_slot` and the slot loops after the breakables). Still undrawn (`docs/systems/cameras-turrets.md` §5, the `// draw hand-off:` comments):

1. **The camera sight beam.** The lens proxy (param 0, 0x2000360-class) is posed at the camera's matrix every tick and stretched to the beam's hit length (`Sensor_SetBeam` 0x1af398:57-75: lens obj+0xE8 = range, or the `Collide_RayIntersect` hit distance). Find how the draw applies obj+0xE8 to that object [M — the view/draw path for objects carrying +0xE8] and draw it the same way.
2. **The 4a satchel charges.** The charges are drone bullets of the 0x3b family (`crates/nf-game/src/bullet.rs`, the fuse fields). Check whether the projectile-mesh slots already draw their mesh (weapon+0x5C, 0x20007b3) in flight and once landed on the lift cab; if not, draw them. Then the two countdown digit sprites (`SatchelDigits`, `Bullet_DoTrails` 0x12d890:485-553) and the per-digit beep 0x447 [M 0x12e290-0x12e2b4].
3. **The GT turret wreck** (glist 0x2000039 puff + the destroy burst, the events in `drone.rs` at the GT destroy arm).

Each item: a test that the draw lists carry the piece at the right pose (fails before), and a headless `--shot` (or `--cam`) render per item named in the report with its path, so the result can be compared with the game.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/camera-beam-satchel-draw`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox3 at 2026-10-24T18:51Z

---
## Report (done, 2026-10-24T20:09Z)

