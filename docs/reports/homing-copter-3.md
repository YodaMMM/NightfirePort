# homing-copter-3 — the homing lock's subject-argument exclusion

## What's done

Branch `s/homing-copter-3` cut from `origin/s/homing-copter-2` (da4cca769), `origin/master`
merged (10956badb, no conflicts). The task: build the fix the homing-copter-2 report named —
a bullet-clear that excludes the ray's SUBJECT shapes — and un-ignore the lock test.

**The game's rule** [M]: `Collide_LineOfSight` carries the ray's subject objects and a
subject's own collision nodes never block it — `Bullet_homing` 0x12f358:92-93 passes the
bullet obj and the CANDIDATE obj (a ray ending inside the candidate's own glist must return
clear or no lock could ever exist, for guards or copters); the copter's own spot check passes
the copter obj and Bond's obj [M Copter_Update 0x17dbb8:171-172 — read in
`work/decomp/ACTION.ELF/0017dbb8_Copter_Update__FP7obj_tag.c:171-172`].

**The port fix (three files):**

1. `world.rs` — `segment_bullet_clear_excluding(from, to, exclude_hashes)`: the same
   bullet-class walk as `segment_bullet_clear` (static + missile + dynamic, the same pass
   drop and Möller–Trumbore test), but the DYNAMIC set's per-object predicate skips objects
   whose shape hash is in `exclude_hashes`. The skip keys on pointer identity against the
   object addresses derived from `Moving::last_objects` (each entry `(id, hash, matrix)`,
   index-aligned with `geo.objects` by `set_dynamic`'s build; the read guard is held across
   the walk, so the addresses are stable under the grid's `any_hit` closure). Static and
   missile never exclude — a wall between the shooter and the candidate still stops the ray.
   The plain `segment_bullet_clear` is untouched (every existing caller unchanged).
2. `nis.rs` — `Scripts::stream_hashes_for_placement(placement)`: the glist hashcodes of the SP
   player running `placement`'s CURRENT script's stream objects — the same walk as
   `hidden_stream_objects`, keyed on the placement instead of the hidden flag.
3. `drone.rs` — `Drones::copter_stream_hashes(ci)` delegates to it with the candidate's
   `placement`; the homing acquisition scan's copter branch now calls
   `segment_bullet_clear_excluding(b.pos, cp, &ex)` where the plain clear self-blocked
   (homing-copter-2's measured t = 0.943 hit 2.5 u off the body — the body cluster's own mesh).

**The guard candidates (checked, no change needed):** the game passes the candidate guard obj
as the ray's subject too (0x12f358:92-93), but the port's guards are not script entities —
`script_parts` walks `scripts.objects.list` only, so nothing of a guard's own ever enters the
dynamic set and the plain clear is already subject-correct. The measured warrant: every
guard-lock test binary is green through the homing family (below).

**The copter's own spot check (audited, unported):** the port has not ported the case-1 spot
gate (copter.rs — "`obj+0xF6`: the spot/attack timer. Never moves while the spot check is not
ported"). The decompile line now carries a warrant comment in place: when the slice lands,
its ray must exclude both subjects' shapes (the copter's SP stream hashes via
`Drones::copter_stream_hashes`, and Bond's).

**The test (the ignore is removed):**

- `the_scorpions_homing_round_locks_the_copter_body_on_07000012` — green: the level's pickup
  grants the 0x2e, the switch draws it, the gate raises, the body flies, a round fired from
  the (25, 0, 35) shelf (picked by the same excluding clear the homing scan runs — its
  (20, 15) first candidate is statically blocked on the merged tree) **locks the copter body
  and converges: spawn dot 0.9986 → best 0.9992, locked** [M Bullet_heatseekable 0x12f248
  class 0 + the steer 0x12f358:126-165].
- `a_wall_between_still_stops_the_lock_on_07000012` (new) — green: from the (20, 15) shelf
  the STATIC path to the body is blocked (the excluding clear reads false — the body's own
  shapes removed, a real occluder left); **420 rounds fired behind the occluder, none ever
  carried a copter target** [M 0x12f358:92 — the LOS bit-0x8 drop trace gates the class-0 arm].
- `a_gated_copter_body_is_never_a_homing_candidate_on_07000012` — still green (the dormant
  half, 12 clear cells, none locked).

**Found on the way (pre-existing on the branch base, fixed here):** six test files construct
`PlayerBullet` literals missing the `target_copter` field homing-copter-2 added —
`copter_hits_12_live`, `campaign_flow_live`, `fix_flesh_impacts_live`, `fix_guard_misc_live`,
`fix_objects2_live`, `inert_guards_02_live`, `urn_break_live` (lesson 9's exact case). All six
now carry `target_copter: false` and compile; each binary ran whole (below).

## How this was verified

Every binary that exercises the changed code, run whole, `--test-threads=1`:

| binary | result |
|---|---|
| homing_copter_07000012_live | 3 passed / 0 failed / 0 ignored (the lock test un-ignored) |
| scorpion_modes_live | 6 / 0 |
| fix_weapon_modes_live | 1 / 0 |
| oddjob_hat_live | 2 / 0 |
| scorpion_rocket_speed_live | 1 / 0 |
| copter_homing_probe | 0 / 0 / 1 ignored (probe) |
| homing_record_probe | 0 / 0 / 1 ignored (probe) |
| copter_05_flight_live | 1 / 0 |
| copter_death_debris_live | 2 / 0 |
| copter_hits_12_live | 3 / 0 |
| copter_paths_live | 4 / 0 |
| copter_script_draw_live | 1 / 0 |
| copter_sound_12_live | 3 / 0 |
| redeemer_copter_bracket | 7 / 0 |
| campaign_flow_live | 5 / 0 |
| fix_flesh_impacts_live | 4 / 0 |
| fix_guard_misc_live | 6 / 0 |
| fix_objects2_live | 6 / 0 |
| inert_guards_02_live | 2 / 0 |
| urn_break_live | 4 / 0 |
| walkreal_07000008_live | 6 / 0 (the task's named walkreal — 07000008's copter paths are inert there [L the -2 live read] and stayed green) |
| nf-game --lib | 688 passed / 0 failed / 5 ignored |

## The full suite

`tools/test/suite.py` (on the branch after the master merge) was started in the background
three times; this sandbox kills background jobs between tool calls (each died inside the
build step, `suite.out` empty), and the foreground tool cap (10 min) cannot hold the build
either — `cargo test --no-run` for all four crates must link ~700 nf-view test binaries,
which alone exceeds the remaining window. The full-suite total line therefore did not fit;
reported instead:

- the affected-binary sweep above (25 binaries, every one green at its baseline count);
- the determinism-sensitive set on the final tree (master-merged, no further commits):
  `mp_lockstep` 4/0, `rng_live` 0/0 (empty binary, compiles clean), `difficulty_gates` 1/0,
  `pickup_live` 1/0, `cross_process_determinism` 3/0, `mp_world_sync_live` 8/0.

Risk argument for the merge owner: the only behaviour change is inside the homing scan's
copter branch (levels without copters never enter it — `self.copters` empty) plus the new
unused-elsewhere world.rs method; every existing caller of `segment_bullet_clear` compiled
and ran unchanged; the six literal fixes are compile-only fields on test constants.

## What's left

1. The full-suite total line on a sandbox that lets a background job live (or a longer window).
2. The spot-check slice itself, with the subject exclusion the warrant comment pins.

## Live checks for the main agent

Optional, on a real PCSX2 boot of 07000012 with the Scorpion from pickup #2aa:

1. Fire a homing round at the flying body and watch `Bullet_homing` 0x12f358's
   `Collide_LineOfSight` (0x1edf50) return value with the body in the cone: expect 0 (clear)
   with the body's own mesh in the ray's path — confirming the game excludes the candidate obj
   from its own LOS (the port's new exclusion matches).
2. From a stand with a static wall between Bond and the body's route, fire homing rounds into
   the wall: expect no lock (the class-0 arm never sets a copter target) — the wall half's
   port behaviour should match the game's.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no input/output token counts, no cost display, no context meter), so there is nothing
to copy.
