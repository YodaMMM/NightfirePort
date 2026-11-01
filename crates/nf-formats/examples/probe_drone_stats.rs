//! Drone stats-section probe: for each level named on the command line (default 07000002 and
//! 07000009), every main-map drone placement (0x0F) whose behaviour words carry a stats tag
//! (`behaviour_util_get` 0x1c7a98), with its raw parameter words 15.. and the decoded records.
use nf_formats::drone::DroneStatsTable;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let mut names: Vec<String> = std::env::args().skip(1).collect();
    if names.is_empty() { names = vec!["07000002".into(), "07000009".into()]; }
    for name in names {
        let file = format!("{name}.bin");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&file)).expect("level");
        let data = image.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lvl = Level::parse(&data).expect("parse");
        let m = lvl.main_map().expect("main map");
        println!("=== {name} (main map {m})");
        for p in lvl.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x0f) {
            let param = |j: u32| p.params.iter().find(|(i, _)| *i == j).map(|&(_, v)| v).unwrap_or(0);
            let mut t = DroneStatsTable::default();
            let ty = t.behaviour_get(param);
            let raw: Vec<String> = (15..40).map(|j| format!("{j}:{:#x}", param(j))).collect();
            let has = t.0[ty as usize].iter().any(|r| r.iter().any(|&v| v != 0));
            println!("#{} diff {} type {ty} {}", p.index, param(2), if has { "SECTION" } else { "" });
            if has {
                println!("   raw {}", raw.join(" "));
                for k in 0..3 { println!("   kind {k}: {:?}", t.0[ty as usize][k]); }
                println!("   stats(1) {:?}", t.stats(ty));
            }
        }
    }
}
