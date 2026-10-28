//! Dump [D]: the full stream/command layout of the scripts involved in the five gondola/escort
//! failures — for each stream: every op with its clock (the runner's timing: ops between Wait/End
//! holds fire at the held clock [M Script_Run 0x1d8548]), the Entity hashes/parents/obj_types, the
//! camera op, the SubScript ops and the event ops. Prints the per-stream op ORDER inside a tick so
//! the same-tick camera-vs-keys order is visible (the census's script-level camera tick hides it).
//!
//! Run: NF_ISO=… cargo test -p nf-formats --test nis_stream_dump -- --nocapture

use nf_formats::{disc, iso9660::IsoImage, nis::Cmd};

fn key_time(nis: &nf_formats::nis::Nis, i: usize, first: bool) -> f32 {
    let s = &nis.streams[i];
    let k = if first { s.first_key } else { s.last_key.saturating_sub(1) };
    nis.keys.get(k as usize).map(|k| k.time).unwrap_or(0.0)
}

#[test]
fn dump_streams_of_interest() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let im = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    // The hashes named by the five failures / frame-2's report, and any script containing them.
    const WANT_HASHES: [u32; 3] = [0x0600_0087, 0x0600_004c, 0x0600_0077];
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = im.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(lb) = nf_formats::levelbin::LevelBin::parse(&data) else { continue };
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            let Ok(h) = u32::from_str_radix(&e.name, 16) else { continue };
            if !WANT_HASHES.contains(&h) { continue; }
            let Ok(nis) = nf_formats::nis::Nis::parse(h, lb.data(&data, e)) else { continue };
            println!("=== script {h:#09x} in {} (streams {}, keys {}, length {}) ===",
                &fe.name[..fe.name.len().saturating_sub(4)], nis.streams.len(), nis.keys.len(), nis.length);
            for (si, st) in nis.streams.iter().enumerate() {
                let (k0, k1) = (key_time(&nis, si, true), key_time(&nis, si, false));
                println!("  s{si}: keys {k0}..{k1} (first_key {} last_key {})", st.first_key, st.last_key);
                let mut clock = 0.0f32;
                for cmd in st.decode() {
                    let line = match &cmd {
                        Cmd::Wait(t) => { clock = clock.max(*t as f32); format!("    op@{clock} Wait {t}") }
                        Cmd::End(t) => { clock = clock.max(*t as f32); format!("    op@{clock} End {t}") }
                        Cmd::Marker(t) => format!("    op@{clock} Marker {t}"),
                        Cmd::Entity { hash, parent, spline, obj_type, params } => format!(
                            "    op@{clock} Entity hash {hash:#x} parent {parent:?} spline {spline} obj_type {obj_type:#x} params {:?}",
                            params.iter().map(|p| format!("{p:#x}")).collect::<Vec<_>>()),
                        Cmd::EntityEnd => format!("    op@{clock} EntityEnd"),
                        Cmd::Anim { script, skin, start } => format!("    op@{clock} Anim script {script:#x} skin {skin:#x} start {start:#x}"),
                        Cmd::AnimEnd => format!("    op@{clock} AnimEnd"),
                        Cmd::Event { kind, params } => format!(
                            "    op@{clock} Event kind {kind:#x} params {:?}", params.iter().map(|p| format!("{p:#x}")).collect::<Vec<_>>()),
                        Cmd::Camera { spline, first, last } => format!("    op@{clock} CAMERA spline {spline} keys {first}..{}", last.wrapping_sub(1)),
                        Cmd::CameraEnd => format!("    op@{clock} CameraEnd"),
                        Cmd::SubScript { hash, stream, mode } => format!("    op@{clock} SubScript hash {hash:#x} stream {stream} mode {mode}"),
                        Cmd::Fade(f) => format!("    op@{clock} Fade {f}"),
                        Cmd::Sprite { rect, flags, hash } => format!("    op@{clock} Sprite {rect:?} flags {flags:#x} hash {hash:#x}"),
                        Cmd::Light { parent, kind } => format!("    op@{clock} Light parent {parent:?} kind {kind:#x}"),
                        Cmd::Text { label, time } => format!("    op@{clock} Text label {label:#x} time {time}"),
                        Cmd::Sound { id, flat, has_obj, stream } => format!("    op@{clock} Sound id {id:#x} flat {flat} obj {has_obj} stream {stream}"),
                        Cmd::SoundEnd => format!("    op@{clock} SoundEnd"),
                        Cmd::Skip { op } => format!("    op@{clock} Skip op {op:#x}"),
                        Cmd::Other(op) => format!("    op@{clock} Other {op:#x}"),
                        Cmd::Unread(op) => format!("    op@{clock} UNREAD {op:#x} (decode stops)"),
                    };
                    println!("{line}");
                }
            }
        }
    }
}
