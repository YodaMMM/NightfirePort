//! fusebox-inverter-2 probe: dump every placement whose flags (low half) are 0x41 or 0xF9 on the
//! levels the task names (07000014 / 0700000c / 07000041), with all params — the [D] evidence for
//! which placements the game's InverterTrigger_Create 0x1b3040 (case 0x41 [M 0x1d04f0:265-268])
//! and SoundTrigger_Create 0x1b2628 (case 0xf9 [M 0x1d04f0:504-510]) really build from.
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    for name in ["07000014.bin", "0700000c.bin", "07000041.bin"] {
        let fe = entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| panic!("{name} not in FILES.BIN"));
        let data = image
            .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
            .expect("read");
        let lvl = Level::parse(&data).expect("parse");
        println!("=== {name}");
        for (mi, m) in lvl.maps.iter().enumerate() {
            for p in &m.placements {
                let low = p.flags & 0xffff;
                if low == 0x41 || low == 0xf9 {
                    let ps: Vec<String> =
                        p.params.iter().map(|(k, v)| format!("{k}:{v:#x}")).collect();
                    let name = m
                        .objects
                        .iter()
                        .find(|o| o.hashcode == p.hashcode)
                        .map(|o| o.name.clone())
                        .unwrap_or_default();
                    println!(
                        "map{mi} #{} kind {:#x} flags {:#x} \"{name}\" @({:.2},{:.2},{:.2}) params [{}]",
                        p.index,
                        low,
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
}
