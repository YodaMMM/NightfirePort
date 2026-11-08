# Sound coverage: every ACTION.ELF sound call vs the port

Audit (2026-10-15). Every call site of `Sound_Play3D`, `Sound_Play`,
`Music_Event` and the ambient/loop starters in ACTION.ELF, with the port's status. The site list
is mechanical: a `jal`-scan of the ELF's .text (a `jal`-scan script, not committed) cross-checked against the decompile's call lines (`work/decomp/ACTION.ELF`).
**195 direct call sites** in 46+28+28 caller files. The decompiler scrambles the float-taking
prototypes (`Sound_Play3D(uint id, VECTOR* pos, float, float, float, short, uint, int)` prints
the floats first), so every id below was read from the **disassembly's `$a0`** / the decomp call
line's id slot; `Sound_Play3D`'s constant pattern `(0x42c80000,0xbf800000,0xbf800000,ID,pos,0,0,0)`
is 100.0f/−1.0f/−1.0f/ID [M dis].

Statuses: **PORTED** (the port plays it — cite), **MISSING** (no port path), **N/A** (MP/driving/
front-end only). Ported-this-pass items link the test (`crates/nf-game/tests/sound_coverage_live.rs`).

## Re-audit (2026-11-07)

A fresh `jal` scan of ACTION.ELF: 25 `Sound_Play`, 74 `Sound_Play3D`, 29 `Music_Event` sites (128),
plus the wrappers: 10 `NDrone2_PlaySFX`, 45 speech, 3 `BOT_soundEffect`, 25 `MPSound_Play`. The decompile
splits many `Sound_Play3D` calls across two lines, so a one-line grep sees 27 of the 74. This section
overrides the rows below where they differ.

Closed in this pass:

| Site | What was missing | On-foot levels reached |
|---|---|---|
| `UpdateMFX_02MayhewA` 0x111ed8, `02MayhewD` 0x112bf8, `03TowerA` 0x113558, `03TowerB` 0x1137b0, `PowerStationA1` 0x113c18, `PowerStationA2` 0x113fc0 | the level half behind the shared exit block; every `Music_Event` these levels post died on the board, so the music stayed on the start section through any fight | 6 (01, 04, 09, 0a, 0c, 0d) |
| `UpdateMusicalEvents` 0x114e70, the `MusicVars` writes | e[0xa] → count 1 (zone-10 rooms: 01, 06, 07, 12), the -99 row on `Music_Event(2, 2)` (triggers on 01, 03), the row re-seat when the bonus runs out | 5 |
| `DroneFunc_CheckAlarmRaised` 0x1453a8:34-42 | ch 0x61 posts on every level, not just 05/06 | not counted; mostly masked, the same channel fails the mission |
| `HUD_UpdateStatusPane` 0x19c310:221,233, `Hint_Update` 0x1b3380:26, `Lock_Update` 0x1ae3d8:368,374 | played 3-D at a fixed point; the game's calls are 2-D `Sound_Play`, so a moving player panned and faded them | all with a banner, hint or scan lock |

Rows below that were stale and are ported: `Player_Update` :121 (the cel zone), `SS_Update` :86,
`SpaceLaser_Update`, `GunImp_Update`, `Break_Kill`, `SSys_Msg` (the countdown wiring landed separately), `Copter_Update` / `GT_Update` music.

Still without a port counterpart:

| Site | Why it stays | Reach |
|---|---|---|
| op 4 kind 6 → `Effect_Create(…, 0xc0)` → `Effect_Body` 0x1cf400:10 | the guards' body-on-the-ground sound (EffectInfo+0x18, half alertness); only Bond's death plays it. A port exists but is held back: headless walks never free EE pool nodes, so the extra nodes move the point where the pool fills, and the 07000001 00-agent walk dies (tick 7682). With an unbounded pool the walk is identical with and without the sound. It lands once the harness frees nodes the way the app does | 20, every level with guards (30 death scripts) |
| op 4 kind 7, the anim-driven `Bullet_init` 0x12c5a0:273 with the weapon's +0x60 id | 4 commands in one script, 0x06000858 | 07000016 only; what plays that script is not traced |
| `NDrone2_DSTATE_Interogate` 0x161a40:216,223,240 (0x49, 0x4D, 0x4E) | the 07000005 arm: `DroneFunc_FirstSightState` 0x147c48 returns 0x3C on 07000007 and 07000041 only | 0 |
| `Light_Update` 0x1cafb8:27 | no 0x4E placement on any level has a flicker period, the arm's gate | 0 |
| `Hurt_Update` 0x17e5d0:120 | param 3 is 0 or 0xffff on every placement, all levels [D] | 0 |
| `Lock_Activate` :27-32, `Lock_Unlock` :18, `Lock_Update` :282 | 0xffff on every lock [D] | 0 |
| `Sub_*`, `MiniSubState_Attack` | no 0x36/0x37 placements on any 07* archive | 0 |
| `Car_*` | MP arenas only; the drive and fire path is not wired | 0 on foot |

## Summary

| Class | Sites | Status |
|---|---|---|
| EE pool + map ambients + script sounds + music engine (`Sound_UpdateSounds`, `HandleMapSoundAllocation`, `Sound_ReqestPlaySfx`, `Script_*`, `UpdateMusicalEvents`) | 30 | PORTED (`nf_game::sound`, `sound.rs` MapSounds, `music.rs`) |
| Player on-foot (jump/pain/hit/air/weapon-switch/death) | 40 | PORTED |
| Weapons/bullets (fire, impacts, fly-by, casings) | 25 | PORTED except 0x28D (class-0x51 bullet-end) |
| World objects (doors, searchlights, pickups, hints, sound triggers, locks) | 30 | PORTED except the picklock beeps 0x5DB/0x5DA + activate sounds, `Hurt` loop |
| Drones/guards (foley, pain, gunfire, deaths, seen-music) | 25 | PORTED except class-specific extras (below) |
| Security cameras (`Sensor_Update`) | 7 | PORTED (2026-10-31): `sensor.rs` plays the 0x4AA hum loop, the 0x4DD servo and the 0x4A9 alert beeps; the starts feed AI hearing |
| SpaceLaser (0x50), SS (0x25), Break (0x20) classes | 10 | Update 2026-10-22: all three classes are ported (`space_laser.rs`, `ss.rs`, `breakable.rs` — the Break kill sounds 0x5F2/0x5F3 are in breakable.rs); per-id coverage not re-checked (unverified, 2026-10-22) |
| Sky thunder (`View_DrawSky`) | 2 | PORTED (2026-10-31): `Sky::from_placements` + `Drones::sky_frame` play 0x5D/0x5E; the draws use their own stream (the flash swap stays G-019) |
| Front-end menus (`P_*_Handler`, `Menu_PlaySound`) | 40 | N/A (front-end) |
| Vehicles/driving/MP (`Car_*`, `Copter_*`, `Sub_*`, `GT_*`, `MPSound_*`) | 22 | N/A |

**Ported this pass (5 ids, 4 hooks, each with an NF_ISO test):**
0x10E grenade-ammo launch, 0xA3 jump launch + 0xA3/0x5FB landing, 0x249+0x5DD laser
burn-through, 0x148 special-death cry.

## The table

### Player on foot

| Caller @ addr | Target | Id | Trigger | Status |
|---|---|---|---|---|
| `Player_HandleJump` 0x1916b0:111 | Sound_Play | 0x1DB | level 0x0700007 & ch 0x48 off: the refused jump (action-7 edge, grounded-history ≠ 0, head-hit flag clear; returns without launching) | PORTED `player.rs` `jump_locked`, `scene.rs` + `the_07_party_refuses_the_jump_until_channel_48` |
| `Player_HandleJump` 0x1916b0:184 | Sound_Play3D | 0xA3 | jump press: the launch | PORTED `player.rs:1673-1694` `sfx_out` → `scene.rs:3461` + test |
| `Player_HandleJump` 0x1916b0:195-198 | Sound_Play3D | 0x5FB / 0xA3 | jump landing (flag 8, feet on ground; a head hit, flag 0x20, is silent [M :68-71]); 0x5FB when the feet ray's first hit byte `- 0xd < 2` (0xD/0xE, water), else 0xA3 | PORTED `player.rs` landing arm + `jump_landing_sound_live` |
| `Player_HandlePain` 0x1902c8:113 | Sound_Play3D | 0x88 | pain (1-in-4 roll; type-7 with cheats on skips) | PORTED `bullet.rs:356` roll → `drone.rs:21104`, `scene.rs:3522/3678` |
| `Player_DealWithObjHit` 0x194458:231-255 | Sound_Play/3D | 0x600, 0x412, 0x171 | Bond hit by an object (the 0x180 gate; 0x600 is the MP head-zone arm) | PORTED `drone.rs:21108-21123` |
| `Player_CheckForDeath` 0x190810:41,88 | Music_Event / Sound_Play3D | 6 / 0x89 | Bond dies: the music pulse + the death grunt | PORTED `music.rs` event 6, `death.rs:223-241` |
| `Player_MonitorAir` 0x191a70:106-141 | Sound_Play/3D/Stop | 0x60B, 0x60C, 0x27 | the dive loop 0x27 on the under-entry edge, the gasp 0x60C + stop on the surfaced edge, 0x60B + pain every 30 ticks at air < 1.0 | PORTED `scene.rs:3565-3580` + `3525-3529` (swim latch `sound.rs:55-58`) |
| `Player_ChangeSubState` 0x190630:83-85 / `Player_Init` 0x1910f8:41-43 | Sound_StopAllWithId | 0x12, 0x3D, 0xF0 | substate change into [1..3]∪[5..0x10] (NOT 0, NOT 4-crouch; Player_Init unconditional) stops the loops | PORTED `player.rs change_substate` (every substate write routes through it) + the scene drain `scene.rs:3480` — the trio queues on the NEW substate in [1..3]∪[5..0x10] and `Sound_StopAllWithId` drains into the pool next to the sfx_out drain; `Player_Init`'s unconditional trio is the no-op it is at a fresh pool (sound-sweep-leftovers 2026-11-06) |
| `Player_SetWeaponAnimObj` 0x1a2e38:227-295 | Sound_Play | 0x283/0x280 holster, 0x281/0x282 draw | the weapon-change machine: gadget id in [0x4A,0x5E] → 0x280/0x281, else 0x283/0x282; keyed on the old (holster) / new (draw) record, silent when its anim script (+0xC4/+0xC0) is absent | PORTED `scene.rs:4297/4328/4374/4441` (the 4374 site misses the no-draw-script silence) |
| `Player_WeaponChange` 0x1a6f80:128 | Sound_Play | 0x1DB | the change refused: the party lock's mode-2 cycle (`Player_Weapon` 0x1a24c8:265-275 reads the cycle actions into `Player_WeaponChange(±1,2)` while 07000007's ch 0x48 holds — the only mode-2 call sites) with the held weapon == 1. The byte is NOT a level_tag byte: the reader dereferences `obj+0xDC` — the AnimObject — whose +0x62 is the held weapon id [M obj-tag.md +0xDC; animobject.md +0x62]; id 1 = the fists, the lock's only weapon (`Player_InitWeapon` 0x1a7ef8:184 `anim[0x63] = 1` on case 0x07000007). The mode-2 cycle is bounded to [0,2) (disasm 001a71ec-001a71f0), so the weapon stays; `Sound_Play(100.0, 0x1db, 0, 0)` per press (disasm 001a7200-001a7214) | PORTED `scene.rs` mode-2 route + `the_lock_refuses_the_cycle_with_the_click` (interact_07000007_live) |
| `Player_WeaponInitBullet` 0x1a4bf0:262 | Sound_Play | 0xEC | per bullet of a base-0x54 record (the Micro-Camera's shutter) | PORTED `drone.rs:14102-14109` + test |
| `Player_Zoom` 0x1a1a70:31-84 / `Player_Weapon` 0x1a24c8:164 | Sound_Stop | [BL+0x898] | scope/weapon exit stops the gadget loop — DEAD CODE in the game: +0x898 is only ever zeroed (ELF-wide immediate scan over ACTION.ELF/SLUS/DRIVING), the guard never passes | PORTED as a documented deviation `scene.rs:4288/4303` (stops the old weapon's loop ids at the switch — matching it literally would kill loop sounds) |
| `Player_Creep` 0x1969d0:85 / `Player_Wire` 0x197658:107 | Music_Event | 0xD | every tick of creep (substate 7) / wire (6) | PORTED `scene.rs:3497-3506` (the port's `on_wire` adds SS_ZIPLINE — substate 0xf is silent in the game; dead in shipped data, both wires type 0) |
| `Player_Update` 0x193600:121 | Music_Event | `level_tag+0xA1` byte | the level's constant music event | MISSING (byte unparsed; low impact) |
| `HUD_UpdateStatusPane` 0x19c310:221-233 | Sound_Play | PlayMe1st=**0x5FC** / 0x1D4, PlayMe2nd=**0x1D5** / 1 (id table [M dis 0x19c618-0x19c664]; [L 0x30ccbc=0x5fc]) | the objective banner: the flash jingle / the new-objective sting | PORTED `scene.rs:6416-6434` (1532/469/468) |

### Weapons and bullets

| Caller @ addr | Target | Id | Trigger | Status |
|---|---|---|---|---|
| `Bullet_init` 0x12c5a0:273 | Sound_Play3D | `param_7` | the caller-passed fire sound — NOTE: the guards (`DroneWeap_FireWeapon` 0x178f68) and the player's `Player_WeaponInitBullet` pass 0xffff and are silent here; their real fire ids ride FireWeapon's own `Sound_Play3D(+0x60)` (0x179490) and the fire script's op-1 (0x1c1b38:320) — only the anim-script fire op feeds a real id, unused in ported SP paths | PORTED `drone.rs:11283-11287` (the guards' real fire event) + `scene.rs:4918` (op-1) |
| `Bullet_init` 0x12c5a0:277-282 | Sound_Play3D | **0x10E** | ammo type (+0x90) 0x0E = the grenade pool: the launch whoosh | **PORTED this pass** `drone.rs:14098` + test |
| `Bullet_init` 0x12c5a0:286-290 | Sound_Play3D | 0x5EF | weapon class (+2) == 0x45 — the record is **Oddjob's hat** (base 0x45, +0x60 = 0x5ef, looped), not a grenade kin (grenades base to 0x2c/0x2e/0x30/0x6d) | PORTED `scene.rs` throw arm (loop_handle field on the port's Grenade = BU+0xF0; `Bullet_Delete`'s stop at the Drones retain) — the +2 == 0x45 spawn plays the looped 0x5ef at the spawn; the MP combat has no audio sink so its hat throws stay silent (recorded gap) (sound-sweep-leftovers 2026-11-06) |
| `Bullet_CollisionHandler` 0x12ce80:316-432 | Sound_Play3D | 0x4A7, 0x446, 0x51C, surface id / 0x15, 0x13 | bullet impacts: 0x4A7 the 0x100-family stick clang (satchel/hat), 0x446 the 0x80000 stick (tripbomb), 0x51C the even-count bounce block, 0x15/0x13 the 0x2000/0x20000 first bounces; the surface-id leg is callee `Effect_Bullet` 0x1ce8a8:86-90 | PORTED `drone.rs:21312, 21369-21372, 21008-21010` + `SurfaceSounds` |
| `Bullet_DoTrails` 0x12d890:250 | Sound_Play3D | 0x1DC | the fly-by (weapon +0x6c & 0x40000, not the player's own shot, closest-point SqDist < 9, 1-in-5 roll) | PORTED `bullet.rs:264-268` → `drone.rs:21128` |
| `Bullet_DoTrails` 0x12d890:292,411 | Sound_Play3D | 0x16, 0x14 | 0x16 the stun-grenade flash pop (the 0x2000 fuse expiry), 0x14 the smoke canister pop (the 0x20000 first expiry, re-arms +0xfc) — neither is the frag blast | PORTED `drone.rs:21429, 21461-21466` |
| `Bullet_DoTrails` 0x12d890:442 | Sound_Play3D | 0x447 | the satchel-charge (base 0x3b) countdown beep, on the quarter-second digit change at the (qs&3)/sec<10/sec<5 cadence — there is no smoke-trail sound in DoTrails | PORTED `drone.rs:11657-11661` → `21324-21327` (label fixed 2026-11-06 sweep) |
| `Bullet_DoTrails` 0x12d890:473 | Sound_Play3D | **0x5DD** | the satchel charge's burn life ends under the Lazer-Watch beam (the charge's +0x102 counter drains on class-5 contacts of 0x4E/0x4F rounds) | **PORTED this pass** `drone.rs:15464-15495` + tests (the old label read the beam's life backwards — the ids and event were right) |
| `Bullet_update` 0x12e7b0:104,290,327 | SetPosition / Stop / Play3D 0x27C | — | :104 the in-flight loop follows the grenade; :290 the state-3 stop on landing; :327 the 0x80000 arm-clock expiry creates the beam obj and plays 0x27C — the **tripbomb's laser-arm sound**, not a bounce | PORTED `drone.rs:14809-14812, 15969, 21318-21319` (`grenade.rs:308-315` raises the Arm) |
| `Bullet_Delete` 0x12e660:14,53 | Stop / Play3D 0x28D | — / 0x28D | the loop stop; the claw bolt (+0x70 & 0x40, player-owned, stuck) dying on a class-'Q' (0x51) grapple point plays 0x28D before `Player_SetGrapplePoint` (latch ⇔ sound) | PORTED `scene.rs:3193-3205` `GRAPPLE_LATCH_SFX` + `gadget_grapple_live` (the port fires at the fire-press latch — no bolt flight) |
| `Bullet_casing_update` 0x12faa0:97 | Sound_Play3D | 0x1FD-0x204 by material | the casing lands (once; state ∉ {2,3}, y ≤ the eject-set landing plane) | PORTED `casing.rs:276-282, 320-331` |
| `Break_Kill` 0x12b970:69-75 | Sound_Play3D | 0x5F2 (41% roll) / 0x5F3 / 0x2E | a breakable (class 0x20) dies | MISSING — the Break class is unported (`parsemap` case 0x20 → `Break_Create`; no port list) |

### World objects

| Caller @ addr | Target | Id | Trigger | Status |
|---|---|---|---|---|
| `Door_SetState` 0x135618:11-18 / `Door_Activate` 0x135be0:84 | Sound_Play3D | DOOR data +0x60/+0x62/+0x64 | entering state 8 → +0x60, entering 0x10 → +0x62 (both edge-gated); the locked-hit +0x64 (lock channel off & state & 0x12) and the one-way approach gate (dot = (door−activator)·facing, mode 1 dot<0 / mode 2 dot>0) | PORTED `door.rs:461-462, 696-708` (`SoundEvent` outbox) — the one-way dot was mirrored until this sweep (fixed + `one_way_door_refuses_from_the_game_side`) |
| `Lock_Update` 0x1ae3d8:92 | Sound_Play3D | **0x249** | the laser burn drops +0x105 below 4 → `Lock_Unlock` | **PORTED this pass** `lock.rs:172-181` → `drone.rs:15423-15434` + `sound_coverage_live.rs:161` |
| `Lock_Update` 0x1ae3d8:282 | Sound_Play3D | (unlock click) | the picklock sequence's unlock | MISSING — the picklock minigame is unported (lock.rs covers the burn only) |
| `Lock_Update` 0x1ae3d8:368,374 | Sound_Play | 0x5DB, 0x5DA | the picklock ticks / the success | PORTED `lock.rs` `pending_sounds` + `lock_sounds_live.rs` (was MISSING when this table was written) |
| `Lock_Activate` 0x1aed00:27-32 / `Lock_Unlock` 0x1b0650:18 | Sound_Play3D | LOCK words [2]/[3] = placement params 3/4 (0 → 0xFFFF) | the key-lock rattle / the unlock | DEAD DATA: 0xFFFF on every disc 0x2E placement [D]; `Lock::key_channels` keeps them, the keypad solve plays word [2] |
| `Searchlight_Update` 0x1cb900:401,455 | Sound_Play3D | 0xE5, 0xE6 | 0xE5 the state-3 off→on relay clunk (the on-channel byte 0x34); 0xE6 the >0-damage bullet contact glass break | PORTED `searchlight.rs:377-385, 605-623` |
| `Sensor_Update` 0x1af618:103-255 | Sound_Play3D | 0x4AA (activate), 0x4A9 (accelerating beep: 0.5→0.25→0.125 s), 0x4DD (sweep midpoint + wrap) | the security camera (placement 0x28) | PORTED `sensor.rs` (`SFX_HUM`/`SFX_SERVO`/`SFX_BEEP`) + `sensor_live` |
| `Pickup_Handler` 0x18e650:336 | Sound_Play3D | PICKUPINFO +0x2A (kind overrides 0xF5/0x141/0x14C) | the pickup taken (past the equip/ammo/armour refusals; 0xffff = Pickup_Create's no-sound sentinel) | PORTED `drone.rs:17286-17294` |
| `Hint_Update` 0x1b3380:26 | Sound_Play | HINT data [0] | the hint volume fires (first player-entry, ON-channel set/OFF-channel clear, one-shot by self-delete) | PORTED `scene.rs:3383-3396` (2-D in the game, the port plays 3-D at Bond's head) |
| `SoundTrigger_Update` 0x1b2790:30-59 | Play3D/Play/Stop | TRIGGER data `puVar2[3]`, loop stop | the channel-gated emitter | PORTED `sound.rs` `SoundTriggers::tick` |
| `MusicTrigger_Update` 0x1b31a0:24 | Music_Event | TRIGGER data [0] | the music trigger: a channel-gated 30-tick repeater (`Music_Event(id, 2)` per countdown tick, p1 channel deletes), not a volume test | PORTED `worldobj.rs:514-529, 566-574` (label + location fixed 2026-11-06 sweep) |
| `Hurt_Update` 0x17e5d0:120 | Sound_Play3D | HURT data +0x1A | the hurt volume's activation sound | DEAD DATA — `hurt.rs` `sound` (param 3) is 0xFFFF on every surveyed placement [D] |
| `Light_Update` 0x1cafb8:27 | Sound_Play3D | LIGHT data +0x2C | the flicker counter wraps through 0xF000 | MISSING in code but **dead data**: the placement call sites never write +0x2C (`light.rs:145-148`) |
| `AnimObjectUpdate` 0x1befd8:205 | Sound_Play3D | 0x286 | the anim object's event — the ladder rung step: Bond (obj+0xff==3), the stride-accumulator footstep bit (anim+0x60 & 4), substate 1 climb | PORTED `scene.rs:3598-3630` |
| `AnimProcessScriptCmds` 0x1c1b38:320,388 | Play3D / StopAllWithId | the anim script's sound cmd | the animation-script sounds | PORTED (`anim_sounds`, `d.sfx` paths) |
| `RB_RestingContact(Rot)` 0x1d7168:111 / 0x1d73a0:121 | Sound_Play3D | RIGIDBODY+0x158 (level_tag+0x3c, 0 → 0xFFFF) | a class-5 contact on a rigid body | PORTED `rigid_body.rs:722-729, 874-899` (id at :1203; drain `drone.rs:19649`), 0xFFFF on every disc 0xD8 placement [D] |
| `SS_Update` 0x1a9948:86 | Sound_Play3D | SS data +0x14/+0x26 | the SS (0x25) playback state 0x40 | MISSING — the port's `ss.rs` runs activation only, not the playhead states |
| `SpaceLaser_Update` 0x1aa3f8:82-194 | Play 0x5DE / Play3D 0x46A / Stops | — | the laser grid: the trip hum (2-D) and the beam hit | MISSING — class 0x50 unported |
| `View_DrawSky` 0x1e5ad0:104-107 | Sound_Play | 0x5D (`LightningType==0`) / 0x5E | the sky slot's thunder countdown hits 1 | PORTED `drone.rs` `sky_frame` (0700000a/0b slot 3 [D]) + `sensor_live` thunder test |
| `HandleMapSoundAllocation` 0x1dd118:69-77 | Play3D / Stop | the map's ambient list | the listener inside R starts / 1.2·R² stops | PORTED `sound.rs` `MapSounds` |
| `GunImp_Update` 0x1320d8:263-333 / `GunImp_Deactivate` 0x132c18:62 | Play3D 0x5E4/0x5E5/0x5E6 + the TurretGuns fire id / Stop | — | the gun implacement's servo/fire | MISSING (class unported; SP-visible only in 0700000C/0D) |
| `GT_Update` 0x180498:291-518 | Music_Event 2 / Play3D fire id, 0x10A | — | the gun turret | N/A (the port has no GT list; the drivable sections only) |

### Drones, guards, music, script

| Caller @ addr | Target | Id | Trigger | Status |
|---|---|---|---|---|
| `NDrone2_PlaySFX` 0x141848:121 | Sound_Play3D | `Drone_SfxInfo` entry | the state arms' foley one-shots | PORTED (`foley_reqs` drain, fix-room-sound) |
| `NDrone2_DoHitEffects` 0x1458e0:108-111 | Sound_Play3D | 0x601 (HITDATA+0x52==5) / 0x413 | the guard body hit (damage > 0.5, the firing weapon's +0x70 & 0x180 clear) | PORTED `drone.rs:15571-15576` |
| `NDrone2_DSTATE_SpecialDeath_Anim` 0x163c68 case 5 | Sound_Play3D | **0x148** | the special death of script 0x0600021F: case 5 = the ANIM_DONE message (the script's completion), at the PLAYER's position (`glb_players+0x30`), `glb_players != 0` | **REPAIRED this sweep** — was queued on state ENTER at the drone (right id, wrong event/pos); now `drone.rs:4745-4756` (ANIM_DONE arm) + test (`special_death_0x0600021f_plays_0x148`) |
| `DroneWeap_FireWeapon` 0x178f68:277 | Sound_Play3D | the weapon's fire sound | the guard fires (weapon_data +0x60, skip 0x5EF; the `Rand(500)−250` pitch draw is not consumed — fix-guard-weapons3) | PORTED `drone.rs:11283-11287` |
| `BOT_handlePain` 0x125ed8 / `BOT_soundEffect` 0x126618 | Playing_Ref / Stop_Ref | [DCVars+0x73C] | the pain-grunt loop ref | PORTED (`pain_grunt`, fix-room-sound) |
| `DroneVision_HaveOpponentSight` 0x175ef0:90 / `NDrone2_SeenAndAttacking` 0x145150:68 / `DroneFunc_CheckAlarmRaised` 0x1453a8:15-42 / `Drone_EnableAll` 0x1383e8:28 / `Drone_InitComms` 0x139520:195 / `NDrone2_DSTATE_HostageSaved` 0x15ccf0:21 | Music_Event | 0xF seen, 0x2 combat, 0xE comms, 0x10 hostage | the combat-music state machine | PORTED `music.rs` (`seen_and_attacking`, `attack_gate`, events); the 0x10 hostage cue's outbox drain repaired this sweep (`drone.rs:20585-20592` + `hostage_saved_posts_the_0x10_music_event`); `EnableAll`'s 0xF rides the 0x600001b script (not live on ported levels) |
| `Mission_Update` 0x182c60:25-36 / `Mission_MonitorObjectives` 0x1827d8:122 / `Player_CheckForDeath` 0x190810:41 | Music_Event | 8 objective, 7 fail, 6 death | the mission music pulses | PORTED `drone.rs:7108-7110`, `music.rs` |
| `Trigger_Activate` 0x1b2318:109 | Music_Event | 3 | the exit trigger's end sting | PORTED `drone.rs:19585-19592` (LOAD_LEVEL) — deviation: the port also fires (3,1) on the script 0x12 LoadLevel path (`drone.rs:17049`) where the game's `Script_EventHandler` posts nothing |
| `Copter_Update` 0x17dbb8:189 / `Sub_Update` 0x1ad200:116 / `Sub_DeployTorpedoes` 0x1acf68:74 / `MiniSubState_Attack` 0x181b00:216 / `GT_Update` 0x180498:291 | Music_Event | 2 | the vehicle combat music | N/A (driving) |
| `Script_SoundStart` 0x1d8820:36-40 / `Script_KillStream` 0x1d9860:83 / `Script_Interp` 0x1d9a08:224-233 | Play/Play3D, Stop, SetVolume/SetPosition | the script stream's id | the NIS/script sounds (follow, fade, stop) | PORTED (`script_sounds`, sound.md) |
| `Script_CameraStart` 0x1d9140:48 / `Script_FFwd` 0x1dac78:55 / `Script_KillStream` 0x1d9860:74 / `Script_Play` 0x1db588:40 | Music_Event | 0xB, 5, 0xC, 6 | the NIS music markers | PORTED `music.rs` (`nis_state`) |
| `Sound_UpdateSounds` 0x1dda70:95-98 | `Sound_ReqestPlaySfx` / `Sound_DoSubtitle` | — | the pool's state machine / the subtitle hook | PORTED (`sound.rs` the pool; subtitles are presentation) |
| `Sound_Ready` 0x1dceb8:108-109 | Music_Event | 0, 1 | the sound init per GameState | PORTED `music.rs` |
| `UpdateMusicalEvents` 0x114e70 | SFXStartMusic/SFXStopMusic | `MusicNowPlaying` | the IOP music engine | PORTED `nf-audio` + `music.rs` |
| `MPSound_Play` 0x18bee0:6 | Sound_Play | `param_1` | the MP wrapper | N/A (MP) |
| `SSys_Msg` 0x1ad878:12-43 | Sound_Play | `Laser2_135`/`Cam2_133` tables | the camera/laser alarm HUD messages | MISSING (the alarm message sounds; the tables are static data) |

### Front-end (N/A for the on-foot Sim)

`Menu_PlaySound` 0x1f3278:46 (0x1D9) and the twelve `P_*_Handler` screens' SFXStart(0x1D7 move,
0x470 select) × 2 each + `P_CREDITS` SFXStartMusic/SFXStopMusic(0x27) — the front-end UI sounds.
The port's front-end (`nf-view/menu.rs`) plays none of these; out of scope for the on-foot Sim.

## Live checks (open)

2. **Thunder**: on 07000005 (island night, lightning), gdbwatch `Sound_Play` 0x1dd880's `$a0`
   through a storm cycle: expect 0x5D (or 0x5E if `LightningType != 0`) each time a sky slot's
   +0x48 countdown leaves 1. The port's `sky.rs::tick_slot` already computes the same event —
   wiring it needs the render-side call point.
3. **Objective jingles**: complete an objective on any level; the game plays 0x1D5 at the notice
   and 0x5FC on the banner's flash (0x1D4/1 on the alternate-id levels). The port plays
   1532/469/468 — confirm no audible difference.

## nf-view spots

Not visual — the four tests in `crates/nf-game/tests/sound_coverage_live.rs` are the checks
(NF_ISO-gated, silent).
