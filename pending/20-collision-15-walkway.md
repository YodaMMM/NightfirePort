## Your task: `collision-15-walkway`, Bond's walk step at the 07000015 ladder walkway matches the game

Branch from current origin/master. At the 07000015 south walkway beside the dead ladder (x 84.53 plane), Bond placed at (85.2, 40.5, -2.4) yaw -pi/2 with no input behaves differently in the port and in the game:

- Game [L PCSX2, fresh boot of 07000015, Bond placed after the intro released him, 3 runs]: he stays on the walkway. x 85.2..85.7, z -2.57..-2.68, y alternates 39.4..40.34 frame to frame and never rises above 40.34. Per frame the game runs ONE `Intersect_CylGeom` 0x109da0 for Bond. On contact frames DeltaP is about (0.013, -0.11, -0.04) (a down push). On the frames between there is no contact, and `control_handle_cel_change` 0x133288 writes the position (the out-of-cel recovery, 77 of 237 frames). Traced spines (A, B, game DeltaP), r 0.55:
  (85.5585,40.1362,-2.6238)/(85.5585,39.3764,-2.6238) -> (0.0125,-0.1165,-0.0435);
  (85.571,39.5406,-2.6673)/(85.571,38.7809,-2.6673) -> 0;
  (85.571,40.1307,-2.6238)/(85.571,39.371,-2.6238) -> (0.0127,-0.1096,-0.0414);
  (85.5979,40.1198,-2.6238)/(85.5979,39.3601,-2.6238) -> (0,-0.0109,-0.033).
  The first frames after the placement: game pos f0..f5 (85.2,40.497,-2.4), (85.201,40.493,-2.401), (85.075,40.452,-2.749), (85.096,40.454,-2.745), (85.109,40.463,-2.749), (85.317,40.591,-2.778); DeltaP f0..f4 (-0.1435,-0.0344,-0.3526), (-0.0072,-0.0023,-0.0015), (-0.0195,-0.0086,-0.0037), (0.176,0.1038,-0.0227), (0.1057,0.0836,-0.1031). In those first frames the game's spine is NOT vertical (A-B about (-0.15, 0.74, -0.15) at f0, growing): `Player_Collision` 0x194df0 builds A/B along `Mat_GetUp(obj+0x90)`, the body matrix up.
- Port (master): the same placement is pulled to (84.97, 40.43, -2.70) at tick 1, sinks to y 39.28 by tick 30, and ends at (84.85, 39.70, -1.23) on the north ledge strip by tick 60.
- `World::capsule_push_scoped(cel 123, A, B, 0.55)` already returns the game's DeltaP for the four traced spines above to 1e-4. The triangle push is right. The mismatch is in the step around it.

Second data set, the 16 ring kerb [L PCSX2 fresh boot of 07000016, Bond walking west along the kerb foot]: the game's spine is vertical (obj+0x90 up = (0,1,0)), Bond holds y 6.8702, z -74.97519 exactly every frame from x -4.57 westward, and every `Intersect_CylGeom` for Bond returns DeltaP 0. The 70 distinct traced spines A=(x, 7.1452, -74.97519), B=(x, 6.38548, -74.97519), x from -4.668 west in 0.1 steps, through the port's `capsule_push_scoped` also give 0 on master. So at the kerb too the push function agrees; the port's drift starts in the placement settle (the port rests 0.006 lower and 0.002 further south than the game, see ring16_kerb_live.rs's header) and the walk then meets a contact the game never reaches. Find the settle difference.

Find which part of the port's tick differs from the game's at this spot (candidates: the spine axis, world up versus the body matrix up from 0x194df0; the out-of-cel recovery 0x133288:186-231 and which tick it fires on; the feet point 0x194188; gravity/velocity order in 0x194df0), fix it to match the decompile, and pin the game's per-frame curve above in a new live test (crates/nf-view/tests, 07000015, placement as above, frames 0..120: never above 40.34 + 0.01, x stays 85.0..85.8, z stays -2.9..-2.4). Then check crates/nf-view/tests/ring16_kerb_live.rs (ignored on master: Bond slides off the 16 ring kerb foot while the game holds him), which may share the cause. A toward-zero (chopped) float model of the cylinder push fixes ring16 but breaks 64 other tests (pickup walk-overs on 02/03/09/0b/0d/08, 08 slab walls, this 15 spot), so do NOT take that route.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/collision-15-walkway`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
