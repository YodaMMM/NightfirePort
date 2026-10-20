# mp-remote-fx — what you see and hear of the other players and the bots

Evidence tags: [M] decompile (ACTION.ELF, work/decomp), [D] disc/ELF data, [L] live (port windows), [S] port
choice. Code: `crates/nf-view/src/mpfx.rs` (`MpShot`, `MpFx`, `Sim::flush_mp_fx`, `Sim::mp_body_death`), the
event pushes in `Sim::step_net_combat` / `Sim::step_mp_bots` (scene.rs), the draw arms in `frame_state_for`
(scene.rs: the MP-body flash attachments, the `MP_TRACER_BASE` trail slots, the death pose).

## Shape

The combat code records every resolved shot as an `MpShot` (shooter, bot or human, weapon, origin, direction,
contact point, what it hit: `Miss` / `World{normal, material}` / `Human(slot)` / `Bot(slot)` / `Site`, damage).
Both producers run inside the shared lockstep tick, so every machine records the same events. The humans' shots
come from `step_net_combat` (the hitscan of 700v6); the bots' from `step_mp_bots`. A later projectile combat
(agent 10's work) only has to push the same `MpShot` at its contact.

`Sim::flush_mp_fx` turns the events into effects at the end of the tick — online after `step_net_combat`,
offline at the end of `step_bond`. It writes only draw/audio state: `drones.effects` (decals, debris),
`drones.gas`, `drones.fx` (impact emitters), `drones.lights`, `drones.audio`, `drones.health.flash` / `dir_bits`
and `Sim::mp_fx`. None of these are in `Sim::net_state_hash`. No draw comes from the lockstep stream
(`drones.rng`): each shot's effect draws come from a generator seeded by (frame, shooter, shot index), so every
machine draws the same debris for the same shot [S]. The local slot's own shots are skipped, because its own
fire pipeline (`step_bond`) already draws and plays them on its machine. The test
`mp_remote_fx_live::remote_shots_show_and_sound_without_touching_the_hash` runs a third machine with the pass
disabled, and all three hashes agree on every tick.

## Per effect

| Effect | Bots (drones in the game) | Human players |
|---|---|---|
| Muzzle flash | `DroneWeap_FireWeapon` 0x178f68's flash: mesh `weapon_data+0x4C` on the weapon datum id + 1 (the muzzle), 2 ticks per shot [M; the guards' path]. The bots fire through it: `NDrone2_DSTATE_BotAttackFire` 0x16f5e8 → `DroneWeap_Fire` [M]. | The same third-person flash on the body's gun [S]. The PS2 draws a player's flash on the first-person view model only (`Draw_MuzzleFlash` 0x1a3fe0), so in split screen the other player saw just the light. The port shows the flash so the opponent can see the shot. |
| Muzzle light | `Light_Create(7.0, 1.0, muzzle, rgb +0x56/+0x55/+0x54, life 0, kind 1)` [M 0x178f68:189] | `Light_Create(radius +0x58, 2.0, …)` under the +0xDC/+0x48 gate; MP skips the smoke puff [M `Player_MuzzleFlash` 0x1a4298:26-55] |
| Shot sound | `Sound_Play3D(weapon_data+0x60)` at the body; 0x5EF is skipped [M 0x178f68:275] | The held weapon's fire script op-1 ids (`AnimProcessScriptCmds` 0x1c1b38 op 1 → `Sound_Play3D`) [M], with the per-weapon remap (`remap_op1_sound`). Each id plays at the shooter's head and falls due at its script frame (1 frame per tick [S]). |
| Bullet trail | `GUN_BULLET_TRAIL` 0x2000123 when `weapon_data+0x6C & 0x1010` and the owner is class 2 (a drone) [M `Bullet_update` 0x12e7b0:140-152] | Only when `+0x6C & 0x1000` [M same lines]. The P2K has neither bit, so it draws no trail. |
| World impact | `Effects::bullet_impact` (decal / gas / debris / emitter of the surface's `EffectInfo`), plus the impact sound `EffectInfo+0x14`. The sound needs `+0x70 & 2` and is never played for the laser kind (`+0x6C & 0x100`) [M `Bullet_CollisionHandler` 0x12ce80:186-192]. Ricochets are not followed (there is no remote bullet object) [S]. | same |
| Victim flesh puff | A bot victim gets the drone's `NDrone2_DealWithObjHit` 0x13fff0 case 5 puff: 0x20008bc (3.0, 6.0, life 0x3c), or 0x200026e for 0x32/0x33 [M :235-256]. This happens on every machine, including for the local player's own hits on bots. | No puff on a human victim. A body hit carries material 0 [M `Collide_Jointy` 0x1ec180:125/173], so `Effect_Create` returns [M 0x1ce688:52]. |
| Local hit feedback | — | `Player_DealWithObjHit` 0x194458: the flash +0x968 += min(d·21.333, 128) (capped 255), the direction bits +0x967 from the shooter's position (`PlayerHealth::hit_dir_bits`, the `pain_from` rule) [M :201-226], `Sound_Play(0x412)` when `+0x70 & 0x180 == 0` [M :229-236], and the 1-in-4 grunt 0x88 (drawn from the per-shot generator, not the lockstep stream). The health itself stays the hashed `mp_hp`. The Plr_DMod factor is not applied to the flash [S]. |
| Footsteps | `AnimObjectUpdate` 0x1befd8's stride rule: 1.65 units, then the floor surface's `EffectInfo+0x16` at the floor point [M]. Implemented as `step_bond`'s local rule, run per remote body. | same |
| Jump / head-bump sounds | — | The remote body's `Bond::sfx_out` (previously cleared), played at the flush [M `Player_HandleJump` 0x1916b0] |
| Death | The body plays `DEATH_SCRIPT` from the death tick and holds the last frame until the respawn. It has no gun. [S] for the bots: their game death runs the drone states. | Substate 0xd → `DEATH_SCRIPT` [M `Player_CheckForDeath` 0x190810:89-93, `Player_ChangeSubState` 0x190630]. The gun is dropped (`Player_WeaponNone`, the pickup) [M :43-63]. `Sound_Play3D(0x89)` plays at the body [M :88]. |
| Respawn | No effect: `MP_ReSpawn` 0x185368 only re-stands and re-equips the body. It has no sound, gas or script call [M]. | same |

## Checks

- [L 2026-10-18] Two windows on 07000023 (agent11 pair.ps1, ports 28110, both `NF_SILENT=1`, both
  `NF_NET_AUTOFIRE`).
  - Host captures `r2/nf-shots/001`, `004` and `r3/004` show the joiner's body with a flash on its gun and the
    floor lit around it.
  - `r7/nf-shots/002` shows the joiner's bullet hole on the low wall at the impact the log reports
    ((-20.09, 1.23, -2.04)).
  - The `[snd]` lines (the driver's `Cmd::Start`) start 0x403 at the joiner's head, 6.98 units from the listener.
  - The `r9` run (bots 2, `NF_NET_AUTOMOVE`) logs the same bot shot/footstep sound counts on both machines
    (0x42c ×30 per bot, footsteps 0x83/0x10/0x6) and the moving joiner's footsteps 0x11.
- [L] Headless `--mp-match 1,3 --frame N --mp-look …` (offline bots):
  - `h1/b2_168.png`: a bot firing, with the flash, the light and a trail, and the local damage glow.
  - `h1/d3_1090.png` and `d3_1200.png`: a dead bot in its death script, gun gone.
- Tests: `crates/nf-view/tests/mp_remote_fx_live.rs`, both ISO-gated:
  - Three machines, equal hashes every tick.
  - Every remote shot sound starts at the shooter's head and reaches the audio driver.
  - The flash shows 2 ticks per shot.
  - A remote hit raises the glow and the 0x412 sound.

## Open

- A looped op-1 (the held laser/stunner hum) is not played for remote humans, because the script's own stop is not
  run for them.
- The bots' pain/death voice: `BOT_handlePain` 0x125ed8 picks from the drone pain pool with `Rand_Rand`, which is a
  shared-stream draw in the game.
- Ricochet chains of remote rounds are not followed.
- Remote grenades and rockets need projectile combat (agent 10).
- No PCSX2 comparison yet: split-screen MP on the PS2 would show whether the light flickers and how the death
  body ends.
