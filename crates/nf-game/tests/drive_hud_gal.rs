//! The driving HUD layout from the real `hudNTSC.gal` (docs/systems/driving-hud.md): the slots
//! `FUN_0023ae30` fills and its in-place moves (y −10; x ±78 in widescreen) [M], over the disc
//! gallery [D]. Opt-in via NF_ISO (full image; a slim image skips).

use nf_formats::gal::{Gal, GB};
use nf_formats::iso9660::IsoImage;
use nf_formats::loc::Loc;
use nf_formats::sfn::Font;
use nf_game::driving_hud_draw::{text_glyphs, DriveHudDraw, HudDraw, HudInput, HudLayout, HudStrings, SCALE_1_1, SCALE_1_2, SCALE_1_4};

fn entry(viv: &str, name: &str) -> Option<Vec<u8>> {
    let iso = IsoImage::open(std::env::var_os("NF_ISO")?).ok()?;
    let e = iso.find(viv)?;
    let g = nf_formats::disc::read_bigf(&iso, e).ok()?;
    let x = g.entries.iter().find(|x| x.name.eq_ignore_ascii_case(name))?;
    let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) { return None; }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).ok()? } else { raw })
}

fn hud_gal() -> Option<Vec<u8>> { entry("/DRIVING/MIS01.VIV", r"data\gfx\data\hudNTSC.gal") }

#[test]
fn the_layout_lands_where_fun_0023ae30_moves_it() {
    let Some(data) = hud_gal() else { eprintln!("skip: NF_ISO / full image not available"); return };
    let g = Gal::parse(&data).unwrap();
    let l = HudLayout::from_gal(&g, false, 448.0).unwrap();
    // Gallery keys [D] minus 10 in y [M 0x23c004-0x23c0b4].
    assert_eq!(l.weapon.pos, [605.0, 410.0], "weapon key (605, 420)");
    assert_eq!(l.ammo.pos, [605.0, 390.0], "ammo key (605, 400)");
    assert_eq!(l.iris[0].pos, [46.0, 351.0], "iris1 key (46, 361)");
    assert_eq!(l.arms[0].name, "a007");
    assert_eq!(l.arms[0].pos, [53.0, 377.0], "a007 key (53, 387)");
    assert_eq!(l.health_border.pos, [68.0, 394.0], "healthBorder key (68, 404)");
    assert_eq!(l.health_border.size, [43.0, 43.0]);
    assert_eq!(l.gadgets[0].pos, [540.0, 340.0], "gadget0 key (540, 350)");
    assert_eq!(l.stylin.pos, [510.0, 40.0], "stylin is not in the −10 list");
    assert_eq!(l.stylin.size, [122.0 * (1300.0f32 * 0.001), 41.0 * (1300.0f32 * 0.001)], "stylin carries scale 1300");
    assert_eq!(l.msg_txt.pos, [260.0, 54.0]);
    assert_eq!(l.centre, [320.0, 224.0]);
    // Widescreen: x ±78 [M 0x23c2c0-0x23c398].
    let w = HudLayout::from_gal(&g, true, 448.0).unwrap();
    assert_eq!(w.weapon.pos, [683.0, 410.0]);
    assert_eq!(w.iris[0].pos, [-32.0, 351.0]);
    assert_eq!(w.gadgets[5].pos, [618.0, 340.0]);
    assert_eq!(w.health_border.pos, [-10.0, 394.0]);
    assert_eq!(w.stylin.pos, [588.0, 40.0]);
    // Sister: the layout differs from the raw keys (the moves happened).
    let raw_y = g.anim("hud_base").unwrap().element("weapon").unwrap().hud_key().unwrap().pos()[1];
    assert_ne!(raw_y, l.weapon.pos[1]);
}

/// The meter bitmaps on the disc: no repeat counts anywhere, no mirror bits except
/// `healthBorder` = 3 (a quarter image drawn four ways around its key) [D]; so a full-health
/// frame is 7 dim + 7 full iris, 8 armour pieces, and 4 border quads [M 0x23da30 + 0x238d40].
#[test]
fn the_meter_bitmaps_and_the_mirrored_border() {
    let Some(data) = hud_gal() else { eprintln!("skip: NF_ISO / full image not available"); return };
    let g = Gal::parse(&data).unwrap();
    let base = g.anim("hud_base").unwrap();
    let names: Vec<String> = (1..=7).map(|i| format!("iris{i}")).chain((1..=7).map(|i| format!("arm{i}")))
        .chain((0..6).map(|i| format!("gadget{i}"))).chain(["a007".into(), "healthBorder".into()]).collect();
    for n in &names {
        let e = base.element(n).unwrap();
        assert_eq!(e.kind, GB, "{n} is a bitmap");
        let k = e.hud_key().unwrap();
        assert_eq!(k.mirror(), if n == "healthBorder" { 3 } else { 0 }, "{n} mirror");
        assert_eq!(k.repeats(), [0; 4], "{n} repeats");
    }
    let l = HudLayout::from_gal(&g, false, 448.0).unwrap();
    let i = HudInput { dt: 1.0 / 60.0, ticks: 1, health: 100.0, health_max: 100.0, armour: 50.0, armour_max: 50.0, weapon: None,
        timer: None, health_always: false, grunt_on_hit: false, suppressed: false, paused: false };
    let font = Font::parse(&entry("/DRIVING/MIS01.VIV", r"data\loading\loading.sfn").unwrap()).unwrap();
    let loc = Loc::parse(&entry("/DRIVING/MISC.VIV", r"data\gfx\locale\IDL_ENG.loc").unwrap()).unwrap();
    let f = DriveHudDraw::new().frame(&l, &i, HudStrings { font: &font, loc: &loc });
    let quads: Vec<_> = f.draws.iter().filter_map(|d| match d { HudDraw::Sprite { element, x, y, .. } => Some((element.as_str(), *x, *y)), _ => None }).collect();
    assert_eq!(quads.len(), 7 + 7 + 8 + 4);
    assert_eq!(quads[0], ("iris1", 46.0, 351.0));
    assert_eq!(&quads[22..], &[("healthBorder", 68.0, 394.0), ("healthBorder", 25.0, 351.0), ("healthBorder", 25.0, 394.0), ("healthBorder", 68.0, 351.0)]);
}

/// Measured text on the disc font: widths and heights as DRIVING.ELF's `FUN_00237ca8` /
/// `FUN_00237ce0` compute them (`Font::measure` matches the game code bit for bit on every string
/// — crates/nf-formats/tests/sfn_loc_disc.rs runs the game's functions on the EE interpreter),
/// pinned here for the HUD's own strings at the scales the passes set [M].
#[test]
fn hud_text_metrics_on_the_disc_font() {
    let Some(fd) = entry("/DRIVING/MIS01.VIV", r"data\loading\loading.sfn") else { eprintln!("skip: NF_ISO / full image not available"); return };
    let font = Font::parse(&fd).unwrap();
    let loc = Loc::parse(&entry("/DRIVING/MISC.VIV", r"data\gfx\locale\IDL_ENG.loc").unwrap()).unwrap();
    let m = |t: &[u8], s: f32| { let e = font.measure(t, [s, s]); (e.w.to_bits(), e.h.to_bits()) };
    assert_eq!(m(b"00:00", SCALE_1_4), (0x428b_fffe, 0x4191_9999), "69.99998 × 18.2: the EE chop, not 70.0");
    assert_eq!(m(b"0", 1.0), (0x4100_0000, 0x4150_0000), "8 × 13");
    assert_eq!(loc.label(1611), b"Machine Gun");
    assert_eq!(m(loc.label(1611), 1.0), (0x42d8_0000, 0x4160_0000), "108 × 14");
    assert_eq!(m(loc.label(99999), SCALE_1_1), (0x4348_332d, 0x4176_6666), "OBJECTIVE COMPLETE 200.2 × 15.4");
    assert_eq!(m(loc.label(99998), SCALE_1_1), (0x4310_1996, 0x4176_6666), "NEW OBJECTIVE 144.1 × 15.4");
    assert_eq!(m(loc.label(519), SCALE_1_2), (0x431a_cccb, 0x4186_6666), "MISSION FAILED 154.8 × 16.8");
    // Sister: plain round-to-nearest arithmetic gives exactly 70 for the clock face.
    assert_ne!(f32::from_bits(0x428b_fffe), 70.0);

    // The message line on the real font: centred at 320 by that width.
    let g = Gal::parse(&hud_gal().unwrap()).unwrap();
    let l = HudLayout::from_gal(&g, false, 448.0).unwrap();
    let mut d = DriveHudDraw::new();
    d.message_show(2.0, 519, loc.label(519).len(), false);
    let i = HudInput { dt: 1.0 / 60.0, ticks: 1, health: 1.0, health_max: 1.0, armour: 0.0, armour_max: 1.0, weapon: None,
        timer: None, health_always: false, grunt_on_hit: false, suppressed: true, paused: false };
    let f = d.frame(&l, &i, HudStrings { font: &font, loc: &loc });
    let w = f32::from_bits(0x431a_cccb);
    let x = nf_formats::ps2float::add_noguard(320.0, -(w * 0.5), true);
    assert!(f.draws.iter().any(|t| matches!(t, HudDraw::Text { text, x: tx, y, .. } if text == b"MISSION FAILED" && *tx == x && *y == 380.0)), "{:?}", f.draws);
    let t = f.draws.iter().find(|t| matches!(t, HudDraw::Text { .. })).unwrap();
    let (shadow, main) = text_glyphs(t, &font, false, 448.0);
    assert_eq!((shadow.len(), main.len()), (14, 14), "one sprite per byte (the space too)");
    assert_eq!(main[0].xy[1] - shadow[0].xy[1], -1);
}
