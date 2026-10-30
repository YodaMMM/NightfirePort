## Your task: `level-0a-walk-clip-flip`, make the move-clip the walk's wall response

level-0a-corner-walk-speed (merged) found the game's walk wall response is the move CLIP (the tick's motion clipped at the contact plane, tangential part carried; the spine rests ~0.011 inside the plane [L]), not a radius push-out [M Intersect_CylGeom 0x109da0:420-470 gates kill the push on a vertical spine; Player_CollisionHandler 0x1949a8]. The port has it env-gated (`capsule_tri_clip`, world.rs ~564-639, NF_WALK_CLIP=1): the 0a corner crosses in 76 ticks (game ~60) but three tests in walkreal_0700000a_live go red under it: the_pick_ref_accumulator_tracks_the_game_ref, walk_office_floor_qworm_keyboard_hatch, walk_office_floor_to_the_security_keyboard. Read that report. Reconcile those three (fix the port or the test, with cites), make the clip the default (remove the env gate), and run EVERY walkreal_* binary plus the collision/player crate tests; quote every total. Report any walk that changes, with the leg and the position delta.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-walk-clip-flip`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

