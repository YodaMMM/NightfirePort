//! EA "gallery" files (`.gal`, magic bytes `LAGG`) — the driving missions' HUD layout
//! (`data\gfx\data\hudNTSC.gal`) and the effects galleries in `MISC.VIV`
//! (`gall1.gal`, `ears_gall1.gal`, `sav_gall1.gal`). Format: docs/formats/hud-gal.md.
//!
//! The file is one tree of 16-byte records. Everything here is read the way DRIVING.ELF reads it
//! [M — addresses are EE, decompile under `work/decomp/DRIVING.ELF/`]:
//!
//! ```text
//! record = { u32 tag, u32 w1, u32 count, u32 w3 }            (little-endian words)
//!   tag   compared as a u32 (`'Name'` = 0x4e616d65). If w1 bit 0 is set the low 16 bits are an
//!         index and the lookup key is (tag & 0xffff0000) | 0x2020        [M 0x2d5cf8]
//!   node: base     = record + w3 * 16   when w1 bit 1 is set, else the absolute address w3
//!                                                                         [M 0x2d6940]
//!         children = base .. base + (w1 >> 5) * 16                        [M 0x2339e8 / 0x236b60]
//!         attrs    = the `count` records right after the children; w1 bit 3 = attrs sorted
//!                    by tag (binary search), else a linear scan           [M 0x2d5a10]
//!   attr: data     = record + w3        when w1 bit 1 is set (bytes, not records), else w3
//!         size     = w1 >> 8 bytes, `count` = element count               [M 0x233aa8, 0x2d5cf8]
//! ```
//!
//! The tree, as found on the disc [D, all 12 copies — tests/gal_disc.rs]:
//! root `GGAL` → attrs `fi`#0 (one blob, not read here), then `Name` + `tx`#n pairs (`tx` = an
//! embedded `SHPS` texture sheet; the loader binds `tx`#0.. in order until one is missing, at most
//! 15 [M `FUN_00236738` 0x236738]) → children `GANM` (an animation: attrs `Flgs`, `Name`, `NUMP`,
//! `LIFE`) → children = the elements (`gb` bitmap, `gt` text, `qd` quad in the HUD; `gp`, `gg`,
//! `gd`, `gs`, `g3` in the effects galleries), each with a `Name` and a `gk` key set.
//!
//! Lookups the HUD uses: animation by name `FUN_00236b60` (strcmp over the root's children's
//! `Name`), element by name `FUN_00236cc0` (strcmp over the animation's children, then the attr
//! `gk` | `(3 << 12 | 0)` = `gk`#0x3000 via `FUN_00236b48`, returns the pointer to that key data),
//! `FUN_00236f30(key, n)` = key + n * max(0x40, key[0x21] + 0x30) [M] — the key stride.
//!
//! Bitmap key fields, as `FUN_002386c8` (set style) and `FUN_00238d40` (draw) read them [M]:
//! +0x00 x, +0x04 y, +0x08 z, +0x10..0x20 a quaternion (x, y, z, w), +0x30 u0, +0x34 u1,
//! +0x38 v0, +0x3c v1, +0x40 width, +0x44 height, +0x48 colour (ARGB8888 — the setter halves it to
//! GS 0..0x80 and swaps R/B), +0x4c texture sheet index (`tx`#n), +0x54 / +0x58 x / y scale
//! × 0.001, +0x5c mirror bits (3 = four quads around the key, 1 = only an x-flipped quad, 2 = only
//! a y-flipped quad — docs/formats/hud-gal.md), signed repeat counts at +0x7b / +0x7c / +0x7d /
//! +0x7e (left, right, up, down). Fields outside this list
//! are kept in `Key::raw` but not named.

use crate::buf::{invalid, Buf, Result};

/// A tag as the game compares it: the u32 whose big-endian spelling is the 4 characters.
pub const fn tag(s: &[u8; 4]) -> u32 { u32::from_be_bytes(*s) }

pub const GGAL: u32 = tag(b"GGAL");
pub const GANM: u32 = tag(b"GANM");
pub const NAME: u32 = tag(b"Name");
pub const FLGS: u32 = tag(b"Flgs");
pub const NUMP: u32 = tag(b"NUMP");
pub const LIFE: u32 = tag(b"LIFE");
/// Indexed tags (the key form, low half `0x2020`).
pub const TX: u32 = tag(b"tx  ");
pub const FI: u32 = tag(b"fi  ");
pub const GK: u32 = tag(b"gk  ");
pub const GB: u32 = tag(b"gb  ");
pub const GT: u32 = tag(b"gt  ");
pub const QD: u32 = tag(b"qd  ");
/// The key set the HUD asks for: `FUN_00236b48(gal, 3, 0)` = 3 << 12 | 0 [M 0x236b48].
pub const HUD_KEYSET: u16 = 0x3000;

/// One raw 16-byte record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record { pub at: usize, pub tag: u32, pub w1: u32, pub count: u32, pub w3: u32 }

impl Record {
    fn read(b: Buf, at: usize) -> Result<Record> {
        Ok(Record { at, tag: b.u32(at)?, w1: b.u32(at + 4)?, count: b.u32(at + 8)?, w3: b.u32(at + 12)? })
    }
    /// The lookup key: an indexed record compares as `(tag & 0xffff0000) | 0x2020` [M 0x2d5cf8].
    pub fn key(&self) -> u32 { if self.indexed() { self.tag & 0xffff_0000 | 0x2020 } else { self.tag } }
    pub fn indexed(&self) -> bool { self.w1 & 1 != 0 }
    pub fn index(&self) -> Option<u16> { self.indexed().then_some(self.tag as u16) }
    pub fn relative(&self) -> bool { self.w1 & 2 != 0 }
    /// Attrs sorted for binary search [M 0x2d5a10 `& 8`].
    pub fn sorted(&self) -> bool { self.w1 & 8 != 0 }
    pub fn children(&self) -> usize { (self.w1 >> 5) as usize }
    /// Attribute payload size in bytes [M 0x2d5cf8 `w1 >> 8`].
    pub fn size(&self) -> usize { (self.w1 >> 8) as usize }
}

/// A parsed node: its record, child nodes and attributes.
#[derive(Clone, Debug)]
pub struct Node { pub rec: Record, pub children: Vec<Node>, pub attrs: Vec<Attr> }

/// An attribute: its record and payload byte range in the file.
#[derive(Clone, Debug)]
pub struct Attr { pub rec: Record, pub data: std::ops::Range<usize> }

fn read_node(b: Buf, at: usize, depth: usize) -> Result<Node> {
    if depth > 8 { return invalid("gal: tree deeper than 8"); }
    let rec = Record::read(b, at)?;
    let n_child = rec.children();
    let n_attr = rec.count as usize;
    if n_child + n_attr == 0 { return Ok(Node { rec, children: vec![], attrs: vec![] }); }
    if !rec.relative() { return invalid(format!("gal: node at {at:#x} uses an absolute pointer")); }
    let base = at.checked_add(rec.w3 as usize * 16).filter(|&x| x <= b.len())
        .ok_or_else(|| crate::buf::FormatError::Invalid(format!("gal: node at {at:#x} base past EOF")))?;
    b.check(base, (n_child + n_attr) * 16, "gal records")?;
    let mut children = Vec::with_capacity(n_child);
    for i in 0..n_child { children.push(read_node(b, base + i * 16, depth + 1)?); }
    let mut attrs = Vec::with_capacity(n_attr);
    for i in 0..n_attr {
        let r = Record::read(b, base + (n_child + i) * 16)?;
        let size = r.size();
        let data = if size == 0 {
            0..0
        } else {
            if !r.relative() { return invalid(format!("gal: attr at {:#x} uses an absolute pointer", r.at)); }
            let s = r.at + r.w3 as usize;
            b.check(s, size, "gal attr data")?;
            s..s + size
        };
        attrs.push(Attr { rec: r, data });
    }
    Ok(Node { rec, children, attrs })
}

impl Node {
    /// The first attr whose lookup key is `key` and (for indexed tags) whose index is `index`.
    pub fn attr(&self, key: u32, index: Option<u16>) -> Option<&Attr> {
        self.attrs.iter().find(|a| a.rec.key() == key && (index.is_none() || a.rec.index() == index))
    }
    /// The `Name` attr as text (NUL-terminated).
    pub fn name<'a>(&self, file: &'a [u8]) -> Option<&'a str> {
        let a = self.attr(NAME, None)?;
        cstr(&file[a.data.clone()])
    }
}

fn cstr(b: &[u8]) -> Option<&str> {
    let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    std::str::from_utf8(&b[..n]).ok()
}

/// One key of an element (a whole `stride`-byte record).
#[derive(Clone, Debug, PartialEq)]
pub struct Key { pub raw: Vec<u8> }

impl Key {
    fn f(&self, o: usize) -> f32 { f32::from_le_bytes(self.raw[o..o + 4].try_into().unwrap()) }
    fn u(&self, o: usize) -> u32 { u32::from_le_bytes(self.raw[o..o + 4].try_into().unwrap()) }
    /// +0x00 x, +0x04 y, +0x08 z [M `FUN_002386c8` copies [0..3]; `FUN_002391e8` draws at (x, y)].
    pub fn pos(&self) -> [f32; 3] { [self.f(0), self.f(4), self.f(8)] }
    /// +0x10..0x20 quaternion (x, y, z, w) [M `FUN_002386c8` copies 16 bytes; reading it as a
    /// quaternion is [D]: the corner pieces `fbw2..4` hold z/w pairs of 90/−90/180°].
    pub fn quat(&self) -> [f32; 4] { [self.f(0x10), self.f(0x14), self.f(0x18), self.f(0x1c)] }
    /// The key stride byte: stride = max(0x40, key[0x21] + 0x30) [M `FUN_00236f30`].
    pub fn stride(&self) -> usize { (self.raw[0x21] as usize + 0x30).max(0x40) }
}

/// Bitmap-element fields (`gb`; keys ≥ 0x80 bytes) [M `FUN_00238d40` / `FUN_002386c8`].
impl Key {
    /// u0, u1, v0, v1 (+0x30..0x40).
    pub fn uv(&self) -> [f32; 4] { [self.f(0x30), self.f(0x34), self.f(0x38), self.f(0x3c)] }
    /// width, height in screen pixels before scale (+0x40, +0x44).
    pub fn size(&self) -> [f32; 2] { [self.f(0x40), self.f(0x44)] }
    /// ARGB8888 colour (+0x48).
    pub fn colour_argb(&self) -> u32 { self.u(0x48) }
    /// The colour as the setter writes it to the draw state: each channel halved to the GS
    /// 0..0x80 range, R into the low byte [M `FUN_002386c8` bit shuffle].
    pub fn colour_gs(&self) -> u32 { argb_to_gs(self.colour_argb()) }
    /// Texture sheet index, i.e. `tx`#n (+0x4c) [M `FUN_002386c8` → gal+0x10 table].
    pub fn sheet(&self) -> u32 { self.u(0x4c) }
    /// x / y scale (+0x54 / +0x58, integer thousandths) [M `FUN_002386c8` × 0.001].
    pub fn scale(&self) -> [f32; 2] { [self.u(0x54) as i32 as f32 * 0.001, self.u(0x58) as i32 as f32 * 0.001] }
    /// Mirror bits (+0x5c) [M `FUN_00238d40` `pf[0xb] & 3 == 3`, `& 1`, `& 2`].
    pub fn mirror(&self) -> u32 { self.u(0x5c) }
    /// Signed repeat counts (+0x7b left, +0x7c right, +0x7d up, +0x7e down) [M `FUN_00238d40`].
    pub fn repeats(&self) -> [i8; 4] { [self.raw[0x7b] as i8, self.raw[0x7c] as i8, self.raw[0x7d] as i8, self.raw[0x7e] as i8] }
}

/// ARGB8888 → the GS word `FUN_002386c8` stores: `(c & 0xfe) << 15 | (c & 0xfe00) >> 1 |
/// (c >> 17) & 0x7f | (c & 0xfe000000) >> 1` [M].
pub fn argb_to_gs(c: u32) -> u32 { (c & 0xfe) << 15 | (c & 0xfe00) >> 1 | (c >> 17) & 0x7f | (c & 0xfe00_0000) >> 1 }

/// One element of an animation.
#[derive(Clone, Debug)]
pub struct Element {
    /// The 2-character kind (`gb`, `gt`, `qd`, ...): the key form of the record tag.
    pub kind: u32,
    pub index: u16,
    pub name: String,
    /// Key sets by id (the `gk`#id attrs), each a list of keys.
    pub keysets: Vec<(u16, Vec<Key>)>,
}

impl Element {
    pub fn kind_str(&self) -> String { String::from_utf8_lossy(&self.kind.to_be_bytes()[..2]).into_owned() }
    /// The key set `FUN_00236cc0` returns for the HUD (`gk`#0x3000), first key.
    pub fn hud_key(&self) -> Option<&Key> {
        self.keysets.iter().find(|(id, _)| *id == HUD_KEYSET).and_then(|(_, k)| k.first())
    }
}

/// One animation (`GANM`).
#[derive(Clone, Debug)]
pub struct Anim { pub name: String, pub flags: u32, pub nump: u32, pub life: u32, pub elements: Vec<Element> }

impl Anim {
    /// `FUN_00236cc0`: the first element whose name matches exactly (strcmp) [M].
    pub fn element(&self, name: &str) -> Option<&Element> { self.elements.iter().find(|e| e.name == name) }
}

/// A parsed gallery.
#[derive(Clone, Debug)]
pub struct Gal {
    /// The root's `tx`#n payloads (embedded `SHPS` sheets), by index, with the `Name` attr that
    /// precedes each in the file (the build-time source path; pairing by file order is [D]).
    pub sheets: Vec<(u16, String, std::ops::Range<usize>)>,
    /// The root's `fi` payload, if any (not decoded).
    pub fi: Option<std::ops::Range<usize>>,
    pub anims: Vec<Anim>,
    pub root: Node,
}

impl Gal {
    pub fn parse(data: &[u8]) -> Result<Gal> {
        let b = Buf(data);
        if data.len() < 16 || &data[..4] != b"LAGG" { return invalid("not a gallery (LAGG)"); }
        let root = read_node(b, 0, 0)?;
        if root.rec.tag != GGAL { return invalid("gal: root tag is not GGAL"); }
        let mut sheets = Vec::new();
        let mut fi = None;
        let mut last_name = String::new();
        for a in &root.attrs {
            match a.rec.key() {
                NAME => last_name = cstr(&data[a.data.clone()]).unwrap_or("").to_owned(),
                TX => sheets.push((a.rec.index().unwrap_or(0), std::mem::take(&mut last_name), a.data.clone())),
                FI => fi = Some(a.data.clone()),
                _ => {}
            }
        }
        let mut anims = Vec::with_capacity(root.children.len());
        for an in &root.children {
            if an.rec.tag != GANM { return invalid(format!("gal: root child at {:#x} is not GANM", an.rec.at)); }
            let word = |t: u32| -> Result<u32> {
                match an.attr(t, None) { Some(a) if a.data.len() >= 4 => b.u32(a.data.start), _ => Ok(0) }
            };
            let mut elements = Vec::with_capacity(an.children.len());
            for el in &an.children {
                let mut keysets = Vec::new();
                for a in el.attrs.iter().filter(|a| a.rec.key() == GK) {
                    let mut keys = Vec::new();
                    let n = a.rec.count as usize;
                    if n > 0 && !a.data.is_empty() {
                        let first = &data[a.data.clone()];
                        if first.len() < 0x40 { return invalid(format!("gal: key at {:#x} shorter than 0x40", a.rec.at)); }
                        let stride = (first[0x21] as usize + 0x30).max(0x40);
                        if stride * n != a.data.len() {
                            return invalid(format!("gal: key set at {:#x}: {n} × {stride:#x} != {:#x}", a.rec.at, a.data.len()));
                        }
                        for k in 0..n {
                            let s = a.data.start + k * stride;
                            keys.push(Key { raw: data[s..s + stride].to_vec() });
                        }
                    }
                    keysets.push((a.rec.index().unwrap_or(0xffff), keys));
                }
                elements.push(Element {
                    kind: el.rec.key(),
                    index: el.rec.index().unwrap_or(0),
                    name: el.name(data).unwrap_or("").to_owned(),
                    keysets,
                });
            }
            anims.push(Anim {
                name: an.name(data).unwrap_or("").to_owned(),
                flags: word(FLGS)?, nump: word(NUMP)?, life: word(LIFE)?, elements,
            });
        }
        Ok(Gal { sheets, fi, anims, root })
    }

    /// `FUN_00236b60`: the first animation whose name matches exactly [M].
    pub fn anim(&self, name: &str) -> Option<&Anim> { self.anims.iter().find(|a| a.name == name) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny gallery built by hand: root (2 attrs: Name + tx#0) → 1 GANM (Name + LIFE) → 1 `gb`
    /// element (Name + gk#0x3000 with one 0x80-byte key). Offsets follow the record semantics in
    /// the module header.
    fn fixture() -> Vec<u8> {
        let mut f = vec![0u8; 0x400];
        let put = |f: &mut Vec<u8>, at: usize, w: [u32; 4]| {
            for (i, x) in w.iter().enumerate() { f[at + i * 4..at + i * 4 + 4].copy_from_slice(&x.to_le_bytes()); }
        };
        // root @0: 1 child, 2 attrs, base = 0 + 1*16 = 0x10. tag bytes "LAGG" = GGAL as LE u32.
        put(&mut f, 0x00, [GGAL, (1 << 5) | 2, 2, 1]);
        // child GANM @0x10: 1 child, 2 attrs, base = 0x10 + 3*16 = 0x40
        put(&mut f, 0x10, [GANM, (1 << 5) | 2, 2, 3]);
        // root attrs @0x20 Name → 0x200 ("sheet"), @0x30 tx#0 → 0x210 (8 bytes)
        put(&mut f, 0x20, [NAME, (6 << 8) | 2, 1, 0x200 - 0x20]);
        put(&mut f, 0x30, [0x7478_0000, (8 << 8) | 3, 1, 0x210 - 0x30]);
        // GANM attrs @0x50 Name "hud" → 0x220, @0x60 LIFE → 0x230
        put(&mut f, 0x50, [NAME, (4 << 8) | 2, 1, 0x220 - 0x50]);
        // GANM child @0x40 gb#1: 0 children, 2 attrs, base = 0x40 + 4*16 = 0x80
        put(&mut f, 0x40, [0x6762_0001, 0x13, 2, 4]);
        put(&mut f, 0x60, [LIFE, (4 << 8) | 2, 1, 0x230 - 0x60]);
        // element attrs @0x80 gk#0x3000 → 0x280 (0x80 bytes), @0x90 Name "dot" → 0x240
        put(&mut f, 0x80, [0x676b_3000, (0x80 << 8) | 3, 1, 0x280 - 0x80]);
        put(&mut f, 0x90, [NAME, (4 << 8) | 2, 1, 0x240 - 0x90]);
        f[0x200..0x206].copy_from_slice(b"sheet\0");
        f[0x210..0x214].copy_from_slice(b"SHPS");
        f[0x220..0x224].copy_from_slice(b"hud\0");
        f[0x230..0x234].copy_from_slice(&30u32.to_le_bytes());
        f[0x240..0x244].copy_from_slice(b"dot\0");
        let k = 0x280;
        f[k..k + 4].copy_from_slice(&175.0f32.to_le_bytes());
        f[k + 4..k + 8].copy_from_slice(&140.0f32.to_le_bytes());
        f[k + 0x21] = 0x50;
        f[k + 0x40..k + 0x44].copy_from_slice(&20.0f32.to_le_bytes());
        f[k + 0x48..k + 0x4c].copy_from_slice(&0xff80_4020u32.to_le_bytes());
        f[k + 0x54..k + 0x58].copy_from_slice(&1300u32.to_le_bytes());
        f
    }

    #[test]
    fn the_record_semantics_walk_the_fixture() {
        let f = fixture();
        let g = Gal::parse(&f).unwrap();
        assert_eq!(g.sheets.len(), 1);
        assert_eq!(g.sheets[0].1, "sheet", "Name pairs with the tx that follows it");
        assert_eq!(&f[g.sheets[0].2.clone()][..4], b"SHPS");
        let a = g.anim("hud").expect("anim by name");
        assert_eq!(a.life, 30);
        let e = a.element("dot").expect("element by name");
        assert_eq!(e.kind, GB);
        assert_eq!(e.index, 1);
        let k = e.hud_key().unwrap();
        assert_eq!(k.pos(), [175.0, 140.0, 0.0]);
        assert_eq!(k.stride(), 0x80);
        assert_eq!(k.size(), [20.0, 0.0]);
        assert_eq!(k.scale()[0], 1300.0f32 * 0.001, "(float)int × 0.001 in single precision");
        // Sister: a child count taken from w1 >> 8 (the leaf size rule) instead of >> 5 reads 0
        // children for the root (0x22 >> 8 = 0) — the element would not be found.
        assert_eq!((0x22u32 >> 8) as usize, 0);
        assert_eq!(g.root.rec.children(), 1);
    }

    /// The colour shuffle of `FUN_002386c8`: ARGB8888 → GS (R low byte, every channel / 2).
    #[test]
    fn argb_halves_into_the_gs_word() {
        // cream text colour 0xfffadbb3 (bytes b3 db fa ff): R=fa G=db B=b3 A=ff
        assert_eq!(argb_to_gs(0xfffa_dbb3), 0x7f59_6d7d);
        assert_eq!(argb_to_gs(0xffff_ffff), 0x7f7f_7f7f);
        // Sister: no swap would put B in the low byte.
        assert_ne!(argb_to_gs(0xff00_00ff) & 0xff, 0x7f, "blue must not land in the R byte");
    }

    #[test]
    fn a_bad_magic_or_a_truncated_tree_is_rejected() {
        let mut f = fixture();
        f[0] = b'X';
        assert!(Gal::parse(&f).is_err());
        let f = fixture();
        assert!(Gal::parse(&f[..0x50]).is_err());
    }
}
