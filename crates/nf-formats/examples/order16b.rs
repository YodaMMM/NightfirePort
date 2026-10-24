//! Probe 2: the cross-reference the order16 pass left open — every placement carrying a chain
//! script hash (0x06000631/0x06000632/0x06000633/0x0600090a) as a param, every placement naming
//! ch 129 (0x81) / 185 / 243 in a channel-reader kind, and every raw 0xDC trigger's type byte.
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn param(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0)
}

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).expect("level parses");
    let m = &level.maps[level.main_map().unwrap()];

    let scripts = [0x06000631u32, 0x06000632, 0x06000633, 0x0600090a, 0x0600090c, 0x060009e9, 0x060009f6, 0x060008ba, 0x06000999];
    println!("## placements carrying a chain script hash as a param");
    for (i, p) in m.placements.iter().enumerate() {
        for (k, v) in &p.params {
            if scripts.contains(v) {
                println!("place[{i}] kind {:#04x} param[{k}] = {v:08x} pos {:?}", p.flags & 0xff, p.translation);
            }
        }
    }
    println!("\n## channel-reader placements naming 129/185/187/240/242/243/244/80/82 (kinds that read channels)");
    let watch = [129u32, 185, 187, 240, 242, 243, 244, 80, 82, 180, 181];
    for (i, p) in m.placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        let reader = matches!(kind, 0xe9..=0xf1 | 0xdc | 0xdb | 0x29 | 0xe4 | 0xd9);
        if !reader { continue; }
        let idx = if kind == 0xdb { 3 } else if kind == 0xe4 { 2 } else { 0 };
        // which param slots are channels for this kind: triggers out=p0(inputs p1..p6,p8,mask p9);
        // door ch_lock=p3; hurt gate=p2; SP out=p6 gate=p9; switch out=p0 gate=p5.
        let chans: Vec<(u32, u32)> = match kind {
            0xdb => vec![(3, param(p, 3))],
            0xe4 => vec![(2, param(p, 2))],
            0xd9 => vec![(6, param(p, 6)), (9, param(p, 9))],
            0x29 => vec![(0, param(p, 0)), (5, param(p, 5))],
            0xe9..=0xf1 | 0xdc => (0..=9).map(|k| (k, param(p, k))).collect(),
            _ => vec![],
        };
        let hits: Vec<(u32, u32)> = chans.into_iter().filter(|(_, v)| watch.contains(v)).collect();
        if hits.is_empty() { continue; }
        println!("place[{i}] kind {kind:#04x} pos {:?} hits {:?} params {:?}", p.translation, hits, p.params);
    }
    println!("\n## raw 0xDC placements");
    for (i, p) in m.placements.iter().enumerate() {
        if p.flags & 0xff == 0xdc {
            println!("place[{i}] type-byte {} pos {:?} params {:?}", param(p, 0), p.translation, p.params);
        }
    }
    println!("\n## objective/switch placements (kind 0x29) and their gate/out");
    for (i, p) in m.placements.iter().enumerate() {
        if p.flags & 0xff == 0x29 {
            println!("switch place[{i}] idx {} pos {:?} out {} gate {} script {:08x} params {:?}",
                p.index, p.translation, param(p, 0), param(p, 5), param(p, 1), p.params);
        }
    }
}
