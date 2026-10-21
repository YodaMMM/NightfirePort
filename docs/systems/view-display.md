# The view system: display masks, the object matrix, the glist draw entry, the sprite queue

Never-mentioned render-side batch `unported-render-side-1` (whole-ELF scan, master 20874c31,
queue `backlog/unported-all-2026-10-20.md`). All six functions are REACHED on the shipped
on-foot path — every one has callers in code the game runs every level [M]. Port: `crates/nf-game/src/view.rs`
(no drawing — the pixels stay the main agent's; this delivers the data, state, flow and getters).

Proof levels: [M] code with address, [D] disc, [L] EE snapshot. Decompiler caveats handled by
disassembly: `work/elf/ACTION.ELF` re-read word by word for 0x1e4ff0/0x1e7848/0x1e7ac8/0x1e7b30
(the C dropped argument registers and two stores).

## obj+0xF8 — the display mask (the state all the mask work hangs on)

- `obj+0xF8` u16: the **display-mask bits**, init `0x1F` by `control_init_object` 0x134630 [M
  docs/fields/obj-tag.md]. Bit k = the object draws in viewer k.
- The consumer: `View_CaptureScene` 0x1e6df0 sets the global `object_display_mask` (0x30d87c)
  = `1 << (viewer+0x28 & 0x1f)` per frame per viewer [M], and the draw-list adders filter on
  it: `View_AddObjects` 0x1e62d8:30 (`(obj+0xF8 & object_display_mask) == 0` → skip) and
  `View_AddForcedObjects` 0x1e5fb8:79 (same) [M].
- The propagation source: an object with flags2 bit 0x8 (`obj+0xFA`, "sphere from AnimObject")
  and a non-0 `obj+0xDC` AnimObject propagates the mask to its **attachment objects**:
  `anim+0x70+0x24` (= `anim+0x94`) = the skin, `skin+0x2D` = the attachment count, `anim+0xC0`
  = the `obj_tag*` array [M disasm 0x1e7ac8; docs/fields/animobject.md +0x94/+0xC0]. The write
  goes directly to each attachment's `+0xF8` — **no recursion** through the attachments' own
  links [M disasm 0x1e7af8..0x1e7b24].

## The six functions

### `View_Init` 0x1e77f0 (8 bytes) — REACHED, empty

Disassembly: `jr $ra; nop`, nothing else [M disasm 0x1e77f0]. Called unconditionally by
`ResetMap_GameInit` 0x1d5080 between `control_init_object_lists` and `Sprite_InitLists` [M
001d5080:26]. The port mirrors it as `view::view_init()` — a documented no-op on the reset
path (the port's reset flow keeps its own ordering; the port does not currently call it, a
no-op call would be dead code — recorded, not wired).

### `View_RotTransScaleMatrix` 0x1e4ff0 (376 bytes) — REACHED

`(const VECTOR* angles, const VECTOR* trans, const VECTOR* scale, MATRIX* out)` — the
decompiler lost the registers; disasm shows `s0`=angles, `s1`=out, `s2`=scale, `s3`=trans,
and both `PS2Sinf3` calls [M disasm 0x1e4ff0..0x1e5164]:

```
PS2Sinf3(a0+π/2, a0, a1+π/2)  → cos a0, sin a0, cos a1      (π/2 = 0x3fc90fdb, lui/ori at 0x1e501c)
PS2Sinf3(a2+π/2, a2, a1)      → cos a2, sin a2, sin a1
out[ 0]=cos a1·cos a2·sx      out[ 1]=sin a2·sx         out[ 2]=−cos a2·sin a1·sx
out[ 4]=(sin a0·sin a1 − cos a0·cos a1·sin a2)·sy
out[ 5]=cos a0·cos a2·sy      out[ 6]=(cos a1·sin a0 + cos a0·sin a1·sin a2)·sy
out[ 8]=(cos a0·sin a1 + cos a1·sin a0·sin a2)·sz
out[ 9]=−cos a2·sin a0·sz     out[10]=(cos a0·cos a1 − sin a0·sin a1·sin a2)·sz
out[12..14] = trans[0..2];  indices 3/7/11/15 never written [M disasm — only the 12 swc1 above]
```

This is **not** `RotMatrix` 0x1e2368 / `RotTransMatrix` 0x1e2478 (the sky's Euler order —
different composition, no scale; the port's `sky::rot_matrix` ports those) [M both decompiles
compared]. Here the scale multiplies the matrix ROWS (in the stored transposed form: the input
axes), and the rotation is the other Euler order (verified entry-by-entry against both).
Products are EE-chopped multiplies, the π/2 adds are EE adds (`ps2float`), the sines are
`PS2Sinf3` = the game's `PS2Sinf` per lane (VU0 macro mode; `trig::sinf` is the bit-exact port
[WORKLOG 0026]).

Callers [M]:
- `View_DrawGlist` 0x1e77f8 (below);
- `View_DrawSky` 0x1e5ad0:176 — `View_RotTransScaleMatrix(obj+0x50 rot, stack vec, stack vec,
  stack matrix)` then `psiDrawObjectMatrix(glist, matrix, param)` per sky slot.

### `View_DrawGlist` 0x1e77f8 (76 bytes) — REACHED

`(celglist_tag* glist, VECTOR& pos, VECTOR& angles, VECTOR& scale, uint param)`: builds the
matrix with 0x1e4ff0 then calls `psiDrawObjectMatrix(glist, matrix, param)` [M 001e77f8].

Shipped caller [M]: `AnimObjectDraw` 0x1c4490:316 — the **laser-sight ray**: glist
0x2000131 (view mode 1) / 0x2000133, `pos` = the ray start, `angles` = `Vec_Cartesian_2_Spherical`
of (hit − start) with the first angle negated, `scale` = {1.0, 1.0, |dir|}, `param` = obj+0xEC.
The port's renderer already draws the laser beam from `Player_LaserPointer`'s endpoints
(nf-view scene.rs `laser_beam`); the ported flow (`view::draw_glist`) hands the draw code the
same (glist, matrix, param) record the game would.

### `View_AddSprite` 0x1e7848 (100 bytes) — REACHED

The runtime sprite queue. Disasm-verified semantics [M disasm 0x1e7848..0x1e78a8]:

```
n = param & 0xffff
if 0x3f − n <= SprCnt (u32 compare, sltu — wraps when n > 0x3f → always flush):
    psiDrawSprites(0x34a790, SprCnt)     // 0x120e68; count BEFORE the reset
    SprCnt = 0
slot = SprBuffList + SprCnt*0x30          // SprBuffList = 0x34a790, slot = 0x30 bytes
SprCnt += 1
return slot
```

Flush-then-allocate: the queued sprites (up to 0x3f) draw whole, the new one lands in slot 0.
Caller [M]: `Font_DrawText` 0x1ee598:199,233 — every HUD/menu text glyph: stage a 0x30-byte
sprite at 0x34c7a0, `View_AddSprite(0)`, `amemcpy(slot, 0x34c7a0, 0x30)`. The port's
`view::SpriteQueue` carries the same count/watermark/slot flow with the flush as a closure —
the draw side supplies `psiDrawSprites`.

### `View_SetViewMask` 0x1e7ac8 (100 bytes) — REACHED

`(obj_tag* obj, ushort mask)`: `obj+0xF8 = mask & 0xffff`; if `obj+0xFA & 8` and
`obj+0xDC != 0`, write the same mask to each of the skin's attachments
(`skin+0x2D` count over `anim+0xC0` array) [M disasm 0x1e7ac8..0x1e7b24].

Caller [M]: `BodyGlow_Update` 0x1ac2c8:223,257 — the glow body and each child sprite (the update loop walks 15, `uVar16 < 0xf` — the count master's review pinned)
sprites get the per-viewer mask (`uVar11 & 0xffff | 0x10`): the bit per OTHER viewer whose
view mode is 2 or whose HUD block +0x244 is set, OR 0x10 (bit 4) always. The port's
`body_glow.rs` carries the spawn record; the update-side mask math is `view::set_view_mask`'s
consumer test.

### `View_DrawInThisViewOnly` 0x1e7b30 (108 bytes) — REACHED

`(obj_tag* obj, uint view)`: same body as 0x1e7ac8 with the mask computed as
`1 << (view & 0x1f)` (sllv, then `& 0xffff`) [M disasm 0x1e7b30..0x1e7b64]. Callers [M]:

| Caller | What it masks |
|---|---|
| `Player_PositionCamera` 0x18f5d8:99 (cases 0/0xe) | `BL+0x7E8` view model — first-person weapon visible only in the owner's view (split-screen) |
| `Draw_MuzzleFlash` 0x1a3fe0:116 | the muzzle-flash attachment (`BL+0x874`) |
| `Player_LaserPointer` 0x1a65d8:177 | the laser-sight attachment (`anim+0x94E` view) |
| `Player_InitWeapon` 0x1a7ef8:89,109,120 | the three weapon attachments created at spawn |

## Port map (`crates/nf-game/src/view.rs`)

| Game | Port |
|---|---|
| `View_Init` 0x1e77f0 | `view_init()` — documented no-op |
| `View_RotTransScaleMatrix` 0x1e4ff0 | `rot_trans_scale_matrix(angles, trans, scale) -> [f32; 16]` |
| `View_DrawGlist` 0x1e77f8 | `draw_glist(...) -> GlistDraw` (the record the renderer consumes) |
| `View_AddSprite` 0x1e7848 | `SpriteQueue` (slots, count, watermark, flush closure) |
| `View_SetViewMask` 0x1e7ac8 | `set_view_mask(&mut ObjMask, mask)` + `MASK_INIT` |
| `View_DrawInThisViewOnly` 0x1e7b30 | `draw_in_this_view_only(&mut ObjMask, view)` |
| — | `viewer_display_mask(view)` / `passes_display_mask` (the CaptureScene/AddObjects filter) |

Draw work for the main agent (needs the renderer, not this module): the sky-slot matrices
(`View_DrawSky`'s 0x1e4ff0 use, `--bond` spot: any outdoor level slot), the laser ray's
per-frame matrix, and the split-screen per-view object filtering through
`passes_display_mask` (observable in MP split screen: each player's own weapon/muzzle flash
visible in their quarter only).
