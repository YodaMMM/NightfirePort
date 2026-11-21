# Driving fonts (`.sfn`, magic `FNTS`)

Reader: `crates/nf-formats/src/sfn.rs`. Disc test: `crates/nf-formats/tests/sfn_loc_disc.rs` (golden
`golden/sfn_census.tsv` from the independent walker `tools/crosscheck/sfn_loc_census.py`; the measure and
layout rules are checked against DRIVING.ELF's own functions run in the EE interpreter).

Evidence tags: [M] DRIVING.ELF decompile/disassembly (EE addresses), [D] disc data, [S] inferred.

## Where they are [D]

| Entry | Copies | Bytes | Use |
|---|---|---|---|
| `data\loading\loading.sfn` | MISC.VIV (plain) + MIS01, MIS11, MIS13A/B/C, MIS3, MIS4, RACE (RefPack), all identical (SHA-1 `fb3fae11…`) | 34,448 | HUD font: `FUN_0023ae30` binds it to `hudNTSC.gal` (`FUN_002378b8` → gal+0xa8); also the loading screen (`FUN_00248480`) |
| `data\loading\error.sfn` | MISC.VIV | 45,840 | error screen font (flags 0x10009) |
| `data\render\debug.sfn` | MISC.VIV | 14,582 | debug text (`FUN_001abd58`) |

## Layout [M]

```
+0x00 'FNTS'  +0x04 u32 size  +0x08 u16 version (0x16e; 0x130 debug)  +0x0a u16 glyph count
+0x0c u32 flags — 0x40000: 16-byte glyphs (i16 advance at +0xe, kern index at +0xc); no disc font sets it
+0x12 u8, +0x13 u8 — newline step = (int)((+0x12 + +0x13) × y scale)
+0x14 glyph table offset   +0x18 kerning table offset (0 = none)   +0x1c glyph sheet record offset
+0x20 colour, +0x28 z (written per draw)   +0x30 / +0x34 f32 u / v texel offset (0.5 on disc)
+0x38 / +0x3c f32 x / y scale — the live draw state (see below)   +0x7c sheet handle (runtime)
glyph (12 B): u16 char, u8 w, u8 h, u16 u, u16 v, i8 advance, i8 x off, i8 y off, u8 has-kerning
kerning: u32 n, n × { u16 previous byte, i8 kern (lb — signed), u8 char }
sheet: one SHPS image record (+ palette attachment), loaded by FUN_0027cd68 like any sheet
```

Glyph tables are sorted by char; 32..255 for loading/error, 32..126 for debug [D].

## Measure: `FUN_00258880` [M]

`FUN_00237ca8(gal, text)` = width output, `FUN_00237ce0` = height output; no scale argument —
they measure at the font's +0x38/+0x3c. Per byte: glyph = the direct entry `c − 0x20` when it holds
`c`, else the binary search `FUN_00259498`; missing → retry with the case swapped (`FUN_00257fb0`)
→ else glyph 0x7f → else skip; a missing `\n` resets pen x to 0 and adds the line step. For a glyph:
pen += kern(prev, c) × sx; left = pen + xoff × sx; top = y + yoff × sy; right = left + w × sx;
bottom = top + h × sy; the box grows (min starts 1e7, max −1e7 — gp−0x7ff0/−0x7fec); pen += advance
× sx; prev = the byte. Width = max − min when min < max, else 0. So the width is the ink box, not the
advance sum. Float ops are EE single (chopped products and sums).

Checked: `Font::measure` gives the same bits as the game code (run on `nf_formats::ee`) for every
string of every loaded language at 1.2 and 1.4, and every English string at 1.0, 1.1, 1.2, 1.4 and
(0.75, 0.9333), plus every English string on error.sfn and debug.sfn at 1.0.

## Layout and glyph sprites: `FUN_00258100` + `FUN_002a1e50` [M]

Same walk with the pen starting at (x, y); each glyph goes to the font callback table's entry 0
(`FUN_00257f80(0x348be0)` installs 0x348be0 = {0x2a1e50, 0x2a20d0, 0x2a25c8, 0x2a31d0,
0x2a3228}). `FUN_002a1e50` writes one GS sprite: corners (int)(pen + xoff × sx), (int)(y + yoff ×
sy) and (int)(pen + (xoff + w) × sx), (int)(y + (yoff + h) × sy); UVs u × 16 + (int)(+0x30 × 16)
… (u + w) × 16 + the same (1/16 texel units). `Font::layout` returns these; checked bit-exact
against the game's sprite writer on every English string at two scales.

## Unproven

- A byte below 0x20 other than a missing `\n`: the game's direct glyph index reads before the glyph
  table (font header or memory before the font); the port treats it as no match.
- The 16-byte glyph form (flag 0x40000) is rejected: no disc font uses it.
- The GS screen offset added to the corners (DAT_00348ba4/ba8, from the begin entry) is the draw
  side's.
