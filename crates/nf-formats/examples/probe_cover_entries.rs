//! cover-nodes-submap probe: dump each mission level's archive entries (order, kind, name)
//! and each map entry's 0xE5/0xE6 cover placements, to answer U-050's list-position question.
use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let iso = IsoImage::open(&iso_path)?;
    let (entries, bin) = disc::files_bin_entries(&iso)?;
    for level_hash in ["07000001", "07000002", "07000004", "07000005", "07000006"] {
        let fe = entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(&format!("{level_hash}.bin")))
            .ok_or(format!("{level_hash}.bin not found"))?;
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)?;
        let level = Level::parse(&data)?;
        println!("== {level_hash} ==");
        let lb = nf_formats::levelbin::LevelBin::parse(&data)?;
        for (i, e) in lb.entries.iter().enumerate() {
            let mark = if matches!(e.kind, 0 | 1 | 2 | 11) { "MAP" } else { "   " };
            println!("  entry {i:2} kind {:2} {mark} {}", e.kind, e.name);
        }
        for (mi, m) in level.maps.iter().enumerate() {
            let covers: Vec<_> = m
                .placements
                .iter()
                .filter(|p| matches!(p.flags & 0xff, 0xe5 | 0xe6))
                .collect();
            println!(
                "  map {mi} (entry {}, kind {}, name {}): {} placements",
                m.entry,
                m.kind,
                m.name,
                m.placements.len()
            );
            for p in covers {
                println!(
                    "    cover placement {}: type 0x{:02x} at ({:.2}, {:.2}, {:.2}) params {:?}",
                    p.index,
                    p.flags & 0xff,
                    p.translation[0],
                    p.translation[1],
                    p.translation[2],
                    p.params
                );
            }
        }
    }
    Ok(())
}
