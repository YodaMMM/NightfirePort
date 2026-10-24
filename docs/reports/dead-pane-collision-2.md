# dead-pane-collision-2

Task: make the breakable panes' collision parts solid while alive, without phantom collision, and turn
`BREAK_WORLD_STOP` on.

## What was done

**The finding (subtask 1): the pairing was never wrong.** `crates/nf-game/tests/pane_shape_census.rs`
(the diagnostic + the sister census in one) builds each level's `CollisionWorld` the way the sim does
(`instances(true)`), resolves every `Breaks::create` row's `shape_key` through the port's own shape
index, and prints the PART bounds (the shape transformed by `b.matrix`) against the PLACEMENT bounds
(`b.lo/hi`) for 01, 07, 08, 09 and 0a:

- every pane part lands exactly on its placement bounds — `d=(0.00,0.00,0.00)` per axis (55 panes on 01,
  19 on 07, 3 on 08, 17 on 09, 45 on 0a);
- no double: `dup_static=0` everywhere (no static instance places a pane's resolved object);
- no world-space triangle data: where a raw AABB differs from the placement (01 k=0/16/17/18), the part
  matches the placement and the matrix does the move; where raw == part the placement matrix is
  identity-authoured data;
- every `shape_key` is the anonymous synthetic (`0x8000_0000 | map << 16 | object`) and it matches the
  resolved object (`key_match=false` is the by_hash lookup correctly missing for `hashcode=ffffffff`
  objects; the synthetic key resolves the same object);
- materials printed per object: the sheets carry 0x10 (glass) on 08/0a, 0x02 on the 01 drive glass,
  mixes elsewhere; some 0x40-flagged frame tris.

So the 983 failures were NOT a phantom: the panes are really there and really between the rays and their
actors — the 01 drive glass (placements 130/136) squares the start-eye→guard-158/261 rays (the probe
names the crossed triangles), and the panes are at their correct placements. The game's rays pass them
because of SCOPING, not a pairing rule: the game's ray queries walk ONE cel's linked objects —
`Collide_LineOfSight` 0x1edf50 forwards a single `cel_tag*` into `Collide_RayIntersect` 0x1ec690, whose
`Collide_Pick` object stage (`Collide_PickObj` 0x1eae80) tests the objects of THAT walk — so a pane
object in another room never tests against a cross-room ray. The port has no per-room object graph. The
pinned behaviours all show the game's rays passing the panes: the 07000001 start guard's sight ack lands
during his scripted walk with the glass square across the ray [L 01-doors.trace; guard_first_shot_01],
the drawn gate keeps the guards drawn (sight_frustum), the escort's cover run crosses the sheet
(escort_cover). The game's own pane-collision rule is [M Break_Kill 0x12b970:59-63 — the pane's
collision block at `*(obj+0xd8)+8`, the dead flag obj+0xFE |= 1, the `if (0) return` silent kill] +
[M Break_Create 0x12c230 `Control_SetGList`], and the object-pick gate [M Collide_PickObj 0x1eae80 —
the pick proceeds only when `obj+0xfa & 8` or the collision block is non-zero].

**The fix (subtask 2).** The pane parts now ride the per-tick `set_dynamic` set (drone.rs, the
`pane_parts` block) carrying the `PANE_PART` id bit (`0x400_0000_0000 | breaks index`,
`world::PANE_PART`):

- the RAY sets skip the pane parts: `world.rs for_ray_tris` (shared by `segment_in_sets_drop_pass` and
  `segment_blocked`, i.e. `segment`, `segment_actor`, `segment_bullet`, `segment_clear`,
  `line_of_sight`) walks the dynamic set per object and drops the `PANE_PART` ids — the port stand-in
  for the game's per-cel object walks (documented at the function);
- the CAPSULE queries still test the pane triangles (dead_pane's intact-pane walk stops Bond at the
  sheet, min z −121.448 vs plane −121.998);
- the pane parts' triangles carry the node pass flag in the dynamic build (`set_dynamic` pushes
  `node_pass` for `PANE_PART` parts), so `segment_bullet`'s first stop is never the pane's own mesh;
- `BREAK_WORLD_STOP` is ON: with the breakable meshes ray-skipped parts, `segment_bullet`'s first stop
  is the scenery behind the breakable, so the bounds contact (grown 0.1) always fires at or before it —
  the old 07000007 point-blank pathology (vase #49's mesh stopping the world at t 0.01 before its
  bounds at 0.058) is gone with the ride; the vase/pane contact survives (`t <= ts`) and the round dies
  at it [M Collide_Update 0x1ea398:216-229];
- the breakable contact registers its world impact AT the sheet + the scenery accuracy hit
  [M Bullet_CollisionHandler 0x12ce80 0x12d0a0..0x12d0d0 — Effect_Create on the glass contact, pass and
  stop alike];
- `dead_pane_live`'s two tests are UN-IGNORED and pass: round 1 registers AT the intact sheet and kills
  the health-0 pane, round 2 fires through the dead pane past the plane; the intact pane stops Bond's
  walk (his walk does not break it) and the same walk crosses after the shot.

**The census (subtask 3).** `pane_shape_census.rs::no_pane_part_lies_outside_its_placement_bounds_on_any_sp_level`
runs over every SP level on the disc (07000001–0d, 11–16, 1b, 41, 4a, 4b) and asserts no pane part lies
more than its own bounds away from its placement bounds (per axis, the gap ≤ the placement extent). It
passes.

## How it was verified

Ran (all with `NF_ISO` set, `--test-threads=1`, release):

- `nf-game`: `pane_shape_census` (2 tests: the 5-level diagnostic print + the all-SP-level census) —
  pass.
- `nf-view`: dead_pane_live (2, un-ignored), sight_frustum_live (2), guard_first_shot_01_live (2),
  escort_cover_live (3), fix_objects_live (6), elevator_buttons_09_live (3), fix_player3_live (6),
  render2_live (8), breakables_live (2), gondola_windows_live (3), interact_07000007_live (7),
  interact_07000009_live (14), interactables_0700000a_live (7) — all pass on the final tree.

The full four-crate suite did NOT fit this session's window (the environment kills background jobs
between tool calls, so the chunked runner is foreground-only; the investigation + the named battery
consumed it). What ran is the task's named-test list in full plus the census. The next session should
run `tools/test/chunk_suite.py` over the whole tree.

## What's left

1. The full four-crate suite on the final tree (chunked; see above).
2. The bullet's glass-PIERCE rule is not ported into the contact arm: the game's round passes an
   INTACT material-0x10 pane (weapon +0x6c & 0x800 excepted) [M 0x12cf44..0x12cf64] and continues to
   the next contact; the port's round dies at the pane contact ("the glass absorbs it"). dead_pane
   round 1 only asserts the impact at the sheet, so this stays green, but a round through intact 0x10
   glass should continue — and the guard-fire gate now fires through glass (ray-skipped) where the
   game's aim trace (mask 0xa) stops on non-0x10 glass. A live check would settle both.
3. The ray sets skip pane parts REGARDLESS of material (0x02 drives glass too), standing in for the
   per-room object scoping; a live check of guard vision across the 01 drive glass (does the inside
   guard see Bond through the intact drive glass in the game?) would confirm the scoping reads the
   same.

## Live checks for the main agent

1. 07000001 start, fresh boot: does the inside guard (behind the drive glass, placements 130/136)
   react to Bond walking the drive BEFORE any glass breaks? (My port says yes — the rays skip the
   panes.) Address the ack in the trace tool: `gdbwatch.py` on the guard's +0x274 (the ticks-since-LOS
   word) while Bond crosses the drive.
2. 07000008 control room, fresh boot: fire one P2K round at the intact front pane from the room side —
   does the round register an impact AT the sheet and continue to the shell behind (the glass-pierce
   rule), or stop at the sheet? (The decompile says pierce for material 0x10 [M 0x12cf44..0x12cf64];
   the port currently stops.)
3. 07000007, fresh boot: point-blank at vase #49 — the round should break the vase (the world stop must
   not beat the vase's contact). The port passes this with BREAK_WORLD_STOP on.

## Usage

not visible to me
