//! reverb-zones probe: dump every room cel (env / underwater / bbox) for the two candidate levels,
//! plus the spawn position. Opt-in NF_ISO; prints only (scratch — not part of the deliverable).
use std::env;

fn iso() -> Option<String> { env::var("NF_ISO").ok() }

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn census_cel_classes_per_level() {
    use nf_formats::iso9660::IsoImage;
    use nf_formats::level::Level;
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    for e in &entries {
        let name = e.name.to_ascii_uppercase();
        if !name.starts_with("07") || !name.ends_with(".BIN") || name.len() != 12 { continue; }
        let data = match img.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let lv = match Level::parse(&data) { Ok(l) => l, Err(_) => continue };
        let Some(mm) = lv.main_map() else { continue };
        let mut c023: Vec<i32> = Vec::new();
        let mut c047: Vec<i32> = Vec::new();
        for p in &lv.maps[mm].placements {
            if !p.creates_cel() { continue; }
            match p.flags & 0xffff {
                0xc023 => c023.push(p.params.iter().find(|(i, _)| *i == 3).map(|&(_, v)| v as u32 as i32).unwrap_or(0)),
                0xc047 => c047.push(p.params.iter().find(|(i, _)| *i == 9).map(|&(_, v)| v as u16 as i16 as i32).unwrap_or(0)),
                _ => {}
            }
        }
        c023.sort(); c023.dedup(); c047.sort(); c047.dedup();
        println!("{}: 0xc023 cels {} param3 {c023:?} | 0xc047 cels {} env {c047:?}", e.name,
            lv.maps[mm].placements.iter().filter(|p| p.creates_cel() && (p.flags & 0xffff) == 0xc023).count(),
            lv.maps[mm].placements.iter().filter(|p| p.creates_cel() && (p.flags & 0xffff) == 0xc047).count());
    }
}

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn dump_room_cels() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    for level in ["07000005.bin", "07000006.bin", "07000007.bin"] {
        let scene = match nf_view::load(&iso, level, false) {
            Ok(s) => s,
            Err(e) => { println!("== {level}: load error {e}"); continue; }
        };
        println!("== {level}: {} room cels", scene.cels.len());
        for (i, c) in scene.cels.iter().enumerate() {
            println!("  cel {i:2} env {:>5} uw {:5} min {:8.1} {:8.1} {:8.1} max {:8.1} {:8.1} {:8.1}",
                c.env, c.underwater, c.min[0], c.min[1], c.min[2], c.max[0], c.max[1], c.max[2]);
        }
        if let Some(sim) = scene.sim.as_ref() {
            println!("  spawn pos {:.2} {:.2} {:.2} head {:.2} {:.2} {:.2}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2],
                sim.bond.head()[0], sim.bond.head()[1], sim.bond.head()[2]);
            println!("  spawn cel_at(pos) {:?} cel_at(head) {:?}", sim.cel_at(sim.bond.pos), sim.cel_at(sim.bond.head()));
        }
    }
}
