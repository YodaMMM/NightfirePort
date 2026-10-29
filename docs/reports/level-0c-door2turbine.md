# level-0c-door2turbine — the report

Task: fix Chain Reaction Part 1's blockers on the real walk, in order, each from the game's
behaviour [M/D], and replace each stand-in in
`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` with the
real-input leg.

## What's done

1. **`door2turbine` #281 is fully decoded — and the door code is NOT the gap.** The disc
   placement census [D, probe `walkreal_probe0c_door_live`]: the door parses kind 0 (swing),
   ch_open 5, ch_lock 0, lock_flag 0, approach 0, state 0x2, glist 0x802d0084, sphere
   (39.25, 12.57, −210.46) r 1.62, 78 collision tris. [M `Door_Activate` 0x135be0]: with
   `info+0x68` (ch_lock) == 0 the channel gate is skipped entirely, and a kind-0 CLOSED door
   takes the swing arm (`Door_SetState(8)`) — the E press opens it. A channel sweep over the
   whole level [D] found NOTHING that writes ch 5 (no trigger with out/list 5, no SS on
   channel 5) — the E press is the door's ONLY opener. The prior session's "state holds 0x2"
   was the ROUTE, not the door: the walker wedged south-west of the leaf, and its use point
   sat 3.1+ from the leaf's live sphere vs the reach 1.0 + 1.62 = 2.62 [M `Collide_PickObj`
   0x1eae80's 0x101 arm]. Every E pulse in the old leg missed by geometry alone.
2. **The door's glist is the leaf slab only [D].** All 78 tris form one slab: x 38.49..40.02,
   y 11.15..14.00, z −210.56..−210.36 (the live sphere centre (40.01, 12.57, −209.69) is the
   spline closed pose — the seed-matrix sphere differs by 1.07; the game rebuilds obj+0x80
   after the pose [M 0x1336f8:209]). No stoop, no frame. Two consequences: (a) the doorway
   floor belongs to other meshes; (b) kind-0xDB door placements never join the port's
   collision instance list (`Level::instances` takes only cel-creating + kind-0x19), so the
   port's world has no door leaf collision at all — the game picks the doors through the
   ForcedList [M `build_LinkToRoom` 0x1c9680:63-66, `Collide_Pick` 0x1eb320:369-375].
3. **The district floor map, all [D] from the disc's collision tris** (the probe commits the
   dump): the SE walkway top strip (x 39.98..40.41, y 12.02, z −218.69..−213.51) ends in a
   wall (y 12.02..14.22) at z −213.51; an 11.61 shelf (x 38.8..39.3, z −214.5..−213.0) and an
   11.09 corridor (x 38.3..39.3, z −212.5..−210.5) lead into the ring SE corner (11.06..11.14,
   x 36..40.5, z −210..−208); the shaft void (bottom 6.17) fills x 40.5..41.7, z −214.5..−211,
   with a 9.06..9.11 step at (x 40.8..41.1, z −210..−210.5); the gate band resumes at
   (44,−209) 11.17 / (44,−208) 10.34 / (46,−207) 10.34. **The E stance exists in the port:**
   from the corner's south-west (39.3, 11.09, −210.2) the use sphere reaches the leaf's live
   sphere at dist ≈ 1.2 < 2.62 — one real-input walk away.
4. **The walkreal keeps its green shape with a new [D] pin.** Leg 9 pins the walkway end wall
   (`segment_hit` at z −213.51 must stay) and the door-closed pin, and its comment now carries
   the whole floor map. `walkreal_0700000c_live`: **2 passed; 0 failed** (re-verified three
   times tonight, including on the final tree shape).
5. Three real-input route iterations attempted the shelf crossing; all wedged or fell. The
   walker balances on a thin ≈13.05 surface at x ≈ 40.45..40.6 (a rail or roof slab whose
   owner is not in the static census — possibly a drone under Bond), walled west at x ≈ 39.99
   (y 12.5), dropping north into the shaft. STUCK dumps are in the run logs.

## How it was verified

- `cargo test --offline --release -p nf-view --test walkreal_0700000c_live -- --test-threads=1`:
  **2 passed; 0 failed** on the final tree.
- The probe binary `walkreal_probe0c_door_live` (committed) reproduces every [D] number above
  from the ISO in one run (`--nocapture`).
- The full four-crate suite (the fast runner, `tools/test/suite.py`) was run on this branch;
  its total line: see below.

## What's left

- The real-input leg through the 11.61 shelf → the 11.09 corridor → the corner E press (the
  walker's rail problem is the only blocker; the stance and the reach are proven above), then
  the 3.5-unit slot jump corner → (44,−209) and the band walk — the district's one remaining
  stand-in.
- Blockers 2–5 from the level-0c-every-objective report (plate #274's cell has no floor in the
  port's static world either — the down ray at (5.79, −111.1) misses; same investigation class).
- The door-leaf collision question (item 2) is a port-wide decision: injecting door glists
  statically would leave a phantom closed leaf in open doorways; the game's per-pick ForcedList
  behaviour is the faithful shape.

## Live checks for the main agent

1. `door2turbine` #281: stand on the ring SE corner (≈ 39.3, 11.1, −210.2), face the leaf,
   press E — expect the state byte (obj+0xF4) 0x2 → 0x8 (opening) and the leaf to swing north;
   note whether any ch 5 activity appears (the port says nothing writes it).
2. The 13.05 surface at (40.5, −214.4): in the real game, what is Bond standing on at the SE
   walkway's east edge — static rail art or a drone? (The port's static census has nothing
   there; the walker grounds on it at 13.04-13.05.)
3. The real crossing from the SE walkway to the ring SE corner: walk the walkway north to its
   end wall (z −213.5) and record how the game descends west onto the 11.61 shelf (a step, a
   drop, or another connection the port's static world lacks).
4. The slot jump: from the corner (x ≈ 40.5, 11.1, −209) record the real jump arc onto
   (44, −209, 11.17) — reachable, or another mechanism.

## Suite

The fast runner on the final tree: build ok (**889 test binaries in 1372 s** — the sandbox disk
filled twice during the build; stale build artifacts were purged between attempts, no repo file
touched). The run was still in progress at the 01:37 UTC stop line: **3 FAILED = exactly the 3
allowed `disc_golden` tests** (the slim ISO's documented allowance) and **no other FAILED /
TIMEOUT / CRASH line** across every binary the run had completed by then. The total line did not
land before the stop; the walkreal pair itself is quoted above (2 passed / 0 failed).

## Usage

not visible to me
