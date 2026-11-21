//! The driving HUD layout from the real `hudNTSC.gal` (docs/systems/driving-hud.md): the slots
//! `FUN_0023ae30` fills and its in-place moves (y −10; x ±78 in widescreen) [M], over the disc
//! gallery [D]. Opt-in via NF_ISO (full image; a slim image skips).

use nf_formats::gal::{Gal, GB};
use nf_formats::iso9660::IsoImage;
use nf_game::driving_hud_draw::{DriveHudDraw, HudDraw, HudInput, HudLayout};

fn hud_gal() -> Option<Vec<u8>> {
    let iso = IsoImage::open(std::env::var_os("NF_ISO")?).ok()?;
    let e = iso.find("/DRIVING/MIS01.VIV")?;
    let g = nf_formats::disc::read_bigf(&iso, e).ok()?;
    let x = g.entries.iter().find(|x| x.name.eq_ignore_ascii_case(r"data\gfx\data\hudNTSC.gal"))?;
    let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) { return None; }
    Some(nf_formats::refpack::decompress(&raw).ok()?)
}

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
    let i = HudInput { dt: 1.0 / 60.0, health: 100.0, health_max: 100.0, armour: 50.0, armour_max: 50.0, weapon: None,
        timer: None, health_always: false, grunt_on_hit: false, suppressed: false };
    let f = DriveHudDraw::new().frame(&l, &i);
    let quads: Vec<_> = f.draws.iter().filter_map(|d| match d { HudDraw::Sprite { element, x, y, .. } => Some((element.as_str(), *x, *y)), _ => None }).collect();
    assert_eq!(quads.len(), 7 + 7 + 8 + 4);
    assert_eq!(quads[0], ("iris1", 46.0, 351.0));
    assert_eq!(&quads[22..], &[("healthBorder", 68.0, 394.0), ("healthBorder", 25.0, 351.0), ("healthBorder", 25.0, 394.0), ("healthBorder", 68.0, 351.0)]);
}
