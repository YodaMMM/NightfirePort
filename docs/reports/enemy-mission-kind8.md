# enemy-mission-kind8 — the EnemyMission 0x15e1e8 kind-8 mission walks

Branch `s/enemy-mission-kind8` (master a67f1a27e + 4 commits). Session 2026-10-23 16:55–18:5x UTC.

## What's done

1. **The state ported** [M `work/decomp/ACTION.ELF/0015e1e8_NDrone2_DSTATE_EnemyMission*.c`]:
   `crates/nf-game/src/drone.rs` gains the full `ENEMY_MISSION` (0x1b) arms:
   - **ENTER** (case 1): the voice-0x13 (+0xD8) skins bail to KikoMission 0x19 (the
     `CIVILIAN_MISSION` 0x15d860 carve-out's sibling, :20-23); the rest re-arm the 0x10000/0x2000
     bits (the port's `vision_on`/`watching`) and start the walk — the run-anim 0x35 at
     alert ≥ 0.66 else the walk 0x37 (:23-31).
   - **UPDATE** (case 3): `look_for_opponent` first, a pick SetStates out (:37-41); else
     `NDrone2_UpdateMissionRoute` 0x154a80 (`FollowRoute` 0x155be0 over `AINetwork_FollowRoute`
     0x15b110) — not arrived and not blocked (creep byte +0x974 ∉ {9, 0xc}) keeps the walk with NO
     anim request (:45-48); blocked at alert ≥ 0.66 does the +0x3b = (+0x228 >> 2) & 1 then
     +0x3b = (!+0x1e) fire-request, `SetAngleToObj` (+0x170 opponent → +0x694, the port's
     `desired_yaw = yaw_to_opp` fold) and the anim-10 aim stand (:49-57); blocked hurt stands idle
     (:60). **Arrival** (code 3): prop 0x1D clears, the route invalidates (+0x9f2 = 0), then
     +0x5A4 (alt state) set with +0x135 (second channel) clear and level ≠ 0x7000006 →
     `NDrone2_ChangeToAttackMode` 0x14a648's return (the port's alt_state/alt_mode flush, the
     mode-2 → 0x19/0x56 arm included); everything else → IDLE (4) (:63-82).
   - **Impacts** 6/8 → `HandleImpact(0x56, 0)`, 7/9/0x17/0x18/0x19 → (0x56, 1); **alerts**
     0x11/0x12/0x14/0x15/0x16/0x1e → `DroneVision_EnemyAlerts`; 0x1f `DoForcedAttack`; 0x21
     `ConsiderExplosive(…, 0x56)` — via the port's shared dispatch rows (see the audit fixes).
2. **The entry transition ported** [M `0015bf18_NDrone2_DSTATE_Idle*` case 1 :30-56,
   `0015c248_NDrone2_DSTATE_Alert*` case 1 :19-37]: the Idle/Alert ENTER now SetStates 0x1b when
   prop 0x1D is set and `AssignAIPath(8)` (0x154190) binds a route (+0x9f2 != 0) — Idle regardless
   of alertness; without a route the prop clears and the pre-existing arms run. The old line only
   cleared the prop on failure and never transitioned.
3. **The disc census** [D, scratch probe `probe_kind8_census.rs` (uncommitted), 2026-10-23]: per
   on-foot level, the class-0x0f placements whose behaviour word carries prop 0x1d AND a kind-8
   AI path node within the assign's 0.3 2D bind, whose type maps through `GetDTYPENEW` 0x14c938 to
   the start-by-mode rows (4 Idle / 6 InitPatrol — no civilian init):
   - 07000001 dtype-2 "Mayhew Route" node 31 (bind 0.129)
   - 07000006 dtype-1 "SnowTalkDronePath - (Dr_13)" (bind 0.013)
   - 0700000d dtype-1 "Mission_G6" node 2 (bind 0.254)
   - 07000012 dtype-3 "MissionPath_Drones enter Office 7" ×2 (binds 0.027/0.069)
   - 07000014 dtype-8 "MissionPath" ×3 (binds 0.023-0.043) — corroborated by the game's own
     `ConsiderAlerted` refusal of EnemyMission alerts on exactly 07000014 [M 0x176250:66]
   - 07000016 dtype-2 "Kiko Walk" (bind 0.152)
   The other kind-8 binds route elsewhere: 07000007's party guests (dtype-0xa/0xb) and
   07000011/12's CivilianMissionPath walkers (dtype-0xe) through CivilianInit/PartyGirlInit →
   CivilianMission 0x16; 07000001's Mayhew (dtype-0xc) starts 0x1c AllyLeadInit; 07000006's
   TruckDriverPath (dtype-0xd) and 0700001b's AstroPath/DrakePath rows are the civilian/astronaut/
   Drake skins. 24 of the 33 on-foot levels carry prop-0x1d placements; only those six levels have
   kind-8 binds.
4. **Tests** (`crates/nf-view/tests/enemy_mission_kind8_live.rs`, both green):
   - `the_snow_talk_guard_follows_its_kind8_mission_path` — the real path on 07000006: the guard
     placed 0.013 u from "SnowTalkDronePath - (Dr_13)" node 0 runs its start script (state 0x3),
     then Idle ENTER → 0x1b carrying a kind-8 route of 10 nodes, walks > 10 u along the path, and
     the arrival arm fires: 0x1b → IDLE with the route dropped [M 0x15e1e8:63-82].
   - `a_level_without_kind8_paths_never_enters_enemy_mission` — the sister: 07000005 places 14
     prop-0x1d guards but its nav carries NO kind-8 path [D census]; asserted
     `paths.all(kind & 8 == 0)` and no drone of the level ever enters 0x1b over 30 s.
   - **Fail-first verified**: with the drone.rs port reverted (git checkout of the pre-fix file)
     the walk test FAILS (no 0x1b entry) while the sister holds; with the port both pass.

## The audit (independent subagent) and its fixes

A branch-by-branch audit against the decompiles confirmed the ENTER/UPDATE port (order, the 0.66
thresholds, the 0x35/0x37/0x21/10/0x56/4 constants, the ChangeToAttackMode flush) and found three
gaps, all fixed:
1. The impact msgs ride the port's SHARED dispatch, not the per-state match rows — `impact_args`
   had no 0x1b entry (the flinch resumed into the current state instead of 0x56), the
   `on_explosion` group allow missed 0x1b (no blast stagger), and `on_dart_impact`'s whitelist
   dropped the tranq dart entirely. All three now carry ENEMY_MISSION with the 0x15e1e8 pairs.
2. msg 0x1e (ALARM) moved from the case-0x14 AlertSound fold to the shared ALARM row (EnemyAlerts
   case 0x1e → Attack) [M 0x177c18:87-118].
3. The blocked/hurt `SetStandIdleAnim(1)` maps to `set_stand_idle(force=false)` — the game's arg 1
   skips a live stand window (0x13ca68:19-21) and the port's force bool is the inverse polarity.

## How it was verified

- The two live tests above (fail-first evidence included).
- The targeted run before every push: `cargo test --offline --release -p nf-view --test
  enemy_mission_kind8_live` — 2 passed.
- `cargo check --offline -p nf-game` clean (the warnings that remain are pre-existing).
- The full four-crate suite (`tools/test/suite.py`) on the final tree (master a67f1a27e + the
  branch, the audit fixes included): the run started 18:10 UTC with `build ok: 738 test binaries
  in 815 s` and was still executing at the 18:33 report cutoff (this sandbox runs the suite at
  ~3× the documented time; the first (pre-fix) run built at 16:58 and had not finished by 17:36
  either). **Partial results at the cutoff — 8 non-passing rows, all environmental, none in the
  ported code path:**
  - `identity_disc` — asserts a full 1,191,072-sector image; the sandbox's
    `Nightfire-onfoot.iso` is the 518,044-sector on-foot cut [D file size 1,061,754,880 B].
  - `disc_golden` ×3 — the documented slim-ISO failures.
  - `fix_0c_pylon_live` ×2 — `wgpu adapter: NotFound (active_backends 0x0)`: the render tests
    need a GPU the sandbox lacks (the merge lessons' "your sandbox has no GPU").
  - `net_soak_faults_live` / `mp_online_failures` / `mp_remote_fx_live` — CRASH exit -9 (SIGKILL,
    the sandbox OOM/kill), `net_internet_live` failed downstream of the same run.
  No nf-game/drone-suite failure appeared in the covered binaries at the cutoff.

## What's left (observations, not blockers)

- The other five census levels' guards are spawner/script-gated at level start (no 0x1b entries
  within 20 s of a plain load: 07000012/14/16/0d/01). A trigger-walk test (e.g. 07000012's office
  drones) would pin the triggered-spawn entry; the state machine is shared with the proven path.
- `assign_path` (0x154190's port) carries a `|dy| ≤ 2.0` filter (drone.rs ~2141) with no
  decompile counterpart (the game's scan is pure 2D). All six census binds are same-floor, so no
  disc impact; flagged for a future pass.
- The game's Idle/Alert ENTER checks +0x9f2 != 0 unconditionally (a stale pending route with the
  prop clear would also enter 0x1b); the port requires `prop && assign`. No live instance
  constructible from the census; a deliberate narrowing, noted here.
- The Idle case-1 Drake arm (0x15bf18:19-24 — level 0x700001b + voice-set 0x14 → SetState 0xc2)
  has no port counterpart anywhere (pre-existing gap, another task's region; flagged by the audit).

## Live checks for the main agent

1. 07000006 fresh boot: the SnowTalk guard (dtype 1, placement ≈ (68.89, 39.29, 27.33), skin
   0x0500002e) — `watch <drone>+0xF4 2` after ~8 s of play: expect the u16 to read 0x1b while he
   walks the snow path, and 0x4 after he reaches the path end (~25 s). Expected arc: 0x3 (start
   script) → 0x1b → 0x4.
2. Same guard mid-walk: `watch <drone>+0x9f2 2` — expect 10 (the kind-8 route's node count)
   during 0x1b, 0 after the arrival, and the behaviour prop 0x1d cleared after.
3. 07000014 fresh boot (the level whose EnemyMission alerts the game refuses): the three dtype-8
   guards at ≈ (−37.7, 1.1, 25.9) / (−36.2, 1.1, 27.8) / (−31.7, 1.1, 25.2) — after their spawn
   trigger, `watch <drone>+0xF4 2` → 0x1b and the +0x974 route-creep byte walking 0/3 (never 9/0xc
   for long) would confirm the triggered-spawn users live.

## Usage

- This agent's own token/credit figures: not visible to me.
- The audit subagent's reported figures (read from its result footer): prompt 191641, completion
  8585, total 200226 tokens.
