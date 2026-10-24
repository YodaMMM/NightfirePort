# level-05-west-lane-4 — 07000005's west lane: the parapet and the shelf answered by the south slope, the crawl #299 walked green, the creepwall +0x8c settled

Branch `s/level-05-west-lane-4` (from origin/master f56c1829c, merged `s/level-05-west-lane-3`
— the west-lane walker and the creepwall bounds fix were not yet on master). Deliverables: the
0.25-unit map that answers both round-3 blockers, the walked-and-un-ignored crawl legs, the
creepwall half-length/centre settled from the decompile, and the parked slab crossing with its
exact blockers.

## The two round-3 blockers — answered by the 0.25-unit map

A floor map of x 23..28.75, z 40..76 at 0.25-unit resolution (the first up-facing hit from
y 45 [D — through the port's collision of the disc's own data, the collision west-lane-2 ray-probed
against the disc]) answers both:

- **The tower-top west parapet is real but never needs crossing.** The parapet's top reads 37.08
  along x 26.0-26.75, z ~48-63 (z 45 reads bare 36.0; z 48 reads 39.5-42.1 — the tower's SW
  corner structure) [D the 0.25 rows]. But the tower top's **south edge has no parapet**: the
  surface slopes down southward from 36.0 (z 62) through 35.2 (z 66) and 34.3-34.5 (z 70-71) to
  33.4-33.7 (z 73-76) along x 27-28.75, and wraps west onto the south ledge (x 23.5-26.0,
  z 73-76, floors 32.4-33.2 — one 0.25-cell floor hole at (26.25, 74.00), passed on its west)
  [D the 0.25 rows]. The ledge runs NORTH along the parapet's west side: 30.0-31.4 at z 58-62
  (the wide shelf), the 0.5-wide waist at x 25.25-25.75, z 67.5-70.5 (the dip west of it reads
  7.0-8.7 — the crawl band), and the slab's east steps 31.7-33.8 at z 49-55 [D]. A BFS over that
  map with the game's edges (up ≤ 0.55 [M Player_Collision 0x194df0 case 0: the r-0.55 bottom
  sphere], down ≤ 6.7 walk-offs, mid-body ray clear, ceiling ≥ 1.6) reaches the crawl shelf AND
  the slab's east end from the tower top with no jump and no gap.
- **The crawl shelf's approach is the same south ledge.** The round-3 stall at (27.06, 35.27,
  71.63) was the UPHILL northbound approach at z 71.5; the downhill wrap at z 73-76 is clear [D].
  The walked test now lands on the shelf the game's way.

## Done and verified

1. **THE CRAWL LEGS WALK GREEN, UN-IGNORED** (`walkreal_07000005_live::
   the_crawl_legs_the_south_slope_shelf_and_slab_east_end`): the cold start → the strip → the
   castle lane → the tower top → the south slope → the south ledge → the crawl shelf → **the
   wall-hug crawl #299 through the real Use press** — entered at (25.51, 31.94, 71.41) substate 7
   ctx 6 (the live entry spot [L fresh boot: one action press enters the wall-hug view at
   (25.51, 32.11, 71.02)]), crawled NORTH across the waist (the port's 0.02/tick documented
   shape), **exited at (25.51, 31.94, 61.17) after 513 ticks** onto the wide ledge — then the
   ledge north and the slab's east steps up to the slab's east end (25.42, 33.98, 51.72), floors
   33.0-33.8 at x 24-25.75, z 49-52 [D]. 4/4 walkreal tests pass (the truck ride, the tower-top
   fall, the west lane, the crawl legs).
2. **THE PORT FIX: the creepwall's +0x8c provenance settled [M]** (`creepwall.rs`). Neither
   `CreepWall_Create` 0x196fc8 nor `control_create_object` 0x1344e0 writes obj+0x8c (the full
   disasm store scan of both: +0x30/+0x40/+0x50/+0x60/+0x90/+0xe0 only). The write is
   **`Control_BuildWorldSph` 0x133070** (called from `build_LinkToRoom` 0x1c9680:40, which every
   Create ends with): obj+0x80..0x8b = the object's LOCAL bound centre through the yaw matrix
   (the world-sphere centre — the same field the icons read), and
   `obj+0x8c = *(float*)(record+0x1c) * ABS(*(float*)(obj+0xe8))` — the **bound-sphere radius ×
   the placement scale**. The port now reads the parsed object's radius (07000005: "Wall Hug
   Short" r 5.60, "Wall Hug Window" r 5.36 [D the parsed objects]) × the scale product, and the
   crawl line's centre is the world-sphere centre (for #299 the +1.95 bound-centre offset rides:
   the centre is (25.79, 31.77, 65.81), the band z 60.21..71.41). Corroboration [D]: the type-6
   prompts sit at z 60.01/70.56 (placements #256/#257) — the old translation-centred band
   (z 62.19..73.33) left the z-60 prompt 2.2 outside; the radius model's band covers both.
   The lane-3 field doc's open point (whether the offset rides along) is settled: it rides [M
   0x133070]. Sister runs: fix_swim_vent_live 2/2, fix_swim_vent5_live 2/2 (the other
   creepwall consumers).
3. **The sister and the round-3 route doc corrected.** `the_notch_holds_the_walk_and_the_grab_
   needs_its_zone` is re-parked (its notch edge sits behind the slab crossing — see below); its
   old west-walk-off approach already failed on the merged tree [L the run at the top of this
   round: the nav "tower top" arrival stood at (29.58, 37.03, 61.15) and the west push never
   landed]. `walk_the_west_ledge_and_cross_the_cable`'s doc now names the south-slope route.

## What's left (round 5)

- **The slab crossing** (the parked `the_cable_legs_the_crawl_notch_and_ride`, re-attached from
  the slab east end). Two blockers, both measured this round:
  - [D] The slab's east tip is fragmented in the disc collision: z 52 is void at x ≤ 23.75;
    (24.0, 54) reads 25.9 between the 33.8 step at (24.0, 52) and the 33.3 strip at (24.0, 55);
    the walkable strip runs z 54-55 at x 2-14 (the 1-unit gap map). The game's link from the east
    steps (x 24-25.75, z 49-52) to the strip needs a 0.25-unit map of x 14..26, z 44..56.
  - [L] Master's one-contact-per-round rework (f56c1829c) drops the ground flag under the
    walker's turn+forward input on the slab top: the walked legs hover ungrounded 1.34 over the
    floor — (25.42, 49.55) and (19.15, 54.55) wedges with floor_below 33.34 — while the SAME
    spots ground under a plain forward push. The raw push then wedges on the [D] tip geometry.
- **The notch grab + ride** (the round-3 legs, code preserved in git history at 96cebefd1) and
  the south ledge legs.
- **The window-ledge crawl #35 and the exit leg**: #35 "Wall Hug Window" at (−34.10, 37.28,
  −31.65) [D], the route's plates at y 37.1-37.3 (#311-#313), the interactables task's open item
  (the port's plunge shell #6 catches the ledge walk at y ≈ 35.3 — band 26.8..35.5 — while the
  game's route runs it safely), the cut-scene guards #65/#68 + the NIS on the approach, and the
  exit door #112 "Locked_Door_Top_I" (37.25, 49.00, −69.00) [D] whose approach is unmapped.

## Live checks for the main agent

1. **Fresh boot into 07000005**, walk Bond down the tower top's SOUTH edge (from the top at
   (30, 36, 60) head south along x 27.5-28 to z 76, then west onto the ledge at (26, 32.6, 75.5),
   then north on x 25.4-25.75 to the shelf) — gdbwatch the player position triple. Expected: the
   game walks the slope and the ledge exactly as the port now does; any z where the game refuses
   is the port-vs-disc divergence to name. Address: the player object's position (obj+0x30 row).
2. Same walk continuing: from the shelf at (25.5, 31, 71) press action (the crawl), ride it to
   the north exit, then walk the slab's east steps (24-25.75, z 49-52) and try WEST along z 52-55
   — gdbwatch. Expected: the game either steps the fragmented tip at a specific z or crosses via
   the z-55 strip; the crossing z is the round-5 leg. Address: the player position triple.
3. `fieldprobe.py` render spots: (28, 36, 64) — the top's bare south edge in frame; (26, 32.6,
   75.5) — the south ledge wrap; (25.5, 31, 71) — the crawl entry shelf; (25, 33, 51) — the slab
   east end with the west strip in frame.

## Verification

- `cargo test --offline --release -p nf-view --test walkreal_07000005_live -- --test-threads=1`
  — **4 passed; 0 failed; 2 ignored** (the parked continuation + the sister) on the final tree.
- `cargo test --offline --release -p nf-view --test fix_swim_vent_live --test fix_swim_vent5_live
  -- --test-threads=1` — 2/2 + 2/2 (the creepwall change's other consumers).
- The full suite CANNOT run to completion in this sandbox: both background starts (23:48 and
  00:48 UTC) died silently between tool calls (the sandbox kills background jobs), and the
  foreground 10-minute chunks do not fit the 768-binary build. What ran instead, on the final
  tree, all green:
  - `cargo test --offline --release -j2 -p nf-game -- --test-threads=1` — 86 test binaries,
    0 FAILED (the creepwall change's own crate).
  - The creepwall-consuming nf-view targets: interactables_07000041_live (3), interactables_
    0700001b_live (2+2 ignored), models_misc_live (6+1), playtest_spots_live (14+2), bath_
    hatch_02_live (10), fix_0c_hatch_live (18), fix_swim_vent_live (2), fix_swim_vent5_live (2).
  - The 07000005 set: walkreal_07000005_live (4 passed / 2 ignored), interact_07000005_live (11),
    walk_07000005_live (2).
  - nf-formats / nf-audio sources are byte-identical to master's on this branch (the merge brought
    only creepwall.rs, this test file, docs and WORKLOG) — master's own gates covered them.
- The decompile readings behind the creepwall fix: `work/decomp/ACTION.ELF/00133070_Control_
  BuildWorldSph__FP7obj_tag.c` (+0x8c write), `001c9680_build_LinkToRoom*` (the call), 
  `00196fc8_CreepWall_Create*` + `001344e0_control_create_object*` (no +0x8c write — the disasm
  store scan), with `00196758_Player_CollCreepWall*` (the +0x8c read as the crawl length).

## Usage

Task system budget not visible to me in this session. The driver-side context: this session ran
from ~23:45 UTC to ~01:00 UTC with the setup pre-existing (repo, work/, ISO); two suite starts
(the first died silently), two full test-target builds, and ~12 walkreal test runs.
