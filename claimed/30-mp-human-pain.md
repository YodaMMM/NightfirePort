## Your task: `mp-human-pain`, players take multiplayer damage by the game's rules

`player_pain_mp` and `bot_pain` in crates/nf-game/src/mp/botweap.rs port `Player_HandlePain` 0x1902c8's multiplayer arm (the `Plr_DMod_Multi` scale, armour absorbing first, health below 1.0 reading 0), but only their unit tests call them. `MpSettings::location_damage` (+0x1BC-area row, [M 0x1902c8:62-78]) has a menu option too. Find how a bullet, a blast and a melee hit actually reach a human and a bot in a match on master, and compare each with `Player_HandlePain` line by line: scale, locational multipliers, the x3 option, armour, the 1.0 floor, and pain types. List in the report which steps the live path has and which it lacks, with addresses. Then route every hit through the ported arm, adding the settings the doc comment says are missing. Damage must stay equal on every machine online and must not touch the shared Rand stream. Pin through a real match: a shot on an armoured player spends armour first, a hit with the location option on scales by body part, and the same shot with it off does not. Run every mp_*_live binary, mp_world_sync_live, the net hash binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-human-pain`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T04:12Z
