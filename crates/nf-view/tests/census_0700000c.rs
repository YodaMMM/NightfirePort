//! level-0c census: dump every 0700000c main-map placement by kind, with params. Opt-in NF_ISO; prints only.
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
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_0700000c_placements_by_kind() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700000c.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    println!("maps {} main {mm} objects[mm] {}", lv.maps.len(), lv.maps[mm].objects.len());
    let placements = &lv.maps[mm].placements;
    println!("main-map placements: {}", placements.len());
    let mut by_kind: Vec<(u32, Vec<usize>)> = Vec::new();
    for (i, p) in placements.iter().enumerate() {
        let k = p.flags & 0xff;
        match by_kind.iter_mut().find(|(kk, _)| *kk == k) {
            Some((_, v)) => v.push(i),
            None => by_kind.push((k, vec![i])),
        }
    }
    by_kind.sort();
    for (k, idxs) in &by_kind {
        println!("== KIND {k:#04x}: {} placements ==", idxs.len());
        for &i in idxs {
            let p = &placements[i];
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let name = target.map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
            let t = p.translation;
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            let cel = if p.creates_cel() { " CEL" } else { "" };
            println!("#{i} kind {k:#04x}{cel} \"{}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
                name, t[0], t[1], t[2], params.join(" "), p.hashcode, p.flags);
        }
    }
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_0700000c_kind_histogram() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700000c.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    let mut counts: Vec<(u32, usize)> = Vec::new();
    for p in &lv.maps[mm].placements {
        let k = p.flags & 0xff;
        match counts.iter_mut().find(|(kk, _)| *kk == k) {
            Some((_, c)) => *c += 1,
            None => counts.push((k, 1)),
        }
    }
    counts.sort();
    for (k, c) in &counts { println!("kind {k:#04x}: {c}"); }
    let unresolved = lv.maps[mm].placements.iter().filter(|p| {
        if p.by_hash() { !lv.by_hash.contains_key(&p.hashcode) }
        else { p.index as usize >= lv.maps[mm].objects.len() }
    }).count();
    println!("unresolved placements: {unresolved}");
}
