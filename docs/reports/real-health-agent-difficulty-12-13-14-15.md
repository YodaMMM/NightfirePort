# real-health-agent-difficulty-12-13-14-15

The Agent (difficulty 2) sisters of the real-health walks of 07000012/13/14/15 — the same route
and the same end as each level's existing real-health walk, loaded at difficulty 2 the menu way.
All four sisters pass; every walk binary that contains one was run whole on the final tree.

## What's done

**The four sisters** (branch `s/real-health-agent-difficulty-12-13-14-15`, master f9c1d69c base —
the 01-07 branch was already merged there, nothing extra to merge):

- `walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_agent`
- `walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent`
- `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_agent`
- `walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent`

Each is the level's real-health walk parameterized on difficulty: the load is
`load_with_carry_difficulty(..., 2)` — the front end's launch path [M C_SBNFDFCTY 0x216ca8:15 →
GameState+0x28] — and each asserts `Drones::difficulty == 2` right after the load [M
GameState+0x28 → the sim's Drones::difficulty]. The difficulty-1 and difficulty-3 call sites are
byte-identical to the pre-sister master's (the 01-07 idiom).

**The per-difficulty pipeline the sisters run on** (checked, not assumed):

- Bond's pain: [M Player_HandlePain 0x1902c8 — `GameState+0x28 == 2 → ×Plr_DMod_Normal`, ==1 →
  ×Easy, ==3 → ×Hard, ==4 → ×2]. The row VALUES are runtime data: [L ee_start_05.bin
  @0x30cc50/0x30cc54/0x30cc58 (the ELF symtab names Plr_DMod_Easy/Normal/Hard) = 0.6/0.7/1.0],
  which matches the port's `damage_factor` 0x05..=0x08 group (0.6, 0.7, 1.0) [M ReadTuningVars
  0x1d3a80's per-level jump table]; 07000015 runs the 0x14..=0x16 group (0.4, 0.6, 1.0). An
  earlier working note claimed the measured 27.73 → 41.60 blast ratio "pins" the d2 row — that
  was cross-geometry reasoning (the 27.73 was the older run's grenade-set blast at 1.7 u, the
  41.60 the rim shot at 2.44 u) and has been corrected in the test comment and the WORKLOG.
- The guards' hit chance: [M DroneWeap_DoBulletAccuracy 0x179740 — the difficulty pick] with rows
  [L ee_start_05.bin @0x30caf0/0x30caf4/0x30caf8 (DroneFiring_Accuracy_Easy/Normal/Hard) =
  0.8/0.9/1.3]; the port's pick is [M drone.rs:11896-98 — 1 → Easy, 3|4 → Hard, else Normal].
- The Agent roster: [M Drone_Create 0x136be0:49 — the min-difficulty gate]; the d2 roster shifts
  the port indices, so every targeted pre-shoot resolves by the guard's [D]/[L] placement spot
  (`agent_drone_near`), never by a fixed index.

**Level 15 was the only sister that died; the fix is tactics, all real input** (no health, ammo
or channel writes, no teleports). The damage log read against the pipeline: the killing chain was
the #526 ankle-beam rim-shot blast (−41.60 health at 2.44 u, the walk's own unavoidable shot —
the doorway rim hides the beam from any further stand) landing on a 48.3-hp Bond, then the
west-room grenadiers' −4.2/−2.5 through the door[29] doorway. Three changes, gated to the
difficulty-2 runs (the d1/d3 paths unchanged):

1. The plank grenadiers (port #7 at (28.63, 41.34, 35.51), #8 at (42.71, 41.34, 35.49) [L the d2
   load probe]) are pre-shot from door[39]'s north lane before the west plank — they had plinked
   the measured run for −16.8 in 8 hits and −28.8 (a grenade) across that leg.
2. The west-room grenadiers (port #10 at (−5.14, 41.34, 67.25), #11 at (−1.87, 41.34, 77.42) [L
   the d2 load probe]) are pre-shot from just inside door[29], before the southwest jog — they
   shot through the doorway at the measured run's tail (frames 6020/6106).
3. A BodyArmourHeavy #802 detour was tried and DROPPED: the pickup is real and at (22.53, 40.35,
   35.53) [L the d2 load probe — `probe_armour_pickups_and_west_room_guards` in
   walkreal_probe15.rs, assertion-free and `#[ignore]`d], but it sits in the x ≈ 25 pit behind
   the ch_lock 0xc8 door at (25.1, 41.6, 36.6) — the detour walk wedged airborne against that
   door at (25.76, 41.47, 36.18) with floor_below 40.28. The blast stays on health; the budget
   comes from the fights instead.

Result: Bond crosses the #526 mark at ~84 hp, takes the 41.6 blast, and finishes; the endgame
fights (#12/#13/#14, #4/#5/#19) all end killed. Levels 12/13/14 needed no tactic changes — their
sisters passed on the shared routes with the difficulty-2 load and each level's existing fight
law (12 keeps head-hunting + unseen-fights; 13/14 keep theirs).

## How it was verified

Per-binary runs on the final tree (this branch HEAD; master f9c1d69c is already an ancestor),
`cargo test --offline --release`, `--test-threads=1`:

- `walkreal_07000012_live`: **3 passed** (master d3, real_damage d3, agent d2)
- `walkreal_07000013_live`: **3 passed**
- `walkreal_07000014_live`: **9 passed**
- `walkreal_07000015_live`: **6 passed** (twice — including the pre-existing d1 real-damage walk
  and the master top-up walk with the route split in place)

nf-game/nf-formats/nf-audio are untouched by this branch (nf-view test files, the WORKLOG and
this report only); the full suite covers them.

**The fast runner** (`tools/test/suite.py`, started 15:15 UTC on this tree, NF_SILENT=1): build
ok, 1096 test binaries in 1395 s. Its per-binary results and the final total line are in
`suite.out` / `../suite.log`; the FAILED/CRASH lines it printed up to 16:05 UTC, read against
this task:

- `FAILED walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent` — the
  suite ran the STALE 15:05 binary; the 15:16 armour-detour edit was never compiled into it
  (cargo considered the target fresh). Superseded by the final-tree 6/6 rerun above.
- `CRASH mp_online_failures`, `CRASH mp_remote_fx_live`, `CRASH interact_0700004a_live`,
  `CRASH mp_death_hands_online`, `CRASH net_soak_faults_live` (all exit −9 = SIGKILL) — OOM kills
  in the sandbox; several landed while this session's own targeted cargo runs competed for RAM
  (my runs are finished now). Not code failures.
- `FAILED fix_0c_pylon_live` (2 tests), `FAILED glass_guards_14_live` — render-visibility tests
  in the GPU-less sandbox (lesson 6: visual work needs the main agent's render). Pre-existing on
  this hardware, untouched by this branch.

## What's left

- The suite's final total line (it was still running at 16:05 UTC; the per-binary lines land in
  `suite.out`/`../suite.log` as it goes). Everything this task touches is already verified by
  the four whole-binary runs above.

## Live checks for the main agent

None required to merge. One optional pin, since no `ee_start_07000015.bin` snapshot exists:

- On a FRESH boot into 07000015 at Agent (difficulty 2), read the float at 0x30cc54
  (Plr_DMod_Normal) and 0x30caf4 (DroneFiring_Accuracy_Normal) — expected 0.6 and 0.9 if the
  0x14..=0x16 tuning group matches the port's table [M ReadTuningVars 0x1d3a80's jump table;
  [L] the same reads on ee_start_05.bin give 0.7 and 0.9]. A save-state resume skips
  ReadTuningVars — boot the disc.

## Usage

not visible to me
