# level-14-full-walk-2 — the release-onto-the-lane leg reconciled and green; the walk test back to `#[ignore]` on the next leg's exact blocker

Branch `s/level-14-full-walk-2` = `origin/s/level-14-full-walk` + merged `origin/master` + this
round (commit 52f9092e). The task's core question — reconcile the release with the game's rule
[M Player_SSGrapple 0x192860 phase 2: the first no-fire tick detaches] — is answered with the
printed state walk the task asked for, and the release leg now runs green end to end in the sim.

## What is done

1. **The reconcile, with the printed grapple state.** The instrumented release loop (a scratch
   repro that teleports to the segment stance and replays latch → reel → hang → release, printing
   `(pos, vel, grapple phase/pull, grounded, substate)` every tick) shows the detach firing on the
   FIRST no-fire tick of the release loop — `grapple Some(2) → None` between release tick 0 and 1,
   substate 2 → 0 — exactly the port's reading of [M 0x192860:56-66 (`Input_Action(9,1)` zero →
   phase 4)]. The prior session's "Bond stayed on the wire at (45.37, 4.22, 49.09)" was a
   misread of two stacked facts, not a detach failure:
   - **Fact 1 — the mouths' hood.** The straight drop from the hang (45.37, 4.59, 43.91) lands on
     a solid COVER at top **3.93** spanning z 43.8..48.5 over x 44.8..46.0 — the mouths' hood
     hanging over the east vent lane. The lane (tops 2.38-2.40) is open to the sky only in the
     strip **z 42.3..43.5** [D the down-ray grid, y-6.0 rays over the lane band; the old
     probe14_tunnel/mouthfin grids started their rays below the hood and never saw it]. Bond
     landing on the hood (feet 3.93, center 4.53) is what the old run actually walked on; the
     "north drift" carried him along the hood to its rim at (45.37, 4.22, 49.09) — never detached,
     never on the wire.
   - **Fact 2 — the walkable underside bug (fixed).** Falling off the hood's south edge, the walk
     FROZE mid-air 1.1 above the lane, grounded. Root cause: the port's walkable contact gate was
     `n[1].abs() > 0.7` — DOWN-facing faces set walkable — so the hood's underside (a ceiling,
     normal ≈ −y) grounded the falling capsule. The game's dot1 gate is **up-facing only**:
     `dot(face normal, axis) > 0.7` [M Intersect_CylGeom 0x109da0 disasm 0x10a650-0x10a674 — the
     port's own comment beside the line already stated this rule; the `.abs()` diverged from it].
     Fixed in `crates/nf-game/src/world.rs` (`capsule_tri_contact`): `n[1] > 0.7` — same sign
     discipline as the adjacent pick bit. With the fix the fall passes the underside and lands the
     lane.
2. **The phase-2 hang drift is ported (new).** The game's hang is not static: `Player_Move(1.0)`
   0x191dd0 runs at the top of `Player_SSGrapple` and eases the stick's wish into the committed
   velocity block (the 0.075/tick lanes, the 0.2 ease, the ±0.1 clamp [M 0x191dd0:111-141]); the
   substate tail (LAB_00192a24) commits it every held tick; the engine's move pass applies it while
   the +0x154 rope block pins y — the hang swings horizontally at the walk's speed, never
   vertically. The port now does the same: `Bond::grapple_drift` ([f32; 2], new field on its own
   line at the struct end, zeroed on the release per the pinned no-momentum drop) eased and applied
   with the walk's own axis add during phase 2 while the fire is held. No push rides the drift (the
   game's push tail is dead for the player's move [M Intersect_CylGeom 0x109da0:447 — the DeltaP
   guard]; the port's walk block documents the same).
3. **The release leg route, green.** Hold fire, push the stick south: the hang drifts
   z 44.10 → 42.44 in ~20 ticks at the eased walk speed (y pinned 4.513 throughout — the rope
   pin), over the open strip; release fire (the first no-fire tick detaches [M 0x192860:56-66]);
   the drop lands **(45.497, 2.954, 42.747)** — center 2.954, feet on the 2.38 lane. In the real
   test run: `[014] the mouths hang [45.367, 4.592, 43.908]` → `[014] the vent lane:
   [45.497, 2.954, 42.747] lane true` — the leg's assert GREEN.
4. **The two other tests in the file re-ran green on the walkable-gate change** (the crossing
   chain `the_east_duct_route_reaches_the_stairwell_slot` and
   `the_vent_grill_blocks_the_west_duct_per_the_game`: 2 passed, 1 ignored).
5. The walk test is **back to `#[ignore]`** with the exact blocker in the test's comment (the
   task's rule: never un-ignored and red).

## The exact blocker (what is left)

**Leg 1f's lane-south walk wedges under the hood.** From the lane landing (45.49, 2.95, 42.75),
`walk_to((45.0, 2.4, 40.0))` — the lane south over the flush lid toward the west connector —
stuck (walker_b2 walk_to's stuck assert) on its first approach. The walking head (center 2.95 +
head ≈ 3.9) grazes the hood's underside band (y ≈ 3.4..3.9 over the strip z 42.2..43.5 [D the
down-ray grid; the underside is a down-facing face, invisible to down-rays]). Whether the GAME's
Bond walks under it (the underside sloping up southward over the lid) or the route runs elsewhere
is unmapped: the next session should map the underside band with UP-facing rays
(`world.segment` from y 2.2 upward — the probe `crates/nf-view/examples/probe14_hood.rs` was
written for exactly this but could not run this window: the suite held the build lock), then
re-unignore and walk legs 1f..6 to the exit. Legs 2..6 are untouched from the old route (they were
green on the pre-rewrite route and the report level-14-full-walk pins them).

## What ran

- `cargo test --offline --release -p nf-view --test walkreal_07000014_live` (NF_ISO set, release):
  **2 passed, 1 ignored** — the crossing chain and the vent grill green on the walkable-gate
  change; the main walk ignored (the blocker above).
- The scratch release repro (teleport → latch → reel → hang → drift → release → lane), run
  repeatedly during the reconcile — the printed state walk behind §1/§3. Removed (scratch, not
  committed).
- `cargo build --offline --release -p nf-game -p nf-formats -p nf-audio -p nf-view` — clean.
- `tools/test/suite.py` started in the background at 08:30 UTC (the toolchain setup — rustup +
  ALSA debs into a user prefix + the ISO — consumed the first ~50 min of the window). The tally
  is appended below if it produced one before the window closed.

### Suite tally

(pending at report time — see the addendum below the Usage section if the run advanced)

## Live checks for the main agent

Fresh boot 07000014:

1. At the phase-2 hang over the lane (the reel to the mouths' point #626), hold fire and push the
   stick south: does the game's Bond drift horizontally along the wire at roughly the walk speed
   while y stays pinned (the ported hang drift [M 0x192860 tail + 0x191dd0:111-141])? gdbwatch the
   player's y and z through the hold.
2. Release fire over the open lane strip (z 42.3..43.5): does the game's Bond drop onto the lane
   (2.38-2.40) — and NOT freeze above it? (The port's underside-walkable bug froze him 1.1 above
   the lane; the game's dot1 gate [M 0x10a650-0x10a674] should never ground on the hood's
   underside.)
3. From the lane at (45.4, 2.38, 42.7), walk south past the flush lid (z 42.3..42.2) toward the
   west connector: does the game's Bond pass UNDER the mouths' hood overhang, or does the route
   run elsewhere (the hood's underside band y ~3.4..3.9 vs the walking head ~3.9)? This decides
   leg 1f's premise.

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
