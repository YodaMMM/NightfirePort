# MP combat online — every weapon, identical on every machine

mp-combat-weapons (agent 10, 2026-10-18). Code: `crates/nf-game/src/mp/combat.rs` (the shared
rounds/grenades/blasts and the MP pain rules), `Sim::step_net_combat` / `Sim::mp_hurt` /
`Sim::mp_combat_bodies` (crates/nf-view/src/scene.rs), the shot description in
`nf_net::Input` (crates/nf-net/src/input.rs) and the app's hand-off (crates/nf-view/src/lib.rs,
`net_shot_pending`). Tests: `crates/nf-view/tests/mp_combat_weapons_live.rs` (two machines, one
test per weapon class), `nf-game` lib `mp::combat::tests`.

## The split [S port design — the PS2 game has no online play]

- **When a round leaves the weapon** is the shooter's own machine's decision: its full local fire
  pipeline (`Sim::run`'s release — the fire rate, burst, clip, reload, the release frame of the
  throw scripts, the stunner/decryptor gates) runs only for the local slot. At the release it
  records `Sim::net_shot` (the fired record `gun.weapon` — the twin id carries the fire mode —
  the aim ray `dir` with free aim / auto-aim / the scope included, and the zoomed-scope flag).
- **The shot description rides the session input** for the next submitted tick (the app's
  `net_shot_pending`, one tick behind the local release plus the input delay, like the 700v6
  flag): `shot` (flag-1 bit 2), `shot_weapon` (byte 22), `shot_flags` (byte 23, bit 0 = zoomed
  scope), `shot_aim` (bytes 24..28: yaw in TAU/65536, pitch in (π/2)/32767). The wire input is 28
  bytes (protocol 5); the relay packet (3 ticks × 15 agents × 28 B + headers = 1279 B) and the
  JOIN_LOG chunk (3 × 16 × 28 B + headers = 1357 B) still fit `MAX_PACKET` 1400.
- **Everything after the release is shared**: every machine decodes the aim with the port's
  deterministic sine (`combat::dir_from_aim`, `trig::sinf`) and runs the round from the shooter's
  head on the shared stream (`drones.rng`), in slot order, against every MP body. The local
  machine's own SP rounds/blasts (player_bullets, grenades) stay as its local visuals; online they
  cannot hurt the local player (`Drones::net_no_self_hurt`) — the slot's health is the shared
  `mp_hp`, written back to the HUD every tick.

A shot whose `shot_weapon` is 0 (the 700v6 shape older test rows send) fires the held record
along the body's view.

## Spawn — `Bullet_init` 0x12c5a0 [M]

- **Detonator** (`+0x68 & 0x20000`, the remote mine's 0x38/0x39): no round; every live grenade of
  the fired record's base owned by the shooter takes `BU+0xFC = −10.0` (the port's fuse model:
  `range = travelled − 10`) → it blasts on the next tick [M `Player_WeaponInitBullet` 0x1a4bf0
  disasm 0x1a4c54-0x1a4c88]. Another player's detonator leaves the mines alone (the owner match).
- **Grenade family** (`+0x6C & 8` gravity with a `+0x70` bounce 0x80 / stick 0x100 / contact 0x4
  bit — the SP routing gate): one spread draw set, a `Grenade` (grenade.rs) with `Bullet_init`'s
  fuse (`combat::grenade_fuse`: 210, 300 with `+0x70 & 0x10000`, the satchel family's
  `((id − 0x3b)·5 + 5)·60`, the player's remote mine 7777.0, the no-fuse impact records'
  `+0x1C / speed` silent range-out [M :103,196-229; 0x12e7b0:158-162]).
- **Bullets**: `+0x18` pellets (the Frinesi's 8), each with the three spread draws
  (`bullet::spread`), `+0x20` speed (× 0.25 for `+0x6C & 0x200` [M :188-190]), `+0x1C` range,
  `+0x0C` damage. Melee is the same path: the fists 0x01 are a 5.0-damage round of range 1.75
  [L record 0x01].
- `+0x6C & 4` (the Sentinel 'Guided' 0x2C) starts the owner's ride [M :234-254].

## Flight — `Bullet_update` 0x12e7b0 [M]

The first tick's `(0.5 + FRand 0.5)·speed` step, case 1's gravity for `+0x6C & 8`, the hit
segment `max(step, 1)`, the range-out; the quarter-speed ramp `speed += full·0.02` per tick after
the spawn tick [M `Bullet_DoTrails` 0x12d890:134-138]; `Bullet_homing` 0x12f358 for `+0x6C &
0x20000` (lock the nearest live body ≠ the owner within 1000 inside the 0.66 cone with a clear
bullet-family LOS, steer each component by `clamp((to − dir)·0.0333, ±0.01)`); the guided rocket
steered by its OWNER's look input that tick (`drone::steer_missile`, the scene's mouse→analog
conversion) [M :177-244]. Grenades run `Grenade::update` (the SP bounce/stick/fuse code); a
contact-explode grenade (the MGL 'Grenade' 0x2A, the AIMS launcher 0x1B) also blasts when its
move crosses a body.

Contacts: the nearest of the world (`segment_surface_bullet` — foliage passes) and the bodies (the
owner excluded). `+0x70 & (4 | 0x100000 | 0x200000)` blasts at the contact [M
`Bullet_CollisionHandler` 0x12ce80:194-206]. The scenario site (Demolition/Protection) is a round
target too (the nearest thing on the segment). MP rounds stop at the world — no pierce, no
ricochet [S].

The placed caps (6 mines / 6 tripbombs per owner, the oldest goes [M 0x12ce80:296-370]) and the
armed tripbomb's beam trigger (a live body within 1.0 of the beam → its 8/100 blast [M
`Bullet_update` case 3 0x12efb8-0x12efc8]) run as in SP.

## Blast — `Explode_Propagate` 0x17c518 [M]

Per blast, in order (a chained blast appends):

- case 3 (players) / the bots: every live body, the owner included. `amount = damage · (1 −
  (d > 0 ? (int)d : 0) / radius)`, `d = |obj+0x80 − pos| − obj+0x8C` — the body's world sphere
  (`Bond::sphere_centre`'s offset, radius 1.0576 [L ee_start_05.bin M_CONTROL+0x80/+0x8C]);
  `combat::blast_falloff` (the game's `(d > 0)` select — the SP `explode::propagate` overshoots
  for an object deep inside its own sphere, its sister test pins the difference);
- case 0x35 (MP objects): the site takes the falloff (`MP_ApplyDamage`);
- case 5 (bullets): the grenade-family records 0x2B, 0x34–0x37, 0x3A inside are destructed
  (`Bullet_handle_object_destruction` [M :141-162]) — frags, mines, tripbombs and MGL 'Delay'
  rounds chain their own blast; the stun/smoke just go (their flash/cloud branches carry no
  `Explode_Create`; their effects are the view's) [S].

## Damage — the MP pain path [M]

`Sim::mp_hurt`, per event:

1. A round's hit on a human registers first: `Player_DealWithObjHit` 0x194458 case 5 →
   `MP_RegisterBulletHit` 0x1857c0 (the team game's friendly-fire gate, the last-shooter word). A
   blast reaches `Player_Hurt` 0x196060 directly — no gate (case 3 registers the killer only when
   the blast killed [M 0x17c518:124-139]).
2. Assassination (scenario 0x400) [M 0x194458:44-74]: the assassin's (`MP_IsAssasin`) round on the
   target (`MP_IsTarget`) deals the target's whole health on part 5, and on parts 1..4 when the
   round's direction is within 70° of the target's facing (from behind).
3. `Player_HandlePain` 0x1902c8's MP arm = `BOT_handlePain` 0x125ed8's [M :49-81 / :49-74]:
   `× Plr_DMod_Multi` when the hit has a part (rounds; blasts pass −1); with **Location Damage**
   (+0x1C8, label 0x24e [L USATxt.dat]; boot value 1 [M bootup 0x1c7e78 @0x1c7ff8]) part 5 ×
   `Plr_DMod_Head`, parts 0x14/0x15/0x20/0x23 × `Plr_DMod_UpperLimb`, 0x31–0x38 ×
   `Plr_DMod_LowerLimb`; **Professional Mode** (+0x1BC, label 0x1ac — the port's
   `MpSettings::capture_reward` word) × 3. The MP arenas' tuning: Multi 4.0, Head 4.0, limbs 0.8
   [M ReadTuningVars group, nf-game tuning.rs] — a P2K torso round costs 14, a head round 56.
4. Armour first (`mp_armour`, `BL+0x8B0`): `absorbed = min(d, armour)`, health − the rest, below
   1 → 0 [M :82-134]; the pain grunt's `Rand_Rand(4)` on the shared stream [M :111]. The local
   slot's HUD flash (`+0x968`) and hit-direction glow (`+0x967`) are this machine's only.
   `MP_ReSpawn` zeroes the armour [M 0x185368:69].
5. Health 0 → `mp_dead_ticks = 1` and `player_killed(victim, Some(owner))` [M MP_PlayerKilled
   0x188f70] — a self-blast is a self-kill. Bots: the same multipliers, no armour, the existing
   death/respawn bookkeeping.

The Player Mods words reach the clients in `MatchSettings` (protocol 5 adds `location_damage`
as its 20th scalar; `capture_reward` rode already) and the host's menu confirm now maps
`mp_player_mods` onto them.

## The guided ride online

`MpCombat::ride[slot]` (riding, the 15-tick restore after the rocket dies [M 0x12e660:28,
0x193600:242-256]) is shared. While it holds, every machine zeroes that slot's movement input —
the remote replicas in `step_net_remotes`, the local body in `step_bond`'s online arm (which only
steers the local visual rocket alongside) — and the local camera (`missile_cam`) follows the
shared rocket. Before this, the local ride froze the body while the other machines kept walking
it on the same input: a desync for anyone firing the Sentinel online.

## Body volume [S]

Upright cylinder, radius 0.35, from the feet (`Bond::pos − 1.1`) to the eye + 0.15. Part from
the contact: the top 0.3 → 5 (head), below the hip (feet + 0.95) → 0x31 (upper leg), a
torso-band contact more than 0.22 off the centre line sideways → 0x14 (upper arm), else 4
(torso). The game tests the player's 16 skeletal boxes (`HumanCollisionBones` 0x2c6be8 via
`Collide_Jointy` 0x1ec180); the MP bodies are not posed in the shared sim. The old 700v6 test
(three 0.45 spheres 0.5–1.5 ABOVE `Bond::pos`) sat mostly over the head.

## Open

- The bots' own shots still use the bots' instant body-sphere ray (`step_mp_bots`) and the raw
  damage word — routing them into `MpCombat::fire` (and the bot-shooter multipliers ×1.5 melee /
  ×1.25 [M 0x194458:31-41]) is the next step.
- Rounds neither pierce nor ricochet online; the stun's flash and the smoke cloud have no shared
  effect (the stun has no damage branch in the game either).
- Armour pickups online belong to the shared pickup work (agent 9): `mp_armour` is ready for them.
- Local-only damage sources other than blasts (falls, hurt volumes) still write the local health
  directly (world interactions).
- The body hit zones are the cylinder stand-in above.
