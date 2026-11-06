# Drone-family port map

Every `Drone*`, `NDrone2_*`, `DroneFunc_*`, `DroneVision_*` function in ACTION.ELF
(SLUS-20579 NTSC-U, build e7ffdcb3) except the talk functions — 617 in total.
One line each: PORTED with the port's code location, or NOT REACHED with the proof
(a function the shipped disc never dispatches cannot have a port-side test; the task's
MISSING class — a reached behaviour with no port code — is empty).

Method: for each decompile file `work/decomp/ACTION.ELF/<addr>_<name>.c` the exact
address (word-boundary hex) was searched in `crates/*/src`, then the function's base
name; test-only comment citations were not accepted — each of those four was resolved
to its implementing function by hand. The dispatch proofs for the NOT REACHED rows are
docs/systems/ai-dmode-inits.md and docs/port-map/unreached.md.

## The one MISSING the sweep found (fixed on this branch)

The escort-abandonment monitor `NDrone2_ControlDTYPE_Zoe` 0x14b058 — while switch
channel 1 is on and a player exists, `Vec_Dist3D(obj+0x30, glb_players+0x30) > 10.0`
fails the escort every tick through `DroneFunc_SetMissionFailReason` 0x13e560's default
arm (reason 0xf). Ported in `crates/nf-game/src/drone.rs` (the `zoe_escort_monitor`
block), tested red-then-green on the real level by `crates/nf-game/tests/zoe_abandon_live.rs`.

## Table

| Address | Name | Status | Port location / proof |
|---|---|---|---|

Statuses: **ported** = the port's code for it, at the cited location (the port cites the
decompile address on the implementing function, or the name where the fold is named);
**not reached** = the shipped disc never dispatches it, with the proof — porting it would
be dead code and no real-level test can reach it.

Counts: 587 ported, 30 not reached, **0 missing**.

Notes:

- **The 22 excluded talk functions** (`*Talk*`): per the task, out of scope.
- **The `NDrone2_DSTATE_Bot*` states** (multiplayer bots, state ids 0xC3–0xF9): ported as
  the lifted registry `mp/bots.rs::BOT_STATES` — each row carries the game address and the
  kind read off that .c (Full / Thin / Stub / Empty; e.g. `BotGuardian` 0x174a28 is really
  just msg 1 → SetState 0xF9, `BotHeardNoise` 0x1752f8 returns `*msg == 0`). The Full-kind
  states have real implementations in `BotBrain` (`bot_idle_*`, `bot_attack_*`,
  `bot_attack_fire_*`, cover states, `goto_goal`, death/door/guard-friend), and
  `bot_global_dispatch` mirrors the `BotGlobal` 0x16E060 router; `react_to_drone_alert_msg`
  delivers the SeenDeadBody/SeenSurrenderedDrone transitions.
- **The four functions whose only repo citations were test comments** — all resolved to
  their implementing code by hand: `DroneAnim_CanDoAnimState` 0x13dfc8 →
  `drone.rs:10505`; `DroneFunc_DropGrenade` 0x14b158 → `drone.rs:5354` (`drop_grenades`,
  the OnInitDeath captain drop + the pending-grenade release); `NDrone2_DSTATE_HeardNoise`
  0x173be8 → the `HEARD_NOISE` 0xa2 state arm `drone.rs:7862` (family 0xa2–0xa5);
  `DroneWeap_BurstDelay` 0x179ba8 → `drone.rs:11167` (`burst_delay`, the ×1.66/1.33/0.66/0.33
  mode factors and the min).
- **`NDrone2_init_DMODE_Normal` 0x14d958** (DroneModeSettings row 0, the only row the
  shipped disc dispatches): ported — `drone.rs:2394` implements row 0 via
  `init_DMODE_Defaults` 0x14d528's 40 behaviour ids.
- **`NDrone2_init_DMODE_Ambush` 0x14e018**: the row function is never dispatched, but the
  ambush *mode* (0x10) is reached through `GetDTYPENEW` and is ported (`AMBUSH_INIT`
  state 0x37, `drone.rs:6466`).
| 0x1291e8 | `NDrone2_initDTYPE_BotInit` | ported | `crates/nf-game/src/drone.rs:2307` |
| 0x136888 | `Drone_PreLoad_Init` | ported | `crates/nf-game/src/aipoint.rs:58` |
| 0x136938 | `Drone_PostLoad_Init` | ported | `crates/nf-game/src/drone.rs:2597` |
| 0x136be0 | `Drone_Create` | ported | `crates/nf-game/src/breakable.rs:522` |
| 0x136ce8 | `Drone_Delete` | ported | `crates/nf-game/src/drone.rs:1267` |
| 0x136ff0 | `Drone_CoverCornerNode` | ported | `crates/nf-game/src/drone.rs:5430` |
| 0x137340 | `Drone_CoverLowNode` | ported | `crates/nf-game/src/drone.rs:5430` |
| 0x137550 | `Drone_AIPoint` | ported | `crates/nf-game/src/drone.rs:668` |
| 0x1376c0 | `Drone_AIVolume_Create` | ported | `crates/nf-game/src/aipoint.rs:42` |
| 0x1378d0 | `DroneSpawner_Create` | ported | `crates/nf-game/src/drone.rs:12543` |
| 0x137aa0 | `DroneSpawner_Init` | ported | `crates/nf-game/src/spawners.rs:32` |
| 0x137cb0 | `DroneSpawner_SpawnDrone` | ported | `crates/nf-game/src/drone.rs:1901` |
| 0x137e68 | `DroneSpawner_Control` | ported | `crates/nf-game/src/breakable.rs:522` |
| 0x1383e8 | `Drone_EnableAll` | ported | `crates/nf-game/src/drone.rs:12454` |
| 0x138588 | `Drone_BrokenObject` | ported | `crates/nf-game/src/breakable.rs:455` |
| 0x1386d8 | `Drone_IsCoverNodeUsable` | ported | `crates/nf-game/src/drone.rs:12893` |
| 0x138b10 | `Drone_BuildDynamicAwarePoints` | ported | `crates/nf-game/src/drone.rs:11814` |
| 0x138e78 | `Drone_ProcessCoverNodes` | ported | `crates/nf-game/src/drone.rs:18195` |
| 0x138f68 | `Drone_GetOpponentInfo` | ported | `crates/nf-game/src/drone.rs:4324` |
| 0x1390c0 | `Drone_ProcessOpponents` | ported | `crates/nf-game/src/drone.rs:12444` |
| 0x139268 | `Drone_Control` | ported | `crates/nf-game/src/drone.rs:1451` |
| 0x139520 | `Drone_InitComms` | ported | `crates/nf-game/src/drone.rs:1541` |
| 0x139f50 | `Drone_ModBulletDamage` | ported | `crates/nf-game/src/drone.rs:12164` |
| 0x13a0a0 | `Drone_DCVfromOBJ` | ported | `crates/nf-game/src/drone.rs:2315` |
| 0x13a0f8 | `Drone_InitOnce` | ported | `crates/nf-game/src/drone.rs:12540` |
| 0x13a108 | `Drone_CoderCreate` | ported | `crates/nf-game/src/drone.rs:2392` |
| 0x13a1d0 | `Drone_AIVolume_Delete` | ported | `crates/nf-game/src/aivolume.rs:89` |
| 0x13a1f8 | `Drone_VisibilityForPosition` | ported | `crates/nf-game/src/aivolume.rs:85` |
| 0x13a290 | `Drone_PointInAnyAIBox` | ported | `crates/nf-game/src/drone.rs:7191` |
| 0x13a338 | `DroneSpawner_Delete` | ported | `crates/nf-game/src/drone.rs:13953` |
| 0x13a3b8 | `DroneSpawner_DroneDelete` | ported | `crates/nf-game/src/spawners.rs:124` |
| 0x13a438 | `Drone_MessageObjVicinity` | ported | `crates/nf-game/src/drone.rs:21465` |
| 0x13a578 | `Drone_GetSkinHash` | ported | `crates/nf-game/src/triggers.rs:666` |
| 0x13a5a8 | `Drone_LevelReset` | ported | `crates/nf-game/src/drone.rs:4766` |
| 0x13a630 | `Drone_SM_SendMsg` | ported | `crates/nf-game/src/drone.rs:1845` |
| 0x13a678 | `Drone_Message` | ported | `crates/nf-game/src/drone.rs:137` |
| 0x13a710 | `Drone_PauseNoDraw` | ported | `crates/nf-game/src/drone.rs:12545` |
| 0x13a7a0 | `Drone_PositionVisible` | ported | `crates/nf-game/src/drone.rs:11984` |
| 0x13a878 | `Drone_FellOutMap` | ported | `crates/nf-game/src/drone.rs:1889` |
| 0x13a8d8 | `Drone_AlertStatusSet` | ported | `crates/nf-game/src/drone.rs:1300` |
| 0x13a910 | `Drone_AlertStatus` | ported | `crates/nf-game/src/drone.rs:1303` |
| 0x13a940 | `Drone_ModPlayerHitDamage` | ported | `crates/nf-game/src/drone.rs:21105` |
| 0x13a9f8 | `Drone_BulletHit` | ported | `crates/nf-game/src/drone.rs:4687` |
| 0x13aab8 | `Drone_ExplosiveHit` | ported | `crates/nf-game/src/drone.rs:522` |
| 0x13ab58 | `DroneAnim_CallAnim` | ported | `crates/nf-game/src/drone.rs:602` |
| 0x13add0 | `DroneAnim_CallHandler` | ported | `crates/nf-game/src/drone.rs:2972` |
| 0x13aee8 | `DroneAnim_SetDAnimInternal` | ported | `crates/nf-game/src/drone.rs:2782` |
| 0x13b1c8 | `DroneAnim_SetHTAnim` | ported | `crates/nf-game/src/anim.rs:471` |
| 0x13b490 | `DroneAnim_PreTransCallback` | ported | `crates/nf-game/src/drone.rs:1744` |
| 0x13b750 | `DroneAnim_EventFunc` | ported | `crates/nf-game/src/drone.rs:497` |
| 0x13b9e0 | `DroneAnim_CanCrouchLean` | ported | `crates/nf-game/src/drone.rs:5428` |
| 0x13bb38 | `DroneAnim_CanCrouchStepOut` | ported | `crates/nf-game/src/drone.rs:5428` |
| 0x13bc60 | `DroneAnim_CanStandLean` | ported | `crates/nf-game/src/drone.rs:5429` |
| 0x13bde8 | `DroneAnim_CanStandStepOut` | ported | `crates/nf-game/src/drone.rs:5429` |
| 0x13bf40 | `DroneAnim_ValidateCoverAnim` | ported | `crates/nf-game/src/drone.rs:6238` |
| 0x13c078 | `DroneAnim_CoverAnim` | ported | `crates/nf-game/src/drone.rs:5425` |
| 0x13c780 | `DroneAnim_LocationDeathAnim` | ported | `crates/nf-game/src/drone.rs:2937` |
| 0x13c8b8 | `DroneAnim_LocationImpactAnim` | ported | `crates/nf-game/src/drone.rs:4026` |
| 0x13ca68 | `DroneAnim_SetStandIdleAnim` | ported | `crates/nf-game/src/drone.rs:9117` |
| 0x13cc70 | `DroneAnim_SetWalkIdleAnim` | ported | `crates/nf-game/src/drone.rs:2529` |
| 0x13ce00 | `DroneAnim_GetMoveQuad` | ported | `crates/nf-game/src/drone.rs:10986` |
| 0x13cfb0 | `DroneAnim_SetCombatMoveAnim` | ported | `crates/nf-game/src/drone.rs:46` |
| 0x13da48 | `DroneAnim_PlayFiringAnim` | ported | `crates/nf-game/src/drone.rs:1471` |
| 0x13dae8 | `DroneAnim_CheckForAnimComplete` | ported | `crates/nf-game/src/drone.rs:3038` |
| 0x13db30 | `DroneAnim_GetDAnimForCall` | ported | `crates/nf-formats/src/drone.rs:90` |
| 0x13dbf8 | `DroneAnim_CanSetAnimCall` | ported | `crates/nf-game/src/drone.rs:2978` (name cite) |
| 0x13dc90 | `DroneAnim_ZeroAnimCall` | ported | `crates/nf-game/src/drone.rs:2932` |
| 0x13dcb8 | `DroneAnim_CallFullyComplete` | ported | `crates/nf-game/src/drone.rs:2930` |
| 0x13dd00 | `DroneAnim_SetEndAIState` | ported | `crates/nf-game/src/drone.rs:2931` |
| 0x13dd78 | `DroneAnim_SetDAnimNow` | ported | `crates/nf-game/src/drone.rs:2889` |
| 0x13ddd0 | `DroneAnim_SetAnimD` | ported | `crates/nf-view/src/scene.rs:332` (name cite) |
| 0x13de40 | `DroneAnim_GetDAnimForAnimState` | ported | `crates/nf-formats/src/drone.rs:5` |
| 0x13def8 | `DroneAnim_SetAnimCall` | ported | `crates/nf-game/src/drone.rs:2973` |
| 0x13df38 | `DroneAnim_SetEndAnim` | ported | `crates/nf-formats/src/drone.rs:357` (name cite) |
| 0x13dfc8 | `DroneAnim_CanDoAnimState` | ported | `crates/nf-game/src/drone.rs:563` (name cite) |
| 0x13e088 | `DroneAnim_SetScript` | ported | `crates/nf-game/src/drone.rs:2898` |
| 0x13e198 | `DroneAnim_SnapRotate` | ported | `crates/nf-game/src/drone.rs:1450` |
| 0x13e230 | `DroneAnim_SnapRotate2` | ported | `crates/nf-game/src/drone.rs:1451` |
| 0x13e238 | `DroneAnim_OnStartAnim` | ported | `crates/nf-game/src/drone.rs:2872` (name cite) |
| 0x13e2c0 | `DroneAnim_OnEndAnim` | ported | `crates/nf-game/src/drone.rs:1449` |
| 0x13e378 | `DroneAnim_CurrentMoveState` | ported | `crates/nf-game/src/drone.rs:11518` |
| 0x13e3b0 | `DroneAnim_InTransition` | ported | `crates/nf-game/src/drone.rs:10939` |
| 0x13e3e8 | `DroneAnim_AnimEndCallback` | ported | `crates/nf-game/src/anim.rs:471` |
| 0x13e3f8 | `DroneAnim_ForceCrouchCover` | ported | `crates/nf-game/src/drone.rs:5432` |
| 0x13e450 | `DroneAnim_CanUseCornerAsLowCover` | ported | `crates/nf-game/src/drone.rs:5447` |
| 0x13e4d8 | `DroneData_GetAnimFuncInfo` | ported | `crates/nf-formats/src/drone.rs:107` |
| 0x13e520 | `DroneData_GetDAStateFuncInfo` | ported | `crates/nf-game/src/drone.rs:2836` |
| 0x13e560 | `DroneFunc_SetMissionFailReason` | ported | `crates/nf-game/src/drone.rs:3844` |
| 0x13e660 | `DroneFunc_SetDeathChannel` | ported | `crates/nf-game/src/drone.rs:2101` |
| 0x13fff0 | `NDrone2_DealWithObjHit` | ported | `crates/nf-game/src/drone.rs:491` |
| 0x140660 | `Drone_FeetOnPoint` | ported | `crates/nf-game/src/drone.rs:1747` |
| 0x140968 | `Drone_CollisionHandler` | ported | `crates/nf-game/src/drone.rs:138` |
| 0x140e90 | `NDrone2_Collision` | ported | `crates/nf-game/src/drone.rs:1282` |
| 0x141150 | `NDrone2_GetHeadPosOld` | ported | `crates/nf-game/src/drone.rs:11514` |
| 0x1412e0 | `NDrone2_AddToGroup` | ported | `crates/nf-game/src/drone.rs:13950` |
| 0x141360 | `NDrone2_SetDronePartner` | ported | `crates/nf-game/src/drone.rs:1370` |
| 0x1416d8 | `NDrone2_DoorGuardHandler` | ported | `crates/nf-game/src/drone.rs:1373` |
| 0x141848 | `NDrone2_PlaySFX` | ported | `crates/nf-game/src/drone.rs:1487` |
| 0x141b00 | `NDrone2_SpeechSFX` | ported | `crates/nf-game/src/anim.rs:708` |
| 0x141e58 | `NDrone2_DoGravity` | ported | `crates/nf-game/src/drone.rs:990` |
| 0x141f28 | `DroneFunc_AllocateTargetID` | ported | `crates/nf-game/src/drone.rs:1828` |
| 0x141fe8 | `NDrone2_SetOpponentAimPos` | ported | `crates/nf-game/src/drone.rs:11456` |
| 0x1422d0 | `NDrone2_SetOpponent` | ported | `crates/nf-game/src/drone.rs:1085` |
| 0x1424e0 | `NDrone2_FindOpponent` | ported | `crates/nf-game/src/drone.rs:952` |
| 0x1431c0 | `DroneFunc_ConsiderExplosive` | ported | `crates/nf-game/src/drone.rs:239` |
| 0x1435f0 | `DroneFunc_HandleExplosives` | ported | `crates/nf-game/src/drone.rs:1061` |
| 0x1437a0 | `NDrone2_ReactToDroneAlertMsg` | ported | `crates/nf-game/src/drone.rs:6735` |
| 0x143e68 | `NDrone2_DroneNearPos` | ported | `crates/nf-game/src/drone.rs:2645` |
| 0x144030 | `NDrone2_SetupHostageKiller` | ported | `crates/nf-game/src/drone.rs:58` |
| 0x1441c0 | `DroneFunc_HostageSaved` | ported | `crates/nf-game/src/drone.rs:1705` |
| 0x145150 | `NDrone2_SeenAndAttacking` | ported | `crates/nf-game/src/drone.rs:1017` |
| 0x1453a8 | `DroneFunc_CheckAlarmRaised` | ported | `crates/nf-game/src/drone.rs:12607` |
| 0x145558 | `NDrone2_HitDamage` | ported | `crates/nf-game/src/drone.rs:1801` |
| 0x1458e0 | `NDrone2_DoHitEffects` | ported | `crates/nf-game/src/drone.rs:15571` |
| 0x145bd0 | `DroneFunc_OnInitDeath` | ported | `crates/nf-game/src/drone.rs:1295` |
| 0x145e28 | `DroneFunc_SendHurtMessage` | ported | `crates/nf-game/src/drone.rs:466` |
| 0x145f28 | `NDrone2_PunchImpact` | ported | `crates/nf-game/src/drone.rs:115` |
| 0x1461c8 | `NDrone2_BulletImpact` | ported | `crates/nf-game/src/drone.rs:3442` |
| 0x146480 | `NDrone2_ExplosiveImpact` | ported | `crates/nf-game/src/drone.rs:128` |
| 0x1469b8 | `DroneFunc_HandleImpact` | ported | `crates/nf-game/src/drone.rs:126` |
| 0x147188 | `DroneFunc_RecoverTime` | ported | `crates/nf-game/src/drone.rs:202` |
| 0x1472c8 | `NDrone2_DoHeadTracking` | ported | `crates/nf-game/src/anim.rs:567` |
| 0x147578 | `NDrone2_DoHeadTrackingBehaviour` | ported | `crates/nf-game/src/drone.rs:1531` |
| 0x147918 | `NDrone2_AimTest` | ported | `crates/nf-game/src/drone.rs:3923` |
| 0x1479d8 | `NDrone2_DoTracking` | ported | `crates/nf-game/src/anim.rs:561` |
| 0x147c48 | `DroneFunc_FirstSightState` | ported | `crates/nf-game/src/drone.rs:271` |
| 0x147f58 | `DroneFunc_FirstAlertState` | ported | `crates/nf-game/src/drone.rs:274` |
| 0x148010 | `DroneFunc_FirstAttack` | ported | `crates/nf-game/src/drone.rs:1224` |
| 0x148338 | `DroneFunc_CombatState` | ported | `crates/nf-game/src/drone.rs:407` |
| 0x148a30 | `DroneFunc_HandleSoundAlerts` | ported | `crates/nf-game/src/drone.rs:1051` |
| 0x148c60 | `DroneFunc_DoForcedAttack` | ported | `crates/nf-game/src/drone.rs:1330` |
| 0x148db0 | `NDrone2_PreDroneControl` | ported | `crates/nf-game/src/drone.rs:988` |
| 0x1490f0 | `NDrone2_ControlSTANDARD` | ported | `crates/nf-game/src/drone.rs:953` |
| 0x149600 | `NDrone2_CreateNinjaEyes` | ported | `crates/nf-game/src/drone.rs:9759` |
| 0x149768 | `NDrone2_Player` | ported | `crates/nf-game/src/drone.rs:18221` |
| 0x1497e8 | `DroneFunc_ChooseSFX` | ported | `crates/nf-game/src/breakable.rs:512` |
| 0x1499d0 | `DroneFunc_ClampPosToCollisionCylinder` | ported | `crates/nf-game/src/breakable.rs:383` |
| 0x149a80 | `NDrone2_FindCel` | ported | `crates/nf-game/src/world.rs:1864` |
| 0x149aa8 | `NDrone2_GetHeadPos` | ported | `crates/nf-game/src/drone.rs:11513` |
| 0x149b88 | `NDrone2_FeetPos` | ported | `crates/nf-game/src/drone.rs:2444` (name cite) |
| 0x149bd0 | `DroneFunc_GetBonePos` | ported | `crates/nf-game/src/drone.rs:11479` |
| 0x149ca0 | `NDrone2_SetupGroups` | ported | `crates/nf-game/src/drone.rs:13949` |
| 0x149d58 | `NDrone2_SpeechSFXd` | ported | `crates/nf-game/src/drone.rs:4038` |
| 0x149dd0 | `NDrone2_Enable` | ported | `crates/nf-game/src/drone.rs:4773` |
| 0x149ec8 | `NDrone2_DoCollision` | ported | `crates/nf-game/src/drone.rs:990` |
| 0x149f88 | `NDrone2_DoAnimation` | ported | `crates/nf-game/src/drone.rs:3001` |
| 0x14a008 | `DroneFunc_TargetSeen` | ported | `crates/nf-game/src/drone.rs:12891` |
| 0x14a060 | `NDrone2_CheckSurrender` | ported | `crates/nf-game/src/drone.rs:10491` |
| 0x14a168 | `NDrone2_CheckUnsurrender` | ported | `crates/nf-game/src/drone.rs:10498` |
| 0x14a218 | `NDrone2_FadeOut` | ported | `crates/nf-game/src/drone.rs:4784` (name cite) |
| 0x14a298 | `DroneFunc_DeadDrone` | ported | `crates/nf-game/src/drone.rs:7178` |
| 0x14a2f0 | `NDrone2_SetAsDead` | ported | `crates/nf-game/src/drone.rs:2449` |
| 0x14a3b8 | `NDrone2_SetIdleTimeOut` | ported | `crates/nf-game/src/drone.rs:2504` |
| 0x14a410 | `NDrone2_SendAttackMessage` | ported | `crates/nf-game/src/mp/bots.rs:319` |
| 0x14a498 | `NDrone2_DroneAlertToPosition` | ported | `crates/nf-game/src/drone.rs:6684` |
| 0x14a538 | `NDrone2_DroneAlertToObject` | ported | `crates/nf-game/src/drone.rs:9024` |
| 0x14a5e8 | `DroneFunc_Using2ndBehaviour` | ported | `crates/nf-game/src/drone.rs:927` |
| 0x14a5f0 | `DroneFunc_Set1stBehaviour` | ported | `crates/nf-game/src/drone.rs:929` |
| 0x14a618 | `DroneFunc_Set2ndBehaviour` | ported | `crates/nf-game/src/drone.rs:928` |
| 0x14a648 | `NDrone2_ChangeToAttackMode` | ported | `crates/nf-game/src/drone.rs:928` |
| 0x14a6e8 | `NDrone2_NearDrone` | ported | `crates/nf-game/src/drone.rs:3944` |
| 0x14a718 | `DroneFunc_AnyoneNear` | ported | `crates/nf-game/src/drone.rs:1408` |
| 0x14a800 | `NDrone2_DroneInSights` | ported | `crates/nf-game/src/drone.rs:8063` |
| 0x14a808 | `NDrone2_SetupHostageExecute` | ported | `crates/nf-game/src/drone.rs:1711` |
| 0x14a940 | `NDrone2_FindElevatorDoor` | ported | `crates/nf-game/src/drone.rs:1581` |
| 0x14aa00 | `NDrone2_KickObject` | ported | `crates/nf-game/src/drone.rs:345` |
| 0x14aa38 | `NDrone2_OpenDoor` | ported | `crates/nf-game/src/drone.rs:1582` |
| 0x14aaa0 | `NDrone2_DoorIsOpen` | ported | `crates/nf-game/src/drone.rs:1582` |
| 0x14aae0 | `NDrone2_IsCivilian` | ported | `crates/nf-game/src/drone.rs:2299` |
| 0x14aaf8 | `NDrone2_HostageSituationActive` | ported | `crates/nf-game/src/drone.rs:9299` |
| 0x14ab00 | `NDrone2_LogDroneDeath` | ported | `crates/nf-game/src/drone.rs:10358` |
| 0x14aba8 | `NDrone2_ExplosionDir` | ported | `crates/nf-game/src/drone.rs:1227` |
| 0x14ac60 | `DroneFunc_ReactionTime` | ported | `crates/nf-game/src/drone.rs:11503` |
| 0x14ad50 | `NDrone2_ObstructedPatrolPath` | ported | `crates/nf-game/src/drone.rs:7695` |
| 0x14ad58 | `NDrone2_HasOpponent` | ported | `crates/nf-game/src/drone.rs:958` |
| 0x14ada0 | `NDrone2_HeadTrackObj` | ported | `crates/nf-game/src/drone.rs:1531` |
| 0x14aed8 | `DroneFunc_SetAsAttacking` | ported | `crates/nf-game/src/drone.rs:8000` |
| 0x14af80 | `DroneFunc_AttackNoWeapon` | ported | `crates/nf-game/src/drone.rs:2151` |
| 0x14aff8 | `NDrone2_PostDroneControl` | ported | `crates/nf-game/src/drone.rs:2449` |
| 0x14b058 | `NDrone2_ControlDTYPE_Zoe` | ported | `crates/nf-game/src/drone.rs:20097` |
| 0x14b0f8 | `NDrone2_ControlDTYPE_Ninja` | ported | `crates/nf-game/src/drone.rs:9761` |
| 0x14b128 | `NDrone2_ControlDTYPE_Astronaut` | ported | `crates/nf-game/src/drone.rs:2326` |
| 0x14b158 | `DroneFunc_DropGrenade` | ported | `crates/nf-game/src/drone.rs:1551` (name cite) |
| 0x14b190 | `DroneFunc_NearPlayerCollision` | ported | `crates/nf-game/src/drone.rs:989` |
| 0x14b1e8 | `DroneInit_Collision` | ported | `crates/nf-game/src/drone.rs:2180` |
| 0x14b300 | `NDrone2_DefaultInit` | ported | `crates/nf-game/src/drone.rs:94` |
| 0x14c600 | `NDrone2_DoModeSettingsNEW` | ported | `crates/nf-game/src/drone.rs:1305` |
| 0x14c938 | `NDrone2_GetDTYPENEW` | ported | `crates/nf-game/src/drone.rs:397` |
| 0x14cbd8 | `NDrone2_GetDroneTypeAttackTypeFriend` | ported | `crates/nf-game/src/drone.rs:56` |
| 0x14cfa0 | `NDrone2_DoModeSettingsOLD` | ported | `crates/nf-game/src/drone.rs:2396` |
| 0x14d2e8 | `NDrone2_DoTypeSettingsOLD` | ported | `crates/nf-game/src/drone.rs:2189` |
| 0x14d528 | `NDrone2_init_DMODE_Defaults` | ported | `crates/nf-game/src/drone.rs:2395` |
| 0x14d7c8 | `NDrone2_CreateObj` | ported | `crates/nf-game/src/breakable.rs:510` |
| 0x14d890 | `NDrone2_PostLoad_Init` | ported | `crates/nf-game/src/drone.rs:2079` |
| 0x14d928 | `DroneInit_TakeXHits` | ported | `crates/nf-game/src/drone.rs:2208` |
| 0x14d958 | `NDrone2_init_DMODE_Normal` | ported | `crates/nf-game/src/drone.rs:2394` |
| 0x14d978 | `NDrone2_init_DMODE_Guard` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14d998 | `NDrone2_init_DMODE_Retreater` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14d9b8 | `NDrone2_init_DMODE_Sniper` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14da78 | `NDrone2_init_DMODE_Stealth` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dab0 | `NDrone2_init_DMODE_Attacker` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dad0 | `NDrone2_init_DMODE_RunToPoint` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14db18 | `NDrone2_init_DMODE_Assassin` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14db38 | `NDrone2_init_DMODE_HostageKiller` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dc18 | `NDrone2_init_DMODE_Hostage` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dc88 | `NDrone2_init_DMODE_HostageTied` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dcf8 | `NDrone2_init_DMODE_JustStand4Demo` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dd20 | `NDrone2_init_DMODE_DeleteMe` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dd28 | `NDrone2_init_DMODE_Civilian` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14de48 | `NDrone2_init_DMODE_CivilianScared` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14df28 | `NDrone2_init_DMODE_MissionFailer` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14df68 | `NDrone2_init_DMODE_Mayhew` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14df90 | `NDrone2_init_DMODE_Ninja` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14df98 | `NDrone2_init_DMODE_AlarmRaiser` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14dfe0 | `NDrone2_init_DMODE_SearchLight` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e018 | `NDrone2_init_DMODE_Ambush` | not reached | DoModeSettingsOLD row 20 never dispatched (ai-dmode-inits.md); ambush **mode** 0x10 itself is reached via GetDTYPENEW and ported — drone.rs `AMBUSH_INIT` 0x37 + drone.rs:6466 |
| 0x14e050 | `NDrone2_init_DMODE_Zoe` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e098 | `NDrone2_init_DMODE_PartyGirl` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e158 | `NDrone2_init_DMODE_CivilianGuard` | not reached | drone.rs:2323 ("unreachable") — row 23, same dispatch proof as rows 1-22 (unreached.md Corrections 2026-10-24) |
| 0x14e238 | `NDrone2_init_DMODE_Interogator` | not reached | row 8 never applied (drone.rs:402: Defaults-only stub, no producer) |
| 0x14e258 | `NDrone2_init_DMODE_CivDoorGuard` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e348 | `NDrone2_init_DMODE_TruckDriver` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e370 | `NDrone2_init_DMODE_CastleChatGuard1` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e3a0 | `NDrone2_init_DMODE_CastleChatGuard2` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e3e0 | `NDrone2_init_DMODE_SniperAlert` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e490 | `NDrone2_init_DMODE_PartyGirlLooker` | not reached | DroneModeSettings row, docs/port-map/unreached.md + docs/systems/ai-dmode-inits.md |
| 0x14e568 | `NDrone2_init_DMODE_Bot` | ported | `crates/nf-game/src/drone.rs:2404` |
| 0x14e570 | `NDrone2_initDTYPE_Sniper` | ported | `crates/nf-game/src/drone.rs:2310` |
| 0x14e580 | `NDrone2_initDTYPE_SniperAlert` | ported | `crates/nf-game/src/drone.rs:2311` |
| 0x14e6f0 | `NDrone2_EvaluateObstacle` | ported | `crates/nf-game/src/drone.rs:1664` |
| 0x14eba8 | `NDrone2_Steer` | ported | `crates/nf-game/src/drone.rs:1663` |
| 0x14eff8 | `NDrone2_Move` | ported | `crates/nf-game/src/drone.rs:1726` |
| 0x14f140 | `DroneMove_SetBoundryFlags` | ported | `crates/nf-game/src/drone.rs:10522` |
| 0x14f640 | `NDrone2_CanStepLeft` | ported | `crates/nf-game/src/drone.rs:1672` |
| 0x14f808 | `NDrone2_CanStepRight` | ported | `crates/nf-game/src/drone.rs:1686` |
| 0x14f9e8 | `NDrone2_CanStrafeLeft` | ported | `crates/nf-game/src/drone.rs:1686` |
| 0x14fbe0 | `NDrone2_CanStrafeRight` | ported | `crates/nf-game/src/drone.rs:1686` |
| 0x14fdf8 | `NDrone2_CanStrafeDodgeLeft` | ported | `crates/nf-game/src/drone.rs:1687` |
| 0x14ffc0 | `NDrone2_CanStrafeDodgeRight` | ported | `crates/nf-game/src/drone.rs:1687` |
| 0x1501a0 | `NDrone2_CanBackoff` | ported | `crates/nf-game/src/drone.rs:10549` |
| 0x150338 | `NDrone2_CanRollLeft` | ported | `crates/nf-game/src/drone.rs:1687` |
| 0x150500 | `NDrone2_CanRollRight` | ported | `crates/nf-game/src/drone.rs:1672` |
| 0x1506d8 | `NDrone2_EvasiveMove` | ported | `crates/nf-game/src/drone.rs:10537` |
| 0x150ad0 | `DroneMove_NoBunching` | ported | `crates/nf-game/src/drone.rs:1662` |
| 0x150c18 | `NDrone2_ChooseCombatMove` | ported | `crates/nf-game/src/drone.rs:1698` |
| 0x1512e0 | `NDrone2_AnimForDist` | ported | `crates/nf-game/src/mp/bots.rs:3508` |
| 0x1514d8 | `NDrone2_UpdatePatrolRoute` | ported | `crates/nf-game/src/drone.rs:2538` |
| 0x151618 | `NDrone2_MoveToObject` | ported | `crates/nf-game/src/drone.rs:10944` |
| 0x1517d8 | `NDrone2_MoveToGoalPosition` | ported | `crates/nf-game/src/drone.rs:5931` |
| 0x151900 | `NDrone2_SetAngleToCover` | ported | `crates/nf-game/src/drone.rs:6231` (name cite) |
| 0x1519e0 | `NDrone2_MoveTo_RouteNode` | ported | `crates/nf-game/src/drone.rs:9559` |
| 0x151b18 | `NDrone2_ClosestAIPoint` | ported | `crates/nf-game/src/drone.rs:669` |
| 0x151d68 | `NDrone2_CoverNodeOK` | ported | `crates/nf-game/src/drone.rs:5425` |
| 0x152028 | `NDrone2_GetPosForCornerCover` | ported | `crates/nf-game/src/cover.rs:113` |
| 0x152250 | `DroneMove_FindSafetyFromScaryPosition` | ported | `crates/nf-game/src/drone.rs:5580` |
| 0x152548 | `NDrone2_FindCover` | ported | `crates/nf-game/src/drone.rs:5487` |
| 0x152958 | `NDrone2_FindAmbushCover` | ported | `crates/nf-game/src/drone.rs:5517` |
| 0x153008 | `NDrone2_FindAlarmPoint` | ported | `crates/nf-game/src/drone.rs:1013` |
| 0x153168 | `NDrone2_FindRunToPoint` | ported | `crates/nf-game/src/drone.rs:5805` |
| 0x153358 | `NDrone2_ReachedDestNode` | ported | `crates/nf-game/src/drone.rs:342` |
| 0x153720 | `NDrone2_InitAIPath` | ported | `crates/nf-game/src/drone.rs:7663` |
| 0x153a30 | `NDrone2_ReFindMissionPath` | ported | `crates/nf-game/src/drone.rs:5662` |
| 0x153f50 | `DroneMove_ObjectMissionPathDistance` | ported | `crates/nf-game/src/drone.rs:1414` |
| 0x154190 | `NDrone2_AssignAIPath` | ported | `crates/nf-game/src/drone.rs:1030` |
| 0x1543f8 | `NDrone2_SetGoalToDroneWithLosToOpponent` | ported | `crates/nf-game/src/drone.rs:5987` |
| 0x154568 | `DroneMove_NextRouteAngle` | ported | `crates/nf-game/src/nav.rs:701` (name cite) |
| 0x154618 | `DroneMove_AstronautCombat` | ported | `crates/nf-game/src/drone.rs:1734` |
| 0x154838 | `NDrone2_CanMoveToRelativePos` | ported | `crates/nf-game/src/drone.rs:10532` |
| 0x154960 | `NDrone2_CanAimCrouch` | ported | `crates/nf-game/src/drone.rs:10594` |
| 0x1549e0 | `NDrone2_CanAltAttack` | ported | `crates/nf-game/src/drone.rs:10506` |
| 0x154a80 | `NDrone2_UpdateMissionRoute` | ported | `crates/nf-game/src/drone.rs:2639` |
| 0x154aa8 | `NDrone2_SetSoundAlertRoute` | ported | `crates/nf-game/src/drone.rs:7892` |
| 0x154b20 | `NDrone2_SetDroneAlertRoute` | ported | `crates/nf-game/src/drone.rs:2475` |
| 0x154ba8 | `NDrone2_MoveToAlertPosition` | ported | `crates/nf-game/src/drone.rs:5408` |
| 0x154bd0 | `NDrone2_SlideToPosition` | ported | `crates/nf-game/src/drone.rs:6001` (name cite) |
| 0x154cb0 | `DroneMove_TurnToObj` | ported | `crates/nf-game/src/drone.rs:8253` (name cite) |
| 0x154d58 | `NDrone2_SetAngleToObj` | ported | `crates/nf-game/src/drone.rs:8490` |
| 0x154e20 | `NDrone2_SetAngleToDest` | ported | `crates/nf-game/src/mp/bots.rs:331` |
| 0x154e90 | `NDrone2_SetAngleToAlert` | ported | `crates/nf-game/src/drone.rs:7881` |
| 0x154f00 | `NDrone2_GetAngleDifferenceToAlert` | ported | `crates/nf-game/src/drone.rs:7879` |
| 0x154f60 | `NDrone2_BondIsFacingMe` | ported | `crates/nf-game/src/drone.rs:6426` |
| 0x154f98 | `NDrone2_BondWithinAngle` | ported | `crates/nf-game/src/drone.rs:2630` |
| 0x155020 | `NDrone2_InvalidateAttackRoute` | ported | `crates/nf-game/src/drone.rs:8061` |
| 0x155050 | `NDrone2_AtDest` | ported | `crates/nf-game/src/drone.rs:5935` |
| 0x1550c8 | `NDrone2_MoveTarget_IsRouteNode` | ported | `crates/nf-game/src/drone.rs:5936` |
| 0x1551c0 | `NDrone2_MoveTest` | ported | `crates/nf-game/src/drone.rs:3364` (name cite) |
| 0x155330 | `NDrone2_MoveTest` | ported | `crates/nf-game/src/drone.rs:10520` |
| 0x1554a8 | `NDrone2_MoveTestFromNode` | ported | `crates/nf-game/src/nav.rs:184` |
| 0x155620 | `NDrone2_MoveTestToNode` | ported | `crates/nf-game/src/nav.rs:185` |
| 0x155798 | `NDrone2_DistanceToEmitter` | ported | `crates/nf-game/src/mp/bots.rs:2065` (name cite) |
| 0x155858 | `NDrone2_DisownCover` | ported | `crates/nf-game/src/drone.rs:5479` (name cite) |
| 0x1558d8 | `NDrone2_NearestNode` | ported | `crates/nf-game/src/nav.rs:274` |
| 0x155958 | `NDrone2_NearestNodeDCV` | ported | `crates/nf-game/src/aipoint.rs:72` |
| 0x1559f8 | `DroneMove_FindSafetyFromScaryObject` | ported | `crates/nf-game/src/drone.rs:5580` |
| 0x155a38 | `NDrone2_CoverAvailable` | ported | `crates/nf-game/src/drone.rs:5683` |
| 0x155ad0 | `NDrone2_CanReachGoalPosition` | ported | `crates/nf-game/src/drone.rs:5936` |
| 0x155b30 | `NDrone2_SetRunToPosition` | ported | `crates/nf-game/src/drone.rs:5937` |
| 0x155be0 | `NDrone2_FollowRoute` | ported | `crates/nf-game/src/drone.rs:9081` |
| 0x155cd8 | `NDrone2_NavNodeCache` | ported | `crates/nf-game/src/drone.rs:1132` (name cite) |
| 0x155d48 | `NDrone2_SetupAlertTarget` | ported | `crates/nf-game/src/drone.rs:5938` |
| 0x155d78 | `NDrone2_DistanceToAIPoint` | ported | `crates/nf-game/src/mp/bots.rs:1912` |
| 0x155db8 | `NDrone2_GetAnglesToMoveTarget` | ported | `crates/nf-game/src/drone.rs:5937` |
| 0x155e10 | `NDrone2_SetGoalToCloserPosition` | ported | `crates/nf-game/src/drone.rs:5975` |
| 0x155e90 | `DroneMove_DistanceToNextRouteNode` | ported | `crates/nf-game/src/nav.rs:701` (name cite) |
| 0x155ea0 | `DroneMove_AstronautAnim` | ported | `crates/nf-game/src/drone.rs:613` |
| 0x155f78 | `DroneMove_AstronautHitMove` | ported | `crates/nf-game/src/drone.rs:7153` |
| 0x156008 | `DroneMove_AstronautDeathMove` | ported | `crates/nf-game/src/drone.rs:1732` |
| 0x15b830 | `NDrone2_DSTATE_Global` | ported | `crates/nf-game/src/drone.rs:56` |
| 0x15b960 | `NDrone2_DSTATE_WaitSwitch` | ported | `crates/nf-game/src/drone.rs:3485` |
| 0x15ba68 | `NDrone2_DSTATE_PlayScript` | ported | `crates/nf-game/src/drone.rs:18` |
| 0x15bf18 | `NDrone2_DSTATE_Idle` | ported | `crates/nf-game/src/drone.rs:284` |
| 0x15c248 | `NDrone2_DSTATE_Alert` | ported | `crates/nf-game/src/drone.rs:284` |
| 0x15c498 | `NDrone2_DSTATE_Patrol` | ported | `crates/nf-game/src/drone.rs:284` |
| 0x15c728 | `NDrone2_DSTATE_HostageKiller` | ported | `crates/nf-game/src/drone.rs:54` |
| 0x15c940 | `NDrone2_DSTATE_HostageKillerAttack` | ported | `crates/nf-game/src/drone.rs:61` |
| 0x15cb78 | `NDrone2_DSTATE_Hostage` | ported | `crates/nf-game/src/drone.rs:67` |
| 0x15ccf0 | `NDrone2_DSTATE_HostageSaved` | ported | `crates/nf-game/src/drone.rs:303` |
| 0x15ce60 | `NDrone2_DSTATE_HostageIdle` | ported | `crates/nf-game/src/drone.rs:250` |
| 0x15cfe0 | `NDrone2_DSTATE_HostageHide` | ported | `crates/nf-game/src/drone.rs:307` |
| 0x15d118 | `NDrone2_DSTATE_Civilian` | ported | `crates/nf-game/src/drone.rs:248` |
| 0x15d3a0 | `NDrone2_DSTATE_CivilianScared` | ported | `crates/nf-game/src/drone.rs:248` |
| 0x15d520 | `NDrone2_DSTATE_CivilianHiding` | ported | `crates/nf-game/src/drone.rs:248` |
| 0x15d610 | `NDrone2_DSTATE_CivilianPatrol` | ported | `crates/nf-game/src/drone.rs:248` |
| 0x15d860 | `NDrone2_DSTATE_CivilianMission` | ported | `crates/nf-game/src/drone.rs:252` |
| 0x15dbb0 | `NDrone2_DSTATE_CivilianMissionWait` | ported | `crates/nf-game/src/drone.rs:260` |
| 0x15df28 | `NDrone2_DSTATE_KikoMission` | ported | `crates/nf-game/src/drone.rs:286` |
| 0x15e0d8 | `NDrone2_DSTATE_KikoMissionRun` | ported | `crates/nf-game/src/drone.rs:10064` |
| 0x15e1e8 | `NDrone2_DSTATE_EnemyMission` | ported | `crates/nf-game/src/drone.rs:281` |
| 0x15e4f0 | `NDrone2_DSTATE_AllyLeadInit` | ported | `crates/nf-game/src/drone.rs:309` |
| 0x15e698 | `NDrone2_DSTATE_AllyLead` | ported | `crates/nf-game/src/drone.rs:312` |
| 0x15ea28 | `NDrone2_DSTATE_AllyLeadPlayerInWay` | ported | `crates/nf-game/src/drone.rs:318` |
| 0x15ec08 | `NDrone2_DSTATE_AllyLeadHide` | ported | `crates/nf-game/src/drone.rs:320` |
| 0x15eda8 | `NDrone2_DSTATE_AllyLeadWait` | ported | `crates/nf-game/src/drone.rs:314` |
| 0x15efc8 | `NDrone2_DSTATE_AllyLeadMissionWait` | ported | `crates/nf-game/src/drone.rs:322` |
| (see symbol) | `NDrone2_DSTATE_AllyLeadBondCombat` | ported | `crates/nf-game/src/drone.rs:324` |
| 0x15f2e8 | `NDrone2_DSTATE_AllyLeadDone` | ported | `crates/nf-game/src/drone.rs:316` |
| 0x15f3c8 | `NDrone2_DSTATE_AllyFollowInit` | ported | `crates/nf-game/src/drone.rs:326` |
| 0x15f4e0 | `NDrone2_DSTATE_AllyFollow` | ported | `crates/nf-game/src/drone.rs:326` |
| 0x15f6a0 | `NDrone2_DSTATE_AllyFollowWait` | ported | `crates/nf-game/src/drone.rs:326` |
| 0x15f858 | `NDrone2_DSTATE_AllyFollowDone` | ported | `crates/nf-game/src/drone.rs:327` |
| 0x15f938 | `NDrone2_DSTATE_AllyGoToGoalPosition` | ported | `crates/nf-game/src/drone.rs:327` |
| 0x15fab0 | `NDrone2_DSTATE_SniperIdle` | ported | `crates/nf-game/src/drone.rs:287` |
| 0x15fc18 | `NDrone2_DSTATE_SniperAim` | ported | `crates/nf-game/src/drone.rs:7764` |
| 0x15fd88 | `NDrone2_DSTATE_SniperFire` | ported | `crates/nf-game/src/drone.rs:7764` |
| 0x15ff80 | `NDrone2_DSTATE_SniperReload` | ported | `crates/nf-game/src/drone.rs:7764` |
| 0x160140 | `NDrone2_DSTATE_PartyGirl` | ported | `crates/nf-game/src/drone.rs:248` |
| 0x1603a8 | `NDrone2_DSTATE_CivilianGuard` | ported | `crates/nf-game/src/drone.rs:249` |
| 0x160648 | `NDrone2_DSTATE_CivilianDoorGuard` | ported | `crates/nf-game/src/drone.rs:249` |
| 0x160bd0 | `NDrone2_DSTATE_TruckDriverInit` | ported | `crates/nf-game/src/drone.rs:250` |
| 0x160d98 | `NDrone2_DSTATE_TruckDriverInitAlert` | ported | `crates/nf-game/src/drone.rs:250` |
| 0x160ed0 | `NDrone2_DSTATE_TruckDriverIdle` | ported | `crates/nf-game/src/drone.rs:250` |
| 0x161030 | `NDrone2_DSTATE_TruckDriverMission` | ported | `crates/nf-game/src/drone.rs:250` |
| 0x1612e0 | `NDrone2_DSTATE_CastleChatGuard1` | ported | `crates/nf-game/src/drone.rs:289` |
| 0x161560 | `NDrone2_DSTATE_AmbushWait` | ported | `crates/nf-game/src/drone.rs:6475` |
| 0x1616c0 | `NDrone2_DSTATE_InterogateAssist` | ported | `crates/nf-game/src/drone.rs:273` |
| 0x161860 | `NDrone2_DSTATE_InterogateAssistWait` | ported | `crates/nf-game/src/drone.rs:276` |
| 0x161a40 | `NDrone2_DSTATE_Interogate` | ported | `crates/nf-game/src/drone.rs:270` |
| 0x162338 | `NDrone2_DSTATE_InterogateWalk` | ported | `crates/nf-game/src/drone.rs:290` |
| 0x162558 | `NDrone2_DSTATE_CivilianChallenge` | ported | `crates/nf-game/src/drone.rs:266` |
| 0x1626a0 | `NDrone2_DSTATE_Surrender_Anim` | ported | `crates/nf-game/src/drone.rs:956` |
| 0x162950 | `NDrone2_DSTATE_Surrendered` | ported | `crates/nf-game/src/drone.rs:3406` |
| 0x162a70 | `NDrone2_DSTATE_Unsurrender_Anim` | ported | `crates/nf-game/src/drone.rs:3406` |
| 0x162b98 | `NDrone2_DSTATE_Taser` | ported | `crates/nf-game/src/drone.rs:212` |
| 0x162eb0 | `NDrone2_DSTATE_Stunned` | ported | `crates/nf-game/src/drone.rs:217` |
| 0x1630b8 | `NDrone2_DSTATE_Stunned_Recover` | ported | `crates/nf-game/src/drone.rs:221` |
| 0x1631c0 | `NDrone2_DSTATE_StunGrenadeLoop` | ported | `crates/nf-game/src/drone.rs:205` |
| 0x163350 | `NDrone2_DSTATE_StunGrenadeRecover` | ported | `crates/nf-game/src/drone.rs:209` |
| 0x163430 | `NDrone2_DSTATE_StunDartImpact` | ported | `crates/nf-game/src/drone.rs:224` |
| 0x163570 | `NDrone2_DSTATE_StunDartLoop` | ported | `crates/nf-game/src/drone.rs:228` |
| 0x163700 | `NDrone2_DSTATE_StunDartRecover` | ported | `crates/nf-game/src/drone.rs:232` |
| 0x1637f8 | `NDrone2_DSTATE_Death_Anim` | ported | `crates/nf-game/src/drone.rs:1274` |
| 0x163a80 | `NDrone2_DSTATE_DeathByExplosion` | ported | `crates/nf-game/src/drone.rs:151` |
| 0x163c68 | `NDrone2_DSTATE_SpecialDeath_Anim` | ported | `crates/nf-game/src/drone.rs:52` |
| 0x163fe8 | `NDrone2_DSTATE_Dead` | ported | `crates/nf-game/src/drone.rs:1868` |
| 0x164210 | `NDrone2_DSTATE_Attack` | ported | `crates/nf-game/src/drone.rs:519` |
| 0x1643e8 | `NDrone2_DSTATE_CombatNoMove` | ported | `crates/nf-game/src/drone.rs:7402` |
| 0x164690 | `NDrone2_DSTATE_CombatOutOfRange` | ported | `crates/nf-game/src/drone.rs:77` |
| 0x164838 | `NDrone2_DSTATE_CombatNoSight` | ported | `crates/nf-game/src/drone.rs:3605` |
| 0x164960 | `NDrone2_DSTATE_CombatWait` | ported | `crates/nf-game/src/drone.rs:44` |
| 0x164c08 | `NDrone2_DSTATE_CombatNoRoute` | ported | `crates/nf-game/src/drone.rs:8103` |
| 0x1650a0 | `NDrone2_DSTATE_NoOpponent` | ported | `crates/nf-game/src/drone.rs:49` |
| 0x165270 | `NDrone2_DSTATE_AlertToPosition` | ported | `crates/nf-game/src/drone.rs:290` |
| 0x165640 | `NDrone2_DSTATE_GoToGoalPosition` | ported | `crates/nf-game/src/drone.rs:80` |
| 0x165a68 | `NDrone2_DSTATE_HostageGoToGoalPosition` | ported | `crates/nf-game/src/drone.rs:84` |
| 0x165bf0 | `NDrone2_DSTATE_SearchArea` | ported | `crates/nf-game/src/drone.rs:290` |
| 0x165ef0 | `NDrone2_DSTATE_DroneStuck` | ported | `crates/nf-game/src/drone.rs:73` |
| 0x166000 | `NDrone2_DSTATE_DrawWeapon` | ported | `crates/nf-game/src/drone.rs:2151` |
| 0x166178 | `NDrone2_DSTATE_AimStand` | ported | `crates/nf-game/src/drone.rs:8279` |
| 0x166328 | `NDrone2_DSTATE_AimStandFire` | ported | `crates/nf-game/src/drone.rs:8354` |
| 0x1666d8 | `NDrone2_DSTATE_AimStandReload` | ported | `crates/nf-game/src/drone.rs:8387` |
| 0x166840 | `NDrone2_DSTATE_AimStandDiscard` | ported | `crates/nf-game/src/drone.rs:7834` |
| 0x166940 | `NDrone2_DSTATE_Prone` | ported | `crates/nf-game/src/drone.rs:410` |
| 0x166a28 | `NDrone2_DSTATE_ProneFire` | ported | `crates/nf-game/src/drone.rs:413` |
| 0x166b98 | `NDrone2_DSTATE_AimBackoff` | ported | `crates/nf-game/src/drone.rs:8515` |
| 0x166d90 | `NDrone2_DSTATE_CrouchCover` | ported | `crates/nf-game/src/drone.rs:367` |
| 0x166f28 | `NDrone2_DSTATE_AimCrouch` | ported | `crates/nf-game/src/drone.rs:368` |
| 0x167060 | `NDrone2_DSTATE_AimCrouchFire` | ported | `crates/nf-game/src/drone.rs:8433` |
| 0x167370 | `NDrone2_DSTATE_AimCrouchReload` | ported | `crates/nf-game/src/drone.rs:8388` |
| 0x167488 | `NDrone2_DSTATE_AltAttack` | ported | `crates/nf-game/src/drone.rs:71` |
| 0x167590 | `NDrone2_DSTATE_StepAimLeft` | ported | `crates/nf-game/src/drone.rs:8451` |
| 0x1676a0 | `NDrone2_DSTATE_StepAimRight` | ported | `crates/nf-game/src/drone.rs:8451` |
| 0x1677b0 | `NDrone2_DSTATE_StrafeAimLeft` | ported | `crates/nf-game/src/drone.rs:8466` |
| 0x167980 | `NDrone2_DSTATE_StrafeAimRight` | ported | `crates/nf-game/src/drone.rs:8466` |
| 0x167b50 | `NDrone2_DSTATE_StrafeDodgeLeft` | ported | `crates/nf-game/src/drone.rs:8467` |
| 0x167cf0 | `NDrone2_DSTATE_StrafeDodgeRight` | ported | `crates/nf-game/src/drone.rs:8468` |
| 0x167e90 | `NDrone2_DSTATE_RollLeftCrouch` | ported | `crates/nf-game/src/drone.rs:8506` |
| 0x167f80 | `NDrone2_DSTATE_RollRightCrouch` | ported | `crates/nf-game/src/drone.rs:8506` |
| 0x168070 | `NDrone2_DSTATE_Investigate` | ported | `crates/nf-game/src/drone.rs:76` |
| 0x1681f8 | `NDrone2_DSTATE_OpenDoor` | ported | `crates/nf-game/src/drone.rs:338` |
| 0x1684c0 | `NDrone2_DSTATE_KickObject` | ported | `crates/nf-game/src/drone.rs:342` |
| 0x168628 | `NDrone2_DSTATE_ActionAnim` | ported | `crates/nf-game/src/drone.rs:348` |
| 0x1687c0 | `NDrone2_DSTATE_StandFiddle` | ported | `crates/nf-game/src/drone.rs:352` |
| 0x168910 | `NDrone2_DSTATE_EnemyRunToPoint` | ported | `crates/nf-game/src/drone.rs:278` |
| 0x168aa8 | `NDrone2_DSTATE_RunAwayFromObject` | ported | `crates/nf-game/src/drone.rs:1053` |
| 0x168d18 | `NDrone2_DSTATE_HideFromScaryObject` | ported | `crates/nf-game/src/drone.rs:240` |
| 0x168ea8 | `NDrone2_DSTATE_PressAlarm` | ported | `crates/nf-game/src/drone.rs:686` |
| 0x169150 | `NDrone2_DSTATE_RunForCover` | ported | `crates/nf-game/src/drone.rs:6304` |
| 0x169488 | `NDrone2_DSTATE_ElevatorJumper` | ported | `crates/nf-game/src/drone.rs:166` |
| 0x169610 | `NDrone2_DSTATE_AbseilInit` | ported | `crates/nf-game/src/drone.rs:166` |
| 0x169730 | `NDrone2_DSTATE_AbseilSlide` | ported | `crates/nf-game/src/drone.rs:167` |
| 0x169940 | `NDrone2_DSTATE_AbseilHang` | ported | `crates/nf-game/src/drone.rs:167` |
| 0x169b28 | `NDrone2_DSTATE_AbseilDeath` | ported | `crates/nf-game/src/drone.rs:168` |
| 0x169cc0 | `NDrone2_DSTATE_UnderCoverInit` | ported | `crates/nf-game/src/drone.rs:6324` |
| 0x169e58 | `NDrone2_DSTATE_UnderCoverIdle` | ported | `crates/nf-game/src/drone.rs:1277` |
| 0x16a2d0 | `NDrone2_DSTATE_UnderCoverAim` | ported | `crates/nf-game/src/drone.rs:6389` |
| 0x16a3e0 | `NDrone2_DSTATE_UnderCoverFire` | ported | `crates/nf-game/src/drone.rs:6402` |
| 0x16a610 | `NDrone2_DSTATE_UnderCoverSniperFire` | ported | `crates/nf-game/src/drone.rs:6424` |
| 0x16a8b0 | `NDrone2_DSTATE_UnderCoverSniperReload` | ported | `crates/nf-game/src/drone.rs:6456` |
| 0x16a9c0 | `NDrone2_DSTATE_UnderCoverReturn` | ported | `crates/nf-game/src/drone.rs:6485` |
| 0x16ab38 | `NDrone2_DSTATE_UnderCoverLeaveNow` | ported | `crates/nf-game/src/drone.rs:186` |
| 0x16acd0 | `NDrone2_DSTATE_SeenDeadBody` | ported | `crates/nf-game/src/drone.rs:2475` |
| 0x16ade0 | `NDrone2_DSTATE_SeenSurrenderedDrone` | ported | `crates/nf-game/src/drone.rs:2476` |
| 0x16af20 | `NDrone2_DSTATE_HeardNoiseAware` | ported | `crates/nf-game/src/drone.rs:291` |
| 0x16b0e0 | `NDrone2_DSTATE_HeardNoiseSuspect` | ported | `crates/nf-game/src/drone.rs:291` |
| 0x16b540 | `NDrone2_DSTATE_HeardNoiseAlert` | ported | `crates/nf-game/src/drone.rs:292` |
| 0x16b878 | `NDrone2_DSTATE_SmokedOut_Loop` | ported | `crates/nf-game/src/drone.rs:142` |
| 0x16b990 | `NDrone2_DSTATE_NinjaStand` | ported | `crates/nf-game/src/drone.rs:3578` |
| 0x16bb50 | `NDrone2_DSTATE_NinjaAttack` | ported | `crates/nf-game/src/drone.rs:3578` |
| 0x16bfc0 | `NDrone2_DSTATE_NinjaAttackLongRange` | ported | `crates/nf-game/src/drone.rs:3578` |
| 0x16c248 | `NDrone2_DSTATE_NinjaAttackMidRange` | ported | `crates/nf-game/src/drone.rs:3579` |
| 0x16c650 | `NDrone2_DSTATE_NinjaAttackShortRange` | ported | `crates/nf-game/src/drone.rs:3579` |
| 0x16cb70 | `NDrone2_DSTATE_NinjaGetCloseToPlayer` | ported | `crates/nf-game/src/drone.rs:3584` |
| 0x16cdd0 | `NDrone2_DSTATE_NinjaSideflipLeft` | ported | `crates/nf-game/src/drone.rs:622` |
| 0x16cf68 | `NDrone2_DSTATE_NinjaSideflipRight` | ported | `crates/nf-game/src/drone.rs:3580` |
| 0x16d100 | `NDrone2_DSTATE_NinjaSword` | ported | `crates/nf-game/src/drone.rs:3579` |
| 0x16d298 | `NDrone2_DSTATE_NinjaSomersault` | ported | `crates/nf-game/src/drone.rs:3579` |
| 0x16d430 | `NDrone2_DSTATE_NinjaBackflip` | ported | `crates/nf-game/src/drone.rs:3580` |
| 0x16d5b8 | `NDrone2_DSTATE_NinjaStandFire` | ported | `crates/nf-game/src/drone.rs:3581` |
| 0x16d878 | `NDrone2_DSTATE_NinjaNoRoute` | ported | `crates/nf-game/src/drone.rs:622` |
| 0x16db90 | `NDrone2_DSTATE_Tester1` | ported | `crates/nf-game/src/drone.rs:427` |
| 0x16dd28 | `NDrone2_DSTATE_AstronautLaunch` | ported | `crates/nf-game/src/drone.rs:292` |
| 0x16dee0 | `NDrone2_DSTATE_SpaceDrake` | ported | `crates/nf-game/src/drone.rs:293` |
| 0x16e060 | `NDrone2_DSTATE_BotGlobal` | ported | `crates/nf-game/src/drone.rs:6735` |
| 0x16f198 | `NDrone2_DSTATE_BotAttack` | ported | `crates/nf-game/src/mp/bots.rs:781` |
| 0x16f390 | `NDrone2_DSTATE_BotAttackRun` | ported | `crates/nf-game/src/mp/bots.rs:3503` |
| 0x16f5e8 | `NDrone2_DSTATE_BotAttackFire` | ported | `crates/nf-game/src/mp/bots.rs:781` |
| 0x16f850 | `NDrone2_DSTATE_BotAttackStrafeAimLeft` | ported | `crates/nf-game/src/mp/bots.rs:1240` |
| 0x16f9b8 | `NDrone2_DSTATE_BotAttackStrafeAimRight` | ported | `crates/nf-game/src/mp/bots.rs:3337` |
| 0x16fb20 | `NDrone2_DSTATE_BotAttackRunChangePosition` | ported | `crates/nf-game/src/mp/bots.rs:332` |
| 0x16fc30 | `NDrone2_DSTATE_BotAttackBackoff` | ported | `crates/nf-game/src/mp/bots.rs:3545` |
| 0x16fe80 | `NDrone2_DSTATE_BotAttackCrouch` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x170190 | `NDrone2_DSTATE_BotAttackReload` | ported | `crates/nf-game/src/mp/bots.rs:339` |
| 0x1702e0 | `NDrone2_DSTATE_BotCoverRunTo` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x170448 | `NDrone2_DSTATE_BotCoverInit` | ported | `crates/nf-game/src/mp/bots.rs:343` |
| 0x1705b0 | `NDrone2_DSTATE_BotCoverIdle` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x1708e8 | `NDrone2_DSTATE_BotCoverAim` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x1709d0 | `NDrone2_DSTATE_BotCoverFire` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x170b90 | `NDrone2_DSTATE_BotCoverReturn` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x170d00 | `NDrone2_DSTATE_BotCoverTypeChange` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x170e20 | `NDrone2_DSTATE_BotCoverLeave` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x170f38 | `NDrone2_DSTATE_BotCoverLeaveNow` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x171048 | `NDrone2_DSTATE_BotAlertToPosition` | ported | `crates/nf-game/src/mp/bots.rs:352` |
| 0x171240 | `NDrone2_DSTATE_BotGotoGoalPosition` | ported | `crates/nf-game/src/mp/bots.rs:353` |
| 0x1713f0 | `NDrone2_DSTATE_BotImpactBullet` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x171518 | `NDrone2_DSTATE_BotDead` | ported | `crates/nf-game/src/mp/mod.rs:1229` |
| 0x1716d0 | `NDrone2_DSTATE_BotDoorOpen` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x1718e0 | `NDrone2_DSTATE_BotGuardFriendFollow` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x171b80 | `NDrone2_DSTATE_BotIdle` | ported | `crates/nf-game/src/mp/bots.rs:1081` |
| 0x171df8 | `Drone_SM_RouteMsgDCV` | ported | `crates/nf-game/src/drone.rs:1056` |
| 0x172128 | `Drone_SM_RouteMsg` | ported | `crates/nf-game/src/drone.rs:1850` |
| 0x172440 | `NDrone2_DSTATE_Disabled` | ported | `crates/nf-game/src/drone.rs:393` |
| 0x1724f0 | `NDrone2_DSTATE_InitPatrol` | ported | `crates/nf-game/src/drone.rs:7648` |
| 0x1725a0 | `NDrone2_DSTATE_HostageDie` | ported | `crates/nf-game/src/drone.rs:66` |
| 0x172628 | `NDrone2_DSTATE_HostageDead` | ported | `crates/nf-game/src/drone.rs:51` |
| 0x172730 | `NDrone2_DSTATE_CivilianInit` | ported | `crates/nf-game/src/drone.rs:248` |
| 0x172810 | `NDrone2_DSTATE_ReturnToPatrolPath` | ported | `crates/nf-game/src/drone.rs:235` |
| 0x172870 | `NDrone2_DSTATE_StandBlind` | ported | `crates/nf-game/src/drone.rs:10070` |
| 0x1728c0 | `NDrone2_DSTATE_PartyGirlInit` | ported | `crates/nf-game/src/drone.rs:248` |
| 0x172960 | `NDrone2_DSTATE_AmbushInit` | ported | `crates/nf-game/src/drone.rs:6466` (name cite) |
| 0x1729f0 | `NDrone2_DSTATE_Interogator` | ported | `crates/nf-game/src/drone.rs:401` |
| 0x172a30 | `NDrone2_DSTATE_KnockedOut_Anim` | ported | `crates/nf-game/src/drone.rs:112` |
| 0x172b48 | `NDrone2_DSTATE_Knocked_Out` | ported | `crates/nf-game/src/drone.rs:117` |
| 0x172bb0 | `NDrone2_DSTATE_StunGrenadeImpact` | ported | `crates/nf-game/src/drone.rs:202` |
| 0x172cd8 | `NDrone2_DSTATE_Fade` | ported | `crates/nf-game/src/drone.rs:1868` |
| 0x172dd8 | `NDrone2_DSTATE_FadeFast` | ported | `crates/nf-game/src/drone.rs:1868` |
| 0x172ee0 | `NDrone2_DSTATE_PunchImpact` | ported | `crates/nf-game/src/drone.rs:120` |
| 0x172fb0 | `NDrone2_DSTATE_ExplosiveImpact` | ported | `crates/nf-game/src/drone.rs:124` |
| 0x173098 | `NDrone2_DSTATE_BulletImpact` | ported | `crates/nf-game/src/drone.rs:3530` |
| 0x173178 | `NDrone2_DSTATE_Alerted1stEncounter` | ported | `crates/nf-game/src/drone.rs:403` |
| 0x1731b8 | `NDrone2_DSTATE_Combat` | ported | `crates/nf-game/src/drone.rs:8087` |
| 0x173280 | `NDrone2_DSTATE_CombatNewSighting` | ported | `crates/nf-game/src/drone.rs:39` |
| 0x173338 | `NDrone2_DSTATE_CombatTooClose` | ported | `crates/nf-game/src/drone.rs:406` |
| 0x173390 | `NDrone2_DSTATE_Obstructed` | ported | `crates/nf-game/src/drone.rs:7695` |
| 0x173480 | `NDrone2_DSTATE_GrenadeThrow` | ported | `crates/nf-game/src/drone.rs:6523` |
| 0x173570 | `NDrone2_DSTATE_HoldItRightThere` | ported | `crates/nf-game/src/drone.rs:416` |
| 0x173658 | `NDrone2_DSTATE_RecoverFromScaryObject` | ported | `crates/nf-game/src/drone.rs:241` |
| 0x1736f8 | `NDrone2_DSTATE_RunToAlarm` | ported | `crates/nf-game/src/drone.rs:1017` |
| 0x1737d0 | `NDrone2_DSTATE_DonePressAlarm` | ported | `crates/nf-game/src/drone.rs:1027` |
| 0x1738e0 | `NDrone2_DSTATE_AbseilStepOff` | ported | `crates/nf-game/src/drone.rs:167` |
| 0x1739b8 | `NDrone2_DSTATE_UnderCoverTypeChange` | ported | `crates/nf-game/src/drone.rs:180` |
| 0x173ac0 | `NDrone2_DSTATE_UnderCoverLeave` | ported | `crates/nf-game/src/drone.rs:184` |
| 0x173b38 | `NDrone2_DSTATE_SeenOpponent` | ported | `crates/nf-game/src/drone.rs:418` |
| 0x173b78 | `NDrone2_DSTATE_SeenDroneShot` | ported | `crates/nf-game/src/drone.rs:18668` |
| 0x173bb8 | `NDrone2_DSTATE_SeenExplosive` | ported | `crates/nf-game/src/drone.rs:420` |
| 0x173be8 | `NDrone2_DSTATE_HeardNoise` | ported | `crates/nf-game/src/drone.rs:7861` (name cite) |
| 0x173c50 | `NDrone2_DSTATE_SmokedOut` | ported | `crates/nf-game/src/drone.rs:139` |
| 0x173d30 | `NDrone2_DSTATE_SmokedOut_Recover` | ported | `crates/nf-game/src/drone.rs:147` |
| 0x173e08 | `NDrone2_DSTATE_NinjaSideflip` | ported | `crates/nf-game/src/drone.rs:3584` |
| 0x173f00 | `NDrone2_DSTATE_DeleteMe` | ported | `crates/nf-game/src/drone.rs:1888` |
| 0x173f68 | `NDrone2_DSTATE_FailMission` | ported | `crates/nf-game/src/drone.rs:422` |
| 0x173fc0 | `NDrone2_DSTATE_JustStand` | ported | `crates/nf-game/src/drone.rs:10149` |
| 0x174150 | `NDrone2_DSTATE_Tester2` | ported | `crates/nf-game/src/drone.rs:427` |
| 0x1741c8 | `NDrone2_DSTATE_Tester3` | ported | `crates/nf-game/src/drone.rs:431` |
| 0x1742d8 | `NDrone2_DSTATE_Tester4` | ported | `crates/nf-game/src/drone.rs:428` |
| 0x1743d8 | `NDrone2_DSTATE_HangUp` | ported | `crates/nf-game/src/drone.rs:435` |
| 0x174430 | `NDrone2_DSTATE_WaitForever` | ported | `crates/nf-game/src/drone.rs:437` |
| 0x174458 | `NDrone2_DSTATE_AstronautCombat` | ported | `crates/nf-game/src/drone.rs:106` |
| 0x174550 | `NDrone2_DSTATE_AstronautCombatMove` | ported | `crates/nf-game/src/drone.rs:109` |
| 0x174690 | `NDrone2_DSTATE_AstronautHit` | ported | `crates/nf-game/src/drone.rs:100` |
| 0x1747b8 | `NDrone2_DSTATE_AstronautDeath` | ported | `crates/nf-game/src/drone.rs:103` |
| 0x1748c8 | `NDrone2_DSTATE_BotInit` | ported | `crates/nf-game/src/mp/bots.rs:1352` |
| 0x174938 | `NDrone2_DSTATE_BotRespawn` | ported | `crates/nf-game/src/mp/bots.rs:302` |
| 0x174980 | `NDrone2_DSTATE_BotCollector` | ported | `crates/nf-game/src/mp/bots.rs:309` |
| 0x174a28 | `NDrone2_DSTATE_BotGuardian` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174a70 | `NDrone2_DSTATE_BotTeamPlayer` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174ab8 | `NDrone2_DSTATE_BotBully` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174b00 | `NDrone2_DSTATE_BotBerserker` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174b48 | `NDrone2_DSTATE_BotGreedy` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174b90 | `NDrone2_DSTATE_BotVengeful` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174bd8 | `NDrone2_DSTATE_BotJudge` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174c20 | `NDrone2_DSTATE_BotAssassin` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174c68 | `NDrone2_DSTATE_BotAttackNoRoute` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174cd0 | `NDrone2_DSTATE_BotAttackNoOpponent` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174d18 | `NDrone2_DSTATE_BotAttackRollLeftCrouch` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174dd0 | `NDrone2_DSTATE_BotAttackRollRightCrouch` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x174e88 | `NDrone2_DSTATE_BotAttackStepAimLeft` | ported | `crates/nf-game/src/mp/bots.rs:1603` |
| 0x174fa8 | `NDrone2_DSTATE_BotAttackStepAimRight` | ported | `crates/nf-game/src/mp/bots.rs:3366` |
| 0x1750c8 | `NDrone2_DSTATE_BotAttackChangeWeapon` | ported | `crates/nf-game/src/mp/mod.rs:1204` |
| 0x175188 | `NDrone2_DSTATE_BotAttackUnarmed` | ported | `crates/nf-game/src/mp/bots.rs:341` |
| 0x175220 | `NDrone2_DSTATE_BotStuck` | ported | `crates/nf-game/src/mp/bots.rs:351` |
| 0x175268 | `NDrone2_DSTATE_BotSeenOpponent` | ported | `crates/nf-game/src/mp/bots.rs:354` |
| 0x1752b0 | `NDrone2_DSTATE_BotSeenDroneShot` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x1752f8 | `NDrone2_DSTATE_BotHeardNoise` | ported | `crates/nf-game/src/mp/bots.rs` BOT_STATES (game body kind: ?); Full kinds have state impls |
| 0x175308 | `NDrone2_DSTATE_BotImpactExplosive` | ported | `crates/nf-game/src/mp/bots.rs:358` |
| 0x175388 | `NDrone2_DSTATE_BotImpactPunch` | ported | `crates/nf-game/src/mp/bots.rs:359` |
| 0x175418 | `NDrone2_DSTATE_BotImpactStunGrenade` | ported | `crates/nf-game/src/mp/bots.rs:363` |
| 0x175508 | `NDrone2_DSTATE_BotDeathAnim` | ported | `crates/nf-game/src/mp/bots.rs:360` |
| 0x1755a8 | `NDrone2_DSTATE_BotDeathByExplosion` | ported | `crates/nf-game/src/mp/bots.rs:4771` |
| 0x175680 | `NDrone2_DSTATE_BotGuardFriendIdle` | ported | `crates/nf-game/src/mp/bots.rs:365` |
| 0x175778 | `NDrone2_ProcessStateMachine` | ported | `crates/nf-game/src/mp/bots.rs:613` |
| 0x1757d0 | `Drone_SM_Init` | ported | `crates/nf-game/src/drone.rs:2557` |
| 0x175838 | `Drone_SM_InitObject` | ported | `crates/nf-game/src/drone.rs:1339` |
| 0x1758c0 | `Drone_SM_SetState` | ported | `crates/nf-game/src/drone.rs:390` |
| 0x1759a0 | `Drone_SM_SendDelayedMsgs` | ported | `crates/nf-game/src/drone.rs:2557` |
| 0x175a38 | `Drone_SM_SendMsgSelf` | ported | `crates/nf-game/src/drone.rs:1849` |
| 0x175aa8 | `Drone_SM_BroadcastMsg` | ported | `crates/nf-game/src/drone.rs:3893` |
| 0x175af0 | `DroneVision_OpponentVisibility` | ported | `crates/nf-game/src/drone.rs:11491` |
| 0x175cb8 | `DroneVision_SeekOpponentLOS` | ported | `crates/nf-game/src/drone.rs:1857` |
| 0x175ef0 | `DroneVision_HaveOpponentSight` | ported | `crates/nf-game/src/drone.rs:946` |
| 0x1760b0 | `DroneVision_DroneCanSeeDrone` | ported | `crates/nf-game/src/drone.rs:18568` |
| 0x176250 | `DroneVision_ConsiderAlerted` | ported | `crates/nf-game/src/drone.rs:279` |
| 0x1766f8 | `DroneVision_DetermineAlertCheck` | ported | `crates/nf-game/src/drone.rs:18544` |
| 0x1767d0 | `DroneVision_FindAlertedDrones` | ported | `crates/nf-game/src/drone.rs:18559` |
| 0x176a28 | `DroneVision_ProcessDroneSight` | ported | `crates/nf-game/src/drone.rs:1541` |
| 0x176f18 | `DroneVision_CanSeeObjectFrom` | ported | `crates/nf-game/src/drone.rs:1261` |
| 0x1770f0 | `NDrone2_ReactToOpponentSighted` | ported | `crates/nf-game/src/mp/bots.rs:2631` |
| 0x177420 | `NDrone2_NearArmedDrone` | ported | `crates/nf-game/src/drone.rs:1430` |
| 0x177640 | `NDrone2_NotNearArmedDrone` | ported | `crates/nf-game/src/drone.rs:1422` |
| 0x177800 | `DroneVision_AlertSound` | ported | `crates/nf-game/src/drone.rs:1052` |
| 0x177a30 | `DroneVision_ShoutFromOtherDrone` | ported | `crates/nf-game/src/drone.rs:3896` |
| 0x177c18 | `DroneVision_EnemyAlerts` | ported | `crates/nf-game/src/drone.rs:468` |
| 0x177ed0 | `DroneVision_EnemyLookForOpponent` | ported | `crates/nf-game/src/drone.rs:5757` |
| 0x1780a8 | `DroneVision_LogSeenOpponent` | ported | `crates/nf-game/src/drone.rs:1804` |
| 0x1781a8 | `DroneVision_ObjectVisibility` | ported | `crates/nf-game/src/drone.rs:18548` |
| 0x1782a8 | `DroneVision_BeginInterogate` | ported | `crates/nf-game/src/drone.rs:1317` |
| 0x1782c8 | `DroneVision_EndInterogate` | ported | `crates/nf-game/src/drone.rs:1317` |
| 0x178308 | `DroneVision_SetInterogateAssist` | ported | `crates/nf-game/src/drone.rs:8628` |
| 0x178318 | `DroneVision_EndInterogateAssist` | ported | `crates/nf-game/src/drone.rs:8671` |
| 0x178348 | `DroneVision_TestEndInterogationAssist` | ported | `crates/nf-game/src/drone.rs:1322` |
| 0x178390 | `DroneVision_LineOfSightToObject` | ported | `crates/nf-game/src/drone.rs:11992` |
| 0x178438 | `DroneVision_CanSeePosition` | ported | `crates/nf-game/src/drone.rs:11475` |
| 0x178520 | `DroneVision_GunThroughWall` | ported | `crates/nf-game/src/drone.rs:885` |
| 0x1785e0 | `NDrone2_CanSeeObject` | ported | `crates/nf-game/src/drone.rs:1261` |
| 0x178678 | `NDrone2_NearArmedBond` | ported | `crates/nf-game/src/drone.rs:3955` |
| 0x178768 | `NDrone2_ObjectIsArmed` | ported | `crates/nf-game/src/drone.rs:3842` |
| 0x1787c8 | `NDrone2_NearArmedPerson` | ported | `crates/nf-game/src/drone.rs:3937` |
| 0x1788a0 | `NDrone2_InShoutingRange` | ported | `crates/nf-game/src/drone.rs:6734` |
| 0x1789c0 | `DroneWeap_CloseCombatImpact` | ported | `crates/nf-game/src/drone.rs:1465` |
| 0x178ae0 | `DroneWeap_AimTarget_IsOpponent` | ported | `crates/nf-game/src/drone.rs:8485` |
| 0x178c48 | `DroneWeap_Ready2Fire` | ported | `crates/nf-game/src/drone.rs:844` |
| 0x178f68 | `DroneWeap_FireWeapon` | ported | `crates/nf-game/src/drone.rs:5333` |
| 0x179558 | `DroneWeap_ThrowGrenade` | ported | `crates/nf-game/src/drone.rs:11099` |
| 0x179740 | `DroneWeap_DoBulletAccuracy` | ported | `crates/nf-game/src/drone.rs:663` |
| 0x179a48 | `DroneWeap_DoOpponentTargetting` | ported | `crates/nf-game/src/drone.rs:737` |
| 0x179ba8 | `DroneWeap_BurstDelay` | ported | `crates/nf-game/src/drone.rs:11166` (name cite) |
| 0x179d40 | `DroneWeap_NextBulletTime` | ported | `crates/nf-game/src/drone.rs:1083` |
| 0x17a080 | `DroneWeap_DoFiring` | ported | `crates/nf-game/src/drone.rs:18433` |
| 0x17a180 | `DroneWeap_HandleFiring` | ported | `crates/nf-game/src/drone.rs:964` |
| 0x17a2a0 | `DroneWeap_DropWeapon` | ported | `crates/nf-game/src/drone.rs:5265` |
| 0x17a4b8 | `DroneWeap_OpponentHasWeapon` | ported | `crates/nf-game/src/drone.rs:3942` |
| 0x17a500 | `DroneWeap_OpponentIsAimingAtMe` | ported | `crates/nf-game/src/drone.rs:10614` |
| 0x17a5a0 | `DroneWeap_GetWeaponInfo` | ported | `crates/nf-game/src/drone.rs:859` |
| 0x17a6e0 | `DroneWeap_ChangeWeapon` | ported | `crates/nf-game/src/drone.rs:1461` |
| 0x17a798 | `DroneWeap_CanThrowGrenade` | ported | `crates/nf-game/src/drone.rs:10872` |
| 0x17a848 | `DroneWeap_PlaceTertiary` | ported | `crates/nf-game/src/drone.rs:3143` |
| 0x17a870 | `DroneWeap_ManageBurstCount` | ported | `crates/nf-game/src/mp/bots.rs:3679` |
| 0x17a8c0 | `DroneWeap_Fire` | ported | `crates/nf-game/src/mp/bots.rs:3469` |
