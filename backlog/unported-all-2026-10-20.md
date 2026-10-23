# ACTION.ELF functions never mentioned in the port's game source (master e19bf230)

Measured 2026-10-20: a function counts as mentioned if its name or address appears anywhere in crates/*/src. Being mentioned does NOT mean fully ported. Platform plumbing (PS2 DMA/GS/IOP/memcard/libc/gcc/vector-math) is excluded.

**850 gameplay/engine functions never mentioned.**

## NDrone2 — 137 functions, 40016 bytes

- `0x1291e8` NDrone2_initDTYPE_BotInit (8 bytes)
- `0x13e770` NDrone2_PowerstationTalk (532 bytes)
- `0x13ed38` NDrone2_HitTalkExtra (384 bytes)
- `0x13f688` NDrone2_HandleTalking (628 bytes)
- `0x13f900` NDrone2_CivilianScaredTalk (288 bytes)
- `0x141150` NDrone2_GetHeadPosOld (392 bytes)
- `0x141fe8` NDrone2_SetOpponentAimPos (744 bytes)
- `0x144340` NDrone2_MonitorTalkToMissionObj (324 bytes)
- `0x1446b0` NDrone2_BondTalk (1740 bytes)
- `0x147578` NDrone2_DoHeadTrackingBehaviour (924 bytes)
- `0x149600` NDrone2_CreateNinjaEyes (360 bytes)
- `0x149768` NDrone2_Player (36 bytes)
- `0x149ca0` NDrone2_SetupGroups (160 bytes)
- `0x149d58` NDrone2_SpeechSFXd (116 bytes)
- `0x14a410` NDrone2_SendAttackMessage (136 bytes)
- `0x14a648` NDrone2_ChangeToAttackMode (160 bytes)
- `0x14a800` NDrone2_DroneInSights (8 bytes)
- `0x14a808` NDrone2_SetupHostageExecute (176 bytes)
- `0x14a8b8` NDrone2_TalkToMissionObj (124 bytes)
- `0x14aaf8` NDrone2_HostageSituationActive (8 bytes)
- `0x14ad50` NDrone2_ObstructedPatrolPath (8 bytes)
- `0x14ad58` NDrone2_HasOpponent (68 bytes)
- `0x14ada0` NDrone2_HeadTrackObj (308 bytes)
- `0x14aff8` NDrone2_PostDroneControl (96 bytes)
- `0x14b128` NDrone2_ControlDTYPE_Astronaut (28 bytes)
- `0x14d2e8` NDrone2_DoTypeSettingsOLD (572 bytes)
- `0x14d528` NDrone2_init_DMODE_Defaults (672 bytes)
- `0x14d958` NDrone2_init_DMODE_Normal (28 bytes)
- `0x14d978` NDrone2_init_DMODE_Guard (28 bytes)
- `0x14d998` NDrone2_init_DMODE_Retreater (28 bytes)
- `0x14d9b8` NDrone2_init_DMODE_Sniper (192 bytes)
- `0x14da78` NDrone2_init_DMODE_Stealth (52 bytes)
- `0x14dab0` NDrone2_init_DMODE_Attacker (28 bytes)
- `0x14dad0` NDrone2_init_DMODE_RunToPoint (68 bytes)
- `0x14db18` NDrone2_init_DMODE_Assassin (28 bytes)
- `0x14db38` NDrone2_init_DMODE_HostageKiller (224 bytes)
- `0x14dc18` NDrone2_init_DMODE_Hostage (112 bytes)
- `0x14dc88` NDrone2_init_DMODE_HostageTied (112 bytes)
- `0x14dcf8` NDrone2_init_DMODE_JustStand4Demo (36 bytes)
- `0x14dd20` NDrone2_init_DMODE_DeleteMe (8 bytes)
- `0x14dd28` NDrone2_init_DMODE_Civilian (288 bytes)
- `0x14de48` NDrone2_init_DMODE_CivilianScared (224 bytes)
- `0x14df28` NDrone2_init_DMODE_MissionFailer (64 bytes)
- `0x14df68` NDrone2_init_DMODE_Mayhew (36 bytes)
- `0x14df90` NDrone2_init_DMODE_Ninja (8 bytes)
- `0x14df98` NDrone2_init_DMODE_AlarmRaiser (68 bytes)
- `0x14dfe0` NDrone2_init_DMODE_SearchLight (52 bytes)
- `0x14e050` NDrone2_init_DMODE_Zoe (68 bytes)
- `0x14e098` NDrone2_init_DMODE_PartyGirl (192 bytes)
- `0x14e158` NDrone2_init_DMODE_CivilianGuard (224 bytes)
- `0x14e238` NDrone2_init_DMODE_Interogator (28 bytes)
- `0x14e258` NDrone2_init_DMODE_CivDoorGuard (240 bytes)
- `0x14e348` NDrone2_init_DMODE_TruckDriver (36 bytes)
- `0x14e370` NDrone2_init_DMODE_CastleChatGuard1 (44 bytes)
- `0x14e3a0` NDrone2_init_DMODE_CastleChatGuard2 (60 bytes)
- `0x14e3e0` NDrone2_init_DMODE_SniperAlert (176 bytes)
- `0x14e490` NDrone2_init_DMODE_PartyGirlLooker (216 bytes)
- `0x14e568` NDrone2_init_DMODE_Bot (8 bytes)
- `0x14e580` NDrone2_initDTYPE_SniperAlert (56 bytes)
- `0x14f640` NDrone2_CanStepLeft (456 bytes)
- `0x14fdf8` NDrone2_CanStrafeDodgeLeft (456 bytes)
- `0x14ffc0` NDrone2_CanStrafeDodgeRight (476 bytes)
- `0x1501a0` NDrone2_CanBackoff (404 bytes)
- `0x150338` NDrone2_CanRollLeft (456 bytes)
- `0x1512e0` NDrone2_AnimForDist (504 bytes)
- `0x1514d8` NDrone2_UpdatePatrolRoute (320 bytes)
- `0x1519e0` NDrone2_MoveTo_RouteNode (312 bytes)
- `0x154838` NDrone2_CanMoveToRelativePos (296 bytes)
- `0x1549e0` NDrone2_CanAltAttack (152 bytes)
- `0x154b20` NDrone2_SetDroneAlertRoute (132 bytes)
- `0x154e20` NDrone2_SetAngleToDest (108 bytes)
- `0x154e90` NDrone2_SetAngleToAlert (108 bytes)
- `0x154f00` NDrone2_GetAngleDifferenceToAlert (84 bytes)
- `0x1554a8` NDrone2_MoveTestFromNode (376 bytes)
- `0x155620` NDrone2_MoveTestToNode (376 bytes)
- `0x155d78` NDrone2_DistanceToAIPoint (60 bytes)
- `0x15c728` NDrone2_DSTATE_HostageKiller (536 bytes)
- `0x15c940` NDrone2_DSTATE_HostageKillerAttack (568 bytes)
- `0x15dbb0` NDrone2_DSTATE_CivilianMissionWait (884 bytes)
- `0x15e1e8` NDrone2_DSTATE_EnemyMission (772 bytes)
- `0x1612e0` NDrone2_DSTATE_CastleChatGuard1 (636 bytes)
- `0x1616c0` NDrone2_DSTATE_InterogateAssist (416 bytes)
- `0x161860` NDrone2_DSTATE_InterogateAssistWait (476 bytes)
- `0x161a40` NDrone2_DSTATE_Interogate (2296 bytes)
- `0x162338` NDrone2_DSTATE_InterogateWalk (540 bytes)
- `0x1643e8` NDrone2_DSTATE_CombatNoMove (680 bytes)
- `0x164690` NDrone2_DSTATE_CombatOutOfRange (420 bytes)
- `0x164838` NDrone2_DSTATE_CombatNoSight (296 bytes)
- `0x164960` NDrone2_DSTATE_CombatWait (676 bytes)
- `0x1650a0` NDrone2_DSTATE_NoOpponent (460 bytes)
- `0x165270` NDrone2_DSTATE_AlertToPosition (976 bytes)
- `0x165bf0` NDrone2_DSTATE_SearchArea (768 bytes)
- `0x165ef0` NDrone2_DSTATE_DroneStuck (268 bytes)
- `0x166178` NDrone2_DSTATE_AimStand (428 bytes)
- `0x1666d8` NDrone2_DSTATE_AimStandReload (360 bytes)
- `0x166840` NDrone2_DSTATE_AimStandDiscard (256 bytes)
- `0x166940` NDrone2_DSTATE_Prone (228 bytes)
- `0x166a28` NDrone2_DSTATE_ProneFire (364 bytes)
- `0x166d90` NDrone2_DSTATE_CrouchCover (404 bytes)
- `0x166f28` NDrone2_DSTATE_AimCrouch (312 bytes)
- `0x167060` NDrone2_DSTATE_AimCrouchFire (784 bytes)
- `0x167370` NDrone2_DSTATE_AimCrouchReload (276 bytes)
- `0x167488` NDrone2_DSTATE_AltAttack (264 bytes)
- `0x1676a0` NDrone2_DSTATE_StepAimRight (272 bytes)
- `0x167980` NDrone2_DSTATE_StrafeAimRight (460 bytes)
- `0x167b50` NDrone2_DSTATE_StrafeDodgeLeft (416 bytes)
- `0x167cf0` NDrone2_DSTATE_StrafeDodgeRight (416 bytes)
- `0x167f80` NDrone2_DSTATE_RollRightCrouch (236 bytes)
- `0x168070` NDrone2_DSTATE_Investigate (392 bytes)
- `0x16a2d0` NDrone2_DSTATE_UnderCoverAim (272 bytes)
- `0x16a9c0` NDrone2_DSTATE_UnderCoverReturn (376 bytes)
- `0x16ab38` NDrone2_DSTATE_UnderCoverLeaveNow (404 bytes)
- `0x16acd0` NDrone2_DSTATE_SeenDeadBody (272 bytes)
- `0x16ade0` NDrone2_DSTATE_SeenSurrenderedDrone (316 bytes)
- `0x16db90` NDrone2_DSTATE_Tester1 (408 bytes)
- `0x1724f0` NDrone2_DSTATE_InitPatrol (176 bytes)
- `0x1725a0` NDrone2_DSTATE_HostageDie (136 bytes)
- `0x172628` NDrone2_DSTATE_HostageDead (260 bytes)
- `0x1729f0` NDrone2_DSTATE_Interogator (60 bytes)
- `0x173178` NDrone2_DSTATE_Alerted1stEncounter (60 bytes)
- `0x173280` NDrone2_DSTATE_CombatNewSighting (180 bytes)
- `0x173338` NDrone2_DSTATE_CombatTooClose (84 bytes)
- `0x173390` NDrone2_DSTATE_Obstructed (236 bytes)
- `0x173570` NDrone2_DSTATE_HoldItRightThere (228 bytes)
- `0x1739b8` NDrone2_DSTATE_UnderCoverTypeChange (260 bytes)
- `0x173ac0` NDrone2_DSTATE_UnderCoverLeave (116 bytes)
- `0x173b38` NDrone2_DSTATE_SeenOpponent (60 bytes)
- `0x173b78` NDrone2_DSTATE_SeenDroneShot (60 bytes)
- `0x173bb8` NDrone2_DSTATE_SeenExplosive (48 bytes)
- `0x173f68` NDrone2_DSTATE_FailMission (88 bytes)
- `0x174150` NDrone2_DSTATE_Tester2 (116 bytes)
- `0x1741c8` NDrone2_DSTATE_Tester3 (272 bytes)
- `0x1742d8` NDrone2_DSTATE_Tester4 (256 bytes)
- `0x1743d8` NDrone2_DSTATE_HangUp (84 bytes)
- `0x174430` NDrone2_DSTATE_WaitForever (36 bytes)
- `0x178678` NDrone2_NearArmedBond (236 bytes)
- `0x1788a0` NDrone2_InShoutingRange (272 bytes)

## P — 4 functions, 8888 bytes

- `0x2067c8` P_TWEAKS_Handler (4488 bytes)
- `0x207950` P_TWEAKS2_Handler (3424 bytes)
- `0x20a968` P_FMVTEST_Handler (968 bytes)
- `0x2169b0` P_FMV_Handler (8 bytes)

## GT — 13 functions, 7832 bytes

- `0x17f9c8` GT_Track (1492 bytes)
- `0x17ffa0` GT_Create (936 bytes)
- `0x180348` GT_RandMove (332 bytes)
- `0x180498` GT_Update (3948 bytes)
- `0x181408` GT_LoseControl (304 bytes)
- `0x181538` GT_Register (36 bytes)
- `0x181560` GT_DeployMiniGun (276 bytes)
- `0x181678` GT_Delete (48 bytes)
- `0x1816a8` GT_Disable (44 bytes)
- `0x1816d8` GT_ApplyDamage (20 bytes)
- `0x1816f0` GT_Activate (76 bytes)
- `0x181740` GT_TakeControl (200 bytes)
- `0x181808` GT_PlayerHasDied (120 bytes)

##  — 23 functions, 4864 bytes

- `0x17e468`  (80 bytes)
- `0x1b1338`  (240 bytes)
- `0x1cdd50`  (616 bytes)
- `0x1f1c18`  (420 bytes)
- `0x1f24e0`  (544 bytes)
- `0x1f2b10`  (460 bytes)
- `0x1f2ce0`  (332 bytes)
- `0x1f33c0`  (208 bytes)
- `0x224368`  (176 bytes)
- `0x224418`  (32 bytes)
- `0x224548`  (144 bytes)
- `0x2246a0`  (44 bytes)
- `0x224840`  (152 bytes)
- `0x224ff0`  (276 bytes)
- `0x225348`  (44 bytes)
- `0x236608`  (8 bytes)
- `0x237058`  (180 bytes)
- `0x239fa0`  (140 bytes)
- `0x23b400`  (104 bytes)
- `0x23b468`  (400 bytes)
- `0x23b870`  (128 bytes)
- `0x23b8f0`  (104 bytes)
- `0x23b958`  (32 bytes)

## List — 6 functions, 4632 bytes

- `0x219488` List_Init (608 bytes)
- `0x21a598` List_Update (3700 bytes)
- `0x21b420` List_GetIndex (76 bytes)
- `0x21b470` List_GetIndex_RealItem (84 bytes)
- `0x21b4c8` List_SetIndex (84 bytes)
- `0x21b520` List_SetIndex_RealItem (80 bytes)

## HUD — 13 functions, 4232 bytes

- `0x19cc90` HUD_CreateOICWPane (612 bytes)
- `0x19cef8` HUD_UpdateOICWPane (800 bytes)
- `0x19d218` HUD_UpdateNightSightPane (412 bytes)
- `0x19e9d8` HUD_MonitorNightSight (820 bytes)
- `0x1a0bb8` HUD_CreateShrink (332 bytes)
- `0x1a0ed8` HUD_CreateAmmoPane (104 bytes)
- `0x1a0f40` HUD_CreateHealthPane (228 bytes)
- `0x1a10b8` HUD_CreateInfoStatusPane (132 bytes)
- `0x1a1140` HUD_CreatePickupStatusPane (132 bytes)
- `0x1a12d0` HUD_UpdateSightPane (8 bytes)
- `0x1a1410` HUD_UpdateXRayPane (296 bytes)
- `0x1a1538` HUD_UpdateSecCamPane (100 bytes)
- `0x1a15a0` HUD_UpdateCarPane (256 bytes)

## AINetwork — 8 functions, 4044 bytes

- `0x156e18` AINetwork_NodeSearchCel (916 bytes)
- `0x157780` AINetwork_HitBoundry (524 bytes)
- `0x157990` AINetwork_LineIntersectsBoundry (572 bytes)
- `0x159268` AINetwork_Emitter_GetNodeAtDistance (1024 bytes)
- `0x15a490` AINetwork_UpdateLinksUsedCount (308 bytes)
- `0x15aa00` AINetwork_ResetRoute (132 bytes)
- `0x15ab78` AINetwork_TestRayCels (248 bytes)
- `0x15b6f0` AINetwork_ModIntersectedLinkFlags_Bounds (320 bytes)

## DroneFunc — 17 functions, 3924 bytes

- `0x13f420` DroneFunc_NewSightTalk (304 bytes)
- `0x13f550` DroneFunc_LostSightTalk (312 bytes)
- `0x13fa20` DroneFunc_InjuredTalk (376 bytes)
- `0x13fb98` DroneFunc_UnderCoverAimTalk (364 bytes)
- `0x13fd08` DroneFunc_RunForCoverTalk (376 bytes)
- `0x13fe80` DroneFunc_UnderCoverTalk (364 bytes)
- `0x141f28` DroneFunc_AllocateTargetID (188 bytes)
- `0x144488` DroneFunc_DoBondTalk (548 bytes)
- `0x147f58` DroneFunc_FirstAlertState (180 bytes)
- `0x149920` DroneFunc_HeardNoiseTalk (176 bytes)
- `0x1499d0` DroneFunc_ClampPosToCollisionCylinder (164 bytes)
- `0x149bd0` DroneFunc_GetBonePos (196 bytes)
- `0x14a008` DroneFunc_TargetSeen (76 bytes)
- `0x14a5e8` DroneFunc_Using2ndBehaviour (8 bytes)
- `0x14a5f0` DroneFunc_Set1stBehaviour (40 bytes)
- `0x14aed8` DroneFunc_SetAsAttacking (164 bytes)
- `0x14b190` DroneFunc_NearPlayerCollision (88 bytes)

## C — 19 functions, 3816 bytes

- `0x208c68` C_LANGUAGE_Handler (524 bytes)
- `0x20db48` C_RBMPFINISH_Handler (784 bytes)
- `0x215ac8` C_LBPMMAP_Handler (292 bytes)
- `0x215c70` C_CHCHFLY_Handler (128 bytes)
- `0x215cf0` C_CHCHDRONES_Handler (108 bytes)
- `0x215d60` C_CHCHBLIND_Handler (100 bytes)
- `0x215dc8` C_CHCHWEAP_Handler (280 bytes)
- `0x215ee0` C_CHCHHEALTH_Handler (100 bytes)
- `0x215f48` C_CHCHHUD_Handler (332 bytes)
- `0x216098` C_CHCHDEBUG_Handler (100 bytes)
- `0x216100` C_CHCHCOORDS_Handler (100 bytes)
- `0x216258` C_CHCHALLOWFREEZE_Handler (92 bytes)
- `0x2162b8` C_CHCHDUMMY_Handler (100 bytes)
- `0x216320` C_CHCHDRAWALL_Handler (100 bytes)
- `0x216400` C_CHCHLOCKUP_Handler (92 bytes)
- `0x216460` C_CHCHBRIGHT_Handler (92 bytes)
- `0x2164c0` C_CHCHUNLOCK_Handler (92 bytes)
- `0x216520` C_CHCHCONTROLS_Handler (92 bytes)
- `0x216810` C_KEYPAD_Handler (308 bytes)

## Text — 5 functions, 3680 bytes

- `0x1b1428` Text_Init (228 bytes)
- `0x1b1640` Text_DelMsg (192 bytes)
- `0x222b80` Text_Init (2688 bytes)
- `0x223600` Text_SendMessage (456 bytes)
- `0x224098` Text_SetupChar (116 bytes)

## Menu — 26 functions, 3584 bytes

- `0x1f13c8` Menu_CursorOverMe (304 bytes)
- `0x1f16d8` Menu_FindControl (1344 bytes)
- `0x1f2708` Menu_GetFirstControl (236 bytes)
- `0x1f27f8` Menu_GetPage (120 bytes)
- `0x1f2870` Menu_InitializeSprite (332 bytes)
- `0x1f2e30` Menu_ResizeControl (20 bytes)
- `0x1f2e48` Menu_ResizeControl (36 bytes)
- `0x1f2e70` Menu_ClipString (168 bytes)
- `0x1f2f18` Menu_Assert (140 bytes)
- `0x1f2fa8` Menu_CreateSprite (60 bytes)
- `0x1f2fe8` Menu_DeleteSprite (32 bytes)
- `0x1f3008` Menu_Malloc (32 bytes)
- `0x1f3028` Menu_Free (28 bytes)
- `0x1f3048` Menu_SetAction (140 bytes)
- `0x1f30d8` Menu_GetLastController (8 bytes)
- `0x1f31e0` Menu_AlphaReset (8 bytes)
- `0x1f31e8` Menu_AlphaGet (8 bytes)
- `0x1f3328` Menu_UpdateTextFmt (108 bytes)
- `0x1f3398` Menu_GetNormalTextFmt (36 bytes)
- `0x1f3508` Menu_ClearDelayedMessages (52 bytes)
- `0x1f3540` Menu_StringInit (8 bytes)
- `0x1f3548` Menu_StringKill (8 bytes)
- `0x201fc0` Menu_MPFadeWhenReady (60 bytes)
- `0x202000` Menu_FlushEnum (8 bytes)
- `0x202280` Menu_UpdateOptionBox (28 bytes)
- `0x2028d8` Menu_SelectCodenameInControl (260 bytes)

## Camera — 8 functions, 3440 bytes

- `0x1b9af0` Camera_Reset (272 bytes)
- `0x1bb0f8` Camera_Create (444 bytes)
- `0x1bb2b8` Camera_CreateCameras (1056 bytes)
- `0x1bb6d8` Camera_Update (332 bytes)
- `0x1bb828` Camera_Orbit (524 bytes)
- `0x1bba38` Camera_Shear (444 bytes)
- `0x1bbbf8` Camera_Set (104 bytes)
- `0x1bbf40` Camera_ApplySwing (264 bytes)

## camera — 4 functions, 3328 bytes

- `0x1b9c00` camera_chasecam (1480 bytes)
- `0x1ba1c8` camera_freechase (796 bytes)
- `0x1ba4e8` camera_fixed (248 bytes)
- `0x1baa78` camera_snake (804 bytes)

## CreateMoveRG2BA — 1 functions, 2828 bytes

- `0x118718` CreateMoveRG2BA (2828 bytes)

## UpdateMFX — 7 functions, 2808 bytes

- `0x114250` UpdateMFX_HT_Level_Tower2A (244 bytes)
- `0x114348` UpdateMFX_HT_Level_Tower2B (476 bytes)
- `0x114528` UpdateMFX_HT_Level_Tower2Elevator (276 bytes)
- `0x114640` UpdateMFX_HT_Level_Tower2C (232 bytes)
- `0x114728` UpdateMFX_HT_Level_EvilBase (780 bytes)
- `0x114a38` UpdateMFX_HT_Level_EvilSilo (256 bytes)
- `0x114b38` UpdateMFX_HT_Level_EvilBaseC (544 bytes)

## Sub — 7 functions, 2764 bytes

- `0x1acda0` Sub_Laser (452 bytes)
- `0x1acf68` Sub_DeployTorpedoes (664 bytes)
- `0x1ad200` Sub_Update (880 bytes)
- `0x1ad570` Sub_ApplyDammage (328 bytes)
- `0x1ad6c0` Sub_Create (424 bytes)
- `0x1ad868` Sub_Collide (8 bytes)
- `0x1ad870` Sub_Delete (8 bytes)

## MP — 15 functions, 2736 bytes

- `0x18a058` MP_UnregisterPickup (272 bytes)
- `0x18abc0` MP_isPlayerOrBotDead (208 bytes)
- `0x18ac90` MP_objectBeingDeleted (1284 bytes)
- `0x18b2c0` MP_ObjectDelete (8 bytes)
- `0x18b468` MP_HasEsponage (36 bytes)
- `0x18b4a0` MP_ObjTeamHasGEObj (152 bytes)
- `0x18b578` MP_Pickup_PostLoadInit (140 bytes)
- `0x18b720` MP_isObjectOnTeam (128 bytes)
- `0x18b848` MP_PostLoadInitObjExt (64 bytes)
- `0x18bbc8` MP_sendTeamBotMessage (88 bytes)
- `0x18bca8` MP_getObjExtObj (208 bytes)
- `0x18be50` MP_resetPickupBotVisitTimes (68 bytes)
- `0x18be98` MP_getMPpickupFromPickupinfo (64 bytes)
- `0x18bed8` MP_pickupCountCheck (8 bytes)
- `0x18bf60` MP_DebugDraw (8 bytes)

## Component — 8 functions, 2516 bytes

- `0x1f0950` Component_SetupInstance (1764 bytes)
- `0x1f1038` Component_InitComponents (36 bytes)
- `0x1f10e8` Component_DestroyAll (172 bytes)
- `0x1f1198` Component_AddComponent (100 bytes)
- `0x1f1200` Component_InitInstance (100 bytes)
- `0x1f1268` Component_GetHandle (116 bytes)
- `0x1f12e0` Component_Free (136 bytes)
- `0x1f1368` Component_Destroy (92 bytes)

## Drone — 13 functions, 2456 bytes

- `0x136ce8` Drone_Delete (776 bytes)
- `0x13a0a0` Drone_DCVfromOBJ (76 bytes)
- `0x13a0f8` Drone_InitOnce (8 bytes)
- `0x13a1d0` Drone_AIVolume_Delete (36 bytes)
- `0x13a1f8` Drone_VisibilityForPosition (148 bytes)
- `0x13a578` Drone_GetSkinHash (28 bytes)
- `0x13a5a8` Drone_LevelReset (132 bytes)
- `0x13a878` Drone_FellOutMap (92 bytes)
- `0x13a910` Drone_AlertStatus (48 bytes)
- `0x149798` Drone_IsTalking (72 bytes)
- `0x172128` Drone_SM_RouteMsg (788 bytes)
- `0x1757d0` Drone_SM_Init (104 bytes)
- `0x1759a0` Drone_SM_SendDelayedMsgs (148 bytes)

## AnimProcessSeqData — 1 functions, 2440 bytes

- `0x1c4ce0` AnimProcessSeqData (2440 bytes)

## Creature — 8 functions, 2424 bytes

- `0x134ab8` Creature_Create (568 bytes)
- `0x134cf0` Creature_CalcPoint (356 bytes)
- `0x134e58` Creature_CalcWaypoints (944 bytes)
- `0x135208` Creature_Init (40 bytes)
- `0x135230` Creature_Delete (52 bytes)
- `0x135268` Creature_Update (100 bytes)
- `0x1352d0` Creature_Hurt (112 bytes)
- `0x135340` Creature_Move (252 bytes)

## Intersect — 1 functions, 2340 bytes

- `0x1e9a70` Intersect_CylTriGeom (2340 bytes)

## BOT — 7 functions, 2328 bytes

- `0x124fb0` BOT_validateStateChange (992 bytes)
- `0x1257c0` BOT_handleOpponentHistory (524 bytes)
- `0x1259d0` BOT_fellOutMap (348 bytes)
- `0x1262b0` BOT_initSys (8 bytes)
- `0x1262b8` BOT_postLoadInit (96 bytes)
- `0x126568` BOT_getMovementSpeedMul (104 bytes)
- `0x126618` BOT_soundEffect (256 bytes)

## DroneAnim — 14 functions, 2304 bytes

- `0x13b9e0` DroneAnim_CanCrouchLean (344 bytes)
- `0x13bb38` DroneAnim_CanCrouchStepOut (296 bytes)
- `0x13bc60` DroneAnim_CanStandLean (388 bytes)
- `0x13bde8` DroneAnim_CanStandStepOut (344 bytes)
- `0x13cc70` DroneAnim_SetWalkIdleAnim (392 bytes)
- `0x13dc90` DroneAnim_ZeroAnimCall (36 bytes)
- `0x13dcb8` DroneAnim_CallFullyComplete (72 bytes)
- `0x13dd00` DroneAnim_SetEndAIState (120 bytes)
- `0x13dd78` DroneAnim_SetDAnimNow (88 bytes)
- `0x13def8` DroneAnim_SetAnimCall (60 bytes)
- `0x13e230` DroneAnim_SnapRotate2 (8 bytes)
- `0x13e3e8` DroneAnim_AnimEndCallback (12 bytes)
- `0x13e3f8` DroneAnim_ForceCrouchCover (88 bytes)
- `0x13e450` DroneAnim_CanUseCornerAsLowCover (56 bytes)

## AnimDebugHandler — 1 functions, 2240 bytes

- `0x1bc6e8` AnimDebugHandler (2240 bytes)

## Collide — 3 functions, 2084 bytes

- `0x1ecc00` Collide_CylinderIntersect (560 bytes)
- `0x1ecfb8` Collide_PlaceObject (1420 bytes)
- `0x1ede00` Collide_FreeHitList (104 bytes)

## BOTSTATE — 10 functions, 2024 bytes

- `0x1267a0` BOTSTATE_reallyWantACombatMove (672 bytes)
- `0x128f80` BOTSTATE_isPathWithinObjectRange (612 bytes)
- `0x1292f0` BOTSTATE_chooseUnarmedAttackAnim (120 bytes)
- `0x129658` BOTSTATE_hasWeapon (36 bytes)
- `0x129918` BOTSTATE_stopAnim (64 bytes)
- `0x1299d0` BOTSTATE_startRecovery (140 bytes)
- `0x129b68` BOTSTATE_reallyCloseCombatRange (44 bytes)
- `0x129b98` BOTSTATE_defaultCombatRange (32 bytes)
- `0x129bb8` BOTSTATE_increaseDistraction (160 bytes)
- `0x129c58` BOTSTATE_isDistracted (144 bytes)

## CreatePointerSpriteList — 1 functions, 1952 bytes

- `0x11a160` CreatePointerSpriteList (1952 bytes)

## Player — 7 functions, 1812 bytes

- `0x193498` Player_InShadow (360 bytes)
- `0x195a88` Player_ResetWalkAnim (16 bytes)
- `0x195bf0` Player_ResetStartPos (16 bytes)
- `0x195d10` Player_InitGrapple (144 bytes)
- `0x1a69d0` Player_SetWatch (1244 bytes)
- `0x1a9790` Player_ClearAimInertia (16 bytes)
- `0x1a97b8` Player_UnEquipKey (16 bytes)

## ASH — 3 functions, 1800 bytes

- `0x109770` ASH_vecutil_Dist2Tri (864 bytes)
- `0x109ad0` ASH_Intersect_RayBox (720 bytes)
- `0x10ae50` ASH_RecurseBoxesBox (216 bytes)

## BodyGlow — 3 functions, 1788 bytes

- `0x1ac158` BodyGlow_Create (368 bytes)
- `0x1ac2c8` BodyGlow_Update (1340 bytes)
- `0x1acd50` BodyGlow_Delete (80 bytes)

## CreateFogSprite — 1 functions, 1764 bytes

- `0x119228` CreateFogSprite (1764 bytes)

## Spin — 3 functions, 1712 bytes

- `0x2224c8` Spin_Init (524 bytes)
- `0x2226d8` Spin_SendMessage (624 bytes)
- `0x222948` Spin_Update (564 bytes)

## Car — 4 functions, 1632 bytes

- `0x130060` Car_SetBits (1356 bytes)
- `0x132d78` Car_Init (44 bytes)
- `0x132da8` Car_Reset (204 bytes)
- `0x132e78` Car_Delete (28 bytes)

## parsemap — 19 functions, 1632 bytes

- `0x1d1430` parsemap_block_map_header (216 bytes)
- `0x1d1e78` parsemap_block_hashlist (152 bytes)
- `0x1d1f80` parsemap_block_texture_header (64 bytes)
- `0x1d1fc0` parsemap_block_palette_header (132 bytes)
- `0x1d2048` parsemap_block_texture_data_pc (24 bytes)
- `0x1d2080` parsemap_block_texture_data_pcdx (44 bytes)
- `0x1d20b0` parsemap_block_palette_data_psx (152 bytes)
- `0x1d2148` parsemap_block_texture_data_GameCube (152 bytes)
- `0x1d21e0` parsemap_block_texture_data_Xbox (152 bytes)
- `0x1d2278` parsemap_block_palette_data_pc (152 bytes)
- `0x1d2310` parsemap_block_dict_xyz (140 bytes)
- `0x1d23a0` parsemap_block_dict_uv (24 bytes)
- `0x1d23b8` parsemap_block_dict_rgba (24 bytes)
- `0x1d23d0` parsemap_block_dict_norm (24 bytes)
- `0x1d24b0` parsemap_block_dict_comlist (24 bytes)
- `0x1d24c8` parsemap_block_coll_data (8 bytes)
- `0x1d24f8` parsemap_block_datums (60 bytes)
- `0x1d2538` parsemap_block_memory_discard (24 bytes)
- `0x1d2550` parsemap_reset_to_block_memory_discard (64 bytes)

## CreateSpaceAndPackTextures — 1 functions, 1516 bytes

- `0x11dcf8` CreateSpaceAndPackTextures (1516 bytes)

## CreateTotalScreenSprite — 1 functions, 1380 bytes

- `0x1181b0` CreateTotalScreenSprite (1380 bytes)

## SSys — 4 functions, 1332 bytes

- `0x1ad878` SSys_Msg (284 bytes)
- `0x1ad998` SSys_Monitor (804 bytes)
- `0x1b01b0` SSys_Init (84 bytes)
- `0x1b0208` SSys_RamLoad (160 bytes)

## DroneMove — 1 functions, 1276 bytes

- `0x14f140` DroneMove_SetBoundryFlags (1276 bytes)

## SpaceLaser — 2 functions, 1240 bytes

- `0x1aa3f8` SpaceLaser_Update (1216 bytes)
- `0x1aa9e8` SpaceLaser_Register (24 bytes)

## CreateZClearSprite — 1 functions, 1224 bytes

- `0x117ce8` CreateZClearSprite (1224 bytes)

## vecutil — 2 functions, 1156 bytes

- `0x1dfac8` vecutil_Dist2Tri_PointGeom (480 bytes)
- `0x1e0aa8` vecutil_intersect_line_line_2d (676 bytes)

## MiniSub — 5 functions, 1148 bytes

- `0x181888` MiniSub_FireTorpedo (336 bytes)
- `0x181f40` MiniSub_Update (488 bytes)
- `0x182128` MiniSub_Collide (8 bytes)
- `0x182130` MiniSub_Create (308 bytes)
- `0x182268` MiniSub_Delete (8 bytes)

## FillMatrixChainRot — 1 functions, 1140 bytes

- `0x122310` FillMatrixChainRot (1140 bytes)

## LList — 6 functions, 1132 bytes

- `0x1cc600` LList_InsertAt (368 bytes)
- `0x1cc958` LList_Init (204 bytes)
- `0x1cca80` LList_Cut (68 bytes)
- `0x1ccac8` LList_Remove (156 bytes)
- `0x1ccb68` LList_Insert (120 bytes)
- `0x1ccbe0` LList_AllocnNodes (216 bytes)

## MiniSubState — 1 functions, 1088 bytes

- `0x181b00` MiniSubState_Attack (1088 bytes)

## CreateTexUploadPacket — 1 functions, 1076 bytes

- `0x11ad08` CreateTexUploadPacket (1076 bytes)

## hashtable — 7 functions, 1076 bytes

- `0x1ca1b0` hashtable_deleteitem (248 bytes)
- `0x1ca2a8` hashtable_create_sections (300 bytes)
- `0x1ca4b8` hashtable_initialise (100 bytes)
- `0x1ca648` hashtable_modify (140 bytes)
- `0x1ca840` hashtable_celglist_to_hashcode (80 bytes)
- `0x1ca938` hashtable_get_hashtype_count (100 bytes)
- `0x1ca9a0` hashtable_get_hashtype_data (108 bytes)

## CreateTexTranPacket — 1 functions, 1028 bytes

- `0x11a900` CreateTexTranPacket (1028 bytes)

## CheckObjectClip — 1 functions, 976 bytes

- `0x121a60` CheckObjectClip (976 bytes)

## Script — 9 functions, 976 bytes

- `0x1daea8` Script_Init (20 bytes)
- `0x1db328` Script_AnimEnd (28 bytes)
- `0x1db920` Script_SetPosRot (136 bytes)
- `0x220f30` Script_DelAllScripts (128 bytes)
- `0x220fb0` Script_AddScript (180 bytes)
- `0x221068` Script_DelScript (152 bytes)
- `0x221100` Script_DelKeyFrame (152 bytes)
- `0x221198` Script_AddMessage (120 bytes)
- `0x221210` Script_DelMessage (60 bytes)

## CheckObjectClipBox — 1 functions, 968 bytes

- `0x121e30` CheckObjectClipBox (968 bytes)

## AnimDebugCreate — 1 functions, 896 bytes

- `0x1bcfa8` AnimDebugCreate (896 bytes)

## Env — 4 functions, 848 bytes

- `0x17be00` Env_Breath (584 bytes)
- `0x17c118` Env_WaterRipple (148 bytes)
- `0x17c1b0` Env_Reset (76 bytes)
- `0x17c328` Env_Tree_Update (40 bytes)

## Combo — 2 functions, 824 bytes

- `0x1f0248` Combo_Init (592 bytes)
- `0x1f0868` Combo_SendMessage (232 bytes)

## AddDMA — 1 functions, 800 bytes

- `0x123a48` AddDMA_IRQ (800 bytes)

## system — 1 functions, 796 bytes

- `0x10c418` system_input_init (796 bytes)

## DroneVision — 6 functions, 788 bytes

- `0x1766f8` DroneVision_DetermineAlertCheck (212 bytes)
- `0x1781a8` DroneVision_ObjectVisibility (252 bytes)
- `0x178308` DroneVision_SetInterogateAssist (16 bytes)
- `0x178318` DroneVision_EndInterogateAssist (44 bytes)
- `0x178348` DroneVision_TestEndInterogationAssist (72 bytes)
- `0x178520` DroneVision_GunThroughWall (192 bytes)

## PageConv8to32 — 1 functions, 776 bytes

- `0x11eba8` PageConv8to32 (776 bytes)

## View — 6 functions, 768 bytes

- `0x1e4ff0` View_RotTransScaleMatrix (376 bytes)
- `0x1e77f0` View_Init (8 bytes)
- `0x1e77f8` View_DrawGlist (76 bytes)
- `0x1e7848` View_AddSprite (100 bytes)
- `0x1e7ac8` View_SetViewMask (100 bytes)
- `0x1e7b30` View_DrawInThisViewOnly (108 bytes)

## Conv4to32 — 1 functions, 720 bytes

- `0x11e8d8` Conv4to32 (720 bytes)

## Checkbox — 2 functions, 708 bytes

- `0x1efd68` Checkbox_SendMessage (396 bytes)
- `0x1f0110` Checkbox_Init (312 bytes)

## CreateDB — 1 functions, 684 bytes

- `0x10b000` CreateDB (684 bytes)

## dpmul — 1 functions, 680 bytes

- `0x224be0` dpmul (680 bytes)

## PageConv4to32 — 1 functions, 664 bytes

- `0x11e640` PageConv4to32 (664 bytes)

## Sprite — 7 functions, 652 bytes

- `0x1de600` Sprite_InitLists (40 bytes)
- `0x1de628` Sprite_SetFmt (108 bytes)
- `0x1de698` Sprite_SetText (128 bytes)
- `0x1de718` Sprite_SetTextNFmt (152 bytes)
- `0x1de7b0` Sprite_SetParams (88 bytes)
- `0x1de808` Sprite_Create (116 bytes)
- `0x1de918` Sprite_Link2Viewer (20 bytes)

## Copter — 4 functions, 640 bytes

- `0x17d828` Copter_FireGun (452 bytes)
- `0x17e4b8` Copter_GetBody (56 bytes)
- `0x17e4f0` Copter_Init (60 bytes)
- `0x17e568` Copter_Delete (72 bytes)

## setLoadImageTagsYX — 1 functions, 636 bytes

- `0x107910` setLoadImageTagsYX (636 bytes)

## Button — 2 functions, 636 bytes

- `0x1ef7a0` Button_SendMessage (420 bytes)
- `0x1efc90` Button_Init (216 bytes)

## DList — 3 functions, 620 bytes

- `0x1cc868` DList_RemoveFromInUse2Free (240 bytes)
- `0x1cccb8` DList_MoveFromFree2InUse (164 bytes)
- `0x1ccd60` DList_AllocnNodes (216 bytes)

## Input — 3 functions, 616 bytes

- `0x17ebe0` Input_Check4Combo (500 bytes)
- `0x17f1c0` Input_Ready (60 bytes)
- `0x17f710` Input_RumblePause (56 bytes)

## Debris — 1 functions, 576 bytes

- `0x1ac808` Debris_CreateEx (576 bytes)

## CheckAndAddTextureToSpace — 1 functions, 564 bytes

- `0x11d8b8` CheckAndAddTextureToSpace (564 bytes)

## quorem — 1 functions, 532 bytes

- `0x238a40` quorem (532 bytes)

## Font — 3 functions, 524 bytes

- `0x1ef588` Font_GetTextExtent (220 bytes)
- `0x1ef668` Font_GetTextCountExtent (188 bytes)
- `0x1ef728` Font_GetAlignment (116 bytes)

## Reduce — 1 functions, 520 bytes

- `0x11daf0` Reduce (520 bytes)

## LinkedList — 6 functions, 508 bytes

- `0x219280` LinkedList_Init (32 bytes)
- `0x2192a0` LinkedList_ReInit (28 bytes)
- `0x2192c0` LinkedList_Add (88 bytes)
- `0x219318` LinkedList_Insert (152 bytes)
- `0x2193b0` LinkedList_Remove (116 bytes)
- `0x219428` LinkedList_GetIndex (92 bytes)

## VIFDMA — 1 functions, 504 bytes

- `0x123e90` VIFDMA (504 bytes)

## InitTLB32MB — 1 functions, 500 bytes

- `0x23fe38` InitTLB32MB (500 bytes)

## AddDMAChainRegSet — 1 functions, 476 bytes

- `0x123520` AddDMAChainRegSet (476 bytes)

## RunNextModule — 1 functions, 436 bytes

- `0x120008` RunNextModule (436 bytes)

## SP — 3 functions, 436 bytes

- `0x1dc958` SP_RemoveObj (52 bytes)
- `0x1dc9e8` SP_SetPosRot (108 bytes)
- `0x1dcad8` SP_SwitchToScript (276 bytes)

## MulMatrix0 — 1 functions, 436 bytes

- `0x1e2820` MulMatrix0 (436 bytes)

## cvt — 1 functions, 432 bytes

- `0x2386a0` cvt (432 bytes)

## Radio — 1 functions, 428 bytes

- `0x21ff80` Radio_Update (428 bytes)

## FillMatrixChainSkin — 1 functions, 416 bytes

- `0x122788` FillMatrixChainSkin (416 bytes)

## RotTransMat — 1 functions, 412 bytes

- `0x1e29d8` RotTransMat (412 bytes)

## Txt — 4 functions, 408 bytes

- `0x1caa10` Txt_GetStringFromHeap (192 bytes)
- `0x1cad00` Txt_LanguageInit (56 bytes)
- `0x1cae10` Txt_UnlockString (84 bytes)
- `0x1cae68` Txt_LockString (76 bytes)

## DroneWeap — 2 functions, 396 bytes

- `0x17a5a0` DroneWeap_GetWeaponInfo (316 bytes)
- `0x17a870` DroneWeap_ManageBurstCount (80 bytes)

## SFXInitialise — 1 functions, 388 bytes

- `0x1157b0` SFXInitialise (388 bytes)

## LinkCreep — 2 functions, 384 bytes

- `0x159cc8` LinkCreep_CalcToRouteEnd (264 bytes)
- `0x15b420` LinkCreep_Dest (120 bytes)

## expf — 1 functions, 372 bytes

- `0x232f70` expf (372 bytes)

## FillSpriteData — 1 functions, 364 bytes

- `0x120cf8` FillSpriteData (364 bytes)

## MulMatrix2 — 1 functions, 364 bytes

- `0x1e26b0` MulMatrix2 (364 bytes)

## RecurseBoxesSph — 1 functions, 356 bytes

- `0x1edae0` RecurseBoxesSph (356 bytes)

## dpdiv — 1 functions, 356 bytes

- `0x224e88` dpdiv (356 bytes)

## Trigger — 3 functions, 352 bytes

- `0x1b2ed8` Trigger_MultiplexOut (208 bytes)
- `0x1b2fa8` Trigger_Type (28 bytes)
- `0x1b2fc8` Trigger_Locked (116 bytes)

## GetBitPos — 1 functions, 348 bytes

- `0x11f808` GetBitPos (348 bytes)

## AnimObjectDelete — 1 functions, 344 bytes

- `0x1bee80` AnimObjectDelete (344 bytes)

## RecurseBoxesBox — 1 functions, 344 bytes

- `0x1edc48` RecurseBoxesBox (344 bytes)

## scalbnf — 1 functions, 344 bytes

- `0x235710` scalbnf (344 bytes)

## AnimScriptNew — 1 functions, 340 bytes

- `0x1c1398` AnimScriptNew (340 bytes)

## Spline — 1 functions, 340 bytes

- `0x1e4330` Spline_Eval3D (340 bytes)

## dmaRefImage — 1 functions, 340 bytes

- `0x22ffd8` dmaRefImage (340 bytes)

## sendToIOP2area — 1 functions, 328 bytes

- `0x107698` sendToIOP2area (328 bytes)

## Leaf — 1 functions, 328 bytes

- `0x1abee0` Leaf_Update (328 bytes)

## control — 5 functions, 324 bytes

- `0x1341f0` control_AnimUpdate (28 bytes)
- `0x134388` control_init_object_lists (60 bytes)
- `0x134608` control_unlink_object (40 bytes)
- `0x134878` control_init_collision (8 bytes)
- `0x134880` control_ClosestPlayerPnt (188 bytes)

## Coll — 2 functions, 324 bytes

- `0x1edda0` Coll_ResetHitHeap (92 bytes)
- `0x1ede68` Coll_AddHitToList (232 bytes)

## videoCallback — 1 functions, 316 bytes

- `0x108848` videoCallback (316 bytes)

## fmodf — 1 functions, 316 bytes

- `0x2330e8` fmodf (316 bytes)

## MapPlayerToSlot — 1 functions, 312 bytes

- `0x10cc48` MapPlayerToSlot (312 bytes)

## AddDMAChainSprite — 1 functions, 312 bytes

- `0x123910` AddDMAChainSprite (312 bytes)

## SphereSphereSweep — 1 functions, 312 bytes

- `0x14e5b8` SphereSphereSweep (312 bytes)

## RecurseAndDrawBoxes — 1 functions, 308 bytes

- `0x1233e8` RecurseAndDrawBoxes (308 bytes)

## RotTransMatrix — 1 functions, 304 bytes

- `0x1e2478` RotTransMatrix (304 bytes)

## Page — 2 functions, 304 bytes

- `0x21f250` Page_FindHelpControl (104 bytes)
- `0x21f2b8` Page_SetHelpText (200 bytes)

## Create — 1 functions, 300 bytes

- `0x1aa8b8` Create_SpaceLaser (300 bytes)

## SFXSuspendFileAccess — 1 functions, 296 bytes

- `0x1165d0` SFXSuspendFileAccess (296 bytes)

## SFXUnSuspendFileAccess — 1 functions, 296 bytes

- `0x1166f8` SFXUnSuspendFileAccess (296 bytes)

## MiniSubMove — 1 functions, 296 bytes

- `0x1819d8` MiniSubMove_Forward (296 bytes)

## Stack — 4 functions, 296 bytes

- `0x1cce38` Stack_Init (120 bytes)
- `0x1cceb0` Stack_IsEmpty (28 bytes)
- `0x1cced0` Stack_Top (68 bytes)
- `0x1ccf68` Stack_Pop (80 bytes)

## AddDMAChainPacket — 1 functions, 292 bytes

- `0x123d68` AddDMAChainPacket (292 bytes)

## nodataCallback — 1 functions, 288 bytes

- `0x108a60` nodataCallback (288 bytes)

## AllocateTListPos — 1 functions, 288 bytes

- `0x121650` AllocateTListPos (288 bytes)

## CirBuf — 2 functions, 284 bytes

- `0x1ccfb8` CirBuf_Init (132 bytes)
- `0x1cd040` CirBuf_Add (152 bytes)

## LS — 3 functions, 284 bytes

- `0x1ce1b0` LS_FlushStates (12 bytes)
- `0x1ce1c8` LS_LoadFromBuffer (148 bytes)
- `0x1ce488` LS_LoadBonus (124 bytes)

## MenuManager — 4 functions, 280 bytes

- `0x21b570` MenuManager_Init (156 bytes)
- `0x21de90` MenuManager_Exists (32 bytes)
- `0x21deb0` MenuManager_GetStatus (60 bytes)
- `0x21def0` MenuManager_Enable (32 bytes)

## EndDMAChain — 1 functions, 276 bytes

- `0x1221f8` EndDMAChain (276 bytes)

## DroneInit — 1 functions, 276 bytes

- `0x14b1e8` DroneInit_Collision (276 bytes)

## BlockConv4to32 — 1 functions, 268 bytes

- `0x11e530` BlockConv4to32 (268 bytes)

## AnimScriptDelete — 1 functions, 268 bytes

- `0x1c14f0` AnimScriptDelete (268 bytes)

## AnimObjectSetHeadTracking — 1 functions, 264 bytes

- `0x1c6748` AnimObjectSetHeadTracking (264 bytes)

## fflush — 1 functions, 260 bytes

- `0x239e28` fflush (260 bytes)

## asinf — 1 functions, 252 bytes

- `0x232d48` asinf (252 bytes)

## Sound — 4 functions, 244 bytes

- `0x1ddf88` Sound_Init (68 bytes)
- `0x1ddfd0` Sound_Kill (40 bytes)
- `0x1de190` Sound_SetVolume (128 bytes)
- `0x1de368` Sound_SetLogSfxFile (8 bytes)

## FillTextureDMA — 1 functions, 240 bytes

- `0x11d110` FillTextureDMA (240 bytes)

## CacheAndFillNewGlistTextures — 1 functions, 240 bytes

- `0x11f180` CacheAndFillNewGlistTextures (240 bytes)

## AnimSeqGetInfo — 1 functions, 240 bytes

- `0x1c79a0` AnimSeqGetInfo (240 bytes)

## CacheTextureSpace — 1 functions, 232 bytes

- `0x11cc70` CacheTextureSpace (232 bytes)

## Ascii2Sjis — 1 functions, 232 bytes

- `0x1201c0` Ascii2Sjis (232 bytes)

## floorf — 1 functions, 228 bytes

- `0x235600` floorf (228 bytes)

## CreateSortedTextures — 1 functions, 220 bytes

- `0x11d7d8` CreateSortedTextures (220 bytes)

## AnimPreLoadInit — 1 functions, 216 bytes

- `0x1c63b8` AnimPreLoadInit (216 bytes)

## audioCallback — 1 functions, 212 bytes

- `0x108988` audioCallback (212 bytes)

## vision — 2 functions, 212 bytes

- `0x1e8848` vision_generate_display_list (192 bytes)
- `0x1e8990` vision_GetCamPos (20 bytes)

## iopGetArea — 1 functions, 204 bytes

- `0x1075c8` iopGetArea (204 bytes)

## setD4 — 2 functions, 200 bytes

- `0x232618` setD4_CHCR (100 bytes)
- `0x232920` setD4_CHCR (100 bytes)

## Control — 3 functions, 188 bytes

- `0x134940` Control_SetEffectType (28 bytes)
- `0x1349e8` Control_DeleteAllObjectsOfType (76 bytes)
- `0x134a38` Control_ReturnNextObjectOfType (84 bytes)

## CacheFlipBuffers — 1 functions, 184 bytes

- `0x1095d0` CacheFlipBuffers (184 bytes)

## litodp — 1 functions, 184 bytes

- `0x225158` litodp (184 bytes)

## fillBuff — 1 functions, 180 bytes

- `0x108c08` fillBuff (180 bytes)

## Sunreflection — 1 functions, 180 bytes

- `0x18db40` Sunreflection_Update (180 bytes)

## Bubble — 1 functions, 180 bytes

- `0x1aca48` Bubble_Update (180 bytes)

## AllocateNextTListPos — 1 functions, 176 bytes

- `0x124458` AllocateNextTListPos (176 bytes)

## RotMatrixXY — 1 functions, 176 bytes

- `0x1e4818` RotMatrixXY (176 bytes)

## exit — 1 functions, 176 bytes

- `0x2358b0` exit (176 bytes)

## deci2Putchar — 1 functions, 176 bytes

- `0x23cc90` deci2Putchar (176 bytes)

## LimitRadAngle — 1 functions, 172 bytes

- `0x120510` LimitRadAngle (172 bytes)

## AnimObjectInitialPose — 1 functions, 172 bytes

- `0x1c64e0` AnimObjectInitialPose (172 bytes)

## sbrk — 1 functions, 172 bytes

- `0x23c298` sbrk (172 bytes)

## FS — 1 functions, 168 bytes

- `0x10bc48` FS_Init (168 bytes)

## auxVec — 3 functions, 168 bytes

- `0x1e4f10` auxVec_Add (52 bytes)
- `0x1e4f48` auxVec_Sub (52 bytes)
- `0x1e4f98` auxVec_AddMulR32 (64 bytes)

## SFXReset2 — 1 functions, 164 bytes

- `0x116a38` SFXReset2 (164 bytes)

## SFXSetFilePath — 1 functions, 164 bytes

- `0x116c50` SFXSetFilePath (164 bytes)

## Process — 2 functions, 164 bytes

- `0x21f780` Process_Init (36 bytes)
- `0x21f860` Process_Destroy (128 bytes)

## zerobss — 1 functions, 164 bytes

- `0x2242a8` zerobss (164 bytes)

## VSync2 — 1 functions, 164 bytes

- `0x23c0e8` VSync2 (164 bytes)

## BOTWEAP — 3 functions, 160 bytes

- `0x12af70` BOTWEAP_setDroneArmour (8 bytes)
- `0x12b010` BOTWEAP_getWeaponClipSize (36 bytes)
- `0x12b140` BOTWEAP_getWeaponAnimSet (116 bytes)

## OneSided — 1 functions, 160 bytes

- `0x1e7d58` OneSided_Update (160 bytes)

## dptoul — 1 functions, 160 bytes

- `0x2252a8` dptoul (160 bytes)

## audioDecCreate — 1 functions, 156 bytes

- `0x1072b8` audioDecCreate (156 bytes)

## build — 4 functions, 156 bytes

- `0x1c9378` build_alloc_viewer (76 bytes)
- `0x1c93c8` build_alloc_world (44 bytes)
- `0x1c94d8` build_link_world_to_viewer (28 bytes)
- `0x1c94f8` build_MakeCelOverlapBox (8 bytes)

## AIBounds — 1 functions, 156 bytes

- `0x1d3008` AIBounds_Prepare (156 bytes)

## AsciiString2Sjis — 1 functions, 152 bytes

- `0x120b88` AsciiString2Sjis (152 bytes)

## AnimScriptPeekSfx — 1 functions, 152 bytes

- `0x1c6ca8` AnimScriptPeekSfx (152 bytes)

## ApplyMatrixLV — 1 functions, 148 bytes

- `0x1e48c8` ApplyMatrixLV (148 bytes)

## dptoli — 1 functions, 148 bytes

- `0x225210` dptoli (148 bytes)

## cmd — 1 functions, 148 bytes

- `0x230d58` cmd_sem_init (148 bytes)

## iWakeupThread — 1 functions, 148 bytes

- `0x23c698` iWakeupThread (148 bytes)

## Light — 1 functions, 144 bytes

- `0x1cc340` Light_Init (144 bytes)

## mceIntrReadFixAlign — 1 functions, 144 bytes

- `0x228248` mceIntrReadFixAlign (144 bytes)

## VSync — 1 functions, 144 bytes

- `0x23c058` VSync (144 bytes)

## ftoi — 1 functions, 144 bytes

- `0x23cd78` ftoi (144 bytes)

## AnimGetFrameTransRaw — 1 functions, 140 bytes

- `0x1c73d0` AnimGetFrameTransRaw (140 bytes)

## tanf — 1 functions, 136 bytes

- `0x232bc0` tanf (136 bytes)

## new — 1 functions, 136 bytes

- `0x23e5f8` new_iob (136 bytes)

## AgeSkinCache — 1 functions, 132 bytes

- `0x111b08` AgeSkinCache (132 bytes)

## DrawAllChildBoxes — 1 functions, 132 bytes

- `0x124680` DrawAllChildBoxes (132 bytes)

## Cameras — 1 functions, 132 bytes

- `0x1bc3a8` Cameras_PostLoad (132 bytes)

## Window — 2 functions, 132 bytes

- `0x224110` Window_Init (124 bytes)
- `0x224270` Window_Update (8 bytes)

## mceStorePwd — 1 functions, 132 bytes

- `0x228a08` mceStorePwd (132 bytes)

## sendToIOP — 1 functions, 128 bytes

- `0x1077e0` sendToIOP (128 bytes)

## SwapBGFMVFrame — 1 functions, 128 bytes

- `0x1087c8` SwapBGFMVFrame (128 bytes)

## backgroundCallback — 1 functions, 128 bytes

- `0x108b80` backgroundCallback (128 bytes)

## cdvd — 1 functions, 128 bytes

- `0x230df0` cdvd_exit (128 bytes)

## audioDecEndPut — 1 functions, 124 bytes

- `0x107530` audioDecEndPut (124 bytes)

## SFXSetNumberOfListeners — 1 functions, 124 bytes

- `0x116ec0` SFXSetNumberOfListeners (124 bytes)

## PlarStat — 2 functions, 124 bytes

- `0x199520` PlarStat_LogTimerPause (44 bytes)
- `0x199550` PlarStat_LogTimerUnpause (80 bytes)

## LoaderShrink — 1 functions, 124 bytes

- `0x1d6b68` LoaderShrink (124 bytes)

## RotMatrixY — 1 functions, 124 bytes

- `0x1e4718` RotMatrixY (124 bytes)

## write — 1 functions, 124 bytes

- `0x23c190` write (124 bytes)

## SyncDCache — 1 functions, 124 bytes

- `0x23c410` SyncDCache (124 bytes)

## initcache — 1 functions, 120 bytes

- `0x109688` initcache (120 bytes)

## RotMatrixZ — 1 functions, 120 bytes

- `0x1e4798` RotMatrixZ (120 bytes)

## vfprintf — 1 functions, 120 bytes

- `0x237110` vfprintf (120 bytes)

## Door — 2 functions, 116 bytes

- `0x136810` Door_IsLocked (56 bytes)
- `0x136848` Door_Init (60 bytes)

## DroneSpawner — 1 functions, 116 bytes

- `0x13a338` DroneSpawner_Delete (116 bytes)

## read — 1 functions, 116 bytes

- `0x23c210` read (116 bytes)

## allocache — 1 functions, 112 bytes

- `0x109700` allocache (112 bytes)

## AIPath — 2 functions, 112 bytes

- `0x1d2ea8` AIPath_Init (32 bytes)
- `0x1d2ec8` AIPath_Link2Cel (80 bytes)

## BIN — 3 functions, 112 bytes

- `0x1e8ad8` BIN_GetByte (20 bytes)
- `0x1e8af0` BIN_GetWord (32 bytes)
- `0x1e8b48` BIN_GetFloat (60 bytes)

## receiveDataFromIPU — 1 functions, 112 bytes

- `0x230130` receiveDataFromIPU (112 bytes)

## kCopy — 2 functions, 112 bytes

- `0x23c650` kCopy (56 bytes)
- `0x23fda8` kCopy (56 bytes)

## get — 1 functions, 108 bytes

- `0x23e680` get_iob (108 bytes)

## DisableIntc — 1 functions, 104 bytes

- `0x23c4b0` DisableIntc (104 bytes)

## EnableIntc — 1 functions, 104 bytes

- `0x23c518` EnableIntc (104 bytes)

## DisableDmac — 1 functions, 104 bytes

- `0x23c580` DisableDmac (104 bytes)

## EnableDmac — 1 functions, 104 bytes

- `0x23c5e8` EnableDmac (104 bytes)

## GunImp — 1 functions, 100 bytes

- `0x133008` GunImp_PlayerHasDied (100 bytes)

## AnimSeqDelete — 1 functions, 100 bytes

- `0x1c72e0` AnimSeqDelete (100 bytes)

## ConvLanguageToFlagIndex — 1 functions, 100 bytes

- `0x216948` ConvLanguageToFlagIndex (100 bytes)

## dpsub — 1 functions, 100 bytes

- `0x224b78` dpsub (100 bytes)

## PowerOffCB — 1 functions, 100 bytes

- `0x230eb8` PowerOffCB (100 bytes)

## setD3 — 1 functions, 100 bytes

- `0x2325b0` setD3_CHCR (100 bytes)

## ResetDBufData — 1 functions, 96 bytes

- `0x1243a0` ResetDBufData (96 bytes)

## AddTextureToChain — 1 functions, 88 bytes

- `0x124400` AddTextureToChain (88 bytes)

## dpadd — 1 functions, 88 bytes

- `0x224b20` dpadd (88 bytes)

## std — 1 functions, 88 bytes

- `0x239f30` std (88 bytes)

## Set — 1 functions, 84 bytes

- `0x1a98a8` Set_Upgrade (84 bytes)

## Lock — 1 functions, 84 bytes

- `0x1b0750` Lock_ReLock (84 bytes)

## Plane — 1 functions, 84 bytes

- `0x1e3698` Plane_PutPntOnPlane (84 bytes)

## dptofp — 1 functions, 84 bytes

- `0x225378` dptofp (84 bytes)

## mceGetInfoApdx — 1 functions, 84 bytes

- `0x2286c8` mceGetInfoApdx (84 bytes)

## setDMAscTag — 1 functions, 80 bytes

- `0x107bd0` setDMAscTag (80 bytes)

## setGIFtag — 1 functions, 80 bytes

- `0x107c20` setGIFtag (80 bytes)

## InitaliseMemoryCard — 1 functions, 76 bytes

- `0x110f48` InitaliseMemoryCard (76 bytes)

## PlrStat — 1 functions, 76 bytes

- `0x1993e0` PlrStat_OkToUpdate (76 bytes)

## dpcmp — 1 functions, 76 bytes

- `0x225108` dpcmp (76 bytes)

## GetMinWidth — 1 functions, 72 bytes

- `0x1205c8` GetMinWidth (72 bytes)

## GetMinWidth2 — 1 functions, 72 bytes

- `0x120610` GetMinWidth2 (72 bytes)

## FlagDMAChainDone — 1 functions, 72 bytes

- `0x124950` FlagDMAChainDone (72 bytes)

## PlrStats — 2 functions, 72 bytes

- `0x199a70` PlrStats_GetLevelTotals (52 bytes)
- `0x199aa8` PlrStats_GetScoreTable (20 bytes)

## Swap — 1 functions, 72 bytes

- `0x1d3518` Swap (72 bytes)

## isinf — 1 functions, 72 bytes

- `0x23b788` isinf (72 bytes)

## DIntr — 1 functions, 72 bytes

- `0x23fbb8` DIntr (72 bytes)

## CheckSizeAndShrink — 1 functions, 68 bytes

- `0x120928` CheckSizeAndShrink (68 bytes)

## ResetUpgrade — 1 functions, 68 bytes

- `0x1a9900` ResetUpgrade (68 bytes)

## AnimFacialAppend — 1 functions, 68 bytes

- `0x1c6490` AnimFacialAppend (68 bytes)

## mcDelayThread — 1 functions, 68 bytes

- `0x2285a0` mcDelayThread (68 bytes)

## ByteCompare — 1 functions, 64 bytes

- `0x1206c0` ByteCompare (64 bytes)

## AnimGetNDiscreteObj — 1 functions, 64 bytes

- `0x1c66b8` AnimGetNDiscreteObj (64 bytes)

## AnimScriptSetAnimRotation — 1 functions, 64 bytes

- `0x1c7208` AnimScriptSetAnimRotation (64 bytes)

## CompHitData — 1 functions, 64 bytes

- `0x1d33c0` CompHitData (64 bytes)

## fptodp — 1 functions, 64 bytes

- `0x2246d0` fptodp (64 bytes)

## InitTLB — 1 functions, 64 bytes

- `0x23fdf8` InitTLB (64 bytes)

## loadImage — 1 functions, 60 bytes

- `0x107d30` loadImage (60 bytes)

## QueuePeekWriteDone — 1 functions, 60 bytes

- `0x23c760` QueuePeekWriteDone (60 bytes)

## QueuePeekReadDone — 1 functions, 60 bytes

- `0x23c7a0` QueuePeekReadDone (60 bytes)

## isceSifSendCmd — 1 functions, 60 bytes

- `0x23dab0` isceSifSendCmd (60 bytes)

## CompAlphaObj — 1 functions, 56 bytes

- `0x1d3458` CompAlphaObj (56 bytes)

## CompTargetObj — 1 functions, 56 bytes

- `0x1d3490` CompTargetObj (56 bytes)

## Vision — 1 functions, 56 bytes

- `0x1e89a8` Vision_Init_Portal_Recurse (56 bytes)

## isnan — 1 functions, 56 bytes

- `0x23b7d0` isnan (56 bytes)

## kputchar — 1 functions, 56 bytes

- `0x23cc58` kputchar (56 bytes)

## kprintf — 1 functions, 56 bytes

- `0x23d538` kprintf (56 bytes)

## setLoadImageTags — 1 functions, 52 bytes

- `0x107b90` setLoadImageTags (52 bytes)

## setTRXREG — 1 functions, 52 bytes

- `0x107cd0` setTRXREG (52 bytes)

## InitCacheSkin — 1 functions, 52 bytes

- `0x111ad0` InitCacheSkin (52 bytes)

## AnimTagGetUnique — 1 functions, 52 bytes

- `0x1c77c0` AnimTagGetUnique (52 bytes)

## serialPutchar — 1 functions, 52 bytes

- `0x23cd40` serialPutchar (52 bytes)

## setBITBLTBUF — 1 functions, 48 bytes

- `0x107c70` setBITBLTBUF (48 bytes)

## setTRXPOS — 1 functions, 48 bytes

- `0x107ca0` setTRXPOS (48 bytes)

## copysignf — 1 functions, 48 bytes

- `0x235870` copysignf (48 bytes)

## CompBoxList — 1 functions, 44 bytes

- `0x1d3390` CompBoxList (44 bytes)

## CompMsgTxt — 1 functions, 44 bytes

- `0x1d3400` CompMsgTxt (44 bytes)

## setTRXDIR — 1 functions, 40 bytes

- `0x107d08` setTRXDIR (40 bytes)

## UpdateT0Count — 1 functions, 40 bytes

- `0x109468` UpdateT0Count (40 bytes)

## InitShards — 1 functions, 40 bytes

- `0x12bed0` InitShards (40 bytes)

## MPSound — 1 functions, 40 bytes

- `0x18bee0` MPSound_Play (40 bytes)

## AnimObjectIsHeadTracking — 1 functions, 40 bytes

- `0x1c6850` AnimObjectIsHeadTracking (40 bytes)

## AnimObjectIsAiming — 1 functions, 40 bytes

- `0x1c6900` AnimObjectIsAiming (40 bytes)

## CompStrings — 1 functions, 40 bytes

- `0x1d34f0` CompStrings (40 bytes)

## DPRINT — 1 functions, 40 bytes

- `0x226b40` DPRINT (40 bytes)

## Exit — 1 functions, 40 bytes

- `0x23fc28` Exit (40 bytes)

## SaveMemoryMap — 1 functions, 36 bytes

- `0x111298` SaveMemoryMap (36 bytes)

## SetupIRQPointers — 1 functions, 36 bytes

- `0x124998` SetupIRQPointers (36 bytes)

## CompMenuCtrl — 1 functions, 36 bytes

- `0x1d34c8` CompMenuCtrl (36 bytes)

## mcHearAlarm — 1 functions, 36 bytes

- `0x228578` mcHearAlarm (36 bytes)

## CB — 1 functions, 36 bytes

- `0x230b70` CB_DelayTh (36 bytes)

## finitef — 1 functions, 36 bytes

- `0x2355d8` finitef (36 bytes)

## isnanf — 1 functions, 36 bytes

- `0x2356e8` isnanf (36 bytes)

## localeconv — 1 functions, 36 bytes

- `0x23a950` localeconv (36 bytes)

## QueueInit — 1 functions, 36 bytes

- `0x23c738` QueueInit (36 bytes)

## kputs — 1 functions, 36 bytes

- `0x23fd78` kputs (36 bytes)

## behaviour — 1 functions, 32 bytes

- `0x1c7e58` behaviour_util_getStats (32 bytes)

## SFXReset — 1 functions, 28 bytes

- `0x116a18` SFXReset (28 bytes)

## LoaderLoadAnims — 1 functions, 28 bytes

- `0x1d6b48` LoaderLoadAnims (28 bytes)

## Manager — 1 functions, 28 bytes

- `0x21df10` Manager_HandleMessage (28 bytes)

## fabsf — 1 functions, 28 bytes

- `0x2355b8` fabsf (28 bytes)

## index — 1 functions, 28 bytes

- `0x235960` index (28 bytes)

## SetUpIconFile — 1 functions, 24 bytes

- `0x111270` SetUpIconFile (24 bytes)

## EndFreeList — 1 functions, 24 bytes

- `0x1245e8` EndFreeList (24 bytes)

## AddChainFreePtr — 1 functions, 24 bytes

- `0x124928` AddChainFreePtr (24 bytes)

## EIntr — 1 functions, 24 bytes

- `0x23fc00` EIntr (24 bytes)

## audioDecIsPreset — 1 functions, 20 bytes

- `0x1075b0` audioDecIsPreset (20 bytes)

## fstat — 1 functions, 20 bytes

- `0x23c350` fstat (20 bytes)

## SetGsCrt — 1 functions, 16 bytes

- `0x23bc50` SetGsCrt (16 bytes)

## AddIntcHandler — 1 functions, 16 bytes

- `0x23bc90` AddIntcHandler (16 bytes)

## RemoveIntcHandler — 1 functions, 16 bytes

- `0x23bca0` RemoveIntcHandler (16 bytes)

## AddDmacHandler — 1 functions, 16 bytes

- `0x23bcb0` AddDmacHandler (16 bytes)

## AddDmacHandler2 — 1 functions, 16 bytes

- `0x23bcc0` AddDmacHandler2 (16 bytes)

## RemoveDmacHandler — 1 functions, 16 bytes

- `0x23bcd0` RemoveDmacHandler (16 bytes)

## SetAlarm — 1 functions, 16 bytes

- `0x23bd20` SetAlarm (16 bytes)

## DeleteThread — 1 functions, 16 bytes

- `0x23bd50` DeleteThread (16 bytes)

## TerminateThread — 1 functions, 16 bytes

- `0x23bd70` TerminateThread (16 bytes)

## GetThreadId — 1 functions, 16 bytes

- `0x23bdb0` GetThreadId (16 bytes)

## ReferThreadStatus — 1 functions, 16 bytes

- `0x23bdc0` ReferThreadStatus (16 bytes)

## SleepThread — 1 functions, 16 bytes

- `0x23bdd0` SleepThread (16 bytes)

## EndOfHeap — 1 functions, 16 bytes

- `0x23be10` EndOfHeap (16 bytes)

## CreateSema — 1 functions, 16 bytes

- `0x23be20` CreateSema (16 bytes)

## DeleteSema — 1 functions, 16 bytes

- `0x23be30` DeleteSema (16 bytes)

## SignalSema — 1 functions, 16 bytes

- `0x23be40` SignalSema (16 bytes)

## iSignalSema — 1 functions, 16 bytes

- `0x23be50` iSignalSema (16 bytes)

## WaitSema — 1 functions, 16 bytes

- `0x23be60` WaitSema (16 bytes)

## PollSema — 1 functions, 16 bytes

- `0x23be70` PollSema (16 bytes)

## ReferSemaStatus — 1 functions, 16 bytes

- `0x23be90` ReferSemaStatus (16 bytes)

## FlushCache — 1 functions, 16 bytes

- `0x23bf20` FlushCache (16 bytes)

## iFlushCache — 1 functions, 16 bytes

- `0x23bf40` iFlushCache (16 bytes)

## SetVSyncFlag — 1 functions, 16 bytes

- `0x23bf90` SetVSyncFlag (16 bytes)

## isceSifSetDma — 1 functions, 16 bytes

- `0x23bfc0` isceSifSetDma (16 bytes)

## isceSifSetDChain — 1 functions, 16 bytes

- `0x23bfe0` isceSifSetDChain (16 bytes)

## Deci2Call — 1 functions, 16 bytes

- `0x23c020` Deci2Call (16 bytes)

## GetMemorySize — 1 functions, 16 bytes

- `0x23c030` GetMemorySize (16 bytes)

## errorCallback — 1 functions, 8 bytes

- `0x108c00` errorCallback (8 bytes)

## CheckSize — 1 functions, 8 bytes

- `0x120970` CheckSize (8 bytes)

## DAStateFunc — 1 functions, 8 bytes

- `0x13e518` DAStateFunc_AimStand (8 bytes)

## Emitter — 1 functions, 8 bytes

- `0x17b960` Emitter_Kill (8 bytes)

## Mission — 1 functions, 8 bytes

- `0x1830c8` Mission_FailLabel (8 bytes)

## InitRainBox — 1 functions, 8 bytes

- `0x18d838` InitRainBox (8 bytes)

## PlrStarts — 1 functions, 8 bytes

- `0x199b68` PlrStarts_GetRewardsTableSize (8 bytes)

## AnimCacheInit — 1 functions, 8 bytes

- `0x1c6390` AnimCacheInit (8 bytes)

## AnimCacheEnable — 1 functions, 8 bytes

- `0x1c6398` AnimCacheEnable (8 bytes)

## AnimCacheDisable — 1 functions, 8 bytes

- `0x1c63a0` AnimCacheDisable (8 bytes)

## AnimCacheDump — 1 functions, 8 bytes

- `0x1c63b0` AnimCacheDump (8 bytes)

## Discrete — 1 functions, 8 bytes

- `0x1c64d8` Discrete_Delete (8 bytes)

## AnimScriptCallOnEvent — 1 functions, 8 bytes

- `0x1c71f8` AnimScriptCallOnEvent (8 bytes)

## AnimScriptCallOnStop — 1 functions, 8 bytes

- `0x1c7200` AnimScriptCallOnStop (8 bytes)

## AnimDistanceTableInit — 1 functions, 8 bytes

- `0x1c76e8` AnimDistanceTableInit (8 bytes)

## AnimSeqGetHdrBuf — 1 functions, 8 bytes

- `0x1c7998` AnimSeqGetHdrBuf (8 bytes)

## IsMapLoaded — 1 functions, 8 bytes

- `0x1d6be8` IsMapLoaded (8 bytes)

## close — 1 functions, 8 bytes

- `0x23c288` close (8 bytes)

## lseek — 1 functions, 8 bytes

- `0x23c290` lseek (8 bytes)

## isatty — 1 functions, 8 bytes

- `0x23c348` isatty (8 bytes)

## TerminateLibrary — 1 functions, 8 bytes

- `0x23fc18` TerminateLibrary (8 bytes)

