//! The lens-flare pane's sprite table on the disc: `LensFlareSprInfo` 0x2bd8d8 (pane 8 →
//! `HUD_UpdateLensFlarePane` 0x19d770) [D ACTION.ELF; the update maths are the port's
//! `nf_view::hud::flare_quads`]. Opt-in via NF_ISO.

use nf_formats::hud::HudTables;

fn tables() -> Option<HudTables> {
    let iso_path = std::env::var("NF_ISO").ok()?;
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open ISO");
    let elf = iso.read_file(iso.find("/ACTION.ELF").expect("ACTION.ELF on the disc")).expect("read ACTION.ELF");
    Some(HudTables::from_action_elf(&elf).expect("HudTables"))
}

/// [D/L ee_start_05.bin @0x2bd8d8] all seven sprites share the update's colour shape (white,
/// alpha 0 — the update rewrites it with the ramped alpha), the hidden template layer −1 and
/// x=y=w=h=0 (position/size come from the update); the per-sprite texture, uv extent and flags:
/// 0x0d00 (additive + tile-mirror + centred) on the four big quads, 0x0900 (additive + centred)
/// on sprites 2/5/6.
#[test]
fn lens_flare_table_rows() {
    let Some(t) = tables() else { eprintln!("NF_ISO not set - skipping"); return };
    assert_eq!(t.flare.len(), 7);
    let want: [(u32, i16, u16, [i16; 4], u32); 7] = [
        (0xffff_ff00, -1, 0x0d00, [0, 0, 31, 31], 0x0300_004f),
        (0xffff_ff00, -1, 0x0d00, [0, 0, 31, 31], 0x0300_0050),
        (0xffff_ff00, -1, 0x0900, [0, 0, 0, 0], 0x0300_0051),
        (0xffff_ff00, -1, 0x0d00, [0, 0, 63, 63], 0x0300_0052),
        (0xffff_ff00, -1, 0x0d00, [0, 0, 31, 31], 0x0300_0050),
        (0xffff_ff00, -1, 0x0900, [0, 0, 0, 0], 0x0300_0051),
        (0xffff_ff00, -1, 0x0900, [0, 0, 0, 0], 0x0300_0082),
    ];
    for (k, sp) in t.flare.iter().enumerate() {
        assert_eq!((sp.colour, sp.layer, sp.flags, sp.uv, sp.texture), want[k], "flare sprite {k}");
        assert_eq!((sp.x, sp.y, sp.w, sp.h), (0, 0, 0, 0), "flare sprite {k} geometry is the update's");
    }
}
