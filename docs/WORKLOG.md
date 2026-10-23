# WORKLOG

## level-0a-office-hatch-2
Started 2026-10-23 18:22 UTC. Branch s/level-0a-office-hatch-2 from origin/master a67f1a27e.
Task: 0700000a office floor → Q-Worm keyboard → hatch (round 2). The walk
crates/nf-view/tests/walkreal_0700000a_live.rs::walk_office_floor_qworm_keyboard_hatch is still
#[ignore]: the floor bridge is fixed; the walk still stops later on the route. The ignore message
names the residue: the walker's stuck→E-toggle closes the starts-open door #4 mid-leg and Bond
hangs grounded=false at a (+0.06 displaced) rest — the floor band [M Player_FeetOnPoint
0x194188:106: HITDATA+0xC − 0.1 <= BLData+0xCC] has ~1.5 cm of margin at rest, so the door-slab
push unhooks him. Same threshold-strip class on 07000001 (x 4.4..5.6, z 8.6..11.0).
Plan:
1. Reproduce with the ignored test (real input only), capture the wedge dump.
2. Port what the game does at the blocker from the decompile [M] / disc [D] — the feet-on-point
   floor band and the door-slab's player push.
3. Un-ignore the test when it passes, with a sister that can fail. Live checks listed if a fresh
   PCSX2 number is needed.

Work log:
- (18:22) branch created, WORKLOG started; repo/ISO/work/queue-worktree present from the previous
  session (disk 2.0 G free).
- (18:30) reproduce: the ignored walk fails at ring leg 6 "down past the west door" — STUCK at
  (−17.87, 1.18, 4.87) target (−17.5, 1.3, 2.5), grounded=false, floor_ny 1.00, cel 182,
  floor_below ≈ 0.0000002, door#4 state 0x4 pos01 0.000 (open; the walker's use_door sees open and
  never toggles — the ignore message's E-toggle premise is stale on this master).
- (18:50) per-tick probe (scratch, removed): from t+16 a per-tick push (−0.0156, +0.0165, +0.0988)
  cancels the walk's south-east press; y dithers 1.1336↔1.2019, grounded flips each few ticks
  (pick_ref grows +0.016/tick). The contacts (named by replicating capsule_tri_contact per
  triangle): thin 1.1-tall plates at z ≈ 4.5−4.6 spanning x −19.5..−17.5 (the stair stringer,
  [D]) whose north face pushes +z, and a 0.734..0.838 step block at x −17.5..−16.59, z 4.43..4.79
  ([D]) whose west face pushes −x; the below-B edge contacts add the +y lift.
- (19:10) the band chain: Bond glides out of the lift at y 1.1336 over the y-0 office floor (the
  walk-off glide — the band [M Player_FeetOnPoint 0x194188:106] holds while the floor is within
  cc+0.1 = 1.1347 below), so his rest sits 0.0011 under the band's unhook edge. The corner lift
  crosses the edge → grounded=false → gravity → fall → re-ground → infinite dither, z frozen, the
  6000-tick cap. The game's own push tail [M Intersect_CylGeom 0x109da0 ~:412-470: v = q −
  HITTEST+0x28 (the A arm), push = v/|v|·(+0x4c − |v|)] produces the SAME negative-magnitude
  upward flip for below-span contacts — the lift is game-shaped; what the game does with it
  (the handler-side clamp 0x1949a8:136-140, or the slide the real player makes) is the
  un-pinned piece.
- (19:31) the un-ignore is NOT achieved; root cause instrumented, live checks written into the
  report. Baseline: the file's other 2 tests pass (the ring legs west_doorway_repro walks clean).
  Finishing blocked — the next message continues from this branch.
