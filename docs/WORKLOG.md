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
