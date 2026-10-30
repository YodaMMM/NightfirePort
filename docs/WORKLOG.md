# WORKLOG

### level-0b-spawn-to-end-walk (start, 04:56 UTC, 2026-10-30)

Task: 0b from a fresh spawn to the end. `walkreal_0700000b_live::walk_balcony_to_the_roof_jump`
starts from `load_warm()` (mid-mission). Add one walk from a FRESH load of 0b reaching the
mission's end on real input, every objective set by the game's own writers (no channel pokes;
reuse the existing legs' routes). Fix any port gap it hits with evidence [D/M]. Every test in
the file green; run the touched binaries and quote their totals. Branch from origin/master 814c8976.

### level-0b-spawn-to-end-walk (results, 06:35 UTC, 2026-10-30)

The fresh-spawn walk landed: `walk_fresh_spawn_to_the_mission_end` boots 0b FRESH (no channel
pokes), walks the shared legs (the cab ride boarded for real this run, the lock E, the Q-Worm
install, the roof cut-scene, the parachute jump) and latches the mission end in 37886 ticks.
The port gap the fresh walk exposed was NOT a gap: obj 9's real writer is the terminal script
06000035 stream 8's SPAWNED 0xEA TouchOnce (out ch 6, mask 4 [D nis dump; M 0x1d8968 case 0xEA]),
which the port's script-trigger rebuild already materializes — the fresh walk walks to it and
E-fires it (ch 6 → OR #295 → ch 0x4E). The carried records 1-5 stay OFF at the fresh boot
[M 0x182270 previous parts untouched; L ee_snapshot init values 0], so the monitored AND cannot
fold; the end rides OUT #304's own ch 100 write [D] + Mission_Update's latch [M 0x182c60].
walkreal_0700000b_live: 3 passed / 0 failed. The full suite ran chunked (the sandbox kills
background jobs): 61/916 binaries, 287 passed / 0 failed / 3 ignored before the window ended.
Report: docs/reports/level-0b-spawn-to-end-walk.md.
