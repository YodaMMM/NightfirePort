### level-0a-office-hatch-5 (started 01:41 UTC, 2026-10-24)

Task: 0700000a — the post-spawn settle (the game's spawn ray from (−3.9319, 0.1, 10.3978) hits
y 0.0000 in the spawn cel, surface 0x15 [L fresh boot, Collide_RayIntersect breakpointed from
build_PointOnFloor] — the SAME floor the port finds — so Player_StandAtNewPosition stands 1.1000
in both, and the snapshot's 1.0891 [L ee_start_0700000a 0x18694e0] is the AFTER-spawn settle:
−0.0109 from the collision push against the floor at the capsule's cc 1.0347 band, the
feet-on-point settle, the anim-driven cc [M Player_Collision 0x194df0 reads AnimObject+0xcc
every tick]; the -0.0109 surface hunt is moot). Then the hall glide (game 1.1147 after the
doorway-base climb at x −8.304) and the corner at (−17.87, 4.87) (the game walks through at
y 1.127 → 1.120); un-ignore the three sisters and the main walk in walkreal_0700000a_live.

Plan: log the port's root per tick from the spawn with no input and find the lowering step;
port it [M]; then the hall glide line, the corner push landing, and the un-ignores.

- (03:15 UTC) The settle found and ported: the placement is 1.1000 in both ([L fresh boot] the
  breakpointed spawn ray hits y 0.0000, surface 0x15; [M 0x1c95a8 writes the hit back, 0x190d88
  adds 1.1f]); the snapshot's 1.0891111 (0x1b78640+0x34 = 0x3f8b67fe) reproduces BIT-EXACTLY as
  1.1f + f32(4·(−9.8f·(1/60))·(1/60)) — FOUR `vel += WldGravity·REC` adds ([M 0x194df0:107], the
  constants [L] 0x2d8894/0x30d0dc) integrating ONCE ([M 0x194df0:189-197]), then the band rest
  ([M 0x194188:106]). Ported as `Bond::settle_hold` (armed in `stand_at`): four held ticks, the
  release tick integrates the held velocity in the game's op order, the band catches. The anim
  path was cleared first: the stance script 0x06000a00 = one op-0 (seq 0400084f, 2 frames, root
  FLAT at 1.0108399 [D 07F0000a.bin]; cc = root + anim+0xD0 0.0238784 = 1.0347183 [L bit-exact]),
  so no root motion exists at the spawn. `the_spawn_height_matches_the_game_snapshot` un-ignored
  and green (4 passed in walkreal_0700000a_live). The glide/corner sisters re-noted (the settle is
  in; the remaining gap = the doorway base climb's +0.0256 settle — the port pushes off the jamb
  base (−8.604..−8.304, z 16.278 [D tris]) for +0.0036 only); the main walk re-noted with this
  round's run (wedged at the N-S lane leg, the segment hit 0.21 ahead). KNOWN REGRESSION: 05's
  walk_to_the_gatehouse_and_ride_the_truck misses the ch-0x70 plate with the settle in (the ride
  route needs re-calibration); nf-game lib 546 green, the 0a family green (walkreal 4/0, keyboard
  1/0, walk 3/0), walkreal_07000009 3/0. probes: probe_0a_settle.rs committed as the evidence.
