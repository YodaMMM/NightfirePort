//! The keypad page's draw (page 0x40000021 of the mission menu defs, `Lock_Update` case 3's
//! `MenuManager_Create(0x80000003, 0x40000021)` [M 0x1ae3d8:242-243]) and the flag it turns on.
//!
//! Pinned here:
//! - the port's page table (`fe_pages::KEYPAD_PAGE`) and the keys' SET 9 / the frame's SET 0xb
//!   are the disc's, on every level that carries the page [D];
//! - the load turns `keypad_page_drawn` on, so a use press at a keypad on 0700000a / 07000012
//!   puts the page over the view and the page holds Bond;
//! - the frame's overlay carries the ten digit glyphs while the page stands and none once △
//!   closes it; the display reads the typed digits;
//! - only the focused key draws its highlight (`Button_Update` 0x1ef948's masks over SET 9).
//!
//! The headless renders of the page live under the audits' render-1108-keypad folder
//! (`nf-view <iso> <level> [--difficulty 3] --frame 240 --keypad 0[:digits] --shot …`).

use nf_formats::disc;
use nf_formats::iso9660::{IsoImage, SECTOR_SIZE};
use nf_game::player::Input;
use nf_view::fe_pages::{self, KeypadPart, KEYPAD_GLYPHS_TEX, KEYPAD_PAGE};

fn iso_path() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Every level's page 0x40000021 with its def (levels without the page are skipped).
fn pages() -> Vec<(String, nf_formats::menudef::MenuDef)> {
    let iso = IsoImage::open(&iso_path()).expect("open ISO");
    let (files, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");
    let mut names: Vec<String> = files.iter().map(|f| f.name.to_ascii_uppercase())
        .filter(|n| n.len() == 12 && n.starts_with("0700") && n.ends_with(".BIN")).collect();
    names.sort();
    names.dedup();
    let mut out = Vec::new();
    for name in names {
        let Some(fe) = files.iter().find(|f| f.name.eq_ignore_ascii_case(&name)) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).expect("read");
        let Ok(lb) = nf_formats::levelbin::LevelBin::parse(&data) else { continue };
        let Some(e) = lb.entries.iter().find(|e| e.hash() == 0x0800_0001) else { continue };
        let Ok(def) = nf_formats::menudef::MenuDef::parse(lb.data(&data, e), 0x8000_0003) else { continue };
        if def.page(0x4000_0021).is_some() { out.push((name, def)); }
    }
    out
}

fn msg(c: &nf_formats::menudef::ControlDef, id: u8) -> Option<(u32, u32)> {
    c.msgs.iter().find(|m| m.0 == id).map(|m| (m.1, m.2))
}

#[test]
fn the_page_table_is_the_discs() {
    let pages = pages();
    assert!(pages.len() >= 2, "the disc's keypad pages ({} found)", pages.len());
    for (name, def) in &pages {
        let page = def.page(0x4000_0021).unwrap();
        assert_eq!(page.controls.len(), 24, "{name}: 24 controls");
        assert_eq!(page.controls[0].b, 0xd, "{name}: [0] is the page window");
        for (i, &(id, b, x, y, w, h, layer, colour, part)) in KEYPAD_PAGE.iter().enumerate() {
            let c = &page.controls[i + 1];
            let at = format!("{name} control [{}]", i + 1);
            assert_eq!((c.id, c.b, c.x as i32, c.y as i32, c.w as i32, c.h as i32, c.f1), (id, b, x, y, w, h, layer), "{at}: id, type, rect, layer");
            assert_eq!(msg(c, 0x1b).map(|m| m.0).unwrap_or(0), colour, "{at}: colour (msg 0x1b)");
            assert_eq!(msg(c, 0x1c).map(|m| m.0).unwrap_or(0), colour, "{at}: colour (msg 0x1c)");
            match part {
                KeypadPart::Display => assert_eq!(c.id, 0x1000_0075, "{at}: the display label"),
                KeypadPart::Tex { tex, uv_a, uv_b } => {
                    assert_eq!(msg(c, 0x24).map(|m| m.0), Some(tex), "{at}: texture (msg 0x24)");
                    assert_eq!(msg(c, 0x2d), Some((uv_a, uv_b)), "{at}: uv (msg 0x2d)");
                }
                KeypadPart::Digit(k) => {
                    assert_eq!(c.f2, 9, "{at}: SET 9");
                    let r = nf_game::lock::KEYPAD_DIGIT_RECTS[k];
                    assert_eq!((c.x, c.y, c.w, c.h), r, "{at}: the input's digit rect");
                    assert_eq!(c.d1, nf_game::lock::KEYPAD_DIGIT_VALUES[k], "{at}: the digit");
                }
                KeypadPart::Frame => assert_eq!(c.f2, 0xb, "{at}: SET 0xb"),
            }
        }
        // The two sets the buttons draw with.
        let set = |id: u16| def.sets.iter().find(|s| s.id == id).unwrap_or_else(|| panic!("{name}: SET {id:#x}"));
        let comp0 = |id: u16| set(id).components[0].clone();
        for (setid, want) in [(9u16, &fe_pages::SET9_KEYPAD_KEY[..]), (0xb, &fe_pages::SETB_BUTTON[..])] {
            let c = comp0(setid);
            assert_eq!(c.hash, fe_pages::ATLAS_BUTTONS, "{name}: SET {setid:#x} atlas");
            let got: Vec<_> = c.sprites.iter().map(|s| (s.w, s.d1, s.d2, s.f1, s.f2)).collect();
            let exp: Vec<_> = want.iter().map(|s| (s.w, s.colour, s.flags, s.wf, s.hf)).collect();
            assert_eq!(got, exp, "{name}: SET {setid:#x} COMPONENT 0");
        }
    }
    eprintln!("{} levels' keypad pages match the table", pages.len());
}

#[test]
fn only_the_focused_key_shows_its_highlight() {
    for focus in 0..10 {
        for held in [false, true] {
            let elems = fe_pages::keypad_elems("****", focus, held);
            let red: Vec<_> = elems.iter().filter(|e| e.2.colour == 0x7f00_00ff).collect();
            assert_eq!(red.len(), 1, "focus {focus} held {held}: one highlight");
            let (x, y, w, h) = nf_game::lock::KEYPAD_DIGIT_RECTS[focus];
            let key = fe_pages::control_rect(0x1000_00ba, 1, x as i32, y as i32, w as i32, h as i32);
            let r = red[0].2.rect;
            assert!(r[0] >= key[0] && r[1] >= key[1] && r[0] + r[2] <= key[0] + key[2] && r[1] + r[3] <= key[1] + key[3] && r[2] > 0,
                "focus {focus}: the highlight {r:?} sits on the key {key:?}");
        }
    }
    // The glyphs draw over the keys and the frame behind everything.
    let elems = fe_pages::keypad_elems("12", 0, false);
    assert_eq!(elems.first().map(|e| e.0), Some(0x28), "the frame first");
    assert_eq!(elems.iter().filter(|e| matches!(e.2.what, fe_pages::ElemWhat::Tex { tex, .. } if tex == KEYPAD_GLYPHS_TEX)).count(), 10, "ten glyphs");
    assert!(elems.iter().any(|e| matches!(&e.2.what, fe_pages::ElemWhat::Text { s, .. } if s == "12")), "the display text");
}

/// The glyph texture's quads in this frame's HUD overlay.
fn glyph_quads(scene: &nf_view::scene::Scene) -> u32 {
    let Some(ti) = scene.hud_textures.iter().position(|(h, _)| *h == KEYPAD_GLYPHS_TEX) else { return 0 };
    let f = nf_view::hud::overlay(scene, 1280.0 / 720.0);
    f.runs.iter().filter(|r| r.0 == ti + 1).map(|r| r.3 / 6).sum()
}

fn the_page_draws_and_holds(mut scene: nf_view::scene::Scene, level: &str, digits: &str) {
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.keypad_page_drawn, "{level}: the load turns the page's input on");
    }
    assert!(scene.hud_textures.iter().any(|(h, _)| *h == KEYPAD_GLYPHS_TEX), "{level}: the glyph texture loads");
    assert_eq!(glyph_quads(&scene), 0, "{level}: no page before the press");
    nf_view::keypad_drive(scene.sim.as_mut().unwrap(), 0, digits).expect("the press arms the page");
    assert_eq!(glyph_quads(&scene), 10, "{level}: the page's ten digit glyphs draw");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.drones.keypad_display().as_deref(), Some(digits), "{level}: the display shows the typed digits");
        let from = sim.bond.pos;
        for _ in 0..30 { sim.step_bond(Input { forward: -1.0, ..Default::default() }, false, false, false); }
        assert_eq!(sim.bond.pos, from, "{level}: the page holds the pad [M 0x193600:80-87]");
        sim.step_bond(Input { keypad_back: true, ..Default::default() }, false, false, false);
        sim.step_bond(Input::default(), false, false, false);
        assert!(sim.drones.keypad.is_none(), "{level}: △ closes the page");
    }
    assert_eq!(glyph_quads(&scene), 0, "{level}: the closed page draws nothing");
}

#[test]
fn the_page_draws_on_0700000a() {
    let scene = nf_view::load(&iso_path(), "0700000a.bin", false).expect("load 0700000a");
    the_page_draws_and_holds(scene, "0700000a", "25");
}

#[test]
fn the_page_draws_on_07000012() {
    // Agent: the Operative load unlocks 07000012's keypads at create [M 0x1ae1f8 tail].
    let scene = nf_view::load_with_carry_difficulty(&iso_path(), "07000012.bin", false, None, 3).expect("load 07000012");
    the_page_draws_and_holds(scene, "07000012", "907");
}
