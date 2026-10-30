## Your task: `guard-opponent-pointer`, who a guard is fighting (+0x170) set the way the game sets it

`crates/nf-game/src/drone.rs` still stands in for the game's opponent pointer (drone +0x170) with "the guard currently sees Bond": the NoOpponent 0x60 return to Attack (~line 8199: "Which writer re-points +0x170 for a team-1 guard is not yet ported [S]") and `NDrone2_DoorGuardHandler` 0x1416d8 (~lines 13516 and 13533: "the port's stand-in: the guard currently sees Bond [S]"). Find every writer of +0x170 for team-1 (enemy) guards in the decompile (the vision/comms path, `NDrone2_SendAttackMessage`, `NDrone2_ChangeToAttackMode`, `HasOpponent` 0x14ad58 and its +0x238 stamp, the 11 abseil finding that the game's +0x170 is Bond every tick for the roping guard [M 0x1424e0:100-120]) and port the pointer as a field the AI reads, so NoOpponent, door guards and every other +0x170 reader follow the game, not sight. Earlier work on the same rule: the queue's done/30-combat-sight-loss-exit.md report and branch `origin/s/combat-sight-loss-exit` - read them first and say what of it is on master. Pin with real-input tests on a level with door guards (07000014 or 07000007) and a guard that loses sight of Bond. Run the FULL suite and quote its total line; read every red (guard behaviour touches every level walk).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-opponent-pointer`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T09:17Z
