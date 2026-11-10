## Your task: `level-05-basin-floor-4`, the solid hurt planes land without drawing them or losing their damage

Branch from origin/s/level-05-basin-floor-3 and merge current origin/master (823ff190b). A full suite of master + basin-floor-3 is 4872 passed / 7 failed; fix all seven the game's way:
1. fix_striped_box_census::census_no_marker_brush_drawn_on_any_level and ::striped_marker_brushes_not_drawn_on_07000005: the folded hurt planes are DRAWN (no-draw bit 0x10 marker brushes, striped texture, e.g. at (36.94, 5.x, ...) on 05). The fold must add collision only; the game never draws them [M View_AddObjects 0x1e62d8's 0x8010 gate].
2. interactables_07000011_live::hurt_volumes_sting_on_crossing ("the hurt volume stings for its param-0 damage [D #271 p0 = 6]"), interactables_07000015_live::the_silo_fall_dies_on_the_death_plate ("the shaft drop never began"), interact_07000005_live::the_valley_fall_dies_in_the_hurt_volume ("the rest position sits inside a hurt volume"): with the plane solid, Bond stands on it and the port's per-tick volume contact no longer fires. In the game a solid hurt object damages through the player's object hit [M Player_DealWithObjHit 0x194458:95-103 — obj+0xFF == 0x27 applies Hurt_GetDamage]: port that path so standing on / touching the plane applies the damage the way the game does, and check which hurt objects actually carry collision (a volume with no collision block stays a pass-through trigger).
3. interact_07000006_live::the_well_shaft_is_open_and_its_hurt_volume_kills_on_the_touch: Bond now stands at the well mouth (y 36.11) instead of dropping ~6.2 to the shaft floor. LIVE FACT (PCSX2, 2026-11-10): Bond placed at (31.12, 40.5, 17.26) drops and holds at y 36.0-37.1 and never reaches the shaft floor, so the game DOES stop him at the mouth like the branch does (y 36.11): the test's expected drop is wrong; re-pin it on the mouth stand and the touch damage (god mode hid the damage in the capture, so pin the damage from the decompile).
4. load_scene_equality::the_cut_costs_build_the_same_scene: find what the fold changed in the scene build.
Then run every walkreal_*_live and interactables_*/interact_* binary one level per tool call, plus fix_striped_box_census and load_scene_equality.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-10T15:15Z
