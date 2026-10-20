//! level-4a (Tower-2 elevator) census: dump every placement by kind, with params, across ALL maps
//! (the 162 shaft doors may live on sub-maps), plus a mesh-resolution pass (every model draws:
//! no UNRESOLVED hash). Opt-in NF_ISO; prints only, no assertions — the pinned assertions live in
//! interact_0700004a_live.rs.
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
fn dump_0700004a_placements_by_kind() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700004a.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    println!("maps {} main {mm} objects[mm] {}", lv.maps.len(), lv.maps[mm].objects.len());

    // All-maps per-kind counts first (the shape of the level at a glance).
    let mut all: Vec<(u32, usize)> = Vec::new();
    for m in &lv.maps {
        for p in &m.placements {
            let k = p.flags & 0xff;
            match all.iter_mut().find(|(kk, _)| *kk == k) {
                Some((_, n)) => *n += 1,
                None => all.push((k, 1)),
            }
        }
    }
    all.sort();
    println!("== ALL-MAPS kind counts ==");
    for (k, n) in &all {
        let per: Vec<String> = lv.maps.iter().enumerate()
            .map(|(mi, m)| (mi, m.placements.iter().filter(|p| p.flags & 0xff == *k).count()))
            .filter(|(_, n)| *n > 0)
            .map(|(mi, n)| format!("m{mi}:{n}"))
            .collect();
        println!("kind {k:#04x}: {n}  ({})", per.join(" "));
    }

    // Full dump, every map, kind-sorted.
    for (mi, m) in lv.maps.iter().enumerate() {
        let placements = &m.placements;
        if placements.is_empty() { continue; }
        println!("== MAP m{mi} (kind {} objects {}) placements {} ==",
            m.kind, m.objects.len(), placements.len());
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
            println!("-- KIND {k:#04x}: {} placements --", idxs.len());
            for &i in idxs {
                let p = &placements[i];
                let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
                let name = target.map(|(ma, o)| lv.maps[ma].objects[o].name.trim().to_string())
                    .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
                let t = p.translation;
                let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
                let cel = if p.creates_cel() { " CEL" } else { "" };
                println!("m{mi} #{i} kind {k:#04x}{cel} \"{}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
                    name, t[0], t[1], t[2], params.join(" "), p.hashcode, p.flags);
            }
        }
    }

    // Mesh-resolution pass: every placement must resolve to an object (no UNRESOLVED hash) for
    // its model to draw — the "every model draws" census line.
    println!("== MESH RESOLUTION (unresolved only) ==");
    let mut unresolved = 0usize;
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            let ok = if p.by_hash() { lv.by_hash.contains_key(&p.hashcode) }
                else { (p.index as usize) < m.objects.len() };
            if !ok {
                unresolved += 1;
                let t = p.translation;
                println!("UNRESOLVED m{mi} #{i} kind {:#04x} hash {:08x} @({:.2},{:.2},{:.2})",
                    p.flags & 0xff, p.hashcode, t[0], t[1], t[2]);
            }
        }
    }
    println!("mesh resolution: {unresolved} unresolved placements");
}
