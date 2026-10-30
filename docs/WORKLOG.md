
## level-0a-hall-glide-settle

- (10:14 UTC, 10-30) Branch created from origin/master (06b95819). Task: un-ignore
  `walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game` — the route crosses the
  doorway at the game's rate but the settled y reads 1.1201 vs the game's 1.1285 [L 2026-10-26];
  the missing mechanism is the hall glide's +0.0256 climb settle (the feet-band walked-dy growth,
  "band-settle finding 1"). Port it from the decompile [M Player_FeetOnPoint 0x194188 and its
  callers; cite lines], un-ignore the test, keep every other test in the file and every walkreal
  binary that walks stairs/ramps green. Sandbox kept repo/ISO/queue/target from the level-14
  session; work/ re-downloaded (work-slim.7z); branch pushed as checkpoint.
