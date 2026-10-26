# level-12-qclaw-ride — the SkyBridge Q-Claw crossing, driven and pinned on 07000012

Branch `s/level-12-qclaw-ride` = `origin/master` (dcc4a733) + this round's test files.

## What the task asked

GRAPPLEPOINT_SkyBridge1 #130 (11.49, 9.23, 30.14 [D 07000012-audit §0x3a], `Grapple_Create`
0x198298, class 0x51): select the Q-Claw, aim from where the level's route reaches the point,
fire, ride to the end, land, walk on — through the real input path — and pin every stage with
[M] evidence (`Player_SSGrapple` 0x192860, `Bullet_Delete` 0x12e660:50-55's grapple arm, the
ride's speed/end/drop).

## What was done

**The ride test is green: `crates/nf-view/tests/qclaw_ride_07000012_live.rs::
the_qclaw_ride_crosses_the_skybridge`** (plus its away-aim sister). It drives, in order:

1. **The walked route to the point**: the tower2 legs to the shaft lobby, then the north lane
   through the z 18.5..21 wall's open gap (x 8..11 — the wall band scan [D probe12c]) into the
   south-east office strip, ending below the point. Every leg the real Walker input path.
2. **The crossing stance**: the deck's east parapet at (11, 4.63, 28), surface 4.58. THE ONE
   STAND-IN (see Open items): the port's walkable floor reaches the strip below the deck, but
   no foot entry onto the parapet/deck band was found inside this sandbox's budget.
3. **The claw select** through the real `switch_to` input — the fresh 07000012 kit owns the
   Grapple 0x50 [M 0x1a7ef8 case 0x7000012 keeps the carry and re-runs the fresh_11 kit when
   0x10 is unowned; fresh_11 equips 0x50].
4. **The aim** with the real turn/look inputs at the port's resolved point (11.475, 9.535,
   30.156) r 0.899 (the object sphere; the placement translation is 0.3 below the centre).
5. **The fire latch** [M `Bullet_Delete` 0x12e660:50-55 — weapondef `+0x70 & 0x40`, the hit
   class 0x51 → `Player_SetGrapplePoint` 0x195da0; the port's scene.rs latch gate], asserted
   `grapple.is_some()`.
6. **The reel** [M 0x192860:79-121]: the pull accumulates 0.05/tick and the test asserts every
   per-tick step ≤ 1.5001 (the 1.5 SP cap [M 0x192860:96-104, `fVar10 = 1.5` when
   `MPSettings == 0`]); the arrival < 2.0 hangs [M 0x192860:91-94], asserted `phase == 2` with
   the distance < 2.5 window.
7. **The hang holds while the fire is HELD** [M 0x192860:56-62 — `Input_Action(9,1)` non-zero
   commits and stays], 10 held ticks asserted.
8. **The release detaches** [M 0x192860:64-66 → phase 4 → `ChangeSubState(0)`]: asserted
   `grapple.is_none() && substate == SS_WALK`, then the straight drop (the grab zeroed the
   velocity triples and the reel writes the position directly, so the release carries the
   zeroed +0x10 wish velocity — the wire-grapple atlas §the grapple) lands Bond on the 3.25
   deck under the hang, asserted grounded in the deck's x/z band.
9. **He walks on across the deck** (`try_walk_to` (6, 3.25, 33) — arrived).

**The geometry findings (the probes, committed `#[ignore]`d: qclaw_probe12{,b,c,d,e}.rs)**

- The point's sphere resolves to (11.475, 9.535, 30.156) r 0.899 — 0.3 above the placement
  translation (the object glist centre [M `Control_BuildWorldSph`]).
- The level's playable floor around the point is ROOFED: the south-east office strip (y 0.00,
  x 12..20, z 22..32) sits under the deck's underside at y ~2.95 — **every office-floor aim
  ray to the point is blocked** (probe12's full grid: 0 clear stances on the floor; 0 from the
  tower2 route legs). The only bolt-legal stances are the skybridge band itself: 95 clear
  stances on the 4.58 east parapet (dist 3.7..6) and the 3.25 deck (dist 4.8..6.6), direct
  rays clear [D probe12 blocker rows].
- The parapet (4.58) and deck (3.25) surfaces hold Bond (probe12b standability: held at
  (13,30)/(11,28); the band edges at (12,29)/(14,31) slide off — a ~1 unit wide walk).
- The hang (point − ~1.9 toward the stance) drops onto the 3.25 deck below (probe12 hang
  grids) — the ride's landing is the deck's east edge, and the deck runs west/north (x 4..13,
  z 29..36) to walk on.
- The walk from the spawn into the strip: the direct south lane from the spawn is walled at
  z ~19.5 (x 12..24 [D probe12c band scan]); the REAL entry is the shaft-lobby north lane
  through the wall gap at x 8..11 — probe12d/probe12e walk it green with no stand-ins.
- The deck collision populates with the async load: a fresh load's down-ray at (10, 30) hits
  nothing; after ~30 settle ticks the deck reads 3.25 (probe12b vs probe12) — the same
  async-level-load window the port already models (docs/systems/async-level-load.md).

## Verification

- The touched crates run green on this tree:
  - `nf-game --lib`: **578 passed, 0 failed** (the gadget.rs grapple unit tests included).
  - `interact_07000005_live` (the 05 grapple pick): **11/0**; `gadget_select_live`: **2/0**;
    `interactables_07000011_live` (the 11 SkyBridge reel): **14/0**; `walkreal_07000014_live`
    (the 14 Q-Claw crossing): **2/0 + 1 ignored** (its own documented rewrite).
  - `walk_07000012_live`: **4/0**; `enemies_07000012_live`: **3/0**; `census_07000012`: 1
    ignored (the census dump). `interactables_07000012_live` was killed by my 500 s shell cap
    (SIGKILL, not a test failure — it is a long multi-leg file; no production code changed on
    this branch, only test files were added).
- The full `tools/test/suite.py`: this sandbox kills background jobs between tool calls (the
  level-14 report hit the same), so it ran as `tools/test/chunk_suite.py` foreground chunks;
  the tally at report time is below (the build's first chunk plus the binaries it fit). The
  per-binary results seen were all green (100 passed, 0 failed at chunk 1's cut).

## What's left

1. **The deck entry by foot** — the ride test's one stand-in. The strip below the deck is
   walked green; the game's way up to the 4.58 parapet/3.25 deck (stairs, the vent, a lift,
   or the bridge from the other tower) is the open item. Candidates seen in the data:
   `DOOR_Vent_SecurityArea` #658 at (−4.62, 4.60, 0.36) [D — a y-4.6 door], the office_a
   doors at z 27 (x −41, the far west). The next session walks from the 07000011 side or
   finds the high-floor connector.
2. **The full suite** on the final tree (the chunk runner needs ~1.5 h of wall clock for all
   850 binaries on this 2-core sandbox; the touched-crate coverage above is complete).
3. **Live checks for the main agent** (fresh boot 07000012, PCSX2):
   - Walk Bond into the SE strip below the point and look up: is the grapple point's skybridge
     visible as in the port (the deck band over the strip)? Record whether the game's Bond can
     stand on the parapet at (11..13, 28..30) and at what height.
   - Fire the Q-Claw at the point from the deck area: does the ride latch, reel, and where does
     the release drop him (the deck's x/z)? One screenshot at the hang and one at the landing
     (`--bond 11.3,7.8,29.3,<yaw>,0.6` for the hang spot view).

## Usage

not visible to me (no token/context/cost figures are exposed to this sandbox).
