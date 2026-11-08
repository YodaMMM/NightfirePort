# agent-lvl-0b-0c: Night Shift part 3 (0700000b) and Chain Reaction part 1 (0700000c)

**Moves the shared random stream** (the one-way blocker rule changes where Bond stands against
blocker panes on other levels): red on this branch, green on master — walkreal_07000003
`..._real_damage_agent`, walkreal_07000011 `walk_spawn_to_the_lower_floor_load_point_real_damage_00agent`,
walkreal_07000013 `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`. All three are
real-health fight walks; the 03 one passes with the rule switched off, and its log shows Bond resting
0.1 behind 2-tri blocker quads that the old code pushed him out of. The other level walk binaries
are green.

## What a player notices

1. **0700000c: the crawl out of the display vent works.** After the hinge burn and the drop into
   the vent, crouching south used to stop dead on the pocket's south face, which left the vent a
   dead end and the start walkway with no way off the level. The face is `ventblocker_01` #457
   (z -112.363, surface byte 0x40, normal pointing south into the hall). The game skips a face
   whose plane the query point is behind [M ASH_vecutil_Dist2Tri 0x109770 returns code 99;
   Intersect_CylGeom 0x109da0 skips on 99], so a body coming from the vent passes and the hall
   side stays shut. The cel portal #212-#285 fills the same opening [D block 0x21]. The port now
   applies the one-way rule to dedicated blocker meshes (objects whose every face carries the
   surface byte 0x40), not to every face: the general rule (NF_CAPS_D2) stays held back.
2. **Simple-script operate samples play at the object.** The operate arm read the position from
   the matrix's fourth column (zero), so 0c's hatch grind and 06's trap samples played at the world
   origin; a zero sample word now reads as none, as `SS_Create` 0x1a9c58 does (0c's secret door).

## Routes walked on input alone (test changes)

- **0c, whole level, no placements:** walkway, hinge burn, vent drop (plate #274, objective 1),
  the crouched crawl into the hall, staircase #359, photograph (objective 2), the east ledge and
  the 6.29 stair to the 2.37 deck, the ramp out of ST-1, the x 46 lane to the yard, ladder, ring,
  three wires, door #281, slot jump, the east ground, two crate hops, up the sloped support girder
  `obj_turb3_supportbeam02` #30 (0.8 wide, walkable), a sidestep onto the 10.85 strip, the deck,
  E at the gate, 0d arrival (objective 3). The earlier notes calling the deck a collision island
  and the display ground sealed were wrong: both are reachable.
- **0b, the balcony-to-exec leg, no placement:** the cab roof at the top of its travel, E on
  plate #327 works the exterior grill #326, the duct, E on the floor grate #98, the drop past
  plate #353 (ch 0x76) into the waiting room. The cab is not left from inside (it is a sealed cage,
  as pinned); its roof is the route.
- The real-damage walks of both levels keep their tuned legs (they are fight sequences pinned to
  the old timing) with their placements; they write no health, armour or ammo.

## Tests

- New: `interactables_0700000c_live::the_vent_crawl_passes_the_one_way_pane_and_the_hall_side_stays_shut`
  (red on the old code: the crawl stopped at z -111.813), and
  `fix_ss_operate_0c_live::the_operate_sample_plays_at_the_object_and_a_zero_word_is_none` (red
  on the old code: #544's word read 0).
- Green on this branch: walkreal_0700000c_live 3/3 (+1 ignored as before), walkreal_0700000b_live
  7/7, interactables_0700000c 7/7, interactables_0700000b 14/14, walk_0700000b 4/4, and the 0c
  set fix_0c_hatch, hatch_crouch, hatch_crouch_real, walk_0700000c, fix_level_0c, fix_0c_vent2,
  interact_0700000c, the eastroute and barrelrow walks, laser_watch_0c, walkreal_probe0c_lip,
  fix_ss_operate_0c, ss_operate_sound_06, nf-game ss:: and world:: unit tests.
- The other levels' walkreal binaries (every walkreal_0*_live except 0b/0c): green except the three
  real-health walks named at the top.

## Unproven / for the real game

- Whether the game's crouched body stops anywhere in the vent pocket (the port's now does not):
  the PCSX2 stand check at z -111.38 was standing only.
- Whether the sloped girder #30 is the designers' route to the gate deck (it is walkable in the
  disc's collision and the port's physics; no PCSX2 run on it).
- The SS use pick still tests the object origin, not the world sphere centre (obj+0x80): on the
  0b floor grate Bond has to face west (towards the origin corner) for E to take.
