//! Per-level census of the guard behaviour bits the alert, hearing and surrender paths read.
//! Opt-in NF_ISO; prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

#[test]
#[ignore = "assertion-free census; run with --ignored"]
fn census() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("elf")).expect("elf bytes");
    let tables = nf_formats::drone::PropertyTables::from_action_elf(&elf).expect("tables");
    let ids: [usize; 16] = [0x1f, 0x26, 0x27, 0x28, 0x29, 0x31, 0x32, 0x33, 0x3c, 0x43, 0x4a, 0x4f, 0x51, 0x57, 0x21, 0x40];
    print!("level    n ");
    for id in ids { print!(" p{id:02x}"); }
    println!("  modes(p16)");
    for &id in nf_view::dev::LEVELS {
        let name = format!("{id:08x}.bin");
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else { continue };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let Ok(lv) = Level::parse(&data) else { continue };
        let Some(mm) = lv.main_map() else { continue };
        let mut n = 0; let mut c = [0u32; 16]; let mut modes = std::collections::BTreeMap::new();
        for p in &lv.maps[mm].placements {
            if p.flags & 0xff != 0x0f { continue; }
            let g = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let (normal, _alert) = nf_formats::drone::behaviour_words(&g);
            n += 1;
            for (i, pid) in ids.iter().enumerate() { if tables.get(*pid, &normal) != 0 { c[i] += 1; } }
            *modes.entry(g(16)).or_insert(0u32) += 1;
        }
        print!("{:02x} {n:6} ", id & 0xff);
        for v in c { print!(" {v:3}"); }
        println!("  {modes:x?}");
    }
}

/// The spawned drones on every level: what the placement words cannot show (mode, team, start state).
#[test]
#[ignore = "assertion-free census; run with --ignored"]
fn census_live() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    use std::collections::BTreeMap as M;
    for &id in nf_view::dev::LEVELS {
        let name = format!("{id:08x}.bin");
        let Ok(s) = nf_view::load(&iso, &name, false) else { println!("{:02x} no load", id & 0xff); continue };
        let Some(sim) = s.sim.as_ref() else { println!("{:02x} no sim", id & 0xff); continue };
        let d = sim.data();
        let (mut modes, mut teams, mut starts, mut voices) = (M::new(), M::new(), M::new(), M::new());
        let (mut t3see, mut me57, mut src31, mut obs29, mut obs28, mut p33) = (0, 0, 0, 0, 0, 0);
        for x in &sim.drones.list {
            *modes.entry(x.mode).or_insert(0u32) += 1;
            *teams.entry(x.team).or_insert(0u32) += 1;
            *starts.entry(x.return_state).or_insert(0u32) += 1;
            *voices.entry(x.voice_type).or_insert(0u32) += 1;
            if x.team == 3 && x.prop(&d, 0x3c) { t3see += 1; }
            if matches!(x.mode, 7 | 0xe) && !x.prop(&d, 0x57) { me57 += 1; }
            if x.team == 1 && x.prop(&d, 0x31) { src31 += 1; }
            if x.team != 2 && x.prop(&d, 0x29) { obs29 += 1; }
            if x.team != 2 && x.prop(&d, 0x28) { obs28 += 1; }
            if x.team == 1 && x.prop(&d, 0x33) { p33 += 1; }
        }
        println!("{:02x} n{} modes{modes:x?} teams{teams:x?} start{starts:x?} voice{voices:x?}\n    t3+3c {t3see} m7E!57 {me57} dying31 {src31} see29 {obs29} see28 {obs28} shout33 {p33}", id & 0xff, sim.drones.list.len());
    }
}
