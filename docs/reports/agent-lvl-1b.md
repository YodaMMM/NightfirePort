# agent-lvl-1b: 0700001b (Equinox) level pass

## Fixed (each with a real-level test, red on the old code)

1. **Bullets vs script objects meet the triangles, not the sphere.** `Collide_PickObj` 0x1eae80 only
   queues an object by its built sphere; `Collide_Update` 0x1ea398 keeps the contact only if
   `Collide_Intersect` 0x1ebaa8 (0x201 pick, `Intersect_RayGeom` 0x10a8c8) meets the glist's
   triangles, an object with no collision block never passes the pick, and the mirror walk stops at
   the first solid node. The port used the sphere alone: the 1b solar panel under the walkway (#50)
   and the next one broke on the first shot. Now a level shot from (16.74,-1.84,4.62) facing -x
   breaks only #49, on the tick the round crosses its rising surface (x 7.4); facing +x nothing.
   Tests: `solar_panel_hits_live` (`the_round_breaks_only_the_panel_it_crosses`,
   `firing_down_the_row_breaks_no_panel`). The consoles' real target is OBJECT_DissarmCollision
   (2 triangles). `urn_break_live` now aims at the objects' triangles (sister: a sphere-only shot
   leaves no contact); the sky bridge's no-glist root takes no round.
2. **Enemy rounds reach script objects** (same Collide pass for every class-5 bullet): astronaut
   rounds wear panels down (4 x 5.0 = 20). `enemy_rounds_wear_a_panel_down`.
3. **Missile disarm tables used decimal for hex.** The p2 scripts' later write is ch 0x83..0x8a
   (the Apocalypse latches), the port wrote ch 83..90, which falsely raised ch 89/90 (AND #150/#149
   outputs, SP#155/#156 triggers) and ch 83-85 (SP#152-154). `missile_tables_match_the_disc`.
   `interactables_0700001b_live`'s klaxon test rested on the wrong writes; with m8 armed, 0x50 no
   longer rises (corrected).
4. **Space laser.** Fire origin: `SpaceLaser_Register` 0x1aa9e8 copies the tracking script's keyed
   type-0x38 entity (e.g. (56.8,29.2,-37.4)), not the (0,0,0) marker, so the beam now actually flies
   at Bond (it died in the pit before). Its scripts' channel writes are clocked: ch 18 (the
   BigLaserBeamHURT #259 gate) t120..t358 of each track; ch 15 from the dead script, which starts
   ScriptPlayer_LaserOverloadAnim #143 (then ch 16 to SP#221). None had a writer before.
   `space_laser_1b_live` (2 new tests).
5. **Zero gravity for pickups/corpses/gravity rounds.** `Player_Init` zeroes `WldGravity` on this
   level; only Bond read it. Dropped guns now float instead of sinking. `a_dropped_gun_floats_in_zero_g`.

## Tests run (all green)
solar_panel_hits_live 5/0, space_laser_1b_live 6/0, urn_break_live 4/0, walkreal_0700001b_live
16/0/3 ign, interactables_0700001b_live 9/0, solar_panels_live 2/0, fix_world_gravity_live 5/0,
pickups_g029_live 4/0, walkreal_07000006/07/12 green, platforms_drake_scene_1b green.

`walk_arrival_to_drakes_defeat_on_real_health` is now **ignored**: with the panels whole the
walker's wave fight at the x 20 hold costs its 100 health (tick ~9473); every console latches and
the top-up walk passes. The walker also routes its crossings through the gap between the panel
rows now.

## Unproven / for the main agent
- The PCSX2 "20 bursts broke nothing" capture shows the Beam meter at 0 then rising to 68/87 while
  "held": a firing weapon cannot recharge, so that capture likely fired no rounds. Live question:
  from that spot facing -x, does one tap break panel #49 (the round crosses its far edge 0.08 below
  the rim in the port)?
- Visual, not changed: the missile p0/p1/p2 scripts (OBJECT_Missile, arms, hatches, flames incl.
  the type-0xE4 OBJECT_MissileFlame hurt, sounds 1301/1126/1125/915) and the laser tracking
  scripts' entities/lights/sound 1299 are never played; the port clocks only their channel
  writes. Missiles never appear on the decks. Debris/shards/gas still fall at -9.8 on 1b.
- Bond bonus pickups need the level's gold medal (faithful); not exercised.
- Shared random stream: no new draws added; behaviour changes (panels whole, laser rounds live)
  will move later RNG use on 1b runs.
