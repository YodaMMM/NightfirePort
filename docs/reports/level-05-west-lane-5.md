# level-05-west-lane-5 — 07000005: the slab crossing walked, the ungrounding bisect clean, the band gap parks the notch legs

Branch `s/level-05-west-lane-5` (from origin/master 77d245668; the creepwall +0x8c radius model
re-landed from the lane-4 tip 031d162ac — master's batch-1023c had held it back). Deliverables:
the slab crossing walked green through real input, the grounding bisect the driver asked for,
the wallhug prompt test re-anchored for the radius model, and the band gap measured and parked.

## The bisect — the round-4 attribution withdrawn

The driver's ask: run the walk at f56c1829c^ and f56c1829c before believing the one-contact
blame. A self-contained repro (`probe_slab_ground`, scratch example: Bond stood at the slab's
east end via `Bond::stand_at`, then (a) a plain fixed-yaw forward push, (b) walk_to's exact
steering — `turn: err.clamp(−0.07, 0.07)` + forward with the 60-tick stuck bookkeeping and the
sidestep mitigation) ran at f56c1829c^ (d97c0b754), f56c1829c, and the branch tip: **byte-identical
output at all three** [L the repro's per-30-tick prints]. The one-contact rework is NOT the cause.

What the repro actually shows:
- A **transient ground-flag flicker** on the slab top (grounded 0 for a few frames at (24.33,
  52.18) and (21.35, 54.56) while Bond stands at the right height — pos.y ≈ feet + 1.15 root,
  floor_below 33.34) that recovers within a few frames — both modes cross it.
- A **hard block at (23.61, 53.96)**: full-height rays west are BLOCKED at h 0.2-1.6 while the
  north/south rays are clear [D the ray probe] — the void band (z 52-54.25, x ≤ 23.25) has a
  face there; the steering stalls against it (moved 0.00 for 90+ ticks, grounded), a sidestep
  recovers. The earlier rounds' wedge was this stall plus the walker's stuck ladder (the
  rail-jump drops Bond under the slab — the y-24-27 under-slab region the nav routes through).
- The earlier "ungrounded 1.34 above the floor" readings were the flicker seen at the give-up
  dumps, not a persistent hover.

## The slab crossing — walked green (the round-5 answer)

The 0.25-unit map (x −24..26.75, z 40..60) gives the slab's real shape [D, through the port's
collision of the disc's own data]:
- The **strip**: z ~54.3-55.7, floors 33.1-33.8, from x 24 west to **x −1** (the round-4 report's
  "fragmented east tip" was a z-64 row misread — corrected there; the strip is solid).
- The **diagonal band**: from the strip's west end the 3-cells step down south-westward — z 54→53
  at x −1.75..0, z 52→50 at x −2.75..−0.5, z 49.5→47.75 at x −9..−2, ending at x −9.5 (z 46.75).
- z 56 is the y-29-32 lower roof (3+ below); the z 52-54 band west of x 23.25 is VOID (the face
  the walk stalls on).

The walked legs (all through `walk_to` real-input steering): the steps top (24.28, 53.0) → the
strip east (21.4, 54.7) → the strip mid (9.4, 55.0) → the strip nose (1.9, 54.7) → the bend
(−1.4, 52.6) → the diagonal (−4.1, 49.7) → the band's west end (−9.0, 46.9) — all grounded on
33-34.7 (feet 32.5-33.55). UN-IGNORED and green: `walkreal_07000005_live::
the_cable_legs_the_crawl_notch_and_ride` now walks the whole west lane through the crawl #299 and
the slab band to its west end (5/5 walkreal tests pass; 2 ignored).

## The wallhug prompt test — re-anchored for the radius model

`fix_wallhug_prompt_live::the_crawl_prompt_arms_only_inside_the_games_sphere` built its away-spot
from the OLD band model's mouth (+2.2 past it). Under the radius model [M Control_BuildWorldSph
0x133070] the Window wall's mouth (creepwalls[0], placement 13, bound centre (0,0,0) [D]) sits
1.53 off icon[1], so the old +2.2 spot landed at 3.73 — still inside the game's straddle reach
(the port arms ctx 6 there). The [L]-measured silent spot is ~5.2 from the icon (the old
construction landed there only because the old mouth sat ~3.0 out). Fix: build the away spot at
the live distance directly (`icon_c + away_dir·5.2`), same asserts, same fail-first property
(5.2 < the old 6.0 window). Both creepwall mouths still arm ctx 6 [L the spot probe] — 2/2 green.

## What's left (round 6)

- **The band gap** (the parked `the_cable_legs_the_gap_the_notch_and_the_ride_parked`): the 0.25
  map shows the band's west pieces (−19.75..−15.75 at z 46-47.5, including the (−24..−23, 45-47)
  cable-ledge step [D]) separated from the diagonal's west end by a void/2-cell gap (x −16..−9.5,
  z 45.5-48). The walked crossing drops ~3.5 onto a y-29.6 roof whose south-west run descends to
  22.6 and dead-ends [L this round's walk]. The game's link is unfound — a live read (fresh boot,
  gdbwatch the player triple while Bond walks the band's west end westward) names the crossing,
  or the gap hides a thin/one-way surface the down-ray misses.
- Then the notch grab (the round-3 legs are in git history at 96cebefd1 — note the round-3 nav
  arrivals were XZ-only, so those anchors may have "arrived" on the wrong floor; re-verify heights
  when re-attaching), the ride, the south ledge, the window-ledge crawl #35 ((−34.10, 37.28,
  −31.65) [D]) and the exit door #112.

## Live checks for the main agent

1. **Fresh boot into 07000005**, walk Bond down the slab band to its west end (the crawl #299
   exit → the strip at z 55 → the diagonal to (−9, 47)), then push WEST — gdbwatch the player
   position triple (obj+0x30 row). Expected: the game either steps the gap at a specific z or
   drops onto the y-29.6 roof and takes a line this sandbox's walker refused; the crossing is the
   round-6 leg.
2. Render spots (`--bond x,y,z,yaw,pitch`): (9.4, 34.5, 55.0) — the strip mid; (−4.1, 33.9, 49.7)
   — the diagonal; (−9, 33, 47) — the band's west end with the gap in frame; (−17, 30.7, 47.9) —
   the walk-off landing.

## Verification

- `cargo test --offline --release -j2 -p nf-view --test walkreal_07000005_live -- --test-threads=1`
  — **5 passed; 0 failed; 2 ignored** on the final tree.
- `fix_wallhug_prompt_live` 2/2, `fix_wallhug_camera_live` 1/1, `fix_swim_vent_live` 2/2,
  `fix_swim_vent5_live` 2/2 (the creepwall radius model's consumers).
- The full suite cannot run in this sandbox (background jobs die between tool calls — both
  starts again this round; the driver's own runs should cover it). nf-game is byte-identical to
  the lane-4-validated tree (creepwall.rs re-landed unchanged; 86/86 binaries ran green there);
  nf-formats/nf-audio untouched.

## Usage

Task-system token/context figures are not visible to me this session. Session wall-clock:
01:06-02:30 UTC; two suite starts (both died silently — background jobs are killed between tool
calls in this sandbox), one branch setup, the bisect (3 example builds/runs), ~6 walkreal
iterations, one report.
