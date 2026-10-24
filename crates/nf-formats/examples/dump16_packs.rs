//! Probe: dump the 07900016 (06000632) and 07a00016 (06000633) pack
//! scripts the round-2 probe skipped, plus the 07F00016.bin world pack if it carries scripts.
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for name in ["07900016.bin", "07a00016.bin", "07F00016.bin"] {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)) else {
            println!("## {name}: not on disc"); continue;
        };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(e) => { println!("## {name}: parse failed {e}"); continue } };
        println!("\n######## {name}: {} entries", lb.entries.len());
        let mut scripts: Vec<_> = lb.entries.iter().filter(|e| e.kind == 7).collect();
        scripts.sort_by_key(|e| u32::from_str_radix(&e.name, 16).unwrap_or(0));
        for e in &scripts {
            let hash = u32::from_str_radix(&e.name, 16).unwrap_or(0);
            match Nis::parse(hash, lb.data(&data, e)) {
                Ok(n) => {
                    println!("=== {name} script {hash:08x} length {} streams {} keys {}", n.length, n.streams.len(), n.keys.len());
                    for (si, s) in n.streams.iter().enumerate() {
                        let mut clock = 0.0f32;
                        let mut parts = Vec::new();
                        for c in &s.decode() {
                            match c {
                                nf_formats::nis::Cmd::Wait(t) => { clock += *t as f32; parts.push(format!("wait{t}@{:.0}", clock)); }
                                nf_formats::nis::Cmd::Event { kind, params } => parts.push(format!("EVENT {kind:#04x} params{:?}", params)),
                                nf_formats::nis::Cmd::Entity { hash, .. } => parts.push(format!("ENTITY {hash:08x}")),
                                nf_formats::nis::Cmd::EntityEnd => parts.push("entityEnd".into()),
                                nf_formats::nis::Cmd::Anim { script, .. } => parts.push(format!("ANIM {script:08x}")),
                                nf_formats::nis::Cmd::AnimEnd => parts.push("animEnd".into()),
                                nf_formats::nis::Cmd::SubScript { hash, stream, mode } => parts.push(format!("SUB {hash:08x} s{stream} m{mode}")),
                                nf_formats::nis::Cmd::End(_) => parts.push("end".into()),
                                nf_formats::nis::Cmd::Marker(_) => {}
                                _ => parts.push("other".into()),
                            }
                        }
                        let kr = if s.last_key >= s.first_key && (s.last_key as usize) < n.keys.len() {
                            let k0 = &n.keys[s.first_key as usize]; let k1 = &n.keys[s.last_key as usize];
                            format!(" keys{}..{} t{:.1}..{:.1} p0({:.2},{:.2},{:.2})", s.first_key, s.last_key, k0.time, k1.time, k0.pos[0], k0.pos[1], k0.pos[2])
                        } else { " (no keys)".into() };
                        if !parts.is_empty() || s.last_key >= s.first_key {
                            println!("  s{si}:{kr} :: {}", parts.join(" "));
                            if s.last_key >= s.first_key && (s.last_key as usize) < n.keys.len() {
                                for k in s.first_key..=s.last_key.min(s.first_key + 8) {
                                    let key = &n.keys[k as usize];
                                    println!("      key{k} t{:.1} pos({:.2},{:.2},{:.2})", key.time, key.pos[0], key.pos[1], key.pos[2]);
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("=== {name} script {hash:08x}: PARSE FAILED {e}"),
            }
        }
    }
}
