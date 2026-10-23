# The 0x8019/0x8018 statics: the 0x8000000 mark is collision-only (floating-rifle-02-2)

The disc's kind-0x8019/0x8018 placements (`flags & 0xffff`) are static-scenery records the
static parse turns into runtime cels: `parsemap_block_map_data_static` 0x1d1998:71-120 allocates
a cel record for every placement whose flags have bit 0x8000, copies the glist pointer (cel+0x38
= the pCelList record), the placement's two float triples (translation → cel+0x60..0x6c,
rotation → cel+0x70..0x7c) and the glist flags word (cel+0x94 = celglist+0x44), then runs
`parseentity_fixup_entity` 0x1d00c0 and `parseentity_transform_bounding_box` 0x1d0fb0.

This page replaces an earlier reading (which made the port skip every identity-rotation 0x8019/0x8018
and broke `decal_order_live` — 07000007's `hall_shadows*` /
`OBJECT_banistershadows*` dropped and their textures never loaded).

## What the 0x8000000 mark actually is [M]

`parseentity_fixup_entity` 0x1d00c0 marks a 0x8019/0x8018 (and 0x8017) record with flags bit
0x8000000 when **the placement's translation AND rotation vectors are BOTH exactly zero**.
The disassembly of the 0x8019 arm (0x1d02e0-0x1d033c) settles what the C decompile blurs:

- the comparison constant is the ZERO vector at 0x2d6200 (16 zero bytes in the ELF, file offset
  0x1d0100), not an identity row;
- the epsilon is exactly 0.0f (`mtc1 $zero, $f20` at 0x1d02e4, passed in $f12) — an exact
  componentwise compare of cel+0x60..0x6c and cel+0x70..0x7c against (0,0,0);
- cel+0x60/+0x70 are the placement's translation/rotation, not matrix rows: the parse copies
  them from the placement record (+0x0C/+0x18 [D placement layout, nf-formats
  placement.rs]) and `parseentity_transform_bounding_box` 0x1d0fb0 builds the cel matrix as
  `RotMatrix(cel+0x70)` + `Mat_SetTrans(cel+0x60)` — an Euler-angle vector and a translation.

So the mark means "this placement adds NO transform": the record's art draws at its own
(glist-local, world-absolute) coordinates — the artist left the placement blank because the art
was placed in the glist export itself. The ONLY consumers of the bit in the whole decompile are
the collision identity fast paths — `Collide_Pick` 0x1eb320:166,249 and `Collide_Intersect`
0x1ebaa8:142 read it into a variable the decompiler names `Ident`. **No draw path tests it.**

## The full draw chain [M]

1. `build_link_objects_to_rooms` 0x1c9150 links every kind-0x8017/0x8018/0x8019 world cel into
   a room's inner draw chain (room+0x18) when the cel's TRANSFORMED CENTRE (cel+0x80 =
   `RotMatrix(cel+0x70)·glist_centre + trans`, set by 0x1d0fb0) passes, for some room candidate:
   radius gate (cel+0x8c ≤ room+0x8c, both the glist radii), an inclusive bbox test of the
   centre against the room's glist bounds (room+0x40..0x58 vs centre +0x80..0x88; the y test's
   decompile polarity is a Ghidra artifact — the asm at 0x1c9260-0x1c928c is
   `min_y <= y && y <= max_y`), and the closest-geometry pick: `Collide_SphereIntersect`
   0x1ec940 with radius 4001 from the centre, closest hit under 4000 wins, the hit's cel
   (hit+0x54) becomes the owner. A static whose centre passes NO room's gates is linked
   nowhere.
2. `View_AddCels` 0x1e6ba0 walks the visible rooms' inner chains and enqueues each cel into the
   viewer's list 0 at ±(squared distance from the eye to cel+0x80) — gated on the cel's glist
   +0x44 bit 0x10 (the not-drawn flag, docs/fields/glist.md), a kind-0x8018 special case (skip
   when the node at viewer+4 carries kind 0xc023), the 0x40000/display-mask word, and
   `Vision_InView` 0x1e89e0 (the frustum test against cel+0x80/+0x8c).
3. `View_DrawCelGlist` 0x1e6538 draws each queued cel: `psiSetTweakARGB(alpha, 0xff, 0xff,
   0xff)` where `alpha = GameState[0x34] · (cel+0xa3 >> 4)` for modes 1/2 of `cel+0xa3 & 3`
   (mode 1 inverted, modes 0/3 → 0xff) — the byte is set for 0x8019 by the fixup
   (0x1d00c0:162) to `props[0] | props[1] << 4` from the placement's parameter table, and by
   `build_alloc_cel` 0x1c93f8:11 to 0x10 for everything else — then the glist through
   `psiDrawObjectMatrix(cel+0x38, RotZYX(cel+0x70..0x7c) + Trans(cel+0x60))` (cel+0x94 bit 8
   copies the viewer's rotation instead).

## The sandals' numbers [D] (07000002.bin, main map 51)

| record | kind | translation | rotation | params | glist centre | glist radius | glist+0x44 |
|---|---|---|---|---|---|---|---|
| obj 143 bath-sandalblue | 0x8019 | (0,0,0) | (0,0,0) | — | (−39.098, 5.395, 11.266) | 0.339 | 0x0 |
| obj 144 bath-sandalorange | 0x8019 | (0,0,0) | (0,0,0) | — | (−39.073, 5.396, 9.961) | 0.379 | 0x0 |

Their art's own spatial bounds (the glist's GFX boxes) sit at the same world coordinates
(x −39.31..−38.83, y 5.34..5.46, z 9.64..11.55) — the art IS world-absolute — and both records
carry t=r=0, so the 0x8000000 mark is set and the game draws them through the identity
transform at exactly those coordinates, tweak byte cel+0xa3 = 0 (mode 0 → alpha 0xff). The
rooms' gates pass too: the centre lands inside `bath_dressingroom` (0xc047, bounds
x −40.108..−36.185, y 4.289..10.184, z 2.960..12.989, radius 6.107 ≥ 0.379 [D]) and inside
`garden7` (0xc047, radius 48.5). So by the decompile the game draws them at the spot right of
the bathhouse doorway — the port does the same — while the fresh-boot PCSX2 check at the
doorway eye [L 2026-10-20] shows nothing there. The static analysis could not pin the last
step (the likely candidates are depth occlusion by the shared MergedEntity/bath_dressingroom
wall — the eye at (−43.11, 6.46, 10.37) is outside the dressing room, the sandals inside at its
west wall — or the sandals' own visibility at ~4.5 m), so this is held open as a
live check; no port-side hide rule is invented for it.

## The port

`scene.rs`'s static World walk draws every resolved placement — the earlier identity-rows skip is
REMOVED (this page's first section). The port's chain/visibility model already mirrors the
game's draw chain: `build_chains` (scene.rs) implements `build_link_objects_to_rooms`'s owner
pick (`link_owner_room`), the glist+0x44 bit-0x10 skip is `skipped_by_cel_pass`
(nf-formats level.rs:47), the per-frame room visibility is the portal walk
(`portal_visible_cels_game_from_cell_doors`), and the list-0 ordering (rooms at −90000+10k
first, then ±dist²) is gpu.rs's pass-1 sort. Collision is untouched.

Tests: `crates/nf-view/tests/baked_statics_02_live.rs` — the sandals draw AT the game's
measured position (the inverse of the earlier assertion), the 0x8000000 mark mirrors 0x1d00c0's
zero-placement condition, the rotated `northerngarden4_lightfixture` (yaw 0.733) still draws,
and the room cels survive; `decal_order_live` (07000007's shadow decals and their textures)
must stay green.
