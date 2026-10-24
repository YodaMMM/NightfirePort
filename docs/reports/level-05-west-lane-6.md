# level-05-west-lane-6 — 07000005: the band gap solved and walked green; the step shelf's kink parks the notch legs on a live read

Branch `s/level-05-west-lane-6` (from origin/master 8df0d8a4). The sandbox was reset at round
start — fresh clone, fresh work-slim + ISO downloads, fresh toolchain; the two-hour window held.

## The band gap — the game's link is found, and it is walked

The driver's ask: find the game's link across the "void/2-cell gap (x −16..−9.5, z 45.5-48)"
that the lane-5 walk could not cross. **The gap does not exist where lane-5 mapped it.** The
collision column dump (`crates/nf-view/examples/probe_gap_05.rs`, built over
`level.instances(true)` — the exact instance list scene.rs b9 feeds `CollisionWorld::build_scoped`,
winding-agnostic Möller–Trumbore down-rays on the transformed triangles) shows:

- **The band's 33-floor continues south-west past x −9.5** [D the dump]: z 45.0: x −14..−8 at
  33.0; z 45.5: x −12.5..−8 at 33.0. Lane-5's map stopped at the diagonal's west end and read
  the region west of it as void.
- **A narrow corridor links it to the west pieces** [D]: z 44.5-45.0, x −15..−14.5, floors
  33.3-33.8 — 1.0-1.5 units wide, the void at z 44.0 south and the y-29.3-29.5 roof at z 45.5-46.0
  north. From its west end the pieces' south strip (x −16.5..−15.5, z 45.0-46.0, 33.5-33.8) runs
  north to the west pieces (x −18..−16.5, z 46.5-47.5, 33.2-34.0).
- **A 0.5-unit void seam at x −20.0 (z 44..48)** separates the pieces from the cable ledge
  (x −23..−20.5, z 46-47, 33.2-33.8) — narrower than the capsule's 1.1 diameter; the walk crosses
  it grounded on the edge contacts [M Player_FeetOnPoint 0x194188 grounds the pick bit; the walk
  exercises it]. Lane-5's "west pieces (−19.75..−15.75)" start exactly at this seam.
- The real voids sit **north** of the corridor: the deep y-17-21 slot x −13..−9.5 at z 47-48 and
  the y-29 roof x −16..−11.5 at z 46-47 — the lane-5 "walked drop dead-ends at 22.6" was this
  slot's floor, reached by falling north-west off the diagonal's end.
- No jump is needed anywhere: the game's jump arc [M Player_HandleJump 0x1916b0 — the launch
  rides the −0.4 scale (the port's `vel[1] = −GRAVITY·0.4` = +3.92 u/s), gravity −9.8 u/s², run
  0.105 u/tick → ~5.0 u level reach, ~7.1 u landing 2.5-3 lower] is irrelevant to this link.

**Walked green through real input** (no teleports after the spawn; `walk_to`/`walk_nav_to`
steering only): the whole route from the cold start #26 — the courtyard, the lane, the tower
top, the south slope, the crawl shelf, the wall-hug crawl #299 (the real Use press), the wide
ledge, the slab strip, the band's diagonal, the band's south-west continuation, the corridor,
the pieces' south strip, the west pieces, the x −20 seam, the cable ledge and the step's spine
start. `walkreal_07000005_live::the_cable_legs_the_gap_and_the_ledge_step` — **un-ignored and
green**: walkreal_07000005_live **6 passed; 0 failed; 2 ignored** on the final tree. The lane-5
route docs on `walk_to_the_slab_east_end` are corrected in place (the "THE STOP" comment now
points at the found continuation).

## The step shelf's kink — probed to the tri, parked on the live read

South of the seam the route narrows to a 0.5-1.0-unit diagonal shelf [D the dump]: z 46.5:
x −23.5..−22; z 46.0: x −24..−23.5; z 45.5: x −24.5..−24; z 45.0: x −25..−24.5; z 44.5:
x −25.5..−25; z 44.0: x −26..−25.5. The walk follows its spine to z ~46.5, then stalls at the
**kink**: `Room_Outerwall_Corner` (inst 13) carries a 45° ramp there (33.0→34.0, its base line
(−23.39, 47.09)→(−26.03, 44.44), normal (−0.52, 0.68, 0.52) [D the dump]) that crosses the spine
exactly at the stall. Four walk iterations + three hop variants (the jump press + the forward
hold with the aim set, the same shape as the round-3 grab; hold 6/7/11 ticks) all land on the
y-29 roof west of the ramp base — the landable window between the ramp base and the ledge spine
is ~0.3 u wide in the flight path and the ramp contact deflects the flight [L the runs]. The
kink is the ONLY blocker between the walked route and the notch.

**Live checks for the main agent** (the driver pre-authorized stopping here):
1. **Fresh boot into 07000005**, walk the west lane to the step spine (the walked route above),
   stand at (−23.5, 33, 46.5) and push south-west into the kink — gdbwatch the player triple
   (obj+0x30 row). Expected: the real game either climbs/slides the 45° ramp (does the pick
   ground on it?), jumps it from the spine, or drops to the y-29 roof and re-ascends somewhere
   this sandbox's probes have not found. The answer names the kink leg; the rest of the route is
   already written and height-verified.
2. Render spots (`--bond x,y,z,yaw,pitch`): (−13.4, 34.1, 45.1) — inside the corridor, the drops
   both sides in frame; (−20.1, 34.7, 47.5) — mid-seam crossing; (−23.5, 34.6, 46.5) — the spine
   at the kink with the 45° ramp ahead; (−25.4, 33.4, 44.4) — the ledge bend past the kink.

## The notch legs — written, height-verified, parked

The grab/ride legs are in `the_notch_grab_and_ride_parked` (ignored, with the exact blocker in
the attribute): the round-3 shape (git 96cebefd1) with the heights re-verified against the dump
[D] — the ledge run z 36→14 at x −25..−23 (33.2-35.4; the coarse census "x −28..−23, z 19..40,
floors 30-34" at the dump's resolution — the round-3 anchors (−24, 33, 40)/(−24, 33, 28) are on
void at the finer resolution, replaced by the bend route through (−26.5, 40.4)), the notch's
north shelf 30.8-31.0 at x −25.5..−24.5, the grab at the notch's north edge (−23.8, 14.5) [D
icon #310 (−23.80, 32.74, 12.33)], the ride [M Player_Wire 0x197658]. The notch sister stays
ignored behind the same kink (from the walked spine start its hold would pass vacuously — not
proven behaviour). The window-ledge crawl #35 (−34.10, 37.28, −31.65) and the exit door #112
remain beyond the ride.

## What ran

- `cargo test --offline --release -p nf-view --test walkreal_07000005_live -- --test-threads=1`
  — **6 passed; 0 failed; 2 ignored** on the final tree (the un-ignored gap legs included).
- `--test fix_wallhug_prompt_live` 1/1, `--test fix_wallhug_camera_live` 2/2,
  `--test interact_07000005_live` 11/11 (the 05 neighbours).
- The full four-crate suite cannot run in this sandbox (background jobs die between tool calls —
  the lane-5 finding, reproduced: the suite start died silently twice). The driver's own runs
  should cover it; nf-game/nf-formats/nf-audio are untouched by this round (tests + example +
  docs only).

## Live checks for the main agent

See the kink item above (the one live read that unblocks the notch legs).

## What's left (round 7)

1. The kink leg (the live read above, or a port-side walk that rounds the ramp from its east).
2. Un-ignore `the_notch_grab_and_ride_parked` and the notch sister; the grab/ride legs run as
   written.
3. The south ledge after the ride, the window-ledge crawl #35 (−34.10, 37.28, −31.65) [D] and
   the exit door #112.

## Usage

Task-system token/context figures are not visible to me this session. Session wall-clock:
04:22-05:06 UTC; one fresh clone + two downloads (work-slim 34.9 MB, ISO 369823203 B exact), one
toolchain install (rust stable 1.99.1, py7zr, user-prefix ALSA), one branch setup, the gap column
dump + the sim.world probe (the evidence pair that found the link), 8 walk iterations (the gap
route green on the 4th; the kink's 4 walk + 3 hop variants), three committed pushes
(e97fefb2 claim, 8494ceca gap green, 91428058 kink), one report.
