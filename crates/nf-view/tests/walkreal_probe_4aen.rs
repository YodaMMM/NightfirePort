//! level-4a-complete probe: the static shape of the Tower-2 scripts (0x06000571, 0x060006cf,
//! 0x060006d3) and the base-composition witnesses (07000014's fusebox 0x06000A1B, 07000008's
//! safe 0x060009E2) — stream kinds/parents/obj_types and the event commands. Decides whether the
//! base composes only at a parent-chain end [M 0x1d9a08:114-135] for every render-verified
//! stream. Diagnostic only; no behaviour.

use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

#[test]
fn dump_nis_stream_summaries() {
    let iso = IsoImage::open(std::env::var("NF_ISO").unwrap().as_str()).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for lvname in ["0700004a.bin", "07000014.bin", "07000008.bin"] {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(lvname)) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(e) => { eprintln!("{lvname}: parse {e}"); continue } };
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            let h = e.hash();
            if !matches!(h, 0x0600_06cf | 0x0600_06d3 | 0x0600_0571 | 0x0600_0a1b | 0x0600_09e2) { continue; }
            let Ok(n) = Nis::parse(h, lb.data(&data, e)) else { continue };
            eprintln!("== NIS {h:08x} from {lvname}: {} streams, length {}, loop_start {}", n.streams.len(), n.length, n.loop_start);
            for (si, s) in n.streams.iter().enumerate() {
                let cmds = s.decode();
                let mut notes = String::new();
                for c in &cmds {
                    match c {
                        nf_formats::nis::Cmd::Event { kind, params } => notes.push_str(&format!(" ev{kind:02x}{params:?}")),
                        nf_formats::nis::Cmd::Entity { hash, parent, spline, obj_type, params } =>
                            notes.push_str(&format!(" ENT h{hash:08x} parent{parent:?} sp{spline} ot{obj_type} p{params:?}")),
                        nf_formats::nis::Cmd::Anim { script, skin, start } => notes.push_str(&format!(" ANIM s{script:08x} skin{skin:08x}")),
                        nf_formats::nis::Cmd::Camera { spline, first, last } => notes.push_str(&format!(" CAM sp{spline} k{first}..{last}")),
                        nf_formats::nis::Cmd::Wait(t) => notes.push_str(&format!(" W{t}")),
                        nf_formats::nis::Cmd::End(t) => notes.push_str(&format!(" end{t}")),
                        nf_formats::nis::Cmd::Marker(t) => notes.push_str(&format!(" M{t}")),
                        other => notes.push_str(&format!(" {other:?}")),
                    }
                }
                let keys = if s.last_key > s.first_key { format!("keys {}..{}", s.first_key, s.last_key) } else { "no keys".to_string() };
                eprintln!("stream {si}: {keys}{}", notes);
            }
        }
    }
}

#[test]
fn dump_060006cf_pool_tail() {
    let iso = IsoImage::open(std::env::var("NF_ISO").unwrap().as_str()).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0780004a.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = LevelBin::parse(&data).unwrap();
    for e in lb.entries.iter().filter(|e| e.kind == 7) {
        let h = e.hash();
        if h != 0x0600_06cf { continue; }
        let n = Nis::parse(h, lb.data(&data, e)).unwrap();
        eprintln!("pool len {}", n.keys.len());
        for k in 1404..n.keys.len().min(1412) {
            let key = &n.keys[k];
            eprintln!("pool[{k}]: t {:.1} pos ({:.3},{:.3},{:.3},{:.3}) scale {:.3}", key.time,
                key.pos[0], key.pos[1], key.pos[2], key.pos[3], key.scale);
        }
        // stream 0's last key (the camera's final pose) for reference
        let s0 = &n.streams[0];
        for k in s0.last_key-2..s0.last_key {
            let key = &n.keys[k as usize];
            eprintln!("cam pool[{k}]: t {:.1} pos ({:.3},{:.3},{:.3})", key.time, key.pos[0], key.pos[1], key.pos[2]);
        }
    }
}
