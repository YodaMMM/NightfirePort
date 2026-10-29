//! level-14-capture-placement probe: dump 07000014's capture scene — the pack scripts
//! 07900014.bin (0x0600062F) and 07A00014.bin (0x06000848): every stream (kind via its ops,
//! first/last key) and every event with params, plus the ScriptPlayer placements #382/#407's
//! base matrices from 07000014.bin [D]. Prints only.

use nf_formats::iso9660::IsoImage;
use nf_formats::nis::Cmd;

fn read_file(img: &IsoImage, name: &str) -> Vec<u8> {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).expect("FILES.BIN");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("{name} not in FILES.BIN"));
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read")
}

fn dump_script(nis: &nf_formats::nis::Nis, tag: &str, base: Option<[f32; 3]>) {
    println!("== {tag}: script {:08x} streams {} ==", nis.hashcode, nis.streams.len());
    // The keys of the entity streams (the PlayerEnable markers): first_key indexes nis.keys [D].
    for (si, s) in nis.streams.iter().enumerate() {
        let cmds = s.decode();
        let has_e = cmds.iter().any(|c| matches!(c, Cmd::Entity { .. }));
        let has_c = cmds.iter().any(|c| matches!(c, Cmd::Camera { .. }));
        if !has_e && !has_c {
            continue;
        }
        for (ki, k) in nis.keys.iter().enumerate().filter(|(ki, _)| *ki as u16 >= s.first_key && *ki as u16 <= s.last_key) {
            let _ = ki;
            let local = [k.pos[0], k.pos[1], k.pos[2]];
            let world = base.map(|b| {
                // rotation is identity for both SPs [D dump below]; the compose is the translation sum.
                [b[0] + local[0], b[1] + local[1], b[2] + local[2]]
            });
            println!(
                " stream {si} key#{} t{:.0}: local ({:.4},{:.4},{:.4}) quat ({:.4},{:.4},{:.4},{:.4}){}",
                ki, k.time, local[0], local[1], local[2], k.quat[0], k.quat[1], k.quat[2], k.quat[3],
                world.map(|w| format!("  SP-base+local = ({:.4},{:.4},{:.4})", w[0], w[1], w[2])).unwrap_or_default()
            );
        }
    }
    for (si, s) in nis.streams.iter().enumerate() {
        let mut kinds = String::new();
        let mut events: Vec<String> = Vec::new();
        for c in s.decode() {
            match c {
                Cmd::Entity { hash, obj_type, params, .. } => {
                    kinds.push_str(&format!(" E(hash={hash:08x},type={obj_type:#x},p={:x?})", params));
                }
                Cmd::Anim { script, skin, start } => {
                    kinds.push_str(&format!(" A(skin={skin:08x},script={script:08x},start={start})"));
                }
                Cmd::Camera { spline, first, last } => {
                    kinds.push_str(&format!(" C(spline={spline},{first}..{last})"));
                }
                Cmd::SubScript { hash, stream, mode } => {
                    kinds.push_str(&format!(" SUB(hash={hash:08x},stream={stream},mode={mode})"));
                }
                Cmd::Event { kind, params } => {
                    kinds.push_str(&format!(" ev{kind:#04x}"));
                    events.push(format!("  stream {si}: event {kind:#04x} params {:x?}", params));
                }
                Cmd::Wait(t) => kinds.push_str(&format!(" W{t}")),
                Cmd::End(t) => kinds.push_str(&format!(" END{t}")),
                Cmd::Marker(t) => kinds.push_str(&format!(" M{t}")),
                Cmd::Fade(f) => kinds.push_str(&format!(" FADE{f}")),
                Cmd::Sound { id, .. } => kinds.push_str(&format!(" SND{id:08x}")),
                Cmd::Text { label, time } => kinds.push_str(&format!(" TXT{label:08x}/{time}")),
                Cmd::Sprite { hash, .. } => kinds.push_str(&format!(" SPR{hash:08x}")),
                Cmd::Light { .. } => kinds.push_str(" LIGHT"),
                Cmd::Other(op) => kinds.push_str(&format!(" o{op:#02x}")),
                Cmd::Skip { op } => kinds.push_str(&format!(" s{op:#02x}")),
                Cmd::Unread(op) => kinds.push_str(&format!(" UNREAD{op:#02x}")),
                _ => kinds.push('?'),
            }
        }
        println!(" stream {si}: keys {}..{} |{}", s.first_key, s.last_key, kinds);
        for e in events {
            println!("{e}");
        }
    }
}

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    for (pack, hash, tag) in [
        ("07900014.bin", 0x0600_062fu32, "kiko scene (SP #382)"),
        ("07A00014.bin", 0x0600_0848u32, "capture/escape scene (SP #407)"),
        ("07800014.bin", 0x0600_0833u32, "opening scene (SP #276)"),
    ] {
        let data = read_file(&img, pack);
        let lb = nf_formats::levelbin::LevelBin::parse(&data).unwrap_or_else(|e| panic!("{pack}: {e:?}"));
        for se in lb.entries.iter().filter(|e| e.kind == 7) {
            if se.hash() != hash {
                continue;
            }
            match nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) {
                Ok(nis) => {
                    let base = match hash {
                        0x0600_062f => Some([-51.5703, 0.0, 31.2664]),
                        0x0600_0848 => Some([-61.4302, 0.0, 37.6964]),
                        _ => Some([52.1566, 2.6365, 46.6214]),
                    };
                    dump_script(&nis, &format!("{pack} {tag}"), base)
                }
                Err(e) => println!("== {pack} {tag}: script {:08x} PARSE ERR {:?} ==", se.hash(), e),
            }
        }
    }
    // The ScriptPlayer placements' base matrices [D].
    let data = read_file(&img, "07000014.bin");
    let level = nf_formats::level::Level::parse(&data).expect("level");
    for (mi, map) in level.maps.iter().enumerate() {
        for (pi, p) in map.placements.iter().enumerate() {
            if p.flags & 0xff == 0xd9 {
                let m = nf_formats::placement::object_matrix(p.rotation, p.translation);
                println!(
                    "SP map {mi} placement #{pi} kind 0xD9 params {:x?} base t=({:.4},{:.4},{:.4})",
                    p.params.iter().map(|(k, v)| format!("{k:#x}={v:08x}")).collect::<Vec<_>>().join(","),
                    m[3][0], m[3][1], m[3][2]
                );
            }
        }
    }
}
