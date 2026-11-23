# s/drive-integrate-1121: four driving branches on one tree

Base: origin/master 1434d9b29. One commit per branch, each squashed to its own changes:

| # | Commit | From | What |
|---|---|---|---|
| 1 | 7f8248d08 | f2595d91f | the player car carries its model's hull points (a tipped car is righted after 30 ticks) |
| 2 | ffcd7b57e | s/drive-difficulty f065c5320 | difficulty plumbing: `nf_game::ptp` helpers, `load_driving_scene_at`, `DrivingState::difficulty`, MSet from the `Map` window, the `paris_hench` swap, MISC.VIV fallback |
| 3 | b84d022ce | s/drive-mis01-early-lose 626d03bea (own commit only) | spline rate 0.5, the path velocity, no body step while the spline owns the car, `ESetNextAISpline` |
| 4 | 2fe370491 | s/drive-hostile-facts-2 7627d7eab (own commit only) | enemy cars in the trigger walk as kind 2, traffic as kind 3, element facts, the speed condition reads the running car, rule / trigger logs drained |

Items 1 and 2 applied with no conflict. s/drive-mis01-early-lose's base carried an older copy of
master's traffic land pass (d16f06ce6, same change as master's 242b4b7c0) and of item 1
(282cf9237, identical to f2595d91f); only its own commit was taken.

## Conflicts and resolutions

### Item 3 onto item 2 (crates/nf-view/src/driving.rs, crates/nf-view/src/lib.rs)

Both branches added `load_driving_scene_at(iso, viv, game_difficulty)`, a `DrivingState::difficulty`
field and the same 1 -> 0, 2 -> 1, 3/4 -> 2 remap [M 0x174bf0:70-81].

- `DrivingState::difficulty`: kept item 2's `u32` (mission +0x3b8, fed to the hostiles, the MSet
  factors and the start health). Item 3's `i32` field and its casts were dropped; the script facts
  take `self.difficulty as i32` (item 2's line).
- The mapping: kept `nf_game::ptp::driving_difficulty` (item 2). Item 3's
  `nf_view::driving::drive_difficulty` was removed. Both map the same values, other values pass
  through in both, so no reader sees a different number.
- `BOOT_DIFFICULTY` (item 3, `GameState+0x28` = 2 [M 0x1c7e78:25]) kept: `load_driving_scene` now
  calls `load_driving_scene_at(.., BOOT_DIFFICULTY)` in place of item 2's literal 2.
- `load_driving_scene_at` body, the HUD start health, the state constructor, the unit-test
  constructor: item 2's side (it also carries master's `hostiles`, which item 3's base lacked).
- `lib.rs` front-end call: both made the same call; kept item 2's comment.
- `driving_mis01_opening_live::drive_difficulty_maps_the_front_end_pick` now tests
  `nf_game::ptp::driving_difficulty` with `BOOT_DIFFICULTY`.
- Item 3's non-conflicting hunks (the spline skip in `step`, the velocity comment) applied as is.

### Item 4 onto items 1-3 (crates/nf-view/src/driving.rs)

Three textual conflicts, all side-by-side additions: the struct fields (`difficulty` from item 2,
`rule_log` / `trigger_log` from item 4), the scene constructor and the unit-test constructor.
Kept both sides in each.

Semantic conflict: item 4 drains `DriveScript::take_fired` into `DrivingState::trigger_log` every
tick, so item 3's MIS01 test (which called `take_fired` after each step) saw nothing. The test now
reads the triggers new to `trigger_log` each tick (first fire per trigger, the same first tick the
old test used).

Semantic conflict: with item 4's enemy cars in the walk, item 3's
`operative_flies_the_whole_phase_with_no_lose` failed: the Operative run with no input is lost at
tick 2912. Traced on the disc data [D MIS01 `Map `]: an enemy car (walk slot 10, kind 2) runs the
bit-3 trigger T730 (flags 0x40a) on tick 2796: `ECallStage(35)`, `EKillAIElement`,
`ESetAutoDriveCamOffMovie`, `ELetterBoxOn`, `EPlayCameraAnim`, `ESetTimer(5, 2.0)`,
`EStartTimer(5, down)`; stage 8's R20 (`SRuleTimer` timer 5 below 0.1) runs `EPlayerLose` 116
ticks later. Neither branch alone showed it (item 3 had no enemy cars in the walk; item 4 lost at
644 on Agent first and had no Operative long run). Resolution:
- the flight test (rate, hover track via 742, velocity) runs with `hostiles = None`, the state
  item 3 tested;
- new `operative_with_the_pursuers_left_alone_fails_on_t730s_timer`: 644 fires, R5 never runs, T730
  is run by a non-player car after 644, the lose is R20 on timer 5, 110..=120 ticks after T730;
- docs/systems/driving-mis01-opening.md section 8a records it.

Docs: no conflicts. The one removed line (driving-missions.md, the `el` list open item) is item 4's
own settled replacement; no line of items 1-3 was removed.

## Behaviour versus the individual branches

- Against s/drive-mis01-early-lose: Operative MIS01 with no input now fails at tick 2912 (T730 /
  R20) instead of flying the whole phase, because the pursuers run their route triggers. Agent
  still fails on 644 at the same tick. The MSet start health and enemy health factors of item 2
  apply on its runs.
- Against s/drive-hostile-facts-2: the helicopter reaches 644 at tick 849 (was 425 at rate 1.0)
  and the drive runs at the picked difficulty with the MSet factors; its MIS01 kill test (R105 on
  tick 40, no lose through 700) and RACE test pass unchanged.
- Against s/drive-difficulty: none beyond items 3 and 4's own changes.

## Tests (NF_ISO set, NF_SILENT=1, --release)

- nf-view `driving_*` / `drive_*` integration binaries: 22 binaries, 30 passed, 0 failed.
- nf-game `drive*` / `driving*` integration binaries: 40 binaries, 293 passed, 0 failed.
- nf-game lib: 1058 passed, 4 ignored, 0 failed.
- nf-view lib `driving::`: 3 passed.
- nf-extract and nf-android build.

## Unproven

- Whether the game's pursuers reach T730 on Operative with no input, and when: not checked against
  a live capture; it depends on how the port's enemy cars follow their roads.
- Everything listed unproven in the three branch reports still stands.
