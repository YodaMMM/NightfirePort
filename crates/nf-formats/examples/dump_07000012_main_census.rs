//! level-12-audit census: dump every 07000012 MAIN-MAP placement by kind with params — the [D]
//! source rows for docs/levels/07000012-audit.md. Prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("07000012.bin"))
        .expect("archive");
    let data = img
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    println!("maps {} main {mm} placements {}", lv.maps.len(), lv.maps[mm].placements.len());
    let mut by_kind: Vec<(u32, Vec<usize>)> = Vec::new();
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
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
            let p = &lv.maps[mm].placements[i];
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let name = target
                .map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
            let t = p.translation;
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            let cel = if p.creates_cel() { " CEL" } else { "" };
            println!(
                "#{i} kind {k:#04x}{cel} \"{name}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
                t[0], t[1], t[2], params.join(" "), p.hashcode, p.flags
            );
        }
    }

    // The level's kind-7 scripts (the ScriptPlayers' and the mission's NIS streams) [D bytes].
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("levelbin");
    for hash in [0x06000530u32, 0x060005bb, 0x06000958, 0x0600098A] {
        for se in lb.entries.iter().filter(|e| e.kind == 7 && e.hash() == hash) {
            let nis = match nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) {
                Ok(n) => n,
                Err(e) => {
                    println!("script {hash:08x}: parse error {e}");
                    continue;
                }
            };
            println!(
                "== SCRIPT {:08x} len {} streams {} ==",
                nis.hashcode, nis.length, nis.streams.len()
            );
            for (si, s) in nis.streams.iter().enumerate() {
                println!(" stream {si}: keys {}..{}", s.first_key, s.last_key);
                for c in s.decode() {
                    match &c {
                        nf_formats::nis::Cmd::Entity { hash, parent, spline, obj_type, params } => {
                            println!("   Entity glist {hash:08x} parent {parent:?} spline {spline} type {obj_type:02x} params {params:?}")
                        }
                        nf_formats::nis::Cmd::Anim { skin, script, start } => {
                            println!("   Anim skin {skin:08x} script {script:08x} start {start}")
                        }
                        nf_formats::nis::Cmd::Event { kind, params } => {
                            println!("   Event kind {kind:02x} params {params:?}")
                        }
                        other => println!("   {other:?}"),
                    }
                }
            }
        }
    }
}
