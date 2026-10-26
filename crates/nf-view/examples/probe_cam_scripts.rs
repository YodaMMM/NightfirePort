//! Probe: for the 07000001 and 07000014 archives — every script that contains a camera op (0x0d),
//! listing its entity streams: stream index, parent, obj_type, hash, key range, first key pos.
//! Decides the scoping of the camera-started keys-alone rule. Scratch print (not committed).
use std::collections::BTreeMap;

fn main() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    for e in entries.iter() {
        let name = e.name.clone();
        let lower = name.to_ascii_lowercase();
        if !(lower.len() == 12 && lower.ends_with(".bin") && lower[..8].bytes().all(|b| b.is_ascii_hexdigit())) { continue; }
        let level = u32::from_str_radix(&lower[4..8], 16).unwrap_or(0); // e.g. 0001 / 0014
        let Ok(data) = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64) else { continue };
        let Ok(lb) = nf_formats::levelbin::LevelBin::parse(&data) else { continue };
        for se in lb.entries.iter().filter(|e| e.kind == 7) {
            let h = se.hash();
            let Ok(nis) = nf_formats::nis::Nis::parse(h, lb.data(&data, se)) else { continue };
            // Any camera op in any stream?
            let mut cam_streams = Vec::new();
            for (si, s) in nis.streams.iter().enumerate() {
                for cmd in s.decode() {
                    if let nf_formats::nis::Cmd::Camera { .. } = cmd { cam_streams.push(si); break; }
                }
            }
            if cam_streams.is_empty() { continue; }
            let mut lines = Vec::new();
            for (si, s) in nis.streams.iter().enumerate() {
                for cmd in s.decode() {
                    if let nf_formats::nis::Cmd::Entity { hash, parent, obj_type, params, .. } = cmd {
                        let k0 = nis.keys.get(s.first_key as usize).map(|k| k.pos).unwrap_or([0.0; 4]);
                        lines.push(format!(
                            "    s{si:2} parent {:>4} obj_type {obj_type:#04x} hash {hash:#010x} keys {}..{} first ({:.3},{:.3},{:.3}) params {:?}",
                            parent.map(|p| p.to_string()).unwrap_or("-".into()),
                            s.first_key, s.last_key, k0[0], k0[1], k0[2],
                            params.iter().take(4).collect::<Vec<_>>()));
                    }
                }
            }
            if lines.is_empty() { continue; }
            println!("{} script {h:#010x} streams {} camera in {cam_streams:?} [{}]", name, nis.streams.len(), lines.len());
            for l in lines { println!("{l}"); }
        }
    }
}
