# level-14-tripbomb-real-route-2 — the report

Branch `s/level-14-tripbomb-real-route-2` (cut from `origin/s/level-14-tripbomb-real-route`,
origin/master merged clean). The previous session's blocker — "the last ~4 u into the rack lane" —
is solved, and the whole task's chain is green.

## What's done

**The whole route runs on real input, every tick, from the level load** — `walk_the_tripbomb_route_pick_up_place_and_shoot_it`
in `crates/nf-view/tests/walkreal_07000014_live.rs`, passing (`walkreal_07000014_live`: 7 passed,
89.8 s):

1. **The lane entry is the game's own nav path.** From the catwalk head the nav web routes the AI
   over the crate row at y 1.20 — nodes (−37.33, 1.20, 33.61) → (−36.74, 1.20, 33.61) → (−36.12,
   1.20, 33.62) → (−35.02, 1.20, 33.61), then the lane node (−33.73, 1.20, 35.81) at the rack front
   [L the nav probe, committed as part of `probe_lane_south.rs`]. The previous session's floor-level
   legs stalled on the 1.45 platform at x ≈ −36.2 (the row is the deck; the under-shelf floor
   passage is not the walker's route).
2. **The armoury door.** The under-shelf passage is closed by a door at (−35.6, 0.8, 33.6) locked
   on ch 46 (0x2E) [L the crawl fail dump: kind 2, state 0x2, ch_lock 46]. Its writers are the
   shelf-side keypad pair — kind-0x2e Locks with outs 0x2C at (−35.84, 2.30, 32.17) and 0x2D at
   (−35.84, 2.30, 35.09) [D the 2026-11-02 census] — burned with the Q-Claw laser Bond already
   carries from the SkyBridge crossing [M `Lock_Update` 0x1ae3d8 case 0: the 0x4E −3/tick burn
   writes `switch_channels[param 5]` on the open, :57-66 + :194-195]. The test draws the laser with
   the real switch request, burns both keypads (each burn's channel rises), ch 0x2E lifts, and the
   walker E-opens the door; the slide is waited out to pos01 0.9.
3. **The crouch crawl.** The shelf slab (tops 3.5, underside ~2.2 — the lane-entry probe's open
   band y 0.9..2.2 [L]) rides over the crate row; standing head (~3.2) hits it. The game's crouch
   is a toggle: one press crouches, the crawl runs, one press stands [M `Player_SSCrouch`
   0x1924c0:84-105]. `try_walk_to` has no crouch acts, so the crawl is the raw input path — the
   same on the way back out.
4. **The take** through `Pickup_Update` 0x18e148's real touch (+15 ticks walking into the rack;
   the pickup tombstoned, `Taken::Weapon { weapon: 0x3a }` [M `Pickup_Handler` 0x18e650]).
5. **The place**: the real switch request draws the tripbomb, the real fire throws it at the wall
   face 0.8 u below the eye-ray's hit — it sticks at (−32.097, 1.746, 36.300) normal (−1, 0, 0)
   [M `Bullet_CollisionHandler` 0x12ce80's world-hit arm; the 120-tick arm 0x12ce80:330]. The low
   throw is deliberate: a bomb at eye height (~2.5) sits above the under-slab band and shadows
   every shot line; at 1.75 the band stays shootable.
6. **The retreat**: south off the beam line first, the crawl back out, down the catwalk to the
   0.40 lane — **12.03-12.05 u from the bomb (the ≥12 law), beam distance 9.8** — asserted.
7. **The shot**: the level's own clear line is crouched inside the lane under the slab, through
   the open doorway: seg eye→body `None`, and **round 2 of the P2K detonates it** through
   `Bullet_handle_object_destruction` 0x12cd98 — the gadget deleted, the beam deleted. The final
   position (−34.618, 1.450, 33.600) and the shot line eye (−34.618, 1.700, 33.600) → body
   (−32.197, 1.756, 36.300) (3.63 u) are logged for the app replay.

**Named groups green**: `tripwire_shot_live` 4 passed (5.4 s), `interactables_07000014_live` 10
passed (11.3 s), `walkreal_07000014_live` 7 passed (89.8 s, the new test included).

## The one deviation, with the evidence

**No ≥12 shooting position exists in the port's collision.** 26 candidate spots probed on the
final run (the diagnostics print per-spot rejections): every ≥12 line to the rack-wall bomb is
blocked — the shelf slab shadows the lane from the west, the pit walls block the south-west
(t 0.09-0.36 blockers), the doorway's north jamb kills the sill-head lines (t 0.33 at
(−35.82, 1.44, 34.42)), and the doorway axis dies in the pit wall at (−41.2, 29.5). The ≥12
**retreat** is real and asserted; the shot then happens from the level's only clear line at 3.63 u
(inside the 8-u blast [M the 8/100 blast], outside the 1.0 beam trigger at 2.7-3.9; the walker's
top-up — this file's own walk allowance — covers the blast damage). The walk back in is real input
(crouch-crawl again).

Also: `tripwire_shot_live`'s `blast_script_playing` assert is unusable on this level — 07000014's
destructible props pre-register 0x0600_0052 SP players at load, so `scripts.players` always
contains one; the deletion observables (gadget gone, beam gone) carry the 0x12cd98 rule here.

## Suite note

The ~1000-binary whole-suite build cannot fit this 9.9 GB sandbox (proven across three sessions on
this disk; two prior sessions died on rustc "No space left on device" mid-build). The task's named
groups ran green (above). This task changed NO shared code: only `walkreal_07000014_live.rs` and
the two probe files — every other test group is untouched by construction.

## What's left

Nothing for the route. Optional polish if the driver wants it: a live check whether the real
game's armoury offers a ≥12 clear shot the port's collision cannot see.

## Live checks for the main agent

1. **The armoury door + keypads** (PCSX2, 07000014): stand Bond on the crate row at
   (−36.5, 1.35, 33.6), burn both keypads (−35.84, 2.30, 32.17) and (−35.84, 2.30, 35.09) with the
   Q-Claw laser — do both burns fire (ch 0x2C/0x2D up, the door at (−35.6, 0.8, 33.6) unlock on
   ch 0x2E)? The port needed both; if the disc's door opens on ONE, the mux is an OR and the test
   can drop a burn.
2. **The ≥12 shot**: at the placed-bomb spot (−32.1, 1.75, 36.3), is there any player position
   ≥12 u with a clear shot (a window/ledge the parse misses)? The port says no (26 spots probed).

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
