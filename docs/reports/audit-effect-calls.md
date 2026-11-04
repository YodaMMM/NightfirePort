# audit-effect-calls — every visual effect call in ACTION.ELF vs the port

Branch `s/audit-effect-calls` (cut from origin/master b2cd660b3). Shell casings out of scope.
Method: two independent censuses (a decompile sweep of every producer + call site, and a port
sweep of every spawn site), merged into the table below, then fixes at the clear port sites.

## Producers and the port's coverage

`[M]` decompile address, `[D]` disc, `[S]` port choice, `✓` ported, `→` where.

| Producer | Decomp call sites (on-foot) | Port site | Status |
|---|---|---|---|
| Effect_Create 0x1ce688 | Bullet_CollisionHandler 0x12ce80 (0x61/0x10 arms), AnimObjectUpdate 0x1befd8:201 (footsteps 0x42), Script_Interp 0x1d9a08 case 2 (0x42), AnimProcessScriptCmds 0x1c1b38:407 (0xc0) | effect.rs:465-537 + drone.rs:15638/:20570 (bullet), drone.rs:15636 (laser), scene.rs footstep block (foot), — (0xc0) | ✓ bullet/laser/foot; 0xc0 arm = sound-only, see table |
| Effect_Bullet 0x1ce8a8 | via Effect_Create flags&1 | effect.rs `bullet_impact` (emitter→debris→gas→decal→ricochet) | ✓ |
| Effect_Player 0x1cec48 | via Effect_Create flags&2 (footsteps) | scene.rs:3575 (sound) + **NEW decal arm** scene.rs:3578-3600 | ✓ (fixed this task) |
| Effect_Laser 0x1cef38 | via Effect_Create flags&0x10 | drone.rs:15636 (burn puff); second GameState&1 puff → table | ✓ main arm |
| Effect_Body 0x1cf400 | via Effect_Create flags&0x80 (anim cmd 6) | — (sound only; sound audit's scope) | table |
| Effect_RicochetProb 0x1cf290 | Bullet_CollisionHandler 0x12ce80 | effect.rs:519-537 | ✓ |
| Explode_Create 0x17cbd0 | 13 sites (see below) | drone.rs `spawn_blast` + per-site pushes | ✓ 9 of 13; 4 MP-only → table |
| Explode_Propagate 0x17c518 | Explode_CollisionHandler 0x17cde0, GunImp_Update 0x1320d8:434 | explode::propagate (drone.rs:14154+), beam drain drone.rs:16288/:16337 | ✓ |
| Gas_Init 0x1aacc8 | 26 sites | gas.rs wrappers (below) | ✓ 20; 6 → table |
| gases_smoke 0x1ab2d0 | type-0xe dispatch (no C callers) | gas.rs `update` kinds 0/4/7 | ✓ (kinds 9/10 → table) |
| Bullet_trail_effect_smoke 0x12cba8 | Bullet_DoTrails 0x12d890:146 | gas.rs `init_trail_smoke` (drone.rs:20830) | ✓ kind-7 arm; kind-0xb arm → table |
| psiCreateShard 0x1178e0 | ShatterPolyRecurse 0x12b270 ← Break_Kill 0x12b970 | shatter.rs `spawn_glass` (breakable.rs:447+) | ✓ |
| Player_MuzzleFlash 0x1a4298 | Player_CollisionHandler 0x1949a8:190 | scene.rs:5168-5210 (light latch +0x95b, Light_Create, Gas_Init smoke) | ✓ |
| Draw_MuzzleFlash 0x1a3fe0 | Player_CollisionHandler 0x1949a8:201 | scene.rs:7448/8300/8460 (`muzzle_flash_matrix` 0x938-timer) | ✓ |
| Player_SetFlashBang 0x196040 | Bullet_DoTrails 0x12d890:349, Apocalypse_Update 0x18d4f0:35 | drone.rs:20962 (stun grenade); Apocalypse → table | ✓ main arm |
| Camera_Shake 0x1bc430 | Explode_Create internal (radius≥0.5 [M 0x17cbd0:63]), Bullet_DoTrails 0x12d890:291 (2.0), Mine_Update 0x17c930:105 | drone.rs:14324/:20971 set `Drones::shake`; Mine via spawn_blast | ✓ sim-side; view consumption → table |
| HUD_UpdateBloodPane 0x19a6a8 | 0 C callers (HUD pane fn-pointer table) | death.rs state machine + hud.rs:222 `blood_pane` | ✓ |
| NDrone2_DoHitEffects 0x1458e0 | 26 sites (ninja msg 6/8 arms, PunchImpact 0x145f28, BulletImpact 0x1461c8) | drone.rs:3950→3985 `do_hit_effects` + blood/bolt puffs :15430 | ✓ |
| Drone_ExplosiveHit 0x13aab8 | Explode_Propagate 0x17c518 (msg 9) | explode::propagate → drone.rs:3512 | ✓ |
| Light_Create 0x1cc3d8 / Emitter_Create 0x17ae00 / Debris_Create 0x1aaa00 / Debris_CreateEx 0x1ac808 / Swoosh_Create 0x18d8e8 / Occlude_Create 0x17ea80 | throughout | light.rs / world_fx.rs / effect.rs+explode.rs / drone.rs:15731 / drone.rs:21011 | ✓ (Swoosh has one [S] puff stand-in, drone.rs:14702) |

Explode_Create's 13 sites: bullet arms ×3 (0x12ce80:198/211/223 — 0x6000052, 0x60007c4×2) ✓
drone.rs:15662/:15409; grenade fuse (0x12d890:391) ✓ drone.rs:21027; Mine_Update 0x17c930 ✓
(mines ported, blast attacker = mine, drone.rs:14095); Copter_Update 0x17dbb8:270 ✓ drone.rs:18766;
Destroy_Smash 0x12bb60:72 ✓ drone.rs:15022; Monitor_Update 0x1afcc8:43 ✓ drone.rs:14987;
Bullet_handle_object_destruction 0x12cd98:20 ✓ drone.rs:14945; **Car_Deactivate 0x131d88:66 —
fixed this task**; GT_Create chain 0x17ffa0:157 / GT_Update self-destruct 0x180498:435 /
MP_DemolitionProtectionUpdate 0x1870a0:144 / Drone_Delete→GT chain 0x136ce8:92 → table (MP-only).

## Fixed this task

1. **The footstep's footprint decal (Effect_Player's decal arm).** The game's every footstep is
   `Effect_Create(1.0, hit, obj, foot | 0x42)` [M AnimObjectUpdate 0x1befd8:196-201] — flag 0x40
   plays the surface sound (the port had this) and flag 0x2 runs Effect_Player 0x1cec48:59-140,
   which spawns a decal per step: gfx = `EffectInfo[mat]+0x8` on foot 4 / `+0xc` on foot 8
   [M 0x1cec48:84-89], the foot alternating each step [M 0x1befd8:186-190 `ScriptFootstepFoot`
   flip → :196-198], y = the ground hit's y + 0.05 [M 0x1cec48:118-124], aligned to the surface
   up and hosted on the ground cel [M 0x1cec48:107-136]. The port dropped the whole decal arm.
   Fix: `EffectRecord.foot_decal_4/foot_decal_8` parsed from EffectInfo+8/+0xc (sfx.rs; the disc
   table has prints on CONCRETE 0x02000262/0x02000567, WOOD, METAL/DIRT 0x02000128, MUD/GLASS
   0x02000127, GRASS/SAND 0x02000262/0x02000214, GRAVEL/SNOW 0x02000126, FABRIC 0x020006fd —
   19 of 23 rows carry at least one), and the decal spawn in scene.rs's footstep block
   (`segment_surface_host` supplies normal + cel host; y = floor + 0.05; `add_decal`). [S] the
   print's x/z sits at the body's ground point — the game uses the foot-bone world matrices
   (`obj+0xdc → +0xb8` bones +0xdc0/+0xcc0) the port does not build for first-person Bond.
   Proof: `footprint_decals_live` — real input from the 07000005 start (no teleport), 7-unit
   walk ⇒ ≥2 prints, meshes alternate +8/+0xc starting foot 8, y = floor+0.05, cel host.

2. **Car_Deactivate's dropped Explode_Create.** Every non-99 entry of Car_Deactivate 0x131d88
   runs `Explode_Create(10.0, 10.0, 50.0, car, obj+0x30, 0x2d6210, 0x6000052, 0xff, 0xff, 100,
   car)` unconditionally, BEFORE the TankSpawns pose restore [M 0x131d88:66-73]; the port's
   `deactivate()` only reset state. Fix: `Car::deactivate`/`CarSystem::delete` return the
   pending-blast tuple (live obj+0x30 position, radius 10, damage 10, script 0x06000052,
   scale = radius per the 0x4-arm convention, debris kind 6 = the 0x6000052 arm's copter-death
   precedent) for the caller to push into `Drones::pending_blasts`; owner `usize::MAX-1` (the
   car has no drone-registry id, the copter-death mark). Note: the Car system is not yet wired
   into a live sim tick (car.rs module doc), so the proof is unit-level:
   `car_state_machine::deactivate_matches_car_deactivate_0x131d88` (blast values + the state-99
   no-op re-entry [M 0x131d88:20]) and `car_registry_and_set_bits::car_delete_is_car_deactivate_0x132e78`.

## Unported calls / needs work (the table, with data)

| # | Missing/wrong | Decomp | Data |
|---|---|---|---|
| 1 | Screen shake never reaches the camera | Camera_Update 0x1bb6d8:35-43 | per viewer: `viewer+0x150 += Rand_FRand_MVar2_Vec(shake·FRAME_RATE)`, `shake -= Rand_FRand(shake·0.25)`, clamp ≥0. The sim keeps `Drones::shake` (set at drone.rs:14324 radius≥0.5 [M 0x17cbd0:63], :20971 = 2.0 flashbang) with an [S] decay at :21200; nf-view has no reader. Needs the Camera_Shake store (viewer+0x200) + the Camera_Update consumption wired into the port's camera — render work for the main agent. |
| 2 | NPC footsteps | AnimObjectUpdate 0x1befd8:160-201, Script_Interp 0x1d9a08 case 2 | `Effect_Create(1.0, obj+0xdc hitdata, obj, 4/8\|0x42)` when the per-frame accumulator (stride 1.65, 0.4125 when obj+0xf6==1) wraps; uses the STORED hitdata at obj+0xdc (the port doesn't keep per-NPC cached hitdata); suppressed while ScriptCam≠0; the run branch (obj+0xf6==1) plays only Sound_Play3D(0x286). The player's own footstep is ported (fixed above); NPCs' is not. |
| 3 | Water-hit ripple never fires | Env_WaterRipple 0x17c118 ← Player_DealWithObjHit 0x194458:80-83, NDrone2_DealWithObjHit 0x13fff0:79-84 | the flesh handlers call it when the hitdata records water (hitdata+0x60 & 4; the ripple body itself gates material 0xd/0xe) → `Gas_Init(0.5, 1.01, hit+0x40, 0,0, 0x2000129, kind 1, 0x7f7f7f7f, 2)`. The port has `env_water_ripple` (env.rs:230) with NO production caller; wiring needs the +0x60 hitdata flag (or an equivalent water-ground gate) in the flesh-hit path. |
| 4 | Fire flicker's smoke puffs | Flicker_Update 0x1ab960 | `Gas_Init(1.2, 0.8, fire+0x80, drift, random fire gfx, kind 4, rgba 0x0a460003)` every other frame while the fire is active — skipped behind the stubbed `Light_FindMapLight` (world_fx.rs:14-17). |
| 5 | Gas kinds 9/10 | Gas_Init 0x1aacc8 + gases_smoke 0x1ab2d0:152/:175 | hidden cloud emitters (kind 9 spawns kind-8 children on a 20 s clock, emitters spawn kind-7 children) — gas.rs:357-359. |
| 6 | Turret muzzle smoke ×3 | GT_Update 0x180498:356/:530, GunImp_Update 0x1320d8 | `Gas_Init(0.05, 0.25, muzzle, …, 0x2000039, kind 7, 0x48403080/0x484030e4, 1)` on each turret shot. The port's GT Fire drain (drone.rs:17473) plays the sound + spawns the round, no puff. |
| 7 | Rigid-body scrape dust | RB_Update 0x1d6e38 | `Gas_Init(0.5, 1.0, rb+0xc0, …, 0x2000129, kind 1)` while scraping, frame&0xf≠0 — rigid_body.rs has no FX. |
| 8 | Anim-script effect cmds 6/7 | AnimProcessScriptCmds 0x1c1b38:407/:453 | cmd 6 = `Effect_Create(…, 0xc0)` (body-impact sound via Effect_Body); cmd 7 = muzzle event: `Light_Create` + `Gas_Init(0.05,0.05, muzzle bone, 0x2000039, kind 7, 0x48403080, 1)` for any NPC playing a firing anim. The port's interp (scene.rs:4087+) handles op 1 sounds only. |
| 9 | Apocalypse flashbang | Apocalypse_Update 0x18d4f0:35 | `Player_SetFlashBang(clamped, glb_blokes, …)` + rumble + direct viewer shake when the level-script record's +0xd flag is set — world_fx.rs:367-369 notes the consumer unported. |
| 10 | MP-only blasts | GT_Create 0x17ffa0:157, GT_Update 0x180498:435, MP_DemolitionProtectionUpdate 0x1870a0:144 (script 0x060004f, pos obj+0x80), Drone_Delete 0x136ce8:92 | MP chain/self-destruct/demolition/drone-owned-GT detonations — mp/combat.rs has contact/fuse/tripbomb blasts but none of these four. |
| 11 | Laser impact's env puff | Effect_Laser 0x1cef38:117 | second `Gas_Init(0.005, 0.02, kind 0, 0xc8c8c87f → 0x20202006)` when GameState&1 — the port spawns only the EffectInfo+0x10 burn puff (drone.rs:15636). |
| 12 | Bullet trail's big puff | Bullet_trail_effect_smoke 0x12cba8:41-53 | the `+0xf6 != 0` branch: `(frame+id)%15==0` → `Gas_Init(0.2, 5.0, 0x200001f, kind 0xb, 0x404040ff→0x404040ff)` with 3 velocity draws + spin. The port runs only the kind-7 arm (drone.rs:20829, also note: its gate drops the game's `+id` offset — `GameState+id & 3`). |
| 13 | Wall-puff velocity + the +0x51 roll | Effect_Bullet 0x1ce8a8 | the port zeroes wall-puff velocity [S] (Vec_NormToRot unported, effect.rs:502-504) and skips the HITDATA+0x51 second-byte material roll (effect.rs:14-15). |
| 14 | Guard muzzle-flash sprite | DroneWeap_FireWeapon 0x178f68 | the kind-0x40 flash object + its 3 orientation draws are skipped (drone.rs:11283-11291, RNG draws deliberately not consumed) — pending a live gdbwatch. |

## Live checks for the main agent

1. Footprints: stand Bond on any concrete floor (07000005 start porch), walk forward — expect
   alternating print decals (meshes 0x02000262 / 0x02000567) at floor+0.05 under Bond. The x/z
   sits at the body centre [S]; the game puts it under the stepping foot bone — if the print
   looks centred, that's the port's [S], not a bug.
2. Screen shake: detonate any blast (radius ≥ 0.5) near the camera — currently NOTHING shakes
   (table row 1); confirm and pick the wiring spot (Camera_Update consumption).
3. Guard muzzle flash: fire a guard weapon in a cutscene-free fight — the sprite object is not
   created (drone.rs:11283-11291); confirm the game shows the billboard flash on PCSX2
   (`gdbwatch` on the kind-0x40 create) so the skip can be resolved.

## Verification (every run on this branch, NF_ISO set, --test-threads=1)

- lib tests: nf-audio 23/0, nf-formats 92/0, nf-game 661/0 (5 ignored), nf-view 100/0.
- nf-game effect-adjacent binaries: blast_smoke_live 4, break_debris_live 2,
  car_registry_and_set_bits 7, car_state_machine 5, grenade_live 4, grenade_rattle_live 4,
  guard_drops_flashbang_live 6, laser_gas_mesh_live 1, searchlight_live 3, stun_grenade_live 1 — 37/0.
- nf-view decal/weapon binaries: decal_door_follow_07000011_live 4, decal_order_live 1,
  fix_weapon_effects_live 6, impacts_live 5, impact_remainder_live 2, mp_shot_effects_live 2,
  fix_flesh_impacts_live 4, fix_foliage_bullets_live 3, weapon_fx_levels_live 2 (+2 ignored),
  footprint_decals_live 1 (new) — 30/0.
- EVERY walkreal_*_live binary: 07000001 8, 02 3, 03 6, 04 10, 05 14, 06 6, 07 5, 08 6, 09 8,
  0a 13, 0b 7, 0c 3, 0d 1, 11 3, 12 3, 13 3, 14 9, 15 6, 16 14, 1b 17, 41 15, 4a 4,
  4a_satchel 2 — 166/0 (17 ignored). No walkreal regression from the footprint arm.
- Not re-run whole: the disc_golden trio (allowed fail on the slim ISO; the full ISO is set
  here and the nf-formats lib run covers the parse), and the 956-binary long tail untouched by
  the three changed sources (car.rs is unwired; sfx.rs adds parse-only fields; scene.rs's arm
  only adds decals — covered by the walkreal + decal sets above).

## Usage

not visible to me
