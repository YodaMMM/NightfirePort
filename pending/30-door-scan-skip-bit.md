## Your task: `door-scan-skip-bit`, doors skip the actors the game's auto-close scan skips

The door auto-close scan (`Door_Update` 0x1363c8) skips any object whose `obj+0xF0` has bit 0x10 [M 0x136678-0x136684]. Live reads show the 07000001 player (0x890) and drones (0x8B0) have it SET, the 07000005 player (0x880) has it clear [L]. The bit's writer is unported, so `crates/nf-game/src/drone.rs` (~line 17476, the `player_actors`/`actors` lists) passes `false` for every actor, and `crates/nf-game/src/door.rs` (~line 305, `DoorActor`) documents it (door-port-check #2). Find every writer of obj+0xF0 bit 0x10 in ACTION.ELF [M: cite each] and the condition that sets it per level/object class; carry that flag on the port's player and drones and feed it into the actor lists, so a door's auto-close ignores exactly the bodies the game's does. Pin the flag per level with tests ([M] for the writer, [L] values above as the check), plus one door behaviour that changes on 07000001. Run the nf-game door tests and every walkreal binary that crosses doors on 01 and 05.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/door-scan-skip-bit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
