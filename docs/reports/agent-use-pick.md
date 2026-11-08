# agent-use-pick: the use key picks objects the game's way

**Changes walk timing and the shared random stream.** A press now lands only from where the
game's pick allows: about 0.5-1.9 off the object's face, looking at it. Doors open a little later
on the walked routes, so the real-damage fight walks run on a different timeline. Seven of them
fail on this branch: walkreal_07000001 `..._real_damage_00agent`, walkreal_07000006
`..._real_damage` and `..._real_damage_agent`, walkreal_07000008 `..._real_damage_00agent`,
walkreal_0700000a `..._real_damage_00agent`, walkreal_07000015 `..._real_damage` and
`..._real_damage_agent`. Each one dies or stalls in a fight. I did not rerun them on master. Every
other test in the 82 binaries listed under Tests is green.

## What the game does [M]

`Player_Activate` 0x1a5170:100-108 builds one sphere of radius 1.0 (asm 0x1a5278) at
`Player_GetHeadPos` + the view direction. It then runs `Collide_SphereIntersect` 0x1ec940.
An object stays in the hit list only when it passes two stages:

1. **The world-sphere candidate:** `|centre − obj+0x80| < 1.0 + obj+0x8C` (strict), from
   `Collide_PickObj` 0x1eae80. obj+0x80 is the glist's local bounds centre run through the object
   matrix (`Control_BuildWorldSph` 0x133070:33-48). It is not the matrix translation.
2. **The refine:** `Collide_Intersect` 0x1ebaa8 moves the centre into object space. For mask
   0x101 it presets the node distance to 1e8 (:186-189) and runs `Intersect_PointGeom` 0x1e92d8
   on the glist's collision triangles. A triangle counts when the signed plane distance is in
   (0, 1.0] (front side only, since HITTEST+0x9a is 0) and the clamped point-to-triangle
   distance is ≤ 1.0. With no such triangle the distance stays 1e8 (:331), and 0x1ec940's
   finalize frees every 1e8 node (`if (fVar10 == 1e+08) LList_Add`).

The class walk then activates every node, with no break. The pick itself has no facing cone.
The only facing rules belong to classes: the lock arm's `Mat_GetDir(lock+0x90) · (player+0xC0 −
lock+0xC0) < 0` refusal (0x1a5170:158-166) and `SP_Activate` 0x1dc338's switch-link dot.

docs/systems/door-use.md said the refine "cannot reject the candidate". That reading looked only
at `Collide_Intersect`'s return value and missed the 1e8 drop. Both docs are corrected.

## What a player notices

- **0700000b floor grate #98 (and its twin on 07000011, #84):** E is no longer tied to facing
  west. Standing on the grate, E works from every facing when Bond looks down at it (about 55° or
  more). Looking level, the use point is 1.8 above the grate and the game refuses. The old port
  measured from the grate's corner origin and took E only facing west.
- **Every use target is measured from its middle and must be looked at from in front.** This
  covers doors, SimpleScripts, keypad locks, Q-Worm PCs, monitors, mounted guns and script
  entities (lift and elevator buttons, the keyboard, the fax). E no longer works through walls
  or from behind a panel. Examples: 0700000a's security keyboard was reachable from the corridor
  behind the cab wall, and 07000002's SS #272/#308 were reachable from their back faces.
- Monitors had a 2.2 head-distance stand-in. They now use the same pick.
- Locks had an unrotated centre and tested facing from that centre. They now use the posed
  sphere, and facing is tested from the matrix translation [M].

## Code

- `crates/nf-game/src/usepick.rs` (new): `candidate`, `refine` (scale-aware inverse), `picks`,
  `picks_in`, plus `Target` and `Placed` for the placed classes. Unit tests are included.
- `ss.rs`: `Ss.centre` (the local sphere centre) and `world_sphere()`. The SS arm in `scene.rs`
  (local and net paths) uses the shared pick. The net path's old "nearest within 2.2" probe is
  replaced, and it activates every hit, nearest first.
- `door.rs` `player_use`: adds the refine. `drone.rs`: lock and monitor arms, plus
  `Drones::use_picks`. `scene.rs`: the mounted-gun arm.
- `useobj.rs`: the SP entity arm and the switch-through-SP arm refine against the entity glist
  (`nis::World::use_shape`, `Player::entity_use_picks`). PCs carry `pick_matrix` and `pick_hash`.
- The direct switch walk (`near(s.pos) ≤ 1`) is unchanged. It is a port stand-in: the class walk
  has no 0x39 case. It is left alone and not removed here.

## Tests

- New `use_pick_census_live` (21 tests, all green). Every story level's E-armed SimpleScripts,
  closed unlocked doors, keypad locks, E-armed PCs and camera monitors are pressed through the real
  E path. Each object is approached from in front of each face its disc collision shape has (four
  compass bins plus floor faces on four facings), and the result is checked against the game's
  two stages plus the lock facing rule (refusals asserted). The row
  `the_0b_floor_grate_takes_e_from_every_facing_looking_down` covers #98.
- **Red on the old code:** 7 of the 21 tests (rebuilt against master's sources): 0b SS #98 and 11
  SS #84 from their east face, 02 SS #272/#308 taken from behind, 0a SS #663 and 12 door #658
  taken from behind, 14 monitors #838/#841, 02 monitor #323, and the grate row (old: level-west
  accepted).
- The nf-game unit tests: usepick, door, useobj.
- Tests repaired because they stood Bond at an object's origin, or E'd through a wall: the walker
  `use_door` aims at the leaf's live centre, closes to 1.85, waits out the swing and stops on a
  twin leaf. Others: fix_ss_grate,
  fix_door_react_path, level_end, use_key_census (monitor rows), interactables 01/02/0a/0b/11/12/14/4a,
  keyboard_0700000a, garage_lift_calls, walkreal_0a (keyboard), walkreal_13 (panel aim + E pulses).
  The shared helper is `tests/common/usepick.rs`.
- Also repaired: walkreal_0a (the keypad #648 E goes from a stand in front of the keypad and
  checks that it is #648's keypad, not a neighbour's) and walkreal_08 (the ally walk drops the
  "station doors" stop, as the whole-mission walk already does: that leg dropped Bond into the
  pit).
- **Final tally (82 binaries):** every story level's walkreal binary, every
  interactables_*/interact_* binary, use_pick_census, use_key_census, doors_remainder,
  door_glist0_e_dead, fix_door_react_path, fix_ss_grate, level_end, hud2, gunimp_mounted,
  0700000d_gunimp_door/aim, elevator_buttons_09, garage_lift_calls, kiko_button_real_walk,
  pc_screen, keyboard_0700000a, keypad_unlock_lod, fix_safe08, party_door_lock_07000007,
  decryptor_07000012, decryptor_hold_real_path, campaign_flow, fix_level_chain, fix_mission,
  fix_npc_doors, doublecross_transitions, level_carry, mayhew_end_transition, mp_world_sync,
  walkreal_07000011_route, walkreal_07000041/4a. All green except the seven real-damage walks
  above. nf-game lib: 694 passed.

## Unproven / for the real game

- **0700000a's security keyboard** faces into the security room (screen normal −z [D]). The
  walks used to press it from the corridor behind the cab wall. They now place Bond once inside
  the room: the room's door #725 (lock ch 0x31) is not walked yet. That route is a level task.
  Question for the real game: is the keyboard usable from the corridor? The code says no.
- **0700000b #98:** whether the real game takes E standing on the grate looking level. The code
  says no, because the use point is 1.8 above the grate.
- **07000009 car-park call station:** the button art is a sub-script entity (owner 0, not a target
  [M]). The press must land on the outer door leaf 0x020006a9, and from directly in front of the
  button the leaf is 1.06 from the use point. Question: where did the earlier live press stand?
- **07000007 straddle doors #244-248:** the pick keeps them from their broad faces, yet the door
  does not move in the port. A direct `Doors::activate` predicts it should. That is a door-class
  matter (the census lists it and does not fail on it). Not investigated.
- **4a lift riders:** from the car's centre the head never comes within 2.25 of the landing doors'
  faces. The test now steps the rider to the car edge. Whether a rider in the real game can E
  those doors from the car is open.
