# drone-bump-collision-8 — the east door's guard wall taken down from the live measurements; two tests green, two legs into new ground

Branch `s/drone-bump-collision-8` = `origin/s/drone-bump-collision-7` merged with
`origin/master` (clean). The task: port the game's measured Bond-vs-door-guard push and turn
five tests green.

## The fix (one gate, cited)

**The at-post `CIVILIAN_DOOR_GUARD` (0x31) state no longer joins the player's push set**
(`crates/nf-view/src/scene.rs`, the `bot_capsules` producer). Cited to the -7 live answers
[L audits/live-07-eastdoor.txt, fresh boot 07000007, 2026-10-28]: Bond placed ON the E-spot
(94.6, 44.4, −47.4) — 0.48 from d10's shifted centre — STANDS there 120 frames; the guards do
not relocate; placed between the pair the game walks him out NORTH. The port's double 0.95
contour (Bond r 0.55 [M Player_Collision 0x194df0:316] + bot r 0.4 [M NDrone2_Collision
0x140e90:91-94]) is what pinned the four reds; the live contract says the at-post pair's
capsules are inert to the player.

The static chain was reconciled term by term first, and it does NOT explain the measurement —
every gate checks out on paper:

- `Collide_Jointy` 0x1ec180:229-281: the kind-0x800 resolve, the spine closest-point rounds,
  the r-sum from the query's HITTEST+0x8C plus the other's +0xD4-block+0x8C (for a drone
  obj+0xD4 → instance+0x3D0, so the radius slot is Drone+0x45C), the SP arm fVar12 = 1.0
  against a class-2 other, and the guard's own query returning 0 against a class-3 opponent
  in SP (:229-233) — bots never push themselves off Bond.
- `NDrone2_DoCollision` 0x149ec8 and `NDrone2_Collision` 0x140e90:59-124: the marker +0x464
  = 0x800 whenever the control pass runs; radii 0.4 (0.55 type 0xC), 0.3 with a degenerate
  point capsule on the +0x2F branch; the A/B arms pos+up·r and pos+up·(r − +0xA0).
- `Collide_Intersect` 0x1ebaa8:~95-104 (the Jointy precondition the port had not modelled):
  the query kind != 0x100, the hit obj's flags2 (+0xFA) bit 8 set — set for every anim object
  by `AnimObjectNew` 0x1be910:58 — and the hit obj's glist+8 == 0 (the breakable-list slot,
  per `Break_Kill` 0x12b970's read; the guards are not breakable, so it passes).
- `Intersect_Portal` 0x1ea7b8's 0x800 branch: the straddle crossing is the projection of the
  portal centre onto the spine, `|Dist2Tri| < HITTEST+0x8C × 1.5` — the port's `r × 1.5` is
  the game's own constant, not a fudge.

The port's capsule math, arming, straddle and portal test are all decompile-faithful — so the
arm that zeroes the at-post pair's player-side push is not visible statically; it is the live
check below. The behaviour gate is [L]-backed and the mechanism question is open, which is why
the filter carries the full citation chain in its comment.

## What it bought (measured on the rebuilt tree, foreground runs, NF_ISO set)

- `cutscene_timer_pause_live`: **1 passed / 0 failed — GREEN** (8.1 s).
- `walkreal_07000007_live`: **2 passed / 0 failed — GREEN** (11.5 s; was 1/1).
- `party_door_lock_07000007_live`: **1 passed / 1 failed** (was 0/2). The survivor's failure
  moved PAST the old wedge: it is now the sister's `#244 still answers the use key` assert
  (test line 272) — Bond reaches the door area, the leaf never moves across the E presses.
  The door dump at the wedge showed the use-triggers `trig kind=1 out=0x16 mask=0x8 at
  (93.7, 45.4, −45.7)` and `trig kind=1 out=0x48 mask=0x8 at (94.3, 43.7, −45.9)`: the 0x48
  use zone sits ~1.5 NORTH of the door face, while the tests aim the Use at the E-spot
  (94.6, −47.4) — the game's E-interaction point for this door needs re-derivation from the
  disc's own play (the -7 report's open question, now pointed at the trigger zone).
- `probe_07_route`: still red, but the route now RUNS THROUGH the guards — "the hall east ok",
  then the E leg lands (the old wall is gone; Bond reached (94.86, −48.03), overlapping d10's
  boot zone exactly as the live contract predicts). The failure is the NEXT leg ("the hall's
  north-west corner", 6000-tick cap): Bond STUCK at (93.40, 44.26, −46.32), d=1.43,
  **grounded=false, floor 0.23 below his feet** — a hover wedge in the hall's north-west, not
  a guard push (the crowd_aware steer exemption for the 0x31 guards landed too
  (`common/walker.rs`) and did not change this leg — the hover is a different mechanism).

## What's left (next window)

1. The route's post-E hover wedge: Bond at (93.40, −46.32) hangs 0.23 over the floor with
   grounded=false through a 6000-tick leg. First suspects: the scoped floor ray's portal
   crossing at the hall's north-west (the same family as capsule-11's stairwell finding), or
   the open door leaf's dynamic tris feeding the floor band.
2. The party sister's #244 use: derive the door's real use zone from the 0x48 trigger's
   bounds, re-derive the leg's stand point, and re-run.
3. The full suite (see Suite).

## Live checks for the main agent

1. **Name the arm that kills the at-post push**: fresh boot 07000007, Bond at the E-spot, read
   d10's Drone block: +0x464 (kind — expect 0x800 per [M 0x140e90:95-101]... the measurement
   says the push never fires, so either it reads 0 — name the writer — or it reads 0x800 and
   the kill is in the pick), +0x45C (radius), +0x3F0/+0x400 (A/B), +0xA0 (anim height),
   +0x4F8 (bits 0x20/0x200), +0x570 & 2, +0x538, obj+0xFA & 8, and obj+0xD8's target's +8.
   Also gdbwatch the player's pick over 30 ticks at the E-spot: does a hit node for d10 ever
   reach `Collide_Jointy`? One of these names the arm; the port's filter then swaps the
   [L]-behaviour gate for the [M] mechanism.
2. **The east door's use zone**: walk Bond to the 0x48 trigger at (94.3, 43.7, −45.9) and
   press E — does the door answer there (ch 0x48 fires)? And at the E-spot (94.6, −47.4)?
   The trigger's bounds vs the door's own use radius decide where the tests' E legs stand.

## Suite

The background fast runner dies between tool calls in this sandbox (setsid included — two
launches this window died silently during the build/run phases), so the runs were foreground
binaries. What ran on the final tree (the guard-filter fix + the steer exemption, both in):
cutscene_timer_pause_live `1 passed; 0 failed`, walkreal_07000007_live `2 passed; 0 failed`,
party_door_lock_07000007_live `1 passed; 1 failed`, probe_07_route `0 passed; 1 failed`.
The rebuild of the four-crate test set and the full sweep did not fit the window (the two
10-minute build calls and the nf-view relink consumed it); the next message resumes from the
branch (5436bded + this report) with the runner.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for
this session (no usage banner, no /cost, no token readout anywhere in the interface this
worker sees).
