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
- (19:55) banked a partial suite run for the next session: the tree has no source deltas
  (docs only), so the state is valid — /home/z/suite_0a_hatch2_state.json, logs
  /home/z/chunk_0a_hatch2_*.out; chunk 2's 51 binaries all green so far. Re-invoke
  `python3 tools/test/chunk_suite.py /home/z/suite_0a_hatch2_state.json` until ALL DONE.

## level-0a-office-hatch-3
Started 2026-10-23 22:07 UTC. Branch s/level-0a-office-hatch-3 from origin/master 8b101249d,
merged s/level-0a-office-hatch-2 (docs only — -2 ended with no source deltas).
Task: the 0700000a west-hall glide height (the port 1.1336/1.1000 vs the game 1.1147) + the corner
wedge; un-ignore walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch.

Work log:
- (22:15) suite.py baseline started in the background (suite.out).
- (22:30) the slim work/decomp lacks the player functions (only the 0xa00000 region); extracted
  ACTION.ELF (2.36 MB) from the ISO and wrote a capstone MIPS disassembler (scratch, /home/z/mipsdis.py).
- (22:40) [L ee_start_0700000a.bin] the game's player position record at 0x18694e0 reads
  (-3.9319, 1.0891, 10.3978) — the game's spawn root is 1.0891; the port spawns 1.1000.
- (22:50) [D] the start #0 placement (map37 plc 0, flags 0x24) translation y = 5.55e-17 — both sides
  raise the spawn: [M Player_StandAtNewPosition 0x190d88 disasm 0x190ee0-0x190f18] build_PointOnFloor
  (ray, arg 3.0) then + 1.1f (0x3f8ccccd) — the game's spawn = floor_ray + 1.1; the port's
  Bond::stand_at = floor_below + 1.1 (player.rs:506). Same rule, DIFFERENT floor: the game's ray
  finds -0.0109 under the cab (1.0891 [L]), the port's floor_below finds 0.0000.
- (23:00) [D port world] fine floor grid + tri dumps: the port's world floor is 0.0000 everywhere
  along the route (cab interior, lobby, hall, corner); no 0.08 sill, no threshold at x -7.75; the
  doorway has roomjoin17 a/b (jambs) and a 0.1622-tall wall base at x -8.304 (z 14.3..16.3) IN the
  port's world; the SS cab dynamic floor is not in the port's static world.
- (23:05) [port] scratch walkout with the door wait: the port walks cab->hall at y 1.1000 FLAT
  (no climb, no ride) — the -2 session's "leaves the cab at 1.1336" did not reproduce on a straight
  real-input walk; the wedge dither remains the -2 finding at the corner.
- (23:10) [port] capsule_push probe at the x -8.304 wall base DOES lift (+y up to 0.15) when the
  capsule presses it — the port's push tail is not obviously missing the lift; the walk path's ease
  may have missed the base. Reading the game's Player_Collision + the +0xCC (anim root height)
  writers next to settle the rest model (strict contact 1.0347 vs anim-driven root).
- (23:20) [M Intersect_CylGeom 0x109da0 disasm] the push tail read: after ASH_vecutil_Dist2Tri
  (0x109770 called at 0x10a324/0x10a4fc) the game tests the Dist2Tri status (0xb0(sp) == 0/0x63
  skip, 0x10a32c/0x10a520), the ≤ 0 lane guard (0x10a568), dist ≥ 0, then dist < r (f3 vs
  HITTEST+0x8c at 0x10a5c8), push = dir·(r − dist) (0x10a5d8-0x10a5f8) accumulating HITTEST+0x10 —
  the same shape the port's capsule_tri_contact models; the VU0 macro pairs hide the exact
  lane math (the open disasm item).
- (23:30) [M Player_Collision 0x194df0 disasm 0x194e2c-0x194e34] **the cc is ANIM-DRIVEN**:
  `lw $a0, 0xdc($s3)` (the AnimObject) then `lwc1 $f22, 0xcc($a0)` EVERY collision tick — the
  root height comes from AnimFrameResolve (docs/fields/animobject.md:66, [L-read] standing
  1.0347). The port's cc is the constant ROOT_HEIGHT (player.rs:15) — matches standing but
  carries no walk-cycle bob.
- (23:45) [M Player_StandAtNewPosition 0x190d88 disasm 0x190ee0-0x190f18] the spawn rule:
  build_FindCel from pos+0.1f → build_PointOnFloor (the (0,−1,0)·3.0 world ray [M 0x1c95a8]) →
  y += 1.1f (0x3f8ccccd). The port's stand_at (player.rs:506) is the same rule over the flat
  static floor — the game's cel-scoped ray finds −0.0109 under the cab (the snapshot root
  1.0891 [L]), the port's finds 0.0000 → the port spawns 1.1000.
- (23:52) THE LEDGER CLOSES: the game's hall glide 1.1147 [L driver] = the spawn 1.0891 + the
  0.1622-tall doorway wall-base climb at x −8.304 (the base is in the port's world too [D tris];
  the touch at x ≈ −7.98, the driver's "climbs at x −7.75") settling via the band catch; the
  port's hall glide 1.1000 = its spawn height (its walked line missed the base). 1.1147 − 1.1000
  = 0.0147 = 0.0256 (the climb) − 0.0109 (the spawn). The corner: the port enters the lift
  dither at 1.1297 → 1.1957 (repro'd), the game at 1.1147 walks through (1.127 → 1.120 [L]).
- (23:58) LANDED: three #[ignore]d sister tests in walkreal_0700000a_live.rs pinning the [L]
  numbers (the spawn 1.0891, the hall glide 1.1147 ±0.003, the corner crossing ~25 frames at
  y 1.120) with the evidence in the ignore messages; the main walk stays ignored (the wedge
  needs the spawn-floor fix + the corner push landing). The non-ignored tests pass (repro
  3.5 s, wedge_dump 3.1 s). The full suite.py baseline was killed between tool calls (empty
  suite.out — background jobs die here); the touched binary ran green.
