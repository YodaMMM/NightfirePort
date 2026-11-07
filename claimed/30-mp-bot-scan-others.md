## Your task: `mp-bot-scan-others`, MP bots choose targets from the game's per-player scan

`BotBrain::scan_others` in crates/nf-game/src/mp/bots.rs ports `BOT_setOtherPlayerInfo` 0x125390 (candidate, same-team and seen bits, distance squared, the +0x770 scan flag), and its comment says it runs once per tick like ControlSTANDARD:164, but nothing calls it on master. Comments in the same file say the +0xB0 block is recomputed from the slots every tick. Find what actually fills `OtherPlayer::flags` and `dist2` each tick, and whether the seen bit comes from the game's `NDrone2_CanSeeObject` line of sight or from something looser. If a live pass already matches 0x125390 in every field, say so in the report with the line numbers and delete the unused copy. If it does not, call the ported pass from the per-tick bot step at the game's point and remove the stand-in. Then check each reader of those bits against the decompile. Pin through a real match: a bot does not pick a target behind a wall, picks one in the open, and never picks a teammate in a team game. Keep the shared Rand stream and the state hash unchanged. Run every mp_*_live binary, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-scan-others`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-07T04:04Z
