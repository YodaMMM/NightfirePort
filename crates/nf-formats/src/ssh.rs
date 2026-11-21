//! EA `SHPS` (`.ssh`) PS2 texture sheet reader (`DRIVING/*.VIV`: track sheets, car sheets, UI).
//!
//! Layout, checked against the driving engine's texture loader `FUN_0027cd68` (DRIVING.ELF) [M]:
//! - Header: `"SHPS"` (4 B), `file_size` u32 LE, `count` u32 LE, `tag` (4 B, `"G354"` and others).
//! - Directory: `count` × 8-byte `{ [u8; 4] name, u32 offset }` from +0x10 (`FUN_00260104` 0x260104
//!   reads `sheet + i*8 + 0x10`, bounded by `count` at +8 [M]).
//! - Record at `offset` (an image, then a chain of attachments): word 0 = `type | next << 8`, where
//!   `next` (signed 24-bit) is the byte distance to the next attachment, 0 = last (`FUN_0027f190`
//!   0x27f190 / `FUN_00306ef0` 0x306ef0 walk `p + (*(int*)p >> 8)` [M]); `width` i16 at +4, `height`
//!   i16 at +6; flags u32 at +0xc: top nibble + 1 = mip levels (capped at 7), 0x1000 = pixel data at
//!   `record + u32 at +0x10` instead of `record + 0x10`, 0x2000 = swizzled upload (8-bit / 4-bit
//!   pixels pre-arranged for a PSMCT32 transfer) [M 0x27cd68].
//! - Bits per pixel come from the table at 0x3438c8 indexed by `type & 0x7f` (`FUN_00260058`
//!   0x260058 [M]); `FUN_0027f0e8` 0x27f0e8 maps them to the GS format: 4 → PSMT4, 8 → PSMT8,
//!   15/16 → PSMCT16, 24 → PSMCT24, 32 → PSMCT32 [M]. Image types on the disc: 0x02 (8-bit),
//!   0x04 (24-bit), 0x05 (32-bit) [D, tests/ssh_disc.rs]; 0x01 (4-bit), 0x03 (15-bit) and 0x06
//!   (16-bit) are decoded from the same tables but no disc sheet uses them.
//! - Palette: the first attachment of type 0x20 or 0x21 (`FUN_0027f190`: `type - 0x20 < 2` [M]);
//!   0x20 is 15-bit (PSMCT16), 0x21 is 32-bit (table 0x3438c8 [M]). The loader always uploads it as
//!   a 16×16 image for 8-bit textures and 8×2 for 4-bit ones, whatever width the palette record
//!   states (`FUN_0027cd68`: `uVar14`/`uVar5` [M]), so 256 (or 16) entries are read from its data.
//! - Colour expansion follows the GS state the engine sets in `FUN_0027d340` 0x27d340 [M]: TEX0
//!   is built with CSM = 0 (CSM1, bit 55 never set) so 256-entry palettes carry the CSM1 order;
//!   TEXA is 0x8080 for PSMCT24 (alpha = TA0 = 0x80) and 0x80_0000_0080 otherwise (16-bit alpha =
//!   0x80 for both A-bit values). PS2 alpha 0x80 = opaque, mapped to 255 like `texture::ps2_alpha_to_8bit`.

use crate::buf::{invalid, FormatError, Result};
use crate::texture::ps2_alpha_to_8bit;

pub const MAGIC_SHPS: &[u8; 4] = b"SHPS";

/// One decoded texture (top mip level) from an EA `.ssh` (`SHPS`) sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshTexture {
    pub name: String,
    /// Image record type with the high bit cleared (`type & 0x7f`, the table index).
    pub format: u8,
    /// Bits per pixel from the table at 0x3438c8.
    pub bpp: u8,
    pub width: u32,
    pub height: u32,
    /// Mip levels the loader uploads (flags top nibble + 1, capped at 7).
    pub mips: u32,
    /// Flag 0x2000: the pixel block was laid out for a PSMCT32 upload.
    pub swizzled: bool,
    /// Palette attachment type (0x20 / 0x21) for indexed formats.
    pub clut: Option<u8>,
    /// Width the palette record states (+4); the loader ignores it and reads 256 / 16 entries.
    pub clut_width: Option<u32>,
    /// Indexed formats: texel indices (after unswizzling) and the palette in index order.
    pub indices: Option<Vec<u8>>,
    pub palette: Option<Vec<[u8; 4]>>,
    pub rgba: Vec<u8>,
}

/// Parsed EA `SHPS` texture sheet.
#[derive(Clone, Debug)]
pub struct Ssh {
    pub tag: [u8; 4],
    pub textures: Vec<SshTexture>,
}

fn u16_le(d: &[u8], o: usize) -> Result<u16> {
    let b = d.get(o..o + 2).ok_or_else(|| FormatError::Invalid("ssh u16 oob".into()))?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}

fn u32_le(d: &[u8], o: usize) -> Result<u32> {
    let b = d.get(o..o + 4).ok_or_else(|| FormatError::Invalid("ssh u32 oob".into()))?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Bits per pixel for the formats the loader can upload (table 0x3438c8 rows 1..6, 0x20, 0x21 [M]).
pub fn bits_per_pixel(ty: u8) -> Option<u8> {
    match ty & 0x7f {
        0x01 => Some(4),
        0x02 => Some(8),
        0x03 => Some(15),
        0x04 => Some(24),
        0x05 => Some(32),
        0x06 => Some(16),
        0x20 => Some(15),
        0x21 => Some(32),
        _ => None,
    }
}

/// PS2 GS CSM1 256-entry CLUT order: within each 32 entries, 8..16 and 16..24 trade places.
pub fn csm1_index(i: usize) -> usize {
    match i & 0x18 {
        0x08 => i + 8,
        0x10 => i - 8,
        _ => i,
    }
}

/// PSMCT16 texel → RGBA8. The GS shifts each 5-bit channel left by 3; alpha comes from TEXA,
/// which the engine sets to 0x80 for both A-bit values (0x27d340 [M]).
fn ct16(v: u16) -> [u8; 4] {
    let r = ((v & 0x1f) << 3) as u8;
    let g = (((v >> 5) & 0x1f) << 3) as u8;
    let b = (((v >> 10) & 0x1f) << 3) as u8;
    [r, g, b, ps2_alpha_to_8bit(0x80)]
}

/// Data start of a record: +0x10, or `record + u32 at +0x10` when flag 0x1000 is set (0x27cd68 [M]).
fn data_start(d: &[u8], rec: usize) -> Result<usize> {
    let flags = u32_le(d, rec + 0xc)?;
    if flags & 0x1000 != 0 {
        Ok(rec + u32_le(d, rec + 0x10)? as usize)
    } else {
        Ok(rec + 0x10)
    }
}

/// The first palette attachment (type 0x20 / 0x21) chained after `rec` (`FUN_0027f190` [M]).
fn find_clut(d: &[u8], rec: usize) -> Result<Option<usize>> {
    let mut p = rec;
    for _ in 0..64 {
        let word = u32_le(d, p)? as i32;
        if (0x20..=0x21).contains(&(word as u8)) {
            return Ok(Some(p));
        }
        let next = word >> 8;
        if next == 0 {
            return Ok(None);
        }
        let q = p as i64 + next as i64;
        if q < 0 || q as usize + 16 > d.len() {
            return invalid("SHPS attachment chain runs out of the file");
        }
        p = q as usize;
    }
    invalid("SHPS attachment chain too long")
}

/// `entries` palette colours from the palette record at `clut`. Bytes past the end of the file read
/// as zero (the loader uploads a fixed 16×16 / 8×2 block whatever the record holds).
fn read_clut(d: &[u8], clut: usize, entries: usize) -> Result<Vec<[u8; 4]>> {
    let ty = d[clut];
    let start = data_start(d, clut)?;
    let byte = |o: usize| d.get(start + o).copied().unwrap_or(0);
    let raw: Vec<[u8; 4]> = (0..entries)
        .map(|k| {
            if ty == 0x20 {
                ct16(u16::from_le_bytes([byte(k * 2), byte(k * 2 + 1)]))
            } else {
                [byte(k * 4), byte(k * 4 + 1), byte(k * 4 + 2), ps2_alpha_to_8bit(byte(k * 4 + 3))]
            }
        })
        .collect();
    if entries == 256 {
        Ok((0..256).map(|i| raw[csm1_index(i)]).collect())
    } else {
        Ok(raw)
    }
}

// GS local-memory layouts (GS User's Manual, ch. 8 "Memory arrangement").
const BLOCK32: [[usize; 8]; 4] = [
    [0, 1, 4, 5, 16, 17, 20, 21],
    [2, 3, 6, 7, 18, 19, 22, 23],
    [8, 9, 12, 13, 24, 25, 28, 29],
    [10, 11, 14, 15, 26, 27, 30, 31],
];
const COLUMN32: [[usize; 8]; 8] = [
    [0, 1, 4, 5, 8, 9, 12, 13],
    [2, 3, 6, 7, 10, 11, 14, 15],
    [16, 17, 20, 21, 24, 25, 28, 29],
    [18, 19, 22, 23, 26, 27, 30, 31],
    [32, 33, 36, 37, 40, 41, 44, 45],
    [34, 35, 38, 39, 42, 43, 46, 47],
    [48, 49, 52, 53, 56, 57, 60, 61],
    [50, 51, 54, 55, 58, 59, 62, 63],
];
const COLUMN8: [[usize; 16]; 16] = [
    [0, 4, 16, 20, 32, 36, 48, 52, 2, 6, 18, 22, 34, 38, 50, 54],
    [8, 12, 24, 28, 40, 44, 56, 60, 10, 14, 26, 30, 42, 46, 58, 62],
    [33, 37, 49, 53, 1, 5, 17, 21, 35, 39, 51, 55, 3, 7, 19, 23],
    [41, 45, 57, 61, 9, 13, 25, 29, 43, 47, 59, 63, 11, 15, 27, 31],
    [96, 100, 112, 116, 64, 68, 80, 84, 98, 102, 114, 118, 66, 70, 82, 86],
    [104, 108, 120, 124, 72, 76, 88, 92, 106, 110, 122, 126, 74, 78, 90, 94],
    [65, 69, 81, 85, 97, 101, 113, 117, 67, 71, 83, 87, 99, 103, 115, 119],
    [73, 77, 89, 93, 105, 109, 121, 125, 75, 79, 91, 95, 107, 111, 123, 127],
    [128, 132, 144, 148, 160, 164, 176, 180, 130, 134, 146, 150, 162, 166, 178, 182],
    [136, 140, 152, 156, 168, 172, 184, 188, 138, 142, 154, 158, 170, 174, 186, 190],
    [161, 165, 177, 181, 129, 133, 145, 149, 163, 167, 179, 183, 131, 135, 147, 151],
    [169, 173, 185, 189, 137, 141, 153, 157, 171, 175, 187, 191, 139, 143, 155, 159],
    [224, 228, 240, 244, 192, 196, 208, 212, 226, 230, 242, 246, 194, 198, 210, 214],
    [232, 236, 248, 252, 200, 204, 216, 220, 234, 238, 250, 254, 202, 206, 218, 222],
    [193, 197, 209, 213, 225, 229, 241, 245, 195, 199, 211, 215, 227, 231, 243, 247],
    [201, 205, 217, 221, 233, 237, 249, 253, 203, 207, 219, 223, 235, 239, 251, 255],
];

/// Byte address of PSMCT32 pixel (x, y) in a buffer `bw` × 64 pixels wide.
fn addr32(x: usize, y: usize, bw: usize) -> usize {
    let page = (y / 32) * bw + x / 64;
    let block = BLOCK32[(y % 32) / 8][(x % 64) / 8];
    (page * 2048 + block * 64 + COLUMN32[y % 8][x % 8]) * 4
}

/// Byte address of PSMT8 texel (x, y) in a buffer whose pages (128×64) are `pages_across` wide.
/// PSMT8 uses the same block order as PSMCT32 (blocks of 16×16).
fn addr8(x: usize, y: usize, pages_across: usize) -> usize {
    let page = (y / 64) * pages_across + x / 128;
    let block = BLOCK32[(y % 64) / 16][(x % 128) / 16];
    page * 8192 + block * 256 + COLUMN8[y % 16][x % 16]
}

/// The 8-bit indices of a flag-0x2000 texture: the loader transfers the block as PSMCT32,
/// `w >> 1` × `h >> 1` pixels with DBW = `((w >> 1) + 63) >> 6`, and the GS samples it as PSMT8
/// with TBW = `(pow2(w) + 63) >> 6` (`FUN_0027f480` 0x27f480 sets the upload shifts to 0x11 and
/// DPSM 0; `FUN_0027f850` 0x27f850 writes TRXREG = `w >> 1`, `h >> 1` [M]). Texels the transfer
/// never wrote read as 0 (on the console they read whatever VRAM held).
pub fn unswizzle8(src: &[u8], w: usize, h: usize) -> Result<Vec<u8>> {
    let (uw, uh) = (w >> 1, h >> 1);
    if uw == 0 || uh == 0 || src.len() < uw * uh * 4 {
        return invalid("SHPS swizzled 8-bit block too small");
    }
    let dbw = (uw + 63) >> 6;
    let tbw = (w.next_power_of_two() + 63) >> 6;
    // PSMT8 pages are two TBW units wide; one page across covers widths up to 128.
    let pages_across = (tbw / 2).max(1);
    if tbw > 1 && tbw % 2 != 0 {
        return invalid("SHPS swizzled 8-bit width with odd TBW");
    }
    let rows32 = uh.div_ceil(32) * 32;
    let mut mem = vec![0u8; (dbw * 2048 * 4) * (rows32 / 32) + 8192 * 2];
    for y in 0..uh {
        for x in 0..uw {
            let a = addr32(x, y, dbw);
            mem[a..a + 4].copy_from_slice(&src[(y * uw + x) * 4..(y * uw + x) * 4 + 4]);
        }
    }
    let mut out = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            out[y * w + x] = mem.get(addr8(x, y, pages_across)).copied().unwrap_or(0);
        }
    }
    Ok(out)
}

impl Ssh {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 16 || &data[0..4] != MAGIC_SHPS {
            return invalid("not a SHPS (.ssh) file");
        }
        let count = u32_le(data, 8)? as usize;
        if 16 + count * 8 > data.len() {
            return invalid("SHPS directory runs past EOF");
        }
        let tag = [data[12], data[13], data[14], data[15]];
        let mut textures = Vec::with_capacity(count);
        for i in 0..count {
            let dir_o = 16 + i * 8;
            let name = String::from_utf8_lossy(&data[dir_o..dir_o + 4]).trim_end_matches('\0').to_string();
            let off = u32_le(data, dir_o + 4)? as usize;
            if off + 16 > data.len() {
                return invalid(format!("SHPS entry {name} header OOB"));
            }
            textures.push(Self::decode(data, off, name)?);
        }
        Ok(Self { tag, textures })
    }

    /// One image record (and its palette attachment) at `off` in `data`, outside a `SHPS`
    /// directory — the fonts keep their glyph sheet this way (`FUN_002a31d0` hands
    /// `font + u32 at +0x1c` to the same record loader `FUN_0027cd68` [M]).
    pub fn decode_record(data: &[u8], off: usize, name: String) -> Result<SshTexture> {
        if off + 16 > data.len() { return invalid(format!("SHPS record {name} header OOB")); }
        Self::decode(data, off, name)
    }

    fn decode(data: &[u8], off: usize, name: String) -> Result<SshTexture> {
        let ty = data[off];
        let format = ty & 0x7f;
        let Some(bpp) = bits_per_pixel(format).filter(|_| format < 0x20) else {
            return invalid(format!("SHPS unsupported image type {ty:#04x} for {name}"));
        };
        // The uploader masks both sizes to 12 bits (0x27f480 [M]).
        let w = (u16_le(data, off + 4)? & 0xfff) as usize;
        let h = (u16_le(data, off + 6)? & 0xfff) as usize;
        let flags = u32_le(data, off + 0xc)?;
        let mips = ((flags >> 28) + 1).min(7);
        let swizzled = flags & 0x2000 != 0;
        let pix = data_start(data, off)?;
        let n = w * h;
        let need = match bpp { 4 => n.div_ceil(2), 8 => n, 15 | 16 => n * 2, 24 => n * 3, _ => n * 4 };
        if pix + need > data.len() {
            return invalid(format!("SHPS {bpp}-bit {name} {w}x{h} pixels run past EOF"));
        }
        let px = &data[pix..pix + need];
        let (mut clut, mut clut_width, mut indices, mut palette) = (None, None, None, None);
        let rgba: Vec<u8> = match bpp {
            32 => px.chunks_exact(4).flat_map(|c| [c[0], c[1], c[2], ps2_alpha_to_8bit(c[3])]).collect(),
            // PSMCT24: alpha is TEXA.TA0 = 0x80 (0x27d340 [M]).
            24 => px.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
            15 | 16 => px.chunks_exact(2).flat_map(|c| ct16(u16::from_le_bytes([c[0], c[1]]))).collect(),
            8 | 4 => {
                let Some(c) = find_clut(data, off)? else {
                    return invalid(format!("SHPS {bpp}-bit {name} has no palette attachment"));
                };
                clut = Some(data[c]);
                clut_width = Some(u16_le(data, c + 4)? as u32);
                let pal = read_clut(data, c, if bpp == 8 { 256 } else { 16 })?;
                let idx: Vec<u8> = if bpp == 8 {
                    if swizzled { unswizzle8(px, w, h)? } else { px.to_vec() }
                } else {
                    if swizzled {
                        return invalid(format!("SHPS swizzled 4-bit {name} not supported"));
                    }
                    // PSMT4: low nibble is the first texel.
                    (0..n).map(|p| { let b = px[p / 2]; if p % 2 == 0 { b & 0x0f } else { b >> 4 } }).collect()
                };
                let rgba = idx.iter().flat_map(|&i| pal[i as usize]).collect();
                indices = Some(idx);
                palette = Some(pal);
                rgba
            }
            _ => unreachable!(),
        };
        Ok(SshTexture { name, format, bpp, width: w as u32, height: h as u32, mips, swizzled, clut, clut_width, indices, palette, rgba })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sheet with one record per `(name, record bytes)`, laid out back to back after the directory.
    fn sheet(recs: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(b"SHPS");
        d.extend_from_slice(&[0; 4]);
        d.extend_from_slice(&(recs.len() as u32).to_le_bytes());
        d.extend_from_slice(b"G354");
        let mut off = 16 + recs.len() * 8;
        for (name, r) in recs {
            d.extend_from_slice(*name);
            d.extend_from_slice(&(off as u32).to_le_bytes());
            off += r.len();
        }
        for (_, r) in recs {
            d.extend_from_slice(r);
        }
        let len = d.len() as u32;
        d[4..8].copy_from_slice(&len.to_le_bytes());
        d
    }

    /// A record header: type, next-attachment distance, size, flags.
    fn rec(ty: u8, next: u32, w: u16, h: u16, flags: u32, body: &[u8]) -> Vec<u8> {
        let mut r = (ty as u32 | next << 8).to_le_bytes().to_vec();
        r.extend_from_slice(&w.to_le_bytes());
        r.extend_from_slice(&h.to_le_bytes());
        r.extend_from_slice(&[0; 4]);
        r.extend_from_slice(&flags.to_le_bytes());
        r.extend_from_slice(body);
        r
    }

    fn px(t: &SshTexture, i: usize) -> [u8; 4] {
        t.rgba[i * 4..i * 4 + 4].try_into().unwrap()
    }

    #[test]
    fn truecolour_formats_and_alpha_scaling() {
        let c32 = rec(0x05, 0, 2, 1, 0, &[10, 20, 30, 0x80, 1, 2, 3, 0x40]);
        let c24 = rec(0x04, 0, 2, 1, 0, &[7, 8, 9, 4, 5, 6]);
        // 0x7fff = all channels 31, A bit clear: alpha still 0x80 via TEXA.
        let c16 = rec(0x06, 0, 1, 1, 0, &[0xff, 0x7f]);
        let s = Ssh::parse(&sheet(&[(b"c32a", c32), (b"c24a", c24), (b"c16a", c16)])).unwrap();
        assert_eq!(px(&s.textures[0], 0), [10, 20, 30, 255]);
        assert_eq!(px(&s.textures[0], 1), [1, 2, 3, 128]);
        assert_eq!(px(&s.textures[1], 1), [4, 5, 6, 255]);
        assert_eq!(px(&s.textures[2], 0), [248, 248, 248, 255]);
    }

    /// Index i of a 256-entry CLUT reads stored slot csm1_index(i); a decoder without the swap
    /// would return slot 8's colour for index 8.
    #[test]
    fn eight_bit_uses_csm1_palette_order_and_walks_the_chain() {
        let pixels = [8u8, 16, 0, 255];
        let mut pal = Vec::new();
        for k in 0..256u32 {
            pal.extend_from_slice(&[k as u8, 0, 0, 0x80]);
        }
        let img_len = 16 + pixels.len() as u32;
        // image → name attachment (0x70) → palette (0x21) declared 24 wide: the loader still reads 256.
        let name_att = rec(0x70, 16, 0, 0, 0, &[]);
        let clut = rec(0x21, 0, 24, 1, 0x2000, &pal);
        let mut r = rec(0x02, img_len, 2, 2, 0, &pixels);
        r.extend_from_slice(&name_att);
        r.extend_from_slice(&clut);
        let s = Ssh::parse(&sheet(&[(b"idx8", r)])).unwrap();
        let t = &s.textures[0];
        assert_eq!(t.clut, Some(0x21));
        assert_eq!(px(t, 0)[0], 16, "index 8 reads stored slot 16");
        assert_eq!(px(t, 1)[0], 8, "index 16 reads stored slot 8");
        assert_eq!(px(t, 2)[0], 0);
        assert_eq!(px(t, 3)[0], 255);
    }

    #[test]
    fn fifteen_bit_palette_and_four_bit_nibble_order() {
        // 4-bit 2x1: first texel = low nibble (index 1), second = high nibble (index 2).
        let mut pal = Vec::new();
        for k in 0..16u16 {
            pal.extend_from_slice(&(k & 0x1f).to_le_bytes());
        }
        let mut r = rec(0x01, 17, 2, 1, 0, &[0x21]);
        r.extend_from_slice(&rec(0x20, 0, 16, 1, 0, &pal));
        let s = Ssh::parse(&sheet(&[(b"idx4", r)])).unwrap();
        let t = &s.textures[0];
        assert_eq!(t.clut, Some(0x20));
        assert_eq!(px(t, 0), [8, 0, 0, 255]);
        assert_eq!(px(t, 1), [16, 0, 0, 255]);
    }

    #[test]
    fn eight_bit_without_palette_is_an_error() {
        let r = rec(0x02, 0, 2, 2, 0, &[0; 4]);
        assert!(Ssh::parse(&sheet(&[(b"bad8", r)])).is_err());
    }

    #[test]
    fn indirect_data_offset_flag_0x1000() {
        let mut r = rec(0x05, 0, 1, 1, 0x1000, &[0x18, 0, 0, 0, 0, 0, 0, 0]);
        r.extend_from_slice(&[1, 2, 3, 0x80]);
        let s = Ssh::parse(&sheet(&[(b"ind5", r)])).unwrap();
        assert_eq!(px(&s.textures[0], 0), [1, 2, 3, 255]);
    }

    /// Independent PSMT8-from-PSMCT32 formula (the widely used `unswizzle8` closed form for a
    /// texture whose transfer is one page or more wide): must agree with the GS-table emulation.
    fn formula8(x: usize, y: usize, w: usize) -> usize {
        let block = (y & !0xf) * w + (x & !0xf) * 2;
        let swap = (((y + 2) >> 2) & 1) * 4;
        let pos_y = (((y & !3) >> 1) + (y & 1)) & 7;
        let column = pos_y * w * 2 + ((x + swap) & 7) * 4;
        let byte = ((y >> 1) & 1) + ((x >> 2) & 2);
        block + column + byte
    }

    #[test]
    fn unswizzle8_matches_the_closed_form_and_is_not_identity() {
        for (w, h) in [(128usize, 128usize), (128, 32), (128, 64)] {
            let src: Vec<u8> = (0..w * h).map(|i| (i * 7 + i / 251) as u8).collect();
            let got = unswizzle8(&src, w, h).unwrap();
            let want: Vec<u8> = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).map(|(x, y)| src[formula8(x, y, w)]).collect();
            assert_eq!(got, want, "{w}x{h}");
            assert_ne!(got, src, "{w}x{h} unswizzle must move texels");
        }
    }

    #[test]
    fn rejects_bad_magic_and_unknown_type() {
        assert!(Ssh::parse(b"SHPX\0\0\0\0\0\0\0\0G354").is_err());
        let r = rec(0x7d, 0, 1, 1, 0, &[0; 4]);
        assert!(Ssh::parse(&sheet(&[(b"what", r)])).is_err());
    }
}
