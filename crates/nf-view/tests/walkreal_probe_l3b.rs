//! level-4a-lift-3 probe b: every placement in every map with any param == 0x62 (the mission
//! FAIL channel, written only by Player_HandleDeath [M 0x190bb0:32] — the disc writer hunt for
//! the live god-mode kill at y -95.80), plus every trigger's out channel. Assertion-free.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_formats::iso9660::IsoImage;

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn dump_channel_0x62_writers() {
    let iso = iso();
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            let k = p.flags & 0xff;
            let hit: Vec<_> = p.params.iter().filter(|(_, v)| *v == 0x62).collect();
            let is_trigger = matches!(k, 0xe8..=0xf2);
            if !hit.is_empty() || (is_trigger && p.translation[1] < 10.0) {
                let ps = p.params.iter().map(|(j, v)| format!("p{j}={v:#x}")).collect::<Vec<_>>().join(",");
                println!("map{mi} #{i} kind {k:#04x} ({:.2},{:.2},{:.2}) params [{ps}] hit{hit:?}",
                    p.translation[0], p.translation[1], p.translation[2]);
            }
        }
    }
    println!("done");
}
