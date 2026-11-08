//! The `Light_Create` census, site 13 (`parsemap_create_dynamic_objects` 0x1d04f0 case 0x4E): every
//! type-0x4E placement of the level creates its light with the game's argument conversions — the
//! radius as `(float)(unsigned)` of the params block's +0x2C word [M 0x1d0b8c-0x1d0bd0: `lw` → `mtc1` →
//! `cvt.s.w`, the `bltz` arm the uint→float expansion], the rgb bytes from +0x30/+0x34/+0x38, the
//! channel +0x3C, the flicker period +0x40, the seed +0x44, the mask +0x48 [M 0x1d0bd4-0x1d0c04].
//! Those offsets are on `$s1`, the dispatcher's SECOND argument (0x1d050c `daddu s1, a1, zero`) — the
//! marshaled params block, whose +0x2C..+0x4C are params p0..p8 — not on the placement record `$a0`
//! (translation at +0x0C). An earlier version of this test read the record's raw words
//! (`Placement::extra`), which pinned the wrong block; [L] ee_start_07000001.bin's LightList holds
//! radius 6.0 / rgb 255,121,103 at the four placements, the params values. The old
//! port read the radius as float BITS (`f32::from_bits`), which makes a real radius like 6 read
//! ~8.4e-45 — `1/radius²` overflows and the light can never contribute (`atten = max(0, 1/d² − 1/r²)`).
//! Opt-in: set `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
use nf_game::drone::Rng;

#[test]
fn the_4e_placements_create_lights_with_the_game_s_uint_radius() {
    let Some(path) = std::env::var_os("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    // params p0..p8 sit at +0x2C..+0x4C of the block `$s1` points at [M 0x1d050c, 0x1d0b8c-0x1d0bec].
    let e = |p: &nf_formats::placement::Placement, k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);

    let mut rng = Rng::new(1);
    let mut lights = nf_game::light::LightList::default();
    let mut checked = 0usize;
    for m in 0..level.maps.len() {
        for (i, p) in level.maps[m].placements.iter().enumerate() {
            if p.flags & 0xff != 0x4e || p.flags & 0xa000 != 0 { continue; }
            checked += 1;
            nf_game::light::placement_light(&level, m, i, &mut rng, &mut lights);
            let l = lights.lights.last().unwrap();
            let raw_radius = e(p, 0);
            // [M 0x1d0b8c-0x1d0bd0]: the game converts the raw word as an unsigned int.
            assert_eq!(l.radius, raw_radius as f32,
                "0x4E placement {i} map {m}: radius must be (float)(uint){raw_radius:#x}");
            assert!(l.radius >= 0.0, "the old from_bits reading would be ~1e-44");
            assert_eq!(l.rgb, [(e(p, 1) & 0xff) as u8, (e(p, 2) & 0xff) as u8, (e(p, 3) & 0xff) as u8],
                "rgb bytes from +0x30/+0x34/+0x38 [M 0x1d0b90-0x1d0b9c]");
            assert_eq!(l.channel, (e(p, 4) & 0xffff) as u16, "channel +0x3C [M 0x1d0bd8]");
            assert_eq!(l.flicker_period, (e(p, 5) & 0xffff) as i16, "period +0x40 [M 0x1d0be0]");
            assert_eq!(l.flicker_mask, (e(p, 7) & 0xffff) as u16, "mask +0x48 [M 0x1d0be8]");
            assert_eq!(l.life, 0, "life -1 → permanent [M 0x1d0bdc]");
            assert_eq!(l.kind, 0, "kind 0 [M 0x1d0bf8]");
        }
    }
    assert!(checked > 0, "expected type-0x4E light placements in 0700000c — none found, pick a level that has them");
}
