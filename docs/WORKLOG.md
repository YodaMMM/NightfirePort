
## level-0a-hall-glide-settle

- (10:14 UTC, 10-30) Branch created from origin/master (06b95819). Task: un-ignore
  `walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game` — the route crosses the
  doorway at the game's rate but the settled y reads 1.1201 vs the game's 1.1285 [L 2026-10-26];
  the missing mechanism is the hall glide's +0.0256 climb settle (the feet-band walked-dy growth,
  "band-settle finding 1"). Port it from the decompile [M Player_FeetOnPoint 0x194188 and its
  callers; cite lines], un-ignore the test, keep every other test in the file and every walkreal
  binary that walks stairs/ramps green. Sandbox kept repo/ISO/queue/target from the level-14
  session; work/ re-downloaded (work-slim.7z); branch pushed as checkpoint.

- (12:05 UTC, 10-30) Fresh sandbox (the 10:14 one expired): repo re-cloned, branch checked out,
  work-slim/ISO/rust/alsa-prefix rebuilt. Read Player_FeetOnPoint 0x194188 + both collision
  handlers + Player_Move/SSWalk/Update + Intersect_CylGeom/Collide_Intersect with disasm.
  **No port landed — every statically-pinned y writer is ruled out for the flat glide:**
  the walk move is yaw-only ([M] Player_Move 0x191dd0:166-170 writes +0x10/+0x18/+0x34 only;
  the walk matrix RotMatrix(BL+0x30) at 0x192310:70 reads the pitch lane that Player_Update
  0x193600:134 zeroes each tick and nothing rewrites before the RotMatrix — the look writes
  BL+0x40/+0x44/+0x8a8 at 0x1a1d28:268-274, not +0x30); the SP body anim set is a static
  2-frame stub (set 0x2b91b0 = all 0x06000a00, its one op-0 command plays seq 0x84f frames
  1..2 [D ACTION.ELF @0x2b91b0 + the probe dump]) so no anim root bob rides the walk; gravity
  is band-gated off while grounded [M 0x194df0:104]. The press-y rides the radial tail's ref
  (HITTEST+0x50 = the lifetime DeltaP accumulator [M 0x1ebaa8:353-373], _DeltaP zeroed per
  Intersect_CylGeom call [M 0x109da0:136]) — on a walk the ref provably stays the boot (0,0,0)
  and every doorway contact pushes horizontally (press-y 0.0000, NF_CAPS_LOG trace). The
  settle needs the ref's live walked value — the settle-2 live check, still open. Measured on
  master: the glide settles 1.0905 (the ignore note's 1.1201 was the settle-5 tree);
  NF_WALK_KILL=1 and NF_WALK_CLIP=1 both leave the y at 1.09. Landed: the note re-pinned +
  assertion-free crates/nf-view/tests/band_settle_probe.rs (the set/script/seq dump).
  walkreal_0700000a_live 7 passed 0 failed 2 ignored.
