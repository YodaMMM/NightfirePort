//! lock census part 3: the level-bin script streams. Every kind-7 script of the candidate
//! levels, event 0xD / 0x11 writes (`Script_EventHandler` 0x1d94c0) whose target channel is one
//! of that level's kind-0 lock out channels. Opt-in NF_ISO; prints only.
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
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_lock_channel_script_writes() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    for a in ["0700000a.bin", "0700000b.bin", "07000011.bin", "07000012.bin", "07000014.bin", "07000015.bin", "07000046.bin"] {
        let data = read_archive(&iso, a);
        let lv = Level::parse(&data).expect("parse");
        let mm = lv.main_map().expect("main map");
        let mut chans: Vec<u32> = Vec::new();
        for p in lv.maps[mm].placements.iter() {
            if p.flags & 0xff != 0x2e { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            if param(0) == 0 && param(5) != 0 { chans.push(param(5)); }
        }
        chans.sort(); chans.dedup();
        if chans.is_empty() { println!("== {a}: none =="); continue; }
        println!("== {a}: lock out channels {chans:02x?} ==");
        for archive in [a.to_string(), format!("0780{}.bin", &a[6..8]), format!("0790{}.bin", &a[6..8])] {
            let Ok((d2_raw, d2)) = std::panic::catch_unwind(|| {
                    let raw = read_archive(&iso, &archive);
                    let lb = LevelBin::parse(&raw).map_err(|_| ())?;
                    Ok::<_, ()>((raw, lb))
                }).map_err(|_| ()).and_then(|x| x) else { continue };
            for se in d2.entries.iter().filter(|e| e.kind == 7) {
                let Ok(nis) = Nis::parse(se.hash(), d2.data(&d2_raw, se)) else { continue };
                for s in nis.streams.iter() {
                    for c in s.decode() {
                        if let Cmd::Event { kind, params } = c {
                            if kind == 0xd || kind == 0x11 {
                                if params.iter().any(|&p| chans.contains(&p)) {
                                    println!("  script {hash:08x} stream event {kind:02x} params {params:02x?}", hash = se.hash());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
