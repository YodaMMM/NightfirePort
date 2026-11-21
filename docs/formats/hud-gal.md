# Gallery files (`.gal`, magic `LAGG`) — driving HUD layout and effects galleries

Reader: `crates/nf-formats/src/gal.rs`. Disc test: `crates/nf-formats/tests/gal_disc.rs`
(goldens `golden/gal_census.tsv`, `golden/hud_gal_elements.tsv`, written by a separate walker
script over the same disc bytes, not by this parser). Consumer: `crates/nf-game/src/driving_hud_draw.rs`
(docs/systems/driving-hud.md).

Evidence tags: [M] = DRIVING.ELF decompile/disassembly (EE addresses), [D] = disc data.

## Where they are [D]

| Archive entry | Stored | Bytes | Content |
|---|---|---|---|
| `data\gfx\data\hudNTSC.gal` in MIS01, MIS11, MIS13A/B/C, MIS3, MIS4, RACE `.VIV` | RefPack (`10 FB`), 251,059 B | 1,051,392 decompressed | the HUD; all 8 copies decompress to the same bytes (SHA-1 `7c14d9be…`) |
| the same name in `MISC.VIV` (#5) | plain | 1,051,392 | byte-identical to the decompressed copies |
| `gall1.gal`, `ears_gall1.gal`, `sav_gall1.gal` in `MISC.VIV` (#2-#4) | plain | 161,280 / 137,216 / 108,800 | effect animations (bullet hits, damage smoke, explosions); no texture sheets inside |

(docs/formats/driving-viv.md's per-extension list put `.gal` among the plain magics; that holds
for MISC.VIV only — the mission copies are compressed.)

## Record tree [M]

Every structure is a 16-byte record `{u32 tag, u32 w1, u32 count, u32 w3}` (little-endian).

- **Tags** compare as u32; the u32 `0x4e616d65` spells `Name` big-endian, so the bytes on disc
  read `emaN`. When `w1 & 1` is set the low 16 bits are an index and the lookup key is
  `(tag & 0xffff0000) | 0x2020` (`FUN_002d5cf8`, `FUN_00236cc0`, `FUN_00237210`, `FUN_00236738`
  build keys this way: `'tx'|n`, `'fi'|n`, `'gk'|n`).
- **Node**: `base = record + w3*16` when `w1 & 2`, else the absolute address `w3`
  (`FUN_002d6940`). Children are the `w1 >> 5` records from `base` (`FUN_002339e8`,
  `FUN_00236b60`); attributes are the `count` records after the children (`FUN_002d5a10`
  searches them; `w1 & 8` = sorted, binary search via `FUN_002d4178`).
- **Attribute**: data at `record + w3` (bytes) when `w1 & 2`, else `w3`; size `w1 >> 8` bytes;
  `count` elements; `(w1 >> 3) & 0x1f` is passed on as a type (`FUN_002d5cf8`).

Every node and non-empty attribute on the disc uses the relative form [D]. The only absolute
records are 11 empty `gk` sets in `ears_gall1.gal` (size 0) [D]; the reader treats an empty attr as
empty and rejects any other absolute pointer.

## Contents [D, `golden/gal_census.tsv`]

```
GGAL (root)
  attrs: fi#0 (165,076 B blob, not decoded), then Name + tx#n pairs (n = 0..5 in the HUD)
  children: GANM ... (animations)
    attrs: Flgs, Name, NUMP, LIFE (u32 each; LIFE = 30 in all four HUD animations)
    children: elements — gb (bitmap), gt (text), qd (quad) in the HUD;
              gp, gg, gd, gs, g3 in the effects galleries
      attrs: Name, gk#0x3000 (the key set; count = number of keys)
```

| Gallery | `tx` sheets | GANM | elements | keys |
|---|---|---|---|---|
| hudNTSC | 6 | 4 (`hud_base`, `hud_msg`, `pauseN`, `paus_err`) | 196 (150 gb, 41 gt, 5 qd) | 204 |
| gall1 | 0 | 106 | 407 | 502 |
| ears_gall1 | 0 | 77 | 357 | 436 |
| sav_gall1 | 0 | 79 | 275 | 354 |

- `tx`#n is an embedded `SHPS` sheet (all 6 parse with `ssh::Ssh::parse` [D]). The loader
  `FUN_00236738` binds `tx`#0, #1, ... into the gallery's texture table (gal+0x10) until one is
  missing, at most 15 [M]. Each `tx` is preceded by a `Name` attr holding a build path
  (`d:\bond2\datapipe\gfx\auf2_hud.ssh`, `auf2_hud2.ssh`, `pausen2.ssh`, `pausen0.ssh`,
  `pausen1.ssh`, `tradel12i.ssh`); the game does not read those names [M — no lookup of the root
  `Name`]; the pairing by file order is [D].
- The key set id the HUD asks for is `FUN_00236b48(gal, 3, 0)` = `3 << 12 | 0` = 0x3000 [M];
  every element on the disc has exactly that set [D].

### Keys

Key stride = `max(0x40, key[0x21] + 0x30)` (`FUN_00236f30`) [M]; on the disc 0x80 for `gb`,
0x54 for `gt`, 0x60 for `qd`, 0xf4 `gp`, 0x40 `gg`, 0x60 `gd`, 0x70 `gs` [D] (each set's size is
exactly count × stride).

Common head (all kinds): +0x00 x, +0x04 y, +0x08 z, +0x0c (a second key of `rdrarrow`
holds 30 there — a key time is [S]), +0x10..0x20 copied as 16 bytes by `FUN_002386c8` (a
quaternion x, y, z, w: the corner pieces `fbw2..4` hold z/w = sin/cos of 45°/−45°/90°, i.e.
90°/−90°/180° — reading it as a rotation is [D]), +0x21 the stride byte. Bytes 0x22-0x2f are 0xcc
filler [D].

Bitmap (`gb`) fields, as `FUN_002386c8` (set style) and `FUN_00238d40` (draw) read them [M]:

| Offset | Field |
|---|---|
| +0x30, +0x34, +0x38, +0x3c | u0, u1, v0, v1 |
| +0x40, +0x44 | width, height (screen pixels), multiplied by the scale |
| +0x48 | colour, ARGB8888; the setter writes `(c&0xfe)<<15 \| (c&0xfe00)>>1 \| (c>>17)&0x7f \| (c&0xfe000000)>>1` = the GS word with R in the low byte and every channel halved |
| +0x4c | texture sheet index (`tx`#n) |
| +0x54, +0x58 | x / y scale in thousandths (×0.001) |
| +0x5c | mirror bits (see the draw rule below) |
| +0x7b, +0x7c, +0x7d, +0x7e | signed repeat counts left, right, up, down |
| +0x64.. | an ASCII source-image name (`AUF2_HUD2.fsh`, `pause1.tga`, ...) [D], not read [M] |

`FUN_00238d40(gal, x, y, key, 0)` draws (x, y = the integer key position for `FUN_002391e8`):
repeat steps −w/+w and −h/+h for left/right/up/down counts > 0; the call count is the sum of
the four counts (0 → 1, negative → nothing). Mirror 0 → one quad at (x, y). Mirror 3 → four
quads: normal at (x, y), both-flipped at (x−w, y−h), x-flipped at (x−w, y), y-flipped at
(x, y−h). Mirror 1 → only the x-flipped quad at (x−w, y); mirror 2 → only the y-flipped quad at
(x, y−h). UVs are inset by DAT_0033aae0 = 1/512 (0x3b000000 [D]; no store to it in the
decompile [M]). The sprite primitive `FUN_0019f9c8` puts (x, y) at the top-left [M].

Text (`gt`) keys: only the position (+0x00/+0x04) is read by the ported HUD passes [M
`FUN_00240c98` reads `elem[0]`, `elem[1]`]. On the disc +0x30 holds an ARGB colour (`b3dbfaff`
bytes for `ammo`/`weapon`) and +0x38/+0x3c 1000/1000 [D]; the font binding
`FUN_002377d8(gal, element)` is not decoded, so those fields stay unnamed.

Quad (`qd`) keys (`statusback*`, 3 keys each) and the effects kinds are walked and pinned by
count and size only.

## Open

- The `fi` blob (165,076 B; `FUN_00237210` returns it) — likely the font image, not decoded.
- Text key fields beyond the position; the `qd` layout; the effects element kinds' fields.
- Whether keys after the first are animation frames over time (+0x0c) — the HUD passes ported
  here read only the first key.
