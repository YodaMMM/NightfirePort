//! level-1b-platform-touch probe 4: the collision blocks of the break candidates' glists —
//! the solar panels (02000849), the console screens, the Drake-arena entities. Opt-in NF_ISO.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::{Cmd, Nis};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_1b_glist_collision() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    let lb = LevelBin::parse(&data).expect("bin");

    // Every script's Entity glists with the object's collision/shape info.
    let mut want: Vec<u32> = Vec::new();
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        for s in nis.streams.iter() {
            for c in s.decode() {
                if let Cmd::Entity { hash, .. } = c {
                    if !want.contains(&hash) { want.push(hash); }
                }
            }
        }
    }
    println!("== script entity glists: collision blocks ==");
    for h in want {
        if h == 0 { continue; }
        let Some(&(m, o)) = lv.by_hash.get(&h) else { println!("{h:08x} UNRESOLVED"); continue };
        let obj = &lv.maps[m].objects[o];
        let col = match obj.collision.as_ref() {
            Some(c) => format!("{} boxes, {} tris", c.boxes.len(), c.tris.len()),
            None => "NONE".to_string(),
        };
        println!("{h:08x} map{m}#{} \"{}\" collision {col}", o, obj.name.trim());
    }
}
