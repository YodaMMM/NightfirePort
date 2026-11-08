//! The type-0x4E placement lights on 07000001 read the placement's PARAMS block, not the raw
//! quaternion/scale words.
//!
//! Game model: `parsemap_create_dynamic_objects` 0x1d04f0 case 0x4e marshals the light call from
//! the level_tag the block parser filled — the placement's param pairs land at
//! `level_tag+0x2C+4k` and the case reads exactly those (`lw 0x2c(s1)` radius p0 with the
//! uint→float `cvt.s.w` [M 0x1d0b8c-0x1d0bd0], `lbu 0x30/0x34/0x38(s1)` rgb p1/p2/p3
//! [M 0x1d0b90-0x1d0b9c], `lhu 0x3c(s1)` channel p4 [M 0x1d0bd8], −1 life / 0 kind / 1.0
//! intensity constants [M 0x1d0bdc-0x1d0bf8], `lhu 0x40(s1)` flicker period p5 [M 0x1d0be0],
//! `lhu 0x44(s1)` flicker seed p6 [M 0x1d0be4], `lhu 0x48(s1)` mask p7 [M 0x1d0be8],
//! `lw 0x4c(s1)` sound p8, 0 → 0xffff [M 0x1d0bb0-0x1d0bd0]); `Light_Create` 0x1cc3d8 stores
//! them into the 0x50-byte record. The placement's raw +0x24..+0x44 words are the quaternion
//! and scale the dispatcher stages for OTHER cases — on 07000001's four lights they read
//! (0,0,0,1) + (1,1,1) floats [D the probe], not light data.
//!
//! The disc rows [D 07000001.bin, the four 0x4e placements]: params p0=6 (radius), p1=255
//! p2=121 p3=103 (rgb), p7=1 (mask), p4/p5/p6/p8 all 0.

use nf_game::drone::Rng;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn level() -> nf_formats::level::Level {
    let isopath = iso();
    let iso = nf_formats::iso9660::IsoImage::open(&isopath).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000001.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    nf_formats::level::Level::parse(&data).unwrap()
}

#[test]
fn the_0x4e_lights_read_the_params_block() {
    let level = level();
    let m = level.main_map().expect("main map");
    let mut rng = Rng::default();
    let mut lights = nf_game::light::LightList::default();
    let mut n = 0usize;
    for (i, p) in level.maps[m].placements.iter().enumerate() {
        if p.flags & 0xff != 0x4e { continue; }
        nf_game::light::placement_light(&level, m, i, &mut rng, &mut lights);
        n += 1;
    }
    assert_eq!(n, 4, "07000001 carries four 0x4e placements [D]");
    assert_eq!(lights.lights.len(), 4, "each creates one light");
    for (k, l) in lights.lights.iter().enumerate() {
        assert_eq!(l.radius, 6.0, "light {k}: radius = (float)param0 = 6 [D p0=6; M 0x1d0b8c-0x1d0bd0]");
        assert_eq!(l.rgb, [255, 121, 103], "light {k}: rgb = params 1..3 [D]");
        assert_eq!(l.channel, 0, "light {k}: channel = param4 [D p4=0]");
        assert_eq!(l.life, 0, "light {k}: permanent (life arg -1) [M 0x1d0bdc]");
        assert_eq!(l.kind, 0, "light {k}: kind byte 0 [M 0x1d0bf8]");
        assert_eq!(l.intensity, 1.0, "light {k}: intensity 1.0 [M 0x1d0bf0]");
        assert_eq!(l.flicker_period, 0, "light {k}: flicker period = param5 [D p5=0]");
        assert_eq!(l.flicker_base, 0, "light {k}: flicker base = seed>>1 [M 0x1cc4f4]");
        assert_eq!(l.flicker_mask, 1, "light {k}: flicker mask = param7 [D p7=1; M 0x1d0be8]");
        assert_eq!(l.sound, -1, "light {k}: sound = param8, 0 → 0xffff [M 0x1d0bb0-0x1d0bd0]");
    }
}
