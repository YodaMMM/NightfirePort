## Your task: `level-06-rook-door`, 07000006: Rook gets stuck on a door in real play

In real play on 07000006 Rook still ends up stuck on a door. Earlier rounds (rook-door-06, -2, -3: `rook_door_06_live.rs`, `rook_door_06_walk_live.rs`, `fix_06_driver_live.rs`, the TruckDriverMission state 0x35) fixed the scripted pieces the tests drive, but not the real-path behaviour.

1. Drive the real level from its start with real input along the player's route until Rook's scripted walk, and log Rook's state, position and the door's state per tick; find where he stops and why — the door's open/close vs his path, his DSTATE_OpenDoor (0x81) use, the nav route through the doorway, a collision with Bond or the door frame [M — the drone door-open path, `Door_Activate` 0x135be0, the drone route].
2. Fix the cause [M], with a real-path test that Rook passes the door and reaches his scripted destination; keep the existing rook_* tests green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-rook-door`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
