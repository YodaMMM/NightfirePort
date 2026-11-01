//! pc-screen census: every script Entity stream (the case-1 streams `Script_Interp` 0x1d9a08 poses
//! and whose key +0x24 becomes obj+0x106, the draw's tweak alpha [M 0x1d9a08:99-104, Script_GetInterp
//! 0x1da158:186-189, View_SetupRenderModes_Tweak 0x1e5888:12,47]) on every level archive: how many
//! key all their keys below 1/255 (never drawn), how many ramp, how many hold 1.0.
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let (mut full, mut ramp, mut zero) = (0usize, 0usize, 0usize);
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().starts_with("070000") && e.name.to_ascii_lowercase().ends_with(".bin")) {
        let Ok(data) = image.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) else { continue };
        let Ok(lvl) = Level::parse(&data) else { continue };
        let mut hs: Vec<_> = lvl.nis.keys().copied().collect();
        hs.sort();
        for h in hs {
            let nis = &lvl.nis[&h];
            for (si, st) in nis.streams.iter().enumerate() {
                let Some((hash, ot)) = st.decode().into_iter().find_map(|c| match c {
                    nf_formats::nis::Cmd::Entity { hash, obj_type, .. } => Some((hash, obj_type)), _ => None }) else { continue };
                if matches!(ot, 0xea | 0xeb | 0xf5 | 0xf2) || hash == 0 { continue; }
                let (a, b) = (st.first_key as usize, st.last_key as usize);
                if b < a || b >= nis.keys.len() { continue; }
                let v: Vec<f32> = nis.keys[a..=b].iter().map(|k| k.f24()).collect();
                let lo = v.iter().cloned().fold(f32::MAX, f32::min);
                let hi = v.iter().cloned().fold(f32::MIN, f32::max);
                if hi < 0.5 / 255.0 { zero += 1; println!("{} {h:08x} s{si} {hash:08x} type {ot:#x}: all keys 0 ({} keys)", fe.name, v.len()); }
                else if lo >= 254.5 / 255.0 { full += 1; }
                else { ramp += 1; println!("{} {h:08x} s{si} {hash:08x} type {ot:#x}: ramps {lo:.3}..{hi:.3} ({} keys)", fe.name, v.len()); }
            }
        }
    }
    println!("entity streams: {full} hold 1.0, {ramp} ramp, {zero} all 0");
}
