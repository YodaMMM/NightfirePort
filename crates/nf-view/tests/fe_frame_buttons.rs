//! The front end's type-1 buttons: what their sprite sets draw, pinned against the disc's own
//! CONTROL and SET records (07000048.bin entry 08000002; the mission archives' 08000001) and the
//! button's draw path [M Button_Init 0x1efc90 (handle = NEW_CONTROL+0x14 << 16 | 0),
//! Button_Update 0x1ef948 (the `+0x7a & 1` hide arm, the 0x10/0x20/0x80/0x40 state masks),
//! Component_SetupInstance 0x1f0950 (a COMPONENT with count 0 returns before any sprite),
//! MenuManager_Create 0x21b908:262-266 (the CONTROL record's f2 word lands in NEW_CONTROL+0x14)].
//!
//! 1. The sets the buttons name (2, 7, 8, 0xc, 0xe) are the disc's COMPONENT 0 records, and
//!    SET 2's is empty. Sister: a one-word change is caught.
//! 2. `FRAME_BUTTONS` is exactly the visible type-1 controls with a non-empty set on the six
//!    pages the port draws them on. Sister: a moved rect is not on the disc.
//! 3. The 007 Bonus page's button 0x10000165 (SET 2) draws nothing — no element, no quad at its
//!    rect. Sister: the same rect under SET 7 draws the 9-slice.
//! 4. A hidden button (msg 0x2b bit 0) draws nothing; the debriefing shows one column box per
//!    participant. Sisters: the visible flag draws; a fifth participant does not add a box.
//! 5. The Records overlay draws the band from SET 0xc's atlas and the two boxes from SET 7's.
//!
//! The disc tests are opt-in through NF_ISO.

use nf_formats::menudef::{ControlDef, MenuDef, SpriteDef};
use nf_view::fe_pages::*;
use nf_view::menu::{self, FrontEnd, Page};

fn iso_path() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

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

fn front_end(iso: &str) -> MenuDef {
    def(iso, "07000048.BIN", 0x0800_0002)
}

fn mission(iso: &str) -> MenuDef {
    def(iso, "0700000A.BIN", 0x0800_0001)
}

fn msg(c: &ControlDef, id: u8) -> Option<(u32, u32)> {
    c.msgs.iter().rev().find(|m| m.0 == id).map(|m| (m.1, m.2))
}

/// The CONTROL record's first byte is the widget type (the 0x21b908 placement arms test it for
/// 5 / 0xa / 0xd); `ControlDef::b` holds it.
fn is_button(c: &ControlDef) -> bool {
    c.b == 1
}

fn hidden(c: &ControlDef) -> bool {
    msg(c, 0x2b).is_some_and(|(v, _)| v & 1 != 0)
}

fn same(disc: &[SpriteDef], ours: &[CompInst]) -> bool {
    disc.len() == ours.len()
        && disc.iter().zip(ours).all(|(d, o)| d.w == o.w && d.d1 == o.colour && d.d2 == o.flags && d.f1 == o.wf && d.f2 == o.hf)
}

fn comp0(md: &MenuDef, set: u16) -> &nf_formats::menudef::ComponentDef {
    &md.sets.iter().find(|s| s.id == set).unwrap_or_else(|| panic!("SET {set:x}")).components[0]
}

#[test]
fn button_sets_are_the_discs_component_zero() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = front_end(&iso);
    // Every set a front-end button names is one the port knows, with the disc's instances.
    let mut sets: Vec<u16> = md.pages.iter().flat_map(|p| p.controls.iter()).filter(|c| is_button(c)).map(|c| c.f2).collect();
    sets.sort_unstable();
    sets.dedup();
    assert_eq!(sets, vec![2, 7, 8, 0xc, 0xe], "the sets the front end's buttons name");
    for &s in &sets {
        let (insts, tex) = button_component(s).unwrap_or_else(|| panic!("SET {s:x} unknown to the port"));
        let c = comp0(&md, s);
        assert!(same(&c.sprites, insts), "SET {s:x} COMPONENT 0's instances");
        if !insts.is_empty() { assert_eq!(c.hash, tex, "SET {s:x}'s atlas"); }
    }
    assert_eq!(comp0(&md, 2).count, 0, "SET 2's COMPONENT 0 has no instances — Component_SetupInstance returns at count 0");
    // The mission defs' buttons: SET 2 (the pause tabs, the hint-row buttons) and 0xb (the hint box).
    let m = mission(&iso);
    let mut msets: Vec<u16> = m.pages.iter().flat_map(|p| p.controls.iter()).filter(|c| is_button(c)).map(|c| c.f2).collect();
    msets.sort_unstable();
    msets.dedup();
    assert_eq!(msets, vec![2, 0xb]);
    assert_eq!(comp0(&m, 2).count, 0);
    // Sisters: a one-word change is caught; SET 7 is not SET 0xe.
    let mut bad = SET7_BUTTON;
    bad[8].w[4] = 125;
    assert!(!same(&comp0(&md, 7).sprites, &bad));
    assert!(!same(&comp0(&md, 7).sprites, &SETE_BUTTON));
    assert!(button_component(3).is_none(), "a set no button names");
}

#[test]
fn frame_buttons_are_the_pages_visible_framed_buttons() {
    let Some(iso) = iso_path() else { eprintln!("NF_ISO not set — skipping"); return };
    let md = front_end(&iso);
    let pages: Vec<u32> = { let mut v: Vec<u32> = FRAME_BUTTONS.iter().map(|b| b.0).collect(); v.dedup(); v };
    assert_eq!(pages, vec![0x4000_003a, 0x4000_003b, 0x4000_0020, 0x4000_0022, 0x4000_0049, 0x4000_0033]);
    let mut want: Vec<(u32, u32, u16, i32, i32, i32, i32)> = Vec::new();
    for &pid in &pages {
        let p = md.page(pid).unwrap();
        for c in p.controls.iter().filter(|c| is_button(c) && !hidden(c)) {
            if button_component(c.f2).is_some_and(|(i, _)| !i.is_empty()) {
                want.push((pid, c.id, c.f2, c.x as i32, c.y as i32, c.w as i32, c.h as i32));
            }
        }
    }
    let r = DEBRIEF_RESULT_BOX;
    let mut ours: Vec<_> = FRAME_BUTTONS.to_vec();
    ours.push((0x4000_0033, r.0, r.1, r.2, r.3, r.4, r.5));
    let key = |b: &(u32, u32, u16, i32, i32, i32, i32)| (b.0, b.4, b.3, b.1);
    want.sort_by_key(key);
    ours.sort_by_key(key);
    assert_eq!(ours, want, "the table is every visible framed button on those pages");
    // Every one's label draws nothing (colour 0 or alpha 0) except the Controls Y-axis button.
    for p in &pages {
        for c in md.page(*p).unwrap().controls.iter().filter(|c| is_button(c) && c.f2 != 2) {
            let col = msg(c, 0x1b).map(|m| m.0).unwrap_or(0);
            if c.id == 0x1000_00c9 { assert_eq!(col, 0x645a_49ff); } else { assert_eq!(col & 0xff, 0, "{:08x}: an invisible label", c.id); }
        }
    }
    // Sister: a moved rect is not a disc control.
    let moved = (0x4000_003a, 0x1000_0001, 7u16, 68, 286, 505, 60);
    assert!(!want.contains(&moved));
}

#[test]
fn the_bonus_button_draws_no_frame() {
    // The element list has no 0x10000165 entry.
    let bind = |id: u32| format!("<{id:x}>");
    let elems = nfbonus_elems(None, &bind);
    assert!(elems.iter().all(|e| e.ctrl != 0x1000_0165), "no element for the SET-2 button");
    let r = control_rect(0x1000_0165, 1, 62, 372, 61, 59);
    assert!(button_elems(0x1000_0165, 2, r, 4, 0x80, 0).is_empty(), "SET 2 draws nothing, focused or not");
    // Sister: the same rect under SET 7 draws its nine instances.
    assert_eq!(button_elems(0x1000_0165, 7, r, 4, 0x80, 0).len(), 9);
    // The overlay: no untextured quad (the old framed-box stand-in) over the button's rect.
    let font = test_font();
    let tex = textures();
    let mut fe = FrontEnd::new();
    fe.page = Page::NfBonus;
    let frame = menu::menu_overlay(&fe, &font, &font, None, &tex, 4.0 / 3.0);
    let solid = quads(&frame, 0);
    let inside = |q: &(f32, f32, f32, f32)| q.0 >= r[0] as f32 - 1.0 && q.2 <= (r[0] + r[2]) as f32 + 1.0 && q.1 >= r[1] as f32 - 1.0 && q.3 <= (r[1] + r[3]) as f32 + 1.0 && q.2 - q.0 > 20.0;
    assert!(!solid.iter().any(inside), "a plate over 0x10000165's rect {r:?}: {:?}", solid.iter().filter(|q| inside(q)).collect::<Vec<_>>());
}

#[test]
fn hidden_buttons_and_debrief_columns() {
    let r = [100, 100, 80, 60];
    assert!(button_elems(1, 7, r, 1, BUTTON_IDLE, 0).is_empty(), "msg 0x2b bit 0 hides the sprite set");
    assert!(button_elems(1, 7, r, 5, BUTTON_IDLE, 0).is_empty());
    // Sister: 6 / 4 (the authored visible values) draw.
    assert_eq!(button_elems(1, 7, r, 6, BUTTON_IDLE, 0).len(), 9);
    assert_eq!(button_elems(1, 7, r, 4, BUTTON_IDLE, 0).len(), 9);
    // SET 0xc's first instance is focus-only (0x20).
    assert_eq!(button_elems(1, 0xc, r, 6, BUTTON_IDLE, 0).len(), 5);
    assert_eq!(button_elems(1, 0xc, r, 6, 0x20, 0).len(), 6);
    // The debriefing: 9 per shown column plus the result box.
    assert_eq!(debrief_frame_elems(0).len(), 9);
    assert_eq!(debrief_frame_elems(2).len(), 27);
    assert_eq!(debrief_frame_elems(4).len(), 45);
    // Sister: only four columns exist.
    assert_eq!(debrief_frame_elems(5).len(), 45);
    // Column 1's box sits at its authored x 280 → runtime 225 [M FixupResolution].
    let e = debrief_frame_elems(2);
    assert_eq!(e[9].rect[0], control_rect(0x1000_015d, 1, 280, 94, 96, 253)[0]);
}

#[test]
fn records_draws_its_sprite_sets() {
    let font = test_font();
    let tex = textures();
    let mut fe = FrontEnd::new();
    fe.page = Page::Records;
    let frame = menu::menu_overlay(&fe, &font, &font, None, &tex, 4.0 / 3.0);
    let slot = |h: u32| tex.iter().position(|(t, _)| *t == h).unwrap() + 1;
    let band = quads(&frame, slot(ATLAS_SETC));
    assert_eq!(band.len(), 5, "SET 0xc unfocused: five instances");
    let b = control_rect(0x1000_0001, 1, 68, 120, 505, 34);
    let x0 = band.iter().map(|q| q.0).fold(f32::MAX, f32::min);
    let x1 = band.iter().map(|q| q.2).fold(f32::MIN, f32::max);
    assert!((x0 - b[0] as f32).abs() < 0.01 && (x1 - (b[0] + b[2]) as f32).abs() < 0.01, "the band spans its rect {b:?}: {x0}..{x1}");
    // The two SET 7 boxes: nine quads each, from the frame atlas (which the hint glyphs share —
    // count only quads at least 4 wide that sit inside the boxes).
    let boxes = [control_rect(0x1000_0001, 1, 68, 162, 505, 113), control_rect(0x1000_0001, 1, 68, 285, 505, 60)];
    let f = quads(&frame, slot(ATLAS_FRAMES));
    for bx in boxes {
        let n = f.iter().filter(|q| q.0 >= bx[0] as f32 - 0.01 && q.2 <= (bx[0] + bx[2]) as f32 + 0.01 && q.1 >= bx[1] as f32 - 0.01 && q.3 <= (bx[1] + bx[3]) as f32 + 0.01).count();
        assert_eq!(n, 9, "SET 7 box {bx:?}");
    }
    // Sister: no untextured framed box remains on the page.
    let solid = quads(&frame, 0);
    assert!(!solid.iter().any(|q| (q.0 - boxes[0][0] as f32).abs() < 1.0 && (q.2 - (boxes[0][0] + boxes[0][2]) as f32).abs() < 1.0 && q.3 - q.1 > 50.0), "the old plate stand-in");
}

fn test_font() -> nf_formats::hud::Font {
    let glyph = |code: u16| nf_formats::hud::Glyph { u: 0, v: 0, w: 8, h: 12, yoff: 0, pre: 0, post: 8, code, kerns: vec![] };
    nf_formats::hud::Font { first: 33, last: 126, gap: 5.0, space: 3.0, glyphs: (33..=126).map(glyph).collect() }
}

fn texture(hash: u32) -> nf_view::scene::TextureImage {
    let (w, h) = match hash { 0x0300_0042 => (64, 64), 0x0300_014c => (256, 256), 0x0300_0075 => (256, 128), _ => (256, 256) };
    nf_view::scene::TextureImage { key: (0, hash), width: w, height: h, rgba: vec![255; (w * h * 4) as usize], more: vec![], rate: 1 }
}

fn textures() -> Vec<(u32, nf_view::scene::TextureImage)> {
    let mut v: Vec<u32> = menu::MENU_TEXTURES.to_vec();
    for &t in menu::mp_textures() { if !v.contains(&t) { v.push(t); } }
    v.into_iter().map(|h| (h, texture(h))).collect()
}

/// The quads of one texture slot (0 = untextured), as (x0, y0, x1, y1) in 512×448 units.
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
