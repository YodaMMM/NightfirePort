## Your task: `mp-bot-traits`, MP bots use their 360-degree vision and health regen traits

`BotRecord::has_360_vision` and `has_hp_regen` in crates/nf-game/src/mp/bots.rs read the bot record's flags, but nothing calls either, so these traits do nothing in a match. The game's record copy lands at the drone's +0xd1c block, and the flag byte is read at +0xac: 0x10 arms a timer at `BOT_init` 0x1249e8:165 (BOT_vars+0x738 = now + FRAME_RATE_INT), and 0x08 feeds `NDrone2_FindOpponent` 0x1424e0:288. Other sites read the same byte: `NDrone2_SetOpponentAimPos` 0x141fe8:146, `NDrone2_DoHitEffects` 0x1458e0:43/49, `NDrone2_DSTATE_BotGlobal` 0x16e060:187/209/317/376 and `Player_DealWithObjHit` 0x194458:127. Read each one, state which bit it tests and what it does, and compare with the port. Port the regen tick from BOT_vars+0x738's reader and the vision arm in the opponent search, plus any other bit the port lacks. List which bots on the Bot Setup pages carry which flag [L ee_snapshot rows are in the file]. Any Rand draw belongs to the shared match stream, and the state hash must stay equal on every machine. Pin through a real match: a bot with the regen bit recovers health at the game's rate and one without does not, and a bot with the vision bit acquires a target behind it where one without does not. Run every mp_*_live binary, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-traits`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

