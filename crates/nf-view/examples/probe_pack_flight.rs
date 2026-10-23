//! Probe: the launch sequence's script truth. Dumps every kind-7 script
//! of 07000016.bin and the loadable pack 07800016.bin — per stream the decoded commands with a
//! cumulative Wait clock (the script tick), every key (pos/time) — so the Bond-actor streams and
//! the player-addressed events (0x04 disable, 0x06 snapshot, 0x0f handler call, 0x12 events) of
//! 0x0600090a "ShuttleLaunch" and 0x06000631 "the pack cinema" are visible. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

fn dump_script(n: &Nis, tag: &str) {
    println!("=== {tag} hash {:08x} length {} streams {} keys {}", n.hashcode, n.length, n.streams.len(), n.keys.len());
    for (si, s) in n.streams.iter().enumerate() {
        let mut clock = 0.0f32;
        let mut parts = Vec::new();
        let cmds = s.decode();
        if cmds.is_empty() && s.last_key >= s.first_key {
            parts.push(format!("<no commands, keys {}..{}>", s.first_key, s.last_key));
        }
        for c in &cmds {
            match c {
                nf_formats::nis::Cmd::Wait(t) => { clock += *t as f32; parts.push(format!("wait{t}")); }
                nf_formats::nis::Cmd::Marker(t) => parts.push(format!("marker{t}")),
                nf_formats::nis::Cmd::End(t) => parts.push(format!("end{t}")),
                nf_formats::nis::Cmd::Entity { hash, parent, spline, obj_type, params } => parts.push(format!(
                    "ENTITY {:08x} parent{:?} spline{} type{} params{:?} @t{:.0}", hash, parent, *spline as u8, obj_type, params, clock)),
                nf_formats::nis::Cmd::EntityEnd => parts.push("entityEnd".into()),
                nf_formats::nis::Cmd::Anim { script, skin, start } => parts.push(format!("ANIM {:08x} skin{:08x} start{}", script, skin, start)),
                nf_formats::nis::Cmd::AnimEnd => parts.push("animEnd".into()),
                nf_formats::nis::Cmd::Event { kind, params } => parts.push(format!("EVENT {kind:#04x} params{:?} @t{:.0}", params, clock)),
                nf_formats::nis::Cmd::Sound { id, flat, has_obj, stream } => parts.push(format!("SOUND {:08x} flat{} obj{} stream{} @t{:.0}", id, *flat as u8, *has_obj as u8, stream, clock)),
                nf_formats::nis::Cmd::SoundEnd => parts.push("soundEnd".into()),
                nf_formats::nis::Cmd::Camera { spline, first, last } => parts.push(format!("CAMERA spline{} keys{first}..{last}", *spline as u8)),
                nf_formats::nis::Cmd::CameraEnd => parts.push("cameraEnd".into()),
                nf_formats::nis::Cmd::SubScript { hash, stream, mode } => parts.push(format!("SUBSCRIPT {:08x} stream{stream} mode{mode} @t{:.0}", hash, clock)),
                nf_formats::nis::Cmd::Fade(f) => parts.push(format!("fade{f} @t{:.0}", clock)),
                nf_formats::nis::Cmd::Sprite { rect, flags, hash } => parts.push(format!("SPRITE {:08x} rect{:?} flags{:#x}", hash, rect, flags)),
                nf_formats::nis::Cmd::Light { parent, kind } => parts.push(format!("LIGHT parent{parent:?} kind{kind}")),
                nf_formats::nis::Cmd::Text { label, time } => parts.push(format!("TEXT {:08x} time{time}", label)),
                nf_formats::nis::Cmd::Skip { op } => parts.push(format!("skip{op:#04x}")),
                nf_formats::nis::Cmd::Unread(op) => { parts.push(format!("UNREAD {op:#04x}")); break; }
                nf_formats::nis::Cmd::Other(op) => parts.push(format!("other{op:#04x}")),
            }
        }
        let krange = if s.last_key >= s.first_key && (s.last_key as usize) < n.keys.len() {
            let k0 = &n.keys[s.first_key as usize]; let k1 = &n.keys[s.last_key as usize];
            format!(" keys{}..{} t{:.1}..{:.1} p0({:.2},{:.2},{:.2})", s.first_key, s.last_key, k0.time, k1.time, k0.pos[0], k0.pos[1], k0.pos[2])
        } else { " (no keys)".into() };
        println!("  s{si}:{krange} :: {}", parts.join(" "));
        if s.last_key >= s.first_key && (s.last_key as usize) < n.keys.len() {
            for k in s.first_key..=s.last_key.min(s.first_key + 40) {
                let key = &n.keys[k as usize];
                println!("      key{k} t{:.1} pos({:.2},{:.2},{:.2}) w28{:.3}", key.time, key.pos[0], key.pos[1], key.pos[2], key.w28);
            }
        }
    }
}

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for name in ["07000016.bin", "07800016.bin"] {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap_or_else(|| panic!("{name} on the disc"));
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        println!("\n######## {name}: {} entries", lb.entries.len());
        let mut scripts: Vec<_> = lb.entries.iter().filter(|e| e.kind == 7).collect();
        scripts.sort_by_key(|e| u32::from_str_radix(&e.name, 16).unwrap_or(0));
        for e in &scripts {
            let hash = u32::from_str_radix(&e.name, 16).unwrap();
            if let Ok(n) = Nis::parse(hash, lb.data(&data, e)) {
                let tag = if hash == 0x0600_090a { "ShuttleLaunch" } else if hash == 0x0600_0631 { "pack-cinema" } else { "script" };
                dump_script(&n, tag);
            } else {
                println!("=== {name} script {hash:08x}: PARSE FAILED");
            }
        }
    }
}
