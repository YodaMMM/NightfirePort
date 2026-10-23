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
