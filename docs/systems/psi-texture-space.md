# psi texture-space pipeline (ACTION.ELF) — Reduce, FillTextureDMA, width helpers

The level-load texture pipeline that packs every map texture into PS2 VRAM "texture space".
`master 20874c31`'s whole-ELF scan flagged several of these as never-mentioned; this doc is
the port-side record. Evidence tags: [M] code (decompile file + address), [D] disc.

## Flow at level load

`GameFlow_Main` 0x1c98c8 → `ResetMap_Load` 0x1d6360 → `psiPreGameRun` 0x11e2e8:

1. `psiCreateMapTextures` 0x11d200 (also called from `parsemap_block_entity_params`
   0x1d1508 while the map parses) walks the map's texture header, loads pixels, and for the
   map's biggest texture calls **`GetMinWidth` 0x1205c8** to size 64-texel strips
   [M 0x11d200:90].
2. **`FillTextureDMA` 0x11d110** (240 B) allocates 0x40 bytes per texture
   (`Mem_Malloc(0x40, 0x1180, 0)`) at `DAT_00254ae8[NumPS2TexturesLoaded * 8]` and fills a
   16-word GS/DMA packet template: `[2]=0x10000000 [3]=0x50000003 [4]=0x8002 [6]=0xe
   [10]=6 [0xe]=8`, rest 0 [M decomp].
3. **`Reduce` 0x11daf0** (520 B) packs: for each loaded texture still marked unplaced
   (record `+0x4 == -1`), claim a free slot in the texture-space table at 0x25caf0
   (0x400 entries, stride 0x1c, used flag at +9), store width/height/offsets
   (`SpaceSize` 0x31697c, `nTextureRectList`/`TextureRectList`), mark the record placed,
   then call `CheckAndAddTextureToSpace` 0x11db58 for every other unplaced texture of the
   same height [M decomp]. The texture-space entries' widths feed `psiDrawSprites`
   0x120e68 and the sprite creators via **`GetMinWidth2` 0x120610**.
4. `CreateSpaceAndPackTextures` 0x11dcf8 → `Conv4to32` 0x11e8d8 → `PageConv4to32`
   0x11e640 → **`BlockConv4to32` 0x11e530** (268 B) converts 4-bpp swizzled blocks to
   32-bit words through the `lut4` table for the upload [M decomp]; its last act calls
   `SaveMemoryMap` 0x111298 — a 36-byte stub that memsets 18 bytes of its own stack and
   returns (verified in the ELF at file 0xb198), i.e. dead debug code [M ELF bytes].

## The pure helpers (ported in `crates/nf-game/src/psi_tex.rs`)

- **`GetBitPos` 0x11f808** (348 B): bit length of n for 1 ≤ n ≤ 0x400, except exact powers
  of two return log2(n); n ≤ 0 or n > 0x400 → 0 [M decomp + disassembly: the >0x400 → 0
  arm is a `movz v0, zero, v1` at 0x11f958 in a `jr ra` delay slot].
- **`GetMinWidth` 0x1205c8** (72 B): `n < 0x41 → 1`, else `(1 << GetBitPos(n)) >> 6`. The
  decomp's `w + 0x3f` is dead — `movn v0, v1, a0` at 0x1205f4 selects `w` whenever `w ≥ 0`,
  which `1 << b` always is, so the real formula is `w >> 6` and n > 1024 → 0 [M
  disassembly]. Values: 1 / 2 / 4 / 8 / 16 for the 64/128/256/512/1024 bands.
- **`GetMinWidth2` 0x120610** (72 B): byte-identical except the small arm is
  `n < 0x81 → 2` — pointer/blob sprites always get ≥ 2 strips [M decomp]. Callers:
  `CreatePointerSpriteListColour` 0x119910, `CreatePointerSpriteList` 0x11a160,
  `CreateLoadingBlob` 0x11bf70, `CacheTexture` 0x11cd58, `psiDrawSprites` 0x120e68.
- **`ByteCompare` 0x1206c0** (64 B): first-mismatch byte equality, `n ≤ 0 → equal`
  [M decomp]; used by `psiCreateMapTextures` to dedup texture records.

## What the port does instead

- No VRAM packing: textures are decoded once (`nf_formats::texture`) and uploaded to normal
  GPU memory (`nf_view::scene`). The packing logic's portable core (the width maths) is
  tested in `nf_game::psi_tex`; Reduce/FillTextureDMA's GS/DMA packets have no host
  equivalent and are cited at `nf_formats::texture`'s module doc.
- `FillSpriteData` 0x120cf8 (364 B, called by `psiDrawSprites`) builds the 52-word GS
  sprite packet (GIF tags 0x6000000c/0x5000000c, TEX0 from the texture-space entry, colours
  from `SpriteColourR/G/B/A` 0x2469d0..); the port's `nf_view::hud` carries the same
  per-sprite data in `HudFrame` sprites and is cited there.
