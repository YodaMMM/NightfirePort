# per-guard-motion-stamps — report

Branch `s/per-guard-motion-stamps` (cut from origin/master 6042ee53, merged with origin/master — no
drift). **Status: done.**

## What the task was

The port kept ONE global first-moved / first-stopped stamp for the player's motion; the game keeps
one per guard. Port the per-guard stamps with the addresses, and pin that two guards that start
tracking at different ticks clear their first-moved terms at different ticks.

## What the game does (evidence)

`DroneWeap_DoOpponentTargetting` 0x179a48 (ACTION.ELF decompile
`work/decomp/ACTION.ELF/00179a48_DroneWeap_DoOpponentTargetting__FP10DCVars_tag.c`):

- :31-33 — the whole body gates on the target object pointer `Drone+0x170 != 0`; nil freezes the
  latches.
- :34-41 — when the target's class byte `obj+0xFF == 3` (the player object), the drone's moving
  byte `+0x1DC` is recomputed from the player's move quad `|(BL+0x10, 0, BL+0x18)| != 0`; a
  non-class-3 target (the team-2 drone pick) RETAINS the byte (:45).
- :47-59 — the stop arm: `+0x1DD` and `+0x1DE` clear; when neither `+0x1DF` nor `+0x1E0` stands the
  drone latches ITS stamp `+0x224 = GameState+0x52` and arms `+0x1DF`; `+0x1DF` clears once
  `stamp + fptui(DroneFiring_TargetFirstStopped_TimeToHit * FRAME_RATE) < now`.
- :61-73 — the move arm, mirrored: `+0x1DF`/`+0x1E0` clear; the `+0x1DD`/`+0x1DE` halfword test
  arms `+0x224 = GameState+0x52` + `+0x1DD`; `+0x1DD` clears once
  `stamp + fptui(DroneFiring_TargetFirstMoved_TimeToHit * FRAME_RATE) < now`.

The stamps live on EVERY `Drone` — nothing global. And the targetting runs PER DRONE:
`DroneWeap_HandleFiring` 0x17a180 calls `DoOpponentTargetting` in BOTH arms (:29 the not-firing
arm, :59 the firing arm), so each guard latches the tick IT observes the change.

## What the port did

`crates/nf-game/src/drone.rs`:

- `Drone` gains the per-guard state (each on its own line): `moved_since: Option<u32>` (the `+0x224`
  stamp while the `+0x1DD` latch stands), `stopped_since: Option<u32>` (the `+0x1DF` mirror),
  `target_motion: Motion` (the drone's own `+0x1DC/+0x1DD/+0x1DF` read). Each latch+flag pair folds
  into one Option: `Some(stamp)` past the window keeps the flag standing (no re-latch while the
  phase runs, [M :65]); the opposite transition clears it ([M :48-49, :62-63]).
- New `Drone::observe_target_motion` — the per-drone latching exactly as 0x179a48:31-74 says, called
  at the top of `Drone::handle_firing` (before the `firing` gate — the game's not-firing arm runs
  the targetting too [M 0x17a180:29]), gated on the port's `+0x170` fold `opp_ptr()`; the `+0x1DC`
  recompute only for the player-class target (`opp_bond`), retained otherwise ([M :45]).
- The tick (`Drones::tick_inner`) computes the shared quad read once (it is the same player object
  read [M :34-41]) and hands it to every drone's weapon pass; the old global latch update and the
  global `moved_since` / `stopped_since` / `target_motion` fields are DELETED — the game has no
  global copy.
- `fire_weapon`'s hit roll now consumes the firing drone's own `target_motion`
  ([M DroneWeap_DoBulletAccuracy 0x179740:52-73 reads the drone's bytes]).

## Tests

- NEW pin `two_guards_that_start_tracking_at_different_ticks_clear_at_different_ticks`
  (nf-game lib tests): guard A latches at frame 1, guard B (entering the object walk at frame 50,
  the port's "later spawn starts tracking later") latches at 50; A's first-moved clears at frame
  122 while B still holds; B clears at its own stamp + 121 = frame 171. The old global model could
  not even represent this (one shared `moved_since` cleared every guard together) — the pin is the
  reproduce: the symptom was the port's structure itself, and no existing test asserted the global
  sharing (it had no per-guard observable).
- `target_moving_reads_the_move_quad_not_a_position_delta` re-pointed: it observed the deleted
  global mirror through an EMPTY drone list. The latches live per guard ([M 0x179a48:47-73]), so
  the test now drives ONE real guard through `tick_inner` and reads its `Drone::target_motion`.
  Same game behaviours pinned, same addresses, none weakened: quad moving [M :39-41], first-moved
  arms [M :65-68] with the guard's own stamp at :67, holds through stamp+120, clears at stamp+121
  [M :70-73], the stamp stays past the window (no re-latch [M :65]), the stop arms `first_stopped`
  [M :51-55], the zero-g thruster pair is the quad ([M 0x192ab0:113,135] via `player_move_local`).
- `enemy_accuracy.rs` untouched — it builds explicit `Motion` values for `hit_chance`, which is
  unchanged.
- `walkreal_0700001b_live.rs` re-pointed to the per-guard read: the hit diagnostics log the HITTING
  drone's own `target_motion`; the float pins (`the_zero_g_float_reads_moving_while_its_thrusters_decay`)
  read the first present live guard's own bytes through a `first_tracker` helper (the station's
  opening wave is present from the arrival). Same addresses cited; the zero-g semantics pins are
  unchanged in meaning.

## What I ran (all on the final tree = branch merged with origin/master 6042ee53; slim on-foot ISO)

- nf-game lib unit tests incl. both motion tests: `cargo test --release -p nf-game --lib` — the two
  re-worked/new tests pass; full lib suite green.
- `enemy_accuracy_live`: **5 passed, 0 failed, 4 ignored** (the ignored are the exploration
  printouts).
- `ai_live`: **37 passed, 0 failed, 0 ignored**.
- every walkreal_* real-damage variant (12 of them, one run each):
  walkreal_07000001_live/02/04: **16 passed, 0 failed**; 05/06/07/09: **26 passed, 0 failed**;
  0a/13/41/4a: **17 passed, 0 failed, 4 ignored**; plus `walkreal_0700001b_live` (the edited file,
  incl. `walk_arrival_to_drakes_defeat_on_real_health` and the re-pointed float test) inside the
  62-passed chunk below.
- `nf-game` — the whole package, all 98 test binaries: **217 + 170 passed = 387 passed, 0 failed,
  4 ignored**.
- `nf-formats` + `nf-audio` (all binaries): **114 passed, 4 failed, 3 ignored** — the 4 failures are
  the slim-ISO disc-data checks, not code: `disc_golden::bigf_header_fields_behave_as_documented`,
  `disc_golden::bigf_tables_match_the_golden_manifest`,
  `disc_golden::directory_tree_matches_the_golden_manifest` (the 3 the suite rules allow to fail on
  the slim image) and `identity_disc::the_real_disc_passes_the_identity_check` (a pure
  `nf-formats` disc-identity check; this task touches no nf-formats code — it fails on the slim
  image's contents, it fails identically without this branch's changes).
- nf-view chunk 1 of the alphabetical walk (185 binaries, 144 completed before the wall-clock cut):
  **344 passed, 2 failed, 41 ignored** — the 2 failures are
  `fix_0c_pylon_live::the_0c_pylon_fills_the_view_from_the_closeup_spot` /
  `..._is_on_screen_at_the_user_spot`, which die at `wgpu` adapter creation
  (`adapter: NotFound … no_fallback_backends`) — this sandbox has no GPU backend; not a code
  failure (the lessons: visual work needs the main agent's render).
- nf-view chunk 2 (180 binaries, 117 completed before the cut): **358 passed, 0 failed, 26 ignored**.
- The fast runner (`tools/test/suite.py`) cannot stay alive between tool calls in this sandbox —
  background processes are killed; the above is the same suite run in foreground `--only` chunks,
  same `--test-threads=1`, same wall-clock limit per binary.
- **Left unrun**: nf-view chunks 3-5 of 5 (433 binaries — the alphabet later half, mostly probes and
  render/gpu-heavy tests). No failure outside the two documented environment classes appeared in
  what ran.

## Usage

not visible to me
