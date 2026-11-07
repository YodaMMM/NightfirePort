## Your task: `bot-weapon-anim-set`, MP bots hold each weapon with the game's animation set

`BOTWEAP_changeWeapon` 0x12a028 calls `BOTWEAP_getWeaponAnimSet` 0x12b140 and writes the result to Drone+0xDA. `weapon_anim_set` in crates/nf-game/src/mp/botweap.rs ports the switch, but only a unit test calls it. Read 0x12a028 in full and find every reader of +0xDA in the game, then find what a bot's held-weapon animation uses in the port today, and whether a guard's set (single player) comes from the same field. Compare weapon by weapon. Store the set on the real bot change path so each reader gets it, and remove any stand-in. The animation is a visual, so list frames for a render check (one pistol, one rifle, one launcher) under the report's live checks for the main agent. Animation choice must not change any Rand draw or the state hash. Pin through a real match: a bot changes weapon and the set and the anim it plays match the table, for one weapon in each set. Run every mp_*_live binary, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/bot-weapon-anim-set`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T08:16Z
