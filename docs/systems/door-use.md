# Door use pick — what the E key actually touches

*Evidence tags: [M] decompile (`work/decomp/ACTION.ELF/<addr>_<name>.c`, disassembly-checked
where the C looks incomplete), [D] disc bytes, [L] EE snapshot, [S] port-only choice.
doors-remainder-4 (820c) re-derived the pick chain below Player_Activate; the 820b leaf-mesh
read was wrong and this document is the correction of record.*

## 1. The dispatch chain [M]

`Player_Activate` 0x1a5170 (after the creepwall arm and the 0x95F==6 gate):

1. `Player_GetHeadPos(&centre, player, 0)` 0x195aa0 — the pick origin is Bond's HEAD
   (`obj+0x30`, + the eye column for substates < 5 and 8..9), not the render camera.
2. `Mat_GetDir(&dir, (&glb_viewer)[BLData+0x94e] + 0x120)` — the active viewer's direction row.
3. `centre += dir` — the use point.
4. `Collide_SphereIntersect(1.0f, centre, player obj+0x20 cel, player obj, body obj, &list,
   0, 0, 0, 0)` 0x1ec940 — mask `HITTEST+0x94 = 0x101` (param 8 is 0), pick centre copied to
   HITTEST+0x60, radius to +0x6c.
5. The class walk over the (distance-sorted) HITDATA list — **no distance gate**: every node's
   `HITDATA+0x58` object dispatches on `obj+0xFF`; `0x1C → Door_Activate(obj, player, 1)`
   (0x1a5170:113-137).

## 2. The candidate is sphere-vs-sphere [M — the part 820b misread]

`Collide_Pick` 0x1eb320 walks the cels the pick sphere straddles (`Collide_StraddleCels`) and
fills the HITDATA list from two routes, both sphere-based:

- **Per-object route** — `Collide_PickObj` 0x1eae80 (every object of the straddled cel with
  `obj+0xF0 & 0x30000000 == 0`): for mask 0x101 the arm at the tail computes
  `|pick_centre − obj+0x80|² < (HITTEST+0x6c + obj+0x8C)²` — the pick sphere vs the object's
  **world sphere** — and allocates the node (`HITDATA+0x58 = obj`, +0x54 = 0). The 0x200
  arm's segment variant and the 0x800 arm (radius ×1.5) are other masks; 0x101 never takes
  them. The `obj+0xD8` (glist) gate only ADMITS the object to the pick (for mask ≠ 0x100 it
  requires a non-empty glist); it does not shape the test.
- **Linked-object route** — the cel's `+0x18` list, same sphere test
  (`pick_radius + obj+0x8C`), with an AABB variant when `obj+0x3C & 0x8000000`.

`obj+0x80`/`obj+0x8C` is the world sphere `Control_BuildWorldSph` 0x133070 maintains: the
glist bounds centre (glist+0x10) posed by the object matrix, radius glist+0x1c · |obj+0xe8|;
the door's moved pass rebuilds it after the walk [M 0x1336f8:209] — the port's
`Door::live_sph()`.

**The refinement cannot reject the candidate.** `Collide_SphereIntersect` 0x1ec940's post-pick
loop calls `Collide_Intersect` 0x1ebaa8 per node (sphere vs the object's mesh triangles), but
uses its return ONLY for the `param_10 & 0x10` early-out — param_10 is 0 here. A node whose
mesh test fails keeps its sphere-candidate distance and stays in the list; the class walk
dispatches it anyway. `Collide_Intersect`'s `obj+0xFD & 4` fast path (mask 0x101) is purely
spherical too (it recomputes the contact from the two spheres).

## 3. Consequences [M ⇒ the port's rule]

- The E reaches a door iff `|use_point − door.live_sphere_centre| < 1.0 + door.sphere_radius`.
- No occlusion test exists anywhere in the chain — the door opens through a wall when the
  sphere is touched from beside the doorway (pinned by doors_remainder_live's span-side stand).
- The leaf mesh never gates the activation; the reach is the world sphere, which for a door is
  the whole doorway's bounds sphere (r ≈ 1.3-1.6 on the census doors), not a thin plane.
- The open leaf's sphere is the moved one: a kind-2 slide's sphere travels with the leaf
  (07000001's door #0: z 1.94 → 3.53 open; 07000016's lift door: y 1.02 → 3.30 — probe), so
  the E-close must be aimed at the moved leaf, and a stand inside the leaf's sweep gets the
  close reversed by the anti-crush (`Door_ReactToHit` 0x135980's kind-2 arm reopens a CLOSING
  door that meets an actor) [M].

Port: `crates/nf-game/src/door.rs` `Doors::player_use` — the use sphere vs `live_sph()`
(glist-resolved doors); doors whose glist never resolved keep the pre-sphere placement-distance
stand-in (< 2.5) [S]. The aim-hint route (`Doors::pick_target`, Check_Target 0x1a7810) is a
different pick and keeps its own documented stand-in.

Tests: `doors_remainder_live::the_use_pick_is_the_world_sphere_candidate` (4.0 out short /
1.2 out opens / 2.3 span-side opens through the wall), the three master E tests
(interactables_0700000b `waiting_room_door_opens_on_use`, level_end
`door_e_opens_then_e_closes_on_07000001`, `the_walkin_leaves_a_closing_straddle_door_alone_on_07000016`).
