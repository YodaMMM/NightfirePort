//! fusebox-inverter-2 probe 2: across every level .bin on the disc, count main-map placements
//! whose kind byte is a worldobj/sound kind (0x41/0xd2/0x3b/0xe7/0x71/0xfb/0xf9) but whose flags
//! word carries the dispatcher's skip bits (`flags & 0xa000 != 0` [M parsemap_block_map_data_
//! dynamic 0x1d1b68 — the caller never reaches parsemap_create_dynamic_objects for those]) —
//! i.e. rows the port's WorldObjects::create (no skip) over-creates vs the game.
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let kinds = [0x41u32, 0xd2, 0x3b, 0xe7, 0x71, 0xfb, 0xf9];
    let mut total = 0usize;
    for e in entries.iter().filter(|e| {
        e.name.len() == 12 && e.name.ends_with(".bin") && e.name.starts_with("07")
    }) {
        let data = image
            .read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64)
            .expect("read");
        let Ok(lvl) = Level::parse(&data) else { continue };
        let Some(m) = lvl.main_map() else { continue };
        for p in &lvl.maps[m].placements {
            if p.flags & 0xa000 != 0 && kinds.contains(&(p.flags & 0xff)) {
                println!(
                    "{} #{} kind {:#x} flags {:#x}",
                    e.name, p.index, p.flags & 0xff, p.flags
                );
                total += 1;
            }
        }
    }
    println!("TOTAL affected rows: {total}");
}
