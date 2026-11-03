# level-41-every-objective — every 07000041 objective on real input

Branch `s/level-41-every-objective` (cut from `origin/master` 6f16146e; master had not moved
when re-merged at the end, so the branch IS the final tree).

## The objective-list finding: the wrapper has zero objectives

The task's objective records do not exist for this level, and that is now pinned where the game
reads it:

- The `MissionData` table (ACTION.ELF 0x2a4350, 24 records × 0x28) has **no 0x07000041 row** —
  re-read row by row from the snapshot this session [L ee_start_05.bin; the same 24 rows the
  port's `mission.rs` LEVELS mirrors].
- `Mission_Init` (0x182270) takes the no-record path: BaseMap stays −1 and InternalState =
  State = 5 [M 0x182270:22-24, 199-203]; `Mission_Update`'s case 5 is a bare no-op [M
  0x182c60:361-367], so `Mission_MonitorObjectives` never runs — **nothing can announce and
  nothing can complete**.
- The tests pin this on the mission layer's own state (`base_map == u32::MAX`, internal 5,
  state 5, `mission_events` empty) at the load, at the chain boundary, at the movie trigger and
  at the far-south walk end.

What the level DOES define as its play is its trigger chain [D the placements]: the chain
trigger #99 (out ch 1, mask 0x4 = the E dispatch [D PL#99 params 0:0x1 1:0x4 7:0x6e]) and the
walk-through plate #100 (out 0x6e, mask 0x8 player [D PL#100]), whose channel consumers are the
turret #584's arm, the DynoRod #117's gate, the ctx-2 icon #105 and the two voice triggers [D
params; the audit's consumers list]. The playthroughs below play that chain both ways a player
can, on real input from the fresh start.

## A decompile correction: the plate does NOT lock the E out (the test was wrong)

`Trigger_Activate` 0x1b2318's case-1 (TouchOnce) arm gates only on `mask & 4` and writes the out
channel — **no lock read anywhere in the arm** [M 0x1b2318:38-49, read this session]. The lock
channel is read by `Trigger_Locked` 0x1b2fc8 (its non-type-10 arm: locked iff ext+0x24 is on),
whose ONLY caller is `Check_Target` 0x1a7810:197 — inside the **type-10 aim-hint** branch (via
`Player_AutoAim` 0x1a7c98) — never the E-fire path.

interactables' `the_chain_trigger_answers_the_e_until_the_plate_locks_it` asserted the opposite
("after the plate the same E is refused") and passed **vacuously**: its leg 2 never aimed, so
the use sphere [M 0x1a5170:100-108] at pitch 0 passed over #99's 0..0.75 y-band and the E
reached no trigger at all. Per the rules the TEST was wrong: it is now
`the_chain_trigger_answers_the_e_before_and_after_the_plate`, and its leg 2 aims at #99 (the
same aim leg 1 uses) and pins the aimed E **firing ch 1 with the plate's ch 0x6e on** [M
0x1b2318:38-49]. The port needed no code change — its `activate_request` TOUCH_ONCE arm already
matches the game.

## The playthroughs (all real input from the fresh load; no teleports, no forced fields, no set_script stand-ins)

All in `crates/nf-view/tests/walkreal_07000041_live.rs` (11/11 pass):

1. **`the_chain_order_played_from_the_fresh_start`** — the E on #99 first: ch 1 rises (checked
   on the channel [M 0x1b2318:38-49]), the chain consumers answer the REAL channel (the
   turret's one shot [D #584 p0=1; M Shooter_Update 0x17d0c0 — the fire + the ext+0xfe flag],
   both voices [D #588/#589; M SoundTrigger_Update's channel gate]), then the plate walk arms
   0x6e [D PL#100; M 0x1b2b88], and the **DynoRod deletes** with both channels armed [D #117
   params 0:0x1 1:0x6e; M DynamicObject_Update 0x18d3b8's gate + the mode-0 delete] — the
   interactables pin for that delete used the set_script stand-in for ch 1; this is the player's
   E path. The walk reaches the plate/rod inside the port's stand-in exit window and asserts
   `level_exit == None` there.
2. **`the_plate_first_walk_locked_e_dyno_rod_tether_and_the_far_south`** (+ its real-damage
   sister) — the plate first, no E anywhere (ch 1 stays cold — the E is #99's only toucher [D
   PL#99 mask 0x4] and the walk never presses it), the DynoRod stands [M 0x18d3b8's gate], the
   tether crossing at the ctx-3 icon #114 [M 0x196e80; 0x1916b0:126-147; 0x1977c0], the wire
   drops Bond grounded, and the walk ends on the far south floor (z −81) with Bond alive, ch 1
   cold, no exit, the mission layer inert.
3. **`the_north_detour_walks_to_the_movie_trigger_and_its_e_is_a_no_op`** — the movie trigger
   #81's volume (16.13, 0, −0.41 [D PL#81]) reached ON REAL INPUT from the airlock (the older
   tests teleport to it), then the real E: `movie_request = Some(0)`, no movie, no exit [M
   0x1b2318:51-62 case 0xd — the audit's consumer chain].

Route facts the probes established (deterministic, both runs identical): the north-west walk
reaches #81's volume via a single deterministic wedge at (17, −4) (the stuck ladder lands at
(6.3, 4.1) both times); the north deck's mid-floor jams the walker's straight steers at
(22.8, −4.1) with CLEAR segments to every target — the port's own collision [S], never measured
against the game.

## Verification runs

- `walkreal_07000041_live`: **11/11 pass** (3 new playthroughs + the real-damage sister + the
  7 pre-existing tests), direct run this session.
- `interactables_07000041_live`: **18/18 pass** (with the corrected chain-trigger test),
  direct run this session.
- `nf-game --lib`: **655 passed, 0 failed** (5 pre-existing ignores).
- Full suite (the fast runner, started in the first minutes): the build phase finished clean
  ("build ok: 1089 test binaries in 1517 s"). The run phase on this 2-core box is far slower
  than the docs' 13 min (~3 h projected; 284 binaries done when stopped), so **no total line
  exists** — the run cannot fit the window. Partial results on the then-unchanged tree:
  `identity_disc::the_real_disc_passes_the_identity_check` and
  `probe_pylon_render::probe_render_pylon_only` FAILED and `bigf_header_fields_behave_as_
  documented` / `bigf_tables_match_the_golden_manifest` / `directory_tree_matches_the_golden_
  manifest` FAILED — all five also fail in the previous session's suite.log on master's tree
  (the slim-ISO disc-golden set + the no-GPU pylon probe): pre-existing, not this task's.
  `mp_online_failures` CRASHed (exit −9, SIGKILL) while my own cargo builds ran concurrently —
  an OOM artifact of that contention, not a code change (this task touches two test files and
  docs only). Re-run alone on the final tree it HANGS in
  `join_in_progress_from_the_menus_then_leave_and_reconnect` past 560 s — an
  environment-sensitive MP socket test (untouched by this task; the previous session's own
  suite.log never reached it either — its run phase died after 4 binaries). The runner was
  restarted on the final tree after the merge (master had not moved — the merge was a no-op;
  its build phase printed `build ok: 1089 test binaries in 3 s`, everything cached); the run
  phase is the same ~3 h shape and its partial results live in `suite.out`/`suite.log`.
- Every binary this task's changes touch was re-run directly on the final tree and is green
  (the two listed above); no shared code changed — `crates/nf-game`, `nf-formats`, `nf-audio`
  are untouched, and no `crates/nf-view/tests/common/` file was modified.

## What's left / coordination

- **The exit event (30-level-41-exit-event-2)**: the port's documented stand-in (ch 1 on for
  241 ticks latches `level_exit = Some(0x0700001b)`, drone.rs) is the wrapper's only exit
  carrier and this task stayed off it: the chain-order walk stops at the plate/rod inside the
  window (asserting only `level_exit == None` there), the plate-first walk never raises ch 1,
  and neither test asserts the stand-in's value. Because the E fires ch 1 regardless of the
  plate (the correction above), a "full chain + full south walk" single test cannot exist while
  the stand-in stands — the exit-event task's rewrite decides what that walk should pin.
- The north deck jam is [S] port-collision; a live check can settle it (below).
- The wrapper's own end stays the driving-side flow (doc §7); nothing new this session.

## Live checks for the main agent

1. Fresh 07000041 boot, walk north from the airlock toward the north deck (x ~17-25, z −8..0):
   does the retail floor really block straight walking there (matching the port's jam at
   (22.8, −4.1)), or is the port's collision wrong? Expected per the port: a wall/step edge
   with clear line-of-sight over it.
2. Same boot, stand at #99 (20.59, ·, −32.0), aim down into its volume, press E after walking
   through the plate #100: expect ch 1 (0x26fc90+1) to RISE (the decompile's case-1 arm reads
   no lock) — confirming the test correction against the real game.

## Usage

not visible to me
