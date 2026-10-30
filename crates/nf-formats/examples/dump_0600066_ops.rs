//! script-channel-events [D]: 06000066's stream opcodes — is there a camera op (0x0d)?
use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::Cmd;
fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    for archive in ["07000006.bin"] {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lb = LevelBin::parse(&data).expect("levelbin");
        for se in lb.entries.iter().filter(|e| e.kind == 7 && e.hash() == 0x0600_0066) {
            let nis = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)).expect("nis");
            println!("script {:08x}: len {} loop {} streams {}", nis.hashcode, nis.length, nis.loop_start, nis.streams.len());
            for (si, s) in nis.streams.iter().enumerate() {
                let ops: Vec<String> = s.decode().iter().map(|c| match c {
                    Cmd::End(_) => "end".into(), Cmd::Wait(t) => format!("wait{t}"), Cmd::Marker(_) => "mark".into(),
                    Cmd::Entity { .. } => "ent".into(), Cmd::Anim { .. } => "anim".into(),
                    Cmd::Event { kind, params } => format!("ev{kind:02x}({params:x?})"),
                    Cmd::Camera { .. } => "CAMERA".into(), Cmd::CameraEnd => "camend".into(),
                    Cmd::Sound { id, .. } => format!("snd{id:x}"), Cmd::SoundEnd => "sndend".into(),
                    Cmd::Fade(_) => "fade".into(), Cmd::Sprite { .. } => "spr".into(),
                    Cmd::SubScript { hash, .. } => format!("sub{hash:08x}"), Cmd::Text { .. } => "txt".into(),
                    other => format!("{other:?}"),
                }).collect();
                println!("  s{si}: {}", ops.join(" "));
            }
        }
    }
}
