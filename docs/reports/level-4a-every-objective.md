# level-4a-every-objective — report

## What the level lists, and what the tests already covered

0700004a (Tower-2 elevator, `UpdateMFX_HT_Level_Tower2Elevator` [M 0x114e70 case 0x700004a])
lists exactly ONE objective in its mission record (`c0700004a.rs` [L ee_start_05.bin @ 0x2a41e8]):
"Gain access to lobby", enable ch 0x4d, complete ch 0x4a. That is the whole list — the objective
records have one row and the fail conditions are the shared alarm/civilians monitor
[M 0x1827d8:128-141], with no writer for 0x61/0x60 anywhere in the level [D census].

Coverage before this task:

- `walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby` — the WHOLE
  mission on real input from the fresh load: spawn on the platform (#1 [D]), the first steps
  fire the reveal plate #277 (ch 0x4d [D]) and the objective is asserted revealed
  (`mission.states[17][0] == 3`), the laser watch drawn [M 0x1a7ef8:318-319], the keyed
  150-unit descent ridden with no re-seating (the scrape check runs every enabled tick), both
  satchels burned under the beam [M 0x12e2f0], the descent plates asserted in order, the bottom
  plate #62's ch 0x84 [D] and the mission's LevelEnd asserted (`level_exit.is_some()`). The
  completion channel 0x4a has NO writer on 0700004a [D scan, all plates/doors/spawner outs/SP
  outs/script events] — the real writer is 07000013's spawn touch #153 [D 07000013.bin], watched
  across every part by `Mission_MonitorObjectives` [M 0x1827d8:22-38]. That cross-level fold is
  pinned on real input by `walk_07000013_live` (the warm chain arrives with 0x4A on — re-run
  green this session, 2 passed).
- `walkreal_0700004a_satchel_live` — the sister ride (burn both satchels, reach the lobby alive).

Both were already green on the merged master (1b2309ea) — re-run green this session (5.5 s /
7.9 s before the fix below; 6.6 s / 10.2 s after it, with the garrison live).

## The work: GAP-1 (the rider spawner release) — real root found and fixed

The two `interact_0700004a_live` spawner tests were `#[ignore]`d on the story "the car interior's
walls do not block the port's `Drone_PositionVisible` stand-in, so the not-visible gate never
opens; the game's enclosed car spawns them [S]". The probe (`probe_4a_spawn_live.rs`, committed)
redid the diagnosis and the story was wrong twice over:

1. At the parked position (z −12.4) the gate was never the problem — the rays ARE blocked there
   (segment_bullet/segment_actor hits) — but ch 0x17 never fires because the park leaves every
   plate's volume: the spawner never even armed. The park workaround was self-defeating.
2. On the ride line the spawner arms (ch 0x17 rises at t≈1140) but the port's min-dist screen
   rejects every template forever: the group-2 spots sit 7.6–7.7 away in x/z for the whole
   descent, and the port's compare was **2-D x/z**. The game's compare is
   `Vec_SqDist3D(player+0x30, spot) >= min*min` [M 0x137cb0:28-31 — `Vec_SqDist3D`, the 3-D
   squared distance, with usable templates skipped without consuming the draw; the 10.0 default
   when p8 = 0 is 0x1378d0:35-37's 0x41200000]. A vertical rider outgrows the 3-D sphere; the
   2-D one never moves.

**Fix** (`drone.rs`, the spawner release closure): the min-dist metric is now the game's 3-D
distance, with the decompile addresses in the comment. Result: the garrison releases during the
ride on the ride line — no park, no poke — and the visibility abort [M 0x137cb0:43 → 0x13a7a0]
operates as decoded (the probe measured the rays blocked on the ride line; the shaft's own
geometry does the blocking, so the old "car walls" sub-claim never engaged on this level).

Test moves (each with its evidence, per the discipline):

- `the_floor_spawner_releases_its_drones_on_the_floor_plate` — **un-ignored and green**: asserts
  the start channel [D #233], the temporal sister (nothing before the plate), the release after
  the plate, and (moved in) the no-fire health sister: the spawn health is the port's
  stats-default 10.0 [M 0x1c7e58 — the drone_stats word +8 for these guards is 0 at run time,
  the 07000005 snapshot [L] → the 10.0 default]. The old "param 5 = 0x64 = 100 [D]" reading
  pinned the behaviour-word table, not the health — **the TEST was wrong and was fixed**.
- `a_spawned_drone_takes_the_riders_fire_across_the_shaft` — **re-parked with the new exact
  blocker**: the garrison DOES release (the release test pins that), but from the ride line no
  fire ray reaches the group-2 spots — 0 of 2400 ticks had a clear eye→drone segment_bullet ray
  (the spots sit behind the shaft core at x ≈ −4.3 [D census]). Whether the real game's rider
  can hit these spots mid-ride is unverified [S] — live check below.

## How it was verified (what I ran)

- `cargo test -p nf-view` binaries, direct: `walkreal_0700004a_live` 1 passed; `walkreal_0700004a_satchel_live`
  2 passed; `interact_0700004a_live` 6 passed / 1 ignored; `walk_0700004a_live` 1 passed;
  `rappel_vanish_0700004a_live` 2 passed; `probe_0700004a_cutscenes_live` 1 passed;
  `walk_07000013_live` 2 passed (the objective's cross-level fold, ch 0x4A on [D #153]);
  `fix_finale_live` 4 passed (the 0700001b Drake fight — the spawner-sensitive regression net).
- `cargo test -p nf-game` (lib, 98 test binaries): all ok.
- The FULL suite (tools/test/suite.py) on the final tree (this branch, master merged — master
  never moved past 1b2309ea), 893 binaries, chunked in the foreground with a 60–120 s per-binary
  wall clock, aggregated from the per-chunk logs:

  **3433 passed, 7 failed, 197 ignored, 17 binary-timeouts** (893/893 binaries started).

  The 7 failed test cases, each classified:
  - `directory_tree_matches_the_golden_manifest`, `bigf_header_fields_behave_as_documented`,
    `bigf_tables_match_the_golden_manifest` — the 3 `disc_golden` tests, the documented slim-ISO
    exception (rule 5).
  - `the_real_disc_passes_the_identity_check` (identity_disc) — the same slim-ISO disc-identity
    class (0.01 s, the full-ISO identity does not match the slim image).
  - `the_0c_pylon_is_on_screen_at_the_user_spot`, `the_0c_pylon_fills_the_view_from_the_closeup_spot`
    (fix_0c_pylon_live) — **pre-existing on master**: re-run with the pre-fix drone.rs (1b2309ea)
    and they fail identically (visual spot tests; unrelated to the spawner change).
  - `probe_render_pylon_only` (probe_render4) — environmental: passes when run directly, with
    both the pre-fix and the fixed drone.rs; failed only inside the chunked suite run.

  The 17 binary-timeouts, each classified:
  - Slow-but-green (re-run directly, uncapped): `walkreal_07000003_live` 4 passed in 387 s;
    `walkreal_07000008_live` 3 passed in 94.8 s; `weapon_matrix_live` 6 passed in 133 s. The
    same limit class: `walkreal_07000002_live`, `walkreal_07000005_live`, `walkreal_07000014_live`
    (full-mission walks over the per-binary cap).
  - The sandbox's hang/slow class (the same binaries hung or were cut in previous sessions):
    `campaign_flow_live`, `levels_sweep_live`, `guard_stuck_sweep_live`, `stuck_guard_combat_05_live`,
    `fix_npc_doors_live`, `npc_doors_live`, `net_soak_faults_live`, `frame_cost_live`,
    `rook_door_06_live`, `pss_vag_golden`, `nf_view`.

## What's left

- The fire-across-the-shaft leg (parked, exact blocker + evidence in the ignore note).
- GAP-2 (the hand-back POSE after the landing scene) — pre-existing, parked live check, untouched.
- The two live checks below.

## Live checks for the main agent

1. **The garrison spawn timing.** Fresh boot to 0700004a, `tools/trace/gdbwatch.py` on
   `Drone_Create` 0x136be0 (entry) while Bond rides; record the first group-2 spawn's tick against
   Bond's y through the −40 floor. The port spawns them at the arm tick (ch 0x17's rise — the
   far-down templates pass the 3-D min-dist sphere at once). Expected if the game matches [M
   machinery]: the first spawn within ~a second of ch 0x23 (0x17) latching; if the game instead
   spawns later (the rider leaving the sphere tick-by-tick), the port's release timing needs a
   second look.
2. **The fire reachability.** Same boot, ride with the PP7 and fire at the group-2 spots' ledge
   (x ≈ −4.3, z −0.55, y −43..−59 [D census]) as the car passes: does any shot land (a hit
   reaction / the drone's health byte moving)? The port: 0/2400 clear rays. If the game CAN hit
   them, the port's fire ray path differs from the game's on this geometry (the game's
   `Collide_LineOfSight` walks the AI cels — maybe the shaft core is portal-crossable there).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session.
