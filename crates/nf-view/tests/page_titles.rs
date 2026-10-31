//! The page title (header) placement every front-end page shares, and the corner stamp on the 007
//! Bonus page.
//!
//! Evidence: the menu-def 08000002 in 07000048.bin [D]; `MenuManager_Create` 0x21b908:305-314 (the
//! placement arms), `FixupResolution` 0x21b610, `Label_Update` 0x218cb8:178-205 (pen y = ctrl.y +
//! h/2 + extent_h/2 − 1), `Font_GetTextExtent` 0x1ef588 → `__Font_DrawText` 0x1ee598:65-71 (the
//! extent height of one line = glyph 0's h + gap + 1) [M]; PCSX2 stats/results captures: title ink
//! rows 43.1–56.8 of 480 [L].
//!
//! 1. Every header control on the disc (NAME face 1, type 5, x 50) is (50,40,·×17) — bar page
//!    0x40000012's y 39 — and `page_header_pen` puts FONTNFont2's pen at (40,53) / (40,52).
//!    Sister: the old fixed baseline 54.5·448/480 is not that pen.
//! 2. A drawn page's header glyphs sit at that pen. Sister: a face two units taller moves it.
//! 3. The 007 Bonus page draws the corner stamp at its unscaled (340,198) rect clipped at the
//!    512-wide screen, the same quad as the stats page's.

use nf_view::menu::{self, FrontEnd, Page};

fn iso() -> Option<nf_formats::iso9660::IsoImage> {
    let path = std::env::var_os("NF_ISO")?;
    Some(nf_formats::iso9660::IsoImage::open(path).expect("open ISO"))
}

fn menu_def(iso: &nf_formats::iso9660::IsoImage) -> nf_formats::menudef::MenuDef {
    let (files, bin) = nf_formats::disc::files_bin_entries(iso).expect("FILES.BIN");
    let fe = files.iter().find(|f| f.name.eq_ignore_ascii_case("07000048.BIN")).expect("07000048.bin");
    let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("parse 07000048.bin");
    let e = lb.entries.iter().find(|e| e.hash() == 0x0800_0002).expect("menu-def entry");
    nf_formats::menudef::MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

#[test]
fn every_header_control_gets_the_label_update_pen() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set; skip"); return };
    let def = menu_def(&iso);
    let mut n = 0;
    for p in &def.pages {
        for c in &p.controls {
            let (font, _) = c.name_raw.as_deref().map(menu::name_format).unwrap_or((None, None));
            if font != Some(1) || c.b != 5 || c.x != 50 || c.label.is_none() { continue; }
            n += 1;
            let want_y = if p.id == menu::PAGE_MP_OPTIONS { 39 } else { menu::HEADER_RECT.1 as i16 };
            assert_eq!((c.x, c.y, c.h), (menu::HEADER_RECT.0 as i16, want_y, menu::HEADER_RECT.3 as i16), "page {:#010x} header {:#010x}", p.id, c.id);
            assert!(c.w == menu::HEADER_RECT.2 as i16 || c.w == 540, "page {:#010x}: width {} (376 or 540 — neither moves the pen)", p.id, c.w);
        }
    }
    assert!(n >= 30, "the menu def has a header on most pages ({n})");
    let elf = iso.read_file(iso.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read");
    let t = nf_formats::hud::HudTables::from_action_elf(&elf).expect("HUD tables");
    assert_eq!(t.font_header.line_height(), 20.0, "FONTNFont2's extent height: 13 + 6 + 1");
    // (50,40,376×17) → FixupResolution (40,37,300×15); pen y = (short)(37 + 7 + 10 − 1) = 53.
    assert_eq!(menu::page_header_pen(&t.font_header, 40), (40.0, 53.0));
    assert_eq!(menu::page_header_pen(&t.font_header, 39), (40.0, 52.0), "0x40000012: FixupResolution y 36");
    // Sister: the old fixed baseline is a different pen.
    assert!((menu::page_header_pen(&t.font_header, 40).1 - 54.5 * 448.0 / 480.0).abs() > 2.0);
}

fn test_font(h: i16, gap: f32) -> nf_formats::hud::Font {
    let glyph = |code: u16| nf_formats::hud::Glyph { u: 0, v: 0, w: 8, h, yoff: 0, pre: 0, post: 8, code, kerns: vec![] };
    nf_formats::hud::Font { first: 33, last: 126, gap, space: 3.0, glyphs: (33..=126).map(glyph).collect() }
}

fn texture(hash: u32) -> nf_view::scene::TextureImage {
    let (w, h) = match hash { 0x0300_0042 => (64, 64), 0x0300_014c => (256, 256), 0x0300_0075 | 0x0300_0003 => (256, 128), _ => (512, 128) };
    nf_view::scene::TextureImage { key: (0, hash), width: w, height: h, rgba: vec![255; (w * h * 4) as usize], more: vec![], rate: 1 }
}

fn textures() -> Vec<(u32, nf_view::scene::TextureImage)> {
    let mut v: Vec<u32> = menu::MENU_TEXTURES.to_vec();
    for t in [0x0300_0003, 0x0300_0005] { if !v.contains(&t) { v.push(t); } }
    v.into_iter().map(|h| (h, texture(h))).collect()
}

/// The quads drawn with one texture, as (x0, y0, x1, y1) in 512×448 units (aspect 4:3).
fn quads_of(frame: &nf_view::gpu::HudFrame, tex: &[(u32, nf_view::scene::TextureImage)], hash: u32) -> Vec<(f32, f32, f32, f32)> {
    let slot = tex.iter().position(|(h, _)| *h == hash).unwrap() + 1;
    let mut out = Vec::new();
    for &(t, _, first, count) in &frame.runs {
        if t != slot { continue; }
        for q in (first..first + count).step_by(6) {
            let v = &frame.v[q as usize..q as usize + 6];
            let xs = v.iter().map(|p| p.p[0] * 256.0 + 256.0);
            let ys = v.iter().map(|p| (1.0 - p.p[1]) * 224.0);
            out.push((xs.clone().fold(f32::MAX, f32::min), ys.clone().fold(f32::MAX, f32::min), xs.fold(f32::MIN, f32::max), ys.fold(f32::MIN, f32::max)));
        }
    }
    out
}

/// The header glyphs' main-pass top (the shadow pass sits 1.5 lower).
fn header_top(hfont: &nf_formats::hud::Font, page: Page, lang: Option<&nf_formats::hud::Lang>) -> f32 {
    let font = test_font(12, 5.0);
    let tex = textures();
    let mut fe = FrontEnd::new();
    fe.page = page;
    if page == Page::Stats { fe.mission_stats = Some(vec![]); }
    let frame = menu::menu_overlay(&fe, &font, hfont, lang, &tex, 4.0 / 3.0);
    let q = quads_of(&frame, &tex, 0x0300_0005);
    assert!(!q.is_empty(), "{page:?}: the header draws with the FONTNFont2 atlas");
    q.iter().filter(|q| q.1 < 100.0).map(|q| q.1).fold(f32::MAX, f32::min)
}

/// USATxt.dat out of FILES.BIN (the header strings are language labels).
fn iso_lang() -> Option<nf_formats::hud::Lang> {
    let iso = iso()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).ok()?;
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat"))?;
    let bytes = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
    nf_formats::hud::Lang::parse(&bytes).ok()
}

#[test]
fn the_drawn_header_sits_at_the_pen() {
    // A face with FONTNFont2's metrics (glyph h 13, gap 6 → extent 20).
    let hfont = test_font(13, 6.0);
    let (_, py) = menu::page_header_pen(&hfont, 40);
    assert_eq!(py, 53.0);
    let glyph_dy = hfont.layout("M", 0.0, 0.0).0[0].y;
    // The stats page's header has a literal fallback, so it draws without the language file.
    let top = header_top(&hfont, Page::Stats, None);
    assert!((top - (py + glyph_dy)).abs() < 0.01, "Stats: header glyph top {top}, want {}", py + glyph_dy);
    // Sister: a face two units taller (extent 22) moves the pen down by one, and the glyphs with it.
    let tall = test_font(15, 6.0);
    assert_eq!(menu::page_header_pen(&tall, 40).1, 54.0);
    let top_tall = header_top(&tall, Page::Stats, None);
    let want_tall = 54.0 + tall.layout("M", 0.0, 0.0).0[0].y;
    assert!((top_tall - want_tall).abs() < 0.01 && (top_tall - top).abs() > 0.5, "taller face: {top_tall} (want {want_tall}) vs {top}");
    // With the disc's strings: the list pages share the pen; Scenario Options is one unit higher.
    let Some(lang) = iso_lang() else { eprintln!("NF_ISO not set; the label-titled pages skip"); return };
    for page in [Page::Codename, Page::Difficulty, Page::MissionSelect, Page::Dossier] {
        let top = header_top(&hfont, page, Some(&lang));
        assert!((top - (py + glyph_dy)).abs() < 0.01, "{page:?}: header glyph top {top}, want {}", py + glyph_dy);
    }
    let (_, py39) = menu::page_header_pen(&hfont, 39);
    assert_eq!(py39, 52.0);
    let top = header_top(&hfont, Page::MpOptions, Some(&lang));
    assert!((top - (py39 + glyph_dy)).abs() < 0.01, "MpOptions header glyph top {top}, want {}", py39 + glyph_dy);
}

#[test]
fn the_bonus_page_draws_the_corner_stamp_unscaled_and_clipped() {
    let font = test_font(12, 5.0);
    let hfont = test_font(13, 6.0);
    let tex = textures();
    let mut fe = FrontEnd::new();
    fe.page = Page::NfBonus;
    let frame = menu::menu_overlay(&fe, &font, &hfont, None, &tex, 4.0 / 3.0);
    let stamp = quads_of(&frame, &tex, 0x0300_014c);
    assert_eq!(stamp.len(), 1, "one stamp quad");
    let s = stamp[0];
    assert!((s.0 - 340.0).abs() < 0.01 && (s.1 - 198.0).abs() < 0.01, "at (340,198) of 512×448: {s:?}");
    assert!((s.2 - 512.0).abs() < 0.01, "clipped at the 512-wide screen's right edge (not 340 + 255): {s:?}");
    assert!((s.3 - 453.0).abs() < 0.01, "255 tall, unscaled: {s:?}");
    // Sister: the stats page's stamp is the same quad.
    fe.page = Page::Stats;
    fe.mission_stats = Some(vec![]);
    let frame = menu::menu_overlay(&fe, &font, &hfont, None, &tex, 4.0 / 3.0);
    let st = quads_of(&frame, &tex, 0x0300_014c);
    assert_eq!(st.len(), 1);
    assert_eq!(st[0], s, "the stats page's corner stamp");
}
