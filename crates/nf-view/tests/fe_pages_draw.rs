//! The credits roll (page 0x40000030), the 007 Bonus page (0x40000038) and the win page
//! (0x40000053): the element lists `nf_view::fe_pages` builds — what `menu_overlay` draws — pinned
//! against the disc's own PAGE records (07000048.bin, kind-8 entry 08000002) and the handlers'
//! runtime writes [M P_CREDITS 0x2152b0, P_NFBONUS 0x209670, P_WINGAME 0x215100]. Each pin has a
//! sister that shows the comparison can fail. The disc tests are opt-in through NF_ISO.

use nf_formats::menudef::{ControlDef, MenuDef, PageDef};
use nf_view::credits::{CreditsRoll, FADE_FRAME, LIST_SLOTS, PAGE_CN_AV};
use nf_view::fe_pages::*;
use nf_view::menu::{self, FrontEnd, Page};

fn iso_path() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn menu_def(iso_path: &str) -> MenuDef {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).expect("open ISO");
    let (files, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = files.iter().find(|f| f.name.eq_ignore_ascii_case("07000048.BIN")).expect("07000048.bin");
    let data = iso
        .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64)
        .expect("read 07000048.bin");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("parse 07000048.bin");
    let e = lb.entries.iter().find(|e| e.hash() == 0x0800_0002).expect("menu def 08000002");
    MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

fn msg(c: &ControlDef, id: u8) -> Option<(u32, u32)> {
    c.msgs.iter().rev().find(|m| m.0 == id).map(|m| (m.1, m.2))
}

/// The disc control a transcribed (id, authored rect) names.
fn find<'a>(page: &'a PageDef, id: u32, rect: (i32, i32, i32, i32)) -> Option<&'a ControlDef> {
    page.controls.iter().find(|c| c.id == id && (c.x as i32, c.y as i32, c.w as i32, c.h as i32) == rect)
}

/// The msg-0x2d words as the uv rect (u, v, w, h) [M Label_SendMessage 0x218890 case 0x2d].
fn uv_of(c: &ControlDef) -> [f32; 4] {
    let (a, b) = msg(c, 0x2d).unwrap_or((0, 0));
    [(a >> 16) as f32, (a & 0xffff) as f32, (b >> 16) as f32, (b & 0xffff) as f32]
}

/// The type byte as the parse holds it (the first control byte: 5 label, 1 button, 0xd window).
fn ctype(c: &ControlDef) -> u8 {
    c.b
}

#[test]
fn fixup_resolution_matches_the_games_integer_math() {
    // [M FixupResolution 0x21b610] the 007 Bonus picture (89,112,171×256) → (71,105,136×238).
    assert_eq!(fixup_resolution(89, 112, 171, 256, false), [71, 105, 136, 238]);
    // A 16-high list slot keeps h 14 (16·0.9333 → 14); a type-0xa list floors at 16.
    assert_eq!(fixup_resolution(58, 8, 254, 16, false)[3], 14);
    assert_eq!(fixup_resolution(58, 8, 254, 16, true)[3], 16);
    // The forced stamp and the full-screen label arm [M 0x21b908:305-314].
    assert_eq!(control_rect(0x1000_0224, 5, 340, 198, 255, 255), [340, 198, 255, 255]);
    assert_eq!(control_rect(0x1000_023c, 5, 0, 0, 640, 480), [0, 0, 512, 448]);
    // Sister: a type-0xd window keeps its authored rect, any other control does not.
    assert_eq!(control_rect(0x4000_0030, 0xd, 0, 0, 640, 480), [0, 0, 640, 480]);
    assert_ne!(control_rect(0x1000_01cf, 5, 89, 112, 171, 256), [89, 112, 171, 256]);
}

#[test]
fn nfbonus_elements_follow_page_0x40000038() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = menu_def(&iso);
    let page = md.page(0x4000_0038).expect("page 0x40000038");
    let bind = |id: u32| format!("<{id:x}>");
    let elems = nfbonus_elems(None, &bind);
    assert_eq!(elems.len(), NFBONUS_CONTROLS.len(), "one element per drawn control");
    for &(id, ty, x, y, w, h) in NFBONUS_CONTROLS.iter() {
        let c = find(page, id, (x, y, w, h)).unwrap_or_else(|| panic!("{id:08x} ({x},{y},{w},{h}) is not on the disc page"));
        assert_eq!(ctype(c), ty, "{id:08x}: the type byte");
        let want = control_rect(id, ctype(c), c.x as i32, c.y as i32, c.w as i32, c.h as i32);
        let e = elems.iter().find(|e| e.ctrl == id && e.rect == want).unwrap_or_else(|| panic!("{id:08x}: no element at {want:?}"));
        if let Some((col, _)) = msg(c, 0x1b) { if id != 0x1000_0165 { assert_eq!(e.colour, col, "{id:08x}: the msg-0x1b colour"); } }
        match &e.what {
            ElemWhat::Tex { tex, uv } => {
                assert_eq!(Some(*tex), c.texture, "{id:08x}: the msg-0x24 texture (no reward line stands)");
                assert_eq!(*uv, uv_of(c), "{id:08x}: the msg-0x2d uv");
            }
            ElemWhat::Text { label: Some(l), .. } => assert_eq!(Some(*l), c.label, "{id:08x}: the msg-0x18 label"),
            ElemWhat::Text { label: None, s, face, align, .. } => {
                assert_eq!(id, 0x1000_01d0);
                assert_eq!((*face, *align), (3, Align::Left), "the text list's 0x30d348 format: ff 03, no fe");
                assert_eq!(s, " ", "no reward text: msg 0x18 with 0 → the blank string");
            }
            ElemWhat::Frame => assert_eq!(id, 0x1000_0165),
            ElemWhat::Fill => panic!("the bonus page has no fill"),
            ElemWhat::Pen { .. } => panic!("the bonus page has no laid-out pen lines"),
        }
    }
    // Sister: a shifted rect is not on the disc page (the matcher can miss).
    assert!(find(page, 0x1000_01cf, (90, 112, 171, 256)).is_none());
    // The page animates nothing: its only script is the page's own (one keyframe, the focus msg).
    let scripted: Vec<u32> = page.controls.iter().filter(|c| c.scripts.iter().any(|s| s.keyframes.len() > 1)).map(|c| c.id).collect();
    assert!(scripted.is_empty(), "animated controls on the bonus page: {scripted:x?}");
    let root = &page.controls[0];
    assert_eq!(root.scripts.len(), 1);
    assert_eq!(root.scripts[0].keyframes[0].msgs.iter().map(|m| (m.id, m.a)).collect::<Vec<_>>(), vec![(0x22, 0x1000_0165)]);
}

#[test]
fn nfbonus_text_is_the_handlers_composition() {
    // [M P_NFBONUS 0x209670:22-43] tier 1 "%s\n", then each of tiers 4, 3, 2 "%s\n%s" onto the buffer.
    let line = |has, label| menu::RewardLine { has, label, texture: 0x0300_0001 };
    let b = menu::BonusFields { level: 0x0700_0005, lines: [line(true, 1), line(true, 2), line(false, 3), line(true, 4)], texture: Some(0x0300_0001), text_labels: vec![1, 4, 2], has_text: true };
    let bind = |id: u32| format!("L{id}");
    assert_eq!(nfbonus_text(Some(&b), &bind), "L1\n\nL4\nL2");
    let e = nfbonus_elems(Some(&b), &bind);
    assert!(e.iter().any(|e| e.ctrl == 0x1000_01cf && e.what == ElemWhat::Tex { tex: 0x0300_0001, uv: [0.0, 0.0, 172.0, 256.0] }), "tier 1's texture into 0x100001cf");
    // Sister: without tier 1 the buffer starts empty and the first append leads with the newline.
    let b2 = menu::BonusFields { lines: [line(false, 1), line(true, 2), line(false, 3), line(false, 4)], texture: None, text_labels: vec![2], ..b.clone() };
    assert_eq!(nfbonus_text(Some(&b2), &bind), "\nL2");
    assert_eq!(nfbonus_text(Some(&menu::BonusFields { has_text: false, ..b }), &bind), " ");
}

#[test]
fn credits_elements_follow_page_0x40000030() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = menu_def(&iso);
    let page = md.page(0x4000_0030).expect("page 0x40000030");
    let bind = |id: u32| format!("<{id:x}>");
    // The list controls: 26 slots per column, d1 = the slot, each authored 16 high (14 at runtime).
    for id in [CREDITS_LEFT, CREDITS_RIGHT] {
        let mut slots: Vec<u32> = page.controls.iter().filter(|c| c.id == id).map(|c| c.d1).collect();
        slots.sort_unstable();
        assert_eq!(slots, (0..LIST_SLOTS as u32).collect::<Vec<_>>(), "{id:08x}: the slot list");
        for c in page.controls.iter().filter(|c| c.id == id) {
            assert_eq!(control_rect(c.id, ctype(c), c.x as i32, c.y as i32, c.w as i32, c.h as i32)[3], CREDITS_ITEM_H);
            assert_eq!(msg(c, 0x1b).map(|m| m.0), Some(CREAM));
        }
    }
    let roll_at = |n: u32, bonus: bool| { let mut r = CreditsRoll::new(PAGE_CN_AV, 100, bonus); for _ in 0..n { r.tick(); } r };
    let r = roll_at(1200, false);
    let elems = credits_elems(&r, &bind);
    // The background and the fade labels against their controls.
    let bg = page.controls.iter().find(|c| c.id == CREDITS_BG).expect("0x1000023c");
    assert_eq!(elems[0].rect, control_rect(bg.id, ctype(bg), bg.x as i32, bg.y as i32, bg.w as i32, bg.h as i32));
    assert_eq!(elems[0].what, ElemWhat::Tex { tex: bg.texture.unwrap(), uv: uv_of(bg) });
    assert_eq!(elems[0].colour, 0x0000_00ff, "msg 0x1a 0xff with GameState[0x52] clear [M 0x2152b0:83-88]");
    let fade = page.controls.iter().find(|c| c.id == CREDITS_FADE).expect("0x10000215");
    assert_eq!((fade.texture, uv_of(fade)), (Some(0x0300_0042), [8.0, 23.0, 1.0, 1.0]));
    assert_eq!(control_rect(fade.id, ctype(fade), fade.x as i32, fade.y as i32, fade.w as i32, fade.h as i32), [0, 0, 0x200, 0x1c0]);
    // Every visible line: one wide left item or a left + right pair at the slot's y.
    let vis = r.visible();
    let texts: Vec<&PageElem> = elems.iter().filter(|e| matches!(e.what, ElemWhat::Text { .. })).collect();
    assert_eq!(texts.len(), vis.iter().map(|v| if v.wide { 1 } else { 2 }).sum::<usize>());
    let mut k = 0;
    for v in &vis {
        let l = nf_view::credits::credits_data::CREDITS[v.line];
        assert_eq!(texts[k].ctrl, CREDITS_LEFT);
        assert_eq!(texts[k].rect, [0x10, v.y, if v.wide { 0x1e0 } else { 0x100 }, CREDITS_ITEM_H]);
        let ElemWhat::Text { face, align, shadow, .. } = &texts[k].what else { unreachable!() };
        assert_eq!((*face, *align, *shadow), (credit_face(l.kl), Align::Centre, false));
        k += 1;
        if !v.wide {
            assert_eq!((texts[k].ctrl, texts[k].rect), (CREDITS_RIGHT, [0xf0, v.y, 0x100, CREDITS_ITEM_H]));
            k += 1;
        }
    }
    assert!(vis.len() >= 20, "a full screen of lines at frame 1200 ({} visible)", vis.len());
    // Sisters: nothing before the first line enters; the bonus flag changes the background word;
    // the fade plate appears only once the roll fades, black by its 15th frame.
    assert_eq!(credits_elems(&roll_at(0, false), &bind).len(), 1);
    assert_eq!(credits_elems(&roll_at(10, true), &bind)[0].colour, 0x4040_40ff);
    assert!(!elems.iter().any(|e| e.ctrl == CREDITS_FADE));
    let faded = credits_elems(&roll_at(FADE_FRAME + 20, false), &bind);
    assert_eq!(faded.last().map(|e| (e.ctrl, e.colour)), Some((CREDITS_FADE, 0xff)));
    assert_eq!(credits_fade_alpha(0), 0);
    assert!(credits_fade_alpha(7) > 0 && credits_fade_alpha(7) < 0xff);
}

#[test]
fn credit_kinds_pick_the_handlers_formats() {
    // [M 0x2152b0:122-147; D ELF 0x30d560/0x30d568/0x30d570] kind 1 ff 01, kind 2 ff 03, else ff 02.
    let faces: Vec<u8> = nf_view::credits::credits_data::KIND_FORMAT.iter().map(|f| f[1]).collect();
    assert_eq!(faces, vec![credit_face(0), credit_face(1), credit_face(2)]);
    // Sister: the faces differ per kind.
    assert_ne!(credit_face(1), credit_face(2));
}

#[test]
fn wingame_draws_no_control() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = menu_def(&iso);
    let page = md.page(0x4000_0053).expect("page 0x40000053");
    assert_eq!(page.controls.len(), 1, "the win page is the root window alone");
    assert_eq!((page.controls[0].id, page.aux), (0x4000_0053, 0), "no aux backdrop: the handler's movie is the picture");
    assert!(wingame_elems().is_empty());
    // Sister: the bonus page's record carries controls, so the count can fail.
    assert!(md.page(0x4000_0038).unwrap().controls.len() > 1);
}

fn tex(hash: u32) -> nf_view::scene::TextureImage {
    nf_view::scene::TextureImage { key: (0, hash), width: 256, height: 256, rgba: vec![255; 256 * 256 * 4], more: vec![], rate: 1 }
}

/// The overlay draws the lists: the credits' kind-2 headings through the FONTMedium atlas, the
/// background through 0x03000129, and the win page nothing but the stand-in plate.
#[test]
fn the_overlay_draws_the_lists_with_their_faces() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("iso");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read");
    let t = nf_formats::hud::HudTables::from_action_elf(&elf).expect("HUD tables");
    let hashes = [0x0300_0003u32, 0x0300_0005, 0x0300_00d8, CREDITS_BG_TEX];
    let textures: Vec<(u32, nf_view::scene::TextureImage)> = hashes.iter().map(|&h| (h, tex(h))).collect();
    let lang = {
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat")).expect("USATxt.dat");
        nf_formats::hud::Lang::parse(&img.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read")).expect("parse")
    };
    let runs_of = |fe: &FrontEnd, h: u32| -> usize {
        let f = menu::menu_overlay_fonts(fe, &t.font, &t.font_header, &t.font_medium, Some(&lang), &textures, 4.0 / 3.0);
        let i = hashes.iter().position(|&x| x == h).unwrap() + 1;
        f.runs.iter().filter(|r| r.0 as usize == i).map(|r| r.3 as usize).sum()
    };
    let mut fe = FrontEnd::new();
    fe.open_credits(PAGE_CN_AV, 100, false);
    for _ in 0..1200 { fe.credits.tick(); }
    let kind2 = fe.credits.visible().iter().any(|v| { let l = nf_view::credits::credits_data::CREDITS[v.line]; l.kl == 2 || (!v.wide && l.kr == 2) });
    assert!(kind2, "frame 1200 shows a heading");
    assert!(runs_of(&fe, 0x0300_00d8) > 0, "the headings draw in FONTMedium");
    assert!(runs_of(&fe, CREDITS_BG_TEX) > 0, "the background label draws");
    // Sister: before any line enters, no FONTMedium glyph draws.
    let mut fe0 = FrontEnd::new();
    fe0.open_credits(PAGE_CN_AV, 100, false);
    assert_eq!(runs_of(&fe0, 0x0300_00d8), 0);
    // The win page: no control quads (the plate only — white texture slot or the font texel).
    let mut fw = FrontEnd::new();
    fw.just_completed = true;
    fw.win_enter();
    fw.page = Page::WinGame;
    let f = menu::menu_overlay_fonts(&fw, &t.font, &t.font_header, &t.font_medium, None, &textures, 4.0 / 3.0);
    assert_eq!(f.v.len(), 6, "the win page draws only its stand-in plate");
    fw.backdrop_live = true;
    let f = menu::menu_overlay_fonts(&fw, &t.font, &t.font_header, &t.font_medium, None, &textures, 4.0 / 3.0);
    assert!(f.v.is_empty(), "with the movie up the overlay draws nothing");
}
