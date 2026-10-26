# level-0c-display-hatch — the vent route into the display case (0700000c)

**Status: research round complete, both tests still `#[ignore]` (nothing weakened, nothing un-ignored).**
The window filled with [D] geometry mapping and [M] mechanism censuses; the two open legs are now
pinned to exact numbers, and one new route element (the elevated west walkway) was found and walked
in the sim. The remaining unknowns are narrow and are filed as live checks below.

## What the level actually has [D]

All from `nf-extract placements-all 0700000c.bin` + the port's own collision world
(`crates/nf-view/examples/probe_0c_vent.rs`, `probe_0c_standmap.rs`, `probe_0c_openmap.rs` —
committed; the last two print standable-height grids):

- The walkway (floor 14.01, z −110.02..−108.67) ends at a wall face z −110.02 spanning y 13.33..17.13.
  The leg-1 walk trace confirms the port stalls there (capsule r 0.55 short of the face).
- Behind that face is a closed bay: floor **11.84** (z −112.34..−110.39, x 4.78..6.74) with a roof
  **17.28/17.42** (x 4.20..7.32) — that roof IS the previous round's "duct ramp top 17.2-17.5".
- The vent wall plane z −112.25..−112.34 carries the opening **15.03..17.42** with sill lips at
  13.57 / 14.81 / 15.03 jutting north into the bay.
- The balcony (14.61) runs right up to the wall plane (north edge z −112.34); its north strip
  flanks the roof on both sides (x < 4.2 and x > 7.32 up to z −112.0).
- The mid floor **11.27** (z −112.37..−115.50, x 1.88..5.71 + an 11.45 slab east to x 6.80) sits
  under the balcony's north half; its south edge is z −115.5.
- The **display case** is the ground pocket **6.27/6.35** (z −112.0..−109.5) plus the ramp **7.2**
  (z −109.5..−109.0) with the jetpack #414 at (5.50, 7.55, −110.37) — directly under the bay, i.e.
  under the walkway. This matches the test's "floor ~7.3 at (5.4, −113) under the 11.27 mid floor"
  (the 7.2-7.7 ramp region).
- Crate stack `InvPoly_StorageCrates` #443 (4.65, 12.56, −114.24): tops 12.47..14.27 — connects the
  mid floor (11.27) to the balcony (14.61) on the atrium side.
- **The skylight mechanism**: three kind-0xDC touchplates `touchplate_skylight01` (#275-277, trigger
  kind 2 → channel 27) plus Touch triggers **#274 (out ch 27) / #476 (out ch 102)** (both lock ch 8)
  placed at (5.78, 14.02-14.09, −111.05..−111.10) — INSIDE the bay at mid height. The bay roof is
  the display case's glass skylight; the game's route into the case is to walk on that roof and have
  the touchplate fire (kind 2 = break-on-touch family), not to climb down inside the duct.

## Mechanisms ruled out for the ascent [M]

(Explore-agent census, addresses in docs/WORKLOG.md.)

- Jump: launch = WldGravity·(−0.4) = +3.92 (Player_HandleJump 0x1916b0:166-183), gravity 9.8 →
  **max gain 0.784 m**; crouched jump blocked (Player_SSCrouch 0x1924c0 never calls it).
- Step: no step-offset constant anywhere; effective capsule step ≤ 0.55 m; ramps with ny ≥ 0.5 walk
  for any height (the 58° pier ramp precedent).
- Climb: entered ONLY by ladder-object contact (class 0x2D → Player_CollLadder 0x196118:44 via
  Player_DealWithObjHit 0x194458:108-112); a census of every ChangeSubState caller found no other
  writer of substate 1 — and there is no ladder on 0c near the vent [D kind census].
- Lifts: the carry mechanism exists (Control_InheritVelocity 0x133e88:74-79) but 0c has no lift
  placements at the vent; the winch SSs #351/#364/#377 are dead contact-sway props (re-confirmed).
- Q-Claw grapple: the five 0x3A points #308/#346/#500/#508/#511 [D] are all far from the vent
  (nearest #500 at z −189.7); the pull itself is a direct position write
  (Player_SSGrapple 0x192860:91-121) and the port matches it (one divergence noted below).

## What the sim walks showed (real input, this round)

- Leg 1 (walkway → duct top): stalls at z −109.47 — the wall face at z −110.02; nothing on the
  walkway side reaches the roof.
- Legs 2-3 (roof hole → balcony strip): still green — the crouch-walk from the roof crosses the
  opening and lands on the balcony (z −117.7 by tick 90).
- Leg 4 as scripted (straight south): crosses the balcony and OSCILLATES at the pier-ramp base
  (15.62/−120.85 ↔ 15.35/−120.67) — never descends. The mid floor 11.27 is only reachable by
  leaving the balcony sideways, not by walking south.
- NEW: walking WEST off the balcony at (5.8, −112.8) the sim's Bond walks up a ramp onto an
  **elevated west walkway at y 16.2-16.5** (x ≈ −2.8, z −111..−137+) — the same surface that reads
  "16.3" across the vent's north side (z −110.5, x −2..14.5) in the stand maps. So the "16.3 band"
  is a real static walkway ringing the vent, connected south down the atrium's west side and east
  down a ramp (x 15-18, down to ~13.3), and the balcony connects UP to it. From the band the roof
  (17.1-17.2 at its north edge) is a **0.8-0.9 step** — just above the 0.784 jump ceiling [M].

## What is left (next session)

1. The 0.8-0.9 step from the 16.3 walkway onto the skylight roof — the game's own way onto the
   glass (a sloped lip my 0.5-grid under-samples, a script, or a jump the game permits another way).
2. Leg 4's walk with steering: the mapped candidate is sill/balcony → W onto the mid floor 11.27 →
   off its north edge (z −112.4) into the 6.27 pocket → N up the 7.2 ramp to the jetpack; the sim
   walk harness for it is `crates/nf-view/examples/probe_0c_leg4walk.rs` (needs the yaw-per-phase
   walk that run showed working).
3. Un-ignore the two tests as the walks land green.

## Verification: what was run

- `cargo test --offline --release -p nf-view --test fix_0c_hatch_live -- --ignored --nocapture` —
  the two ignored tests execute their walks and fail at the documented stall/oscillation points
  (their other assertions were not touched; both remain ignored).
- The three probe examples + `probe_0c_leg4walk.rs` (nf-view examples) — build clean on this tree.
- `nf-extract` dumps (placements-all, bin-list) — [D] evidence above.
- The full fast runner was started twice in the background and was killed between tool calls both
  times (same as the rook-door-3 round's record); no full-suite number from this sandbox. The
  crates touched by this round: nf-view (examples only — no product code changed), nf-game
  (untouched). Master's party/0c tests are untouched by this branch.

## Live checks for the main agent

1. **The roof step**: with Bond standing on the 16.3 walkway near the roof's north-west corner
   (≈ (5.0, 16.4, −110.5) facing south), press forward — does the real game step/jump onto the
   roof (17.28)? Watch player feet y for a value ≥ 17.0. (If he does it without jumping, the lip
   is walkable and my grid under-sampled it.)
2. **The skylight break**: poke Bond onto the roof (≈ (5.75, 17.6, −111.0)) and walk south along
   it — watch channel 27 (the touchplate_skylight01 output) rise and the roof triangles at
   y 17.28/17.42 (x 4.2..7.3, z −112.3..−110.4) disappear from collision.
3. **The blocker**: gdbwatch the SS #179 matrix y while the last hinge burns — confirm 16.3 → 20.0
   (the stand maps read 16.3 there even in the open state; if the game's poly stays, the band IS
   the poly and the roof access is different).
4. **Leg 4**: poke Bond onto the mid floor (≈ (3.0, 11.4, −114.0)) walking north — does the real
   game let him drop off the mid floor's north edge (z −112.4) into the 6.27 pocket, and does the
   7.2 ramp carry him to the jetpack at (5.5, −110.4)?

## Usage

not visible to me
