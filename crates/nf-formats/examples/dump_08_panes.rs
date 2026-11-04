//! level-08-pane-shootout probe: the 07000008 pane timeline.
//! Dumps every Entity op with obj_type 0x20 (Break_Create panes) in every kind-7 script of
//! 07000008.bin (level bin + packs), with stream index, op tick, hash, obj_type, params, parent;
//! every EntityEnd op with its tick; every event 0xC tick (the keep-objects arm
//! [M Script_EventHandler 0x1d94c0 case 0xC]); and every event 0xa/0x10/0x13.
//! Prints only. [D]
use nf_formats::iso9660::IsoImage;
use nf_formats::nis::Cmd;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000008.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("levelbin");
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        let mut hits = 0usize;
        for (si, s) in nis.streams.iter().enumerate() {
            let mut clock = 0u32;
            let mut stream_created = false;
            for c in s.decode() {
                match c {
                    Cmd::End(t) | Cmd::Wait(t) | Cmd::Marker(t) => clock += t as u32,
                    Cmd::Entity { hash, parent, spline, obj_type, params } => {
                        stream_created = true;
                        if obj_type == 0x20 || obj_type == 31 {
                            println!("script {:08x} s{si} t{clock} Entity hash {hash:08x} type {obj_type:#x} parent {:?} spline {spline} params {:x?}",
                                nis.hashcode, parent, spline);
                            hits += 1;
                        }
                    }
                    Cmd::EntityEnd => {
                        if stream_created {
                            println!("script {:08x} s{si} t{clock} EntityEnd", nis.hashcode);
                            hits += 1;
                        }
                    }
                    Cmd::Event { kind, params } => {
                        if matches!(kind, 0xC | 0xA | 0x10 | 0x13) {
                            println!("script {:08x} s{si} t{clock} event {kind:#04x} params {:x?}", nis.hashcode, params);
                            hits += 1;
                        }
                    }
                    Cmd::SubScript { hash, stream, mode } => {
                        println!("script {:08x} s{si} t{clock} SubScript {:08x} stream {stream} mode {mode:#x}", nis.hashcode, hash);
                    }
                    _ => {}
                }
            }
        }
        if hits > 0 { println!("--- script {:08x}: {hits} pane-relevant ops, {} streams", nis.hashcode, nis.streams.len()); }
    }
}
