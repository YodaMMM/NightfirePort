## Your task: `mp-bot-voices`, MP bots grunt when hit and call out when they die

Bots in a multiplayer match are silent when hurt or killed. `Drones::bot_handle_pain` (drone.rs) and `BotBrain::bot_sound_effect` (mp/bots.rs) hold the game's line pools but neither has a production caller, and `body.sfx_out` on a bot is only cleared. The doc comment above `bot_handle_pain` still says the port is single-player, which is out of date: fix it. Read `BOT_handlePain` 0x125ed8, `BOT_soundEffect` 0x126618 and their callers (DealWithObjHit 0x13fff0:336, BotDeathByExplosion 0x1755a8:15, BotDeathAnim 0x175508:13), and row 2 of "Left as a table" in docs/reports/audit-voice-triggers.md on origin/s/audit-voice-triggers. Wire both into the MP bot damage and death flow in scene.rs's `step_mp_bots` and mp/bots.rs, using each path's own pool and gates. The two existing readings disagree on which ids are pain and which are death: `bot_handle_pain`'s comment has pain 0x14d.. and death 0x154.., the audit row has them the other way round. Settle it from the disassembly and report the address. Bot sound must stay in step on every machine: any Rand draw belongs to the shared match stream, and online the state hash must stay equal across machines. Pin it through a real match: a bot takes a hit and the pain line plays, a bot dies and the death line plays. Run mp_world_sync_live, every mp_*_live binary and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-voices`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-05T04:43Z
