# 20-level-08-hind-gun-through-cabin

In 07000008's Hind fight (the gondola ride ends with Bond inside the carriage at about (111.1, 13.52, -179.7)),
the Hind's machine gun (copter.rs `fire_gun_tick` -> drone.rs `CopterEvent::Gun` -> `spawn_copter_bullet`,
weapon 0x61, -2.0 a round every 6 ticks) keeps landing on Bond while the Hind is SOUTH of the carriage
(z -193..-208) and a world trace from Bond's chest to the Hind (`segment_bullet_scoped`) is blocked by the cabin
wall (t 0.11-0.72). Measured on master 06db4a156 with NF_DMG_TRACE on
`walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent`: hits from frame 20332 every
6 ticks, Bond never able to launch a rocket because the same wall blocks his launch line.

Question: does the game let the Hind's gun rounds pass through the gondola carriage, or does the carriage block
them? Read `Copter_FireGun` 0x17d828 and the bullet's collision path (Bullet_init 0x12c5a0 / Bullet_update
0x12e7b0, Collide_Pick 0x1eb320) for which geometry a copter round tests - in particular whether the gondola
carriage (a moving object) is in its collision set, and from which cel the round starts. If the port differs,
fix it with a test that is red on the old code (a round fired from south of the carriage at Bond inside must
stop on the wall, or must not, per the decompile). Do not change the walk tests' tactics in this task.
Tests: the copter unit tests in nf-game, walkreal_07000008_live, interactables_07000008_live.
Branch: s/level-08-hind-gun-through-cabin.

Main 11-14 lead: Copter_FireGun 0x17d828 fires through the plain Bullet_init (shooter = the gun obj, weapon 0x61 on 07000008, sound 0x5df) - no special collision. The port (drone.rs spawn_copter_bullet) spawns a normal Bullet; its flight is cel-scoped from find_cel(bullet pos) each tick (bullet.rs ~298 segment_flight_contacts_scoped). From the Hind s cel (outside) the carriage wall may not be in the scoped set while Bond s inside-cel trace sees it - compare the game s Collide_Pick scope for a round crossing from the outside cel into the carriage cel (does the game test the destination cel / the portal crossing?).
