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
