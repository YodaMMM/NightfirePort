# agent-lvl-13: 07000013 object sweep

This branch moves the shared random stream on every level with a spawner. Merge it with the stream batch and one re-pin.

## What a player notices

On 07000013 the first guard killed (a ch 0x2A or 0x29 death channel, p12 [D]) brings the lift squad in. Spawner #327
(mode 3, max 4, waves 0 = 32000) now sends all four guards, and sends four more after the wave falls. Before the fix they
arrived one at a time, and a later wave brought back the dead record instead of a new guard.

The rule is in shared code, so every level's spawners change: mode 3 waves fill to max (08, 11, 12, 15, 06), and later
waves on any level come alive.

## Fixes [M]

- `DroneSpawner_Control` 0x137e68: the +0x1C latch. Mode 3 keeps it up until the rollover, so a wave fills; mode 2
  re-decides it every tick.
- The slot scan: a guard leaves the wave at health <= 0, not when the corpse fades.
- The slot walk: every empty slot gets an attempt in the tick.
- `DroneSpawner_SpawnDrone` 0x137cb0: the draw is over the template count (+0x2E), and a draw past the usable templates
  fails. Each spawn starts from the saved template.
- A spawner with no template on this difficulty deletes itself (0x137aa0:16-19). It could otherwise write a done channel.
- 0c/0d: a second stinger arm drew and played on wave 0. Disasm 0x1382fc is `beql` on waves == 0, so wave 0 is silent.
  The arm is removed.

## Docs corrected

- Ch 0x29/0x2A have writers: the guards' death channels. The docs said none.
- Drone #326's p6 = 0x57 is a read channel, not an alarm writer. Ch 0x57 has no writer, and it is off on the real
  12 -> 4A -> 13 chain [probe], so sound triggers #410/#411 stay silent in the game data as well.

## Swept with no divergence found

Level-gated code for 0x13 (talk sets, grenade drop, watch clock, weapons, tuning, music), Hint, Rotor, SP_Update and
SP_Create against the port, Script_EventHandler, the load-level trigger, AI point matching, the carried channels. All
four objectives complete on the walked route with health top-ups.

## Tests

Red on old code: `spawners_07000013_live` (wave count 1, wanted 4). Green on the fix: that test and `spawner_kill` 12/12
(the three new unit tests are the latch, the draw and mode 2).

Run on the fix, green: gate13, walk_07000013, interactables_07000013, guards13_wake/start, guard_barks_07000013,
walkreal_07000001/05/0a/0b/12/4a, enemies_07000012, corridor_guards_07000012, fix_actors, fix_finale, fix_progress,
guard_stuck_sweep, door_wait_07000014, interact 01/05/0c/4a, interactables 06/08/0c/11/15/4a.

Red on the fix, need the re-pin:

| Binary | Test | Symptom |
|---|---|---|
| walkreal_07000013 | both real_damage walks | Bond dies; the topped-up walk passes |
| walkreal_07000008 | real_damage agent, 00agent | Bond dies at tick ~23700 |
| walkreal_07000011 | real_damage_00agent | wedged at the top flight, 6000-tick cap |
| walkreal_07000015 | real_damage_agent | Bond dies |
| walkreal_07000006 | castle interior real_damage, real_damage_agent | first panics in walker.rs:283, the Bond-died path |
| walkreal_07000014 | real_damage, real_damage_agent | descent landed off the route; west drop never cleared |
| walkreal_0700000c | real_damage whole mission | ladder climb never mantled |
| walkreal_0700000d | real_damage | health 19.6 at the fall check, wants > 40 |
| walkreal_07000016 | chasers_punch, chasers_defeat, chasers_defeat real_damage | no punch; mission status 2 at the wait; death |
| walkreal_0700001b | drakes_defeat | one of 8 missiles took the other branch |

Not finished: `walkreal_0700001b` as a whole (`drakes_defeat_on_real_health` was still running when this was written).

## Unproven

- None of the red walks is traced to a cause. More guards and a shifted stream fit them, but a real break is not ruled
  out, and the 16 and 1b rows are not health walks. No master baseline was run in this lane.
- A wave's size has no live capture. It rests on the decompile and the disasm at 0x1382c0-0x138334.
- `SpawnDrone`'s `Rand_Rand(10000)` into obj+0xEC is still not drawn.

## Known limit (draw work, not made here)

A template owns one drone record and one drawn character. The game makes a new object and the corpse stays. The port
reuses the record once the corpse has gone, so a wave can come up to about 17 s late while bodies stay in view. An extra
character binding per spawn would close this.
