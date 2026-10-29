//! script-channel-events: scan every level archive's type-0xC pack records (hash, flag) [D] —
//! flags > 3 would be packs the port's loader skips (lib.rs skips flag > 3). Prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin::LevelBin;
fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    for e in entries.iter().filter(|e| {
        let n = e.name.to_ascii_uppercase();
        n.starts_with("0700") && n.ends_with(".BIN") && n.len() == 12
    }) {
        let Ok(data) = img.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64) else { continue };
        let Ok(lb) = LevelBin::parse(&data) else { continue };
        let mut flags: Vec<String> = Vec::new();
        for se in lb.entries.iter().filter(|s| s.kind == 0xC) {
            let r = lb.data(&data, se);
            if r.len() < 5 { continue; }
            let hash = u32::from_le_bytes([r[0], r[1], r[2], r[3]]);
            flags.push(format!("{:08x}/{}", hash, r[4]));
        }
        if !flags.is_empty() { println!("{}: {}", e.name.to_ascii_uppercase(), flags.join(" ")); }
    }
}
