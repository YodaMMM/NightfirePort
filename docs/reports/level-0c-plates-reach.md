# level-0c-plates-reach — report

Task: the last 0c stand-in — replace the plates-reach teleport in
`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` with a
real-input leg, finding the game's own route to the plates ([D] placements + collision faces:
doors, SS movers, ladders, crouch gaps, the nav web; [M] any class relied on). Branch
`s/level-0c-plates-reach` = origin/master (06b95819, which carries the merged
level-0c-vent-pulley-class whose walk kept this stand-in).

## What's done

**The plates' spot is fully characterised and the route is mapped end to end except ONE link;
the mapped parts are WALKED under real input in a sim probe.** The walk's stand-in is
unchanged — replacing it was not possible this window; everything needed to finish it is on
the branch (the probe test + the scans below).

1. **The plates' geometry [D]:** the co-located touch plates #558/#559, box
   [52.24, 0.90, −174.45]..[53.29, 3.20, −172.73] (`nf-extract triggers`), hang over the
   warehouse nook east of the secret door #544 (51.93, 2.15, −174.36 [D placements]): the
   corridor strip west of the wall carries floor y 0.69 only to x 51.71 [D coll-scan], the
   warehouse floor (0.69/0.86) starts at z −176.08, and the nook's own pit floor sits at
   y 0.76..1.02 around z −170.8..−171.1 (x 51.91..55.61 [D]). Standing at the pit floor's
   south edge (z ≈ −172.2) puts the player capsule (spine feet+0.55..+1.86, r 0.55
   [M Toucher::player, triggers.rs:186]) inside the box's north face, and the E-use sphere
   (head + look×1.0, r 1.0 [M Player_Activate 0x1a5170:100-137; the port's AABB gate,
   drone.rs:15121-15135]) reaches #558 from the same spot. The existing leg-5 chain (ch 2 +
   E → ch 3 → AND #560 → ch 8) then runs unchanged.
2. **The route from the display ground, WALKED for real** (`probe0c_plates_route`, the sim,
   real input throughout — committed on the branch, `#[ignore]`d as a route record):
   - the display ground west lane → the south edge at x 10.5 (z ≈ −116.5 [D probe]);
   - the crouch walk-off (the crossings session's proven entry) → **lands grounded on the
     2.18 lane at (10.57, 3.25, −121.34)** ✓ walked;
   - the 2.18 floor is ONE room (room_storage01 #143): the deep lane (x 8.5..12.2) continues
     west through the storage room (x −1.72..5.62, z −127.5..−150.3) [D coll-scan bands
     z −127.49..−150.28 all stitched] → **walked to (1.15, 3.35, −146.6)** ✓;
   - the 2.37 pocket south of it (z −150.28..−155.77, x −1.8..1.99, via the 0.19 lip)
     [D] → **walked to (1.19, 3.45, −155.17)** ✓.
3. **The one unmapped link: the pocket → yard descent.** The slab's south face stands at
   z −155.77..−155.82 from y 2.37 to 6.15 and spans **x −4.5..15.25** [D coll-scan — this
   window's scans extend the vent-pulley report's "x 2.5..15.25" west to −4.5, closing the
   x 0.45..2.5 slot hypothesis]; the pocket's west edge (x −1.8) hangs over void (the yard
   floor starts at z −158.5..−159.38); the z −158.5 wall (x −5.65..0.45, top sloping
   4.55→6.69) closes the south-west. The walk_to off the pocket edge wedges at the face
   (segment_to_target hit at 0.168 ≈ z −155.78).
4. **The yard side is fully mapped [D coll-scan]** (not yet walked): the yard floor y ≈ 0
   (x −8.25..7.57, z −166..−185, incl. the north-west arm x −8.25..−11.83 at z −166..−174)
   → the bridge (x 7.57..16.39, y 0.00, z −169..−184) → the corridor east
   (x 15.3..51.71, y 0.00→0.38 at x 40.5→0.63 at x 49.8→0.69, z ~−174) → **the east arm
   continues into the warehouse (x 50.63..57.61, z −176.08..−178.27) — the corridor passes
   SOUTH of the secret door wall** → the warehouse floor (0.86) → north into the pit
   (0.76..1.02) → the plates' box graze at z ≈ −172.2.
5. **The nav web [D]:** no AI link crosses between the south-strip patrol (G2, y 6.7,
   x −2.4..13.7, z −133..−154) and the yard patrols (y 0.4) — the game's guards never walk
   the connection, so it is a player-only traversal. Ladder #495 tops at mesh y 2.0 (the
   shelf only; "the top has no landing" [M the ladder495 test in walkreal_0700000c_live]);
   ladder #164 serves the yard→exit direction. The zipline #512 (46.79, 17.85, −195.26) and
   the hand-over-hand wires (#282/#283/#284) all anchor in the south silo/turbine region —
   none crosses the display-ground/yard gap.

## What's left

1. **Map the pocket→yard descent** (the one link). Candidates, in order: (a) a wider
   coll-scan west of x −4.5 and south of z −155.7 (the pocket's west edge may land on a
   floor the r-2.5 scans missed); (b) the z −158.5 wall's sloping top as a walk (from the
   display ground at x ≈ −1..−1.5 the top is ~6.0-6.2, just under the 6.27 floor, descending
   west to 4.55 at x −5.65) with the drop at its west end; (c) the yard's north-west arm
   (x −8.25..−11.83 at z −166) — what is above it at z −155..−166? If the disc data truly
   shows no route, pin that with sister asserts and list the live check: "in PCSX2, walk Bond
   off the display ground's south-west (the walkway04 strip at 6.27, x −1.85..5.71,
   z −153.6..−155.7) and record the descent" — a fresh-boot measurement of where the game
   lets the body go.
2. **Then rewrite leg 4** in walkreal_0700000c_live with the real legs (the probe's waypoint
   shape transfers directly: the walk-off is raw forward ticks while crouched — the walker
   will not cross a floor edge by itself) and run the touched binary.
3. **The full suite** on the final tree (the fast runner; quote the total line). Nothing on
   the branch touches shared code this window (one new test file + docs), so the risk is the
   new probe file only.

## How it was verified

- Every coordinate above is [D] (`nf-extract coll-scan`/`triggers`/`placements-all` over the
  onfoot ISO, 0700000c.bin) or [M] (cited addresses); the walked parts are [D probe] — the
  sim probe's own eprintln log (real input, no teleports after the leg-3 end state).
- `probe0c_plates_route`: **1 passed (the nav dump), 1 ignored (the route record)** —
  `cargo test -p nf-view --test probe0c_plates_route` on the branch tree.
- The touched binary walkreal_0700000c_live was not modified this window (the stand-in stays
  byte-identical to master), so its green status is inherited from the merged
  level-0c-vent-pulley-class report (2 passed / 0 failed).

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout).
