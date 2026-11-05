## Your task: `mp-site-destruction-blast`, the Demolition and Protection site blows up when it falls

`MP_DemolitionProtectionUpdate` 0x1870a0 (decomp lines 125-160) handles the site's death: MPSound 0x14c, the team and player score changes, `SP_UnPause`, `Explode_Create(5.0, 5.0, 500.0, ..., script 0x600004f, ...)` at obj+0x80, the status sprite text (labels 0x200002e/0x200002f), `MPGame+0x188 = 6`, then `MP_sendTeamBotMessage(0x41)`. crates/nf-game/src/mp/rules.rs has `site_update` and `damage_site`, but no 0x600004f reference exists under mp/, and docs/reports/fx-gas-and-blasts.md reports the blast as not ported. Read the whole function and walk it line by line against rules.rs: state which steps the port has and which it lacks, with addresses, then port the missing ones. The blast must go through the shared pending-blast path so it hurts bodies and bots, and it must stay in step on every machine online. Pin it through a real Demolition match: the site falls, the blast fires at its position, nearby players take the damage, the status text shows and the round ends. Do the same for Protection. Run every mp_*_live binary, mp_world_sync_live, the net hash binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-site-destruction-blast`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

