## Your task: `mp-object-deleted`, bots drop a goal whose objective object is gone

`control_delete_object` 0x133178:20-22 calls `MP_objectBeingDeleted` 0x18ac90 in a match, which clears every mode-object row holding the deleted object and the 64 `MPObjects` slots, then routes msg 0x3D to BotGlobal [M :66-278]. `object_being_deleted` in crates/nf-game/src/mp/mod.rs ports the shape and `get_obj_ext_obj` ports the 0x18bca8 tail, but nothing calls either outside tests, so a bot can keep walking at an object that no longer exists. Read both in full and list every mode row 0x18ac90 clears against the port's stores (flag and bases, GoldenEye, Hill, Uplinks, Demolition, Protection). Find which objects can really be deleted during a match in each scenario, and wire the call from the port's delete path, with `BotGlobalOutcome::GoalCancel` reaching the bots in the game's order, and the 0x40 re-target from the site getter. Remove any stand-in. State must stay equal on every machine online and must not touch the shared Rand stream. Pin through a real match: delete a mode object a bot is heading for and the bot leaves that goal the same tick the game does. If no scenario can delete one in play, say so in the report and keep a unit pin. Run every mp_*_live binary, mp_world_sync_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-object-deleted`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-07T10:41Z
