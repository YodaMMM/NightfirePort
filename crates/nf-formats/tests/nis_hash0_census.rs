//! Census [D]: every script's HASH-0 entity streams in camera-op scripts, for the two levels the
//! five failures run in (07800001 = 07000001's scripts, 07000008 = 07000008's). Prints per script:
//! the camera-op tick and each hash-0 Entity stream (parent, obj_type, op tick, key window) — the
//! candidates the general keys_alone rule flips from composed to keys-alone.
//!
//! Run: NF_ISO=… cargo test -p nf-formats --test nis_hash0_census -- --nocapture

use nf_formats::{disc, iso9660::IsoImage, nis::Cmd};

#[test]
fn census_hash0_entity_streams_in_camera_scripts() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let im = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    const LEVELS: [&str; 2] = ["07800001", "07000008"];
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let lvl = &fe.name[..fe.name.len().saturating_sub(4)];
        if !LEVELS.contains(&lvl) { continue; }
        let data = im.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(lb) = nf_formats::levelbin::LevelBin::parse(&data) else { continue };
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            let Ok(h) = u32::from_str_radix(&e.name, 16) else { continue };
            let Ok(nis) = nf_formats::nis::Nis::parse(h, lb.data(&data, e)) else { continue };
            let mut cam_tick = f32::INFINITY;
            let mut rows: Vec<(usize, f32, u8, Option<u8>, u16, u16)> = Vec::new();
            for (si, st) in nis.streams.iter().enumerate() {
                let mut clock = 0.0f32;
                for cmd in st.decode() {
                    match &cmd {
                        Cmd::Wait(t) | Cmd::End(t) => clock = clock.max(*t as f32),
                        Cmd::Camera { .. } => cam_tick = cam_tick.min(clock),
                        Cmd::Entity { hash: 0, parent, obj_type, .. } => {
                            rows.push((si, clock, *obj_type, *parent, st.first_key, st.last_key));
                        }
                        _ => {}
                    }
                }
            }
            if rows.is_empty() || !cam_tick.is_finite() { continue; }
            println!("script {h:#09x} ({lvl}): camera op tick {cam_tick}");
            for (si, t, ot, par, fk, lk) in &rows {
                println!("    s{si} hash0 obj_type {ot:#x} parent {par:?} op@{t} first_key {fk} last_key {lk}");
            }
        }
    }
}
