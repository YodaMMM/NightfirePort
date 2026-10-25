## Your task: `level-4a-satchels-lift`, 0700004a: the satchel charges on the lift ride, as the game does them

Spots from real play on 0700004a (Bond root x, y, z, yaw, pitch; frame since load):
- (3.61, -54.06, -2.58, -1.486, -0.662) frame 1979 — most of the frame black: only a hazard-striped strip, a column and a green case draw;
- (4.37, -79.29, -1.85, -1.551, -0.605) frame 3024 and (4.37, -83.87, -1.85, -1.461, -0.613) frame 3177 — riding down: a satchel on the cab floor drawn full-bright (unlit, like the old ghost guards), its display showing no digits, the laser watch beam hitting it with no effect.

In real play the satchels on the descending lift look frozen in place and brightly textured, the countdown text never shows, and the laser watch does nothing to them. Master has the timed-satchel path (Bullet_init fuse block, DoTrails fuse + laser burn, landed charges ride the cab, the countdown digits draw — camera_beam_satchel_wreck_live / the 4a lift tests), so something on the real path differs from what the tests drive.

1. Drive the real level from its start with real input to the lift ride and log each satchel per tick (who throws it, where it lands, its parent/cab link, its fuse, its draw: mesh, lighting/ambient path, the digit sprites). Compare with the game [M — `Bullet_init` 0x12d890:418-443 the satchel block, `Bullet_CollisionHandler` 0x12ce80:256-266 the stick, `Bullet_DoTrails` the fuse + laser burn, the lighting the satchel mesh gets (its glist flags and the level's light list), the digit draw].
2. Fix every difference [M]: the charge lands and moves with the cab, is lit like the cab around it, shows its counting digits, and the laser watch does what the game's laser does to it.
3. The black frame at (3.61, -54.06, -2.58): find what is missing from the draw there (culling, a missing cel/room, the cab interior) [D + M] and fix it.
4. Real-input tests from the level start: a thrown satchel lands on the cab, rides it, shows its digits and counts down; the laser hits it with the game's result; the frame at the first spot draws the room. Headless renders of the three spots named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-satchels-lift`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
