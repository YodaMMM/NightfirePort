# Bullets-family port map

Every `Bullet_*`, `Gun*`, `Weapon*`, `Explode_*`, `Effect_*` function in ACTION.ELF
(SLUS-20579 NTSC-U, build e7ffdcb3) — **32 in total** (13 `Bullet_*`, 5 `GunImp_*`,
5 `Explode_*`, 9 `Effect_*`; no function is named `Weapon*` or bare `Gun*` in this build —
the `Player_Weapon*`/`Player_WeaponFiring` set is weapon-switching's map,
docs/port-map/weapon-switching.md, and the `DroneWeap_*` firing set is the drones map's).
DRIVING.ELF and the SLUS ELF carry none of the family. One line each: PORTED with the
port's code location, or NOT REACHED with the proof.

Method: the decompile files `work/decomp/ACTION.ELF/<addr>_<name>.c` matching the five
name prefixes were enumerated (32; a case-insensitive sweep for other spellings found
none), then each exact address was searched in `crates/*/src` and the citation resolved
to its implementing function by reading the code around it — a comment mention alone did
not count. There are **no NOT REACHED rows**: every one of the 32 is dispatched by the
shipped disc on real levels (the bullets fly, casings eject, grenades explode, impacts
draw, and 0700000d's mount reaches the GunImp set), and each has port code.

## Table

| Address | Name | Status | Port location / proof |
|---|---|---|---|
| 0x12c5a0 | `Bullet_init` | ported | `crates/nf-game/src/bullet.rs` — `Bullet::init_fuse` (the 210.0 default [M :103], the 300.0 `+0x70 & 0x10000` records [M :192-198], the satchel family `((kind − 0x3b)·5 + 5)·60` and the burn counter [M :199-228]) with `spread`/`cone` (the dir jitter) and the BU fields' docs |
| 0x12cba8 | `Bullet_trail_effect_smoke` | ported | `crates/nf-game/src/gas.rs` `init_trail_smoke` (the 0x0200_001f projectile-trail puff [M disasm 0x12cc10-0x12cc74, gas.rs:46]); the every-4th-tick emission from the `+0x6C` bit 0x2 records in `drone.rs` (21332, the [L ee_start_05.bin @0x2bf150] record sweep) |
| 0x12cd98 | `Bullet_handle_object_destruction` | ported | `crates/nf-game/src/grenade.rs` `destruction_gate` (the class-5 victim's record gate [M]); the fuse-death and impact explode sites in `drone.rs` (21220 the fuse arm, 21598 the near-ray guard kill) and the tripbomb gadget take (15122 — the gadget deletes, `Explode_Create` from the GADGET's record, the 0x80000 beam delete) |
| 0x12ce80 | `Bullet_CollisionHandler` | ported | `crates/nf-game/src/bullet.rs` `Bullets::update` (the world/player arms, the kill) + `drone.rs` `flush_world_impacts` (15779) — the ricochet count `bounces` killed at 3 [M the 0x12d798 handler], the object arm's pass-bit and damage gates at the drone.rs sites |
| 0x12d890 | `Bullet_DoTrails` | ported | `crates/nf-game/src/bullet.rs` (`flyby` = `weapon+0x6C & 0x40000` [M], `fuse_ticks` = `& 0x200`) and `drone.rs` `tick_charge_fuse`'s DoTrails arm (11630-11658: the family 0x59/0x5b/0x37(7777) hold, the −1.0/tick, the 0x2000 stun and 0x20000 smoke death branches) |
| 0x12e660 | `Bullet_Delete` | ported | `crates/nf-game/src/drone.rs` — the bullet-pass flush's retain (15759, a revived bullet keeps its handle) and every death funnel through the same flush (14108 case 3, 15940 the rocket-sound-leak note [M :13-16]) |
| 0x12e7b0 | `Bullet_update` | ported | `crates/nf-game/src/bullet.rs` `Bullets::update` (the state machine 0/1/2/3, the first-tick half-step draw, the `weapon+0x1C` range kill with the boundary clamp [M 0x12e95c-0x12e984, 0x12f014-0x12f02c], the segment-vs-cylinder narrow phase) |
| 0x12f248 | `Bullet_heatseekable` | ported (3 of 4 arms) | `crates/nf-game/src/drone.rs` 15724-15779: the class-2 drone arm (guards while `+0xFC` set, health > 0, state ≠ 1; the port's `drawn` IS +0xFC [M View_AddObjects 0x1e62d8:70-71]) and — this branch's fix — the class-0 arm (a copter BODY: drawn, NO health read [M the CopterList walk]; reached on 07000012 copter #707 + pickups #2aa/#2f0 and 0700000b #148 + #15d [D the sweep], the live harness `homing_copter_07000012_live.rs` lands ignored pending the give's 0x2e draw). NOT ported: the class-3 arm (Bond, `BLData+0x894 > 0`) is comment-only in SP and lives only in the MP fold (`mp/combat.rs:462-489`) — the copter's own 0x6d rounds at Bond are the reached slice (copter.rs:147 [M]); the class-0x28 arm is unported. THE REMAINING GAP — next round |
| 0x12f358 | `Bullet_homing` | ported | `crates/nf-game/src/drone.rs` — the `Bullet_homing` block (15695, keep/acquire within the 0.66-dot cone and 1000 units with clear LOS [M :64-104], the `clamp((to_target − dir)·0.0333, ±0.01)` steer [M :126-166]); the thrown-hat copy for the grenade path (21270-21300, the 0x45's `+0x6C = 0x20C08` [L ee_start_05.bin @0x2c398c]) |
| 0x12f678 | `Bullet_init_casing_ex` | ported | `crates/nf-game/src/casing.rs` (the eject: the view-model bone transform, `Rand_FRand_MVar2` velocity/spin jitter, the landing plane, the fire sites `Player_WeaponInitBullet` 0x1a4bf0 and `DroneWeap_FireWeapon` 0x178f68) |
| 0x12faa0 | `Bullet_casing_update` | ported | `crates/nf-game/src/casing.rs` (gravity ×0.1's state-1 arm documented unreachable for class-0x0d [M control_funcs 0x26f890], the one-shot landing sound parked at −1000000.0 [M 0x12fb88-0x12fb94], the 240-tick despawn [M 0x12fc58-0x12fc94]; the opt-in `set_keep` PC option rests at 0 = the game's behaviour) |
| 0x12fd48 | `Bullet_InitCasingLists` | ported | `crates/nf-game/src/casing.rs` (the 32-object round-robin pool, the free/in-use shape) |
| 0x12fde0 | `Bullet_init_casing` | ported | `crates/nf-game/src/casing.rs` (the delayed activation, the class byte 0x0d [M :24]) |
| 0x131f20 | `GunImp_Create` | ported | `crates/nf-game/src/gunimp.rs` (the class-0x47 create; the TurretGuns entry 0xb laser pinned by hash [D ACTION.ELF @0x26f520], the SP arm reads 0xb [M :52-63]; 0700000d's #225 mount) |
| 0x1320d8 | `GunImp_Update` | ported | `crates/nf-game/src/drone.rs` `gunimp_tick` (16362-16374; the fire-interval FRAME_RATE_MUL countdown [M 0x1320d8:107-112, :324]) |
| 0x132c18 | `GunImp_Deactivate` | ported | `crates/nf-game/src/drone.rs` `gunimp_tick`'s dismount arm (the cam-mode/substate restore and the velocity zeroes [M 0x132c18], the `active` flag the use-key clears) |
| 0x132f00 | `GunImp_Activate` | ported | `crates/nf-game/src/gunimp.rs` (the use-key mount: `Player_ChangeSubState(0xc)`, cam 0xe, `Player_Disable(player, 1)`, `Player_WeaponNone`, `Input_SetAction(0xe)`, state `obj+0xf4 = 1` [M 0x1a5170's class walk]) |
| 0x133008 | `GunImp_PlayerHasDied` | ported | `crates/nf-game/src/props.rs` `gunimp_player_has_died_drops_the_mount` (393; the walk for the class-0x47 mount linked to the dead player [M :4-17], the SP single-imp fold documented at 141-147) |
| 0x17c350 | `Explode_Update` | ported | `crates/nf-game/src/explode.rs` `Explosion::update` (the lifetime ticks, the debris-kind switch 5/6 metal, 8/10/0xb stone-on-0x7000008 [M the EXPLODEINFO +0x38 docs]) |
| 0x17c518 | `Explode_Propagate` | ported | `crates/nf-game/src/explode.rs` `propagate` (the cel-scoped sphere test, the radius falloff, the attacker attribution) — consumers `drone.rs` `blast_mines` (14305), `mp/combat.rs`, the breakable/worldobj arms |
| 0x17cbd0 | `Explode_Create` | ported | `crates/nf-game/src/explode.rs` `Explosion::create` + `create_debris`/`debris_arm` (the EXPLODEINFO field map [M every field off the 0x17cbd0 disasm]; the fuse call site 0x12cd98's radius/damage/script pins) |
| 0x17cdc0 | `Explode_Delete` | ported | `crates/nf-game/src/explode.rs` (the update's done handoff ends the entity) — the mounted-laser beam explode drains through the same path (`drone.rs` 21642) |
| 0x17cde0 | `Explode_CollisionHandler` | ported | `crates/nf-game/src/explode.rs` (the once-latch `+0x3A` firing `Explode_Propagate` exactly once [M the field doc]) |
| 0x1ce688 | `Effect_Create` | ported | `crates/nf-game/src/effect.rs` (the material-0 early-out, the debris-count draw order [M the RNG-discipline header]) with the hit dispatch in `drone.rs` (15637) and `world_fx.rs` |
| 0x1ce8a8 | `Effect_Bullet` | ported | `crates/nf-game/src/effect.rs` `add_decal` (the decal FIFO: the free/in-use node pair [M DList_MoveFromFree2InUse 0x1cccb8], the per-material mesh from the EffectInfo table [D 0x3037c0]) |
| 0x1cec48 | `Effect_Player` | ported | `crates/nf-view/src/scene.rs` `step_foot` (the foot decal arm: `EffectInfo+0x8/+0xc` per foot, the y + 0.05, the `Mat_Align2Up` host [M 0x1cec48:59-140]; the foot-bone position is a documented [S] fold — the port uses the body's ground point) |
| 0x1cef38 | `Effect_Laser` | ported | `crates/nf-game/src/drone.rs` 15831-15870 (the laser impact: the surface-gated gas puff [M :59,80-83], NO decal, NO impact sound, NO ricochet) |
| 0x1cf268 | `Effect_InitLists` | ported | `crates/nf-game/src/effect.rs` (the module: the 200-entry decal pool at 0x3496e0 [M]; the eviction test `the_201st_decal_evicts_the_first`) |
| 0x1cf290 | `Effect_RicochetProb` | ported | `crates/nf-game/src/effect.rs` (the per-material probability record + `frand_half`); pinned by `ricochet_draws_once_and_reflects` (826) |
| 0x1cf300 | `Effect_UnLink` | ported | `crates/nf-game/src/effect.rs` `free_decal` (170; the node back to the free list) |
| 0x1cf360 | `Effect_RemoveSObjs` | ported | `crates/nf-game/src/effect.rs` `follow_hosts` (206; the deleted-host frees its holes test at 758) |
| 0x1cf400 | `Effect_Body` | ported | `crates/nf-game/src/death.rs` (the corpse contact's surface sound from `EffectInfo+0x18` with `Sound_ModAlertness(0.5)` [M 0x1de220]; the material-0 guard [M 0x1ce688]) |

Counts: **31 fully ported, 1 ported-with-a-gap-slice** (`Bullet_heatseekable`: 3 of its 4
class arms on the branch — the class-3 Bond arm is the remaining slice), **0 not reached,
1 missing slice** (details in the row and the notes).

Notes:

- The family's port files: `bullet.rs` (flight/collision/spread/fuse), `casing.rs`
  (the casings), `explode.rs` (the blast entity), `effect.rs` (the decal/gas/debris/
  ricochet pool), `gunimp.rs` (the mounted gun), with the per-tick folds in `drone.rs`
  (the bullet pass, the trails, the homing, the gunimp tick) and the render-side folds in
  `nf-view/src/scene.rs` (the footsteps) — the same split the drones map documents.
- **1 gap found and fixed on this branch**: the sweep + the weapon-record probe
  (`homing_record_probe.rs` — the 0x20000 homing carriers are 0x2e/0x30/0x45/0x6d [L
  ee_start_05.bin @0x2bf150]) and the copter-vs-pickup sweep (`copter_homing_probe.rs` —
  0700000b copter #148 + the 0x2e pickup #15d; 07000012 copter #707 + pickups #2aa/#2f0
  [D]) proved `Bullet_heatseekable`'s class-0 arm REACHED on the disc while the port's
  homing scan knew only drones. Fixed in `drone.rs` (the PlayerBullet homing block, own
  commit); the live red/green harness lands `#[ignore]`d — the give path refuses the 0x2e
  equip on 07000012 (the fp-model gate), so the run itself is the next round's first item.
  **One slice remains missing**: the class-3 arm (Bond as a homing target) in SP — the
  copter's own 0x6d rounds at Bond reach it [M copter.rs:147], the port's SP scan cannot
  name Bond; the MP fold has it (`mp/combat.rs:462-489`).
- The two closest-to-missing rows were `Effect_Player` and `Effect_Body`, whose port homes
  are OUTSIDE the family's own files (`nf-view/src/scene.rs`'s step path and `death.rs`'s
  corpse contact); both are real code with [M]-cited arms, not comments (the independent
  audit confirmed them).
- The two documented [S] folds in the family (not gaps, port choices with the game rule
  cited): `Effect_Player`'s foot decal sits at the body's ground point instead of the foot
  bone (the port does not build first-person Bond's bone matrices), and
  `Bullet_casing_update`'s PC-only `Casings::set_keep` rest option (0 = the game's exact
  behaviour).
