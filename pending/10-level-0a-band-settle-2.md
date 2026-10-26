## Your task: `level-0a-band-settle-2`, Bond's walk on 0700000a's ring route, with the live answers

Start from `origin/s/level-0a-band-settle` (merge it, then origin/master) and read its report. Its three live checks were run on the real game (PCSX2, 0700000a loaded through the game's own level loader, god mode held) [L 2026-10-26]:
1. **Wall press stop:** Bond held walking west (yaw -1.5708) until pinned against the west hall wall at (-45.956, 1.13, 16.828); a write watchpoint on obj+0x30..0x3b for 9 s caught exactly ONE writer, 33 hits: PC 0x194c70 = `Player_CollisionHandler` 0x1949a8 + 0x2c8. So the handler's tail IS alive and is what stops the walk (your prediction 1). Port that tail (find the decompile line for 0x194c70 and its inputs) in place of the port's stand-in.
2. **The doorway jamb:** placed at (-7.6, 1.12, 16.0) and walked west toward the jamb at x -8.304, Bond was NOT stopped: the game slid him north to z 16.83 and on through (-8.9142, 1.1298, 16.8295 after 40 frames). Walking in from (-8.0, 1.12, 16.29) the same way: frames 5/13/21/28/36 read (-7.754, 1.1402, 16.117), (-7.754, 1.1465, 16.2723), (-7.7878, 1.1285, 16.4685), (-8.2106, 1.1285, 16.8208), (-9.0106, 1.1285, 16.8295); the rest after release (-9.8677, 1.1285, 16.8295). BLData+0xcc read 0.0 at every sample (so the walked height is not a cc change at that offset).
3. **The corner:** placed at (-17.87, 1.12, 4.87), yaw 2.987, stick forward 60 frames, every 5 frames: (-17.9135, 1.12, 4.8866), (-17.9656, 1.12, 4.7502), (-18.062, 1.1199, 4.2986), (-18.067, 1.1189, 3.8144), (-18.0198, 1.1186, 3.4177), (-17.9274, 1.1186, 2.8248), (-17.8504, 1.1186, 2.3308), (-17.7734, 1.1186, 1.8368), (-17.8419, 1.1186, 1.5305), (-17.9967, 1.1186, 1.1415), (-18.0788, 1.1174, 0.5754), (-18.0485, 1.1159, 0.0778); end (-17.9715, 1.1159, -0.4162). y barely moves and the xz path bends (a slide), never a wedge.
Port the collision-handler tail, re-land your experiment (a)'s below-B kill if it is still needed, re-pin the three ignored tests to these [L] numbers (the placement start heights here were set to 1.12 by the probe, so pin the settled values, not the first sample), un-ignore them, and keep walkreal_07000001/05 and every other walkreal test green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

