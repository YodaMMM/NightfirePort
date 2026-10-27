## Your task: `cutscene-handback-sp-path`, the anim-actor hand-back in real play

`cutscene-handback-anim-prev` (on master) ported the game's event-6 snapshot of a kind-2 (anim) previous stream [M Script_EventHandler 0x1d94c0:73-84, :125-134] and the census asserts it, but in real play two gaps remain (its report's What's left): (1) on the SP runtime path (`Scripts::update_player`) the kind-2 stream's object slot is None at event time (`NF_SCRIPTLOG`: 0x06000084 `prev kind Some(2) -> NO entity`), so the real hand-back still falls back to the player's matrix - find which SP-path kill/freeslot clears it and match the game; (2) the runtime `Data` lacks the cut-scene packs' anim sets (`078/079/07A-000NN` [D]), so `anim_obj_matrix` returns None for their sequences - merge them the way the census's `AnimWorld::for_level` does [M LoadableLoad 0x1d69e8]. Real-path test: play 07000002's SP #399/#400 and 07000003's #475 scenes and assert Bond's position after each matches the census resolution (on a floor).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-sp-path`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

