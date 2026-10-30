
## level-0c-plates-reach-2

- (14:16 UTC, 10-30) Branch created from origin/master (a19be699). Task: close the ONE
  unmapped link from level-0c-plates-reach — the pocket→yard descent. Work the What's-left
  candidates: (a) a wider coll-scan west of x −4.5 / south of z −155.7; (b) the z −158.5
  wall's sloping top (4.55→6.69) as a walk with the drop at its west end; (c) what lies above
  the yard's north-west arm (x −8.25..−11.83 at z −166). If a route exists, rewrite leg 4 of
  walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff on real
  input with no teleport; if none, pin that with sister asserts. The walk must stay green;
  quote the totals. Sandbox kept repo/ISO/work/target from the level-0a-walk-clip-flip-2
  session; suite runner started in the background at 14:15 UTC.

- (15:50 UTC, 10-30) The descent candidates worked [D coll-scans + sim probe
  probe0c_pocket_descent (committed, #[ignore]d route record)]. FINDINGS: the prior scans'
  "pocket west edge x −1.8" was a radius artifact — the 2.37 floor runs west to x −8.25
  (z −143.11..−155.77) and IS walked (probe A1/A2: grounded to (−6.07, 3.50, −150.92)); but
  every edge is walled (x −8.25 / z −153.76 / z −155.77 faces y 2.37..6.15 [D scan_swdrop];
  sim B1/C2/C5 wedge). The z −158.5 "sloping top" is room_ex10_5's north-face top EDGE (a
  vertical triangle, no walkable surface [D scan_topbox]); the tower's real top is 6.69 strips
  (x −3.19..−2.40, −0.34..2.17, z −158.5..−159.74) over an open notch shaft (x −2.40..−0.34,
  floor y≈0 [D scan_pocketvoid]). A 7.24 ledge spans z −156.17 (spine 8.33) x −4.4..1.16 and
  catches every southward jump off the 6.27 floor (E1/G/K4 land there) and is WALLED south
  (I1/I2/I3, J1-J3 all stop at z −156.17) — the 7.24 route is a dead end. THE YARD SIDE IS
  REACHED: the 6.92 walkway (room_sk3, x −10.03..−5.65) has a cross-wall gap at x −10.03..−8.17
  (K1 walked south through it), its south part's west edge drops onto the room_sk3 ramp decks
  (4.52..4.83 — D4 jump-off LANDED (−17.35, 4.92, −150.95) no damage; K2 mid-drop), the ramp's
  south-west walk-off lands the y≈0 yard (F2: (−15.46, 1.08, −158.28)), and the tower top's
  south edge drops to the yard with a plain walk-off (H1: (0.97, 1.04, −162.86) no damage;
  E7 jump: (−8.51, 1.09, −173.72) no damage). THE ONE LINK LEFT: the 6.92 walkway north part's
  ENTRY from the display ground — the display floor walks west past x −3.8 at z −112.6 (L1,
  spine 7.40 grounded) and the walkway continues north past z −110.36 (L2 grounded at
  (−7.94, 8.06, −110.91)); the 0.65 step-up / junction spot is next. Leg 4 UNCHANGED this
  window (the stand-in stays); the walkreal walk is untouched and green (inherited).

- (15:40 UTC, 10-30) The full-route probe iterated (probe0c_pocket_descent::
  probe_the_full_display_ground_to_plates_walk, committed): the display ground walks west to
  (−0.53, 7.38, −110.35) [1 west 1-4 measured], the standing jump up onto the 7.24 platform's
  wide part LANDS (−4.48, 8.36, −109.22) [D room_storage01 7.24 floor, x −5.05..−2.04,
  z −108.67..−116.47], the platform's south rib (x −4.54..−4.37, z −109.80..−116.47 [D]) is
  walked to z −112.6 — and the rib→walkway crossing is BLOCKED in the sim: a parapet at
  x −4.84..−4.79 tops 8.67 (z −112.21..−116.70 [D scan_westentry]) walls the rib's west side;
  the west jumps bounce back onto the rib (3b × 3 measured); north of the parapet's end
  (z −112.21) the room_sk3 7.24 strip (x −5.65..−5.06) stops at z −112.24 and the walkway's
  east portion starts at z −111.86. SO CLOSE: the route is open from the display eye to the
  rib, and from the walkway south through the gap/west-drop/ramp to the yard — ONE bridge hop
  (the parapet at x −4.8) is unsolved in the port's sim. The leg-4 stand-in stays; the walk is
  untouched and green.
