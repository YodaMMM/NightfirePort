//! bath-hatch-02 probe: 07000002's SimpleScripts, triggers and the channel graph around the
//! bath hatch above the deep ladder. Scratch print, not committed.

use nf_formats::level::Level;

fn main() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000002.bin")).expect("07000002.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse");
    let main = level.main_map().expect("main map");
    println!("maps: {} (main #{}: placements {})", level.maps.len(), main, level.maps[main].placements.len());

    // 1. every 0xE2 SS placement on every map
    println!("\n=== 0xE2 SimpleScript placements (all maps) ===");
    for (mi, map) in level.maps.iter().enumerate() {
        for (i, p) in map.placements.iter().enumerate() {
            if p.flags & 0xff != 0xe2 { continue; }
            let keys = p.extra[7] & 0xffff;
            let rate = p.extra[7] >> 16;
            let path = p.extra[8] & 0xffff;
            let pos = p.translation;
            let in_main = mi == main;
            println!("map {mi}{} pl#{} {} keys {keys} rate {rate} path {path} at ({:.2}, {:.2}, {:.2}) params {:?}",
                if in_main { "(MAIN)" } else { "" }, i + 0, p.hashcode, pos[0], pos[1], pos[2], p.params);
        }
    }

    // 2. the triggers of the main map (nf_game::triggers::build) — kind/out/list
    let trig = nf_game::triggers::build(&level);
    println!("\n=== triggers (main map, {} total) ===", trig.len());
    for t in &trig {
        let p = &level.maps[main].placements[t.placement];
        let pos = p.translation;
        println!("pl#{:03} kind {} out {} list {:?} mask {:#x} at ({:.2}, {:.2}, {:.2})",
            t.placement, t.kind, t.out, t.list, t.mask, pos[0], pos[1], pos[2]);
    }

    // 3. any placement on the main map whose params mention ch 2/61/62/64 (raw scan)
    println!("\n=== main-map placements with params referencing 2/61(0x3d)/62(0x3e)/64(0x40) ===");
    for (i, p) in level.maps[main].placements.iter().enumerate() {
        let hit: Vec<(u32, u32)> = p.params.iter()
            .filter(|&&(k, v)| v == 2 || v == 61 || v == 62 || v == 64)
            .copied().collect();
        if !hit.is_empty() {
            let pos = p.translation;
            println!("pl#{:03} type {:#04x} at ({:.2}, {:.2}, {:.2}) hits {:?} all {:?}",
                i, p.flags & 0xff, pos[0], pos[1], pos[2], hit, p.params);
        }
    }

    // 4. placements near the ladder top (x -50.35, z 5.50), radius 8, main map
    println!("\n=== main-map placements within 8 of the deep ladder top (-50.35, 5.50) ===");
    for (i, p) in level.maps[main].placements.iter().enumerate() {
        let pos = p.translation;
        let d2 = (pos[0] - -50.35).powi(2) + (pos[2] - 5.50).powi(2);
        if d2 < 64.0 {
            println!("pl#{:03} type {:#04x} hash {:08x} at ({:.2}, {:.2}, {:.2}) params {:?} extra {:?}",
                i, p.flags & 0xff, p.hashcode, pos[0], pos[1], pos[2], p.params, p.extra);
        }
    }
}
