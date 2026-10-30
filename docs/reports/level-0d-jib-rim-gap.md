# level-0d-jib-rim-gap — report

Branch `s/level-0d-jib-rim-gap` (from origin/master 76081986). Task: find how the game gets Bond
from the rotate crane's deck to the ST-3 yard, fix any port gap, and replace the remaining
teleport in `walkreal_0700000d_live::walk_arrival_to_the_c5_door` with real input.

## What's done

**The deck→yard route is now mapped end-to-end with committed, re-runnable probes
(`probe_crane17/18/19`, print-only, `--ignored --nocapture`), and two of the three unknowns are
settled with evidence. The teleport replacement itself is NOT finished — one leg resists and is
documented precisely below.**

### The findings

1. **The swing is the cut-scene's beat, not the player's ride.** The walkway surface the route
   needs (12.13, a ~1.5 m strip spanning r 6.5-8 along the jib's rest direction) is a STATIC map
   object: probe19's ray-downs at the swing's end pose find NOTHING at r 6.5-8 (off −0.6..+0.2,
   y 13.5→10) after SP #492 finishes, while the rest-pose spot keeps reading 12.13 through the
   whole swing (probe18). The deck's collision DOES rotate with the glist (probe18: the rest
   spot 13.04 empties at mid-swing, the rotated spot gains it; probe17's hold prints show the
   rider carried r 3.2 through the full 61°→−11° swing [D 0x06000a08 keys]) — so script-entity
   collision poses per tick, but the walkway belongs to the static world, not the crane.
2. **The game's jump cannot bridge the rim gap — the port is faithful.** `Player_HandleJump`
   0x1916b0 zeroes the velocity (`Vec_Zero(obj+0x50)`) then adds a fixed impulse derived from
   `WldGravity` — the walk-state caller `Player_SSWalk` 0x192310:69 passes the constant
   0x2d8890, which the ELF symtab names `WldGravity` [D work/action-symtab.tsv]. No horizontal
   momentum survives. The port's `vel = [0, −GRAVITY·0.4, 0]` (player.rs, the HandleJump arm)
   matches. probe17's stall-jumps confirm it live: five jumps at the deck's rim (r 4.02) all
   land back on the same spot, y 14.12→14.79→14.12, zero horizontal travel.
3. **The deck's rim geometry** (probe16/17): the deck is walkable to r ≈ 4.0 at the cab's
   bearing and to a corner at r ≈ 5.1; the walkway strip starts at r 6.5 — a 1.4-2.5 m radial
   gap. A mid-swing walk-off reaches r 7.4 mid-fall but lands ~0.13 m outside the strip's
   positive edge (the swing's tangential carry drifts the fall +0.25 lateral — probe17 v3's
   traced fall: (197.90, 13.88, −266.80) → (197.89, 11.91, −269.29) → the trench y 1.06).
4. **The dock side is mapped** (probe17's dock map + the placements): the dock apron 9.42-9.53
   at (195,−280)/(200,−275); the dock building's roof 18.58-19.9 over its south half; the north
   steps (12.67→9.56) at (188.6,−268.9) [probe16's r13.5 ray]; the control corner — E-plate
   #460 (189.02, 9.72, −288.92), SS #112 `shipping_handle` (186.25, 10.43, −288.33), the
   buttons #113/#114 — all at dock level [D placements]; the building's interior floor 1.06
   (probe17's fall landing (199.0, 1.06, −283.1), grounded, walkable).
5. **The walk-off got closer than any earlier attempt and is the next window's first move**: the
   fall from the deck's edge crosses the walkway's plane INSIDE its r band but ~0.13 m outside
   its 1.5 m width, because the walk-off point (the deck's corner) sits laterally offset from
   the strip. The candidate fixes, in order: (a) walk off at a spot whose lateral offset is
   inside the strip (the deck's edge at other local bearings — the edge radius varies 4.0-5.1),
   (b) bias the walk-off aim harder to the strip's negative side (probe17 v4 tried −1.0 at rest;
   the rest-pose edge at r 4.02 stalls the run before any walk-off, so the attempt must be
   MID-SWING like v3), (c) land on the strip's roof-side edge (18.58) instead — probe16 shows
   the roof slab begins at off +0.75 at r 8.

### What is NOT done

- The teleport at `walk_arrival_to_the_c5_door` (after the swing beat) stands. The chain after
  the crossing is probe-mapped but not test-driven: the dock E #460 (proven separately by
  `interactables_0700000d_live::crane_e_plate_arms_the_handle_ss_and_the_shipping_sp`, ch 0x2b),
  the shipping beam's shuttle (SP #115, 3000 ticks [D]), the beam boarding (the catwalk 18.58
  step-off candidate [S unverified]) and the west ride into the yard.
- No port gap was fixed in nf-game: the two suspects examined both turned out faithful (the
  jump's vertical-only impulse; the per-tick script-entity collision posing, which the deck
  proves works).

## How it was verified

- `walkreal_0700000d_live` — **1 passed / 0 failed** (26.6 s) — unchanged route, the header and
  the blocker comment updated with this window's findings.
- `interactables_0700000d_live` — **11 passed / 0 failed** (31.9 s) — untouched, confirms the
  crane-side mechanics the route relies on.
- The probes 17/18/19 are committed and re-runnable
  (`cargo test -p nf-view --test probe_craneN -- --ignored --nocapture`).
- The full suite (`tools/test/suite.py`) was started detached in the window (22:57 UTC and
  again later); this sandbox's session rotations killed the runner before it printed a total
  (empty `suite.out`). No nf-game/nf-formats/nf-audio source changed on this branch — the diff
  is tests + docs only — so the suite risk is the two touched test binaries quoted above. The
  final tree needs a clean suite run before merge.

## What's left (the next window's list)

1. The walk-off tuning (the three candidates above) — the crossing's last ~0.15 m.
2. The dock walk + E #460 in-route (the corners are mapped; the interactables test holds the E
   evidence).
3. The beam boarding: reach the roof via the north steps (188.6,−268.9), walk the roof/catwalk
   (18.58, x 147..193 over the beam's z −287.11 line), step off onto the beam's westward pass
   (SP #115's shuttle 137.54 → 177.22 → 137.54 over 3000 ticks [D]), ride west, drop into the
   yard at x 137.54, then the existing legs 2-5 run unchanged.
4. A clean full-suite run on the final tree.

## Live checks for the main agent

1. **The walkway's collision at the swung poses** — fresh boot 0700000d, E the rotate plate
   #450 on the crane deck, let the 72° swing finish, then walk Bond off the deck's edge toward
   the jib. Address of interest: the EE collide pass's handling of the crane glist 02000a9c —
   expected: either the walkway collides at every pose (the port's static-strip reading is
   wrong and the walk-off should aim at the ROTATED strip) or the game crosses on the static
   strip at rest (then the port matches and only the walk-off tuning is left). Access type:
   ray/capsule contact vs the walkway during/after the swing.
2. **The walk-off's landing window** — fresh boot, on the deck mid-swing (rot ≈ 0.3-0.5), walk
   Bond off the deck's south-east edge with forward held. Expected: the fall carries him onto
   the walkway strip (12.13) — watch whether the real game's walk speed at the edge differs
   from the port's ~0.04 m/tick, which decides the landing point inside the 1.5 m strip.
3. **The jump's horizontal** — the decompile reading (Vec_Zero + a WldGravity-derived impulse,
   0x1916b0 + 0x192310:69 @0x2d8890) says vertical-only; a fresh-boot jump off the deck's edge
   (no wire context) should show zero horizontal carry, confirming the port.

## Usage

not visible to me — this sandbox exposes no token, cost or context-window figures for the
session, so there are no numbers to copy.
