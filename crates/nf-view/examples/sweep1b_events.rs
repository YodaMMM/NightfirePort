//! level-1b-scripted-carry: the [D] sweep tool — every 0700001b kind-7 script decoded through
//! the Nis parser, printing every player-touching event row (kinds 4/5/6/8/9/0xb/0x12) with its
//! tick. Result: exactly one row on the level (0x06000a20 s4 ev05). LEVEL=<bin> to sweep others.
use nf_formats::nis::Cmd;

fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let level = std::env::var("LEVEL").unwrap_or_else(|_| "0700001b.bin".into());
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&level)).expect("level bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = nf_formats::levelbin::LevelBin::parse(&data).unwrap();
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let nis = match nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) {
            Ok(n) => n,
            Err(e) => { println!("script {:08x} PARSE-ERR {e}", se.hash()); continue; }
        };
        let mut hits = Vec::new();
        for (si, s) in nis.streams.iter().enumerate() {
            let mut t: u32 = 0;
            for c in s.decode() {
                match c {
                    Cmd::Wait(w) => t += w as u32,
                    Cmd::Marker(m) => { if m as u32 > t { t = m as u32; } }
                    Cmd::Event { kind, params } => {
                        if matches!(kind, 4 | 6 | 9 | 0xb | 5 | 8 | 0x12) {
                            hits.push(format!("s{si} t{t} ev{kind:02x} {params:?}"));
                        }
                    }
                    Cmd::End(_) => break,
                    _ => {}
                }
            }
        }
        if !hits.is_empty() {
            println!("script {:08x} len {} streams {} :: {}", nis.hashcode, nis.length, nis.streams.len(), hits.join(" | "));
        }
    }
}
