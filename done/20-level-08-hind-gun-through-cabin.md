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

Main 11-14 decompile read: the drone/copter round flight is Bullet_update 0x12e7b0:223 Collide_SphereIntersect(r 0.1, cel = obj+0x20, ..., mask 0xa0a) -> Collide_Pick 0x1eb320, whose candidate set comes from Collide_StraddleCels (:125) - the swept test spans every cel the move straddles, so a wall between the outside cel and the carriage cel IS tested in the game. Check (a) whether the port s segment_flight_contacts_scoped straddles the same way for a round crossing INTO the carriage, and (b) whether the carriage walls are static or the gondola s moving geometry (dynamic) - the port s bullet pass may skip dynamic objects that the game s cel object lists include.

Main 11-14: (b) answered in part - world.rs segment_flight_contacts_scoped (~1932) DOES include the dynamic geo (and the missile set), and Bond s own chest-to-Hind segment_bullet_scoped trace stops on the carriage wall, so the walls are in the collision world. Next suspects: the 0x61 record s flags (glass pass bit 0x800 / a pass-through bit), the round s spawn cel (spawn_copter_bullet starts at the muzzle - find_cel(muzzle) may be None outside every cel -> the flat-world fallback), or the hit coming from a second muzzle (gun B, Copter_FireGun sets gun B +0xf4 = 2). Log one Hind round end-to-end (pos per tick, cel, contacts) to settle it.

CLOSED 11-14 (main): NOT A PORT BUG. Logging the Hind rounds end to end (weapon 0x61, glass pass on, cel 12 = Bond s cel) shows them entering through the carriage WEST DOORWAY (x 103 -> 111, z -179.6..-183, y 16 -> 14) - the gun has a real line down through the doorway, the walls are not crossed. The asymmetry is height: the Hind flies at y ~19-20 above the doorway s lintel line (y ~14.3) from Bond s spot, so Bond s upward launch line hits the lintel while the Hind s downward rounds pass under it.
