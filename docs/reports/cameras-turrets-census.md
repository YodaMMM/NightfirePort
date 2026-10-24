# cameras-turrets-census — report

Branch `s/cameras-turrets-census` (from origin/master 26543436). Task: census every security
camera / turret / gun emplacement / auto-gun on the on-foot SP levels, map what the port has,
port the missing simulation, hand off the draw work.

## What's done

1. **The census** — `docs/systems/cameras-turrets.md`:
   - The five creator families under `parsemap_create_dynamic_objects` 0x1d04f0, each row
     tagged [M]: case 0x27 `GT_Create` 0x17ffa0 (auto-gun turret), case 0x28 `Sensor_Create`
     0x1af010 (the security camera), case 0x30 `Monitor_Create` 0x1b08b0 (the camera console),
     case 0x34 `GunImp_Create` 0x131f20 (the mounted gun), case 0xd2 `Shooter_Create` 0x17d030
     (the one-shot fixed gun) — with each one's update function, behaviour (sweep, alarm
     channel, firing, damage, destruction) and draw list.
   - **The brief's "drone templates that are cameras or turrets" does not exist** [M]:
     `Drone_Create` 0x136be0 + every `NDrone2_init_DMODE_*` (0x14d528..0x14e568) are character
     AI modes; no NDrone2 class sweeps a cone or fires a turret weapon. The cameras and
     turrets are exactly the five placement kinds above.
   - **The per-mission [D] table** (read from this ISO's bins, main map, spawn-gated): the
     wall guns live in Night Shift 2/3 (6+5, glist 0x200064c, enable ch 0xf-0x12/0xe/0x1c) and
     Phoenix Fire 2 (3 miniguns, 0x2000359); the cameras + consoles in Double Cross 1/2
     (1+2 cameras), Night Shift 2/3 (2+2), Phoenix Fire 2 (5+1) and Countdown 1 (14 cameras +
     12 monitors — the security centre); the laser mount in Chain Reaction 2 (#225); the
     shooters in Double Cross 1, Countdown 2, Phoenix Fire 3's elevator (2) and 07000041;
     2 GunImps per MP arena. Notable placements listed with indices/params.
2. **The port side, per row, pinned with tests on real level data** — the port on master
   already simulates all five families (creation, update, alert/fire, damage/destruction are
   wired); the census test proves creation per level:
   - `crates/nf-game/tests/cameras_turrets_census_live.rs` (new): for 11 census levels the
     port's five creators produce exactly the [D] placement counts (sensors/monitors/gunimps/
     shooters/GT), re-deriving the disc table on every run.
   - The per-family pins that already existed are catalogued in the doc §4 (gun_turrets_live,
     sensor_radius_live, sensor_live, monitor/gunimp/worldobj unit tests).
3. **One real simulation gap found and fixed** [M]: `GT_Track`'s per-candidate
   `Collide_LineOfSight(…, 0x72f)` (0x17f9c8:258,271-278) was supplied `los_ok = true` in the
   port's turret tick — turrets tracked and shot through walls. `crates/nf-game/src/drone.rs`
   now runs the ray per candidate through the collision world with the new
   `LOS_TURRET_TRACK = 0x72f` mask const (crates/nf-game/src/world.rs). Pinned by
   `crates/nf-view/tests/gt_los_live.rs` (new, 0700000a wall gun #636): a spot in the gun's
   front cone but behind the bridge structure must hold the gun DORMANT — **fails on the
   pre-fix tree** (state reached 4 = COMBAT), passes with the fix; the open-deck spot still
   deploys to COMBAT (no regression — `gun_turrets_live` 2/2 still passes).
4. **Draw hand-off rows** in the doc §5 + `// draw hand-off:` comments left at the spots:
   the GunImp gun child (TurretGuns art at `gunimp.rs barrel_matrix()`), the sensor lens/mount
   proxies + SetBeam beam (sensor.rs), the GT barrel-spin/wreck smoke events (drone.rs).

## How it was verified

- `cargo test --offline --release -p nf-game --lib` — 546 passed, 0 failed.
- `cargo test -p nf-game --test cameras_turrets_census_live --test difficulty_gates --test
  level_live` — all pass.
- `cargo test -p nf-view --test gun_turrets_live --test gt_los_live` — 2/2 + 2/2.
- `gt_los_live` re-run with only `drone.rs` reverted: the blocked-spot test FAILED (state 4)
  — the fail-before-fix proof — then the fix was reapplied.
- Full suite (`python3 tools/test/suite.py`, background): build ok (775 test binaries,
  1125 s), run still in progress at this report's time; failures so far are the documented
  slim-ISO set (3 × `disc_golden`), `identity_disc` (fails on the untouched tree too — it
  failed at 07:32 before any of this branch's code was built), the no-GPU render probe
  (`probe_pylon_render`: `adapter: NotFound` — this sandbox has no GPU), and the MP-network
  set + `stdio_pipe_closed_live` + `load_time_probe` (exit −9 kills — sandbox network/memory).
  None touch this task's code; the final-tree caveat: origin/master was already an ancestor
  of the branch when the suite ran, and the suite's binaries include this branch's
  nf-game/nf-view rebuilds. Final tally at the last push is appended below if the run
  completes in time.

  **Tally at the sandbox deadline (08:26 UTC, run still in progress, ~17 of 775 binaries
  reported)**: every failure line is environmental or pre-existing — the documented slim-ISO
  set (`identity_disc` + 3 × `disc_golden`), the no-GPU render probe (`probe_pylon_render`,
  `adapter: NotFound`), the MP/network set (`mp_online_failures`, `mp_death_hands_online`,
  `mp_remote_fx_live`, `mp_world_sync_live`, `mp_shot_effects_live`, `net_soak_faults_live`,
  `net_desync_report_live`, `net_join_live`, `net_resync_live` — exit −9, no network peers),
  and the memory-killed `load_time_probe`. Zero failures in camera/turret code or any test
  this branch added or touched; the most relevant `nf_game` live binaries the run had not
  reached were run manually instead (ai_live 36, ai_missing_states_live 5,
  door_glist_census_live 7, abseil_death_live, level_live, difficulty_gates,
  cameras_turrets_census_live — all green).

## What's left

- The draw side of the family (§5 hand-offs) — not this task per the brief.
- The suite's remaining binaries at push time.
- Live confirmation of the turret LOS on hardware (optional, doc §6 lists the gdbwatch check).

## Usage

not visible to me (no token/credit/context-window or cost figures are exposed to this session;
the sandbox shows none).
