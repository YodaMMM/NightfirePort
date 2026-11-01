//! The ROOM cels (placement flags 0xc023 / 0xc047) sit at the identity, whatever their placement's
//! translation words hold: `parseentity_fixup_entity` 0x1d00c0 zeroes the cel's +0x60/+0x64/+0x68
//! and sets `cel+0x3c = 0x804c0xx`, whose bit 0x8000000 makes `Collide_Intersect` 0x1ebaa8 test the
//! room untransformed [M]. On 07000012 the security room #69 carries z 0.2955; at the identity its
//! door[19] frame lines up with the corridor's (the game's doorway, crossed live on z −0.812..−0.794
//! [L PCSX2 2026-10-31]). Opt-in: NF_ISO.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::transform;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

/// The z of every vertical +z/−z facing collision face of `inst` crossing the door[19] jamb band
/// (x 6.75..6.95, y 0.5..1.5, z −1.6..0.2).
fn jamb_faces(lv: &Level, inst: &nf_formats::level::Instance) -> Vec<f32> {
    let c = lv.maps[inst.map].objects[inst.object].collision.as_ref().expect("the room's collision");
    let mut out = Vec::new();
    for t in c.triangles() {
        let t = t.map(|p| transform(&inst.matrix, p));
        let lo = [0, 1, 2].map(|k| t[0][k].min(t[1][k]).min(t[2][k]));
        let hi = [0, 1, 2].map(|k| t[0][k].max(t[1][k]).max(t[2][k]));
        if hi[0] < 6.75 || lo[0] > 6.95 || hi[1] < 0.5 || lo[1] > 1.5 || hi[2] < -1.6 || lo[2] > 0.2 { continue; }
        if hi[2] - lo[2] > 1e-3 { continue; } // a z = const face only
        out.push(lo[2]);
    }
    out
}

#[test]
fn room_cels_ignore_the_placement_translation_07000012() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set; skipping"); return };
    let lv = Level::parse(&read_archive(&iso, "07000012.bin")).expect("parse");
    let (insts, _) = lv.instances(true);
    let room = |plc: u16| insts.iter().find(|i| i.placement.index == plc && i.placement.flags & 0xffff == 0xc047)
        .unwrap_or_else(|| panic!("room cel #{plc} [D]"));
    let sec = room(69);
    // The sister: the placement's own words DO carry an offset — an applied translation would show.
    assert!((sec.placement.translation[2] - 0.2955).abs() < 1e-3, "#69's placement z [D], got {:?}", sec.placement.translation);
    assert_eq!(sec.matrix, nf_formats::placement::object_matrix([0.0; 3], [0.0; 3]),
        "the room cel sits at the identity [M parseentity_fixup_entity 0x1d00c0]");
    let corr = room(120);
    // The two rooms' door[19] jambs: the corridor's narrow strips at z −0.262 / −1.462 and the
    // security room's at −0.266 / −1.466 — aligned to 4 mm, a 1.2 opening (with the 0.2955
    // shift the security strip sat at −1.166 and left a 0.904 pinch for the 1.1 capsule).
    let near = |v: &[f32], z: f32| v.iter().any(|&f| (f - z).abs() < 0.01);
    let (c, s) = (jamb_faces(&lv, corr), jamb_faces(&lv, sec));
    assert!(near(&c, -0.262) && near(&c, -1.462), "the corridor's jamb strips [D], got {c:?}");
    assert!(near(&s, -1.466) && !near(&s, -1.166), "the security room's south jamb at the identity [D], got {s:?}");
}
