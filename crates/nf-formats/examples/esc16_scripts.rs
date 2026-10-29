//! Probe [D]: dump every kind-7 script across the 07000016 packs, but print full command
//! traces only for the escape-relevant hashes (0600090a launch, 060009e9 lift, 0600090c/060008bc
//! exhaust SPs, 06000631/32/33 cinema) and, for the rest, only their event-0xd/0x11 channel ops.
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

const FULL: [u32; 7] = [0x0600090a, 0x060009e9, 0x0600090c, 0x060008bc, 0x06000631, 0x06000632, 0x06000633];

fn trace(hash: u32, n: &Nis, full: bool) {
    println!("=== script {hash:08x} length {} streams {} keys {}", n.length, n.streams.len(), n.keys.len());
    for (si, s) in n.streams.iter().enumerate() {
        let mut clock = 0.0f32;
        let mut parts = Vec::new();
        for c in &s.decode() {
            match c {
                nf_formats::nis::Cmd::Wait(t) => { clock += *t as f32; parts.push(format!("wait{t}@{:.0}", clock)); }
                nf_formats::nis::Cmd::Event { kind, params } => {
                    if full || *kind == 0x0d || *kind == 0x11 || *kind == 6 {
                        parts.push(format!("EVENT {kind:#04x} p{:?}@{:.0}", params, clock));
                    }
                }
                nf_formats::nis::Cmd::Entity { hash, .. } => parts.push(format!("ENT {hash:08x}")),
                nf_formats::nis::Cmd::EntityEnd => parts.push("entEnd".into()),
                nf_formats::nis::Cmd::Anim { script, .. } => parts.push(format!("ANIM {script:08x}")),
                nf_formats::nis::Cmd::AnimEnd => parts.push("animEnd".into()),
                nf_formats::nis::Cmd::SubScript { hash, stream, mode } => parts.push(format!("SUB {hash:08x} s{stream} m{mode}")),
                nf_formats::nis::Cmd::End(_) => parts.push("end".into()),
                nf_formats::nis::Cmd::Marker(_) => {}
                _ => parts.push("other".into()),
            }
        }
        println!("  s{si}: {}", parts.join(" | "));
    }
}

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for name in ["07000016.bin", "07800016.bin", "07900016.bin", "07a00016.bin", "07F00016.bin"] {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(_) => continue };
        let mut hashes = Vec::new();
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            let Ok(h) = u32::from_str_radix(&e.name, 16) else { continue };
            hashes.push(h);
        }
        hashes.sort();
        for h in hashes {
            let e = lb.entries.iter().find(|e| e.kind == 7 && u32::from_str_radix(&e.name, 16) == Ok(h)).unwrap();
            if let Ok(n) = Nis::parse(h, lb.data(&data, e)) {
                trace(h, &n, FULL.contains(&h));
            }
        }
    }
}
