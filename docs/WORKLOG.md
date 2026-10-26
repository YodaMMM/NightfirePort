
## level-1b-space-laser (2026-10-26, this window)

Progress:
- Branch from origin/master (dcc4a733). Task: port SpaceLaser_Update 0x1aa3f8 fully
  (states 0-4, the 10 s re-fire timer data+0x38, the charge script + sound 0x5de, the
  player raycast + Bullet_init with the weapon def @0x2c6154, the beam object (control
  idx 0x10) + Emitter_Create + 3D sound 0x46a, the ch-99 (data+0x40) state-3 write, the
  ch-14 disable -> obj+0xfe kill), wire it at the dispatcher case 0x38 @0x1d06fc
  (control_funcs row 0x50), expose the beam endpoints per tick, tests: arms/charges/
  fires [M], ch 14 kills, ch 99 rises in state 3.
- LANDED: SpaceLaser_Update 0x1aa3f8 wired into the Drones control pass (drone.rs, next to
  the missiles): the state-2 shot raycasts the game's mask-8 ray (segment_bullet, 1000 units),
  rewrites the aim lane to the impact [M :153-160], spawns Bullet_init's record-107 bullet
  [M :172-175, def @0x2c6154 = weapon_data + 107*0x10c], pushes the class-0x10 beam as
  Drones::space_laser_beams (endpoints+alpha+life 5, Debris_Update 0x1aac00's decay), the
  0x2d6210 impact emitter via fx.spawn_impact, the 0x46a 3D sound + the 0x5de whine handle
  through Drones.audio; the script clock = the [D] NIS lib lengths (360/360/10), the load
  tick's own Script_Update counted [M :103]; state 4's delete drops the laser the same frame.
  FIX: from_placement now seeds cooldown = 10.0 [M 001aa8b8:38 — the creator's constant; the
  old 0.0 seed fired on the first gate tick]. The fire origin seeds from the placement
  position [D audit:275 the Fire Marker at (0,0,0); the register's entities are bare-created
  at the zero quad — M 0x1d8968/0x1d8d78].
- TESTS: crates/nf-view/tests/space_laser_1b_live.rs 4/4 GREEN (arms/charges/fires on the
  game's timings incl. the [D] 360-tick script + the 10 s re-fire; ch 14 kills; ch 99 rises
  in state 3 after the [D] 65 s re-raise; BigLaserBeamHURT arms through its ch-18 gate).
  Harness notes: the level start clears switch_channels ~frame 49 (the gate is re-poked per
  tick); the missiles share ch 19 and their fail chain writes ch 99 @t1000 [D] — inerted
  (state 4) in the laser tests. nf-game lib 578/0.
