//! level-4a-ride-bond-float probe 4: the pack 0x060006cf's raw key positions for the entity
//! streams (s7 satchel keys 133..594, s10 hatch, s12 P99, s13 lift floor keys 1405..1408) and the
//! camera (s0 keys 0..124) — the raw disc values, no compose. Opt-in via NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

#[test]
fn dump_the_pack_raw_keys() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = match IsoImage::open(&iso_path) { Ok(i) => i, Err(e) => { eprintln!("iso: {e}"); return } };
    let (entries, bin) = match disc::files_bin_entries(&iso) { Ok(x) => x, Err(e) => { eprintln!("FILES.BIN: {e}"); return } };
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = match iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) { Ok(d) => d, Err(_) => continue };
        let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(_) => continue };
        for e in lb.entries.iter() {
            if e.hash() != 0x0600_06cf { continue; }
            let d = lb.data(&data, e);
            let nis = match Nis::parse(e.hash(), d) { Ok(n) => n, Err(err) => { eprintln!("parse: {err}"); return } };
            eprintln!("pack {} keys total", nis.keys.len());
            let want: &[usize] = &[0, 1, 62, 124, 133, 134, 300, 594, 877, 878, 1405, 1406, 1407, 1408];
            for k in want.iter().copied() {
                let Some(key) = nis.keys.get(k) else { continue };
                eprintln!("key[{:04}] t {:9.3} pos ({:9.3}, {:9.3}, {:9.3})", k, key.time, key.pos[0], key.pos[1], key.pos[2]);
            }
        }
    }
}
