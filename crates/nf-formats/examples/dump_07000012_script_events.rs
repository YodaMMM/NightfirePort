//! level-12-audit: sweep every kind-7 script in 07000012.bin and print the channel-writing script
//! events (op 0x12 kinds 0xd / 0x11 / 0x12 — `Script_EventHandler` 0x1d94c0's channel and level
//! writers) with the owning script/stream/key [D]. Prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::nis::Cmd;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("07000012.bin"))
        .expect("archive");
    let data = img
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("levelbin");
    let mut count = 0usize;
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        count += 1;
        for (si, s) in nis.streams.iter().enumerate() {
            for c in s.decode() {
                if let Cmd::Event { kind, params } = c {
                    if matches!(kind, 0xd | 0x11 | 0x12) {
                        println!(
                            "script {:08x} stream {si} event {kind:#04x} params {:x?}",
                            nis.hashcode, params
                        );
                    }
                }
            }
        }
    }
    println!("scripts scanned: {count}");
}
