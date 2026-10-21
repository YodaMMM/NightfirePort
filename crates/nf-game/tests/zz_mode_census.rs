use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
#[test]
fn census() {
    let Some(path) = std::env::var_os("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
    let props = nf_formats::drone::PropertyTables::from_action_elf(&elf).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for e in entries.iter().filter(|e| e.name.to_ascii_lowercase().starts_with("070000")) {
        let raw = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).unwrap();
        let Ok(level) = Level::parse(&raw) else { continue };
        let Some(m) = level.main_map() else { continue };
        let mut n1a = 0; let mut n1a_alert = 0; let mut tot = 0;
        for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x0f) {
            let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|x| x.1).unwrap_or(0);
            let (n, a) = nf_formats::drone::behaviour_words(&param);
            tot += 1;
            if props.get(0x1a, &n) != 0 { n1a += 1; if e.name.contains("07000007") { eprintln!("  07 prop1a: skin {:#x} dtype p15 {:#x} p16 {:#x} p5 {:#x} pos {:?} params {:x?}", param(0), param(15), param(16), param(5), p.translation, p.params); } }
            if param(5) == 100 && props.get(0x1a, &a) != 0 { n1a_alert += 1; }
        }
        if tot > 0 { eprintln!("{}: drones {tot} prop1a {n1a} alert-block prop1a {n1a_alert}", e.name); }
    }
}
