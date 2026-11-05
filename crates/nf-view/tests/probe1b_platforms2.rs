//! level-1b-platform-touch probe 2: the Drake-arena break candidates — the SP hit-point
//! placements (41-56 the panels, 212-220 the consoles), the 0xE1 destroyables' targets, and
//! every placement within 25 units of the Drake-arena centre (−5, −1, 0). Opt-in NF_ISO; prints only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn name_of(lv: &Level, hash: u32) -> String {
    match lv.by_hash.get(&hash).copied() {
        Some((m, o)) => format!("map{m}#{} \"{}\"", o, lv.maps[m].objects[o].name.trim()),
        None => "UNRESOLVED".to_string(),
    }
}

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_1b_arena_break_candidates() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();

    println!("== SP players (0xd9 placements) with hit points or arena positions ==");
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff != 0xd9 { continue; }
        let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
        let t = p.translation;
        let name = if p.by_hash() { name_of(&lv, p.hashcode) } else { format!("idx{}", p.index) };
        // print all; mark the near-arena ones
        let near = (t[0] + 5.0).abs() < 25.0 && (t[2]).abs() < 25.0;
        println!("#{i} {name} @({:.2},{:.2},{:.2}) params [{}] flags {:#x}{}",
            t[0], t[1], t[2],
            p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect::<Vec<_>>().join(" "),
            p.flags, if near { "  <== ARENA" } else { "" });
    }

    println!("== 0xE1 destroyables ==");
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff != 0xe1 { continue; }
        let t = p.translation;
        let name = if p.by_hash() { name_of(&lv, p.hashcode) } else { format!("idx{}", p.index) };
        println!("#{i} {name} @({:.2},{:.2},{:.2}) params [{}]",
            t[0], t[1], t[2],
            p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect::<Vec<_>>().join(" "));
    }
}
