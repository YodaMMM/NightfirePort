//! level-1b-drake-fight probe: dump the Drake-area placements (the fight chain's disc data):
//! every placement index 180..195 with kind, position and full params, plus every spawner
//! (0xF1) and drone (0x0F) of the main map with its params. Assertion-free diagnostic.
//! [D data via nf-formats::level.]

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_0700001b_drake_chain_placements() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    let placements = &lv.maps[mm].placements;
    println!("main-map placements: {}", placements.len());
    println!("\n== placements 175..200 ==");
    for (i, p) in placements.iter().enumerate().skip(175).take(25) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v}")); }
        println!("#{i} kind={:#04x} pos=({:.2},{:.2},{:.2}){ps}", p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2]);
    }
    println!("\n== every spawner (0xF1) ==");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xf1) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v}")); }
        println!("SPAWNER #{i} pos=({:.2},{:.2},{:.2}){ps}", p.translation[0], p.translation[1], p.translation[2]);
    }
    println!("\n== every placement near the shaft wall x 3.0..7.0, z -14..14 (all maps) ==");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            if p.translation[0] < 3.0 || p.translation[0] > 7.0 { continue; }
            if p.translation[2] < -14.0 || p.translation[2] > 14.0 { continue; }
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("map{mi} #{i} kind={:#04x} flags={:#08x} pos=({:.2},{:.2},{:.2}){ps}",
                p.flags & 0xff, p.flags, p.translation[0], p.translation[1], p.translation[2]);
        }
    }
    println!("\n== every door-ish placement (kinds 0xd9/0xda/0xdb/0xdc) on the main map ==");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| matches!(p.flags & 0xff, 0xd9 | 0xda | 0xdb | 0xdc)) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
        println!("DOOR? #{i} kind={:#04x} pos=({:.2},{:.2},{:.2}){ps}", p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2]);
    }
    println!("\n== every placement on ANY map whose params carry 0x0600091c/0x0600091b/0x0600091a ==");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            for (_, v) in &p.params {
                if *v >= 0x0600_0919 && *v <= 0x0600_091d {
                    let mut ps = String::new();
                    for (k, w) in &p.params { ps.push_str(&format!(" p{k}={w:#x}")); }
                    println!("map{mi} #{i} kind={:#04x} pos=({:.2},{:.2},{:.2}){ps}", p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2]);
                }
            }
        }
    }
    println!("\n== map6 placements (0xd9) ==");
    for (mi, m) in lv.maps.iter().enumerate() {
        if m.kind != 1 && mi != 6 { continue; }
        if m.kind == 1 { continue; }
        for (i, p) in m.placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xd9) {
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("map{mi} SP #{i} pos=({:.2},{:.2},{:.2}){ps}", p.translation[0], p.translation[1], p.translation[2]);
        }
    }
    println!("\n== every trigger-family placement (0xE8-0xEF, 0xDC) ==");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| matches!(p.flags & 0xff, 0xe8..=0xef | 0xdc)) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v}")); }
        println!("TRIG #{i} kind={:#04x} pos=({:.2},{:.2},{:.2}){ps}", p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2]);
    }
    println!("\n== every AI volume (0xF5/0xF8) ==");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| matches!(p.flags, 0xf5 | 0xf8)) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v}")); }
        println!("VOL #{i} kind={:#04x} pos=({:.2},{:.2},{:.2}) scale=({:.2},{:.2},{:.2}){ps}",
            p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2],
            p.scale()[0], p.scale()[1], p.scale()[2]);
    }
    println!("\n== every drone (0x0F) ==");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x0f) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v}")); }
        println!("DRONE #{i} pos=({:.2},{:.2},{:.2}){ps}", p.translation[0], p.translation[1], p.translation[2]);
    }
}
