//! drone-block-swaps-g055-2 probe: decode the 07000006 ch-16 guard pair's behaviour blocks —
//! the props FirstSightState [M 0x147c48] reads (0x43 surprise, 0x33 shout, 0x1a level-run,
//! 0x1b/0x18/0x19 combat-move, 0x4a grenade) and the type-table rows for their modes.
//! Opt-in NF_ISO; prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_06_guard_pair_props() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000006.bin")).expect("06 archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    let elf = img.read_file(img.find("/ACTION.ELF").expect("elf")).expect("elf bytes");
    let tables = nf_formats::drone::PropertyTables::from_action_elf(&elf).expect("tables");
    let wanted = [439usize, 440];
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff != 0x0f || !wanted.contains(&i) { continue; }
        let g = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
        println!("#{i} skin {:08x} p3 {} p5 {} p6 {} p8 {} p14 {} p15 {} p16 {} p21 {}",
            g(0), g(3), g(5), g(6), g(8), g(14), g(15), g(16), g(21));
        let (normal, alert) = nf_formats::drone::behaviour_words(&g);
        for (name, words) in [("A", &normal), ("B", &alert)] {
            println!("  block {name}: words {:08x?}", words);
            for id in [0x11u32, 0x13, 0x18, 0x19, 0x1a, 0x1b, 0x2e, 0x33, 0x41, 0x42, 0x43, 0x4a, 0x52, 0x56, 0x20] {
                let v = tables.get(id as usize, words);
                if v != 0 { println!("    prop {id:#x} = {v}"); }
            }
        }
        // The type rows: mode_normal p16, mode_alert p21 — start/alt per DroneTypeSettings.
        for m in [g(16) as usize, g(21) as usize] {
            if let Some(t) = tables.types.get(m) {
                println!("  DroneTypeSettings[{m:#x}] start {} alt {} init {:?}", t.start, t.alt, t.init);
            }
        }
    }
}
