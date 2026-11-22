# s/touch-mesh-contact

**This moves the shared random stream on 0700000b's real-damage walks.** The dome laser grid
no longer trips ch 0x0D/0x0E on the walk's pass, because the beams are overhead. The 00 Agent
walk is re-pinned from lever 2200/30 to 1800/30. A 13-value sweep found that only 1800/30 and
4000/60 complete.

Compares where the port's standing trigger touch and ladder grab fire with where the game's
two-stage HITTEST pick fires, and ports the pick where the gap shows in play.

## The game's test

Bond's walk HITTEST picks the object. Triggers are passive: `Trigger_Create` 0x1b1720 arms no
HITTEST of its own.

- Stage 1, `Collide_PickObj` 0x1eae80 mode 0x800 [M :152-168]: the object's world sphere
  (`Control_BuildWorldSph` 0x133070) ×1.5 plus the HITTEST radius, measured against the closest
  point on the segment, with a strict `<`.
- Stage 2, `Collide_Intersect` 0x1ebaa8 into `Intersect_CylGeom` 0x109da0, run on the glist
  collision triangles. A glist with flags bit 0x40 takes the raw one-sided refine
  (0x1ebaa8:89-96 sets HITDATA+0x51 |= 2 [M]).
- The segment comes from the walk default arm: A = pos + 0.275, B = pos + 0.55 − cc, r 0.55
  [M 0x194df0]. Standing at rest that covers the floor up to floor + 1.91.

## Measurement (`tests/touch_mesh_census.rs`, print-only, every level in `dev::LEVELS`)

The census places Bond standing on the cel floors (rays straight down) on a 0.2 grid that
covers each volume's footprint plus 3.0 on every side. A moving volume is sampled at 8 poses
over its key pass.

- **Ladders, 20 in all:** each one already runs the two-stage pick on its own 0x2E triangles
  (`Ladder::pick_hit` → `refine` → `capsule_hits_tris_strict`), the same code as the wire.
  None differ and none lack a mesh. No change.
- **Touch and TouchOnce volumes a player can fire, 468 in all:** 449 glists carry bit 0x40.
  The stage-1 sphere never changed the answer (0 samples).
- **Before the change,** 362 volumes differed at one or more sampled points.
  - Port fires where the game does not, 130963 samples:
    - the box's extra height: 80923 samples, 159 volumes
    - both-sided against one-sided: 44343 samples, 173 volumes
    - containment: 5697 samples, 21 volumes
  - The game fires where the port does not: 18467 samples, 233 volumes. All of these sit in a
    band no more than 0.55 outside a footprint edge, from the segment's lower reach.
  - No disagreement lies more than 0.55 outside a footprint horizontally. No plate fires from
    too far across the floor.
- **The gap a player notices is the moving laser beams** (glist 0x645): 0700000a #606,
  0700000b #270-277 (ch 0x0d) and 07000014 #315-322 (ch 0x4f). The box's top reach (floor +
  2.9) caught Bond while a beam swept 1.9 to 2.9 above the floor, over his head. The game's
  segment stops at floor + 1.91.
- **After the change,** all 26 beam volumes match the two-stage test at every sample. Overall
  342 volumes still differ, with 79519 port-only samples.

## Change

`triggers::BOX_TOUCHER` marks the measured standing box in `drone::human_touchers`.
`Trigger::touched_by` keeps that box off `seg_beam` glists, which leaves the HITTEST segment
(raw refine) as the only standing contact there. Bond's movement does not change. The crouch
arm, the drones, the riding reach and every non-beam volume stay as they were.

Test: `triggers::tests::overhead_beam_misses_standing_bond`. It fails on the old code, where an
overhead beam at floor + 2.2..2.46 caught Bond, and passes on the new.

## Tests run

These binaries pass:
- nf-game lib `triggers::` (31) and `touch` (5)
- walkreal 0700000a (15), 0700000b (8), 07000014 (9)
- interactables 0700000a (14), 0700000b (14), 07000014 (10)
- interactables_census_live (21), interactables_gadget_census_live (21)
- tripwire_shot_live, walk_0700000b_live, walk_07000014_live, invisible_guard_07000014_live

## Not changed, and why

- **Non-beam volumes keep the box.** Its upper reach is [S]. It is the one the 011 #235 and
  05 gatehouse contacts rely on, and earlier attempts to drop it broke walk chains. The
  volumes left over differ only in edge bands of 0.55 or less, or are cases nobody would see:
  - stacked duplicates on one channel (06 #743-745)
  - kill planes over pit floors that cannot be reached (12 #788-791, 05 #289)
  - volumes with no walkable floor in reach
- **Unproven:** the non-raw branch's ref-point law is not ported. The census measures it as a
  strict both-sided capsule, which matters only for 14 flags-0 volumes. Also unproven are the
  nearest-solid cut-off of the reflected contact list (`Collide_Update` 0x1ea398) and the cel
  scoping. A live check would settle it: on 0700000b's laser grid, stand under a beam while it
  is at its top and confirm ch 0x0d stays 0.
