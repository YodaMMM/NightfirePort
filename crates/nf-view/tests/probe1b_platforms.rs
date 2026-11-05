//! level-1b-platform-touch probe: resolve the Drake-fight scripts' entity glists to object
//! names, list every "*Platform*"-named object in the level with its map/hash/position, and
//! dump the fight-window entities the port carries at the fight gate. Opt-in NF_ISO; prints only.

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

fn name_of(lv: &Level, hash: u32) -> String {
    match lv.by_hash.get(&hash).copied() {
        Some((m, o)) => format!("{:08x} map{m}#{} \"{}\"", hash, o, lv.maps[m].objects[o].name.trim()),
        None => format!("{hash:08x} UNRESOLVED"),
    }
}

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_1b_drake_fight_platforms() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");

    // 1. Every object whose name mentions platform (any map), with its hash.
    println!("== objects named *platform* ==");
    for (m, map) in lv.maps.iter().enumerate() {
        for (o, obj) in map.objects.iter().enumerate() {
            if obj.name.to_ascii_lowercase().contains("platform") {
                println!("map{m}#{o} hash {:08x} \"{}\" min {:?} max {:?}",
                    obj.hashcode, obj.name.trim(), obj.min, obj.max);
            }
        }
    }

    // 2. The fight-window scripts' entity glists → names.
    let lb = LevelBin::parse(&data).expect("bin");
    let fight = [0x0600_08f0u32, 0x0600_0900, 0x0600_08ff, 0x0600_0901, 0x0600_08fc, 0x0600_091b, 0x0600_085c];
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        if !fight.contains(&se.hash()) { continue; }
        let Ok(nis) = Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        println!("== script {:08x} ({} streams) ==", se.hash(), nis.streams.len());
        for (si, s) in nis.streams.iter().enumerate() {
            for c in s.decode() {
                if let Cmd::Entity { hash, parent, obj_type, params, .. } = c {
                    println!("  stream{si} Entity glist {hash:08x} parent {parent:?} type {obj_type:#x} params {params:?} -> {}",
                        name_of(&lv, hash));
                }
            }
        }
    }
}
