//! start-corridor-guard-2 probe: dump every drone placement (kind 0xF) and AIPoint (0xE9) on
//! 07000011 with all params — the [D] evidence for the corridor guards' behaviour words
//! (prop 0x20 = word1 bits 0-1 = param 19; prop 0x56 = word2 bit 26 = param 20; alert block
//! params 23/24/25) and the group-0 AIPoints' owners.
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let name = "07000011.bin";
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("{name} not in FILES.BIN"));
    let data = image
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    let lvl = Level::parse(&data).expect("parse");
    for (mi, m) in lvl.maps.iter().enumerate() {
        for p in &m.placements {
            let low = p.flags & 0xffff;
            if low != 0xf && low != 0xe9 {
                continue;
            }
            let ps: Vec<String> = p.params.iter().map(|(k, v)| format!("{k}:{v:#x}")).collect();
            let name = m
                .objects
                .iter()
                .find(|o| o.hashcode == p.hashcode)
                .map(|o| o.name.clone())
                .unwrap_or_default();
            if low == 0xf {
                // behaviour words: normal = params 18/19/20, alert = 23/24/25
                let pv = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
                println!(
                    "map{mi} #{} DRONE flags {:#x} \"{name}\" @({:.2},{:.2},{:.2}) p2(dif) {:#x} p4 {:#x} NORMAL p18 {:#010x} p19 {:#010x} p20 {:#010x} ALERT p23 {:#010x} p24 {:#010x} p25 {:#010x} all [{}]",
                    p.index,
                    p.flags,
                    p.translation[0],
                    p.translation[1],
                    p.translation[2],
                    pv(2),
                    pv(4),
                    pv(18),
                    pv(19),
                    pv(20),
                    pv(23),
                    pv(24),
                    pv(25),
                    ps.join(" ")
                );
            } else {
                println!(
                    "map{mi} #{} AIPNT flags {:#x} \"{name}\" @({:.2},{:.2},{:.2}) all [{}]",
                    p.index,
                    p.flags,
                    p.translation[0],
                    p.translation[1],
                    p.translation[2],
                    ps.join(" ")
                );
            }
        }
    }
}
