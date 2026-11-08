# Door use pick — what the E key actually touches

*Evidence tags: [M] decompile (`work/decomp/ACTION.ELF/<addr>_<name>.c`, disassembly-checked
where the C looks incomplete), [D] disc bytes, [L] EE snapshot, [S] port-only choice.
This page re-derives the pick chain below Player_Activate; an earlier leaf-mesh
reading was wrong and this document is the correction of record.*

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

## 2. The candidate is sphere-vs-sphere [M — the part the earlier reading got wrong]

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

**The refinement decides (use-pick, correcting the earlier reading of this page).**
`Collide_SphereIntersect` 0x1ec940's post-pick loop calls `Collide_Intersect` 0x1ebaa8 per node.
For mask 0x101 it copies obj+0x90, inverts it, moves the pick centre into object space, presets the
node's distance to 1e8 (:186-189) and runs `Intersect_PointGeom` 0x1e92d8 over the glist's
collision triangles: a triangle registers when the signed plane distance lies in (0, 1.0] (front
side only — HITTEST+0x9a is 0, so `fVar25 = 0`) and the clamped point-to-triangle distance is
≤ 1.0; with no triangle the distance stays 1e8 (:331). The finalize after the loop frees every
1e8 node before the list is returned [M 0x1ec940 `if (fVar10 == 1e+08) LList_Add(free list)`].
The earlier reading looked only at `Collide_Intersect`'s return value (used for the
`param_10 & 0x10` early-out) and missed the 1e8 drop. `Collide_Intersect`'s `obj+0xFD & 4` fast
path is purely spherical, but no door/SS create sets that bit (Door_Create ORs obj+0xfd with 8
only; SS_Create sets obj+0xfa |= 0x44).

## 3. Consequences [M ⇒ the port's rule]

- The E reaches a door iff `|use_point − door.live_sphere_centre| < 1.0 + door.sphere_radius`
  AND the use point lies within 1.0 in front of one of the door glist's collision triangles.
- No occlusion test exists anywhere in the chain: a wall between Bond and the leaf does not stop
  the E when both stages pass.
- A use point inside the leaf (behind every face) or more than 1.0 off its surface misses: the
  press lands from roughly 0.5-1.9 off the leaf, looking at it.
- The open leaf's sphere and faces are the moved ones: a kind-2 slide's sphere travels with the
  leaf (07000001's door #0: z 1.94 → 3.53 open; 07000016's lift door: y 1.02 → 3.30 — probe), so
  the E-close must be aimed at the moved leaf.

Port: `crates/nf-game/src/usepick.rs` (both stages, shared by every use class) called from
`Doors::player_use`. Tests: `use_pick_census_live` (every story level's doors, SimpleScripts,
locks, PCs and monitors from each face the shape allows), `doors_remainder_live`,
`fix_door_react_path_live`, `level_end_live`.

## The glist-0 door stand-in is gone (door-glist0-use-standin, 2026-10-22)

`player_use`'s old fallback — a glist-0 door activating on placement distance < 2.5 [S stand-in] —
is deleted. The game's rule, now ported:

- A dynamic placement resolves its glist by-index (`ref < MapEntityCount` → `pCelList + ref·0x50`,
  else NULL) or by-hash (`hashtable_getitem`) [M parsemap_block_map_data_dynamic 0x1d1b68:77-87],
  and `parsemap_create_dynamic_objects` — which runs `Door_Create` 0x136100 with that glist as
  `param_4` — is called ONLY when it resolved [M 0x1d1b68:93-95, `if (iVar5 != 0)`]. The game never
  creates a glist-less door.
- `Door_Create` stores the glist at `obj+0xd8` (`Control_SetGList` 0x134960) and ORs
  `obj+0xfa |= 0xc4` — bit 3 stays clear — so the use pick's admission for the 0x101 mask
  [M Collide_PickObj 0x1eae80:88-90: `(mask == 0x100 || (obj+0xfa & 8)) || (obj+0xd8 != 0 &&
  *(obj+0xd8+8) != 0)`] reduces to the glist test. A glist-less door is E-dead even if one existed.

The port now skips glist-0 records in `player_use` (they can still exist through the create
fallback `unwrap_or((0, translation, 0.0))` if the by_hash map breaks — the census guards that)
and runs the live-sphere test unconditionally otherwise. The census pin
(`nf-game/tests/door_glist_census_live.rs`): all 21 on-foot SP archives (0700*), 443 doors, every
one resolves a glist through the real closure — zero glist-0. Tests: `a_glist0_door_is_e_dead_at_
the_old_stand_in_reach` + `the_same_door_with_a_glist_still_opens_on_the_sphere_rule` (nf-game
door.rs unit), `door_glist0_e_dead_live.rs` (real path: break the glist → E-dead at the 1.6 stand
the sphere rule opens from; sister opens). No visual difference on any shipped level — the arm was
unreachable on real data; nothing new to render.
