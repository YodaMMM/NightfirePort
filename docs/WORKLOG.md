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


