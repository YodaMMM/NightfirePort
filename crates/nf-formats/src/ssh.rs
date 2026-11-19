//! EA `SHPS` (`.ssh`) PS2 texture sheet reader (`DRIVING/*.VIV`: track sheets, car sheets, UI).
//!
//! Measured on `MIS01.VIV#73` (`data\track\paris_mis01.ssh`, 103 entries):
//! - Header: `"SHPS"` (4 B), `file_size` u32 LE, `count` u32 LE, `tag` (4 B, `"G354"`).
//! - Directory: `count` × 8-byte `{ [u8; 4] name, u32 offset }`.
//! - Entry at `offset`: `rec_type` u8, `size` u24 LE, `width` u16 LE, `height` u16 LE, 8 B flags/pos.
//!   - `0x01`: 4-bit indexed (`w*h/2` nibbles, followed by CLUT attachment).
//!   - `0x02`: 8-bit indexed (`w*h` bytes + optional mipmaps, followed by `0x21`/`0x23`/`0x24` CLUT).
//!   - `0x04`: 24-bit RGB (`w*h*3`).
//!   - `0x05`: 32-bit RGBA (`w*h*4`, PS2 0..=0x80 alpha).

use crate::buf::{invalid, FormatError, Result};

pub const MAGIC_SHPS: &[u8; 4] = b"SHPS";

/// One decoded texture from an EA `.ssh` (`SHPS`) sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Parsed EA `SHPS` texture sheet.
#[derive(Clone, Debug)]
pub struct Ssh {
    pub textures: Vec<SshTexture>,
}

fn u16_le(d: &[u8], o: usize) -> Result<u16> {
    let b = d.get(o..o + 2).ok_or_else(|| FormatError::Invalid("ssh u16 oob".into()))?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}

fn u24_le(d: &[u8], o: usize) -> Result<usize> {
    let b = d.get(o..o + 3).ok_or_else(|| FormatError::Invalid("ssh u24 oob".into()))?;
    Ok(b[0] as usize | ((b[1] as usize) << 8) | ((b[2] as usize) << 16))
}

fn u32_le(d: &[u8], o: usize) -> Result<u32> {
    let b = d.get(o..o + 4).ok_or_else(|| FormatError::Invalid("ssh u32 oob".into()))?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn ps2_alpha(a: u8) -> u8 {
    ((a as u16 * 255 + 64) / 128).min(255) as u8
}

/// PS2 GS CSM1 256-entry CLUT unswizzle: blocks 1 and 2 of each 32-entry group are swapped.
fn csm1_index(i: usize) -> usize {
    if (i & 0x18) == 0x08 {
        i + 8
    } else if (i & 0x18) == 0x10 {
        i - 8
    } else {
        i
    }
}

fn decode_clut(d: &[u8], off: usize) -> Option<Vec<[u8; 4]>> {
    if off + 16 > d.len() {
        return None;
    }
    let ty = d[off];
    if !matches!(ty, 0x20..=0x24) {
        return None;
    }
    let count = u16_le(d, off + 4).ok()? as usize;
    let pal_start = off + 16;
    let need = count * 4;
    if pal_start + need > d.len() {
        return None;
    }
    let mut out = vec![[255u8; 4]; count];
    for i in 0..count {
        let src_i = if count == 256 { csm1_index(i) } else { i };
        let o = pal_start + src_i * 4;
        out[i] = [d[o], d[o + 1], d[o + 2], ps2_alpha(d[o + 3])];
    }
    Some(out)
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
        let mut textures = Vec::with_capacity(count);
        for i in 0..count {
            let dir_o = 16 + i * 8;
            let name_bytes = &data[dir_o..dir_o + 4];
            let name = String::from_utf8_lossy(name_bytes)
                .trim_end_matches('\0')
                .to_string();
            let off = u32_le(data, dir_o + 4)? as usize;
            if off + 16 > data.len() {
                return invalid(format!("SHPS entry {name} header OOB"));
            }
            let rec_type = data[off];
            let rec_sz = u24_le(data, off + 1)?;
            let w = u16_le(data, off + 4)? as u32;
            let h = u16_le(data, off + 6)? as u32;
            let pix_off = off + 16;
            let n = (w as usize) * (h as usize);

            let rgba = match rec_type {
                0x05 => {
                    if pix_off + n * 4 > data.len() {
                        return invalid(format!("SHPS 32-bit {name} OOB"));
                    }
                    let mut out = Vec::with_capacity(n * 4);
                    for p in 0..n {
                        let o = pix_off + p * 4;
                        out.extend_from_slice(&[
                            data[o],
                            data[o + 1],
                            data[o + 2],
                            ps2_alpha(data[o + 3]),
                        ]);
                    }
                    out
                }
                0x04 => {
                    if pix_off + n * 3 > data.len() {
                        return invalid(format!("SHPS 24-bit {name} OOB"));
                    }
                    let mut out = Vec::with_capacity(n * 4);
                    for p in 0..n {
                        let o = pix_off + p * 3;
                        out.extend_from_slice(&[data[o], data[o + 1], data[o + 2], 255]);
                    }
                    out
                }
                0x02 => {
                    if pix_off + n > data.len() {
                        return invalid(format!("SHPS 8-bit {name} OOB"));
                    }
                    let clut_off = off + rec_sz;
                    let pal = decode_clut(data, clut_off).unwrap_or_else(|| {
                        (0..256).map(|k| [k as u8, k as u8, k as u8, 255]).collect()
                    });
                    let mut out = Vec::with_capacity(n * 4);
                    for p in 0..n {
                        let idx = data[pix_off + p] as usize;
                        let c = pal.get(idx).copied().unwrap_or([255, 0, 255, 255]);
                        out.extend_from_slice(&c);
                    }
                    out
                }
                0x01 => {
                    let bytes_needed = n.div_ceil(2);
                    if pix_off + bytes_needed > data.len() {
                        return invalid(format!("SHPS 4-bit {name} OOB"));
                    }
                    let clut_off = off + rec_sz;
                    let pal = decode_clut(data, clut_off).unwrap_or_else(|| {
                        (0..16).map(|k| [(k * 17) as u8, (k * 17) as u8, (k * 17) as u8, 255]).collect()
                    });
                    let mut out = Vec::with_capacity(n * 4);
                    for p in 0..n {
                        let b = data[pix_off + (p / 2)];
                        let idx = if p % 2 == 0 { b & 0x0f } else { b >> 4 } as usize;
                        let c = pal.get(idx).copied().unwrap_or([255, 0, 255, 255]);
                        out.extend_from_slice(&c);
                    }
                    out
                }
                other => {
                    return invalid(format!("SHPS unsupported entry type {other:#04x} for {name}"));
                }
            };

            textures.push(SshTexture {
                name,
                width: w,
                height: h,
                rgba,
            });
        }
        Ok(Self { textures })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synthetic_8bit_and_32bit_shps() {
        let mut d = vec![0u8; 0x100];
        d[0..4].copy_from_slice(b"SHPS");
        d[4..8].copy_from_slice(&(0x100u32).to_le_bytes());
        d[8..12].copy_from_slice(&2u32.to_le_bytes());
        d[12..16].copy_from_slice(b"G354");
        // Entry 0: "test" 2x2 32-bit RGBA at 0x20 (size 16+16=32)
        d[16..20].copy_from_slice(b"test");
        d[20..24].copy_from_slice(&0x20u32.to_le_bytes());
        d[0x20] = 0x05;
        d[0x21..0x24].copy_from_slice(&[32, 0, 0]);
        d[0x24..0x26].copy_from_slice(&2u16.to_le_bytes());
        d[0x26..0x28].copy_from_slice(&2u16.to_le_bytes());
        d[0x30..0x34].copy_from_slice(&[10, 20, 30, 0x80]);
        // Entry 1: "idx8" 2x2 8-bit at 0x40 (size 16+4=20), CLUT at 0x54
        d[24..28].copy_from_slice(b"idx8");
        d[28..32].copy_from_slice(&0x40u32.to_le_bytes());
        d[0x40] = 0x02;
        d[0x41..0x44].copy_from_slice(&[20, 0, 0]);
        d[0x44..0x46].copy_from_slice(&2u16.to_le_bytes());
        d[0x46..0x48].copy_from_slice(&2u16.to_le_bytes());
        d[0x50..0x54].copy_from_slice(&[0, 1, 0, 1]);
        // CLUT at 0x54: type 0x21, 2 entries
        d[0x54] = 0x21;
        d[0x58..0x5a].copy_from_slice(&2u16.to_le_bytes());
        d[0x64..0x68].copy_from_slice(&[255, 0, 0, 0x80]);
        d[0x68..0x6c].copy_from_slice(&[0, 255, 0, 0x80]);

        let ssh = Ssh::parse(&d).unwrap();
        assert_eq!(ssh.textures.len(), 2);
        assert_eq!(&ssh.textures[0].rgba[0..4], &[10, 20, 30, 255]);
        assert_eq!(&ssh.textures[1].rgba[0..8], &[255, 0, 0, 255, 0, 255, 0, 255]);
    }
}
