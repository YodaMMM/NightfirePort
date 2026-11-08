# agent-lvl-0d-11: levels 0700000d and 07000011

This branch moves the shared random stream and the guard rosters on every level with a spawner:
the spawner draw range, the retry count and the guards that spawn all change. Walks on other
levels that were tuned to the old rosters go red (listed below). They need the batch re-pin.

## Fixed

| What a player sees | Cause | Evidence |
|---|---|---|
| A spawner with more than one wave only ever brought its first wave: 0700000d group 6 gave 5 guards, not 15. A later "wave" healed a live guard or revived a corpse in place. | The port's list entry was both template and object. A template's record now survives: a later wave restores it, or adds a guard with a pool body while the entry is still in use. | [M 0x137cb0 builds a new object per spawn; 0x137e68:145-172] |
| Reinforcements waited for a corpse to fade (up to ~17 s). | The alive scan read `present`. A slot now frees at the death. | [M 0x137e68:96-124] |
| Mode-3 groups (all seven spawners on 07000011) sent one guard, then waited for him to die. | The +0x1C permission stays set in mode 3 until the wave rolls over. | [M 0x137e68:176-203] |
| Guards could spawn right beside Bond. | p8 is an integer converted to a float, and the port read it as raw float bits (1.4e-44). | [M 0x1379c0-0x1379d4 `cvt.s.w`] |

## Real paths confirmed (tests, no game code)

- **0700000d crane boarding.** The walk no longer places Bond on the gantry. From the control
  corner #460 the claw reaches pulley #370 and Bond hangs. At the east apex he drops onto the east
  girder and the aboard plate fires ch 0x67, a Bond-moment channel; sound trigger #516 sits beside
  that spot. The ride and the drop into the yard follow on real input. The earlier conclusion that
  no route existed never tried the claw.
- **07000011 vent route.** The ledge, the claw to #265, the bridge roof, grill #312, the duct, grill
  #84, plate #235, then plate #283 unlocks door #213 from inside. New test
  `the_vent_route_reaches_the_storeroom`.
- Both levels' objectives, channels and scripted scenes: writers and readers read against the
  decompile, with no further divergence found. Read in depth this round: the spawners, the
  DefaultInit ranges, the hints, every level-specific branch and the claw. Doors, triggers, locks
  and the rest were spot-read, not re-derived line by line.

## Found but held back (not on this branch)

- **Guard ranges.** `NDrone2_DefaultInit` overrides the engage/combat range: 07000011 sets 6/9 (5/8
  for anim set 9), and 07000014 sets engage 10 (7 for set 9; set 5 keeps its draw) [M
  0x14b300:429-457, raw 0x14bb60-0x14bc1c]. The port lacks both. With them ported, the port's
  corridor guards close to ~4 u from Bond, where the fresh-boot capture has them at 5.8-9 [L
  docs/live/2026-11-01-07000011-start-corridor-guards.md]. So the port's approach rule diverges
  somewhere downstream, and that rule needs fixing before the ranges go in.
- **The claw latches through walls and floors.** The pick has no world test, while the game's bolt
  is a flying bullet [M Bullet_Delete 0x12e660:50-55]. A bullet-ray check broke 07000014's
  pinned SkyBridge latch: the ray hits a down-facing mat-5 face 0.13 above Bond's head
  (40.82, 3.73, 65.14). Whether the game's bolt stops there is unproven, so the change is reverted.

## Tests

- **New, red on the base and green here:** `spawner_waves_live` (4 tests). The base gave 5 guards
  where the game gives 15, 1 where it gives 2 in a fill, and a distance of 1.4e-44 instead of 10.0.
- **Route pins, green on old code too:** `the_vent_route_reaches_the_storeroom` and the walk's
  boarding.
- **Pins corrected because they encoded the old behaviour:** `spawner_kill_live`,
  `interactables_07000015_live::the_east_room_touch_wakes_its_squad`,
  `qclaw_reels_to_the_crane_pulley` (it stood under a deck) and `grapple_point_reels_bond_up`
  (it fired through a ceiling).
- **Green here:** walkreal_07000011 (5/0, 00 Agent included), walkreal_0700000d (1/0/2i),
  interactables_0700000d 11/0, interactables_07000011 17/0, start_corridor_guard 8/0,
  spawner_kill 5/0/1i, interactables_07000015 17/0, and the walkreal binaries for 01-05, 07, 09-0b,
  12 and 4a.
- **Ignored with the measured reason:** `walk_arrival_to_the_c5_door_real_damage`. The boarding
  hang and ride are in the open: the walk reaches the west park at 13.2 health and the drop kills
  him. Snipers #29, #15 and #32 need stands first.
- **Red here, green on the base 10956badb** (the walks were tuned to the old rosters):
  - walkreal_07000006: both real_damage.
  - walkreal_07000008: 5 tests, including the plain walk ("Escape with Zoe" objective) and
    `the_ally_fights_only_the_guards_through_the_noise_path`.
  - walkreal_0700000c: real_damage (the camera switch reads 30).
  - walkreal_07000013: real_damage.
  - walkreal_07000014: both real_damage (the descent landing).
  - walkreal_07000015: real_damage_agent.
  - walkreal_07000016: both (ch 0x82 not within 3000 ticks). Group 2 is mode 2, max 1, p3 0, so
    it now respawns until kill ch 0x34, as the game's counter allows.
  - walkreal_0700001b: `walk_arrival_to_drakes_defeat` (one missile took the branch 3).

## Unproven

- The boarding and the vent route have not been watched in the real game. The question for a live
  check: from the corner #460, does the claw latch #370, and does the fall onto the gantry fire
  ch 0x67?
- A later-wave guard added beside a live template draws with a pool body (3 per skin). That body
  carries the skin's first placement's held weapon, so a mixed-weapon group can draw the wrong gun.
  No render was taken. A spawn waits when no body is free.
- The 08 plate test and the 0c camera assert above may hide more than roster drift. They were not
  traced.
