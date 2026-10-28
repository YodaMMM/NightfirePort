//! level-14-panel-sequence probe: dump 07000014's escape sequence — pack 07A00014.bin
//! script 0x06000848 (the SP whose trigger is ch 0x9), every stream's ops with the full
//! event params, every key time/pos, every sound, so the chain the two fusebox panels
//! start can be pinned [D]. Prints only.

use nf_formats::iso9660::IsoImage;
use nf_formats::nis::Cmd;

fn read_file(img: &IsoImage, name: &str) -> Vec<u8> {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).expect("FILES.BIN");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("{name} not in FILES.BIN"));
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read")
}

fn dump_pack(img: &IsoImage, pack: &str, hashes: &[u32]) {
    let data = read_file(img, pack);
    let lb = nf_formats::levelbin::LevelBin::parse(&data).unwrap_or_else(|e| panic!("{pack}: {e:?}"));
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        if !hashes.is_empty() && !hashes.contains(&se.hash()) {
            continue;
        }
        let nis = match nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) {
            Ok(n) => n,
            Err(e) => {
                println!("== {pack} {:08x} PARSE ERR {e:?} ==", se.hash());
                continue;
            }
        };
        println!(
            "== {pack} script {:08x}: {} streams, {} keys ==",
            nis.hashcode,
            nis.streams.len(),
            nis.keys.len()
        );
        for (si, s) in nis.streams.iter().enumerate() {
            println!("--- stream {si}: keys {}..{} ---", s.first_key, s.last_key);
            for c in s.decode() {
                match c {
                    Cmd::Entity { hash, parent, spline, obj_type, params } => println!(
                        "  t? ENTITY hash={hash:08x} type={obj_type:#04x} parent={parent:?} spline={spline} params={params:x?}"
                    ),
                    Cmd::Anim { script, skin, start } => {
                        println!("  t? ANIM skin={skin:08x} script={script:08x} start={start}")
                    }
                    Cmd::Camera { spline, first, last } => {
                        println!("  t? CAMERA spline={spline} keys {first}..{last}")
                    }
                    Cmd::Event { kind, params } => {
                        println!("  t? EVENT kind={kind:#04x} params={params:x?}")
                    }
                    Cmd::Sound { id, flat, has_obj, stream } => println!(
                        "  t? SOUND id={id:08x} flat={flat} has_obj={has_obj} stream={stream}"
                    ),
                    Cmd::SubScript { hash, stream, mode } => {
                        println!("  t? SUBSCRIPT hash={hash:08x} stream={stream} mode={mode}")
                    }
                    Cmd::Wait(t) => println!("  WAIT {t}"),
                    Cmd::End(t) => println!("  END {t}"),
                    Cmd::Marker(t) => println!("  MARKER {t}"),
                    Cmd::Fade(f) => println!("  FADE {f}"),
                    Cmd::Text { label, time } => println!("  TEXT label={label:08x} time={time}"),
                    Cmd::Sprite { rect, flags, hash } => println!(
                        "  SPRITE hash={hash:08x} rect={rect:?} flags={flags:#06x}"
                    ),
                    Cmd::Light { parent, kind } => println!("  LIGHT parent={parent:?} kind={kind}"),
                    Cmd::EntityEnd | Cmd::AnimEnd | Cmd::SoundEnd | Cmd::CameraEnd => {}
                    Cmd::Other(op) => println!("  OTHER op={op:#04x}"),
                    Cmd::Skip { op } => println!("  SKIP op={op:#04x}"),
                    Cmd::Unread(op) => println!("  UNREAD op={op:#04x} (decoding stops)"),
                }
            }
        }
        for (ki, k) in nis.keys.iter().enumerate() {
            println!(
                "  key#{ki} t{:.0} ({:.4},{:.4},{:.4}) quat ({:.4},{:.4},{:.4},{:.4})",
                k.time, k.pos[0], k.pos[1], k.pos[2], k.quat[0], k.quat[1], k.quat[2], k.quat[3]
            );
        }
    }
}

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("all") => dump_pack(&img, "07A00014.bin", &[]),
        Some("list") => {
            let data = read_file(&img, "07A00014.bin");
            let lb = nf_formats::levelbin::LevelBin::parse(&data).unwrap();
            for (i, e) in lb.entries.iter().enumerate() {
                println!(
                    "entry {i}: kind {:#04x} hash {:08x} off {} len {}",
                    e.kind,
                    e.hash(),
                    e.offset,
                    e.length
                );
            }
        }
        Some("writers") => {
            // Every kind-7 script in the level archive and the pack files: the event-0xD/0x11
            // channel writes with their ticks, so the writers of a channel can be enumerated [D].
            for pack in ["07000014.bin", "07800014.bin", "07900014.bin", "07A00014.bin"] {
                let data = read_file(&img, pack);
                let lb = match nf_formats::levelbin::LevelBin::parse(&data) {
                    Ok(lb) => lb,
                    Err(e) => {
                        println!("{pack}: parse err {e:?}");
                        continue;
                    }
                };
                for se in lb.entries.iter().filter(|e| e.kind == 7) {
                    let nis = match nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) {
                        Ok(n) => n,
                        Err(e) => {
                            println!("{pack} {:08x}: parse err {e:?}", se.hash());
                            continue;
                        }
                    };
                    for (si, s) in nis.streams.iter().enumerate() {
                        let mut t = 0u32;
                        for c in s.decode() {
                            match c {
                                Cmd::Wait(w) | Cmd::Marker(w) => t = t.max(w as u32),
                                Cmd::Event { kind, params } if kind == 0xd || kind == 0x11 => {
                                    println!("{pack} {:08x} s{si} t{t}: event {kind:#04x} {params:x?}", se.hash());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        _ => dump_pack(&img, "07A00014.bin", &[0x0600_0848]),
    }
}
