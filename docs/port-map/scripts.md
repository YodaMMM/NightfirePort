# Scripts-family port map

Every `Script_*`, `SP_*`, `Control_*` and `Mission_*` function in ACTION.ELF
(SLUS-20579 NTSC-U) — 97 in total (`Drone_Control` and `DroneSpawner_Control` merely
contain the `_Control_` substring and belong to the drone map). One line each: **ported**
(the port's 1:1 code), **folded** (the behavior is implemented inside other code, or is
obviated by the port's structure so that porting it would be dead code) or **not ported**
(no port code; each such row states why there is no sim-observable behavior to red-test).

Method: the family was enumerated from `work/decomp/ACTION.ELF/index.tsv` (97 files), the
whole decompile cross-referenced for every function's callers (call sites use the mangled
`Name__F` form, data tables the raw address), then each function's decompile was read and
matched against the Rust by semantics — the port rarely cites addresses, so every row
below was verified by reading the implementing code, not by name search. A comment does
not count as ported. Three verification passes were run (one per group) plus a
whole-decompile xref pass; the SP/NIS rows are anchored by `docs/atlas/script-player.md`,
whose field tables the port was built from.

Result: **59 ported, 29 folded, 9 not ported.** The MISSING class — a reached behavior
with no port code that a real-level sim test could red-proof — is empty: all 9 not-ported
rows are either no-ops in the port's object model, MP-only state whose visual is
render-side, or documented deliberate drops. The sweep still found one real adjacent gap
outside the family (the script-created breakables never drain — see the end of this file).

## SP_* — the script-player object (class 0x1B)

Port: `crates/nf-game/src/nis.rs` (`Player` = the `SCRIPTPLAYER_tag`, `Scripts` = the
per-level set). Field tables: `docs/atlas/script-player.md`.

| Address | Name | Status | Port location / proof |
|---|---|---|---|
| 0x1dc338 | SP_Activate | ported | `useobj.rs` activation pass — the mask-4 entity-sphere route with the out-channel gate, cited `[M 0x1dc338]` with tests (useobj.rs:567-577); sets `Player.activated`, consumed as trigger bit 4 (`nis.rs:1844`) |
| 0x1dc7e0 | SP_Create | folded | `Scripts::spawn_copter_sp` (nis.rs:2302, cited `[M 0x1dc7e0]` — the copter's pair SP via a synthetic level record) and `spawn_primed` (2324); the MP create sites (MP_CreateObject / MP_GoldenEyeUpdate) are the `MpObjState` records of `mp/rules.rs` |
| 0x1db9a8 | SP_CreateScriptPlayer | ported | `Player::from_placement` (nis.rs:1605 — the p0-p10 placement param map incl. the p10 scale permille) + the placement walk in `Scripts::new` (2034) |
| 0x1dc8b0 | SP_Delete | ported | `Scripts::delete_player` (nis.rs:2341) / `delete_copter_sp` (2371) — both script slots freed + the delete mark; callers drone.rs:17798-17799, the copter delete route |
| 0x1dc5f8 | SP_GetHitDamage | ported | the bullet pass accumulates `tick_damage` into `SP+0x18` (nis.rs:1947, `add_hit_damage` 2170); the read set (stream objects of every player) at 2051/2107, cited `[M 0x1dc5f8]` |
| 0x1dc900 | SP_GetState | folded | the done-state read lives at its consumers: the per-object done flags over `Script_GetObj` records (gt.rs:272 — states 3/5 and the `+0x9C == 0` arm) and `mp/rules.rs:109` `sp_state` |
| 0x1dcbf0 | SP_HideObj | ported | `Scripts::set_obj_hidden` (nis.rs:2013) / `unhide_sp` (2348); the copter gate's first-ON unhide (`drone.rs:19130`, cited `[M]` at copter.rs:486-487); the view hides the rows (scene.rs:6588 `hidden_stream_objects`) |
| 0x1dc470 | SP_Hit | ported | the trigger-bits pass in `Player::update` (nis.rs:1761-1825 — bit 0x1 out channel, 0x2 class-5 blast with the p8 weapon filter, 0x8 player, 0x10 drone) + `sp_bullet_targets`/`add_bullet_hit` (2113/2160), all cited `[M 0x1dc470]` |
| 0x1dbc08 | SP_LoadScript | ported | `Player::load` (nis.rs:1731) + `prime` (1709) — the isLoadable stream, `Script_Load`/`Set2Start`, and the mode dispatch 1→2 / 8→6 / else→1 |
| 0x1dc958 | SP_RemoveObj | not ported | the kill-on-delete direction (`control_delete_object` → the stream owning the deleted obj) has no port path — recorded at nis.rs:957-961: the port's object rows are `Option` slots, a stream's dead reference no-ops (`objs.list[o] = None`), traced equivalent through op 9, `GetObj`, event prev-obj and interp; no sim-observable difference to red-test |
| 0x1dca90 | SP_SetColour | not ported | the MP uplink terminal tint (only caller `MP_UplinkUpdate`); the port's uplink is state-only (`MpObjState` in mp/rules.rs) — the colour readers are the MP render, unported |
| 0x1dc990 | SP_SetPos | folded | `Scripts::pose_player` (nis.rs:2334; caller drone.rs:17772 — the story hand-back pose) — the game's matrix-write |
| 0x1dc9e8 | SP_SetPosRot | folded | `Scripts::pose_copter_sp` (nis.rs:2360; caller drone.rs:19131 — `Copter_Move`'s re-pose) and the SP-base matrices at spawn |
| 0x1dcad8 | SP_SwitchToScript | ported | `mp/rules.rs:103-109` `sp_script`/`sp_state` model the site SP, cited `[M 0x1dcad8:23-34]`; the state arms at 1065/1158 |
| 0x1dca58 | SP_UnPause | ported | nis.rs:1949-1962 — the activation latch set + `SCRIPTINFO+0x9C = 0`, cited `[M]` |
| 0x1dbda8 | SP_Update | ported | `Player::update` (nis.rs:1765) — the gate/disabled reads, trigger countdown, the 0-7 state machine incl. the 0x6000994/music special case, skip (`Input_Action(-1, 0x19)`) and the stay-flag delete |
| 0x1dcc30 | SP_getScriptInfo | folded | the consumers read the current script identity instead of the pointer: the kick-node SP binding by script hash (drone.rs:1612-1613, `kick_sp`) and the MP SP records (mp/rules.rs) |

## Script_* — the NIS engine (SCRIPTINFO / SSTREAM, 0x1d8-0x1db)

Port: `crates/nf-game/src/nis.rs` (`Script`, the opcode interpreter in `Script::run`, the
per-tick `Script::update`/`Script::interp`); the def parser is `Nis::parse`
(`crates/nf-formats/src/nis.rs:172`). Command set: `docs/atlas/script-player.md` §Script_Run.

| Address | Name | Status | Port location / proof |
|---|---|---|---|
| 0x1d8248 | Script_Load | ported | `Script::load` (nis.rs:394) + `Nis::parse` (nf-formats/src/nis.rs:172 — the 0x1C magic header, word 0-3, per-stream key ranges and the command blob) |
| 0x1d8548 | Script_Run | ported | `Script::run` (nis.rs:576-840) — every opcode 4/5/6/7/9/0xA/0xC/0xD/0xF/0x12/0x13/0x14/0x15/0x16/0x17/0x18/0x1A/0x1B/0x1D/0x1E with its byte-arg footprint, each handler cited by address in the comment |
| 0x1da998 | Script_Update | ported | `Script::update` (nis.rs:495) — the delay skip, per-stream clock advance, the camera-fade bookkeeping and the skip catch-up |
| 0x1db588 | Script_Play | ported | `Script::start` (nis.rs:402 — play bit, loop arm, script_cam latch) |
| 0x1db668 | Script_Stop | ported | `Script::stop` (nis.rs:414) |
| 0x1db630 | Script_IsPlaying | ported | `Script::playing` (nis.rs:421 — the looping/`play&2` condition) |
| 0x1dae60 | Script_IsDeathNIS | ported | `Script::is_death_nis` (nis.rs:424 — the three death hashes) |
| 0x1dac78 | Script_FFwd | ported | `Script::ffwd` (nis.rs:458) — the fade arm, the no-camera kill arm and the catch-up re-run; tests `skip_fast_forwards_a_camera_script_but_not_a_death_nis` (nis.rs:2774) + `nf-view/tests/dev_tools_live.rs` |
| 0x1d94c0 | Script_EventHandler | ported | `Script::event` (nis.rs:1036-1199) — every event id 3-0x13 incl. the prev-stream-object rule, cited `[M 0x1d94c0:...]`; tests for events 0xB/9/0xD/0x11 |
| 0x1d8d78 | Script_EntityStart | ported | `Script::entity_start` (nis.rs:844) — the owner wire, interp tail, sphere build and room link |
| 0x1dafe8 | Script_EntityEnd | ported | the op-9 arm (nis.rs:620-625 — the clock/looping/tag-1 condition) |
| 0x1d8968 | Script_CreateEntity | ported | folded into `entity_start` — the class-differs delete-and-recreate rule (nis.rs:846-854) and the 0xF0 pickup / 0xF5 AI-volume / 0x20 break creator arms (861-907); caveat: the 0x20 arm's `pending_breaks` consumer is not wired yet (see the gap note at the end) |
| 0x1d8f40 | Script_AnimStart | ported | `Script::anim_start` (nis.rs:934) — the root-translation seed and the dead-GetInterp note, cited from the disasm |
| 0x1db328 | Script_AnimEnd | ported | the op-0xC arm → `kill` (nis.rs:631, cited `[M 0x1db328:5-9]` tail-call) |
| 0x1d9140 | Script_CameraStart | ported | the op-0xD arm (nis.rs:685-726) — CAM_INFO object, key range, the 0x20 skip, letterbox/music/fade flags and the identity-base rule |
| 0x1db348 | Script_CameraEnd | ported | the op-0xF arm → `kill` + the camera-active flag clear (nis.rs:727-732) |
| 0x1d9300 | Script_SpriteStart | ported | the op-0x14 arm (nis.rs:742-767) — the 640×480→512×448 scale, layer and flag bits |
| 0x1db3b0 | Script_SpriteEnd | ported | the op-0x15 arm → `kill` case 3 (nis.rs:768-772) |
| 0x1d8820 | Script_SoundStart | ported | the op-0x16 arm (nis.rs:638-667) — the handle clear, start frame, 0x20 gate and the 2-D rule |
| 0x1daf60 | Script_SoundEnd | ported | the op-0x17 arm (nis.rs:668-684) — the live-looping-only stop via `World::sfx_looped` |
| 0x1db1b0 | Script_LightStart | ported | the op-0x18 arm (nis.rs:773-786) — the parent pair and light kind |
| 0x1db298 | Script_LightEnd | ported | the op-0x1A arm → `kill` case 6 (nis.rs:787-792) |
| 0x1db050 | Script_SubScriptStart | ported | the op-0x1B arm (nis.rs:793-815) — the nested `Script::load` + looping start + the trailing interp |
| 0x1db148 | Script_SubScriptEnd | ported | the op-0x1D arm (nis.rs:816-824) — the loop-boundary persistence condition |
| 0x1db2b8 | Script_TextStart | ported | the op-0x1E arm → `Request::Text` (nis.rs:825-832) |
| 0x1db3d0 | Script_FadeStart | ported | the op-0x13 arm → `Request::Fade` (nis.rs:733-741, the skip-latch gate cited) |
| 0x1db420 | Script_StreamEnd | ported | the op-4 arm (nis.rs:594-612) — the pc rewind hold, the loop restart (loop_start, stream clock −1) and the stop+done bits |
| 0x1d9860 | Script_KillStream | ported | `Script::kill` (nis.rs:962-997) — all seven kind arms incl. the keep-objects flag and the camera pop |
| 0x1da880 | Script_Free | ported | `Script::free` (nis.rs:1000-1030) — the stream pass, the 0x80 `Drone_EnableAll` tail and the 0x100 `Player_Enable` tail with the enable-matrix provenance |
| 0x1daec0 | Script_FreeFromHashTable | folded | the port's NIS lib is a resident `HashMap<u32, Arc<Nis>>` for the level's Sim; the hashtable delete+free has no observable effect (the reload path is the next Sim's load) |
| 0x1da158 | Script_GetInterp | folded | the key interpolation/compose lives in `Script::interp` + `keys()` (nis.rs:66, 697, 919, 1231, 1249 — the base-concat flag rule cited from the disasm) |
| 0x1db688 | Script_GetObj | folded | the kind-1/2 handle filter at the read sites (nis.rs:1059, 2053, 2107; gt.rs:272) |
| 0x1db808 | Script_HideObj | ported | `Scripts::set_obj_hidden` (nis.rs:2013) + `hidden_stream_objects` (2382) — the view hides the rows (scene.rs:6588+) |
| 0x1daea8 | Script_Init | folded | the per-level Sim rebuild is the global reset; `Script::load` zeroes the script state (`script_cam: false`, nis.rs:214/397 = `ScriptCam` 0x30d100 = 0) |
| 0x1d9a08 | Script_Interp | ported | `Script::interp` (nis.rs:1199-1438) — all seven kind arms incl. the camera slerp and the skip freeze |
| 0x1db6e0 | Script_RemoveObj | not ported | same as SP_RemoveObj — the kill-the-stream-owning-this-obj direction is a structural no-op in the port's `Option`-row object model (nis.rs:957-961); traced equivalent, no sim-observable difference |
| 0x1daf00 | Script_Set2Start | ported | `Script::rewind` (nis.rs:416-419 — clock 0, pc = cmd base, done bit cleared) |
| 0x1db770 | Script_SetColour | not ported | walks the streams and tints the objects; only caller is `SP_SetColour` (MP uplink) — no port code, MP-render observable |
| 0x1db920 | Script_SetPosRot | folded | the base-matrix write is `Script::load`'s matrix + the pose fold (`pose_player`/`pose_copter_sp`/`spawn_once`), and the delayed re-interp is `Player::prime`'s interp tail |
| 0x1db518 | Script_UpdateCamera | folded | the camera arm of `Script::interp` posts the matrix (nis.rs `camera_stream_inputs` 2409 / `camera` 2402) and the scene applies it — the game's copy into the active camera struct |

## Script_* — the menu script engine (M_CONTROL / M_SCRIPT / M_KEYFRAME, 0x22x)

Port: `MenuRuntime` in `crates/nf-formats/src/menudef.rs` (a headless data+sim layer,
pinned by `crates/nf-formats/tests/menudef_script_runtime.rs` and `menudef_disc.rs`).
Integration note: the playable front end (`nf-view/src/menu.rs`) renders final page
states and does not tick `MenuRuntime` yet — the engine is ported and verified in
isolation (the stale "unported" audit note at menu.rs:6733-6736 refers to the wiring).

| Address | Name | Status | Port location / proof |
|---|---|---|---|
| 0x2201c8 | Script_AddKeyFrame | folded | `MenuDef::parse`'s KFRAME arm + parse-end time sort (menudef.rs:251-286, 376-390) — the sorted-insert/dedup rule cited `[M 0x2201c8:24-38]`, test-pinned |
| 0x221198 | Script_AddMessage | folded | the KMSG arm — dropped when no keyframe is current, as in the game (menudef.rs:288-301) |
| 0x220fb0 | Script_AddScript | folded | the SCRIPT arm — find-or-create + make current (menudef.rs:201-216) |
| 0x2214c8 | Script_CalculateLine | folded | `MenuRuntime::run_frame`'s else branch — `p1 + (p2-p1)·frac` (menudef.rs:792) |
| 0x221410 | Script_CalculateSpline | folded | the Catmull-Rom branch, numerically test-pinned (menudef.rs:788-790; menudef_script_runtime.rs:51-77) |
| 0x220f30 | Script_DelAllScripts | folded | Vec ownership — the scripts die with the page (menudef.rs:455-463, 620-628); the game's destroy chain has no runtime entry point in the port because nothing destroys pages mid-run |
| 0x221100 | Script_DelKeyFrame | folded | nested-Vec ownership (KeyframeDef inside ScriptDef, menudef.rs:459-460) |
| 0x221210 | Script_DelMessage | folded | nested-Vec ownership (KfMsg, menudef.rs:460-461) |
| 0x221068 | Script_DelScript | folded | ScriptDef dropped with the ControlDef (menudef.rs:456-458) |
| 0x2213d0 | Script_ExitLoop | ported | `MenuRuntime::exit_loop` + control-msg arm 0x14 (menudef.rs:684-686, 939) — the `+0x58 &= ~0x8` bit pinned by tests |
| 0x220bc8 | Script_InterpolateLine | folded | `run_frame`'s line path — clamp, lerp, the trunc-cast rect write (menudef.rs:768-797) |
| 0x2208b8 | Script_InterpolateSpline | folded | `run_frame`'s spline path incl. the i=0 wrap to the last keyframe (menudef.rs:774-790) |
| 0x2212a8 | Script_Play | ported | `MenuRuntime::play_script`/`start_script` (menudef.rs:671-679, 948-964) — the mode-0 arm; the game's second arm (param_3 ≠ 0, start from the +0xC list) is unmodeled (no shipped def census shows it used) |
| 0x220350 | Script_PlayDefault | ported | `play_default` via page_shown/focus_msg (menudef.rs:658-735) — the slot map and the type-0xD window skip, test-pinned |
| 0x220ed8 | Script_PrepareScripting | ported | `MenuRuntime::new` (menudef.rs:629-649) — the per-control state (idx −1, timer 0, flags) mirroring the +0x28..+0x58 offsets |
| 0x220580 | Script_RunFrame | ported | `MenuRuntime::tick`/`run_frame` (+ fire_msgs/deliver/run_procs, menudef.rs:737-988) — the interpolation, loop markers, message dispatch (manager/self/by-id, the −4 last-result, the 0xE1 delayed process) |
| 0x221250 | Script_Select | ported | `MenuRuntime::select_script` (menudef.rs:693-701) — position-by-id without resetting keyframes |
| 0x221400 | Script_Stop | ported | `MenuRuntime::stop_script` (menudef.rs:688-691) — the current-script clear |

## Control_* — the object-control helpers

The port does not model the generic `obj_tag`/DynamicObjList layer at all:
`docs/atlas/object.md` maps obj_tag onto per-system records, and the shared computations
live as helpers. The rows below cite where each behavior lives.

| Address | Name | Status | Port location / proof |
|---|---|---|---|
| 0x133070 | Control_BuildWorldSph | ported | `RbObj::world_sphere` (rigid_body.rs:976 — the glist centre through the live matrix rows + radius), with the branch-complete folds `UseObjects::set_pc_spheres` (useobj.rs:201) and `Scripts::held_hit_targets` (nis.rs:2062) |
| 0x1343c8 | Control_CreateObjEx | folded | `Script::entity_start`'s `Obj::Entity` push (nis.rs:850-854) for script objects; every other caller is a typed constructor (Car::create, level_drones, GT, missiles…) — there is no generic allocator because the port re-derives membership per tick (`World::player_cel_relink`, world.rs:1914) |
| 0x1349e8 | Control_DeleteAllObjectsOfType | not ported | only caller `MP_RestartScenario`; recorded deliberately at `MpRules::restart_scenario` (mp/rules.rs:672-685): the port's MP runtime carries no raw classes 5/0x35/0x10/0xE, so the sweeps have no target — the restart re-inits the typed state instead |
| 0x134210 | Control_GetObjMatrix | folded | the anim case re-poses `Obj::Anim.matrix` per tick (nis.rs:1349 + drone.rs `script_parts`); create-time matrices come from `view_rot_trans_matrix` (nf-formats/src/placement.rs:206) — the flags2-conditional arms have no shared helper |
| 0x133e88 | Control_InheritVelocity | folded | `World::ground_motion` (world.rs:1303) + its appliers (scene.rs:3184 ride, drone.rs:6163 feet-on-point, drone.rs:21016 satchel) — the mover's delta carry; the bullet/car collision call sites have no ported equivalent (the port's bullets compute their own motion) |
| 0x1349b0 | Control_NextLOD | folded | the searchlight sim pins `Part::lod_next` (searchlight.rs:106, set at 298/620) and the renderer draws the `lod_target` glist; the other callers (PCQWorm, Lock, FuseBox, Monitor) are no-ops on disc data, documented as such |
| 0x134a90 | Control_ObjectReferenced | folded | obviated: the port's objects are owned indices that cannot dangle (`Explosion::attacker: usize`, explode.rs:44), documented at explode.rs:7 / copter.rs:310 / drone.rs:20390 — porting the flags2-0x400 pin would be dead code |
| 0x134118 | Control_Plr2Ind | folded | obviated: all MP/bot state is slot-indexed (`MpBot::slot`, `BotCtx::my_index`, mp/bots.rs:764) — there are no obj_tag pointers to map (doc mentions only: bots.rs:736, rules.rs:1586/1610) |
| 0x134a38 | Control_ReturnNextObjectOfType | not ported | only caller `MP_RestartScenario`; same recorded drop as DeleteAllObjectsOfType (mp/rules.rs:674-679) — the class walks have no target in the port's model |
| 0x134940 | Control_SetEffectType | not ported | `Trigger_Update`'s per-tick effect word, recorded at `Trigger::glist_0x10` (triggers.rs:130-135): the port carries no per-trigger effect word; its readers are render-side (unported), so no sim test can observe it |
| 0x134960 | Control_SetGList | folded | the create-time copy folds into each typed creator (`DynObject` base_flags/flags, worldobj.rs:496-497; `Door::glist` door.rs:224) and `Destroy_Smash`'s glist swap is the sim's dead-mark + the renderer drawing the destroyed slot (props.rs:278, scene.rs:7170-7188) |

## Mission_* — the mission/objective state machine

Port: `crates/nf-game/src/mission.rs` (`Mission` — the BSS globals + the per-record
states over the static `LEVELS` table). Side effects funnel through `mission::Event`
into the view's banners/menus.

| Address | Name | Status | Port location / proof |
|---|---|---|---|
| 0x182270 | Mission_Init | ported | `Mission::init` — latched at the load site (drone.rs:17927, the `ResetMap_GameInit` equivalent); BaseMap/order from the 24-record LEVELS table, objective/condition reset, warm-vs-cold handling, `PlrStat_ResetForMission` folded beside |
| 0x182c60 | Mission_Update | ported | `Mission::update` (called from the `Game_Run` port, drone.rs:17963/13381) — the 1→2/3/6→7→4/5 machine, the fail/death/complete arms with the per-level `LevelToEndTo` table, the timeout and the 0x40000042 fail menu |
| 0x1827d8 | Mission_MonitorObjectives | ported | `Mission::monitor` (mission.rs, from update's case 1) — the flags&2 inverted-AND conditions, the announce state machine and the standing channel hooks |
| 0x182630 | Mission_ObjectiveState | ported | `Mission::objective_state` over the shared `vis_walk` (the part gate and the state>1 filter); the state-keyed label slot is consciously omitted (documented, no reader) |
| 0x182540 | Mission_NumVisObjectives | ported | `Mission::num_vis_objectives` = `vis_walk().count()` |
| 0x1830d8 | Mission_SetStatus | ported | `Mission::set_state` (mission.rs:318) — both game callers wired (`init`, and the `Boot_LoadPTPData` mirror `DriveReturn::apply` at ptp.rs:241) |
| 0x1830d0 | Mission_Status | ported | `Mission::status` (mission.rs:321) — read at every ported `PlrStat_*` site and the stat gate (drone.rs:14119/15826/18245/21770) |
| 0x183098 | Mission_SetFailLabel | ported | `Mission::set_fail_label` (mission.rs:324) — written by the `DroneFunc_SetMissionFailReason` port (`ally::apply_fail_reason`, ally.rs:42-45) and the monitor's hooks |
| 0x1830c8 | Mission_FailLabel | ported | the `fail_label` field (mission.rs:151-159) → `Event::FailLabel` → the HUD banner's second line (scene.rs:6339, hud.rs:1277-1287) |
| 0x1830c0 | Mission_SetMapHCode | folded | `Mission.base_map` is the modeled global (written by `init`); the `Boot_LoadPTPData` setter call itself is skipped — the driving return is a cold hub boot, `PtpBlock.map_id` is built but unconsumed (ptp.rs:240-244, 311, cited) |
| 0x1830a0 | Mission_BaseMapHCode | folded | `Mission.base_map` read at each call site — `finish_level`/`get_score` (drone.rs:21815), pause/results (lib.rs:4233/6417), the gold-medal gate re-derivation (`DroneContext::has_gold_medal`, scene.rs:385); documented as the port of this getter at progress.rs:422-424 |

## The sweep's findings

1. **No in-family MISSING with a sim-observable behavior.** The 9 not-ported rows are:
   SP_RemoveObj / Script_RemoveObj (structural no-op in the port's `Option`-row object
   model — traced equivalent through every reader), SP_SetColour / Script_SetColour (MP
   uplink tint, visual-only), Control_SetEffectType (render-side readers), and
   Control_DeleteAllObjectsOfType / Control_ReturnNextObjectOfType / Control_Plr2Ind /
   Control_ObjectReferenced (MP restart sweeps and pointer-map helpers that the port's
   slot-indexed, owned model obviates; two are recorded as deliberate drops in
   mp/rules.rs:672-685). None of them can produce a red real-level sim test, because
   there is no sim-side observable that differs.
2. **Adjacent gap found outside the family — script-created breakables never land.**
   `Script_CreateEntity`'s 0x20 arm pushes `Objects::pending_breaks` (nis.rs:902, the
   07000008 gondola-cabin-window route: 16 panes of ride script 0x0600004C) but nothing
   drains it — `pending_pickups` drains at drone.rs:20790 and `pending_volumes` at
   scene.rs:5811, `pending_breaks` nowhere. The comment at nis.rs:893-898 claims "the sim
   drains into `Breaks`" — no such drain exists. Effect: script-created panes render (the
   `Obj::Entity` row) but never get `Breaks` records, so they cannot be shot/smashed and
   their bounds are not tracked. The fix belongs to the breakables family (the
   `Break_Create` port), not here — proposed next round: drain `pending_breaks` into
   `Breaks` at the Sim latch with a 07000008 real-level red test (shoot a gondola pane).
3. **Menu runtime is ported but not wired** — `MenuRuntime` has no construction site in
   `nf-view`; the front end renders final page states. Either integrate (page_shown +
   tick) or replace the stale audit note at menu.rs:6733-6736 with the decision.
