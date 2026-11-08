# agent-guards: guard behaviour against the decompile

**This branch moves the shared random stream** (corpse alerts, the first-attack pick, hearing rows and the
alarm rows all change which guards act and when). It stays on its own branch for a single re-pin.

## Fixed (each with a test red on the old code, `crates/nf-view/tests/guard_parity_live.rs` unless named)

1. **Bodies alert guards.** `DSTATE_Dead` ENTER sets `+0x4FC |= 8` for prop 0x31 unless the kill was a head
   hit (`+0x3A`) [M 0x163fe8:29-33]. `SetAsDead` leaves +0x100 set [M 0x14a2f0], so the corpse stays an alert
   source and observers with prop 0x29 take it [M 0x176250]. The port never set the bit. Props 0x31/0x29 are on
   nearly every guard on every level (census below).
2. **Who a first sighting alerts.** An own-sight first attack with prop 0x33 raises bit 1 and the 0x200000
   latch, so FirstAttack's bit 0x10 is skipped; a FirstSightState pick also skips it [M 0x147c48:55-90,
   0x148010:30-57, 85-88]. The port raised 0x11 always, so observers without prop 0x28 were alerted. The
   prop-0x1B/0x19 move pick is FirstSightState's and now only rolls on own sight [M 0x148010:21-29].
   FirstAttack sets the sighted bit on every path [M 0x148010:36, 65].
3. **07000009/0A (the tower guards, all team 3, mode 9):** no sight widening on the first attack
   [M 0x148010:42-56, 69-83]; they can attack on sight outside their civilian states [M 0x177ed0:29-38];
   the alarm run does not stop for Bond [M 0x165640 case 3]; a sighted guard hit by the dart, smoke or flash
   now gets the same reset and alarm re-arm the tase already had [M 0x1469b8:103-128, 220-246, 283-305,
   331-357]; flash and tase recovery send civilians back to their resume state, and voice 0x10 joins team 1
   first [M 0x163350:18-23, 0x1630b8:16-27].
4. **Hearing (`DroneVision_AlertSound`).** Modes 9/0x12/0x13/0x16 never investigate (attack above 0.7, else
   nothing) [M-dis 0x177928-0x1779b4]; 0C/0D voice 5/6 attack any sound at any range; state 0x1B on 14 is
   deaf; the alarm drops for modes 9/0x12 and attacks otherwise, with no hearing gates [M 0x177c18:107-118].
5. **Sight gates.** A guard holding cover (`cover_claimed`, +0x4F8 & 0x800), holding the interrogate bit, or a
   mode 7/0xE guard without prop 0x57 in states 0x85/99 does not attack on sight [M 0x177ed0:52-69].
6. **Drone-to-drone sight alerts are off on 07000004 and 0700001b** [M 0x1767d0:26-31].
7. **Out-of-range chase** holds in alert idle behind a drone ahead instead of running [M 0x148338 case 2]
   (unit test in drone.rs).
8. **Rescued hostages** with an alternate state stay put (Idle, or Hide with an armed person near), and Idle/Hide
   follow armed drones within 20 [M 0x15ccf0:23-34, 0x15ce60, 0x15cfe0, 0x177420].
9. **Per-mode props at creation**: mode 0xD (ninjas, 04/16) gains 0x3C/0x36/0x35, so smoke no longer floors
   them; modes 0xC/0x11 allies lose the explosive-shy props [M 0x14b300:745-797].

Checked and matching: hearing loudness, alarm cadence, drone-to-drone alert chain, grenade gate, combat move
choice, difficulty (spawn gate, damage, accuracy, captain, health), cover availability, surrender gates.
Verified dead in this build: drone-to-drone shouts (`ShoutFromOtherDrone` returns 0, both arms load s3 = 0).

## Tests

- guard_parity_live 20/20 (19 red on the old drone.rs, the head-kill sister green); nf-game lib + 10 nf-game
  binaries green; consider_alerted, stealth_ai, security_alarm_09, walkreal 01/03/0a green.
- `stealth_ai_live::hearing_09_gunshot_vs_dart` re-pinned: it expected HeardNoise from a mode-9 guard, which
  the bytes rule out.
- Red, with cause measured by per-change toggles and a stream control:
  - walkreal 05 / 0c / 0d real-damage walks: one extra random draw on unmodified master already fails them
    (05 2/5, 0c 4/5, 0d 5/5 pokes). Stream fragility, re-pin.
  - walkreal_16 real-damage chase walk: same class, not controlled.
  - walkreal_09 `the_alarm_switch_fails_objective_1`: caused by fix 3's dart reset (a darted guard who had
    seen Bond wakes with the alarm run armed, as in the game). The walker's takedown needs updating.
  - walkreal_14 infiltration (3 tests): caused by fix 5's mode-7 run-past gate; Bond stalls at the door-wall
    jamb at (30.33, 0.55, 35.44). Not stream fragility (3 pokes on master pass). Not resolved.

## Guards seeing through walls, 07000001 near (11.43, 6.47, -3.51)

Probe `guard_sight_walls_01_live` (ignored). Every sighting at that spot (guards 13, 9, 0) runs through door 0,
which guard 13 opened (state OPEN). No static triangle lies on any sight line in either winding, and the
game's cel walk (`AINetwork_TestRayCels`) reaches. The port's ray tests every triangle (the 0xA27 drops are
off), so it is stricter than the game. Not reproduced. Open question for a live check: at that spot, does the
game let guards in the paper-screen room see Bond through the screens beside door 0?

## Not ported

The voice-0xC CrouchCover roll in CombatNoMove (07000008's ally only, behind the +0x3C fire byte); the
start-script property writes in DoModeSettingsNEW [M 0x14c600:130-155].
