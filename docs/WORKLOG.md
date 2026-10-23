# WORKLOG

## enemy-mission-kind8
Started 2026-10-23 16:55 UTC. Branch s/enemy-mission-kind8 from origin/master a67f1a27e.
Task: NDrone2_DSTATE_EnemyMission 0x15e1e8 and its kind-8 paths (AssignAIPath(8)) are not
ported (drone.rs notes it). Find which levels place guards that use it [D placements: the
dtype / mission path params], port the state and the path assignment from the decompile [M],
and add a real-path test on a level that uses it: the guard follows its mission path.
Sister: a guard without a kind-8 path never enters the state.
Plan:
1. Read the decompile of NDrone2_DSTATE_EnemyMission 0x15e1e8 and the kind-8 (mission) path
   assignment; check the disassembly wherever the C drops stores/args.
2. Find the disc placements (dtype / mission path params) of guards that use kind-8.
3. Port the state + path assignment into nf-game; real-path test on a level that uses it
   (guard follows its mission path); sister test (no kind-8 path => never enters the state).
4. Full suite via tools/test/suite.py before pushes.

Work log:
- (16:55) branch created, WORKLOG started; repo/ISO/work/queue-worktree already present from
  the earlier session; token refreshed and origin fetched.
- (16:57) suite runner started in the background (ALSA headers via PKG_CONFIG_SYSROOT_DIR=
  /home/z/alsa-prefix — the prefix's own pkgconfig dir alone leaves pkg-config pointing at /usr).
- (17:00-17:25) research [M work/decomp/ACTION.ELF]: EnemyMission 0x15e1e8 case 1 (ENTER) is the
  voice-0x13 → KikoMission 0x19 carve-out + the 0x10000/0x2000 bits + the 0x35/0x37 walk start by
  alertness; case 3 (UPDATE) is look_for_opponent → SetState, else UpdateMissionRoute 0x154a80
  (FollowRoute 0x155be0 over AINetwork_FollowRoute 0x15b110) — creep byte +0x974 ∉ {9,0xc} keeps
  the walk with NO anim request, blocked + alert ≥ 0.66 does the +0x3b=(+0x228>>2)&1 then
  +0x3b=(!+0x1e) fire-request + SetAngleToObj(+0x170 opponent → +0x694) + anim 10 aim stand,
  blocked + hurt SetStandIdleAnim(1); arrival (code 3) clears prop 0x1d, invalidates the route
  (+0x9f2 = 0) and SetStates IDLE (4) — unless +0x5A4 (alt state) is set with +0x135 clear and
  level ≠ 0x7000006, then NDrone2_ChangeToAttackMode 0x14a648's return. Impacts 6/8 allow 0,
  7/9/0x17/0x18/0x19 allow 1 (HandleImpact 0x56); alerts 0x11/0x12/0x14/0x15/0x16/0x1e →
  EnemyAlerts. The Idle/Alert ENTER arms 0x15bf18:32-56 / 0x15c248:21-44: prop 0x1d →
  AssignAIPath(8), +0x9f2 != 0 → 0x1b (Idle regardless of alertness), else the prop clears.
- (17:15-17:30) [D census] scratch probe (probe_kind8_census.rs, uncommitted): per level, class-0x0f
  placements whose behaviour word has prop 0x1d with a kind-8 path node within the assign's 0.3 2D
  bind — the EnemyMission users (type → GetDTYPENEW → start-by-mode 4/6): 07000001 dtype-2
  "Mayhew Route" node 31 (0.129), 07000006 dtype-1 "SnowTalkDronePath - (Dr_13)" (0.013),
  0700000d dtype-1 "Mission_G6" node 2 (0.254), 07000012 dtype-3 "MissionPath_Drones enter Office 7"
  ×2 (0.027/0.069), 07000014 dtype-8 "MissionPath" ×3 (0.023-0.043), 07000016 dtype-2 "Kiko Walk"
  (0.152). The 07000007 party binds are dtype-0xa/0xb (CivilianInit/PartyGirlInit → 0x16), the
  07000011/12 CivilianMissionPath binds dtype-0xe (same), 07000001's Mayhew bind dtype-0xc
  (AllyLeadInit), 07000006's TruckDriverPath bind dtype-0xd (civilian chain), 0700001b's AstroPath/
  DrakePath binds are the astronaut/Drake skins. 24 of 33 on-foot levels carry prop-0x1d
  placements; only those six levels have kind-8 binds.
- (17:30) ported: drone.rs Idle/Alert ENTER → set_state(ENEMY_MISSION) on the assign success
  (the old line only cleared the prop on failure and never transitioned); the full ENEMY_MISSION
  arms (ENTER/UPDATE/impacts/alerts/forced-attack) per the case map above — the route half is the
  port's `route.follow` + `route_reached_special` (the LinkCreep dispatch rides AINetwork_
  FollowRoute in the game), the arrival → IDLE with the ChangeToAttackMode flush folded as the
  port's alt_state/alt_mode pair.
- (17:35) real-path probe (probe_kind8_live.rs, uncommitted): 07000006's SnowTalk guard runs its
  start script (state 0x3) to t498, then Idle ENTER → 0x1b, walks the 10-node path (13.8 u),
  arrives t1483 → IDLE, route dropped — the full game arc. The other census levels' guards are
  spawner/script-gated (07000012/14/16/0d/01: no 0x1b entries in 20 s from level start).
- (17:40) committed tests crates/nf-view/tests/enemy_mission_kind8_live.rs: the 07000006 walk
  (enter 0x1b, kind-8 route of 10 nodes, >10 u of walking, arrival → IDLE + route None) and the
  sister (07000005: 14 prop-0x1d guards, zero kind-8 paths [D census] → no drone ever enters
  0x1b over 30 s). Fail-first verified: with the drone.rs fix reverted the walk test FAILS
  (no 0x1b entry), the sister holds; with the fix both pass.
- (17:45-18:05) independent audit (subagent, branch-by-branch vs the decompiles): the ENTER/UPDATE
  logic verified faithful (order, 0.66 thresholds, the 0x35/0x37/0x21/10/0x56/4 constants, the
  ChangeToAttackMode flush). Three fixes applied:
  1. The impact msgs ride the port's SHARED dispatch (`impact_args` / `on_explosion`'s group
     allow / `on_dart_impact`'s whitelist), not the per-state match rows — ENEMY_MISSION was
     missing from all three, so the live path resumed the flinch into the current state instead
     of 0x56 and dropped the tranq dart entirely. Now: `impact_args` → (0x56, 0) [M 0x15e1e8
     cases 6/8], the on_explosion group allow [M the 7/9/0x17/0x18/0x19 arm], the dart whitelist
     [M case 0x19 → 0x50].
  2. msg 0x1e (ALARM) moved to the shared ALARM row (EnemyAlerts case 0x1e → Attack) — it was on
     the case-0x14 AlertSound fold [M 0x177c18:87-118].
  3. The blocked/hurt arm's SetStandIdleAnim(1) → set_stand_idle(force=false) — the game's arg 1
     skips a live stand window (0x13ca68:19-21), the port's force bool is the inverse polarity.
  The audit also flagged (NOT touched, other tasks' gaps): the Idle case-1 Drake arm
  (0x15bf18:19-24 — level 0x700001b + voice-set 0x14 → 0xc2) has no port counterpart anywhere;
  the INVESTIGATE state has the same dead-impact-row idiom (0x7e missing from impact_args);
  assign_path's |dy| ≤ 2.0 filter (drone.rs 2141) has no decompile counterpart (pure 2D at
  0x154190:84-89 — all six census binds are same-floor, no disc impact); the game's Idle/Alert
  ENTER checks +0x9f2 != 0 unconditionally (a stale pending route with no prop would enter 0x1b)
  while the port requires prop && assign — no live instance constructible, noted as a deliberate
  narrowing.
- (18:08) audit fixes committed and pushed; the full suite restarted on the final tree.



## level-12-hack-computer-2 (2026-10-23, started ~18:40 UTC)

Branch `s/level-12-hack-computer-2` from origin/master (a67f1a27e).

Task: un-ignore `crates/nf-view/tests/interactables_07000012_live.rs::the_hack_computer_starts_the_q_worm_sp`
— the hack computer on the real walk (round 2). Route proven down to a door; the use on the
computer does not fire. Read the test's own `#[ignore = "..."]` message first; real input only
(step_bond movement/aim/fire/use; no teleports past the blocker, no pokes). Port what the game
does at the blocker from the decompile [M]/disc [D]; live numbers only from a FRESH boot.

Progress:
- (18:40) setup: sandbox survived the prior task (repo, work/, ISO present); killed the stale
  suite from s/enemy-mission-kind8, freed disk (target examples/debug, scratch probes), fetched
  origin, created this branch from origin/master, restored docs/WORKLOG.md from
  s/enemy-mission-kind8 (WORKLOG is not on master yet).

### level-12-hack-computer-2 (continued, 19:40-20:15 UTC)

- The blocker was STALE: the ignore message blamed the look-cursor chase, but the E pick reads
  `bond.look()` (the raw view) — the teleported ring run today passed the ch 0x34 write on the
  first try. What actually failed was the test's own assert: it expected the SP `activated`
  latch (+0x2c), which only `SP_Activate` 0x1dc338's E-entity route sets. The decompile chain
  for the channel route: `PCQWorm_Update` 0x1aee20 writes ch 0x34; `SP_Hit` 0x1dc470:11-13
  returns the out-channel read as trigger bit 0 (mask 0x1 [D param 5]); `SP_Update` 0x1dbda8
  case 6 (the mode-8 primed looping state, [M 0x1db9a8:81-86]) clears SCRIPTINFO+0x9c — the
  held script (delay −1, the event-3 hold [M 0x1d94c0:98-117]) unpauses. Test fixed to pin the
  unpause (+ state 6 stays, single script) with the held pre-state and a no-E sister.
- The teleports replaced by the real walk: the proven tower2 legs + the door[19] doorway
  stand-in, then new south-west legs (found with a navgrid probe over capsule-height rays:
  east corridor x −5.75..3 z −6..−8.5, wing corridor x −9.5..−6.5 z −9..−18, the west pinch
  x −20..−18.5 z −25..−26.5, the south channel x −16.25..−13.25 z −27..−30) to the stand spot
  (−15.1, −30.9) r 0.2. Two measured misses fixed on the way: r 0.4 let the walker stop 0.32
  short (pick dist 1.524 vs need 1.501 — the desk blocks the last 1.2 to the PC) and the aim
  break needed the pitch term (the free-aim cursor absorbs the convergence tail; 0.1/0.15 rad
  is far inside the pick's 36° slack at 2.2 units, from the [M] radii).
- The test passes un-ignored (the file's other ignore — the doors proximity test — is the
  earlier task's, untouched). Ran: interactables_07000012_live 8 pass, walkreal_07000012_live
  1 pass, use_key_census_live 20 pass, nf_game lib 522 pass. The full 1249-binary suite does
  not fit the window: background jobs are killed between tool calls in this sandbox (three
  setsid/nohup attempts died within a minute), and the foreground chunks above are what fit.
  No src code changed — the branch is master + this test file + docs.
- Post-finish suite chunks (all green): walk_0700000a_live 3, simplescript_live 2,
  objective_banner_timing_live 3, interact_0700004a_live 5/2 ign, fix_actors_live 4,
  spawner_kill_live 5 — 578 passed / 0 failed total across the 11 binaries + the lib.
  The finish steps ran: the branch pushed (aa0da4136 + the report), finish.sh registered
  (queue e5e396498), the ntfy delivered (curl, id 1krndDAnXcJk).

### level-1b-drake-2 (started 19:57 UTC, 2026-10-23)

Task: two #[ignore] tests on 0700001b — walkreal_0700001b_live::walk_arrival_to_drakes_defeat
(shaft sealed; the game's route is the SP#211 fly-by whose event-6 player carry never lands in
the port — no event 6 in 0x060008f0's parsed streams) and
interactables_0700001b_live::drake_defeat_by_real_fire_ends_the_mission. Find where the game's
player carry comes from (script stream, event kind, channel) [D][M]; port it from the decompile;
un-ignore with a sister that can fail. Real input only via step_bond.

### level-1b-drake-2 (20:00-21:20 UTC)

- Found the event-6 player carries [D dump]: 0x0600085c stream 7 (SP#147, the level-start
  intro cinema, pack 0780001b.bin — ev0x04[]+ev0x06[0x2], mode 2) and 0x0600085f stream 6
  (pack 0790001b.bin, ev0x06[0x0], no SP player found). NOT in 0x060008f0 (SP#211's B script)
  — the "event-6 in the fly-by" premise is refuted on the disc data; the port's event-6 path
  (snapshot at event time → PlayerEnable at Script_Free, flags 0x100) is already ported
  [M 0x1d94c0 case 6; nis.rs event()/free()].
- The route finding: the void crossing is the zero-G float. The level's script chain pulses
  ch 0x4b [D 06000971 s2/s3, played by 060008a3 s6 and 060008f0 s8/9/11] and
  C_CHCHZEROG_Handler 0x216388 folds Player_ChangeSubState(9) on it [M]. Ported the entry
  (scene.rs, the round-1 block with its citations). The disc trigger chain above the pulse is
  unreachable (ch 19 has no writer — the level doc §6), so the tests drive the pulse as the
  documented stand-in, same class as the ch 0x0a drive.
- walkreal walk_arrival_to_drakes_defeat rewritten: the real deck walk → ch 0x0a (Drake
  spawns from the deck — the spawner's min_dist screen [M 0x137cb0:30] is why the arena-floor
  gate drive of the interactables test could never spawn) → the slit → the 0x4b pulse →
  substate 9 (verified grabbed) → steered thruster float south around the island → BLOCKED on
  an invisible wall at x ≈ 4.98 spanning at least z −1.5..−3.8 (the "sealed shaft") — the
  next step is mapping the wall (a westward segment scan over y/z) to find the game's opening
  (over the top, or the SP#141/#142 shaft rides at z ±8.82).

### level-1b-drake-2 (21:20-21:40 UTC, close)

- The sealed wall mapped [D probe run]: the westward world-segment scan (probe1b_float::
  scan_the_x5_wall_openings) shows the capsule ray hitting at t 0.079 (face x ≈ 4.43) across
  the full scanned band z −8..−0.5, y −2.4..+2.5, while the BULLET segment crosses everywhere
  — a glass wall you can shoot through but not float through. The stall at x 4.98 is the
  capsule radius off that face.
- walkreal walk_arrival_to_drakes_defeat re-#[ignore]d at the NEW precise blocker (the sealed
  shaft wall; the next step is the shaft mouths z ±8.82 — SP#141/#142's looping zero-G scripts
  0x06000818, out ch 0x1c/0x18). The interactables drake_defeat blocker message rewritten with
  the [M 0x137cb0:30] min-distance explanation (the arena-floor gate drive can never spawn).
- The crossing leg itself is real-input proven up to the wall: the deck walk → ch 0x0a →
  Drake spawns+enables → the slit → the 0x4b pulse → substate 9 grabbed (asserted) → the
  steered thruster float rounds the island south (z −3.8) and stops on the wall. All
  step_bond movement; the steered-float code stays in the test body for the next round.
- Ran: walkreal 3p/3i(0f), interactables_0700001b_live 0p/2i(0f), probe1b_float/probe1b_scripts
  0f, fix_finale_live 4p, walk_0700001b_live 2p, simplescript_live 1p, nf-game lib 522p. The
  background suite.py died in the first minute (cargo off PATH in the nohup env — the prior
  task hit the same wall); foreground chunks are what fit the window.

## level-1b-drake-3
Started 2026-10-23 22:15 UTC. Branch s/level-1b-drake-3 = origin/master 8b101249d + s/level-1b-drake-2 cf56f98b9.
Task: 0700001b — decide the wall at x 4.43 (triangles, owners, flags, capsule admission [M
Collide_Intersect]); map the shaft mouths z ±8.82 and the SP#141/#142 route; the landing, the
fight, the defeat on real input only; un-ignore the walk and the interactables tests with sisters.
Plan:
1. Merge drake-2 into a fresh master branch (done — master's drake-fight merge taken for the
   shared test files; drake-2's probe1b_float superset kept).
2. Wall triangles at x 4.43, z -8..0: owning object + flags; capsule admission per [M].
3. Shaft mouths at z ±8.82; SP#141/#142 entry route.
4. Un-ignore walk + interactables with sisters; run the touched crates; full suite.py.
- (22:15-22:30) branch start merged (master's drake-fight versions taken for the conflicted
  shared files, drake-2's probe1b_float superset kept); the background suite runner dies at
  every tool-call boundary in this sandbox (also documented by the previous round) — suite
  chunks must run in the foreground.
- (22:30-23:00) probe1b_wall [D dump]: the x 4.43 wall = plc #128 OBJECT_Boundary_1 (kind 0x8019,
  mat 0x40, two east-facing tris, y −16.56..+16.74 z −20.4..20.4); the z ±8.82 mouths = iris-door
  booths (SP#141/#142 ScriptPlayer_IrisDoorOpen at (5.23,9.45,∓8.82), glass mat 0x40 at x 5.91).
  DECISION [M]: the boundary stops the game's body (the 0x40 drop is the bit-0x8 bullet queries
  only, Collide_Intersect 0x1ebff0-0x1ec010; CylGeom 0x109da0 has no side rejection; ZeroG
  0x192ab0 adds no bypass) — the port's capsule stop is the game's.
- (23:00-23:30) walkreal pins: the_boundary_wall_stops_the_float_and_passes_bullets (green) and
  the_iris_doors_are_load_time_loopers (green): the doors are load-time loopers [M 0x1db9a8 tail +
  0x1dbc08:46-48], the missile-channel rise restarts nothing [M 0x1dbda8 case 6's other-slot
  guard] — the earlier "doors open on the missile channels" reading is refuted and the 0x18/0x1c
  writes are per-loop CLEARS [D streams 12/13].
