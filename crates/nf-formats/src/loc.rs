//! Driving-mission localized string tables (`data\gfx\locale\IDL_*.loc` in `MISC.VIV`, magic
//! `LOCH`). Format: docs/formats/loc.md. Read the way DRIVING.ELF reads them [M — EE addresses,
//! decompile under `work/decomp/DRIVING.ELF/`]:
//!
//! ```text
//! LOCH: +0x04 u32 offset of the id table   +0x08 flags (bit 0: ids go through the id table)
//!       +0x0c u16 table count   +0x0e u16 table in use   +0x10 u32 × count: table offsets
//! id table (LOCI): +0x08 u32 n, +0x10 n × { u16 id, u16 index }, sorted by id
//! string table (LOCL): +0x0c u32 n, +0x10 n × u32 offset (from the table start) → NUL-terminated bytes
//! ```
//!
//! `FUN_00259360(loc, id)`: with flag bit 0, `id & 0xffff` is binary-searched in the id table
//! (`FUN_002592e0` → bsearch `FUN_002a72d8`, 4-byte entries, compare `FUN_002592c0` = u16
//! difference) and replaced by its index, or −1 when absent; then a negative or out-of-range
//! index gives NULL, else the string. `FUN_0024b1a0(id)` returns that string, or the ELF's
//! `"MISSING"` (0x3ac600 via PTR 0x33ba68) for NULL; with the debug word DAT_0033ba70 set it
//! prints the id with `"%d"` instead (DAT_0033ba70 is 0 at load and written only by the toggle
//! at 0x24b210-0x24b230 [M]).
//!
//! The strings are bytes: 0x80-0xff are the font's own glyphs (`data\loading\loading.sfn` has
//! glyphs up to 0xff); `text_lossy` reads them as Windows-1252 for display only.

use crate::buf::{invalid, Buf, Result};

pub const MAGIC: &[u8; 4] = b"LOCH";
/// `PTR_s_MISSING_0033ba68` → 0x3ac600 [M, D].
pub const MISSING: &[u8] = b"MISSING";

/// The file `FUN_0024afe0` loads: `FUN_0024b068(FUN_00249f98())` [M]. `FUN_00249f98` maps the
/// language word at 0x100988 (the hand-over block from the front end): 0..5 → 0..5, 8 → 6,
/// anything else → 7; `FUN_0024b068` then names the file (`data/gfx/locale/%s`, 0x3ac6b0).
/// `IDL_NA.loc` is on the disc but no DRIVING.ELF string names it [D].
pub fn file_for_language(lang_word: u32) -> &'static str {
    let k = match lang_word { 0..=5 => lang_word, 8 => 6, _ => 7 };
    ["IDL_BRT.loc", "IDL_FR.loc", "IDL_GER.loc", "IDL_SPA.loc", "IDL_ITA.loc", "IDL_DUT.loc", "IDL_SWE.loc", "IDL_ENG.loc"][k as usize]
}

/// Every `.loc` name the driving code can load, in `FUN_0024b068` case order 0..7 [M].
pub const LOADED_FILES: [&str; 8] = ["IDL_BRT.loc", "IDL_FR.loc", "IDL_GER.loc", "IDL_SPA.loc", "IDL_ITA.loc", "IDL_DUT.loc", "IDL_SWE.loc", "IDL_ENG.loc"];

#[derive(Clone, Debug)]
pub struct Loc {
    /// The id table (`id`, `index`) when header flag bit 0 is set.
    pub ids: Option<Vec<(u16, u16)>>,
    /// The string tables (one per +0x10 entry), each a list of byte strings.
    pub tables: Vec<Vec<Vec<u8>>>,
    /// +0x0e: the table `FUN_00259360` reads.
    pub current: u16,
}

impl Loc {
    pub fn parse(data: &[u8]) -> Result<Loc> {
        let b = Buf(data);
        if data.len() < 0x14 || &data[..4] != MAGIC { return invalid("not a string table (LOCH)"); }
        let flags = b.u8(8)?;
        let ids = if flags & 1 != 0 {
            let t = b.u32(4)? as usize;
            let n = b.u32(t + 8)? as usize;
            b.check(t + 0x10, n * 4, "LOCI entries")?;
            Some((0..n).map(|i| (b.u16(t + 0x10 + i * 4).unwrap(), b.u16(t + 0x12 + i * 4).unwrap())).collect::<Vec<_>>())
        } else { None };
        let count = b.u16(0xc)? as usize;
        let current = b.u16(0xe)?;
        let mut tables = Vec::with_capacity(count);
        for k in 0..count {
            let s = b.u32(0x10 + k * 4)? as usize;
            let n = b.u32(s + 0xc)? as usize;
            b.check(s + 0x10, n * 4, "LOCL offsets")?;
            let mut strings = Vec::with_capacity(n);
            for i in 0..n {
                let o = s + b.u32(s + 0x10 + i * 4)? as usize;
                b.check(o, 0, "LOCL string")?;
                let end = data[o..].iter().position(|&c| c == 0).ok_or_else(|| crate::buf::FormatError::Invalid(format!("LOC string {i} unterminated")))?;
                strings.push(data[o..o + end].to_vec());
            }
            tables.push(strings);
        }
        if current as usize >= tables.len() { return invalid("LOCH table in use out of range"); }
        Ok(Loc { ids, tables, current })
    }

    /// `FUN_00259360(loc, id)` [M]: the string, or `None` for the game's NULL.
    pub fn get(&self, id: i32) -> Option<&[u8]> {
        let mut index = id;
        if let Some(t) = &self.ids {
            // bsearch over u16 ids; `FUN_002592e0` returns the entry's +2 or −1.
            index = match t.binary_search_by(|e| e.0.cmp(&(id as u32 as u16))) { Ok(i) => t[i].1 as i32, Err(_) => -1 };
        }
        let tab = &self.tables[self.current as usize];
        if index < 0 || index as usize >= tab.len() { return None; }
        Some(&tab[index as usize])
    }

    /// `FUN_0024b1a0(id)` with the debug word clear [M]: the string or `"MISSING"`.
    pub fn label(&self, id: i32) -> &[u8] { self.get(id).unwrap_or(MISSING) }
}

/// Bytes as Windows-1252 text (display only; the game draws bytes through the font).
pub fn text_lossy(b: &[u8]) -> String {
    const HI: [char; 32] = ['€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž', '\u{8f}',
        '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}', 'ž', 'Ÿ'];
    b.iter().map(|&c| if (0x80..0xa0).contains(&c) { HI[c as usize - 0x80] } else { c as char }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// LOCH with an id table {7 → 1, 0x869f → 0} and two strings "zero", "one".
    fn fixture() -> Vec<u8> {
        let mut f = vec![0u8; 0x80];
        f[..4].copy_from_slice(b"LOCH");
        f[4..8].copy_from_slice(&0x14u32.to_le_bytes());
        f[8] = 1;
        f[0xc..0xe].copy_from_slice(&1u16.to_le_bytes());
        f[0x10..0x14].copy_from_slice(&0x30u32.to_le_bytes());
        f[0x14..0x18].copy_from_slice(b"LOCI");
        f[0x1c..0x20].copy_from_slice(&2u32.to_le_bytes());
        f[0x24..0x28].copy_from_slice(&[7, 0, 1, 0]);
        f[0x28..0x2c].copy_from_slice(&[0x9f, 0x86, 0, 0]);
        f[0x30..0x34].copy_from_slice(b"LOCL");
        f[0x3c..0x40].copy_from_slice(&2u32.to_le_bytes());
        f[0x40..0x44].copy_from_slice(&0x18u32.to_le_bytes());
        f[0x44..0x48].copy_from_slice(&0x1du32.to_le_bytes());
        f[0x48..0x4d].copy_from_slice(b"zero\0");
        f[0x4d..0x51].copy_from_slice(b"one\0");
        f
    }

    #[test]
    fn ids_go_through_the_id_table_low_16_bits() {
        let l = Loc::parse(&fixture()).unwrap();
        assert_eq!(l.get(7), Some(&b"one"[..]));
        assert_eq!(l.get(99999), Some(&b"zero"[..]), "99999 & 0xffff = 0x869f");
        assert_eq!(l.get(0), None, "an id not in the table");
        assert_eq!(l.label(1), MISSING, "1 is an index, not an id: it goes through the table too");
        assert_eq!(l.label(-1), MISSING);
    }

    #[test]
    fn language_word_picks_the_file() {
        assert_eq!(file_for_language(0), "IDL_BRT.loc");
        assert_eq!(file_for_language(3), "IDL_SPA.loc");
        assert_eq!(file_for_language(4), "IDL_ITA.loc");
        assert_eq!(file_for_language(8), "IDL_SWE.loc");
        assert_eq!(file_for_language(6), "IDL_ENG.loc");
        assert_eq!(file_for_language(7), "IDL_ENG.loc");
    }
}
