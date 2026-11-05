//! level-1b-platform-touch probe 3: what the panel script 0x06000832 creates, and the glists'
//! object bounds. Opt-in NF_ISO; prints only.

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
fn probe_1b_panel_script_entities() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    let lb = LevelBin::parse(&data).expect("bin");
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        if se.hash() != 0x0600_0832 { continue; }
        let Ok(nis) = Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        println!("script {:08x} len {} streams {}", se.hash(), nis.length, nis.streams.len());
        for (si, s) in nis.streams.iter().enumerate() {
            for c in s.decode() {
                match c {
                    Cmd::Entity { hash, parent, obj_type, params, .. } => {
                        let name = lv.by_hash.get(&hash).copied()
                            .map(|(m, o)| format!("map{m}#{} \"{}\" min {:?} max {:?}", o,
                                lv.maps[m].objects[o].name.trim(), lv.maps[m].objects[o].min, lv.maps[m].objects[o].max))
                            .unwrap_or_else(|| "UNRESOLVED".into());
                        println!("  stream{si} Entity glist {hash:08x} parent {parent:?} type {obj_type:#x} params {params:?}\n      {name}");
                    }
                    other => println!("  stream{si} {other:?}"),
                }
            }
        }
    }
}
