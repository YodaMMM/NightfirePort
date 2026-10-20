# mp-death-hands-online: the online death's hands switch and the respawn re-equip

The ONLINE multiplayer death path (`step_net_combat`, the per-slot MP combat every machine runs
on the shared input stream): when a human dies, its machine's first-person gun goes to the fists
at once, the wire's `held` byte (nf-net input byte 21) carries 0x47 for the whole dead window so
every machine's remote body shows the fists, and the shared respawn re-equips the spawn kit with
`clip_size·2` rounds with the shot gate re-seeded the same way on every machine. No desync: the
state hash stays equal through deaths.

## 1. The death tick: the gun goes to the hands

`Player_CheckForDeath` 0x190810 runs `Player_WeaponNone` on the death tick, after the state
write (`obj+0xff = 0x12`, `obj+0xec = GameState+0x52`) and before the cam reset:

- [M 0x190810:61] `Player_WeaponNone__FP7obj_tag(param_1);` — both the SP and the MP arm reach
  it (the MP weapon-drop pickup arm 0x190810:43-55 runs first and does not return).
- [M 0x1a9858:12-14] `Player_WeaponNone`: `M_CONTROL+0x64 = M_CONTROL+0x62` (the current weapon
  saved for the toggle-back), `M_CONTROL+0x62 = 0x47` (the held id becomes 'G' — the fists), `+99
  = 0x47` (the switch request — no holster cycle starts). No anim switch: the first-person gun is
  gone on the death tick itself.

Port: `Gun::go_to_hands` [nf-game/src/gun.rs:188-193] sets `saved_weapon`, `weapon = 0x47`,
`switch_request = 0x47`, `switch_state = 0` — the same four writes. The death tick's caller is
`Sim::step_bond` [nf-view/src/scene.rs:4524]:
`if !mp_death_was_active && self.drones.death.active && self.gun.weapon != 0x47 { self.gun.go_to_hands(); }`
(online included — the line is not gated on the net mode; the offline-only respawn arm below it
IS, scene.rs:4526). While dead nothing re-draws a weapon: the input lanes stay neutral and
`go_to_hands` is not reached again (`death.active` is already on).

## 2. The wire: held byte 21 carries 0x47 for the whole dead window

The nf-net input's byte 21 is `held` — "the weapon this agent HOLDS this tick" [M
nf-net/src/input.rs:40-43, encode :168, decode :202]. The app's law builds it from the local
gun: `ni.held = sim.gun.weapon` [nf-view/src/lib.rs:4643]. So from the first tick sampled after
the death tick (the input for tick T is sampled before tick T's sim runs, so the byte switches
one sample after `go_to_hands`), the dead machine submits 0x47 until the respawn re-equips.

`step_net_combat` copies the byte into the per-slot view state on every machine [nf-view/src/scene.rs:10630]:
`self.mp_held[s] = if s == local { self.gun.weapon } else { inputs[s].held };` — the local
slot's own `gun.weapon` (already 0x47 while dead) and the wire byte agree by construction.

The remote body then shows the fists. The draw-side getters:

- `Sim::mp_body_held(slot)` [nf-view/src/scene.rs:10497-10503] returns the slot's held id (bots:
  the brain's weapon). The weapon mesh draws only while `mp_body_held(slot) == w` [scene.rs:8354-8360].
- `weapon_anim_class(wd, 0x47) == 0` [M nf-formats/src/mpanim.rs:139-141 — record 0x47 does not
  exist, the map_or(0) arm] — class 0 is the no-weapon/fists class, the body's aim and gun anims
  never engage (scene.rs:4620-4621, 7909-7924).
- `Sim::mp_body_death(slot)` [nf-view/src/mpfx.rs:118-133] is `Some` for a dead remote human
  (`mp_dead_ticks[slot] > 0`) — the death script pose; the weapon draw additionally refuses while
  it is `Some` ("the dead body holds nothing", scene.rs:8359).

So on every machine the victim's body shows hands and no gun from the first 0x47 byte to the
respawn, matching the game: the dead body's script carries no weapon [M 0x190810:89-93 — substate
0xd's death script] and `Player_WeaponNone` emptied the held id.

## 3. The respawn: the kit comes back with clip_size·2, the gate re-seeds

The shared dead loop in `step_net_combat` [nf-view/src/scene.rs:10695-10722] waits 300 ticks of
`mp_dead_ticks` — the game's 5-second gate [M 0x190bb0:41: `FRAME_RATE_INT * 5` frames against
`GameState+0x52 - obj+0xec`; 60 Hz → 300; the call at :53] — then takes `mp_respawn`'s spawn pick
+ health ([M MP_ReSpawn 0x185368:22-54 the spawn point; :71-73 `Player_SetHealth(MPSettings[slot]
+0x2c + 100.0)`]), zeroes the armour ([M 0x185368:69 `BL+0x8B0 = 0`]) and re-stands the body
([M 0x185368:89 `Player_StandAtNewPosition`]).

The re-equip runs where the game runs it — inside `MP_ReSpawn`'s dead arm (substate 3/0x12):

- [M 0x185368:78] `MP_EquipPlayer__FP7obj_tag(param_1);`
- [M 0x1855f8:14] `Player_InitAmmoWeapons` — the wipe (every clip/owned byte reset);
- [M 0x1855f8:22-26] the fists (id 1) equipped, then the weapon set's spawn weapon
  (`base(PickupMatrix[set][0])`) with `+0x92 = clip_size·2` rounds (the equip splits clip_size
  into the clip and clip_size carried);
- [M 0x1855f8:27-47] the AR's grenade ammo, the grapple when `MPSettings+0x1D0`, the scenario
  bonus weapon 0x3B (Protection team 0 / Demolition team 1), `Player_CheckWeaponsLoaded`;
- [M 0x1855f8:50-52] `+100 = 0x47`, `+0x62 = 0x47`, `+99 = the spawn weapon` — the draw of the
  spawn weapon rides the switch request (the game shows the fists one frame and draws the gun
  through the switch).

Port: on the respawned machine `step_net_combat` clears the death sequence and calls
`mp_equip_local` [scene.rs:10710-10713 → :9280-9295 → nf-game/src/mp/equip.rs:87-117
`mp_equip_player`], which runs the wipe + the set's spawn weapon with `clip_size·2`
(equip.rs:97-99), the grapple and the scenario bonus, and re-draws the gun
(`Gun::new(req, clip, spare)`). Port choice [S]: the port equips instantly (`weapon = req` on the
respawn tick) instead of the game's `+0x62 = 0x47` then `+99` draw — the first-person draw cycle
is the switch machine's job and the wire byte goes straight to the spawn weapon; the kit and the
gate state are the pinned surface.

The shot gate agrees on every machine: the respawn re-seeds the slot's shared gate with the same
kit on EVERY machine, local or not [scene.rs:10716-10721 `gate_equip`; nf-game/src/mp/combat.rs:700].
Between deaths the gate follows the wire: `gate_tick(shooter, inp.held, …)` [combat.rs:734] is
fed the same byte 21 every machine holds, so a dead slot's gate sits on the fists 0x47 and every
machine's gate for that slot agrees (and the gate's state is hashed — §4).

## 4. No desync: the hash carries the death window

`net_state_hash` folds the fixed components [scene.rs:10772-10864]; the death window touches
hashed state only through:

- `vitals`: `mp_hp` / `mp_dead_ticks` / `mp_armour` per slot (the kill, the 300-tick count, the
  respawned health/armour 0);
- `combat`: the shot gate's rounds/thrown (the fists window idles it; `gate_equip` re-seeds it
  identically everywhere);
- `bodies`: the respawned body's pos/yaw (the same `mp_respawn` pick from the shared RNG).

The local death *presentation* (`drones.death` — the blood pane, the camera) is not hashed: it is
this machine's view of its own death, and `step_net_combat` clears it at the respawn
(scene.rs:10711). A 3-machine run with deaths keeps `net_state_hash` equal on every tick —
pinned by `three_machines_deaths_keep_the_state_hash_equal` (crates/nf-view/tests/
mp_death_hands_online.rs), alongside the 2-machine kill test.

## 5. Tests (crates/nf-view/tests/mp_death_hands_online.rs, NF_ISO-gated, NF_SILENT)

- `online_kill_victim_holds_the_fists_wire_byte_and_the_kit_returns` — a real P2K kill on
  07000023, 2 machines, the app's held-byte law (`row[1].held = machine B's gun.weapon`, the
  lib.rs:4643 build). While dead: every sampled byte is 0x47, machine B's `gun.weapon` is 0x47,
  machine A's `mp_body_held(1)` is 0x47 with `mp_body_death(1)` set. After the 300-tick gate: the
  kit is back on the respawned machine (the set's spawn weapon, clip_size in the clip and
  clip_size carried [M 0x1855f8:22-26]), the first post-respawn byte is the spawn weapon again,
  and both machines' gates hold the same fresh kit. Sister: the killer's gun is untouched — its
  `gun.weapon` stays the spawn weapon through the whole window (never 0x47, no re-grant).
- `three_machines_deaths_keep_the_state_hash_equal` — 3 humans on 07000024 (Skyrail), two real
  kills (slot 0 kills 1; later 2), `net_state_hash` equal on all three machines every tick.

Every game-behaviour assertion above cites its [M] address; the wire-byte timing (one sample lag
after the death tick) is the app's input-sampling order [S port design — the PS2 had no online
play], kept out of the hard assertions: the tests assert the byte while `death.active` and the
view getters, not the exact kill-tick byte.
