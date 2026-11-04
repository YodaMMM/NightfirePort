# WORKLOG (branch running copy)

## turret-round-spread (started 2026-11-04 13:27 UTC)

Branch `s/turret-round-spread` cut from origin/master (8deea246; the sandbox reset between
tasks — fresh clone, work-slim + the ISO re-downloaded). Task: settle from the decompile
whether Bullet_init 0x12c5a0's spread term (and the 3 shared-stream draws per shot, +1 in
GT_Update 0x180498:349) applies to owner -1 turret rounds (wall guns, the Ronin); port the
spread and the shared-RNG draws in the game's order if they apply; pin the hit pattern /
draw count (sister: dead-on). Run the gun-turret / ronin binaries, every *0700000a* binary,
cross_process_determinism and nf-game; quote the totals.

Progress:
- [13:27 UTC] Fresh clone; branch created; WORKLOG started. The decompile study next
  (work/decomp/Bullet_init*, GT_Update*), then the port and the pins.
- [13:35-13:52 UTC] The decompile study: Bullet_init 0x12c5a0's spread block is unconditional
  (the 3 draws every round; the owner arms scale only); GT_Update 0x180498:347-348 passes the
  turret obj (class 0x36 [M 0017ffa0:77]) as the owner with the fixed weapon def 0x2c55d0 —
  [L ee_snapshot @0x2c55d0: +0x24 = 4.0, +0x20 = 15.4833, +0x6c 0x1000 (range 300), +0x68 = 0]
  — the full 4.0 cone applies to the turret rounds; the pad draw at :349/:522 makes 4 draws
  per shot in the game's order. The port already carries it (01cac91a, 2026-11-01) — the
  premise was stale; this round verified the law and landed the pins.
- [13:52-14:35 UTC] The pins: bullet.rs's cone_draws_three_even_when_dead_on (the 3-draw law +
  the dead-on sister) green; gun_turrets_live's the_wall_gun_burst_scatters_and_draws_in_the_
  game_order — the 12-shot damage/draw golden sequence (2.4 flat; 11673..16667, +454/shot)
  green. The runs: gun_turrets 3/0, ronin 9/0, interactables_0a 13/0, keyboard_0a 1/0,
  walk_0a 3/0, walkreal_0a 12/0/3i, cross_process_determinism 3/0, nf-game lib 658/0/5i.
  The suite restarted on the final tree; the report written.
