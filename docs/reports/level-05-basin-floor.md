# level-05-basin-floor — the valley basin's floor east of the lip was the hurt plane

**Branch** `s/level-05-basin-floor` (also pushed as `s/10-level-05-basin-floor`, same commits — the
brief names both). Cut from origin/master 4c2203cc6.

## What the task was

Past the lip at (132.0, 14.17, 125.37) every point resolves find_cel = room 14, and room 14's
collision build covers the east basin but had NO floor quads there, so Bond could not walk down
into the valley the game lets him enter; the west basin floor (room 15) worked. The basin is hurt
#3's band (x 80.78..168.38, z 96.66..144.68, y ~5.1). Find why the port's room-14 build drops that
floor, fix the build with a test that fails on the old code, then un-ignore
`walkreal_07000005_live::the_valley_plane_hurts_bond_walked` if the walk reaches the plane.

## The finding [D]

A full block walk of `07000005.bin` (main map entry `01000099`, 683 blocks; probe
`crates/nf-view/examples/probe_05_collblocks.rs`) shows the port parses every collision block
completely — 120 collision blocks on the main map, parsed triangle count equals the header count on
all of them, no block is overwritten or left unattached. Where the floor actually lives:

- The outdoor room's collision (`Room_Road_A1`, plc 86, kind 0x47, block 262, 3121 tris) ends at
  **z 125.98**; its up-facing floor quads at y 3.5–7 (probe `probe_05_floorscatter`: 31 quads,
  x 81.3..148.6, z 70.5..121.5) stop at z ~121.5. The lip at z 125.37 sits at the very end of the
  room's own collision.
- The west room (`Room_Road_B`, plc 87, block 265, 4979 tris) carries the west valley floor
  (84 quads, x 3.6..79.8, z 54.4..84.7) — that is the working "room 15" floor the ledger measured
  ((47.20, 8.37, 84.06) lands on it, find_cel Some(15) [L the ignored-test ledger]).
- The band east of the lip has exactly ONE floor-level surface anywhere in the archive: **hurt
  placement 331's own 2-tri collision plane at y 5.107** (kind 0xE4 "Hurt_B", transformed bbox
  (80.776, 5.107, 96.662)–(168.382, 5.107, 144.678) — the driver's band exactly; probe
  `probe_05_hurtplc.rs`). The west twin is plc 332 at y 4.378. No room block, no 0x19 mesh piece,
  no sub-map object carries floor quads in the band (the placement scan covers every map's
  placements resolved through `by_hash`/index).

So the "missing room-14 floor" was never dropped by a parse: the floor the game walks IS the hurt
volume's own collision.

## Why the game walks it [M]

`Hurt_Create` 0x17e870 (work/decomp/ACTION.ELF/0017e870_Hurt_Create...c): the hurt is a
class-0x1C object — `Control_SetGList` (decompile :44) attaches the placement's celglist, whose
collision block is the 2-tri plane, and `build_LinkToRoom` (:46) links the object to a room —
`build_FindCel` 0x1c8cb0's pick at the object's position. The player's movement capsule meets the
plane as an object hit and `Player_DealWithObjHit` 0x194458:95-103 applies
`Hurt_GetDamage` 0x17ea18 (param 1 = 1000 on all 13 of this level's volumes [D the census]).
Solid ground and the damage are ONE mechanism: walking onto the valley plane is walking onto the
hurt object. The port already had the damage side (`crates/nf-game/src/hurt.rs`, the per-tick
capsule-vs-tris contact); the MISSING side was collision: `Level::instances(true)` builds
creates-cel placements plus the kind-0x19/0xd5 folds — kind 0xE4 was never built, so the plane did
not exist in the collision world (the ignored test's ledger measured exactly that: ground rays past
the lip read none scoped AND unscoped [L]).

**The FindCel relink comparison the task asked for**: the game links each hurt object to ONE room
(the `build_FindCel` pick at its centre; the tie-break is up/down rays scoped to each candidate's
own chain geometry, closest hit wins, first candidate survives a total miss). On 07000005 the east
plane's centre (124.5, 5.1, 120.4) lies in exactly ONE room cel's bbox — cel 14 (Room_Road_A1's
bbox x 0.13..197.8 covers it; cel 15's x ends at 81.6, cells 30/31 are small rooms at y 15.7+) —
so the relink is unambiguous there. The port's fix puts the hurt planes' runs in the `always`
bucket (the 0x19/0xd5 precedent for linked non-cel objects), which every scoped query tests — a
superset of the one linked room, and exact for every query issued from cel 14, which is where the
ledger put every basin point.

## The fix

`crates/nf-formats/src/level.rs` `Level::instances(cels_only = true)`: after the 0xd5 fold, append
an instance for every map's kind-0xE4 placement whose object carries a collision block and whose
gate (param 2 [M Hurt_Create 0x17e870 reads placement+0x34 into ext+0x18]) is **0**. A gated
volume loads with obj+0xF0 bit 0x20 set (the channel starts low [M 0x17e870 tail]) and
`Collide_Pick` 0x1eb320:140 skips that bit — the game's gated plane is not solid while inactive, so
gated hurts stay out of the static fold (their damage side keeps flowing through the port's
per-tick volume contact, which owns the gate). The appended instances ride the `always` bucket like
the 0x19/0xd5 precedents; the creates-cel prefix order is untouched, so the build_scoped /
attribute_scope pairing is unchanged.

## Verification

- **New test** `walkreal_07000005_live::the_basin_floor_exists_past_the_lip`: loads 07000005 and
  asserts, at the ledger's lip stand and four basin interiors, find_cel = Some(14) and a scoped
  down ray (12 u) hitting y 5.107 ± 0.01 with an up normal. **RED on the pre-fix code** (verified
  by stashing level.rs: "no scoped down-ray hit — the basin floor is absent"), **GREEN with the
  fix** (hit fraction 0.7552 from the lip stand = y 5.107; 0.4077 from y 10 = y 5.107 everywhere).
- The flat (unscoped) ray hits the same plane — the floor exists for the flat redo too.

## What is NOT done

- **`the_valley_plane_hurts_bond_walked` stays `#[ignore]`d.** The floor exists now, but the walk
  still does not reach it: the run+jump drive holds Bond at (132.0, 14.2, 125.4) with health 85.6
  and t400..t1000 position-identical, while every direction ray at dy −6..+2 and d 2–6 reads clear
  and the ground rays ahead now HIT (scoped = unscoped). The lip block is not the missing floor
  any more — it is the next bug down (the walk/capsule edge behaviour at a 9-unit drop with a clear
  sweep), and the task's un-ignore was conditional on the walk reaching the plane. The ledger in
  the test body is unchanged and still accurate about the missing-floor probe; the fix removes its
  "(1)" lever.
- **Gated hurt volumes are not folded** (solidness while their gate channel is high). No walk in
  the battery touches one; flagged for the next session.

## Live checks for the main agent

1. Fresh boot of 07000005, truck route: drive the truck past the lip area (the road keys through
   (132, ~14, 125)) and watch the truck's y — if the game's truck rides over the basin band at
   y ≈ 13.2 (the PlungeTrigger plane y 13.17) or descends to 5.1, that settles which surface the
   game's own vehicles walk. Access: the truck script's path keys, PCSX2 watch on the truck
   object's obj+0x30.
2. Fresh boot: push Bond off the lip (the run+jump at (132, 14.2, 125.4)) and record his y as he
   falls — the game's fall should reach y ≈ 6.2 (feet on the 5.107 plane, obj+0x30 = floor + hip)
   and the −1000 damage record should fire from the hurt contact, not from a plunge trigger.
   This distinguishes "the game walks the plane" from "the game only falls into it".
3. `gdbwatch.py` on hurt placement 331's object record: confirm obj+0xF0 bit 0x20 is CLEAR at load
   (gate 0 → no skip bit) and that `Player_DealWithObjHit`'s class-0x27 arm receives the contact
   when Bond stands on the plane.

## Usage

not visible to me (no token/credit/context figures are exposed in this environment).
