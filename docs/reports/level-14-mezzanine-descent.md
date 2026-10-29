# level-14-mezzanine-descent — the mezzanine descent, mapped to two leads; the un-ignore is left for the next window

Branch `s/level-14-mezzanine-descent`, base origin/master (48696227). Window 23:28–01:08 UTC
(continuation; the branch had zero commits when this window started). Task: find the route a
player takes from the vent cut-scene's hand-back spot on the mezzanine (≈ (40.7, 2.8, 39.0))
down into the facility, fix any port collision gap, walk the descent on real input, and
un-ignore `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit`.

## What's done

**The mezzanine is mapped, on real input and on [D] data.** Every probe below is an
assertion-free scratch test (`scratch_l14_*.rs`, untracked) driving Bond through
`Sim::step_bond`/the Walker with real input only; teleport was used to SKIP the ~10k-tick
prologue inside probes, never to cross the terrain being measured.

1. **The hand-back pad** (floor 2.15, x 32..40+ at z 38.5..39.5) walks WEST to
   (34.63, 2.80, 37.94) and SOUTH to (34.57, 2.80, 37.18), then down a 0.43 step onto a
   **landing at floor ≈ 1.72** — (33.09, 2.37, 35.38) grounded. The pad's south-west
   protrusion (x 33.85..35.2, z 36.0..37.2 [D rays]) is walkable; the mezzanine's west face
   wall at x 31.5 (z 38.4..40.8, the ignore reason's first wedge) is confirmed NOT the route.
2. **The landing's door is plc #834 "OBJECT_Door" (32.01, 2.14, 35.81) [D]** — the live-check-3
   "unlocked door at (32.0, 2.1, 35.8)". Sim door #53: kind 2, group 0, ch_open 0, ch_lock 0 —
   unlocked [D params through `Doors::create`]. Leaf tri box x 32.00..32.65, y 1.76..2.52,
   z 35.72..35.82 [D], over the 1.72 landing, beside the tall fusebox column #833
   (x 31.93..32.65, y 0.60..4.60 [D]) and the "Fusebox_Sign(Cameras)" #835. The walker's
   E-press logs "opening door" but the through-walk wedged; the doorway's usable opening
   (and whether this door is even on the route) still needs a dedicated probe.
3. **The notch west of the landing (x 31.2..33.75, z 36.0..38.3) is a bottomless void in the
   port** (flat `world.segment` rays NaN past −9) — **and the game's own nav web never crosses
   it either**: path 0's corridor nodes 57/58/263/257/260 (x 27.3..28.3) have no link east to
   the machine room's nodes 47/48/49 (38.7..41.8, z 31.4..36.1) [D nav links]. The AI's
   corridor and machine-room webs are separate. So the notch is most likely TRUE void (a
   shaft) in the game too, not a port collision gap — the descent should not cross it.
4. **The east lane is the live descent lead.** The pad walks EAST to (44.05, 3.01, 39.13) and
   SOUTH along the lane to (45.13, 3.05, 43.58) — lane floor ≈ 2.35..2.40 continues to z 43.6.
   [D rays]: lane floor 2.38..2.40 at x 40.9..41.5 z 42.8 and x 39.5..41.0 z 44.0..44.5, with
   **holes down to the machine room floor −0.60** at (42.5, 42.5), (44.5, 43.5), (43.0, 44.5).
   The grapple point #331 (40.78, 4.79, 42.86) + "3RD_Person_Icon" #330 (40.79, 0.41, 42.89)
   [D placements] mark the machine-room spot under the lane — the game's own climb-back-up
   marker, which is also where a player would drop down. The walker did NOT find the drop by
   steering (stuck at x 45.13, three 6000-tick caps) — the exact walk-off edge/hole is the
   next session's first probe.
5. **Below, the machine room (floor −0.60..0.0) walks WEST only to x ≈ 37.1**
   ((37.15, 0.05, 42.48) grounded) — matching patrol path 1 node 0 (37.44, 0.0, 42.81), the
   patrol's west turn [D nav]. The machine-room → west-wing connection is still unfound.

## How it was verified

- [D] placements/tris: `Level::parse` over the disc archive + `object_matrix` transforms
  (scratch_l14_mezz/mezz2/mezz3/nav); [D] nav: `sim.nav.paths` links dumped
  (scratch_l14_nav); the door's sim record via `Doors::create`'s params
  (scratch_l14_mezz2 section D).
- Real-input walks: `scratch_l14_walk{,2,3}.rs` through the shared Walker
  (`common/walker_b2.rs`) — no pos writes inside walked legs (teleport only to set the probe's
  start, outside the #301 touch volume).
- Nothing in `crates/` changed this window; the tree equals origin/master's merged state
  (48696227), whose `walkreal_07000014_live` binary was green (5 passed / 0 failed / 1
  ignored) in the level-14-skip-handback report.

## What's left (the next window's plan)

1. Locate the lane's walk-off edge/hole over the machine room (dense ray grid x 41..45,
   z 42..45 + a walk probe that *steps* off instead of steering to a far target), then walk
   pad → lane south → drop → machine room on real input.
2. Find the machine-room → west-wing connection: ray the floor UNDER the pad (z 37..42,
   x 31..40 — never scanned from below) and the lane's south end (z 44.5+); check the corridor
   A5_1's east wall (x ≈ 31, z 36.6..42.8) for an opening.
3. Decide the door #834's role (route or set dressing) with a doorway-specific probe.
4. Re-route `walk_the_infiltration_to_the_delta_exit`'s descent leg through the proven route,
   un-ignore, and run the FULL suite (fast runner) — quote the total line.

## Live checks for the main agent

1. **The descent in the real game**: fresh boot 07000014, let the vent scene end at the
   mezzanine (≈ (40.7, 2.8, 39.0)), then play to the facility floor. Record the path: does the
   player walk the pad EAST/SOUTH and drop off the lane over the machine room (the
   #330/#331 spot), or use the 1.72 landing's door (32.0, 2.1, 35.8)? A gdbwatch on
   `glb_players+0x30` (write) sampling the descent gives the exact route points.
2. **The door #834 in the real game**: stand on the 1.72 landing, press E at
   (32.3, ~2.1, 35.8). Does the leaf move (it is kind 2, spline count 2, path block 0x48
   [D extra[7]/extra[8]])? If it opens, what is on the other side — floor or the shaft?

## Usage

not visible to me — this environment shows no token, credit or context-window figures (no
status line or /usage output available); nothing to copy, and per the task's rule I did not
estimate.
