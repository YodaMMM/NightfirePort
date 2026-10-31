//! The Screen Adjust page (0x40000047), P_PARISENUM (0x4000004a) with its card box page
//! (0x4000002a) and the in-game cheat-medal page (0x4000004c): the element lists
//! `nf_view::fe_pages` builds — what `menu_overlay` draws — pinned against the disc's own PAGE and
//! SET records (07000048.bin entry 08000002; the mission archives' 08000001) and the handlers
//! [M P_SCREENADJUST 0x217a58, C_SBSCREEN 0x217b08, Scroll_Update 0x221b20, P_PARISENUM 0x20a648,
//! PS2_DoCardFlow 0x217bd8, Memo_Update 0x21e490, P_CHEATMEDAL 0x2086b0, List_Update 0x21a598].
//! Each pin has a sister that shows the comparison can fail. The disc tests are opt-in through
//! NF_ISO.

use nf_formats::menudef::{ComponentDef, ControlDef, MenuDef, PageDef, SpriteDef};
use nf_view::fe_pages::*;
use nf_view::menu::{self, Action, FrontEnd, Page};

fn iso_path() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn def(iso_path: &str, file: &str, entry: u32, fh: u32) -> MenuDef {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).expect("open ISO");
    let (files, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = files.iter().find(|f| f.name.eq_ignore_ascii_case(file)).expect("the archive");
    let data = iso
        .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64)
        .expect("read the archive");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("parse the archive");
    let e = lb.entries.iter().find(|e| e.hash() == entry).expect("the menu def entry");
    MenuDef::parse(lb.data(&data, e), fh).expect("parse the menu def")
}

fn front_end(iso: &str) -> MenuDef {
    def(iso, "07000048.BIN", 0x0800_0002, 0x8000_0002)
}

/// A mission def (Paris Prelude's 0700000a — every mission def carries the same pages [D the scan]).
fn mission(iso: &str) -> MenuDef {
    def(iso, "0700000A.BIN", 0x0800_0001, 0x8000_0002)
}

fn msg(c: &ControlDef, id: u8) -> Option<(u32, u32)> {
    c.msgs.iter().rev().find(|m| m.0 == id).map(|m| (m.1, m.2))
}

fn uv_of(c: &ControlDef) -> [f32; 4] {
    let (a, b) = msg(c, 0x2d).unwrap_or((0, 0));
    [(a >> 16) as f32, (a & 0xffff) as f32, (b >> 16) as f32, (b & 0xffff) as f32]
}

fn rect_of(c: &ControlDef) -> [i32; 4] {
    control_rect(c.id, c.b, c.x as i32, c.y as i32, c.w as i32, c.h as i32)
}

/// The visibility a page record authors: msg 0x2b's bit 0 hides the control [M the widgets'
/// Update `(+0x7a & 1) != 0` hide arm, e.g. Label/Button/Memo/Scroll].
fn authored_visible(c: &ControlDef) -> bool {
    msg(c, 0x2b).map(|(v, _)| v & 1 == 0).unwrap_or(true)
}

fn comp<'a>(md: &'a MenuDef, set: u16, ix: usize) -> &'a ComponentDef {
    let s = md.sets.iter().find(|s| s.id == set).unwrap_or_else(|| panic!("SET {set:x}"));
    &s.components[ix]
}

fn same(disc: &[SpriteDef], ours: &[CompInst]) -> bool {
    disc.len() == ours.len()
        && disc.iter().zip(ours).all(|(d, o)| d.w == o.w && d.d1 == o.colour && d.d2 == o.flags && d.f1 == o.wf && d.f2 == o.hf)
}

fn page<'a>(md: &'a MenuDef, id: u32) -> &'a PageDef {
    md.page(id).unwrap_or_else(|| panic!("page {id:08x}"))
}

#[test]
fn the_component_records_are_the_discs_sets() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let fe = front_end(&iso);
    // The scroll / list records the pages use: SET 3 [D 08000002].
    assert_eq!(comp(&fe, 3, 3).hash, ATLAS_BUTTONS);
    assert!(same(&comp(&fe, 3, 3).sprites, &SET3_LIST_FRAME), "SET 3 COMPONENT 3 (the list frame)");
    assert!(same(&comp(&fe, 3, 6).sprites, &SET3_SCROLL_H), "SET 3 COMPONENT 6 (the horizontal scroll)");
    assert!(same(&comp(&fe, 3, 7).sprites, &SET3_SCROLL_V), "SET 3 COMPONENT 7 (the vertical scroll)");
    assert!(comp(&fe, 3, 8).sprites.is_empty(), "no thumb instances (COMPONENT 8, handle set|8 [M Scroll_Init])");
    // The card box's button frame and the memo's empty set.
    assert_eq!(comp(&fe, 0xe, 0).hash, ATLAS_BOX);
    assert!(same(&comp(&fe, 0xe, 0).sprites, &SETE_BUTTON), "SET 0xe COMPONENT 0 (the box frame)");
    assert!(comp(&fe, 2, 3).sprites.is_empty() && comp(&fe, 2, 3).a == 0 && comp(&fe, 2, 3).c == 0, "SET 2 (the memo) has no frame and no insets");
    // The mission defs carry the same SET 3 (the cheat-medal list draws from it).
    let m = mission(&iso);
    assert!(same(&comp(&m, 3, 3).sprites, &SET3_LIST_FRAME));
    assert!(same(&comp(&m, 3, 7).sprites, &SET3_SCROLL_V));
    // The fractions are the game's int → float (1 → 1.0), not raw float bits [M 0x21b908:342-357].
    assert_eq!(comp(&fe, 3, 3).sprites[8].f1, 1.0);
    // Sisters: the horizontal and vertical tracks are told apart; a one-word change is caught.
    assert!(!same(&comp(&fe, 3, 6).sprites, &SET3_SCROLL_V));
    let mut bad = SETE_BUTTON;
    bad[4].w[6] = 3;
    assert!(!same(&comp(&fe, 0xe, 0).sprites, &bad));
}

#[test]
fn component_layout_follows_setup_instance() {
    // The horizontal bar's runtime rect (256,224,128×32) → (205,210,102×29) [M FixupResolution].
    let r = control_rect(SCREENADJUST_BAR_ID, 10, 256, 224, 128, 32);
    assert_eq!(r, [205, 210, 102, 29]);
    // Focused (0x20): the two dim arrows — ◀ at dx 0, ▶ right-anchored (flag 2), both centred
    // in y (flag 8) [M Component_SetupInstance 0x1f0950's anchor arms].
    let e = component_elems(1, &SET3_SCROLL_H, ATLAS_BUTTONS, 0x20, r, 0);
    let got: Vec<([i32; 4], [f32; 4], u32)> = e.iter().map(|e| (e.rect, match e.what { ElemWhat::Tex { uv, .. } => uv, _ => panic!() }, e.colour)).collect();
    assert_eq!(got, vec![
        ([205, 216, 16, 16], [48.0, 0.0, 16.0, 16.0], 0x6f6f_6fdf),
        ([291, 216, 16, 16], [48.0, 16.0, 16.0, 16.0], 0x6f6f_6fdf),
    ]);
    // ▶ held (0x80): ◀ stays dim, ▶ turns bright.
    let e = component_elems(1, &SET3_SCROLL_H, ATLAS_BUTTONS, 0x80, r, 0);
    assert_eq!(e.iter().map(|e| e.colour).collect::<Vec<_>>(), vec![0x6f6f_6fdf, 0x7f7f_7fff]);
    // The 9-slice fills: the box frame's corners at the rect's corners, the edges spanning the
    // rest (the fractions), the centre in between.
    let f = component_elems(2, &SETE_BUTTON, ATLAS_BOX, 0x10, [40, 162, 432, 123], 0);
    assert_eq!(f.len(), 9);
    assert_eq!(f[0].rect, [40, 162, 16, 16], "top-left");
    assert_eq!(f[3].rect, [40 + 432 - 16, 162 + 123 - 16, 16, 16], "bottom-right");
    assert_eq!(f[8].rect, [56, 178, 400, 91], "the centre fills the span between the corners");
    // The focused pulse: Menu_AlphaGet adds to R, G, B on fraction-free instances only.
    assert_eq!(alpha_add(0x6f6f_6fdf, 0x10), 0x7f7f_7fdf);
    let p = component_elems(3, &SET3_LIST_FRAME, ATLAS_BUTTONS, 0x20, [0, 0, 100, 100], 0x10);
    assert_eq!(p[0].colour, 0x8f8f_8f7f, "a corner pulses");
    assert_eq!(p[8].colour, 0x7f7f_7f7f, "the centre (wf/hf 1) does not");
    // Sister: an unfocused horizontal scroll draws nothing (no instance carries 0x10).
    assert!(component_elems(1, &SET3_SCROLL_H, ATLAS_BUTTONS, 0x10, r, 0).is_empty());
}

#[test]
fn screenadjust_elements_follow_page_0x40000047() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = front_end(&iso);
    let pg = page(&md, menu::PAGE_SCREEN_ADJUST);
    let bind = |id: u32| format!("<{id:x}>");
    let elems = screenadjust_elems(0, 0, 0, &bind);
    // Every texture label on the page is an element with the disc's rect, texture, uv and colour.
    let tex_labels: Vec<&ControlDef> = pg.controls.iter().filter(|c| c.b == 5 && c.texture.is_some()).collect();
    assert_eq!(tex_labels.len(), 25, "the picture and 24 brackets");
    for c in &tex_labels {
        let want = rect_of(c);
        let e = elems.iter().find(|e| e.rect == want && matches!(e.what, ElemWhat::Tex { .. })).unwrap_or_else(|| panic!("no element at {want:?}"));
        assert_eq!(Some(e.colour), msg(c, 0x1b).map(|m| m.0), "{want:?}: the msg-0x1b colour");
        let ElemWhat::Tex { tex, uv } = &e.what else { unreachable!() };
        assert_eq!(Some(*tex), c.texture);
        assert_eq!(*uv, uv_of(c));
        assert!(authored_visible(c));
    }
    // The brackets in the transcription are the disc's, in order.
    let disc_brackets: Vec<(i32, i32, i32, i32)> = tex_labels.iter().filter(|c| c.texture == Some(ATLAS_BUTTONS)).map(|c| (c.x as i32, c.y as i32, c.w as i32, c.h as i32)).collect();
    assert_eq!(disc_brackets, SCREENADJUST_BRACKETS.to_vec());
    // The hint label.
    let hint = pg.controls.iter().find(|c| c.label == Some(SCREENADJUST_HINT_LABEL)).expect("the hint");
    assert!(elems.iter().any(|e| e.rect == rect_of(hint) && matches!(&e.what, ElemWhat::Text { label: Some(0x67b), s, .. } if s == "<67b>")));
    // The two scrolls: type 10, SET 3 (the f2 word), one shared id, the d1 axis tags 1 / 0, and
    // the page script focuses that id.
    let bars: Vec<&ControlDef> = pg.controls.iter().filter(|c| c.b == 10).collect();
    assert_eq!(bars.len(), 2);
    assert!(bars.iter().all(|c| c.id == SCREENADJUST_BAR_ID && c.f2 == 3));
    assert_eq!((bars[0].x as i32, bars[0].y as i32, bars[0].w as i32, bars[0].h as i32, bars[0].d1), (256, 224, 128, 32, 1));
    assert_eq!((bars[1].x as i32, bars[1].y as i32, bars[1].w as i32, bars[1].h as i32, bars[1].d1), (304, 176, 32, 128, 0));
    let kf = &pg.controls[0].scripts[0].keyframes[0];
    assert_eq!(kf.msgs.iter().map(|m| (m.id, m.a)).collect::<Vec<_>>(), vec![(0x22, SCREENADJUST_BAR_ID)]);
    // Both draw focused: ◀ ▶ on the horizontal, ▲ ▼ on the vertical, dim.
    let arrows: Vec<&PageElem> = elems.iter().filter(|e| e.ctrl == SCREENADJUST_BAR_ID).collect();
    assert_eq!(arrows.len(), 4);
    assert!(arrows.iter().all(|e| e.colour == 0x6f6f_6fdf));
    let v = rect_of(bars[1]);
    assert_eq!(arrows[2].rect, [v[0] + (v[2] - 16) / 2, v[1], 16, 16], "▲ centred at the top");
    assert_eq!(arrows[3].rect, [v[0] + (v[2] - 16) / 2, v[1] + v[3] - 16, 16, 16], "▼ at the bottom");
    // Total: the picture, 24 brackets, the hint, 4 arrows.
    assert_eq!(elems.len(), 30);
    // Sister: ▲ held lights the top arrow, a shifted bracket is not on the disc page.
    let held = screenadjust_elems(0, 0x40, 0, &bind);
    let up: Vec<u32> = held.iter().filter(|e| e.ctrl == SCREENADJUST_BAR_ID).map(|e| e.colour).collect();
    assert_eq!(up, vec![0x6f6f_6fdf, 0x6f6f_6fdf, 0x6f6f_6fdf, 0x7f7f_7fff], "◀ ▶ dim, ▼ dim, ▲ bright");
    assert!(!pg.controls.iter().any(|c| (c.x, c.y, c.w, c.h) == (84, 73, 4, 64)));
}

#[test]
fn screen_adjust_keys_follow_the_two_scrolls() {
    let mut fe = FrontEnd::new();
    // The boot flow reaches the front-end runner first (the page ticks run from there).
    for _ in 0..8 { fe.tick(); }
    fe.enter_page_for_test(Page::ScreenAdjust);
    fe.screen_adjust_open();
    let _ = fe.drain_sfx().count();
    // Each scroll takes its own axis at once: ▲ / ▼ the vertical (Y ∓ 4), ◀ / ▶ the horizontal
    // (X ∓ 12) [M Scroll_Update 0x221b20:50-140 → C_SBSCREEN 0x217b08:12-19 → psiAdjustScreenPos].
    assert!(fe.action(Action::Up));
    assert_eq!((fe.screen_x, fe.screen_y, fe.screen_adjust_held), (0, -4, [0, 0x40]));
    assert!(fe.action(Action::Left));
    assert_eq!((fe.screen_x, fe.screen_y, fe.screen_adjust_held), (-12, -4, [0x40, 0x40]));
    assert!(fe.action(Action::Right));
    assert!(fe.action(Action::Right));
    assert!(fe.action(Action::Down));
    assert_eq!((fe.screen_x, fe.screen_y, fe.screen_adjust_held), (12, 0, [0x80, 0x80]));
    assert_eq!(fe.cursor, 0, "no cursor moved");
    let sounds: Vec<u8> = fe.drain_sfx().map(|e| e.sound).collect();
    assert_eq!(sounds, vec![3, 3, 4, 4, 4], "MENU_SOUND 3 dec / 4 inc [M Scroll_Update :188/:204]");
    // The lit arrow runs out.
    for _ in 0..menu::SCREEN_ADJUST_HELD_TICKS { fe.tick(); }
    assert_eq!(fe.screen_adjust_held, [0, 0]);
    // Sister: the clamp still applies (X ±432, Y ±48 [M 0x10b890]).
    for _ in 0..20 { fe.action(Action::Up); }
    assert_eq!(fe.screen_y, -48);
}

#[test]
fn parisenum_elements_follow_pages_0x4000004a_and_0x4000002a() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = front_end(&iso);
    let paris = page(&md, menu::PAGE_PARISENUM);
    let bx = page(&md, 0x4000_002a);
    // The plate: P_PARISENUM's full-screen texture label.
    let plate = paris.controls.iter().find(|c| c.b == 5 && c.texture.is_some() && c.w == 640).expect("the plate");
    let only = parisenum_elems(false, None);
    assert_eq!(only.len(), 1);
    assert_eq!((only[0].rect, only[0].colour), (rect_of(plate), msg(plate, 0x1b).unwrap().0));
    assert_eq!(only[0].what, ElemWhat::Tex { tex: plate.texture.unwrap(), uv: uv_of(plate) });
    // The two 0x1000022c labels are authored hidden — not drawn.
    let hidden: Vec<&ControlDef> = paris.controls.iter().filter(|c| c.id == 0x1000_022c).collect();
    assert_eq!(hidden.len(), 2);
    assert!(hidden.iter().all(|c| !authored_visible(c)));
    // The box page: the bar, the button (type 1, SET 0xe), the memo (type 200, SET 2); the list
    // 0x100000fc and the hint are what PS2_DoCardFlow hides on a buttonless message.
    let bar = bx.controls.iter().find(|c| c.id == CARDBOX_BAR.0).unwrap();
    let btn = bx.controls.iter().find(|c| c.id == CARDBOX_BUTTON.0).unwrap();
    let memo = bx.controls.iter().find(|c| c.id == CARDBOX_MEMO.0).unwrap();
    assert_eq!((bar.x as i32, bar.y as i32, bar.w as i32, bar.h as i32), (CARDBOX_BAR.1, CARDBOX_BAR.2, CARDBOX_BAR.3, CARDBOX_BAR.4));
    assert_eq!((btn.b, btn.f2, btn.x as i32, btn.y as i32, btn.w as i32, btn.h as i32), (1, 0xe, CARDBOX_BUTTON.1, CARDBOX_BUTTON.2, CARDBOX_BUTTON.3, CARDBOX_BUTTON.4));
    assert_eq!((memo.b, memo.f2, memo.x as i32, memo.y as i32, memo.w as i32, memo.h as i32), (200, 2, CARDBOX_MEMO.1, CARDBOX_MEMO.2, CARDBOX_MEMO.3, CARDBOX_MEMO.4));
    assert_eq!(msg(memo, 0x1b).unwrap().0, CARDBOX_TEXT);
    assert_eq!(msg(btn, 0x1b).unwrap().0, 0, "the button's own label is colour 0 (invisible)");
    let kf = &bx.controls[0].scripts[0].keyframes[0];
    assert_eq!(kf.msgs[0].a, 0x1000_00fc, "the script focuses the list → the button is unfocused (mask 0x10)");
    // The geometry [M 0x217bd8:186-200]: 3 lines → h 123, y (448 − 123)/2 = 162; memo y 172.
    let (b, m) = cardbox_rects(3);
    assert_eq!(b, [40, 162, 432, 123]);
    assert_eq!(m[1], 172);
    assert_eq!((m[0], m[2]), (rect_of(memo)[0], rect_of(memo)[2]));
    let view = CardBoxView { lines: vec![("A".into(), 100.0), ("BB".into(), 200.0), ("C".into(), 50.0)], prev_lines: 3, stride: 20.0 };
    let e = parisenum_elems(true, Some(&view));
    assert_eq!(e[1].what, ElemWhat::Tex { tex: ATLAS_BUTTONS, uv: uv_of(bar) });
    assert_eq!(e[1].rect, rect_of(bar));
    let frame: Vec<&PageElem> = e.iter().filter(|e| e.ctrl == CARDBOX_BUTTON.0).collect();
    assert_eq!(frame.len(), 9);
    assert_eq!(frame[0].rect, [40, 162, 16, 16]);
    let pens: Vec<[i32; 2]> = e.iter().filter(|e| matches!(e.what, ElemWhat::Pen { .. })).map(|e| [e.rect[0], e.rect[1]]).collect();
    // x = memo.x + (w − extent)/2 + 2; baseline = memo.y + (k + 1)·stride.
    assert_eq!(pens, vec![[m[0] + (m[2] - 100) / 2 + 2, 192], [m[0] + (m[2] - 200) / 2 + 2, 212], [m[0] + (m[2] - 50) / 2 + 2, 232]]);
    // Sister: the first message frame sizes the box for 0 lines (h 30).
    let first = parisenum_elems(true, Some(&CardBoxView { prev_lines: 0, ..view }));
    let f0 = first.iter().find(|e| e.ctrl == CARDBOX_BUTTON.0).unwrap();
    assert_eq!(f0.rect[1], ((448 - 30) / 2) as i32);
}

#[test]
fn the_card_box_opens_with_the_flow_message_and_closes_with_the_flow() {
    let mut fe = FrontEnd::new();
    assert!(fe.action(Action::Confirm), "START → P_PARISENUM");
    assert_eq!(fe.page, Page::ParisEnum);
    assert!(!fe.paris_dialog);
    let mut saw_box = false;
    let mut ticks = 0;
    while fe.page == Page::ParisEnum && ticks < 64 {
        fe.tick();
        ticks += 1;
        if fe.paris_card.flow_msg == 0xb { assert!(fe.paris_dialog, "the 0xb message opens the box [M 0x217bd8:150-158]"); saw_box = true; }
    }
    assert!(saw_box, "the walk's GetDir step raises message 0xb [M 0x110188:37]");
    assert!(fe.paris_msg_ticks >= 1);
    assert_eq!(fe.page, Page::EstHero);
    assert!(!fe.paris_dialog, "the flow's end closes the box");
}

#[test]
fn cheatmedal_elements_follow_the_mission_page_0x4000004c() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = mission(&iso);
    let pg = page(&md, menu::PAGE_CHEAT_MEDAL);
    // The page is the root window plus the one list; the script focuses the list.
    assert_eq!(pg.controls.len(), 2);
    let l = &pg.controls[1];
    assert_eq!((l.id, l.b, l.f2), (CHEATMEDAL_LIST.0, 6, 3), "the list, SET 3");
    assert_eq!((l.x as i32, l.y as i32, l.w as i32, l.h as i32), (CHEATMEDAL_LIST.1, CHEATMEDAL_LIST.2, CHEATMEDAL_LIST.3, CHEATMEDAL_LIST.4));
    assert_eq!(msg(l, 0x28), Some((0, 100)), "column 0 at 100%");
    assert!(!l.msgs.iter().any(|m| m.0 == 0x10 || m.0 == 0x11), "no rows in the record");
    assert_eq!(pg.controls[0].scripts[0].keyframes[0].msgs[0].a, CHEATMEDAL_LIST.0);
    // The front end has no such page.
    assert!(front_end(&iso).page(menu::PAGE_CHEAT_MEDAL).is_none());
    // The elements: the frame over w − 16 and the scroll's ▲ ▼ at x + w − 16, focused.
    let r = rect_of(l);
    let e = cheatmedal_elems(0);
    let frame: Vec<&PageElem> = e.iter().filter(|e| e.rect[0] < r[0] + r[2] - 16).collect();
    assert_eq!(frame.len(), 9);
    assert_eq!(frame[0].rect, [r[0], r[1], 2, 2]);
    assert_eq!(frame[3].rect, [r[0] + r[2] - 16 - 2, r[1] + r[3] - 2, 2, 2]);
    let arrows: Vec<&PageElem> = e.iter().filter(|e| e.rect[0] >= r[0] + r[2] - 16).collect();
    assert_eq!(arrows.iter().map(|e| (e.rect, e.colour)).collect::<Vec<_>>(), vec![
        ([r[0] + r[2] - 16, r[1], 16, 16], 0x6f6f_6fdf),
        ([r[0] + r[2] - 16, r[1] + r[3] - 16, 16, 16], 0x6f6f_6fdf),
    ]);
    // Sister: an unfocused list would draw the dimmest arrows.
    let un = component_elems(l.id, &SET3_SCROLL_V, ATLAS_BUTTONS, 0x10, [r[0] + r[2] - 16, r[1], 16, r[3]], 0);
    assert!(un.iter().all(|e| e.colour == 0x6060_60c0));
}
