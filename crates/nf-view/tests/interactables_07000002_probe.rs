//! interactables-02 probe: dump every 07000002 player-touchable placement by kind, with params.
//! Opt-in NF_ISO; prints only (assertion-free, like the other census dumps).
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
fn dump_07000002_interactable_placements() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000002.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    let placements = &lv.maps[mm].placements;
    println!("main-map placements: {}", placements.len());
    for (i, p) in placements.iter().enumerate() {
        let k = p.flags & 0xff;
        let interact = matches!(k,
            0x20 | 0x22 | 0x28 | 0x2E | 0x29 | 0x30 | 0x31 | 0x33 | 0x36 | 0x37 | 0x38 | 0x3A
            | 0x3E | 0xDB | 0xD8 | 0xD9 | 0xE1 | 0xE2 | 0xE4 | 0xE8 | 0xEA | 0xEB | 0xEC | 0xED
            | 0xF0 | 0xF2 | 0xF9 | 0xFB | 0x0F);
        if !interact { continue; }
        let g = |kk: u32| p.params.iter().find(|(j, _)| *j == kk).map_or(0, |&(_, v)| v);
        let t = p.translation;
        let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        let name = target.map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
            .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
        println!("#{i} kind {k:#04x} @({:.2},{:.2},{:.2}) rot {:.2}/{:.2}/{:.2} obj {name} params [{}]",
            t[0], t[1], t[2], p.rotation[0], p.rotation[1], p.rotation[2], params.join(" "));
    }
}
