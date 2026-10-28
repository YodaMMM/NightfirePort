## Your task: `cutscene-handback-anim-prev-3`, land the kind-2 hand-back with every test green

The event-6 hand-back from a kind-2 (anim) previous stream [M Script_EventHandler 0x1d94c0:73-84, :125-134] is right: the game's fresh 07000002 boot releases Bond at about frame 1531 to (-27.504, 1.131, 32.348) [L PCSX2 2026-10-27, PLAYERS obj+0x30]. Start from `origin/s/cutscene-handback-anim-prev-2` and merge `origin/s/cutscene-handback-sp-path` (the pack-anim loader sweep + handback_sp_path_live) and `origin/master` (which now carries the 07000014 vent-drop fallback for scene 0x06000833 in nis.rs - fold that scoped [S] arm into the general kind-2 rule if the general rule produces the same hand-back, otherwise keep it and say why). Remaining reds named by the -2 report: walkreal_07000002_live `walk_the_castle_all_objectives_to_the_exit` (the Mayhew-computer shot from the raised floor), civilian_no_shoot_live `the_no_shoot_crosshair_keys_on_the_held_weapon` (the skip-path hand-back pose), bathhouse_voices_live (2) and bathhouse_shout_mixer_live (1). For each: find what the game does at that moment [M] and fix the port, or re-derive the test's own placements/routes from the game's real start with the cite (never weaken a decompile-backed assert). Then run the FULL suite (tools/test/suite.py) on the merged tree and quote its total line; every red must be accounted for.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-anim-prev-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-28T04:44Z
