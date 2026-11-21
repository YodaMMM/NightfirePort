//! Disc test for `nf_formats::sfn` (fonts, docs/formats/sfn.md) and `nf_formats::loc` (string
//! tables, docs/formats/loc.md): every `.sfn` and `.loc` entry in every `DRIVING/*.VIV` decodes,
//! and matches the goldens written by the independent walker `tools/crosscheck/sfn_loc_census.py`
//! (own RefPack decoder and field walk). Then the measure rules are checked against the game's own
//! code: DRIVING.ELF's `FUN_00237ca8` (width) and `FUN_00237ce0` (height) run in the EE
//! interpreter (`nf_formats::ee`) over the disc font and every string of every loaded language, at
//! the scales the HUD sets; `Font::text_width` / `text_height` must give the same bits.
//!
//! Opt-in via NF_ISO (the full image; a slim image whose `/DRIVING/` extents are zero skips).

use nf_formats::ee::Machine;
use nf_formats::golden::diff_tsv;
use nf_formats::iso9660::IsoImage;
use nf_formats::loc::{Loc, LOADED_FILES};
use nf_formats::sfn::Font;
use nf_formats::sha1::{hex, sha1};

fn iso() -> Option<IsoImage> { IsoImage::open(std::env::var_os("NF_ISO")?).ok() }

/// (`/DRIVING/<VIV>/<entry name>`, decompressed bytes) for every entry whose name ends in `ext`.
fn entries(iso: &IsoImage, ext: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let mut vivs: Vec<_> = iso.entries().iter().filter(|e| e.path.starts_with("/DRIVING/") && e.path.ends_with(".VIV")).cloned().collect();
    vivs.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = Vec::new();
    for e in &vivs {
        let g = nf_formats::disc::read_bigf(iso, e).ok()?;
        for x in &g.entries {
            if !x.name.to_ascii_lowercase().ends_with(ext) { continue; }
            let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
            if raw.iter().take(64).all(|&b| b == 0) { return None; }
            let data = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
            out.push((format!("{}/{}", e.path, x.name), data));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Some(out)
}

fn sfn_row(key: &str, d: &[u8], f: &Font) -> String {
    let adv: i32 = f.glyphs.iter().map(|g| g.advance as i32).sum();
    let kerned = f.glyphs.iter().filter(|g| g.kerned != 0).count();
    let ksum: i32 = f.kerns.iter().map(|k| k.kern as i32).sum();
    let gt = &d[f.glyph_off as usize..f.glyph_off as usize + f.glyphs.len() * 12];
    format!("{key}\t{}\t{}\t{:#x}\t{}\t{:#x}\t{}\t{}\t{}\t{}\t{adv}\t{kerned}\t{}\t{ksum}\t{:#04x}\t{}\t{}\t{}\n",
        d.len(), hex(&sha1(d)), f.version, f.glyphs.len(), f.flags, f.line[0], f.line[1], f.glyphs[0].ch,
        f.glyphs.last().unwrap().ch, f.kerns.len(), f.sheet.format, f.sheet.width, f.sheet.height, hex(&sha1(gt)))
}

fn loc_row(key: &str, d: &[u8], l: &Loc) -> String {
    let t = &l.tables[l.current as usize];
    let ids = l.ids.as_ref().unwrap();
    let resolved = ids.iter().filter(|&&(id, _)| l.get(id as i32).is_some()).count();
    let joined = t.join(&0u8);
    format!("{key}\t{}\t{}\t{}\t{}\t{}\t{}\t{resolved}\t{}\t{}\t{}\n", d.len(), hex(&sha1(d)), d[8], l.tables.len(), ids.len(), t.len(),
        t.iter().map(|s| s.len()).sum::<usize>(), t.iter().filter(|s| s.contains(&b'%')).count(), hex(&sha1(&joined)))
}

#[test]
fn every_driving_font_and_string_table_matches_the_independent_census() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let Some(fonts) = entries(&iso, ".sfn") else { eprintln!("skip: slim image"); return };
    let locs = entries(&iso, ".loc").unwrap();
    assert_eq!((fonts.len(), locs.len()), (11, 9), "8 loading.sfn copies + error + debug + MISC loading; 9 IDL_*.loc [D]");
    let mut a = String::from("entry\tbytes\tsha1\tversion\tglyphs\tflags\tline_a\tline_b\tfirst\tlast\tadvance_sum\tkerned\tkerns\tkern_sum\tsheet_type\tsheet_w\tsheet_h\tglyph_sha1\n");
    for (k, d) in &fonts {
        let f = Font::parse(d).unwrap_or_else(|e| panic!("{k}: {e}"));
        assert_eq!(f.sheet.rgba.len(), (f.sheet.width * f.sheet.height * 4) as usize);
        a += &sfn_row(k, d, &f);
    }
    let golden = include_str!("../golden/sfn_census.tsv");
    let diff = diff_tsv(golden, &a);
    assert!(diff.is_empty(), "{} differences:\n{}", diff.len(), diff.join("\n"));
    let tampered = a.replacen("\t2123\t84\t753\t-754\t", "\t2123\t84\t753\t-753\t", 1);
    assert!(!diff_tsv(golden, &tampered).is_empty(), "sister: one kern off by one is caught");

    let mut a = String::from("entry\tbytes\tsha1\tflags\ttables\tids\tstrings\tids_resolved\tstring_bytes\twith_percent\tstrings_sha1\n");
    for (k, d) in &locs {
        let l = Loc::parse(d).unwrap_or_else(|e| panic!("{k}: {e}"));
        a += &loc_row(k, d, &l);
    }
    let golden = include_str!("../golden/loc_census.tsv");
    let diff = diff_tsv(golden, &a);
    assert!(diff.is_empty(), "{} differences:\n{}", diff.len(), diff.join("\n"));
    // Every file FUN_0024b068 can load is on the disc [M, D].
    for n in LOADED_FILES { assert!(locs.iter().any(|(k, _)| k.ends_with(&format!("\\{n}"))), "{n}"); }
}

const GAL: u32 = 0x0200_0000;
const FONT: u32 = 0x0201_0000;
const TEXT: u32 = 0x0220_0000;

/// DRIVING.ELF with the font mapped at FONT and a gallery stub at GAL whose +0xa8 points at it.
fn machine(iso: &IsoImage, font: &[u8]) -> Machine {
    let elf = iso.read_file(iso.find("/DRIVING.ELF").expect("DRIVING.ELF")).unwrap();
    let mut m = Machine::from_elf(&elf).unwrap();
    // $gp of our DRIVING.ELF (docs/formats/driving-sym.md): the 1e7 / −1e7 box seeds live at gp − 0x7ff0.
    m.set_gpr(28, 0x003c_fdf0);
    let mut gal = vec![0u8; 0x100];
    gal[0xa8..0xac].copy_from_slice(&FONT.to_le_bytes());
    m.map(GAL, gal);
    // Zeroed memory before the font: the direct glyph index of a byte below 0x20 (a newline)
    // reads there; zero never equals the byte, as the port assumes [S].
    let mut f = vec![0u8; 0x1000];
    f.extend_from_slice(font);
    m.map(FONT - 0x1000, f);
    m.map(TEXT, vec![0; 0x400]);
    m
}

/// `FUN_00237ca8` / `FUN_00237ce0` run on the EE: (width, height) bits.
fn game_measure(m: &mut Machine, text: &[u8], scale: [f32; 2]) -> (u32, u32) {
    let mut s = text.to_vec();
    s.push(0);
    m.poke(TEXT, &s).unwrap();
    m.poke(FONT + 0x38, &scale[0].to_le_bytes()).unwrap();
    m.poke(FONT + 0x3c, &scale[1].to_le_bytes()).unwrap();
    m.steps = 0;
    m.call(0x0023_7ca8, GAL, TEXT, 2_000_000).unwrap();
    let w = m.fpr(0).to_bits();
    m.steps = 0;
    m.call(0x0023_7ce0, GAL, TEXT, 2_000_000).unwrap();
    (w, m.fpr(0).to_bits())
}

/// The HUD's measure scales: 1.0 (ammo), 1.1 (objective banner), 1.2 (message line), 1.4 (clock,
/// pickup line); 0.75 (a widescreen draw leaves x × 0.75 behind) with y = 448/480.
const SCALES: [[f32; 2]; 5] = [[1.0, 1.0], [f32::from_bits(0x3f8c_cccd); 2], [f32::from_bits(0x3f99_999a); 2], [f32::from_bits(0x3fb3_3333); 2], [0.75, f32::from_bits(0x3f6e_eeef)]];

#[test]
fn text_width_and_height_match_the_game_code_on_every_string() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let Some(fonts) = entries(&iso, ".sfn") else { eprintln!("skip: slim image"); return };
    let locs = entries(&iso, ".loc").unwrap();
    let mut checked = 0usize;
    let mut unkerned_differs = 0usize;
    for name in ["loading.sfn", "error.sfn", "debug.sfn"] {
        let (_, fd) = fonts.iter().find(|(k, _)| k.starts_with("/DRIVING/MISC.VIV/") && k.ends_with(name)).unwrap();
        let font = Font::parse(fd).unwrap();
        let mut m = machine(&iso, fd);
        let mut unkerned = font.clone();
        unkerned.kerns.clear();
        for (k, ld) in &locs {
            let loc = Loc::parse(ld).unwrap();
            let eng = k.ends_with("IDL_ENG.loc");
            // The HUD font: English at every HUD scale, the other languages at 1.2 and 1.4; the
            // other fonts: every English string at 1.0.
            if name != "loading.sfn" && !eng { continue; }
            let scales: &[[f32; 2]] = if name != "loading.sfn" { &SCALES[..1] } else if eng { &SCALES } else { &SCALES[2..4] };
            for s in &loc.tables[loc.current as usize] {
                if s.len() > 0x3f0 { continue; }
                for &sc in scales {
                    let (w, h) = game_measure(&mut m, s, sc);
                    let e = font.measure(s, sc);
                    assert_eq!((e.w.to_bits(), e.h.to_bits()), (w, h), "{name} {k} {:?} at {sc:?}", nf_formats::loc::text_lossy(s));
                    checked += 1;
                    if eng && sc == [1.0, 1.0] && unkerned.measure(s, sc).w.to_bits() != w { unkerned_differs += 1; }
                }
            }
        }
        // Strings the HUD builds itself (clock, ammo) and edge cases: newline, unknown byte, empty.
        for s in [&b"00:00"[..], b" 1:15", b"0", b"250", b"A\nBC", b"\x01\x7f", b"", b"s"] {
            for &sc in &SCALES {
                let (w, h) = game_measure(&mut m, s, sc);
                let e = font.measure(s, sc);
                assert_eq!((e.w.to_bits(), e.h.to_bits()), (w, h), "{name} {s:?} at {sc:?}");
                checked += 1;
            }
        }
    }
    assert!(checked > 1530 * 5 + 8 * 1530 * 2, "{checked}");
    assert!(unkerned_differs > 0, "sister: a measure without kerning disagrees with the game on some strings");
}

const TABLE: u32 = 0x0230_0000;
const PACKET: u32 = 0x0231_0000;
const PACKET_DATA: u32 = 0x0232_0000;

/// `FUN_00258100(x, y, font, text, 0)` run on the EE with the font callback table holding only
/// the glyph sprite writer `FUN_002a1e50` (entry 0 of 0x348be0; begin / end entries left out) and
/// the GS offsets at 0 / the UV offsets at 8: the sprites it writes, as (x0, y0, x1, y1, u0, v0, u1, v1).
fn game_layout(m: &mut Machine, x: f32, y: f32, text: &[u8], scale: [f32; 2]) -> Vec<[i32; 8]> {
    let mut s = text.to_vec();
    s.push(0);
    m.poke(TEXT, &s).unwrap();
    m.poke(FONT + 0x38, &scale[0].to_le_bytes()).unwrap();
    m.poke(FONT + 0x3c, &scale[1].to_le_bytes()).unwrap();
    m.poke(PACKET + 8, &0u32.to_le_bytes()).unwrap();
    m.set_fpr(12, x);
    m.set_fpr(13, y);
    m.set_gpr(6, 0);
    m.steps = 0;
    m.call(0x0025_8100, FONT, TEXT, 4_000_000).unwrap();
    let n = u32::from_le_bytes(m.read(PACKET + 8, 4).unwrap().try_into().unwrap()) as usize;
    let q = |i: usize| u64::from_le_bytes(m.read(PACKET_DATA + 8 * i as u32, 8).unwrap().try_into().unwrap());
    assert_eq!(n % 8, 0);
    (0..n / 8).map(|g| {
        let (uv0, xy0, uv1, xy1) = (q(8 * g), q(8 * g + 2), q(8 * g + 4), q(8 * g + 6));
        assert_eq!((q(8 * g + 1), q(8 * g + 3), q(8 * g + 5), q(8 * g + 7)), (3, 0xd, 3, 5), "UV, XYZ2, UV, XYZ2 register order");
        let c = |v: u64, sh: u32| (v >> sh) as u16 as i16 as i32;
        [c(xy0, 0) >> 4, c(xy0, 16) >> 4, c(xy1, 0) >> 4, c(xy1, 16) >> 4, c(uv0, 0), c(uv0, 16), c(uv1, 0), c(uv1, 16)]
    }).collect()
}

#[test]
fn glyph_quads_match_the_game_layout_and_sprite_writer() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let Some(fonts) = entries(&iso, ".sfn") else { eprintln!("skip: slim image"); return };
    let locs = entries(&iso, ".loc").unwrap();
    let (_, fd) = fonts.iter().find(|(k, _)| k.ends_with(r"MISC.VIV/data\loading\loading.sfn")).unwrap();
    let font = Font::parse(fd).unwrap();
    let mut m = machine(&iso, fd);
    // Font callback table: [0] = FUN_002a1e50, no begin / end; gp − 0x7e9c (0x3c7f54) → it.
    let mut t = vec![0u8; 0x20];
    t[..4].copy_from_slice(&0x002a_1e50u32.to_le_bytes());
    m.map(TABLE, t);
    m.poke(0x003c_7f54, &TABLE.to_le_bytes()).unwrap();
    // DAT_00348b94 → packet { +4 data, +8 index }; DAT_00348ba4/ba8 GS offset 0, bac/bb0 UV offset
    // (int)(0.5 × 16) = 8, bb4 z 0 — what the begin entry FUN_002a20d0 would set for this font [M].
    let mut p = vec![0u8; 0x10];
    p[4..8].copy_from_slice(&PACKET_DATA.to_le_bytes());
    m.map(PACKET, p);
    m.map(PACKET_DATA, vec![0u8; 0x10000]);
    m.poke(0x0034_8b94, &PACKET.to_le_bytes()).unwrap();
    for (a, v) in [(0x0034_8ba4u32, 0i32), (0x0034_8ba8, 0), (0x0034_8bac, 8), (0x0034_8bb0, 8), (0x0034_8bb4, 0)] {
        m.poke(a, &v.to_le_bytes()).unwrap();
    }
    let (_, ld) = locs.iter().find(|(k, _)| k.ends_with("IDL_ENG.loc")).unwrap();
    let loc = Loc::parse(ld).unwrap();
    let mut n = 0;
    for (i, s) in loc.tables[0].iter().enumerate() {
        if s.len() > 200 { continue; }
        // A spread of positions and the draw-time scales (x × 0.75 widescreen, y × 448/480).
        let (x, y) = (12.25 + (i % 7) as f32 * 41.5, 30.0 + (i % 11) as f32 * 17.75);
        for sc in [[1.0, f32::from_bits(0x3f6e_eeef)], [0.75 * 1.2, 1.2 * 448.0 / 480.0]] {
            let game = game_layout(&mut m, x, y, s, sc);
            let port: Vec<[i32; 8]> = font.layout(x, y, s, sc).iter().map(|q| {
                let [a, b, c, d] = q.xy;
                let [e, f, g, h] = q.uv16;
                [a, b, c, d, e, f, g, h]
            }).collect();
            assert_eq!(port, game, "{:?} at ({x}, {y}) {sc:?}", nf_formats::loc::text_lossy(s));
            n += port.len();
        }
    }
    assert!(n > 50_000, "{n} glyphs");
    // Sister: dropping the 0.5-texel UV offset (bac/bb0 = 0) must change the game's output.
    m.poke(0x0034_8bac, &0i32.to_le_bytes()).unwrap();
    let off = game_layout(&mut m, 100.0, 50.0, b"Mission", [1.0, 1.0]);
    let port = font.layout(100.0, 50.0, b"Mission", [1.0, 1.0]);
    assert_ne!(off[0][4], port[0].uv16[0], "the comparison sees the UV offset");
}
