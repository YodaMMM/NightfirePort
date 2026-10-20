# mp-shot-effects — MP shots present the SP effect set, through the SP code

The user (2026-10-19): "Bullets flying out of guns doesn't match what it's like in single player."
Evidence tags: [M] decompile (ACTION.ELF, work/decomp), [L] live/record, [S] port choice. Code:
`crates/nf-game/src/drone.rs` (`Drones::fire_player`, `Drones::fire_remote_cosmetic`,
`update_player_bullets`, `flush_world_impacts`), `crates/nf-view/src/mpfx.rs` (`Sim::flush_mp_fx` /
`mpfx_shot`), `crates/nf-view/src/scene.rs` (`step_bond`'s `'shot` block, `step_net_combat`).

## The SP pipeline's per-shot effects ([M], the shooter's own view — and today the MP shooter's own view too)

The player fire path is `Player_WeaponFiring` 0x1a5460 → `Player_RoundToFire` 0x1a18e0 →
`Player_WeaponInitBullet` 0x1a4bf0 → `Bullet_init` 0x12c5a0; the muzzle presentation is
`Player_MuzzleFlash` 0x1a4298 (reached via the +0x95b latch through `Player_CollisionHandler`
0x1949a8):

| Effect | Game source |
|---|---|
| Muzzle flash | `Draw_MuzzleFlash` 0x1a3fe0 on the first-person view model, 2 ticks per shot (the +0x95b latch) [M]. The port: `Sim::last_shot`/`shot_tick` latch. |
| Muzzle light | `Player_MuzzleFlash` 0x1a4298: `Light_Create(radius = weapon_data+0x58, intensity 2.0, the view model's datum 0 (4 for the +0x68 & 0x40000 alternating records with BL+0x958 clear), rgb +0x56/+0x55/+0x54, life 0, kind 1)` under the +0xDC/+0x48 gate [M :26-46, :66-89]. |
| Muzzle smoke | `Gas_Init(0.05, 0.1, …, 0x2000039, 7, 0x48403080, life 0x80)` with its `Rand_FRand(2π)` draw, unless weapon_data+0x68 & 0x2000 [M 0x1a4298; the decomp-dropped life arg 0x80 at 0x1a43f8]. The game's own MP mode skips this puff (the MPSettings+0x180 gate, [M :47-55]); the port deliberately matches SP instead (the user's ask) [S]. |
| Bullet objects | `Bullet_init` 0x12c5a0 per pellet (weapon_data+0x18 pellets, the spread draws `Bullet_init` :166-184), the flight `Bullet_update` 0x12e7b0, the ribbon `Swoosh_Create` for +0x6C & 0x10000 (:116-133), the grenade-ammo launch whoosh `Sound_Play3D(0x10e)` (:285-291). |
| Trail / fly-by | `Bullet_DoTrails` 0x12d890: the `GUN_BULLET_TRAIL` 0x2000123 per the +0x6C & 0x1010 gates (:140-152 — a player's rounds trail only with the 0x1000 bit; the P2K has neither bit and draws no trail), the fly-by whiz `Sound_Play3D(0x1dc)` for the +0x6C & 0x40000 records within 3 units on a 1-in-5 roll [M]. |
| Shell casing | `Bullet_init_casing` 0x12fde0 (from `Player_WeaponInitBullet` 0x1a4bf0:306): one shell per shot, ejected after the +0x78 delay at the view model's gun datum (`Bullet_init_casing_ex` 0x12f678:88-99). |
| World impact | `Bullet_CollisionHandler` 0x12ce80 → `Effect_Bullet` 0x1ce8a8: the decal / gas / debris / emitter stack of the surface's `EffectInfo` + the impact sound `EffectInfo+0x14` (the +0x70 & 2 gate, never the laser kind +0x6C & 0x100) [M :186-192], the penetration roll before the effects [M 0x12cfd0], the ricochet. |
| Shot sound / hearing | The fire script's op-1 `Sound_Play3D` (`AnimProcessScriptCmds` 0x1c1b38 op 1), with the per-weapon remap (docs/systems/sound.md §7) [M]. |
| Flesh puff (bot victim) | `NDrone2_DealWithObjHit` 0x13fff0 case 5: 0x20008bc (3.0, 6.0, life 0x3c), or 0x200026e for the 0x32/0x33 bolt [M :235-256]. |
| Victim hit feedback | `Player_DealWithObjHit` 0x194458: the flash +0x968, the direction bits +0x967, `Sound_Play(0x412)`, the 1-in-4 grunt [M :201-236]. |

## What the MP paths emitted before this task

- **The shooter's own view**: `step_bond`'s `'shot` block runs the full SP list above (online the
  pipeline's draws come from the machine's private generator — scene.rs's `net_rng_shared` swap —
  so the shared stream never sees them) and latches `NetShot` for the session input. This view was
  already SP-shaped.
- **The other machines' view** (mp-remote-fx, docs/systems/mp-remote-fx.md): a bespoke emitter set —
  the muzzle light, the op-1 sounds, a bespoke `MpTracer` slot trail, and a flush-time impact via
  `Effects::bullet_impact` + its sound. **Missing against SP: the shell casing, the muzzle smoke,
  the bullet object's own trail/whiz/ribbon path, and the per-pellet spread** (the impact sat at the
  exact shared segment, no spread draws).

## The fix (this task): the remote view rides the same SP code

For a remote human slot's shot (`MpShot` with weapon ≠ 0), `mpfx_shot` now:

1. Spawns the shot's cosmetic rounds through `Drones::fire_remote_cosmetic` (drone.rs) —
   `fire_player`'s spawn arm (`Bullet_init` 0x12c5a0's per-pellet loop) with the per-shot generator
   (seeded (frame, shooter, index) — the same draws on every machine, never the shared stream):
   the pellets, the quarter-speed gate, the swoosh ribbon, the 0x10e whoosh, and the shell eject
   (`Bullet_init_casing` 0x12fde0) at the shooter's body [S — no view model to pose; the head stands
   in for the gun datum].
2. Emits the muzzle smoke through the same `Gas_Init` shape as SP under the same gates [S — see the
   table; the port matches SP, not the game's MP smoke skip].
3. Retires the bespoke human emitters: the trail and the world impact now come from the cosmetic
   rounds' own flight (`update_player_bullets` → `Bullet_DoTrails`, `flush_world_impacts` →
   `Effect_Bullet`) — one tick after the shot, exactly like the shooter's own rounds (which spawn
   mid-`step_bond` and fly in the same tick's `drones.tick`) [S: the cosmetic spawn lands at the
   end-of-tick flush]. The bots keep their bespoke emitters (their shots are the game's drone path,
   `DroneWeap_FireWeapon` 0x178f68 — the player pipeline never fired them).

What the flush keeps for humans: the third-person muzzle flash latch (the game draws a player's
flash on the first-person view model only — `Draw_MuzzleFlash` 0x1a3fe0 [M]; the port shows it on
the body so the opponent sees the shot [S]), the muzzle light, the op-1 script sounds, the bot
flesh puff and the victim hit feedback (both victim-side presentations).

## Determinism

The shared stream (`drones.rng`) is never drawn at the flush: `fire_remote_cosmetic` takes the
per-shot generator, `casings.eject` its explicit rng parameter, `gas.init_ex` consumes nothing
(G-033), and the cosmetic rounds' later flight draws ride each machine's private generator (the
same window the shooter's own rounds draw in — scene.rs's `net_rng_shared` swap). The hashed
components (frame_rng/bodies/vitals/combat/match/bots/world) are untouched; the effect pools are
not hashed. Sister: `mp_perf_16_live`'s digest and `mp_remote_fx_live`'s per-tick hash equality.

## Tests

`crates/nf-view/tests/mp_shot_effects_live.rs` (ISO-gated):

- `sp_and_mp_shots_emit_the_same_effects` — the SP reference (a plain `nf_view::load` scene, the
  weapon equipped, N shots through `step_bond`) counts per shot: the bullet-object spawns (+0x18
  pellets), the casing ejects, the muzzle lights, the smoke puffs, the flash latch ticks and the
  impact decals. A 2-machine MP match firing the same weapon per the shared gate then counts the
  SAME kinds on the shooter's machine (its own pipeline) and on the other machine (the cosmetic
  path) — the counts match the SP reference's per-shot shape. Sister: both machines' hashes agree
  every tick.
- `the_hash_never_moves` — the third machine with `mp_fx.disabled` (the mp-remote-fx pattern):
  hashes agree with the enabled ones on every tick.
