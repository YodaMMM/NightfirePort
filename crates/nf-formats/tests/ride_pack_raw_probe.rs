//! level-4a-ride-bond-float probe 2: the pack 0x060006cf's raw stream table from the disc —
//! per stream: key range + decoded commands (op, entity hash/parent, anim script/skin, events).
//! Answers which streams carry parents and what Bond's (s2) stream commands say. Opt-in via NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

#[test]
fn dump_the_pack_raw_streams() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = match IsoImage::open(&iso_path) { Ok(i) => i, Err(e) => { eprintln!("iso: {e}"); return } };
    let (entries, bin) = match disc::files_bin_entries(&iso) { Ok(x) => x, Err(e) => { eprintln!("FILES.BIN: {e}"); return } };
    for fe in entries.iter().filter(|e| e.name.eq_ignore_ascii_case("0780004a.bin")) {
        let data = match iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) { Ok(d) => d, Err(e) => { eprintln!("read: {e}"); return } };
        let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(e) => { eprintln!("bin: {e}"); return } };
        for e in lb.entries.iter() {
            if e.hash() != 0x0600_06cf { continue; }
            let d = lb.data(&data, e);
            let nis = match Nis::parse(e.hash(), d) { Ok(n) => n, Err(err) => { eprintln!("parse: {err}"); return } };
            eprintln!("pack {:08x} camera {} loop {} len {} streams {}", e.hash(), nis.camera, nis.loop_start, nis.length, nis.streams.len());
            for (i, s) in nis.streams.iter().enumerate() {
                eprintln!("s{:<2} keys {}..{} cmds {:02x?}", i, s.first_key, s.last_key, &s.commands[..s.commands.len().min(24)]);
            }
        }
    }
}
