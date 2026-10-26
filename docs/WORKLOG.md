
## level-1b-space-laser (2026-10-26, this window)

Progress:
- Branch from origin/master (dcc4a733). Task: port SpaceLaser_Update 0x1aa3f8 fully
  (states 0-4, the 10 s re-fire timer data+0x38, the charge script + sound 0x5de, the
  player raycast + Bullet_init with the weapon def @0x2c6154, the beam object (control
  idx 0x10) + Emitter_Create + 3D sound 0x46a, the ch-99 (data+0x40) state-3 write, the
  ch-14 disable -> obj+0xfe kill), wire it at the dispatcher case 0x38 @0x1d06fc
  (control_funcs row 0x50), expose the beam endpoints per tick, tests: arms/charges/
  fires [M], ch 14 kills, ch 99 rises in state 3.
