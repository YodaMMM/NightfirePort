//! The goggles' two HUD panes on the disc: `NightSightSprInfo` 0x2bd7b0 (pane 7 →
//! `HUD_UpdateNightSightPane` 0x19d218) and `XraySprInfo` 0x2bddb0 (pane 15 →
//! `HUD_UpdateXRayPane` 0x1a1410) [D ACTION.ELF]. Docs: docs/systems/goggles.md. Opt-in via NF_ISO.

use nf_formats::hud::{HudTables, SpriteInfo};

fn tables() -> Option<HudTables> {
    let iso_path = std::env::var("NF_ISO").ok()?;
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open ISO");
    let elf = iso.read_file(iso.find("/ACTION.ELF").expect("ACTION.ELF on the disc")).expect("read ACTION.ELF");
    Some(HudTables::from_action_elf(&elf).expect("HudTables"))
}

/// (colour, layer, flags, x, y, w, h, uv, texture) — numbers only.
fn row(s: &SpriteInfo) -> (u32, i16, u16, i16, i16, i16, i16, [i16; 4], u32) {
    (s.colour, s.layer, s.flags, s.x, s.y, s.w, s.h, s.uv, s.texture)
}

/// [D] the night pane: the green noise (additive), the white add and the magenta subtract over
/// the whole 512×448 screen, then the battery bar's back, bar and icon at (128, 48).
#[test]
fn night_sight_pane_rows() {
    let Some(t) = tables() else { eprintln!("NF_ISO not set - skipping"); return };
    assert_eq!(t.nightsight.len(), 6);
    let want = [
        (0x007f_0020, 38, 0x0100, 0, 0, 512, 448, [0, 0, 127, 255], 0x0300_004a),
        (0xffff_ff7f, 41, 0x0100, 0, 0, 512, 448, [0, 0, 0, 0], 0x0300_0002),
        (0xff00_ffff, 40, 0x0200, 0, 0, 512, 448, [0, 0, 0, 0], 0x0300_0002),
        (0x7f7f_7fff, 30, 0x0000, 128, 48, 256, 16, [0, 0, 256, 16], 0x0300_017f),
        (0x007f_00ff, 31, 0x0100, 128, 48, 256, 16, [0, 16, 256, 16], 0x0300_017f),
        (0x007f_00ff, 31, 0x0100, 108, 48, 0, 0, [0, 0, 0, 0], 0x0300_017e),
    ];
    for (k, w) in want.iter().enumerate() { assert_eq!(row(&t.nightsight[k]), *w, "NightSightSprInfo[{k}]"); }
    // Sprite 0's v extent is 255 — NOT the jitter range: the pane rolls v under the runtime
    // +0x2a word, the texture's height (64), not the template's dv [M 0x1ca3d8, 0x1de3d8].
    assert_eq!(t.nightsight[0].uv[3], 255);
    // Sister (can fail): sprite 2 is the SUBTRACTIVE one (0x200), not additive.
    assert_ne!(t.nightsight[2].flags & 0x200, 0);
    assert_eq!(t.nightsight[1].flags & 0x200, 0);
}

/// [D] the x-ray pane: the white noise (additive), the blue add, then the bar back, bar, icon.
#[test]
fn xray_pane_rows() {
    let Some(t) = tables() else { eprintln!("NF_ISO not set - skipping"); return };
    assert_eq!(t.xray.len(), 5);
    let want = [
        (0x007f_0020, 38, 0x0100, 0, 0, 512, 448, [0, 0, 511, 511], 0x0300_0098),
        (0x0000_ffff, 40, 0x0100, 0, 0, 512, 448, [0, 0, 0, 0], 0x0300_0002),
        (0x7f7f_7fff, 30, 0x0000, 128, 48, 256, 16, [0, 0, 256, 16], 0x0300_017f),
        (0x7f7f_7fff, 31, 0x0100, 128, 48, 256, 16, [0, 16, 256, 16], 0x0300_017f),
        (0x7f7f_7fff, 31, 0x0100, 108, 48, 0, 0, [0, 0, 0, 0], 0x0300_017e),
    ];
    for (k, w) in want.iter().enumerate() { assert_eq!(row(&t.xray[k]), *w, "XraySprInfo[{k}]"); }
    // Sister (can fail): the x-ray pane has no subtractive sprite (blue is added, nothing removed).
    assert!(t.xray.iter().all(|s| s.flags & 0x200 == 0));
}
