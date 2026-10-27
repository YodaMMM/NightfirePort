//! level-4a-ride-bond-float probe 3: the lift script 0x06000571's cab stream (s0, entity
//! 0x020005b4) — the keyed y around the landing window (t1550..t2300) and the hand-back marker.
//! Answers where the cab is held while the pack 0x060006cf plays. Opt-in via NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

#[test]
fn dump_the_cab_keys() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = match IsoImage::open(&iso_path) { Ok(i) => i, Err(e) => { eprintln!("iso: {e}"); return } };
    let (entries, bin) = match disc::files_bin_entries(&iso) { Ok(x) => x, Err(e) => { eprintln!("FILES.BIN: {e}"); return } };
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = match iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) { Ok(d) => d, Err(_) => continue };
        let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(_) => continue };
        for e in lb.entries.iter() {
            if e.hash() != 0x0600_0571 { continue; }
            eprintln!("found lift {:08x} in {}", e.hash(), fe.name);
            let d = lb.data(&data, e);
            let nis = match Nis::parse(e.hash(), d) { Ok(n) => n, Err(err) => { eprintln!("parse: {err}"); return } };
            eprintln!("lift {:08x} camera {} len {} streams {}", e.hash(), nis.camera, nis.length, nis.streams.len());
            for (i, s) in nis.streams.iter().enumerate() {
                eprintln!("s{:<2} keys {}..{} cmds {:02x?}", i, s.first_key, s.last_key, &s.commands[..s.commands.len().min(14)]);
            }
            // The keys: the cab's keyed positions — dump every key whose time falls in the
            // landing window plus the ends.
            for (k, key) in nis.keys.iter().enumerate() {
                let t = key.time;
                if k < 3 || (t >= 1500.0 && t <= 2400.0) || k + 3 > nis.keys.len() {
                    eprintln!("key[{:03}] t {:9.3} pos ({:.3}, {:.3}, {:.3})", k, t, key.pos[0], key.pos[1], key.pos[2]);
                }
            }
        }
    }
}
