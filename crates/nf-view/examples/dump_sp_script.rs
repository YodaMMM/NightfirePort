//! Scratch: dump a level script's streams/commands — LEVEL=<bin name> HASH=<hex> ./dump_sp_script
use nf_formats::nis::Cmd;

fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let level = std::env::var("LEVEL").expect("LEVEL");
    let hash = u32::from_str_radix(std::env::var("HASH").expect("HASH").trim_start_matches("0x"), 16).unwrap();
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&level)).expect("level bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = nf_formats::levelbin::LevelBin::parse(&data).unwrap();
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        if se.hash() != hash { continue; }
        let nis = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)).unwrap();
        println!("script {:08x} len {} streams {}", nis.hashcode, nis.length, nis.streams.len());
        for (si, s) in nis.streams.iter().enumerate() {
            println!(" stream {si}: keys {}..{}", s.first_key, s.last_key);
            for c in s.decode() {
                match c {
                    Cmd::Entity { hash, parent, spline, obj_type, params } =>
                        println!("   Entity glist {hash:08x} parent {parent:?} spline {spline} type {obj_type:02x} params {params:?}"),
                    Cmd::Anim { skin, script, start } => println!("   Anim skin {skin:08x} script {script:08x} start {start}"),
                    Cmd::Event { kind, params } => println!("   Event kind {kind:02x} params {params:?}"),
                    other => println!("   {other:?}"),
                }
            }
        }
        for (ki, k) in nis.keys.iter().enumerate().take(nis.streams.len().max(1) * 0 + 40) {
            println!(" key {ki}: t {} pos {:?} scale {}", k.time, k.pos, k.scale);
        }
    }
}
