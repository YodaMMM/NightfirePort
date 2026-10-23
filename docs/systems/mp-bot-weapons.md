# mp-bot-weapons — the MP bots' weapon layer

The bots' weapons on the ported MP branch: the `BOTWEAP_*` family (`crates/nf-game/src/mp/botweap.rs`),
the firing pipeline's armed arm (`BotBrain::handle_firing_armed`, `crates/nf-game/src/mp/bots.rs`), the
pickup take (the shared `update_pickups_net` walk + the bot arm in `Scene::step_net_world`), and the
weapon choice (`BotBrain::weapon_choice` → `combat_choice`). Built on the MP work (start weapons, shared rounds,
objectives, lazy sight), with the never-fire wedge below fixed.

## The firing pipeline and the never-fire wedge (the fix)

Per tick (`BotBrain::step` → `handle_firing_armed`, gated on `BotArsenal::armed`):

- `burst_left == 0` → `next_bullet_time_mp(start=true)` — `DoFiring`'s `+0xbc0 == 0` arm
  [M DroneWeap_DoFiring 0x17a080:11-13], which rolls the burst and the start delay;
- unseen ≥ 0x10 → the burst is marked spent and the tick returns [M 0x17a080:22-30];
- `tick < next_shot_tick` → wait (the bullet deadline, `Drone+0xBC4`);
- the ammo gate `AmmoInGun < 1` [M DroneWeap_Ready2Fire 0x178c48:73-79 — `BOTWEAP_AmmoInGun`
  0x12ae40 returns 1 for the start weapon];
- the aim turn and the cone `acos(dot) < 0.349` (cos 20°) [M 0x178c48:117-124];
- `FireWeapon`: the shot, `last_shot_tick := tick` (`Drone+0xBC8`), the clip/pool decrement,
  `burst_left −= 1`; the last round marks the burst spent [M 0x17a080:29-36].

`DroneWeap_NextBulletTime` 0x179d40 sets the deadline `max(last_shot + interval, now + delay)`
[M :131-135] with the record interval (`weapon_data+0x40`) clamped to **60 ticks for weapons 6,
0xA and 0xE..0x10** [M :26-47] and the start delay `Rand(15) + 60/aggression` [M :113-121].
Unit pin: `mp::bots::tests::next_bullet_time_interval_floor` (the 60-tick floor for weapon 6, the
sister weapon 0x19 unclamped).

**The wedge (fixed here):** the port cleared the fire latch (`Drone+0x3B`) in the 0xD0 run arm, the
0xD7 backoff arm and 0xD2's route/vision tail. The game has **no `+0x3b` write in any of those
states** [M 0x16f390, 0x16f198, 0x16f5e8, 0x16fc30 — every `+0x3b` access there is a read; the
latch's writers are `DroneWeap_Fire` 0x17a8c0 (set), the anim layer `DroneAnim_SetCombatMoveAnim`
0x13cfb0 (clear, on losing the opponent / the non-combat anims) and the reload/sniper states]. A
cleared latch made the next 0xD2/move enter a *fresh* `DroneWeap_Fire` — whose `NextBulletTime(0,1)`
[M 0x17a8c0:12-13] re-pushed the deadline to `now + delay`. The combat bounce (0xD2's 1-in-3 exit
roll [M 0x16f5e8:48-52], the moves' 2–9 s timers [M 0x16f850 msg 1]) never stays anywhere for the
≥ 60 ticks a fresh deadline needs, so the deadline receded forever: **the bot alternated strafes
(0xD4) with the fire latch set and never fired** — the observed symptom, with weapon 6 (the
deathmatch set-0 start weapon, `PICKUP_MATRIX` row 0 → 6) as the gun every bot holds.

The fix keeps the latch set through the combat arms (0xD0/0xD7/0xD2-tail no longer clear it): the
deadline survives the bounce and the per-tick pipeline fires from any combat state — the game's
bots shoot while they move. The spent burst re-arms in the pipeline itself (`+0xbc0 == 0` arm), so
no state-side re-arm is needed; the port's interval-based re-arms were removed. Verified: the live
run's per-weapon shot tally went from all-fists to weapon 6 + the taken guns (102 and 150 shots in
the pinned run), kills 28 → 35, and the two-machine hashes still agree.

## The pickup take and the preference

- The shared take pass is `update_pickups_net` (`Drones::net_bodies`, the game's taker
  order: the humans by slot, then the bots [M Pickup_Update 0x18e148:203-246]); a bot's weapon/ammo
  take updates the bot's weapon layer in `step_net_world`'s apply loop via
  `BotBrain::take_pickup` → `BotArsenal::take_pickup` (`Pickup_Handler` 0x18e650's class-2 arm),
  and stamps the visit time (`setPickupVisitTime` [M 0x18e650:126-127]). The pickup itself is
  consumed by the shared walk even when the brain refuses it (the inventories are per machine —
  the documented [S deviation] of mp-world-sync).
- The choice: a take queues `pickupWeaponChangeChoice` (`weapon_choice` →
  `BOTSTATE_combatWeaponChangeChoice` 0x128708), which ranks the held+loaded weapons by the game's
  `PREFERENCE` order and the record's preferred class (`BOTSTATE_isPreferredWeapon` 0x129580: the
  fists 1 and the P2K 6 always preferred, pref 0 everything, prefs 1/2/4/5 the class byte
  `weapon_data+0x06`, pref 3 the gadget set {3, 7, 0x14, 0x24, 0x25}).
- Pins: `mp::botweap::tests::is_preferred_weapon_table` [M 0x129580:8-52],
  `mp::botweap::tests::combat_choice_ranks_and_min_range`, and the live
  `weapon6_fires_and_honours_the_interval_floor` (a taken weapon is not only taken but fired).

## Start weapons

The bots arm on their first tick with `BotArsenal::init` (`BOTWEAP_InitWeapon` 0x129e48: the fists
with 999 rounds, the start weapon with two clips, the bomb for the attackers, character 0x1A's
0x45). The start weapon is the weapon set's first gun's base (`PICKUP_MATRIX[set][0]` →
`weapon_data.base` [M 0x129e48:44-45]; `BotWeaponTable::start_weapon`), which reads 1 round in the
gun forever and reloads from nothing (`BOTWEAP_AmmoInGun` 0x12ae40:10-12,
`BOTWEAP_ReloadAmmoType` 0x129d28:27-34 — the game's own endless start weapon).

## Tests

- `crates/nf-game/src/mp/bots.rs`: `next_bullet_time_interval_floor`.
- `crates/nf-game/src/mp/botweap.rs`: `is_preferred_weapon_table` (+ the salvaged
  init/combat-choice/take/pain tests).
- `crates/nf-view/tests/mp_bot_weapons_live.rs`: two machines in lockstep on 07000023 —
  `bots_fight_take_pickups_and_switch_in_lockstep` (takes, switches, kills, hashes agree; the
  arsenal and the pickup state are in the net hash) and
  `weapon6_fires_and_honours_the_interval_floor` (weapon 6 fires; no two weapon-6 shots within
  60 ticks [M 0x179d40:26-47]; a taken weapon gets fired).
- `crates/nf-view/tests/mp_perf_16_live.rs`: the digest re-pinned for the fix (see the comment
  there — the simulated result changed on purpose: the bots fire).

## The online armour take runs the class-2 case-3 arm whole (standins-scene-930)

`step_net_world`'s bot armour take used to hand-roll only the health arm; the brain's armour
pool stayed 0 online. The game's class-2 (drone) armour arm is `Pickup_Handler` 0x18e650:119-133:
the drone's health (+0xAC) += 20.0 clamped to `BOT_vars+0xA4`, AND the armour byte (+0x769) +=
the pickup's amount capped 0x32, then `BOTWEAP_setDroneArmour` 0x12af70 — an empty body
[M disasm 0x12af70, size 8]. The online path now calls the same `BotBrain::take_pickup` the
local path calls (kinds 0/1 = the arsenal cases 0/1 [M 0x18e650:61-116], kind 3 = case 3, other
kinds consumed unhandled [M 0x18e650:117-118 → 334-348]), with the visit stamp after the switch
for every handled kind [M 0x18e650:135-136, `BOTSTATE_setPickupVisitTime` 0x129958 stamps
`time + 45.0`]. Test: `crates/nf-view/tests/mp_bot_armour_online_live.rs` (two machines in
lockstep on 07000023; the pool byte after a bot armour take = min(amount, 50); sister: the
weapon/ammo takes still flow and the machines' hashes agree every tick).

The same pass's taker-loop evidence, for the record: the game's `Pickup_Update` bot loop has no
kind filter [M 0x18e148:221-253] and measures both the distance and the line from the bot's
obj+0x30 [M 0x18e148:240-244] (the players' loop: distance from obj+0x30, line from obj+0x80
[M 0x18e148:210-214]). The port's bot layer models the weapon/ammo/armour kinds the arenas
place; other kinds are the handler's consume-unhandled arm.
