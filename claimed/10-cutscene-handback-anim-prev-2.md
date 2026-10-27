## Your task: `cutscene-handback-anim-prev-2`, re-derive Double Cross Part 2's tests from the game's real start

Start from `origin/s/cutscene-handback-anim-prev` (merge origin/master). Its event-6 anim snapshot is CONFIRMED by a live read: on a fresh 07000002 load the game starts Bond at (-27.172, 1.040, 29.796) and, when the intro scene releases him at frame ~1531, places him at (-27.504, 1.131, 32.348), settling to y 1.055 [L 2026-10-27 PCSX2, PLAYERS obj+0x30] - the branch's resolution (-27.50, 1.16, 32.36). On master Bond stays at the old start, and the 02 tests were calibrated on that wrong spot. With the branch these go red: fix_swim_live (4 tests), civilian_no_shoot_live `the_no_shoot_crosshair_keys_on_the_held_weapon`, walkreal_07000002_live `walk_the_castle_all_objectives_to_the_exit`, ladder_underwater_02_live, fix_ladder2_live, fix_ladder_water_live, ladder_02_live, bath_hatch_02_live, bathhouse_voices_live, bathhouse_shout_mixer_live, speech_facial_fallback_live, fix_swim_entry_live. Re-derive each test's route/expectations from the [L] start (cite it), never weakening a decompile-backed assert; if a failure is not explained by the start, find and fix the real cause. Also fold in `10-cutscene-handback-sp-path`'s item (1) if the SP path still drops the object. Full suite green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-anim-prev-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-27T23:21Z
