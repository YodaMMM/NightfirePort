//! level-02 drone census: dump every 07000002 main-map drone (0x0f) placement with its skin,
//! the port's computed kind/weapon and the raw params. Opt-in NF_ISO; prints only.
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
fn dump_07000002_drone_placements() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000002.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    let placements = &lv.maps[mm].placements;
    println!("main-map placements: {}", placements.len());
    for (i, p) in placements.iter().enumerate() {
        if p.flags & 0xff != 0x0f { continue; }
        let g = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
        let skin = g(0);
        let setup = nf_formats::drone::drone_setup(skin, g(9), 0x0700_0002);
        let weapon = nf_formats::drone::weapon_id(skin, g(9));
        let t = p.translation;
        let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
        println!("#{i} @({:.2},{:.2},{:.2}) skin {skin:08x} kind {:#x} set {:#x} weapon {weapon:#x} p2(diff) {} p14(class) {} params [{}]",
            t[0], t[1], t[2], setup.kind, setup.anim_set, g(2), g(14), params.join(" "));
    }
}
