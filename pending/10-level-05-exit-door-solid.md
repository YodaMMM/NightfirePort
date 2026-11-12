## Your task: `level-05-exit-door-solid`, The Exchange's castle exit door stops Bond like the game

Branch from origin/master. On 07000005 the LoadLevel trigger placement #118 (kind 0xE8, glist Door_Gate8: 240 tris, glist flags 0x80000, x 27.4-32.6, y 36.0-41.7, z 38.0-38.4 [D]) spans the castle exit. In the game it is SOLID to Bond: [L PCSX2-gdb 2026-11-12, level loaded by the game's own loader, Bond placed at (30.0, 37.2, 41.5) facing -z, left stick held 360 frames] he stops at z 38.89 and stays there; nothing is drawn there (collision only). In the port (master 49434f96c, headless: Bond::new at (30, 36.5, 47) yaw PI, forward 1.0) he walks straight through z 38 and slides down the outer slope to the valley floor at y -9.19 - a player fell out of the playable area this way. Find why the 0xE8 glist is not in the port's collision world (instances / CollisionWorld::build_scoped / the trigger build), what the game does with a LoadLevel trigger's glist [M Trigger_LoadLevelCreate 0x1b2950, Trigger_Create 0x1b1720, the cel link], and make it solid the game's way - including what that means for the other 0xE8 placements (#112 Locked_Door_Top_I on 05, and every other level's 0xE8: list them with their glist flags). Starting point: crates/nf-view/src/scene.rs's 0xE8 loop (the `p.flags & 0xff == 0xe8` pass, ~line 10885) adds these glists to the DRAW groups only; they never reach the instance list `CollisionWorld::build_scoped` gets (~line 11003). Check the game's Trigger_Create collision link (obj+0xD8 / the cel link) before adding them, and check the 'Load Point' 0xE8 meshes on 01/03/12/13 (touch-volume exits) separately - a solid surface must not stop their touch from firing. Pin it with a headless walk-into test at the coordinates above (Bond must stop near z 38.9) and make sure the truck exit still ends the level (truck_exit_guard_live if present, walkreal_07000005_live). Run walkreal_07000005_live, interactables_07000005_live, level_carry_live and the walk binaries of every level whose 0xE8 glist you change.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-exit-door-solid`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
