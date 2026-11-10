# los-mask-repairs — report

**This branch moves the shared random stream** (guard sight decides which guards engage and
when). Keep it on its own batch.

Branch `s/los-mask-repairs` = `origin/s/level-0b-roof-sprint-6` (the guard sight masks on,
`LOS_MASKS_ON = true`) merged with `origin/master` (one comment-only conflict in
`walkreal_0700000b_live.rs`, master's text kept) + three repair commits.

## The cause of the named reds

With the 0xa27 mask and the target-cel scope on, the port's guard sight had no cel walk:
`DroneVision_CanSeeObjectFrom` 0x176f18:65-67 calls `AINetwork_TestRayCels` 0x15ab78 whenever
the target's cel differs from the seer's, and a seer cel the straddle walk from the target's cel
does not reach means "not seen", with no collision ray at all. The masked ray passes class-0x10
paper and glass, and the scoped walk only tests the straddled cels' geometry, so guards saw Bond
through walls into rooms no portal on the ray reached:

- 07000012: the east-wing guard engaged across the floor and his round hit a civilian (the
  Killed Civilians fail ended the walk before door plc 545).
- Zoe's gondola: the ally took the fight her sight should not have had (1411 rounds on Bond).
- 07000001: the courtyard shoji ping-pong and the escort's interior doors.

## What changed (game code)

1. `CollisionWorld::ray_reaches_cel` = `AINetwork_TestRayCels` 0x15ab78 (the straddle walk the
   scoped queries already use, then the membership test). `Drone::seek_opponent_los` runs it
   per view bone before the ray, as 0x176f18:65-67 does (disasm 0x176fa8: `a3` = the seer cel).
2. The round robin (`ProcessDroneSight` 0x176a28:175) ends when a probe cast its ray
   (`DAT_00299092` moved); a probe the cel walk refused casts nothing and the walk goes on.
3. The cel walk starts from Bond's linked cel (`Bond::cel`, the target's obj+0x20, :21), not the
   scene's smallest-box lookup: at the 07000002 stair bottom the box lookup reads cel 230 and
   the linked cel 13, the courtyard sniper's own cel, so the walk from 230 refused his sight.

## Test-side repairs (each traced)

- `guard_rounds_glass_live`: the scene takes class-0x10 panes only. 07000003's #338/#339 are
  surface 0x05 [D the pane's collision materials]; the 0xa27 ray drops class 0x10 only
  [M 0x1ebaa8:314-316], so the guard never sees Bond behind them. Coverage is now 01 + 12.
- `escort_hide_findrun_live`: the hide's exit is recorded every tick (the exit can land inside a
  shoot leg; the re-anchor asserts are unchanged).
- `dead_body_collision_live`: the walk waits for the corpse's capsule to rest (a long death
  fall moved the body 1.3 u after the 90-tick beat; Bond walked at the stale spot).
- `fix_ally_death_live`: the blast fuse is 2 ticks. 07000001 #161 now sees Bond 2.3 u away,
  leads (AllyLeadInit → AllyLeadWait at < 3.0 [M 0x15e4f0]), sees the grenade and runs
  [M DroneFunc_HandleExplosives 0x1435f0] out of a 60-tick fuse.
- `fix_mission_live`: the shoji OPEN latch is read from the first walker tick.

## Tests run (release, `--test-threads` 1-4, final tree unless noted)

| binary | result |
|---|---|
| nf-game --lib | 761 / 0 / 4 ignored |
| zoe_gondola_realpath_live (named) | pass (red before) |
| interactables_07000012_live doors | FAIL (green with the cel gate alone; red again with rule 2) |
| fix_mission_live 01 exit | FAIL (escort killed in the ambush on the final tree) |
| fix_ally_death_live | 7 / 0 |
| fix_npc_doors_live | pass (red before) |
| escort_hide_findrun_live | 3 / 0 |
| dead_body_collision_live | pass |
| guard_rounds_glass_live | pass |
| sight_cels_07000001_live | pass |
| walkreal 04, 05, 06, 07, 09, 0a, 0c barrelrow, 0d, 11 route, 13, 16, 1b, 41, 41 wall, 4a, 4a satchel | green |
| walkreal 01 | 5 / 2 (the plain walk + the exit fade: mission restart before the plate) |
| walkreal 02 | 4 / 1 (the plain walk: geisha 1 killed by her captor) |
| walkreal 03 | 2 FAILED (d1 real damage, killing a servant) + the plain walk past the 590 s cap |
| walkreal 08 | 4 / 1 (Agent: the fusebox ch 137 shot) |
| walkreal 0b | 6 / 1 (Agent: died to #4 at 11 u) |
| walkreal 0c, 0c eastroute | the stair-head wedge (Bond at y 12.36 on the ledge leg) |
| walkreal 11 | 2 / 2 (lower floor d1 wedge + 00 Agent death) |
| walkreal 12 | 3 / 1 (Operative: the door plc 545 walk, same civilian fail) |
| walkreal 14 | 5 / 2 (tripbomb route + infiltration doorway wedge) |
| walkreal 15 | 5 / 2 (mine #447 set off by another cause) |

The walkreal probe binaries (walkreal_probe*, celprobe, dump) were not run. mp_perf_16_live was
not re-pinned (the battery is not game-correct yet).

## Open

- The new walk reds (03, 0b Agent, 0c, 14 infiltration, 15) are fights that moved with the
  game's sight rules; none is re-tuned yet. 0c and 15 stay red with rule 3 reverted, so they
  come from rules 1-2.
- The seer's cel is `find_cel(pos)`, not a relinked obj+0x20, and an unresolved cel skips the
  walk instead of refusing sight [S]. The scoped ray still takes the scene's box-lookup cel.
- Unproven without the real game: whether the 07000012 east-wing guard and the 07000001 ambush
  guards hit the civilian / the escort on these routes in the game.

## Round 2

Branch = round-1 tip `3c0b45bfc` + `origin/master` (`ef7822e8b`) + two repair commits. Still
moves the shared random stream.

### ProcessDroneSight 0x176a28, read in full — four port differences, all fixed

1. **The cursor.** `DAT_00299094 = drone+0x108` is written for EVERY drone the walk visits, before
   the eligibility test (:153). The port moved the cursor only on a probed drone, so a tick with
   no ray started the next walk at a different drone. Now `sight_round_robin`.
2. **The counter snapshot.** It is taken right after `FindAlertedDrones`, BEFORE the visibility
   loop (disasm 0x176ab4 `lhu fp,0x282(s2)` = 0x299092). `OpponentVisibility` 0x175af0:24-33 casts
   a head-bone probe for a guard under 2 u with a sight range; that cast ends the walk after its
   first visit. The port returned 1.0 with no ray and its walk never saw that cast.
3. **The sight bit.** Bit 2 of +0x228 is cleared only by `SeekOpponentLOS` 0x175cb8:26 (every
   +0x228 store in the decompile checked): an eligible guard keeps his sight until his next visit.
   The port cleared it every tick under 0.7 visibility. A value under 0.7 is stored as 0 (:107-117).
4. **The counter itself** is the shared ushort `los_probes` (the FindOpponent team-2 pick's
   counter); the stop test is the game's `snapshot < counter`, so a wrap does not stop the walk.

Budget: at most `DAT_00298e1c` visits (the drone LLIST count, `LList_Init(0x298e14, 0xd20)`).
Eligible: alive, vision bit 0x10000, opponent +0x170.

### The guard's own cel

`NDrone2_CanSeeObject` 0x1785e0 passes the guard's `obj+0x20`. That cel is linked at
`NDrone2_DefaultInit` 0x14b300:38 and relinked by `control_handle_cel_change` 0x133288 on every
moved tick, which `control_movement_object_handler` 0x1336f8 runs for every dynamic object after
`Drone_InitComms`. The port's guard now keeps that cel (`Drone::relink_cel`, the player's relink
law) instead of a fresh `find_cel` each probe. Every placed guard on every level resolves a cel at
spawn (census over `nf_view::dev::LEVELS`). Equal cels cast; a null cel on either side refuses
(0x15ab78:66-67); a harness player without `obj_cel` keeps the ray [S].

### Results (release, final tree)

Green: walkreal 03, 04, 05, 07, 08, 09, 0a, 0c barrelrow, 0d, 11 route, 13, 15, 16, 1b, 41,
41 wall, 4a, 4a satchel; interactables_07000012 12/0, fix_mission 4/0, zoe_gondola_realpath 1/0,
fix_ally_death 7/0, fix_npc_doors 1/0, escort_hide_findrun 3/0, dead_body_collision 1/0,
guard_rounds_glass 1/0, ai_live 37/0, nf-game --lib 776/0/4 (with `sight_round_robin_law`).

Red (fights moved by the exact walk; not re-pinned): 01 3/4, 02 3/2 (Agent walks), 06 5/1
(real damage), 0b 6/1 (Agent), 0c main 1/2 and eastroute 0/1 (Bond leaves the west ledge before
wires #282/#283), 11 3/1 (00 Agent), 12 3/1 (Operative), 14 4/3 (stuck at the landing west drop
at full health). 0c eastroute and 14 pass on master and stay red with `LOS_MASKS_ON = false` on
this branch, so the sight changes move them, not the masks. mp_perf_16 not re-pinned.

Attribution on 06 (env toggles, not committed): reverting rule 1 alone or rule 2 alone turns
its real-damage walk green, so that walk was tuned on the old walk order. The guard's own cel
and rule 3 alone changed nothing there. 01 stays red with rule 1 reverted.

### Open [S]

- `+0x3a0 & 0x30000000` adds a second `CanSeeObjectFrom` whose cast bumps the counter; no writer
  of +0x3a0 was found in the decompile, so it is left out.
- Team 2's sight probe is Bond's (the game probes the ally's drone opponent); team 2 skips the
  close-range probe.
- `Drone_FellOutMap` 0x13a878 on a relink miss is not ported (the cel is kept). On 07000001 a
  combat guard stepping down the courtyard cellar stair (-32.1, 3.0, 12.9) passes through the
  parapet's edge and falls out of the map; the game's guard would be deleted there.
