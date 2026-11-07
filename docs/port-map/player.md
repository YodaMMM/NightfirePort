# Player-family port map

Every decompile function whose symbol begins `Player` in ACTION.ELF (SLUS-20579 NTSC-U)
— 90 in total: the 85 `Player_*` functions plus the 5 `PlayerAnim*` anim-set functions of
the same family. The `*Player*`-substring symbols of other families are out of scope
(`Car_PlayerHasDied`, `GT_PlayerHasDied`, `MP_PlayerKilled`, `MP_playerIsDead`,
`MP_PlayerOrBotInd`, `NDrone2_Player`, `PlrStat_LogEnemyDetectedPlayer`,
`Trigger_MoviePlayer`, `Camera_SetToPlayer`, `Effect_Player`, `SP_CreateScriptPlayer`,
`SP_LoadScript`, `SP_Hit`, `P_FMVPLAYER_Handler`, `P_MPPLAYERMODS_Handler`,
`C_GOMULTIPLAYER_Handler`, `Menu_PlayerHasQuitOrRestarted`).

One line each. Statuses: **ported** = the port's code for it, at the cited location (the
port cites the decompile address on the implementing function, or the name where the fold
is named); **absorbed** = the disc dispatches it and the port carries the observable
behavior with no separate site (the reason says why nothing is missing); **not reached** =
the shipped disc never dispatches it on a playable path, with the proof.

Method: for each decompile file `work/decomp/ACTION.ELF/<addr>_<name>.c` the exact address
was searched in `crates/*/src`, then the function's base name; a citation counts only when
it is attached to the implementing function (a test-only citation does not — each such row
was resolved by hand against the .c). Seven rows carry `—` in the address column: their hex
contains a function-key character pair and the push guard trips on it; every address is
also readable off the row's `.c` filename in `work/decomp/ACTION.ELF/`.

## The one MISSING the sweep found (fixed on this branch)

The single-use key ring. `Player_EquipKey` 0x1a97a0 was ported (the pickup side,
`Inventory::keys`), but its mirror `Player_UnEquipKey` 0x1a97b8 — `BL[0x70C + id] = 0`
[M 0x1a97b8:6] — had no port code, and neither half of the game's keypad law was wired:

- `Lock_Unlock` 0x1b0650's state-5 arm (the decryptor solve) equips the lock's key:
  `Player_EquipKey(BL, record u16[+2])` [M 0x1b0650];
- the case-1 tick holds the out channel on [M `Lock_Update` 0x1ae3d8:193-196] and its
  framelist tail leaves the lock in state 2 [M :199-220];
- from state 2, the case-2 tick re-arms a kind-0 lock the moment the out channel reads 0,
  through `Lock_ReLock` 0x1b0750 — the state word back to 0 and
  `Player_UnEquipKey(BL, u16[+2])` [M 0x1b0750:5-10]: the key leaves the ring.

The port solved a keypad into a permanent `unlocked`, equipped nothing, and never re-armed.
Fixed in `crates/nf-game/src/lock.rs` (the `open_done` state-2 phase flag +
`Lock::relock_if_dropped`, the case-2 arm) and `crates/nf-view/src/scene.rs` (the top-of-tick
lock pass: the solve's equip, case 1's hold + phase flip, the re-lock arm clearing the ring
slot), tested red-then-green on the real level through the real aim/fire path by
`crates/nf-view/tests/decryptor_07000012_live.rs::the_solved_keypad_equips_its_key_and_a_dropped_channel_relocks_the_lock`
(07000012's kind-0 lock #656, code index 5, out channel 10 [D census]): the pre-fix run
fails at the equip assert, the post-fix run passes with the pre-existing solve-path test
still green. Every lock-adjacent binary re-run green whole: `decryptor_07000012_live` 2/2,
`decryptor_hold_real_path_live` 3/3, `keypad_unlock_lod_live` 1/1, `gadget_laser_live` 8/8,
the `nf-game` lib tests 686/686/5i.

## The second MISSING (reached, no port code — render-side, not fixed here)

`Player_GrappleSetRope` — called from `Player_CollisionHandler` (0x1949a8:203) on every
grapple grab, SP and MP. It toggles the grapple-point object's `+0xF0` flags (0x10, 0x8000),
sets `+0xFA |= 0x20`, and copies the rope mesh object's transform from the grapple record
[M 0x192728]. The port's `GrappleState` has no rope object at all — `Player_InitGrapple`'s
`Control_CreateObjEx` rope mesh and `+0x158` slot are recorded as not-ported render fields
(gadget.rs:435-437, docs/systems/bldata.md:56/59). The sim-side behavior (hang, reel,
release) is ported; the missing piece is purely the visible rope.

## Live checks for the main agent

- On any level with a grapple point (e.g. the mountain route), `--bond` at the grab: the
  rope mesh should appear anchored at the grapple point and follow the hang/reel phases;
  today nothing renders. The game objects are created at `Player_InitGrapple` (0x195d10's
  `Control_CreateObjEx(1.0, hashtable(0x02000138), parent = player, mode 5)`) and positioned
  by `Player_GrappleSetRope` (0x192728: the point object's `+0xF0 |= 0x10 | 0x8000`,
  `+0xFA |= 0x20`, the transform copy from `+0x144..0x14C`).
- On 07000012 after a keypad solve, walk away until the door behind closes: the lock should
  re-arm (the keypad's LOD variant stays per `Control_NextLOD`) and the key ring slot
  (BL+0x70C+5) should read 0 [M 0x1b0750:8]. A gdbwatch on BL+0x70C+5 through the
  solve→close sequence would confirm the unequip live.

## Table

| Address | Name | Status | Port location / proof |
|---|---|---|---|
| 0x18fcd8 | PlayerAnimSetInitNormal | ported | `nf-formats/src/mpanim.rs:122` (`anim_set_index`) — the selector→descriptor switch incl. the SP-999 arm; consumed via `MpAnimSets::set` (mpanim.rs:215) |
| 0x18fea0 | PlayerAnimSetInitCrouch | ported | `nf-formats/src/mpanim.rs:122` — same switch, crouch = normal+1 at :133 (the decompile's +0x28 offsets) |
| 0x190068 | PlayerAnimCrouch2Stand | ported | `nf-formats/src/mpanim.rs:160` (`crouch2stand_script`) — per-class script table; appended in `MpBodyAnim::tick` (mp/body_anim.rs:284) |
| 0x190198 | PlayerAnimStand2Crouch | ported | `nf-formats/src/mpanim.rs:145` (`stand2crouch_script`) — per-class table incl. the SP class-3 pick |
| 0x1902c8 | Player_HandlePain | ported | `nf-game/src/bullet.rs:347` (`PlayerHealth::apply`) — difficulty factor, armour-first absorb, <1 kills, grunt roll; MP arm mp/combat.rs:291 |
| 0x190630 | Player_ChangeSubState | ported | `nf-game/src/player.rs:1182` (`Bond::change_substate`) — the loop-stop trio 0x12/0x3d/0xf0 at :83-85; the anim-set cases in body_anim/mpanim |
| 0x190810 | Player_CheckForDeath | ported | `nf-game/src/death.rs:216` (`Drones::death_tick`) — the death-start block :223-242 (ch 0x5e, Music_Event(6,1), WeaponNone fists, sound 0x89) |
| 0x190bb0 | Player_HandleDeath | ported | `nf-game/src/death.rs:216` (`Drones::death_tick`) — the per-dead-tick block :244-260 (blood pane, 448-tick cover, ✕ skip, ch 0x62 once) |
| 0x190d88 | Player_StandAtNewPosition | ported | `nf-game/src/player.rs:627` (`Bond::stand_at`) — the floor ray + 1.1 placement; the aim-ease tail (:122) not modeled |
| 0x190fd0 | Player_Start | ported | `nf-game/src/player.rs:2392` (`pick_start`) — the two-pass kind-1 gate scan verbatim; consumed at load_impl (nf-view lib.rs:1554) |
| 0x1910f8 | Player_Init | ported | `nf-view/src/lib.rs:1557` (`load_impl`) — stand_at, settle, WldGravity p3==2, substate 8/4 + crouch drop; loadout via `init_level_loadout` (player.rs:2468) |
| 0x1916b0 | Player_HandleJump | ported | `nf-game/src/player.rs:1689` (`physics_tick` jump block) — crouch toggle, context-3 wire grab, launch vel + 0xA3 (:1727-1732); the landing arm :2094-2107 |
| 0x191a70 | Player_MonitorAir | ported | `nf-game/src/player.rs:1600` (`Bond::monitor_air`) — air drain/refill, underwater fade, drown damage each 30 ticks; the HUD air pane/breath loop not ported |
| 0x191dd0 | Player_Move | ported | `nf-game/src/player.rs:1668` (`physics_tick` ease lanes) — target lanes, 0.2 ease, ±0.1 clamp, dead-zone, multiplier k; the swim look-dir add in swim_tick |
| 0x192310 | Player_SSWalk | ported | `nf-game/src/player.rs:777` (`Bond::walk_tick`) — the water-check entry + physics + ladder grab; jump/crouch handled in physics_tick |
| 0x1924c0 | Player_SSCrouch | ported | `nf-game/src/player.rs:777` (`Bond::walk_tick`) — the shared walk body; the 0.75 multiplier at physics_tick:1668, stand rules :1701-1705 |
| 0x192728 | Player_GrappleSetRope | ported (state half) | the sim-side hang/reel/release is `gadget.rs:492` (`grapple_tick`) via `Bond::grapple_tick` (player.rs:798); the ROPE-OBJECT half (the point object's +0xF0/+0xFA flags and the transform copy) has no port code — render-side, see the MISSING section above |
| 0x192860 | Player_SSGrapple | ported | `nf-game/src/gadget.rs:492` (`grapple_tick`) via `Bond::grapple_tick` (player.rs:798) — reel/hang/release phases, the 1.5 SP pull cap, release-drops-without-momentum |
| 0x192ab0 | Player_ZeroG | ported | `nf-game/src/player.rs:767` (`Bond::zerog_tick`) — dispatch cases 8/9; the body in the physics_tick zero-g arm (player.rs:1745+), thrusters, stabiliser, air refill |
| 0x193098 | Player_SSScanMode | not reached | dispatch case 5 of Player_Update (0x193600:228) is entered only by the debug-fly cheat (`C_CHCHFLY_Handler` 0x215c70 — the cheat menu's fly code), not by normal play; the port routes substate 5 to the walk default (menu.rs:2905 records the cheat entry) |
| 0x193498 | Player_InShadow | ported | `nf-game/src/shadow.rs:36` (`in_shadow`) — the u16-wrap light accumulator, the shade-4 halve; pure fn over the same inputs (`in_shadow_from_list` :65) |
| 0x193600 | Player_Update | ported | `nf-game/src/player.rs:728` (`Bond::tick_ctx`) — the regrab decay + substate dispatch + MonitorAir; the outer arms (dead branch, music zone, hit_push) in `Scene::step_bond` (scene.rs:2588) |
| 0x194188 | Player_FeetOnPoint | ported | `nf-game/src/player.rs:1932` (`physics_tick` floor ray) — the B-arm ray to −(cc+0.6), the hit−0.1≤cc band, grounded = walkable‖band; the climb variant physics_tick_climb:1141 |
| 0x194458 | Player_DealWithObjHit | ported | `nf-game/src/drone.rs:21116` (`Drones::tick_inner`) — the bullet arm (dir bits, 0x412, melee hit-push, pain_from); 0x27 hurt.rs:158, 0x2C grab_wire, 0x2D ladder_grab (player.rs:884) |
| 0x1949a8 | Player_CollisionHandler | ported | `nf-game/src/player.rs:2075` (`physics_tick`) — the fall timer/damage + the void-fall ch 0x7C arm; the +0x95F ctx reset scene.rs:3333; hit dispatch via the grab fns after tick |
| 0x194df0 | Player_Collision | ported | `nf-game/src/player.rs:706` (`Bond::capsule`) — the substate capsule arms; the collide pass physics_tick:1905; the HITTEST body mirror scene.rs:4178 |
| 0x195498 | Player_Enable | ported | `nf-view/src/scene.rs:3836` (`Sim::step_bond` PlayerEnable arm) — re-place at the matrix + re-enable + timer unpause; no spherical-velocity handoff (noted player.rs:619-621) |
| 0x195670 | Player_RamSave | ported | `nf-game/src/player.rs:2238` (`RamSave::capture`) — health/armour/inventory/channels snapshot; the mission tally via with_tally:2267 |
| 0x1957a8 | Player_RamLoad | ported | `nf-game/src/player.rs:2309` (`RamSave::restore`) — the warm-arm restore of health/armour/weapon/channels; the boost arm continue_boost:2288 |
| 0x195928 | Player_SetCamMode | ported | `nf-game/src/player.rs:3308` (`Bond::set_cam_mode_orbit`) — the case-1 orbit init (OrbitCam 1.5/0.15/1.5/0); other modes = scattered cam_mode writes; the viewer+0x236 clear as the goggles reset scene.rs:3811 |
| 0x195a88 | Player_ResetWalkAnim | absorbed | its sole effect is the 16-byte write to BL+0x954, which no decompile code ever reads (write-only [M]) — unobservable; the port documents the no-op at gun.rs:5 |
| 0x195aa0 | Player_GetHeadPos | ported | `nf-game/src/player.rs:643` (`Bond::head`) — the common arm full (y = pos+eye−crouch_drop); the crawl/wire matrix arm lives in the view's creep/wire cameras |
| 0x195b90 | PlayerAnimSetInit | ported | `nf-game/src/mp/body_anim.rs:194` (`MpBodyAnim::rebuild`) — the set rebuild (AnimSetDeleteAll/reset); the crouch pick (obj+0xf6==4) is `MpAnimSets::set` (mpanim.rs:215) |
| 0x195be8 | Player_ChangeState | ported | `nf-view/src/scene.rs:14839` (`Sim::step_net_combat`) — the trivial obj+0xf4 setter = Death.active (death.rs:91); the sole caller MP_ReSpawn(0) clears it at respawn (scene.rs:13311) |
| 0x195bf0 | Player_ResetStartPos | ported | `nf-game/src/player.rs:2367` (`starts_from_placements`) — both globals reset implicit in the per-load Vec rebuild; documented player.rs:2362-2366 |
| 0x195c00 | Player_AddNewStartPos | ported | `nf-game/src/player.rs:2367` (`starts_from_placements`) — appends one LevelStart per type-0x24/0x2D placement; the kind field = the u16 arg |
| 0x195d10 | Player_InitGrapple | ported | `nf-game/src/gadget.rs:439` (`GrappleState`) — the zeroed BL+0x140 block; the rope mesh object render-side, not ported (see GrappleSetRope) |
| 0x195da0 | Player_SetGrapplePoint | ported | `nf-view/src/scene.rs:3226` (`Sim::step_bond`) — the latch: point, phase 1, pull 0, ChangeSubState(2); the remote arm scene.rs:14116 |
| 0x195e08 | Player_ClearInertia | ported | `nf-game/src/player.rs:1168` (`Bond::clear_inertia`) — zeroes vel/local/push_fwd (the +0x10/+0x20/+0x30/+0x40 triples) |
| 0x195e50 | Player_InWater | ported | `nf-game/src/player.rs:1574` (`Bond::enter_water`) — the swim entry arm: ChangeSubState(3), deltas cleared, regrab 5; the MP skip and the +0x100 flag not modeled |
| 0x195f70 | Player_Disable | ported | `nf-view/src/scene.rs:3797` (`Sim::step_bond`) — Request::PlayerDisable → self.disabled + timer-pause + goggles clear; obj+0xF0 bit's later life not ported |
| 0x195fc8 | Player_Cam2Mode | ported | `nf-view/src/scene.rs:3823` (`Sim::step_bond`) — Request::Cam2Mode: mode 3 → cam_mode 2; else cam_mode 0 + ChangeSubState(0) |
| 0x196040 | Player_SetFlashBang | ported | `nf-game/src/drone.rs:21478` (`Drones::tick`) — the flash writes BL+0x963/0x918/0x91C as flash_alpha/flashbang/flash_total; the Apocalypse re-call world_fx.rs:859 |
| 0x196060 | Player_Hurt | ported | `nf-game/src/drone.rs:14327` (`Drones::spawn_blast`) — blast → PropagateHits.player → pain_generic (HandlePain type 0) at 14416; the MP blast route mp_hurt/combat.rs:193 |
| 0x196098 | Player_Kill | ported | `nf-view/src/scene.rs:13472` (`Sim::step_mp_scenario_sync`) — SetHealth(0) on the GoldenEye-strike target; the sole game caller is MP_GoldenEyeUpdate 0x187568 (MP-only reach) |
| 0x1960b8 | Player_SetHealth | ported | `nf-game/src/player.rs:2309` (`RamSave::restore`) — BL+0x894 = PlayerHealth.health (RamLoad; respawn writes scene.rs:13310); the PlrStat_LogHealth snapshot drone.rs:21798 |
| 0x196118 | Player_CollLadder | ported | `nf-game/src/player.rs:884` (`Bond::ladder_grab`) — the guards, the angle π/2 gate, the y-window, the snap −sin·0.5, zero vel, ChangeSubState(1) |
| 0x196310 | Player_Climb | ported | `nf-game/src/player.rs:975` (`Bond::climb_tick`) — the yaw clamp/ease, the dismount hop −0.05, the top lunge +0.6, the bottom exit, regrab 60 |
| 0x196758 | Player_CollCreepWall | ported | `nf-game/src/player.rs:1420` (`Bond::start_creep`) — the entry: axis snap, floor ray +1.0, clear inertia, ChangeSubState(7), cam 0xB |
| 0x1969d0 | Player_Creep | ported | `nf-game/src/player.rs:1454` (`Bond::creep_tick`) — the crawl tick: snap 0.275, end exits with push-out; the anim-root motion replaced by constant speed (documented) |
| 0x197130 | Player_CollWire | ported | `nf-game/src/player.rs:1203` (`Bond::grab_wire`) — the grab guards, the closest point, substate 6/0xF, yaw ±π/2, hang offsets, the hand-reach correction |
| 0x197658 | Player_Wire | ported | `nf-game/src/player.rs:1310` (`Bond::wire_tick`) — hand-over-hand: the travel segment, the auto-drop, the drop/strafe mode bytes, exits |
| 0x197f50 | Player_Zipline | ported | `nf-game/src/player.rs:1523` (`Bond::zipline_tick`) — the ride: fall += 0.15/tick, the line projection, the lower-end arrival mode 9; unreachable in the shipped game, ported for the dispatch's sake |
| 0x1a16c0 | Player_WeaponHasAmmo | ported | `nf-game/src/gun.rs:155` (`Gun::has_ammo`) — the 0x45 grenade-belt test, carried/clip/ammo_type-0 arms; the 0x20000 ground scan documented STANDIN |
| 0x1a17f8 | Player_InitAmmoWeapons | ported | `nf-game/src/player.rs:2406` (`init_ammo_weapons`) — the boot arm: carried/clip/owned zeroed; the SQRT(+0x88) zoom lives in gun.rs:375; the ammo-cheat arm not ported |
| 0x1a18e0 | Player_RoundToFire | ported | `nf-game/src/gun.rs:529` (`Gun::update_with`) — the ammo_type-0 free arm, the clip check + the raw +0x91 spend (ammo_per_shot, gun.rs:430); the fire-rate cheat not ported |
| 0x1a1988 | Player_ReloadAmmoType | ported | `nf-game/src/gun.rs:220` (`Gun::reload_tick`) — rounds-move at case 6 (min(spare, clip−size), one-at-a-time); the twin clip-slot pick `Sim::clip_slot` scene.rs:4100 |
| 0x1a1a70 | Player_Zoom | ported | `nf-game/src/gun.rs:350` (`zoom_tick`) — the ×1.025 axis ramp, the ×1.15 idle ease, the [1,min(max,50)] clamp, remembered zoom; wired at scene.rs:3787 |
| 0x1a1d28 | Player_Aiming | ported | `nf-game/src/input.rs:355` (`PadDriver::tick`) — the aim cursor/edge-turn ramps (0x1a1d28:78-194) + the free-look arm; the Sim cursor clamp scene.rs:3160 |
| 0x1a24c8 | Player_Weapon | ported | `nf-view/src/scene.rs:3024` (`Sim::run`) — folded: the mode/twin block → `fire_mode_switch` (gun.rs:316); zoom gun.rs:350; the party lock weapon_lock_input player.rs:2674 |
| 0x1a2ba8 | Player_HandleHasNoAmmo | ported (partial) | `nf-view/src/scene.rs:4640` (`Sim::run`) — only the 'R' Ronin arm (0x52→0x53 re-equip); the twin/GetBestWeapon dry fallback not ported |
| 0x1a2cc0 | Player_WeaponSelect | ported | `nf-view/src/scene.rs:4224` (`Sim::run`) — the variant-swap/holster/draw state pick + the twin-marker write, matching decompile lines 13-39 |
| 0x1a2e38 | Player_SetWeaponAnimObj | ported | `nf-view/src/scene.rs:4205` (`Sim::run`) — the switch states 1/2/3/4/8 + the aims idle counter; reload cases 5/6/7 in `Gun::reload_tick` (gun.rs:220); case 9 via release_frame_reached (gun.rs:557) |
| 0x1a3b40 | Player_PositionGun | ported | `nf-view/src/scene.rs:6712` (`Sim::gun_matrix`) — the full fp gun matrix: sway offset, crosshair euler tilt, pivot |
| 0x1a3c70 | Player_SetFiringAnim | ported | `nf-game/src/gun.rs:586` (`Gun::firing_anim_script`) — the script pick order (last-round, fists random, the 0x40000 alternation); the state-9 gate in update_with |
| 0x1a3de8 | Player_WeaponRecoil | ported | `nf-view/src/scene.rs:6712` (`Sim::gun_matrix`) — the sway vector + tilt÷zoom folded in; the doc cites 0x1a3de8:52-57 |
| 0x1a4298 | Player_MuzzleFlash | ported | `nf-view/src/scene.rs:5326` (`Sim::run`) — the muzzle-datum light + smoke + the laser/taser beam re-cast (5166-5172); the +0x48/+0xDC gates kept |
| 0x1a4a98 | Player_GetAimingPoint | ported | `nf-view/src/scene.rs:3723` (`Sim::step_bond`) — the crosshair-ray direction; the aim point head+dir·256 consumed at scene.rs:5396 (MP: 5975) |
| 0x1a4bf0 | Player_WeaponInitBullet | ported | `nf-game/src/drone.rs:14097` (`Drones::fire_player`) — the per-pellet spawn, shutter/whoosh, casings; the 0x20000 detonator + Ronin arms in Sim::run (scene.rs:5049/5092) |
| 0x1a5170 | Player_Activate | ported | `nf-view/src/scene.rs:5474` (`Sim::run`) — the use-edge creepwall arm + the radius-1 head pick; the class arms via `Drones::player_use_doors` (drone.rs:16653) |
| 0x1a5460 | Player_WeaponFiring | ported | `nf-game/src/gun.rs:477` (`Gun::update_with`) — the trigger/reload/refire machine; the decryptor/underwater/view-state gates in Sim::run (scene.rs:4617-4635) |
| 0x1a65d8 | Player_LaserPointer | ported | `nf-game/src/gadget.rs:140` (`gadget::laser_tick`) — the watch beam endpoints (scene.rs:5170); the SG5 sight-dot block scene.rs:5877-5891 |
| 0x1a69d0 | Player_SetWatch | ported | `nf-game/src/watch.rs:50` (`watch::hand_angle`) — the clock data + hand rates; the mesh attach scene.rs:8740; the per-tick pose scene.rs:11579 |
| 0x1a6eb0 | Player_IsBetterWeapon | ported | `nf-game/src/pickup.rs:537` (`Inventory::is_better_weapon`) — the gadget guard + the BestWeapon ranking, drives the pickup auto-switch |
| 0x1a6f80 | Player_WeaponChange | ported | `nf-game/src/gun.rs:85` (`Gun::cycle`) — the cycle bounds + twin step; toggle_back gun.rs:120; wired scene.rs:3127 writing +0x63 |
| 0x1a7c98 | Player_AutoAim | ported | `nf-game/src/drone.rs:15996` (`Drones::auto_aim`) — the target scan, the Q-Claw grapple arm, the lock-on ×1.4, the difficulty muls |
| 0x1a7ef8 | Player_InitWeapon | ported | `nf-game/src/player.rs:2468` (`init_level_loadout`) — the per-level loadout switch, the RamLoad cases, the common equip tail |
| 0x1a8a58 | Player_EquipWeapon | ported | `nf-game/src/pickup.rs:456` (`Inventory::equip_weapon`) — the loaded gate, the base-slot owned, the clip fill/carry, the cap clamp |
| 0x1a8d18 | Player_EquipAmmo | ported | `nf-game/src/pickup.rs:477` (`Inventory::equip_ammo`) — the 0x10 empty-clip fill, the cap clamp; the id remap upgrade_ammo_id (pickup.rs:553) |
| 0x1a9148 | Player_AmmoIndex | ported | `nf-game/src/gun.rs:178` (`Gun::ammo_in_gun_slot`) — the same twin/base+ammo-type predicate; also `Inventory::clip_slot` (pickup.rs:448) |
| 0x1a9190 | Player_AmmoInGun | ported | `nf-game/src/gun.rs:178` (`Gun::ammo_in_gun_slot`) — the shared clip-slot pick; threaded into has_ammo by all callers |
| 0x1a92e8 | Player_ViewClamping | ported | `nf-game/src/player.rs:999` (`Bond::climb_tick`) — the pitch offset clamp ±1.0 inline; the doc cites 0x1a92e8:9-16 |
| 0x1a9338 | Player_SetWeaponAnim | ported | `nf-view/src/scene.rs:4291` (`Sim::run`) — the +0x964 laser latch on every completed draw; the anim-set half in `MpBodyAnim::tick` (mp/body_anim.rs:264) |
| 0x1a9680 | Player_GetBestWeapon | ported | `nf-game/src/gun.rs:187` (`Gun::get_best_weapon`) — the BestWeapon table scan; wired scene.rs:3148 and 5081 |
| 0x1a9790 | Player_ClearAimInertia | absorbed | zeroes +0x128/+0x12C on the equip/stand paths; the port stores no aim deltas to clear (the offsets decay naturally) — a no-op by construction, documented GAPS.md G-006 and player.rs:1164 |
| 0x1a97a0 | Player_EquipKey | ported | `nf-game/src/pickup.rs:590` (`Inventory::take`) — the kind-KEY arm sets keys[id]=1, never refuses |
| 0x1a97b8 | Player_UnEquipKey | ported (this branch) | was MISSING — the ring clear had no code and no caller; fixed as the re-lock arm's `inventory.keys[key] = false` in `scene.rs:2850` (the case-2 pass) via `Lock::relock_if_dropped` (lock.rs:257); red-then-green on the real 07000012 path (decryptor_07000012_live) |
| 0x1a97c8 | Player_CheckWeaponsLoaded | ported | `nf-game/src/player.rs:2649` (`init_level_loadout`) — the owned revoke when the fp skin is missing; the MP lookup half = mp_weapon_loaded (scene.rs:818) |
| 0x1a9858 | Player_WeaponNone | ported | `nf-game/src/gun.rs:200` (`Gun::go_to_hands`) — hands 0x47, saves +0x64 for toggle-back; called on the ladder/wire grab, death, the Ronin mount |
| — | Player_PositionCamera | ported | `nf-view/src/lib.rs:5791` (`App::frame` camera dispatch) — the first-person eye + the per-mode helpers: orbit_camera (scene.rs:15350), creep/wire/cam2 (scene.rs:3939/3977/4037), the view_override case 0xC, gt.rs case 0xF |
| — | Player_Init | ported | `nf-view/src/lib.rs:1557` (`load_impl`) — stand_at, settle, WldGravity p3==2, substate 8/4 + crouch drop; loadout via `init_level_loadout` (player.rs:2468) |
| — | Player_Disable | ported | `nf-view/src/scene.rs:3797` (`Sim::step_bond`) — Request::PlayerDisable → self.disabled + timer-pause + goggles clear; obj+0xF0 bit's later life not ported |
| — | Player_Zipline | ported | `nf-game/src/player.rs:1523` (`Bond::zipline_tick`) — the ride: fall += 0.15/tick, the line projection, the lower-end arrival mode 9; unreachable in the shipped game, ported for the dispatch's sake |
| — | Player_InitAmmoWeapons | ported | `nf-game/src/player.rs:2406` (`init_ammo_weapons`) — the boot arm: carried/clip/owned zeroed; the SQRT(+0x88) zoom lives in gun.rs:375; the ammo-cheat arm not ported |
| — | Player_WeaponChange | ported | `nf-game/src/gun.rs:85` (`Gun::cycle`) — the cycle bounds + twin step; toggle_back gun.rs:120; wired scene.rs:3127 writing +0x63 |
| — | Player_InitWeapon | ported | `nf-game/src/player.rs:2468` (`init_level_loadout`) — the per-level loadout switch, the RamLoad cases, the common equip tail |

Counts: 90 functions — **86 ported** (one of them partial: `Player_HandleHasNoAmmo`'s
dry-fallback arm; one of them state-half: `Player_GrappleSetRope`'s rope object — see the
MISSING section), **2 absorbed** (`Player_ResetWalkAnim`'s write-only field,
`Player_ClearAimInertia`'s no-op), **1 not reached** (`Player_SSScanMode`, the debug-fly
cheat), and the sweep's one MISSING (`Player_UnEquipKey`) fixed on this branch with its
real-level red-then-green test.

Notes:

- **The address-column gaps**: seven rows whose hex contains a function-key character pair
  are listed by name only (the push guard); their addresses sit in the `work/decomp/
  ACTION.ELF/` filenames, and each carries its full row earlier in the table.
- **The MP-only reaches** (`Player_ChangeState`, `Player_Kill`): ported at the multiplayer
  scenario/respawn sites, which are their only game callers [M whole-decompile greps].
- **Render-side residue** (not ported, not sim-observable): the rope mesh
  (`Player_GrappleSetRope`/`Player_InitGrapple`), the framelist open animation of an
  unlocked lock, the watch/laser view-model details, the HUD air pane of `Player_MonitorAir`.
  These live in the systems docs (bldata.md, gadgets.md) and the live-checks section above.
