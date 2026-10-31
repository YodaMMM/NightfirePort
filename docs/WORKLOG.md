
## level-14-door-guard-release (started 07:49 UTC 2026-10-31)

Task: make all the released 07000014 door guards reach the locked door —
`crates/nf-view/tests/door_guards_locked_door_07000014_live.rs` (~line 90) is #[ignore]d with
two blockers; take blocker (2) only (NoBunching scan flip is 20-drone-nobunching-side-bits, do
not touch NB_SCAN_LIVE): after the release only ONE guard reaches the door in the port (#25,
cycling at (-50.1,41.7)); #18/#19/#20 stay in state 0x38 near (-51.3,32.4)/(-47.9,33.5)/
(-40.2,25.2). In the game two guards stop 2.4 apart at (-51.6,41.2)/(-51.2,38.8) [L 2026-10-30,
in the test note]. Trace the 0x38 state handler and its exits plus the release writer, fix the
port so the released guards route to the door as the game's do, pin it with a test, run the
touched crates plus every 07000014 test binary. Branch `s/level-14-door-guard-release`, from
origin/master 7d0091a0.

Progress log:
- 07:49 branch created from origin/master 7d0091a0 (sandbox reused from the level-02 session:
  repo, work/, ISO, nf-queue and cargo all present; freed overlay disk for the rebuild). Branch
  pushed with this entry.
- 08:20 the 0x38 handler is GAME-FAITHFUL — the three stay is not the bug. [M 0x161560
  NDrone2_DSTATE_AmbushWait] exits: msg 3 (UPDATE) leaves 0x38 only on the opponent-sight bit
  (drone+0x228 & 4, HaveOpponentSight 0x175ef0:46) or player within 3.0 2-D; msgs 6-9/0x17-0x19
  go through DroneFunc_HandleImpact; msg 0x21 ConsiderExplosive; everything else falls to Global
  (0x15b830) and is dropped — NO channel reads anywhere in the state. The release channel 0x8C
  (mux #717: 0x87 AND 0x88 → 0x8C [D audit :1411]) is the trio's wake channel [D audit :73-76
  p3=0x8c]: GetDroneTypeAttackTypeFriend 0x14cbd8 sets +0xc5 = GetDTYPENEW(drone, +0xc7,
  +0xc9) [M 0x14cbd8] — with p15=8/p16=0/p21=0 [D audit] the trio's prop 0x10 (AMBUSH) routes
  them to mode 0x10 → start 0x37 (DroneTypeSettings row 0x10) → AmbushInit → crouch 0x38, in
  the game AND the port. The game's door pair (CUR 0x5f / PREV 0x5a [L 2026-10-30]) can only be
  the released full-alert guards (prop 0x10 = 0, alert 1.0 → mode 0 → start 4 → the +0x50c>=1.0
  fold → 0x56 → combat route → the A* locked-door gate 0x1571b0:262-271 → 0x5a → 0x5f).
- 09:05 reproduced the real divergence with a scratch probe (zz_probe014, deleted): after
  ch 0x87+0x88, the port spawns only SIX guards — the trio (g18/g19/g20 at 0x38, alert 0.66),
  the group-9 pair (g23≈#778 (-30.6,13.4), g24≈#780 (-23.5,36.0) both 0x5f→0x73/0x74 cycling at
  their spawn area), and g25 = #783 (-37.7,25.9) which walks west and holds 0x5f→0x73/0x74 at
  (-50.05,41.66) by the door — the port's one door guard. The disc's group 0xd has SEVEN
  oDrone templates [D audit :73-76, 85-87: #721 #722 #723 #724 #783 #784 #785]; the port's
  drones.list (37 entries) has NO entry at #723 (-32.78,28.48), #784 (-36.18,27.76) or
  #785 (-31.74,25.16) — they are absent at load, so spawner #1333 (max 7, waves 1 [D audit
  :1531]) can never release them and only #783 ever routes. The spawner mechanics check out
  otherwise: +0x1F screens [M 0x137aa0:33-45], trigger writes +0x0E=0/+0x1E=1 only [M
  0x137e68:53-64], the visible() abort [M 0x137cb0:43] mirrors Drone_PositionVisible 0x13a7a0.
  NEXT: why Drones::attach_level/the level parse drops the three oDrone placements (group-d
  attach → templates), fix the attach, then pin (guard count at the door, their states), run
  door_guards_locked_door_07000014_live + every 07000014 binary + the suite.
- 09:15 the report: docs/reports/level-14-door-guard-release.md (the refuted premise with the
  full [M] chain, the missing-template finding with the probe dump, what's left, two live
  checks, the chunk_suite resume command). Fix + pin test + 07000014 binaries + suite left to
  the next message; branch is current.

## level-14-missing-group-guards — started (in progress)

- 09:55 UTC branch s/level-14-missing-group-guards created from origin/master b44ac857 and
  merged s/level-14-door-guard-release (continue from its report: #723/#784/#785 absent from
  the port's drones.list). NEXT: first rule out Drone_Create 0x136be0's difficulty gate
  (parameter 2 = level tag vs GameState+0x40; port mirror ghost-guards-990) on the three
  placements' [D] rows; if gated, pin present-at-higher/absent-at-lower; else fix the
  parse/attach drop; then pin the release in door_guards_locked_door_07000014_live and run it
  + every 07000014 binary + nf-game.
