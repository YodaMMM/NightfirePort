//! agent-cable08 — the 07000008 zip line's drawn cable runs UP from the castle window to the
//! fitting on the tall wall, not down through the low building's roof.
//!
//! The game's law [M]: `parsemap_create_dynamic_objects` 0x1d04f0 case 0x3e passes the placement's
//! raw rotation (+0x18..+0x20) to `Wire_Create` 0x1981d8, which negates its z, then
//! `control_create_object` 0x1344e0 (no quaternion) builds `obj+0x90` with `View_RotTransMatrix`
//! 0x1e5168; the drawn mesh is the placement's glist at that matrix. The port drew it with the
//! plain placement matrix and rot.z un-negated, so the 0.2094 tilt [D] sloped the cable downward.
//!
//! Disc data [D]: OBJECT_Zip_Line_1 at (118.234, 46.935, -101.105), rot (0.0045, π, 0.2094),
//! glist x span -16.05..8.0. Live [L audits/live-1108 section 1]: the cable's far end is the small
//! fitting on the tall wall next to (133.75, ~50, -97.3), the near end at the castle wall.
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test cable08_endpoint_live`

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

#[test]
fn zip_line_08_far_end_is_the_high_wall_fitting() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000008.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse");
    let mut wires = Vec::new();
    for (mi, m) in level.maps.iter().enumerate() {
        for p in m.placements.iter().filter(|p| p.flags & 0xff == 0x3e) {
            let (om, oi) = if p.by_hash() { level.by_hash[&p.hashcode] } else { (mi, p.index as usize) };
            wires.push((p.clone(), level.maps[om].objects[oi].clone()));
        }
    }
    assert_eq!(wires.len(), 1, "07000008 places one wire [D]");
    let (p, o) = &wires[0];
    assert_eq!(o.name, "OBJECT_Zip_Line_1");
    assert!((p.rotation[2] - 0.2094).abs() < 1e-3, "the placement tilt [D] (got {:?})", p.rotation);
    let m = nf_view::scene::wire_draw_matrix(p);
    let far = nf_formats::placement::transform(&m, o.min);
    let near = nf_formats::placement::transform(&m, o.max);
    eprintln!("far {far:?} near {near:?}");
    // Far end: the wall fitting beside (133.75, ~50, -97.3), ABOVE Bond's feet (48.97) [L].
    assert!((far[0] - 133.94).abs() < 0.05 && (far[1] - 50.25).abs() < 0.05 && (far[2] + 101.09).abs() < 0.05,
        "the far end is the high wall fitting (got {far:?}); the plain placement matrix put it at y 43.58");
    // Near end: the castle wall, lower than the far end — the cable climbs toward the fitting.
    assert!((near[0] - 110.40).abs() < 0.05 && (near[1] - 45.29).abs() < 0.05, "near end at the castle (got {near:?})");
    assert!(far[1] > near[1] + 4.0, "the cable runs UP from the castle to the wall fitting");
}
