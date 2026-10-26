## Your task: `level-11-rope-guard-hitbox`, rounds pass through the roping guards on 07000011

On 07000011 (Phoenix Fire, the stairwell) shots at a guard on the rope go through him: the crosshair sits on the roping guard and the bullet holes land on the wall behind; spent casings also hang frozen in mid-air there. Master now has the room-scoped bullet pick (`segment_bullet_scoped`) and the abseil damage arms (Init/Slide/Hang case 8 [M 0x169610:48-50, 0x169730:68-70, 0x169940:50-52]). Find why a round aimed at the hanging guard's body misses his hit boxes: compare the port's per-bone boxes for the abseil states (the rope pose, `DroneFunc_HandleImpact`, the bone hit test `Collide_Jointy` 0x1ec180) with the game's [M], from the stairwell landings at (14.11, 34.03, 0.26) and the east connector looking at p258 (18.895, 46.516, 1.434) on ch 0x13. Also the casings: shell casings ejected there must fall and settle on the floor like elsewhere (`Bullet_DoTrails`/casing physics [M]). Tests: mouse-look rounds aimed at the hanging guard's chest hit him (not the wall) in every abseil state; casings land.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-rope-guard-hitbox`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

