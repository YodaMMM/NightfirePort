# level-05-west-lane-3 — 07000005's west lane: the two maps settled, the lane walked, the creepwall fixed

Branch `s/level-05-west-lane-3` (from origin/master 8b101249d). Deliverables: the route-map
settle, the west lane walked green and un-ignored through real input, one port fix
(creepwall.rs) with the disc's own evidence, and the sharpened round-4 blockers.

## The two route maps — settled (the disc collision decides)

Master's ignored test doc (the "tower top → walk-off fall → the west roofs → the west lane at
x −27..−22" map) is **wrong as a route**. A connectivity census over the port's collision of
the disc's own data (1-unit grid, up-steps ≤ 0.55 — the game's step rule, see below — walls
respected with mid-body ray checks, ceilings ≥ 1.6 for standing) gives:

- The y-28 walk-off landing's island (x 57..63, z 85..91) is **isolated** — 19-38 cells with no
  walk edge to the courtyard system, the roofs, or the west lane. The "roofs run west"
  continuation is not in the disc. [D — the census is reproducible from
  `crates/nf-view/examples/probe_l05_west.rs`'s `connectivity()` (scratch example, not
  committed; the method is described in the test doc)]
- The **castle-wall lane map (west-lane-2's) is the real one**: the courtyard west strip →
  x 26-35 lane band → the tower top (x 28-34, z 39-64, floor 36.0) is ONE walkable component
  (899-941 cells, floors 23.6-36.0) [D]. The tower top's west edge faces the y-33.5 walkway
  slab (x −19..25, z 45..55, floors 33-34) — the lane-to-ledge link is the walk-off off the
  top's west edge; the slab's west end faces the cable ledge (x −28..−23, z 19..40, floors
  30-34) [D].

Master's `walk_the_tower_top_and_the_walk_off_fall` still pins TRUE geometry (the balcony is a
caged perch; the walk-off lands on the y-28 island) — its doc now says the landing is an island
and names the real lane (fixed on this branch, with the census cited). Not weakened; the
fall/cage asserts are untouched.

## Done and verified

1. **The west lane walks green, UN-IGNORED** (`walkreal_07000005_live::
   walk_the_west_ledge_and_cross_the_cable`): the cold start → the courtyard west strip → the
   castle lane (wall south 1/2 anchors) → the tower top — all through the real input path
   (`step_bond` movement only; no teleports, no state pokes). The final asserts pin the walked
   positions with [D] cites (the 1-unit floor maps + the census, all through the port's
   collision of the disc's own data).
2. **The lip question answered.** The 29.3 → 30.3 lip at x 42-44 is a 1.0-unit face. The
   game's step rule: the walk capsule's bottom sphere (r 0.55 [M Player_Collision 0x194df0
   case 0: radius slot 0x3f0ccccd, B = pos + (0.55 − cc)]) rides a step only below its radius;
   `Player_SSWalk` 0x192310 → `Player_Move` 0x191dd0 contain no step raycast or step-offset
   assist (the full decompile of both read). A 1.0 face is NOT cleared by the game — the test
   takes the strip's sidestep columns west of the lip (z 92-96, x 32-56 at floors 26-29.3, then
   the x 26-35 lane band [D the 1-unit map]) — the game's own geometry offers it, and the walk
   is green. Nothing to fix in the port.
3. **THE PORT FIX: the creepwall half-length.** The port took the placement scale's x as the
   crawl axis half-length [S] — for 07000005 that is (1,1,1), so the crawl spanned 2 units. The
   disc's OBJECT BOUNDS carry the real span [D]: "Wall Hug Short" x −3.62..+7.52 (≈ 11.1 long,
   r 5.60), "Wall Hug Window" x −5.32..+5.32 (≈ 10.6, r 5.36) — and the crawl band's floor dip
   (z 67-70 down to y 7-8 [D the 1-unit map]) is only crossable with the long span. `creepwall::
   all` now reads the resolved object's bound half-span (hash or index placement reference both
   handled; the scale.x stays as the fallback). Open gap, documented in the field doc: the
   object's bound centre is off-axis (centre x +1.95 for Short) — the port's axis model is
   symmetric about the placement translation; the game's +0x8c semantics (whether the offset
   rides along) is an [S] unverified point.
4. **The continuation re-parked with sharpened blockers** (`the_cable_legs_the_crawl_notch_
   and_ride`, still `#[ignore]` with the reason): (a) the tower-top west parapet — the walked
   west push stalls at (27.22, 37.11, 61.17); the exact map reads 37.1 along x 26, z 58-73 [D]
   — the same pattern as the ladder balcony's parapet; the walk-off gap is unfound. (b) the
   crawl #299 prompt's standing shelf (floor cell (25, 71) = 30.9 [D]) is walled from the lane's
   plateau at x ≈ 26.8 (the walked micro-leg stalled at (27.06, 35.27, 71.63)) — the game's own
   approach onto the shelf is unfound. Then the notch grab, the ride and the south legs
   (unwalked).

## What's left (round 4)

- Find the game's approaches past the two parapet walls (a finer 0.25-unit read of x 24-27,
  z 58-76, or the live checks below), then: the crawl through #299 with the real E press (the
  E-entry leg is already written in the continuation), the slab-end → cable-ledge step
  (the (−24..−23, 45-47) cells [D]), the notch grab at icon #310 (−23.80, 32.74, 12.33) [D],
  the ride, the south ledge, the window-ledge crawl #35 and the exit leg.
- The creepwall axis-offset semantics ([S] above) — a live check would settle it.

## Live checks for the main agent

1. **Fresh boot into 07000005**, walk Bond to the castle lane and up the tower top, then walk
   him WEST off the tower top's west edge around z 58-73 — gdbwatch the player position triple
   while he does it. Expected: the game either stops at a parapet (like the balcony) or crosses
   at a specific z; the crossing z is the round-4 walk-off gap. Address: the player object's
   position (player obj+0x30, the matrix row the port mirrors).
2. Same walk but onto the crawl shelf at (25, 71): watch how the game gets Bond from the lane's
   plateau (y ~34-36) onto the 30.9 shelf — a drop the port's walker refused to plan, or a
   ledge the collision data holds that my 1-unit grid read as a wall (the x ≈ 26.8 stall).
3. `fieldprobe.py` poke/teleport Bond to (25.0, 31.9, 71.0) (on the crawl shelf) and press the
   use key: expected — the crawl substate 7 begins and the type-6 prompt icon arms ("Press
   Action"). This isolates the crawl entry from the approach question.
4. Render spots (with `--bond x,y,z,yaw,pitch`): (44, 30, 91) — the lane head from the north;
   (30, 34, 76) — the lane's tower-north bend; (29.6, 37, 61) — the tower top with its west
   parapet in frame (the 37.1 kerb should be visible at x ~26); (−24, 33, 40) — the cable
   ledge's north end and the 9.9-unit gap to the slab's west end.

## Verification

- `cargo test --offline --release -p nf-view --test walkreal_07000005_live walk_the_west_ledge
  -- --nocapture` — the un-ignored lane walk: **PASS** (6.0 s, run several times during the
  anchor iterations; the final tree's run is the pushed one).
- `cargo build --offline --release -p nf-view --tests` — all nf-view test binaries build.
- The full suite (`tools/test/suite.py`, NF_ISO set, background): build ok — 768 test binaries
  in 863 s; the test-binary run was still in progress at the round's cutoff (no FAILED/TIMEOUT
  lines at report time; the earlier start died on a scratch-example compile error mid-edit and
  was restarted on the final tree). The next message should read `tail -1 suite.out` for the
  total line.
- The decompile reading that backs the step rule: `work/decomp/ACTION.ELF/
  00192310_Player_SSWalk*.c` and `00191dd0_Player_Move*.c` (no step assist), with the port's
  documented capsule constants [M 0x194df0 case 0].

## Usage

not visible to me
