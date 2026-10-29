//! Census [D]: every SP script's kind-0x31 entity streams and their world-frame verdict under
//! the game's base rule — an unparented case-1 stream always concats si+0x00 [M 0x1d9c0c flag =
//! (parent == -1), 0x1da838-0x1da844 tail], and si+0x00 is the placement base from `Script_Load`
//! 0x1d8248:148 until `Script_CameraStart` (op 0x0d) overwrites it with the identity, permanently
//! for the script instance [M 0x1d916c; Script_CameraEnd 0x1db348 doesn't restore]. So a stream's
//! keys (and the event-6 hand-back snapshot of its entity) land in WORLD frame from the script's
//! first camera-op tick onward, composed before it.
//!
//! Per script with kind-0x31 Entity commands, this walks every stream's decoded commands with the
//! runner's timing (ops between Wait/End holds fire at the held clock), and prints: the first
//! camera-op tick, each 0x31 stream's execution tick / key window / parent, and each event-6
//! tick — everything needed to judge composed vs keys-alone per scene.
//!
//! Run: `NF_ISO=… cargo test -p nf-formats --test nis_kind31_census -- --nocapture`

use nf_formats::{disc, iso9660::IsoImage, level::Level, nis::Cmd};

fn key_time(nis: &nf_formats::nis::Nis, i: usize, first: bool) -> f32 {
    let s = &nis.streams[i];
    let k = if first { s.first_key } else { s.last_key.saturating_sub(1) };
    nis.keys.get(k as usize).map(|k| k.time).unwrap_or(0.0)
}

#[test]
fn census_sp_scripts_kind31_streams_and_camera_ticks() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let im = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    let mut scripts_with_31 = 0u32;
    // Every archive's kind-7 entries are the NIS scripts (the 079* packs included — SP scripts
    // live there, not in the level bins) [M level.rs's own nis-map build].
    let mut seen: std::collections::BTreeMap<u32, ()> = Default::default();
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = im.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(lb) = nf_formats::levelbin::LevelBin::parse(&data) else { continue };
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            let Ok(h) = u32::from_str_radix(&e.name, 16) else { continue };
            let Ok(nis) = nf_formats::nis::Nis::parse(h, lb.data(&data, e)) else { continue };
            if !seen.insert(h, ()).is_none() { continue; }
            // Static tick walk per stream: ops fire at the stream clock; Wait/End hold it at
            // their absolute time [M Script_Run 0x1d8548: the `clock < wait` gate + `next` walk].
            let mut cam_tick = f32::INFINITY;
            let mut ents31: Vec<(usize, f32, u32, Option<u8>)> = Vec::new();
            let mut ev6: Vec<(usize, f32)> = Vec::new();
            for (si, st) in nis.streams.iter().enumerate() {
                let mut clock = 0.0f32;
                for cmd in st.decode() {
                    match &cmd {
                        Cmd::Wait(t) | Cmd::End(t) => clock = clock.max(*t as f32),
                        Cmd::Camera { .. } => cam_tick = cam_tick.min(clock),
                        Cmd::Entity { hash, parent, obj_type: 0x31, .. } => ents31.push((si, clock, *hash, *parent)),
                        Cmd::Event { kind: 6, .. } => ev6.push((si, clock)),
                        _ => {}
                    }
                }
            }
            let has_cam = cam_tick.is_finite();
            if ents31.is_empty() { continue; }
            scripts_with_31 += 1;
            println!("script {h:#09x} (079 pack level {}): camera op tick {cam_tick}, event6 at {:?}",
                &fe.name[..fe.name.len().saturating_sub(4)], ev6);
            for (si, t, hash, parent) in &ents31 {
                let (k0, k1) = (key_time(&nis, *si, true), key_time(&nis, *si, false));
                let frame = if has_cam && *t >= cam_tick { "WORLD" } else { "composed" };
                println!("    s{si} kind31 hash {hash:#x} parent {parent:?} op@{t} keys {k0}..{k1} -> {frame} under the base rule");
            }
        }
    }
    println!("scripts with kind-0x31 streams: {scripts_with_31}");
}
