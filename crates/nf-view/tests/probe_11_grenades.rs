//! probe (assertion-free, `--ignored`): the 07000011 drone placements' grenade endowment [D] —
//! every kind-0x0f placement: index-in-filter-order, skin, params 2/5/9/11/14, the param-11
//! grenade decode (`DefaultInit` 0x14b300:348-376: low nibble → weapon, count), and the
//! behaviour stats decode. Run: NF_ISO=… cargo test -p nf-view --test probe_11_grenades -- --ignored --nocapture

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::Placement;

fn param(p: &Placement, k: u32) -> u32 {
    p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_07000011_grenade_endowment() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    println!("== 07000011 kind-0x0f placements (ordinal = the create-pass index) ==");
    for (ordinal, p) in lv.maps[mm].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x0f) {
        let raw11 = param(p, 11);
        let low = raw11 & 0xf;
        let hi = (raw11 >> 4) & 0xffff;
        // DefaultInit 0x14b300:348-376 [M via the port]: +0xBCC = (short)(raw >> 4) (the count),
        // +0xBCE = raw & 0xf; the switch on hi picks the weapon: 1→0x36, 2→0x35, 4→0x34,
        // 5→count+0x3a, 6→0x6c count 1, default→0,0.
        let (gw, gc) = match hi {
            1 => (0x36u32, (raw11 >> 4) as u32), 2 => (0x35, (raw11 >> 4) as u32), 4 => (0x34, (raw11 >> 4) as u32),
            5 => (0x3a + low, 1), 6 => (0x6c, 1), _ => (0, 0),
        };
        println!("ordinal {:3} skin {:08x} p2 {} p5 {:5} p9 {:3} p11 {:#010x} (hi {hi:#06x} → gw {gw:#04x} count {gc}) p14 {} p15 {} at ({:.2},{:.2},{:.2})",
            ordinal, param(p, 0), param(p, 2), param(p, 5), param(p, 9), raw11, param(p, 14), param(p, 15),
            p.translation[0], p.translation[1], p.translation[2]);
    }
}
