//! The pause OBJECTIVES tab's objective-hint box — option-box page 0x4000002f of the mission menu
//! defs, opened by `C_GCPAUSE_Handler` 0x204408's control-1 0x4b arm through
//! `Menu_CreateOptionBox` 0x1f9608 and driven by the focused list's scroll (`Scroll_Update`
//! 0x221b20) and `C_LBMSGOPTIONS_Handler` 0x2022a0.
//!
//! 1. (NF_ISO) The page's three controls, SET 0xb's one instance and the empty SET 2 are the
//!    disc's, identical in three mission archives. Sister: SET 7's instances are not SET 0xb's.
//! 2. The geometry: the pitch is chopped (17.999998 for an 18-unit face), the box is
//!    text + 0x15 + 0x36 tall, centred on 448, the memo 10 below its top, the list text + 0x1e
//!    below. Sister: round-to-nearest maths would give a different box.
//! 3. The element list: the SET 0xb fill over the button rect, the memo lines centred with their
//!    chopped baselines, the OK row centred in its 403-wide column in 0x645a4cff, no shadow.
//! 4. The flow: ✕ opens the box (select blip, the box page's sound gates), ~V ticks without
//!    moving anything, △ closes it silently, ✕ closes it with the blip; the pause gates return.
//! 5. The sizing: every box, the first of a pause included, is sized for its own wrapped text
//!    (the 0x44 push's Page_Update counts the memo before msg 0x65 reads it).
//! 6. The overlay: the box hides the objectives list and draws the fill over the button rect,
//!    at half alpha (`psiDrawSprites` halves the sprite alpha byte).
//! 7. The wrap: `Font_WordWrapString`'s measure (spaces free, +1 per glyph, the stop checked
//!    before each character, none after the last); (NF_ISO) the 05 hint breaks after "front".

use nf_formats::menudef::{ControlDef, MenuDef};
use nf_view::fe_pages::*;
use nf_view::menu::{self, Action, FrontEnd, Page, PauseObjective};

fn def(iso_path: &str, file: &str, entry: u32) -> MenuDef {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).expect("open ISO");
    let (files, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = files.iter().find(|f| f.name.eq_ignore_ascii_case(file)).expect("the archive");
    let data = iso
        .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64)
        .expect("read the archive");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("parse the archive");
    let e = lb.entries.iter().find(|e| e.hash() == entry).expect("the menu def entry");
    MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

fn msg(c: &ControlDef, id: u8) -> Option<(u32, u32)> {
    c.msgs.iter().rev().find(|m| m.0 == id).map(|m| (m.1, m.2))
}

#[test]
fn the_box_controls_are_the_mission_defs_page() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
    for file in ["07000005.BIN", "0700000A.BIN", "07000009.BIN"] {
        let md = def(&iso, file, 0x0800_0001);
        let p = md.page(HINTBOX_PAGE).unwrap_or_else(|| panic!("{file}: page 0x4000002f"));
        let ctrl = |id: u32| p.controls.iter().find(|c| c.id == id).unwrap_or_else(|| panic!("{file}: {id:x}"));
        let rect = |c: &ControlDef| (c.id, c.x as i32, c.y as i32, c.w as i32, c.h as i32);
        let (b, m, l) = (ctrl(HINTBOX_BUTTON.0), ctrl(HINTBOX_MEMO.0), ctrl(HINTBOX_LIST.0));
        assert_eq!((rect(b), b.b, b.f2, b.f1), (HINTBOX_BUTTON, 1, 0xb, 0x19), "{file}: the button");
        assert_eq!((rect(m), m.b, m.f2, m.f1), (HINTBOX_MEMO, 200, 2, 0x12), "{file}: the memo");
        assert_eq!((rect(l), l.b, l.f2, l.f1), (HINTBOX_LIST, 6, 2, 0x12), "{file}: the list");
        // The button's LBL 2 is in colour 0 (invisible); the memo and list text is cream.
        assert_eq!((msg(b, 0x1b), msg(b, 0x1c)), (Some((0, 0)), Some((0, 0))));
        assert_eq!((msg(m, 0x1b).unwrap().0, msg(m, 0x1c).unwrap().0), (HINTBOX_TEXT, HINTBOX_TEXT));
        assert_eq!((msg(l, 0x1b).unwrap().0, msg(l, 0x1c).unwrap().0), (0x645a_49ff, 0x645a_49ff));
        // Column 0 at 100 %, selection mode 1 (the record byte List_Init copies to +0x564).
        assert_eq!(msg(l, 0x28), Some((0, 100)));
        assert_eq!(l.d2, 1);
        // SET 0xb COMPONENT 0 is the one fill; every SET 2 component is empty.
        let set = |id: u16| md.sets.iter().find(|s| s.id == id).unwrap_or_else(|| panic!("SET {id:x}"));
        let c0 = &set(0xb).components[0];
        assert_eq!(c0.hash, ATLAS_BUTTONS);
        let same = |ours: &[CompInst]| c0.sprites.len() == ours.len()
            && c0.sprites.iter().zip(ours).all(|(d, o)| d.w == o.w && d.d1 == o.colour && d.d2 == o.flags && d.f1 == o.wf && d.f2 == o.hf);
        assert!(same(&SETB_BUTTON), "{file}: SET 0xb's instance");
        assert_eq!(button_component(0xb).map(|(i, t)| (i.len(), t)), Some((1, ATLAS_BUTTONS)));
        assert!(set(2).components.iter().all(|c| c.count == 0 && c.a == 0 && c.b == 0 && c.c == 0 && c.d == 0), "{file}: SET 2 is empty, no insets");
        // Sister: SET 7's nine-slice is not this fill.
        assert!(!same(&SET7_BUTTON));
    }
}

#[test]
fn the_geometry_follows_menu_create_option_box() {
    let s = hintbox_memo_stride(18.0);
    assert_eq!(s as i32, 17, "the chopped pitch: (18 · 0.01f) · 100 = {s}");
    assert!(s < 18.0);
    // Sister: round-to-nearest single-precision maths lands on 18 exactly.
    assert_eq!((18.0f32 * 0.01) * 100.0, 18.0);
    assert_eq!(hintbox_list_stride(18.0) as i32, 19);
    // No previous text: h = 0 + 0x15 + 0x36 = 75, y = (448 − 75)/2 = 186.
    let (b, m, l) = hintbox_rects(0, s);
    assert_eq!(b, [60, 186, 392, 75], "the button");
    assert_eq!(m, [64, 196, 384, 96], "the memo (its own fixed-up h)");
    assert_eq!(l, [54, 216, 404, 85], "the list (its own fixed-up h)");
    // Seven lines at the integer pitch 17: h = 119 + 75 = 194, y = 127.
    let (b, m, l) = hintbox_rects(7, s);
    assert_eq!((b[1], b[3], m[1], l[1]), (127, 194, 137, 276));
    // Sister: an 18 pitch (nearest rounding) would size the box 7 units taller.
    let (b18, _, _) = hintbox_rects(7, 18.0);
    assert_eq!(b18[3] - b[3], 7);
}

#[test]
fn the_elements_are_the_fill_the_memo_lines_and_the_ok_row() {
    let v = HintBoxView {
        lines: vec![("AA".into(), 10.0), ("BBBB".into(), 20.0)],
        size_lines: 2,
        memo_stride: hintbox_memo_stride(18.0),
        list_stride: hintbox_list_stride(18.0),
        ok: ("OK".into(), 30.0),
    };
    let e = hintbox_elems(&v);
    assert_eq!(e.len(), 4, "fill + two lines + OK: {e:?}");
    // The fill: h = 2·17 + 75 = 109, y = (448 − 109)/2 = 169.
    assert_eq!(e[0].rect, [60, 169, 392, 109]);
    assert_eq!(e[0].colour, 0x0000_007f);
    assert_eq!(e[0].what, ElemWhat::Tex { tex: ATLAS_BUTTONS, uv: [32.0, 19.0, 1.0, 1.0] });
    // Memo lines: x = 64 + trunc((384 − w)/2) + 2; baselines trunc(k·17.999998 + 179 + 17).
    assert_eq!((e[1].rect[0], e[1].rect[1]), (64 + 187 + 2, 196));
    assert_eq!((e[2].rect[0], e[2].rect[1]), (64 + 182 + 2, 213), "the chopped sum 213.99998 truncates");
    assert!(e[1..3].iter().all(|x| x.colour == HINTBOX_TEXT && matches!(&x.what, ElemWhat::Pen { face: 2, shadow: true, .. })));
    // The OK row: list y = 169 + 34 + 30 = 233, baseline 233 + 19; column 403 wide from 56.
    assert_eq!((e[3].rect[0], e[3].rect[1]), (56 + 186, 252));
    assert_eq!(e[3].colour, HINTBOX_OK_COLOUR);
    assert!(matches!(&e[3].what, ElemWhat::Pen { face: 2, shadow: false, .. }));
    // Sister: no OK text, no row; no lines, only the fill.
    let bare = HintBoxView { lines: vec![], ok: (String::new(), 0.0), ..v.clone() };
    assert_eq!(hintbox_elems(&bare).len(), 1);
}

fn fe_with_rows(n: usize) -> FrontEnd {
    let mut fe = FrontEnd::new();
    fe.flow.current_level = 0x0700_0005;
    assert!(fe.open_pause());
    fe.pause_objectives = (0..n).map(|i| PauseObjective {
        text: 0x0400_0016 + i as u32,
        hint: if i == n - 1 { 0xffff_ffff } else { 0x0400_0021 + i as u32 },
        done: false,
    }).collect();
    fe.pause_tab = 1;
    fe.pause_objectives_sel = 0;
    let _ = fe.drain_sfx().count();
    fe
}

fn sounds(fe: &mut FrontEnd) -> Vec<u32> {
    fe.drain_sfx().map(|s| s.id).collect()
}

#[test]
fn the_box_takes_the_lists_input_and_sounds() {
    let mut fe = fe_with_rows(3);
    assert_eq!(fe.enabled_sound_flags, 0xb, "the pause page's gates");
    assert!(fe.action(Action::Confirm), "✕ opens the box");
    assert_eq!(fe.pause_hint, Some(0x0400_0021));
    assert_eq!(sounds(&mut fe), vec![0x1da], "the objectives list's ✕ blip (MENU_SOUND 0)");
    assert_eq!(fe.enabled_sound_flags, 9, "Menu_EnableSounds(0x4000002f) → the default 9");
    // ~V: the one-row list wraps onto itself with the step ticks; nothing moves.
    assert!(!fe.action(Action::Up));
    assert_eq!(sounds(&mut fe), vec![0x1d8]);
    assert!(!fe.action(Action::Down));
    assert_eq!(sounds(&mut fe), vec![0x1d9]);
    assert_eq!((fe.pause_objectives_sel, fe.pause_tab), (0, 1));
    // △: MENU_SOUND 9 — silence; the box closes, the pause stays.
    assert!(fe.action(Action::Back));
    assert!(fe.pause_hint.is_none() && !fe.closed);
    assert!(sounds(&mut fe).is_empty(), "the box's △ is silent");
    assert_eq!(fe.enabled_sound_flags, 0xb, "back on the pause page's gates");
    // Sister: △ on the pause page itself buzzes (MENU_SOUND 8, bit 8 of 0xb) and resumes.
    let mut p = fe_with_rows(2);
    assert!(p.action(Action::Back));
    assert_eq!(sounds(&mut p), vec![0x1d9]);
    assert!(p.closed);
    // ✕ on the box: the OK row's blip under the box's gates, then the close.
    assert!(fe.action(Action::Confirm));
    let _ = sounds(&mut fe);
    assert!(fe.action(Action::Confirm));
    assert_eq!(sounds(&mut fe), vec![0x1da]);
    assert!(fe.pause_hint.is_none());
}

#[test]
fn every_box_is_sized_by_its_own_text() {
    let font = test_font();
    // test_font's game measure: 8 + 0 + 8 + 1 = 17 per glyph, spaces free; 368 wide → 21 glyphs.
    let long = "aaaa bbbb cccc dddd eeee ffff gggg hhhh iiii jjjj kkkk";
    let v = menu::hint_box_view(long, "OK", &font);
    assert_eq!(v.lines.len(), 3, "{:?}", v.lines);
    assert_eq!(v.size_lines, 3, "the box counts its own lines");
    let (b, _, l) = hintbox_rects(v.size_lines, v.memo_stride);
    assert_eq!((b[1], b[3], l[1]), (161, 126, 161 + 51 + 30), "3 · 17 + 75 = 126 tall, y = (448 − 126)/2");
    // The first box of a pause and the reopened box are the same box (the flow keeps no count).
    let mut fe = fe_with_rows(3);
    fe.action(Action::Confirm);
    let first = menu::pause_hint_view(&fe, &font, None);
    fe.action(Action::Confirm);
    fe.action(Action::Confirm);
    assert_eq!(menu::pause_hint_view(&fe, &font, None), first, "reopened = first");
    // Sister: a shorter text sizes a shorter box — nothing carries over from the previous box.
    let short = menu::hint_box_view("aaaa", "OK", &font);
    assert_eq!(short.size_lines, 1);
    assert_ne!(hintbox_rects(short.size_lines, short.memo_stride), hintbox_rects(v.size_lines, v.memo_stride));
    // No text: no lines, the 75-unit box.
    assert_eq!(menu::hint_box_view("", "OK", &font).size_lines, 0);
}

#[test]
fn the_wrap_is_font_word_wrap_strings() {
    let font = test_font();
    // The width is checked before each character and not after the last: "aa bb" (68) keeps one
    // line at 60, while "aa bb c" stops on the space after "bb" and cuts back to "aa".
    assert_eq!(menu::font_word_wrap("aa bb", &font, 60.0), vec!["aa bb"]);
    assert_eq!(menu::font_word_wrap("aa bb c", &font, 60.0), vec!["aa", "bb c"]);
    // Spaces are free: "a a a" is 3 · 17 = 51 < 52.
    assert_eq!(menu::font_word_wrap("a a a x", &font, 52.0), vec!["a a a x"]);
    assert_eq!(menu::font_word_wrap("a a a xx", &font, 52.0), vec!["a a a", "xx"]);
    // A newline ends the line; "\n\n" gives an empty line.
    assert_eq!(menu::font_word_wrap("HINT\n\nab", &font, 368.0), vec!["HINT", "", "ab"]);
    // No break point: the line is cut one character before the stop.
    assert_eq!(menu::font_word_wrap("aaaaa", &font, 40.0), vec!["aa", "aaa"]);
    assert!(menu::font_word_wrap("", &font, 368.0).is_empty());
    // Sister: the port's layout measure (spaces 1.25 × 3, no +1) breaks "aa bb" at 60.
    assert_eq!(menu::wrap_px_for_test("aa bb", &font, 60.0), vec!["aa", "bb"]);
}

/// (NF_ISO) 07000005's front-door hint wraps where the PS2 wraps it: after "front", "you" and
/// "other", six memo lines with the HINT title and the blank line.
#[test]
fn the_05_hint_breaks_after_front() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
    let image = nf_formats::iso9660::IsoImage::open(&iso).expect("the ISO opens");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&image).expect("FILES.BIN");
    let fe_ = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat")).expect("USATxt.dat");
    let bytes = image.read_bytes(bin.lba as u64 * 2048 + fe_.offset as u64, fe_.size as u64).expect("read");
    let lang = nf_formats::hud::Lang::parse(&bytes).expect("the lang parses");
    let elf = image.read_file(image.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read ACTION.ELF");
    let font = nf_formats::hud::HudTables::from_action_elf(&elf).expect("HUD tables").font;
    let text = lang.bind(0x0400_005b).expect("label 0x0400005b");
    let v = menu::hint_box_view(text, "", &font);
    let last_words: Vec<&str> = v.lines.iter().map(|(l, _)| l.rsplit(' ').next().unwrap_or("")).collect();
    assert_eq!(v.lines.len(), 6, "{:?}", v.lines);
    assert_eq!((v.lines[0].0.as_str(), v.lines[1].0.as_str()), ("HINT", ""));
    assert_eq!(&last_words[2..5], &["front", "you", "other"], "{:?}", v.lines);
    assert_eq!(v.size_lines, 6);
    // Sister: the port's layout measure fits "door," on the first line.
    let port = menu::wrap_px_for_test(text.split("\n\n").nth(1).unwrap(), &font, 368.0);
    assert_eq!(port[0].rsplit(' ').next(), Some("door,"));
}

fn test_font() -> nf_formats::hud::Font {
    let glyph = |code: u16| nf_formats::hud::Glyph { u: 0, v: 0, w: 8, h: 12, yoff: 0, pre: 0, post: 8, code, kerns: vec![] };
    nf_formats::hud::Font { first: 33, last: 126, gap: 5.0, space: 3.0, glyphs: (33..=126).map(glyph).collect() }
}

fn textures() -> Vec<(u32, nf_view::scene::TextureImage)> {
    let mut v: Vec<u32> = menu::MENU_TEXTURES.to_vec();
    for &t in &[0x0300_0042u32, 0x0300_0003] { if !v.contains(&t) { v.push(t); } }
    v.into_iter().map(|h| (h, nf_view::scene::TextureImage { key: (0, h), width: 64, height: 64, rgba: vec![255; 64 * 64 * 4], more: vec![], rate: 1 })).collect()
}

/// The quads of one texture slot, as (x0, y0, x1, y1) in 512×448 units.
fn quads(frame: &nf_view::gpu::HudFrame, slot: usize) -> Vec<(f32, f32, f32, f32)> {
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
fn the_overlay_hides_the_list_and_draws_the_box() {
    let font = test_font();
    let tex = textures();
    let slot = |h: u32| tex.iter().position(|(t, _)| *t == h).unwrap() + 1;
    let mut fe = fe_with_rows(2);
    let before = menu::menu_overlay(&fe, &font, &font, None, &tex, 4.0 / 3.0);
    fe.action(Action::Confirm);
    let after = menu::menu_overlay(&fe, &font, &font, None, &tex, 4.0 / 3.0);
    // The fill: one quad of 0x03000042 exactly over the box — no lang, so no text: 0 lines,
    // (60,186,392×75).
    let fill = |f: &nf_view::gpu::HudFrame| quads(f, slot(ATLAS_BUTTONS)).into_iter()
        .filter(|q| (q.0 - 60.0).abs() < 0.01 && (q.1 - 186.0).abs() < 0.01 && (q.2 - 452.0).abs() < 0.01 && (q.3 - 261.0).abs() < 0.01).count();
    assert_eq!(fill(&after), 1, "the box's fill");
    assert_eq!(fill(&before), 0, "sister: no box before the ✕");
    // The fill is black at half alpha: 0x0000007f's alpha byte halved to 0x3f (psiDrawSprites).
    let fill_c: Vec<[f32; 4]> = after.runs.iter().filter(|r| r.0 == slot(ATLAS_BUTTONS))
        .flat_map(|&(_, _, first, count)| after.v[first as usize..(first + count) as usize].iter().map(|v| v.c).collect::<Vec<_>>())
        .filter(|c| c[0] == 0.0 && c[1] == 0.0 && c[2] == 0.0).collect();
    assert!(!fill_c.is_empty() && fill_c.iter().all(|c| (c[3] - 63.0 / 128.0).abs() < 1e-6), "{fill_c:?}");
    // Sister: the word itself reads 0x7f/128 without the halving.
    assert!((menu::sprite_colour(0x0000_00ff)[3] - 127.0 / 128.0).abs() < 1e-6);
    // The objectives list (the rows' "(objective)" fallback text and markers) is hidden.
    let text = |f: &nf_view::gpu::HudFrame| quads(f, slot(0x0300_0003)).len();
    assert!(text(&after) < text(&before), "the list's glyphs are gone under the box: {} → {}", text(&before), text(&after));
}

/// (NF_ISO) The Controls page's Y-Axis row draws in the page data's cream whatever the focus: the
/// label 0x10000001 LBL 0x492 and the button 0x100000c9 carry 0x645a49ff in msg 0x1b and 0x1c
/// [D page 0x40000022] and P_CNCONTROLS_Handler sends them no colour [M 0x213cd0]. Sister: the
/// style list's unfocused colour 0x73330fc0 still shows on the style row when the Y-Axis row is
/// focused.
#[test]
fn the_controls_y_axis_row_is_cream_focused_or_not() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
    let image = nf_formats::iso9660::IsoImage::open(&iso).expect("the ISO opens");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&image).expect("FILES.BIN");
    let fe_ = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat")).expect("USATxt.dat");
    let bytes = image.read_bytes(bin.lba as u64 * 2048 + fe_.offset as u64, fe_.size as u64).expect("read");
    let lang = nf_formats::hud::Lang::parse(&bytes).expect("the lang parses");
    let font = test_font();
    let tex = textures();
    let colours = |cursor: usize, x0: f32, x1: f32| -> Vec<[f32; 4]> {
        let mut fe = FrontEnd::new();
        fe.page = Page::CnControls;
        fe.cursor = cursor;
        let f = menu::menu_overlay(&fe, &font, &font, Some(&lang), &tex, 4.0 / 3.0);
        let slot = tex.iter().position(|(t, _)| *t == 0x0300_0003).unwrap() + 1;
        let mut out = Vec::new();
        for &(t, _, first, count) in &f.runs {
            if t != slot { continue; }
            for q in (first..first + count).step_by(6) {
                let v = &f.v[q as usize];
                let (x, y) = (v.p[0] * 256.0 + 256.0, (1.0 - v.p[1]) * 224.0);
                if x >= x0 && x < x1 && (95.0..130.0).contains(&y) { out.push(v.c); }
            }
        }
        out
    };
    let cream = [100.0 / 128.0, 90.0 / 128.0, 73.0 / 128.0];
    let off = [0x73 as f32 / 128.0, 0x33 as f32 / 128.0, 0x0f as f32 / 128.0];
    let is = |c: &[f32; 4], k: [f32; 3]| (c[0] - k[0]).abs() < 1e-3 && (c[1] - k[1]).abs() < 1e-3 && (c[2] - k[2]).abs() < 1e-3;
    // The Y-Axis row spans 640-x 376..560 → 512-x 300..448.
    for cursor in [0, 1] {
        let ys = colours(cursor, 300.0, 460.0);
        assert!(ys.iter().any(|c| is(c, cream)), "cursor {cursor}: the Y-Axis row draws cream");
        assert!(!ys.iter().any(|c| is(c, off)), "cursor {cursor}: no option-row colour on the Y-Axis row");
    }
    // Sister: the style row (x 48..290) keeps its list's own unfocused colour when not focused.
    assert!(colours(1, 60.0, 290.0).iter().any(|c| is(c, off)), "the style row is 0x73330fc0 while the Y-Axis row has the focus");
}
