# level-05-every-objective-2 — report

Task: un-ignore `walkreal_07000005_live`'s cable route and finish it on real input — walk the
west ledge, jump-grab the cable at the icons, ride it hand over hand, drop off, then continue
the mission; fix any port gap the real leg hits [M/D]. Branch `s/level-05-every-objective-2`
(from origin/master 531c08fb).

## What was already on master, verified this session

The un-ignore happened across level-05-west-lane-3..8 and level-05-zip-door: **all 9 tests in
`walkreal_07000005_live.rs` are un-ignored and the cable crossing itself is walked green** — the
west ledge/lane, the notch jump-grab at icon #310 (`jump: t % 6 == 0` through the ctx-3 zone
[M Player_HandleJump 0x1916b0:126-147]), the strafe-driven hand-over-hand ride ([M Player_Wire
0x1977c0: the strafe axis picks the stroke]) and the action-press drop ([M 0x197748-0x197834]).
This session's first full run of the file reproduced it: **9 passed; 0 failed; 0 ignored**
(94.87 s, slim ISO, `cargo test --offline --release -p nf-view --test walkreal_07000005_live --
--test-threads=1`).

## The real remaining gap: "continue the mission" died at the band's north seal

`the_cable_landing_walks_the_south_walkway_toward_the_zip_door` ended on a pinned negative: the
wall-hug band's north seal at z −20.5 (level-05-zip-door's blocker, two candidate answers
pending a live check). This session answered it **from the disc's own data, no live check
needed**: the seal is real but OFF-ROUTE — the player's lane leaves WEST over the window ledge,
which the previous sessions' lanes (x −23.7..−32) never tried.

Evidence chain (probe sources committed under `crates/nf-view/examples/`):

- `probe_sealtris05.rs` — every collision triangle in the seal box (x −36..−24, y 33..40.5,
  z −22.5..−19) with normals [D the disc's tri set via `level.instances(true)`]:
  - the approach slopes up southward (n(0.35,0.89,0.30), 33.42→35.34 over z −16.7..−20);
  - 45° ramps carry it to z −21.0 (n(0.09,0.86,0.50) / n(−0.06,0.70,0.71), top ~36.0-36.55);
  - the band's north parapet has two NOTCHES (x −31.75..−30.25 and −26.5..−25.24, tops 36.86)
    — but each notch is closed by a PILLAR on its rim (x −31.38..−30.63 / −26.38..−25.62,
    y 37.0..38.78 at z −21.24), leaving sub-lanes 0.12-0.38 wide against the walk capsule's
    1.1 diameter;
  - `probe_sealwalk05.rs` confirmed the west lane's hard wedge at z −20.45 (the ramp/pier
    contact pair) and the east lane's wall at z −19.45 (the disc's own face 34.20..34.98).
- `probe_ledgewalk05.rs` — the bypass: the approach's west end (x −35..−34) runs onto the
  **window ledge** (the disc's floors 36.0-36.43, x −36.26..−34, z −22..−19.13, n(0,0.99,−0.15))
  and walks its whole south run to the crawl prompt #85 zone — five legs, plain full-forward
  real input, no falls, no jump needed (the ledge's z −20 "lip" is the 45° slope
  n(0,0.71,0.71) whose north edge rides under the capsule's bottom sphere).
- `probe_crawl05.rs` — the crawl: the type-6 prompt ctx 0x6 arms along the ledge's south end;
  the E press (one edge per 6 ticks, [M Input_Action mask 4]) with Bond facing ALONG the crawl
  axis (yaw π; the #35 band runs z −26.3..−37.0 [D placement yaw 1.571]) enters substate 7
  ([M Player_Activate 0x1a5170:66-97 → Player_CollCreepWall 0x196758]) — snapped to the band's
  north end (−34.10, 37.00, −26.29) — and the forward crawl ([M Player_Creep 0x1969d0]) runs
  0.02/tick for 490 ticks and exits at the band's south end (−34.38, 37.00, −36.07).

**No port gap was hit**: master's collision, prompt arming and crawl carried every leg as-is.
The seal was a route misread, not a missing behaviour.

## What changed in the repo

`the_cable_landing_walks_the_south_walkway_toward_the_zip_door` (walkreal_07000005_live.rs):

1. The seal negative is KEPT (those lanes are [D]-sealed; the pin now cites the tri anatomy —
   the pillars, the pier wedge, the east wall) and is documented as off-route.
2. NEW real-input legs after the approach: the ledge walk (approach west end → the lip → the
   ledge south 1/2 → the prompt #85 zone), then the E press (yaw π, edge per 6) with the
   substate-7 assert, then the forward crawl south with the exit assert (z < −35.0, feet on the
   band). All asserts cite [M]/[D]/[L] as above.
3. A found press rule, pinned in the comment: a `walk_to::Use` press was REFUSED twice with the
   ctx armed because the act turns Bond toward its target point (north, off the crawl axis);
   the entry needs the player's facing on the band's axis [L: two refused runs vs the probe's
   yaw-π press]. Worth a [M] look later (Player_Activate's facing gate).

Result: **9 passed; 0 failed; 0 ignored** (94.87 s) on the final tree.

## What's left (the honest remainder)

- **The roofed corridor south of the crawl exit** (x −36, z −38..−44, floors 32.5-36.2 [D]):
  the probe's steer toward (−35.7, −38.6) wedged at (−37.88, −36.18) — the corridor mouth needs
  its own decode (the crawl exit stands at (−34.38, −36.07); the mouth may face east, not
  south). Then the second crack (z −46..−50, jumped), the south gallery, the SW loop, the
  SWITCH plate #269, the Q-Claw grapple over the moat inlet to pulley #281 (the draw/fire
  shape exists in gadget_grapple_live), the east stairs and door #112's E → the 0x1E → 07000006
  exit assert. All coordinates + cites are drafted in the test's doc header.
- The 06 open leg (the gallery → dressing-room LoadPoint #746) — untouched this session.
- Live checks for the main agent (unchanged from level-05-zip-door, still valuable):
  1. Fresh boot of 07000005: confirm the real game's window-ledge lane accepts the walk
     (poke Bond to (−34.6, 37.1, −18.6), walk the five ledge legs, screenshot at the prompt).
  2. The E press at prompt #85 in the real game: expected — the wall-hug view of #35 enters
     (the port requires the yaw-π facing; watch the real game's facing gate in
     Player_Activate 0x1a5170).
  3. The corridor mouth: fresh boot, poke Bond to the crawl exit (−34.4, 37.0, −36.1), walk
     south — where does the real game's corridor open?

## Suite

The fast runner (`tools/test/suite.py`, NF_ISO set) was launched in the background at 22:10
UTC per the brief: build ok (908 test binaries, 1142 s), then the serial run phase. Observed so
far in `suite.out` (untracked scratch): the 3 expected slim-ISO `disc_golden` failures, plus
`identity_disc::the_real_disc_passes_the_identity_check` FAILED and `CRASH mp_online_failures
(exit -9)`. The total line is quoted in the WORKLOG entry when the run lands.

**Suite total line:** to be appended (the run outlives this write-up; see WORKLOG).

## Usage

not visible to me
