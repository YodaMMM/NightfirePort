## Your task: `mp-bot-backoff`, multiplayer bots back off only where the game's let them

Branch from origin/s/mp-bot-combat-ranges, merge current origin/master, and read its docs/reports/mp-bot-combat-ranges.md. With the game's 4.0 too-close band, mp_bot_objectives_live::demolition_bots_shoot_the_site went red: the site only becomes the bot's opponent inside 4 u [M 0x1424e0:536-538], so the fire state's back-off arm now fires there. The game's NDrone2_CanBackoff 0x1501a0 has two halves the port lacks: the DroneAnim_CanDoAnimState(anim 4) gate and the behind-room NDrone2_MoveTest (the -0.75 local-Z probe, cel-checked [M 0x1501a0:31-77]). Read both in full and port them in the game's order. If the anim gate needs anim data the port lacks, read it from the disc [D]. Pin it through a real bot match, and make the demolition test green. Multiplayer state must stay equal on every machine and must not touch the shared Rand stream. Run every mp_*_live binary and mp_world_sync_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-backoff`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

