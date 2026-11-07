## Your task: `mp-fists-skin`, multiplayer fists use the game's multiplayer model

`Player_Init` 0x1910f8:108-113 takes a separate multiplayer arm: `weapon_data._488_4_ = 0x50000b0`, `weapon_data[0x3999] = 2`, and the skin from `MP_skins` by the player's MPSettings row, where the single-player arm uses the `PunchSkin` table and `weapon_data[0x3999] = 1`. The port has the single-player arm only (crates/nf-formats weapon.rs and crates/nf-game/src/player.rs) and no reference to 0x50000b0. Port the multiplayer arm, and find what reads `weapon_data[0x3999]` and the +488 skin so each reader gets the right value in a match. Check 0x50000b0 exists in the multiplayer archives [D]. The state must stay equal on every machine online and must not touch the shared Rand stream. Pin through a real match that a player with fists equipped resolves that model and that single-player levels keep their rows. A model change is a visual, so list the frame a render check should look at under the report's live checks for the main agent. Run every mp_*_live binary, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-fists-skin`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T01:49Z

---
## Report (done, 2026-11-07T02:47Z)

Closed by the driver: the report is on `s/mp-fists-skin` (docs/reports/mp-fists-skin.md); the worker didn't run finish.sh.
