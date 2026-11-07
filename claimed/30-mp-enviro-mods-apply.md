## Your task: `mp-enviro-mods-apply`, every Enviro-Mods row changes the match

The Enviro-Mods page (`P_MPENVIROMODS` 0x210030) writes `mp_envmods` in crates/nf-view/src/menu.rs, and the rows' save and summary use it, but the comment near nf-view/src/lib.rs:1748 says the match settings carry no such word, so `gunimps` is cleared for every match. `mp_enviro_value_label` also has no caller. Find what each row does in the game and whether it reaches a match on master: +0x1C0 view distance, +0x1B8 (`GunImp_Create` 0x131f20:52-58 mounted guns), +0x1D4 (the locked row), +0x1D0 (the grapple, see mp/equip.rs) and +0x1CC vehicles. Read every reader of those five words in the decompile and list them in the report, with what the port does for each. Add the words to `MpSettings`, carry them from the menu into the match on host and clients, and apply each reader. Mounted guns must stay in step on every machine, which was the reason they were cleared, so prove that before turning them on. Vehicles: report only what the row gates, and leave car creation alone. Use `mp_enviro_value_label` for the page's rows if the page's text does not already match, and compare each row's label with the decompile's node lists. State must stay equal on every machine online and must not touch the shared Rand stream. Pin through real matches, row by row: on and off give the game's difference. Run every mp_*_live binary, mp_world_sync_live, the menu binaries that walk Enviro-Mods and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-enviro-mods-apply`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T12:25Z
