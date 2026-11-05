## Your task: `mp-bot-body-sounds`, MP bots make their jump and head-bump sounds

A bot's movement core queues the same jump launch and head-bump sounds as a human's (`Player_HandleJump` 0x1916b0 through `Bond::sfx_out`), but crates/nf-game/src/mp/bots.rs:3243 clears them every tick, while scene.rs drains the remote humans' bodies into `mp_fx.body_sounds`. Read docs/reports/mp-bot-voices.md first. Confirm from the decompile that bots reach these sounds [M], then drain the bots' queue the same way so each plays at the bot's position, on every machine, with no change to the shared Rand stream or the state hash. If the decompile shows a bot does not reach one of them, keep that one silent and cite the address. Pin it through a real match with a bot that jumps. Run the mp_*_live binaries, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-body-sounds`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-05T19:00Z
