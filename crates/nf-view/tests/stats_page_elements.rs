//! The Mission Stats page 0x40000037 and the results page 0x40000036: the port's element lists
//! (`menu::STATS_PAGE_ELEMENTS` / `RESULTS_PAGE_ELEMENTS`) against the page data on the disc, and the
//! drawn page against the list widget's own layout.
//!
//! Evidence: the menu-def 08000002 in 07000048.bin [D]; `List_Update` 0x21a598 / `List_Init` 0x219488 /
//! `Label_Update` 0x218cb8 / `Font_ParseFormat` 0x1ee3e0 [M]; the live list control read from a PCSX2
//! run on the stats page after a 07000004 finish (rect (48,73,416×302), stride 19.8, 15 rows, widths
//! 35/21/21/24, aligns 1/2/2/2) [L 2026-10-31].
//!
//! 1. Every control of both pages (bar the page container) matches the port's element list: id, rect,
//!    label, texture, and the font + alignment its NAME format string selects. Sister: a list with one
//!    rect, one font or one control changed fails the same comparison.
//! 2. The stats page's aux movie is its PAGE record's (the backdrop) — in menu_movies.rs's map test.
//! 3. The list font's extent height (its line advance) × 1.1 is the live stride 19.8 [M List_Init :75; L].
//! 4. The drawn stats page: the 8 striped plates of a full 15-row list at the widget's stride, the
//!    corner stamp at its unscaled (340,198) rect, the second row in the highlight colour; sister: a
//!    list of 2 rows draws 1 plate.
//! 5. The drawn results page uses the FONTMedium atlas for its score lines when that face is given.

use nf_view::menu::{self, FrontEnd, Outcome, Page, PageElement};

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

/// The page's controls as elements, in the data's order; the page container (b 13) is skipped.
fn disc_elements(def: &nf_formats::menudef::MenuDef, page: u32) -> Vec<PageElement> {
    let p = def.page(page).expect("the page is in the menu def");
    p.controls.iter().filter(|c| c.b != 13).map(|c| {
        let (font, align) = c.name_raw.as_deref().map(menu::name_format).unwrap_or((None, None));
        PageElement { id: c.id, rect: (c.x, c.y, c.w, c.h), label: c.label, texture: c.texture, font, align }
    }).collect()
}

/// The comparison the pins use: equal length and element-wise equal.
fn same(a: &[PageElement], b: &[PageElement]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y)
}

#[test]
fn the_page_element_lists_are_the_page_data() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set; skip"); return };
    let def = menu_def(&iso);
    for (page, port) in [(menu::PAGE_STATS, &menu::STATS_PAGE_ELEMENTS[..]), (menu::PAGE_RESULTS, &menu::RESULTS_PAGE_ELEMENTS[..])] {
        let disc = disc_elements(&def, page);
        for e in &disc { eprintln!("page {page:#010x}: {e:x?}"); }
        assert!(same(&disc, port), "page {page:#010x}: the port's element list must be the page data\n disc {disc:x?}\n port {port:x?}");
        // Sister: the same comparison catches a moved control, a wrong face and a missing control.
        let mut moved = port.to_vec();
        moved[0].rect.1 += 1;
        assert!(!same(&disc, &moved), "a moved control must fail the comparison");
        let mut face = port.to_vec();
        let i = face.iter().position(|e| e.font.is_some()).unwrap();
        face[i].font = face[i].font.map(|f| f % 3 + 1);
        assert!(!same(&disc, &face), "a wrong face must fail the comparison");
        assert!(!same(&disc, &port[1..]), "a missing control must fail the comparison");
    }
    // The NAME decode itself, on the formats the data carries: the headers "\xff\x01", the score
    // labels "\xff\x03\xfe\x02".
    let res = disc_elements(&def, menu::PAGE_RESULTS);
    assert_eq!((res[0].font, res[0].align), (Some(1), None), "the header face [D]");
    assert_eq!((res[2].font, res[2].align), (Some(3), Some(2)), "Total Score: FONTMedium, right-aligned [D]");
    let p = def.page(menu::PAGE_STATS).unwrap();
    assert_eq!(p.aux, menu::aux_movie_for(menu::PAGE_STATS), "the stats page's backdrop is its PAGE record's aux movie");
    assert_ne!(menu::aux_movie_for(menu::PAGE_STATS), 0, "sister: the record's aux is a movie");
}

#[test]
fn the_list_stride_is_the_list_fonts_height() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set; skip"); return };
    let elf = iso.read_file(iso.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read");
    let t = nf_formats::hud::HudTables::from_action_elf(&elf).expect("HUD tables");
    // Font_GetTextExtent's height = the line advance: glyph 0's h + gap + 1 [M 0x1ee598:65-71].
    let h = t.font.line_height();
    assert!((h * 1.1 - menu::STATS_LIST_STRIDE).abs() < 1e-4, "SerpLight's height {h} × 1.1 is the live stride 19.8 [M 0x219488:75; L]");
    assert!((t.font_medium.line_height() * 1.1 - menu::STATS_LIST_STRIDE).abs() > 0.5, "sister: FONTMedium is a different size");
}

fn test_font(h: i16) -> nf_formats::hud::Font {
    let glyph = |code: u16| nf_formats::hud::Glyph { u: 0, v: 0, w: 8, h, yoff: 0, pre: 0, post: 8, code, kerns: vec![] };
    nf_formats::hud::Font { first: 33, last: 126, gap: 0.0, space: 3.0, glyphs: (33..=126).map(glyph).collect() }
}

fn texture(hash: u32) -> nf_view::scene::TextureImage {
    let (w, h) = match hash { 0x0300_0042 => (64, 64), 0x0300_014c => (256, 256), 0x0300_0075 | 0x0300_0003 => (256, 128), _ => (512, 128) };
    nf_view::scene::TextureImage { key: (0, hash), width: w, height: h, rgba: vec![255; (w * h * 4) as usize], more: vec![], rate: 1 }
}

fn textures() -> Vec<(u32, nf_view::scene::TextureImage)> {
    menu::MENU_TEXTURES.iter().copied().chain([0x0300_0003]).map(|h| (h, texture(h))).collect()
}

fn row(name: &str, cur: &str, tgt: &str, tot: &str) -> nf_game::stats::StatsRow {
    nf_game::stats::StatsRow { id: 1, category: name.into(), current: cur.into(), target: tgt.into(), total: tot.into(), indented: false }
}

/// The quads (6 vertices) drawn with one texture, as (x0, y0, x1, y1) in 512×448 screen units
/// (aspect 4:3: x = ndc·256 + 256, y = (1 − ndc)·224).
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

#[test]
fn the_stats_page_draws_the_list_widgets_rows() {
    let font = test_font(18);
    let tex = textures();
    let mut fe = FrontEnd::new();
    fe.page = Page::Stats;
    fe.mission_stats = Some((0..14).map(|k| row(&format!("Row{k}"), "1", "2", "3")).collect());
    let frame = menu::menu_overlay(&fe, &font, &font, None, &tex, 4.0 / 3.0);
    // Plates: the even rows of the header + 14 data rows = rows 0,2,…,14 → 8, at the widget's
    // stride from y 73 + 3, the list's full width 48..464 [M 0x21a598:271-287; L the live rect].
    let plates = quads_of(&frame, &tex, 0x0300_0042);
    assert_eq!(plates.len(), 8, "15 rows → 8 striped plates");
    for (i, p) in plates.iter().enumerate() {
        let want_y = (2.0 * i as f32 * menu::STATS_LIST_STRIDE + 73.0 + 3.0).trunc();
        assert!((p.1 - want_y).abs() < 0.01 && (p.0 - 48.0).abs() < 0.01 && (p.2 - 464.0).abs() < 0.01, "plate {i}: {p:?} (want y {want_y})");
    }
    // The corner stamp at its own rect in the 512×448 screen [L the capture fit].
    let stamp = quads_of(&frame, &tex, 0x0300_014c);
    assert_eq!(stamp.len(), 1);
    assert!((stamp[0].0 - 340.0).abs() < 0.01 && (stamp[0].1 - 198.0).abs() < 0.01 && (stamp[0].2 - 595.0).abs() < 0.01, "stamp {:?}", stamp[0]);
    // The highlight colour is on the second row's glyphs only: the font quads in 0x7f7f7f.
    let font_slot = tex.iter().position(|(h, _)| *h == 0x0300_0003).unwrap() + 1;
    let white: usize = frame.runs.iter().filter(|r| r.0 == font_slot).map(|&(_, _, first, count)| {
        frame.v[first as usize..(first + count) as usize].iter().filter(|v| (v.c[0] - 127.0 / 128.0).abs() < 1e-3 && (v.c[1] - 127.0 / 128.0).abs() < 1e-3).count()
    }).sum();
    let row1_glyphs = ("Row0".len() + 3) * 6;
    assert_eq!(white, row1_glyphs, "the highlight colour is on row 1's four cells (\"Row0\", 1, 2, 3)");
    // Sister: a 2-row list draws one plate.
    fe.mission_stats = Some(vec![row("Only", "1", "2", "3")]);
    let frame = menu::menu_overlay(&fe, &font, &font, None, &tex, 4.0 / 3.0);
    assert_eq!(quads_of(&frame, &tex, 0x0300_0042).len(), 1, "2 rows → 1 plate");
}

#[test]
fn the_results_page_draws_the_score_lines_in_font_medium() {
    let font = test_font(18);
    let medium = test_font(24);
    let tex = textures();
    let mut fe = FrontEnd::new();
    fe.page = Page::Results;
    fe.outcome = Outcome::Complete;
    let with = menu::menu_overlay_fonts(&fe, &font, &font, Some(&medium), None, &tex, 4.0 / 3.0);
    let lines = quads_of(&with, &tex, menu::MEDIUM_FONT_TEX);
    assert!(!lines.is_empty(), "the score lines draw with the FONTMedium atlas");
    // "Total Score:" is right-aligned on its box's right edge (48 + 264 → 249.6 of 512, rounded 250).
    let right = lines.iter().filter(|q| q.1 < 320.0 && q.1 > 300.0).map(|q| q.2).fold(f32::MIN, f32::max);
    eprintln!("medium quads: {lines:?}");
    assert!(right <= 250.0 + 0.01, "the labels end at the box's right edge: {right}");
    assert_eq!(quads_of(&with, &tex, 0x0300_014c).len(), 1, "the corner stamp");
    // Sister: without the face the same lines fall back to the SerpLight atlas.
    let without = menu::menu_overlay(&fe, &font, &font, None, &tex, 4.0 / 3.0);
    assert!(quads_of(&without, &tex, menu::MEDIUM_FONT_TEX).is_empty());
}
