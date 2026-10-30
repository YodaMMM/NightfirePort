## Your task: `player-trigger-contact`, floor plates the player stands on must fire like the game

The player's trigger toucher in `crates/nf-game/src/drone.rs` (~line 16944, `Toucher::player([pos.x, pos.y - 0.6, pos.z])`) is a point stand-in at a measured reach, not the game's contact. Known misses: 0700004a's reveal plate #277 on the platform's vertical ride (walkreal_0700004a_live walks the platform edge to make it fire), 07000005's spawn reveal plate ch 0x45 (check whether the 0.6 reach fixed it - the comment in walkreal_07000005_live.rs ~line 49 may be stale), and 07000002's bunker plate 0x80 below the floor. Port the game's real rule: how the player's collision body touches trigger volumes each tick [M control_movement_object_handler 0x1336f8:54-68 -> Collide_Update 0x1ea398:160-215 contacts -> Trigger_Hit 0x1b1fc8:27-47], so standing, riding and crouching all fire the plates the game fires and no others. Pin each known case with a real-input test (fires where the game fires, stays off where it does not, e.g. the 05 ch-0x70 truck park plate until the crouch [D fix-truck trace]), and remove the workaround walks that exist only to dodge the miss. Run the FULL suite and quote its total line; this touches every level, so read every red.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/player-trigger-contact`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

