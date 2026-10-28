//! level-4a-ride-bond-float probe: every 07000008 NIS script — whether any stream carries a
//! Camera command (op 0x0d) and which carry Entity/Anim streams. The world-alone rule only
//! touches scripts whose camera op runs before entity streams key. Opt-in via NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::{Cmd, Nis}};

#[test]
fn dump_07000008_script_camera_layouts() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = match IsoImage::open(&iso_path) { Ok(i) => i, Err(e) => { eprintln!("iso: {e}"); return } };
    let (entries, bin) = match disc::files_bin_entries(&iso) { Ok(x) => x, Err(e) => { eprintln!("FILES.BIN: {e}"); return } };
    for fe in entries.iter().filter(|e| e.name.eq_ignore_ascii_case("07800008.bin")) {
        let data = match iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) { Ok(d) => d, Err(e) => { eprintln!("read: {e}"); return } };
        let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(e) => { eprintln!("bin: {e}"); return } };
        for e in lb.entries.iter() {
            if e.hash() & 0xff00_0000 != 0x0600_0000 { continue; }
            let d = lb.data(&data, e);
            let Ok(nis) = Nis::parse(e.hash(), d) else { continue };
            let mut cam_streams: Vec<usize> = Vec::new();
            let mut ent_streams: Vec<usize> = Vec::new();
            for (i, s) in nis.streams.iter().enumerate() {
                for c in s.decode() {
                    match c {
                        Cmd::Camera { .. } => { cam_streams.push(i); break }
                        Cmd::Entity { .. } | Cmd::Anim { .. } => { ent_streams.push(i); break }
                        _ => {}
                    }
                }
            }
            eprintln!("{:08x} flag_cam {} len {} streams {} cam@{:?} ent/anim@{:?}",
                e.hash(), nis.camera, nis.length, nis.streams.len(), cam_streams, ent_streams);
        }
    }
}
