## Your task: `level-0a-office-hatch-7`, 0700000a: the spawn settle without its regressions, then the hall glide, the corner and the office walk

Start from `origin/s/level-0a-office-hatch-6` (NOT merged). Its level-start spawn arrival settle (the 4-tick hold, then the one-tick drop to the 1.0891 rest [L ee_start_0700000a]) turns five tests red on master's tree — each passes without the branch and fails with it:
- `fire_modes3_live::torpedo_is_homing_slow_and_cameraless` ("the torpedo spawned" — a fire in the first ticks),
- `interact_07000009_live::standing_short_leaves_the_armour`,
- `fix_08_collision2_live::the_01_seam_scoped_pass_matches_the_flat_one`,
- `bathhouse_shout_mixer_live::the_bunker_trio_shout_reaches_the_mixer_with_the_ambients_live`,
- `nf-view --lib headless_app_harness::the_crouch_option_picks_the_stand_condition`.

1. For each, find whether the test depends on a first-tick action the game would not allow either (then the test's timing is wrong — fix it with the evidence), or the settle hold swallows something the game's player does do in those ticks (input, firing, the crouch read, the level-start sounds) — then the hold is too broad: narrow it [M]. The hold's gate is still unread ([S]); do not widen it to make tests pass.
2. Then the hall glide (game 1.1147 after the doorway-base climb at x -8.304 [L]), the corner at (-17.87, 4.87) (the game walks through at y 1.127 -> 1.120 [L]) and the main walk `walk_office_floor_qworm_keyboard_hatch` (the corridor mouth): dump the doorway base and corner triangles, log the per-tick capsule contacts and pushes, compare with the slope-blind push [M `Intersect_CylGeom` 0x109da0, `Player_CollisionHandler` 0x1949a8 disasm 0x194c60-0x194c70] and the feet band [M 0x194188], fix [M], un-ignore as they go green.
3. The full suite on the final tree must show none of the five above failing.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox6 at 2026-10-24T16:46Z

---
## Report (done, 2026-10-24T17:37Z)

