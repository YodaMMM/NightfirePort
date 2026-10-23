//! Probe: every 07000016 placement whose params name channel 80 (the
//! lift summon), 240/241/242/243/244 (the launch window) or 123 (0x7b the fail) — triggers, ANDs,
//! blockers — with kind, pos, params. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let _ = LevelBin::parse(&data).unwrap();
    let level = Level::parse(&data).expect("level parses");
    let mi = level.main_map().unwrap();
    let m = &level.maps[mi];
    let watch = [80u32, 240, 241, 242, 243, 244, 123, 99, 98];
    for (i, p) in m.placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        // writer-ish kinds: triggers/ANDs/switches/blocks/doors
        if !matches!(kind, 0xe9..=0xf1 | 0x3b | 0xdb | 0x19) { continue; }
        let hits = p.params.iter().take(2).any(|v| watch.contains(&v.0) || watch.contains(&v.1));
        if !hits { continue; }
        println!("place[{i}] kind {kind:#04x} idx {} pos ({:.2},{:.2},{:.2}) params {:?}",
            p.index, p.translation[0], p.translation[1], p.translation[2], p.params);
    }
}
