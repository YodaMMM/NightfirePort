//! EA bitmap fonts (`.sfn`, magic `FNTS`) of the driving missions: `data\loading\loading.sfn`
//! (the HUD font — `FUN_0023ae30` binds it to `hudNTSC.gal` through `FUN_002378b8`, and
//! `FUN_00248480` loads it for the loading screen), `data\loading\error.sfn` and
//! `data\render\debug.sfn`. Format and rules: docs/formats/sfn.md.
//!
//! Everything here is read the way DRIVING.ELF reads it [M — EE addresses, decompile under
//! `work/decomp/DRIVING.ELF/`, disassembly checked for every float op]:
//!
//! ```text
//! +0x00 'FNTS'   +0x04 u32 file size   +0x08 u16 version   +0x0a u16 glyph count
//! +0x0c u32 flags (0x40000 = 16-byte glyphs with an i16 advance; set on no disc font)
//! +0x12 u8, +0x13 u8 — a newline moves y by (int)((+0x12 + +0x13) × y scale)
//! +0x14 u32 glyph table offset   +0x18 u32 kerning table offset (0 = none)
//! +0x1c u32 offset of the glyph sheet (one SHPS image record + its palette)
//! +0x20 colour, +0x28 z (both written at draw time)   +0x30 / +0x34 f32 u / v texel offset
//! +0x38 / +0x3c f32 x / y scale (the draw state: written by `FUN_002378a0` before measuring)
//! glyph (12 bytes) = u16 char, u8 w, u8 h, u16 u, u16 v, i8 advance, i8 x offset, i8 y offset,
//!                    u8 has-kerning
//! kerning = u32 n, then n × { u16 previous char, i8 kern, u8 char }
//! ```
//!
//! - `FUN_00258880` measures (`FUN_00237ca8` = its width output, `FUN_00237ce0` = its height
//!   output): the bounding box of the glyph quads, not the advance sum.
//! - `FUN_00258100` lays the glyphs out (pen positions); `FUN_002a1e50` (the font callback
//!   table 0x348be0 entry 0, installed by `FUN_00257f80(0x348be0)` in `FUN_00248480` and
//!   `FUN_001abd58`) turns each into one GS sprite: corners truncated to whole pixels, texel
//!   UVs in 1/16 units plus (int)(+0x30 × 16) / (int)(+0x34 × 16).
//! - Glyph lookup: the direct index `c − 0x20` when that entry holds `c`, else the binary search
//!   `FUN_00259498`; a missing char is retried with its case swapped (`FUN_00257fb0`: A-Z, a-z,
//!   0xc0-0xd6, 0xd8-0xde, 0xe0-0xf6, 0xf8-0xfe), then replaced by glyph 0x7f; a char with
//!   none of those is skipped. A missing `\n` starts a new line instead.
//! - Float arithmetic follows the EE FPU model the shared interpreter uses
//!   (`ps2float::mul` chopped, `ps2float::add_noguard(.., true)` for add.s / sub.s).

use crate::buf::{invalid, Buf, Result};
use crate::ps2float::{add_noguard, mul};
use crate::ssh::{Ssh, SshTexture};

pub const MAGIC: &[u8; 4] = b"FNTS";
/// Flag bit for 16-byte glyph records (`FUN_00258880` `& 0x40000`) [M].
pub const FLAG_WIDE_GLYPHS: u32 = 0x40000;
/// Initial min / max of the measured box (gp − 0x7ff0 / − 0x7fec = 0x3c7e00 / 0x3c7e04:
/// 1e7 / −1e7) [M, D].
const BOX_INIT: f32 = 10_000_000.0;

#[inline]
fn add(a: f32, b: f32) -> f32 { add_noguard(a, b, true) }

/// One glyph record (12 bytes) [M `FUN_00258880` / `FUN_002a1e50` field reads].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub ch: u16,
    pub w: u8,
    pub h: u8,
    /// Top-left texel in the sheet.
    pub u: u16,
    pub v: u16,
    pub advance: i8,
    pub x_off: i8,
    pub y_off: i8,
    /// Non-zero: look this glyph up in the kerning table (`FUN_00259140` `+0xb != 0`).
    pub kerned: u8,
}

/// One kerning pair: `kern` pixels (× x scale) added before `ch` when it follows `prev`
/// (`FUN_002590f8`: u16 at +0 == previous byte, u8 at +3 == char; `lb` at +2 — signed) [M].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kern { pub prev: u16, pub kern: i8, pub ch: u8 }

/// What `FUN_00258880` writes to its four outputs (x, y, width, height) [M 0x258b9c-0x258c10]:
/// x = min x and width = max − min when min < max, else 0 (the same for y / height).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

/// One glyph sprite as `FUN_002a1e50` emits it: pixel corners (before the GS offset) and texel
/// UVs in 1/16 texel units (the GS UV register's fixed point).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphQuad {
    pub ch: u16,
    /// Pen position the layout passed for this glyph (after kerning).
    pub pen: [f32; 2],
    /// x0, y0, x1, y1 = (int)(pen + off × scale), (int)(pen + (off + size) × scale).
    pub xy: [i32; 4],
    /// u0, v0, u1, v1 × 16 (u0 = u × 16 + (int)(+0x30 × 16), u1 = (u + w) × 16 + the same).
    pub uv16: [i32; 4],
}

#[derive(Clone, Debug)]
pub struct Font {
    /// The whole file.
    pub raw: Vec<u8>,
    pub version: u16,
    pub flags: u32,
    /// +0x12 / +0x13 (their sum is the line step).
    pub line: [u8; 2],
    pub glyph_off: u32,
    pub kern_off: u32,
    pub sheet_off: u32,
    /// +0x30 / +0x34 texel offsets.
    pub uv_offset: [f32; 2],
    /// +0x38 / +0x3c as stored in the file (the game overwrites them before every measure/draw).
    pub file_scale: [f32; 2],
    pub glyphs: Vec<Glyph>,
    pub kerns: Vec<Kern>,
    /// The glyph sheet (`FUN_002a31d0` → `FUN_0027e1e8` → `FUN_0027cd68`, the SHPS record
    /// loader) [M].
    pub sheet: SshTexture,
}

impl Font {
    pub fn parse(data: &[u8]) -> Result<Font> {
        let b = Buf(data);
        if data.len() < 0x40 || &data[..4] != MAGIC { return invalid("not a font (FNTS)"); }
        let flags = b.u32(0xc)?;
        if flags & FLAG_WIDE_GLYPHS != 0 {
            return invalid("FNTS with 16-byte glyphs (flag 0x40000): no disc font uses it, not decoded");
        }
        let count = b.u16(0xa)? as usize;
        let glyph_off = b.u32(0x14)?;
        let kern_off = b.u32(0x18)?;
        let sheet_off = b.u32(0x1c)?;
        b.check(glyph_off as usize, count * 12, "FNTS glyphs")?;
        let mut glyphs = Vec::with_capacity(count);
        for i in 0..count {
            let o = glyph_off as usize + i * 12;
            glyphs.push(Glyph {
                ch: b.u16(o)?, w: b.u8(o + 2)?, h: b.u8(o + 3)?, u: b.u16(o + 4)?, v: b.u16(o + 6)?,
                advance: b.i8(o + 8)?, x_off: b.i8(o + 9)?, y_off: b.i8(o + 10)?, kerned: b.u8(o + 11)?,
            });
        }
        let mut kerns = Vec::new();
        if kern_off != 0 {
            let n = b.u32(kern_off as usize)? as usize;
            b.check(kern_off as usize + 4, n * 4, "FNTS kerning")?;
            for i in 0..n {
                let o = kern_off as usize + 4 + i * 4;
                kerns.push(Kern { prev: b.u16(o)?, kern: b.i8(o + 2)?, ch: b.u8(o + 3)? });
            }
        } else if glyphs.iter().any(|g| g.kerned != 0) {
            return invalid("FNTS glyph flagged for kerning but no kerning table");
        }
        let sheet = Ssh::decode_record(data, sheet_off as usize, "font".into())?;
        Ok(Font {
            raw: data.to_vec(), version: b.u16(8)?, flags, line: [b.u8(0x12)?, b.u8(0x13)?], glyph_off, kern_off, sheet_off,
            uv_offset: [b.f32(0x30)?, b.f32(0x34)?], file_scale: [b.f32(0x38)?, b.f32(0x3c)?], glyphs, kerns, sheet,
        })
    }

    /// Glyph lookup of one char as `FUN_00258880` / `FUN_00258100` do it [M]: the entry at
    /// `c − 0x20` when the index is below the count and that entry's char is `c`, else
    /// `FUN_00259498`'s binary search.
    fn find(&self, c: u32) -> Option<usize> {
        let idx = c as i32 - 0x20;
        if idx >= 0 && idx < self.glyphs.len() as i32 && self.glyphs[idx as usize].ch as u32 == c {
            return Some(idx as usize);
        }
        // c < 0x20: the game's direct entry lies before the glyph table (header bytes, or memory
        // before the font for a newline); taken here as a mismatch [S]. No disc font has a glyph
        // below 0x20, so the search below then finds nothing and a newline starts a new line.
        // FUN_00259498(c, table, count, stride) [M].
        let (mut base, mut n) = (0usize, self.glyphs.len() as i32);
        loop {
            if n == 0 { return None; }
            let mid = base + (n >> 1) as usize;
            let m = self.glyphs[mid].ch as u32;
            if c == m { return Some(mid); }
            if (c as i32).wrapping_sub(m as i32) > 0 { base = mid + 1; n = (n - 1) >> 1; } else { n >>= 1; }
        }
    }

    /// The glyph a string byte draws with (or `None` = skipped), with the case swap and the 0x7f
    /// fallback [M 0x258970-0x258aa8]. `\n` (10) without a glyph is `Err(())` = new line.
    fn resolve(&self, c: u8) -> std::result::Result<Option<usize>, ()> {
        if let Some(g) = self.find(c as u32) { return Ok(Some(g)); }
        if c == 10 { return Err(()); }
        let s = case_swap(c as u32);
        if s != 0 { if let Some(g) = self.find(s) { return Ok(Some(g)); } }
        // 0x7f: direct entry 0x5f when the count is at least 0x60 and it holds 0x7f, else search.
        Ok(self.find(0x7f))
    }

    /// `FUN_00259140(font, glyph, prev)` [M]: the kern before `g` when `prev` precedes it.
    pub fn kern(&self, g: &Glyph, prev: u8) -> i32 {
        if g.kerned == 0 { return 0; }
        self.kerns.iter().find(|k| k.prev == prev as u16 && k.ch as u16 == g.ch).map_or(0, |k| k.kern as i32)
    }

    /// `FUN_00258880(font, text, &x, &y, &w, &h)` with the font's x / y scale = `scale` [M].
    pub fn measure(&self, text: &[u8], scale: [f32; 2]) -> Extent {
        let [sx, sy] = scale;
        let (mut pen, mut line_y) = (0.0f32, 0.0f32);
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (BOX_INIT, -BOX_INIT, BOX_INIT, -BOX_INIT);
        let mut prev = 0u8;
        for &c in text.iter().take_while(|&&c| c != 0) {
            let gi = match self.resolve(c) {
                Ok(Some(g)) => g,
                Ok(None) => continue,
                Err(()) => {
                    prev = 0;
                    pen = 0.0;
                    let step = mul((self.line[0] as u32 + self.line[1] as u32) as f32, sy) as i32;
                    line_y = add(line_y, step as f32);
                    continue;
                }
            };
            let g = &self.glyphs[gi];
            pen = add(pen, mul(self.kern(g, prev) as f32, sx));
            let left = add(pen, mul(g.x_off as f32, sx));
            let top = add(line_y, mul(g.y_off as f32, sy));
            if left < min_x { min_x = left; }
            if top < min_y { min_y = top; }
            let right = add(left, mul(g.w as f32, sx));
            let bottom = add(top, mul(g.h as f32, sy));
            if max_x < right { max_x = right; }
            if max_y < bottom { max_y = bottom; }
            pen = add(pen, mul(g.advance as f32, sx));
            prev = c;
        }
        let (x, w) = if min_x < max_x { (min_x, add(max_x, -min_x)) } else { (0.0, 0.0) };
        let (y, h) = if min_y < max_y { (min_y, add(max_y, -min_y)) } else { (0.0, 0.0) };
        Extent { x, y, w, h }
    }

    /// `FUN_00237ca8(gal, text)`: the measured width at the font's current scale [M].
    pub fn text_width(&self, text: &[u8], scale: [f32; 2]) -> f32 { self.measure(text, scale).w }
    /// `FUN_00237ce0(gal, text)`: the measured height [M].
    pub fn text_height(&self, text: &[u8], scale: [f32; 2]) -> f32 { self.measure(text, scale).h }

    /// `FUN_00258100(x, y, font, text)` + `FUN_002a1e50` per glyph [M]: the sprites one draw
    /// emits, at the scale the font holds when it draws (`FUN_00237b80` has already applied its
    /// screen adjustments to `scale`, `x` and `y`).
    pub fn layout(&self, x: f32, y: f32, text: &[u8], scale: [f32; 2]) -> Vec<GlyphQuad> {
        let [sx, sy] = scale;
        let (mut pen, mut pen_y) = (x, y);
        let mut prev = 0u8;
        let mut out = Vec::new();
        let uo = (self.uv_offset[0] * 16.0) as i32;
        let vo = (self.uv_offset[1] * 16.0) as i32;
        for &c in text.iter().take_while(|&&c| c != 0) {
            let gi = match self.resolve(c) {
                Ok(Some(g)) => g,
                Ok(None) => continue,
                Err(()) => {
                    prev = 0;
                    pen_y = add(pen_y, (mul((self.line[0] as u32 + self.line[1] as u32) as f32, sy) as i32) as f32);
                    pen = x;
                    continue;
                }
            };
            let g = &self.glyphs[gi];
            pen = add(pen, mul(self.kern(g, prev) as f32, sx));
            let x0 = add(pen, mul(g.x_off as f32, sx)) as i32;
            let y0 = add(pen_y, mul(g.y_off as f32, sy)) as i32;
            let x1 = add(pen, mul((g.x_off as i32 + g.w as i32) as f32, sx)) as i32;
            let y1 = add(pen_y, mul((g.y_off as i32 + g.h as i32) as f32, sy)) as i32;
            out.push(GlyphQuad {
                ch: g.ch, pen: [pen, pen_y], xy: [x0, y0, x1, y1],
                uv16: [g.u as i32 * 16 + uo, g.v as i32 * 16 + vo, (g.u as i32 + g.w as i32) * 16 + uo, (g.v as i32 + g.h as i32) * 16 + vo],
            });
            pen = add(pen, mul(g.advance as f32, sx));
            prev = c;
        }
        out
    }
}

/// `FUN_00257fb0` [M]: A-Z / 0xc0-0xd6 / 0xd8-0xde → +0x20, a-z / 0xe0-0xf6 / 0xf8-0xfe → −0x20,
/// anything else 0.
pub fn case_swap(c: u32) -> u32 {
    if c.wrapping_sub(0x41) < 0x1a { return c + 0x20; }
    if c.wrapping_sub(0x61) <= 0x19 { return c - 0x20; }
    if c.wrapping_sub(0xc0) < 0x17 || c.wrapping_sub(0xd8) < 7 { return c + 0x20; }
    if c.wrapping_sub(0xe0) <= 0x16 { return c - 0x20; }
    if c.wrapping_sub(0xf8) < 7 { return c - 0x20; }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A font with three glyphs ('A', 'V', 0x7f) and one kerning pair (A then V: −2), a 4×4
    /// 8-bit sheet. Offsets follow the header in the module doc.
    pub(crate) fn fixture() -> Vec<u8> {
        let mut f = vec![0u8; 0x200];
        f[..4].copy_from_slice(b"FNTS");
        f[0xa..0xc].copy_from_slice(&3u16.to_le_bytes());
        f[0x12] = 0x12;
        f[0x13] = 5;
        f[0x14..0x18].copy_from_slice(&0x80u32.to_le_bytes());
        f[0x18..0x1c].copy_from_slice(&0xb0u32.to_le_bytes());
        f[0x1c..0x20].copy_from_slice(&0xc0u32.to_le_bytes());
        f[0x30..0x34].copy_from_slice(&0.5f32.to_le_bytes());
        f[0x34..0x38].copy_from_slice(&0.5f32.to_le_bytes());
        let g = |f: &mut Vec<u8>, i: usize, r: [u8; 12]| f[0x80 + i * 12..0x8c + i * 12].copy_from_slice(&r);
        // 'A': w 9, h 13, u 1, v 2, advance 10, x 0, y 5, kerned
        g(&mut f, 0, [0x41, 0, 9, 13, 1, 0, 2, 0, 10, 0, 5, 1]);
        // 'V': w 9, h 13, u 20, v 2, advance 9, x 1, y 5, kerned
        g(&mut f, 1, [0x56, 0, 9, 13, 20, 0, 2, 0, 9, 1, 5, 1]);
        // 0x7f: w 6, h 6, advance 7, x 0, y 10
        g(&mut f, 2, [0x7f, 0, 6, 6, 40, 0, 0, 0, 7, 0, 10, 0]);
        f[0xb0..0xb4].copy_from_slice(&1u32.to_le_bytes());
        f[0xb4..0xb8].copy_from_slice(&[0x41, 0, 0xfe, 0x56]); // prev 'A', kern −2, char 'V'
        // sheet record: type 2 (8-bit), next 0x20, 4×4, then a type-0x21 palette.
        let s = 0xc0;
        f[s..s + 4].copy_from_slice(&(2u32 | 0x20 << 8).to_le_bytes());
        f[s + 4..s + 8].copy_from_slice(&[4, 0, 4, 0]);
        let p = s + 0x20;
        f[p..p + 4].copy_from_slice(&0x21u32.to_le_bytes());
        f[p + 4..p + 8].copy_from_slice(&[0, 1, 1, 0]);
        f.resize(p + 16 + 1024, 0);
        f
    }

    /// "AV": the pen moves 10 for A, then −2 (kern) before V; the box runs from A's left edge (0)
    /// to V's right edge (10 − 2 + 1 + 9 = 18); height = 13 (both glyphs at y 5..18).
    #[test]
    fn the_box_follows_fun_00258880() {
        let f = Font::parse(&fixture()).unwrap();
        let e = f.measure(b"AV", [1.0, 1.0]);
        assert_eq!(e, Extent { x: 0.0, y: 5.0, w: 18.0, h: 13.0 });
        // Sister: without the kern pair "AV" would be 20 wide; the pair is order-sensitive.
        assert_eq!(f.text_width(b"VA", [1.0, 1.0]), 17.0, "V then A: 1..18, the pair is only A→V");
        // Lower case falls back through the case swap; an unknown byte draws 0x7f.
        assert_eq!(f.measure(b"a", [1.0, 1.0]), f.measure(b"A", [1.0, 1.0]));
        assert_eq!(f.measure(b"#", [1.0, 1.0]).w, 6.0);
        // Scale multiplies offsets, sizes and advances (1.2 chopped).
        let s = f32::from_bits(0x3f99_999a);
        assert_eq!(f.text_width(b"A", [s, s]), mul(9.0, s));
        // An empty string has no box.
        assert_eq!(f.measure(b"", [1.0, 1.0]), Extent { x: 0.0, y: 0.0, w: 0.0, h: 0.0 });
    }

    /// A newline resets the pen and moves down by (int)((0x12 + 5) × y scale).
    #[test]
    fn newline_steps_by_the_line_height() {
        let f = Font::parse(&fixture()).unwrap();
        let e = f.measure(b"A\nA", [1.0, 1.0]);
        assert_eq!((e.w, e.y, e.h), (9.0, 5.0, 23.0 + 13.0));
        let q = f.layout(100.0, 50.0, b"A\nA", [1.0, 0.5]);
        assert_eq!(q[1].pen, [100.0, 50.0 + 11.0], "(int)(23 × 0.5) = 11");
    }

    /// `FUN_002a1e50`: corners truncated, UVs × 16 + 8 (the 0.5 texel offset).
    #[test]
    fn glyph_quads_follow_fun_002a1e50() {
        let f = Font::parse(&fixture()).unwrap();
        let q = f.layout(10.5, 20.0, b"AV", [1.0, 1.0]);
        assert_eq!(q[0].xy, [10, 25, 19, 38]);
        assert_eq!(q[0].uv16, [1 * 16 + 8, 2 * 16 + 8, 10 * 16 + 8, 15 * 16 + 8]);
        assert_eq!(q[1].pen[0], 10.5 + 10.0 - 2.0);
        assert_eq!(q[1].xy[0], (18.5f32 + 1.0) as i32);
    }

    #[test]
    fn case_swap_matches_fun_00257fb0() {
        assert_eq!(case_swap(b'A' as u32), b'a' as u32);
        assert_eq!(case_swap(b'z' as u32), b'Z' as u32);
        assert_eq!(case_swap(0xc9), 0xe9);
        assert_eq!(case_swap(0xf7), 0, "÷ has no pair");
        assert_eq!(case_swap(0xd7), 0, "× has no pair");
        assert_eq!(case_swap(b'1' as u32), 0);
    }
}
