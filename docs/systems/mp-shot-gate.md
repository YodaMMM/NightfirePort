# mp-shot-gate: every machine enforces the shooter's fire rate and ammo

Every machine applies a human slot's shot description (`nf_net::Input::shot` + `shot_weapon` +
`shot_aim`) only when the game would let that slot fire: the slot holds that weapon, has the
rounds, and the weapon's refire interval has run. Anything else is dropped, identically on every
machine (the drops change no RNG draws — `MpCombat::fire` is what draws). The local machine runs
the same gate, so its own view agrees with the shared one.

## The game's fire gate ([M] `Player_WeaponFiring` 0x1a5460)

The gate state lives in `BLData` (per player), the per-weapon numbers in the `weapon_data` record
(0x10c bytes each):

- **The refire timer** `BLData+0x92c` (f32). Per tick, while it is above 0 it falls by
  `FRAME_RATE_MUL` (1 at the fixed 60 Hz): `*(BL+0x92c) = timer - FRAME_RATE_MUL` after the early
  return `if (timer <= 0.0) return` — so a tick with the timer expired is the fire tick, and the
  timer never falls below −1. [M 0x1a5460: the `Input_Action(9,4)` block's `if (fVar19 <= 0.0)
  return; *(float *)(iVar1 + 0x92c) = fVar19 - FRAME_RATE_MUL;`]
- **The interval** `weapon_data[w]+0x40` (u32, frames). The fire block resets the timer to
  `max(interval, 1.0)`: `fVar19 = (float)*(uint *)(weapon_data + iVar4 + 0x40);
  *(uint *)(iVar1 + 0x92c) = (int)fVar19 * (uint)(1.0 < fVar19) | (uint)(1.0 >= fVar19) *
  0x3f800000;` [M 0x1a5460:688-693]. Disc values (the port's probe over the ISO's ACTION.ELF
  table): the P2K 0x02 → 8, the Storm M32 0x12 → 6, the SG5 0x16 → 4, the Frinesi 0x1c → 60, the
  MGL 0x2a → 60, the Sentinel 0x2c → 60 (the Unguided twin 0x2d → 30), the frag 0x34 / the remote
  mine 0x37 / the tripbomb 0x3a → 450, the detonator 0x38 → 60, the fists 0x01 → 30 [D].
- **The rounds** `BLData+0x1b8 + w·0xc` (i16 per weapon slot; a twin record with `+0x05` set and
  the same `+0x90` ammo type reads its BASE's slot). `Player_RoundToFire` 0x1a18e0 checks
  `clip < (short)(+0x91 byte)` → 0 (no fire), else spends the `+0x91` byte; an `ammo_type == 0`
  (`+0x90`) weapon checks and spends nothing (melee / the detonator's `+0x91 = 0`) [M 0x1a18e0].
- **The shots left in the pull** `BLData+0x936`: armed from `weapon_data[w]+0x28 + mode·2` while
  the trigger is held with the timer expired [M 0x1a5460: the held branch's
  `+0x92c = 1.0; +0x936 = shots_per_pull`], decremented per shot.
- **A reload blocks every shot**: the fire path runs only at view-anim state 0 (state 9 refires
  unless flag `0x8000`, state 0xb falls through) — the reload states 5/6/7 all return before the
  fire block [M 0x1a5460: the `*(short *)(*(int *)(iVar1 + 0x7e8) + 0xf4)` gates].
- **The timer zeroes when the draw completes** (view-anim case 4), else one weapon's leftover
  interval poisons the next [M `Player_SetWeaponAnimObj` case 4 — the port's scene.rs "draw done"
  note]; the auto-reload branch also zeroes it [M 0x1a6244].

The sender's release timing (the fire script's `+0x44` release frame) stays the sender's business
— the shared gate is the RATE FLOOR (the interval) plus the ammo, which is what a modified client
can violate. The game's own cadence between two releases is ≥ the interval (its own timer), so
the gate never drops a legitimate shot; the tests pin that.

## The shared gate (port design [S] — the PS2 had no online play)

`MpCombat.gates` (hashed with the rest of the shared combat) carries one `ShotGate` per human
slot: the shared held weapon, the slot's shared inventory (`Inventory`'s `clip` / `carried` /
`owned` — the `BLData` fields), the refire timer, the reload state machine, and the trigger's
previous level. Per tick (`MpCombat::gate_tick`, in `step_net_combat`'s slot order, before any
shot is applied):

1. The held weapon follows the wire byte (`inp.held`); on a change the timer and the reload
   machine reset [M the draw-done zero above].
2. The reload machine mirrors `Gun::reload_tick` (state 5 start → 6 shell loop → 7 end): the
   start from the reload key, or the auto-reload (no fresh fire press, the clip can't feed a
   shot, the spare has rounds — the timer zeroes in the auto branch) [M 0x1a5460's action-14 /
   `Input_Action(9,1)==0` branches; the port's `gun.rs` reload machine].
3. The timer falls by 1 per tick while above 0 [M the decrement above].

The filter (`MpCombat::gate_allow`, before `MpCombat::fire`): the shot's record must be the slot's
shared held weapon (`shot_weapon == inp.held` — the game fires the held weapon, slot `+0x62`), the
reload machine must be idle, the timer must be expired, and the shared clip must cover the `+0x91`
byte. On a pass the gate spends the rounds and resets the timer to `max(interval, 1.0)` — exactly
the game's fire tick, minus the view-anim half that stays the sender's.

**The shared inventory** is seeded per slot by `mp_equip_player` 0x1855f8 (the spawn loadout,
`clip_size·2` rounds into the spawn weapon [M nf-game::mp::equip]), updated by:

- the gate's own spends (every machine, identically);
- the shared pickup takes (`step_net_world` runs `Inventory::take` on the taker slot's gate —
  the same call the local inventory gets, same clock/skin flags, so a remote slot's ammo pickups
  land in the shared state too);
- the shared respawn re-running `mp_equip_player` (the game's `MP_ReSpawn` 0x185368 →
  `MP_EquipPlayer` 0x1855f8 re-equip).
- **A weapon the gate never saw** (never equipped, never taken — the test rows' directly-claimed
  `held`) seeds ONCE at the game's spawn-equip split (`clip_size` into the clip, the rest
  carried) and is marked seen: a switch away and back does not refill. [S — the gate cannot see a
  remote machine's local-only inventory events beyond the shared takes; the one-time seed is the
  stand-in, and the rate floor still binds regardless.]

Bots keep their own gate (`DroneWeap_Ready2Fire` 0x178c48's pipeline, already ported) — the
`ShotGate` rows cover the human slots only.

## Tests

`crates/nf-view/tests/mp_shot_gate_live.rs` (NF_ISO-gated, two machines in lockstep):

- `rocket_spam_fires_at_the_launcher_rate_and_stops_when_dry` — a slot sending an unguided
  Sentinel rocket (0x2d, interval 30 [D], 4+4 rounds) every tick: exactly the seeded rounds leave,
  spaced at the interval, then nothing; both machines' hashes agree every tick.
- `normal_rate_loses_no_shots` (sister) — the same slot firing at the real interval passes every
  shot it sends.
- `shot_for_an_unheld_weapon_does_nothing` (sister 2) — `shot_weapon` for a record the slot is
  not holding spawns nothing (the fired list stays empty), hashes agree.
