//! level-4a-lift-3 probe: the raw disc bytes behind the fresh-boot live chain. Dumps
//! 0x06000571 (the lift script) and 0x060006cf (the landing scene, pack 0780004a) — every
//! stream's full command decode and EVERY key (time + pos) — plus every placement whose y is
//! in -160..-40 (the kill volume hunt for the live death at y -95.80). Assertion-free
//! diagnostic, run with --ignored --nocapture.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::{Cmd, Nis};

fn read_bytes(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn dump_nis(name: &str, nis: &Nis) {
    println!("\n=== {name} {:08x}: length {} camera {} loop_start {} streams {} keys {} ===",
        nis.hashcode, nis.length, nis.camera, nis.loop_start, nis.streams.len(), nis.keys.len());
    for (si, st) in nis.streams.iter().enumerate() {
        println!("-- stream {si}: keys {}..{} ({} bytes)", st.first_key, st.last_key, st.commands.len());
        for c in st.decode() {
            match &c {
                Cmd::Entity { hash, parent, obj_type, params, .. } => {
                    let ps = params.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(",");
                    println!("     Entity hash {hash:08x} parent {parent:?} obj_type {obj_type:#04x} params [{ps}]");
                }
                Cmd::Event { kind, params } => {
                    let ps = params.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(",");
                    println!("     Event kind {kind:#04x} [{ps}]");
                }
                Cmd::Marker(t) => println!("     Marker({t})"),
                Cmd::Wait(t) => println!("     Wait({t})"),
                Cmd::End(t) => println!("     End({t})"),
                Cmd::Anim { script, skin, start } => println!("     Anim script {script:08x} skin {skin:08x} start {start}"),
                Cmd::Camera { spline, first, last } => println!("     Camera spline {spline} keys {first}..{last}"),
                Cmd::Sound { id, .. } => println!("     Sound {id:08x}"),
                Cmd::SubScript { hash, stream, mode } => println!("     SubScript {hash:08x} stream {stream} mode {mode}"),
                Cmd::Fade(f) => println!("     Fade({f})"),
                Cmd::Sprite { hash, .. } => println!("     Sprite {hash:08x}"),
                Cmd::Light { parent, kind } => println!("     Light parent {parent:?} kind {kind}"),
                Cmd::Text { label, time } => println!("     Text {label:08x} time {time}"),
                Cmd::Unread(op) => println!("     UNREAD {op:#04x}"),
                Cmd::Skip { op } => println!("     Skip {op:#04x}"),
                other => println!("     {other:?}"),
            }
        }
    }
    // Every key this script owns (the stream first_key/last_key index the global key array).
    let lo = nis.streams.iter().map(|s| s.first_key as usize).min().unwrap_or(0);
    let hi = nis.streams.iter().map(|s| s.last_key as usize).max().unwrap_or(0);
    for (ki, k) in nis.keys.iter().enumerate().skip(lo).take((hi - lo + 1).max(0)) {
        println!("   key[{ki}] t{:.0} ({:.5},{:.5},{:.5},{:.5}) q({:.4},{:.4},{:.4},{:.4}) s{:.3} w24 {:.4} w28 {:.4}",
            k.time, k.pos[0], k.pos[1], k.pos[2], k.pos[3],
            k.quat[0], k.quat[1], k.quat[2], k.quat[3], k.scale, k.f24(), k.w28);
    }
}

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn dump_lift_and_landing_scripts_raw() {
    let iso = iso();
    // The level's own scripts + the loadable packs (the 0xC records).
    let data = read_bytes(&iso, "0700004a.bin");
    let lb = LevelBin::parse(&data).expect("parse 0700004a");
    let mut found: Vec<(String, Nis)> = Vec::new();
    for e in lb.entries.iter() {
        if e.kind == 7 {
            let h = e.hash();
            if h == 0x06000571 { if let Ok(n) = Nis::parse(h, lb.data(&data, e)) { found.push(("level".into(), n)); } }
        }
        if e.kind == 0xC {
            let r = lb.data(&data, e);
            if r.len() >= 5 {
                let (hash, flag) = (u32::from_le_bytes([r[0], r[1], r[2], r[3]]), r[4]);
                println!("0xC record: script {hash:08x} flag {flag}");
            }
        }
    }
    let pack = read_bytes(&iso, "0780004a.bin");
    let plb = LevelBin::parse(&pack).expect("parse 0780004a");
    for e in plb.entries.iter() {
        if e.kind == 7 && e.hash() == 0x060006cf {
            if let Ok(n) = Nis::parse(e.hash(), plb.data(&pack, e)) { found.push(("pack 0780004a".into(), n)); }
        }
    }
    // The lobby scene too (the pack 0790004a) — its Player_Disable + the level exit.
    let pack2 = read_bytes(&iso, "0790004a.bin");
    let plb2 = LevelBin::parse(&pack2).expect("parse 0790004a");
    for e in plb2.entries.iter() {
        if e.kind == 7 && e.hash() == 0x060006d3 {
            if let Ok(n) = Nis::parse(e.hash(), plb2.data(&pack2, e)) { found.push(("pack 0790004a".into(), n)); }
        }
    }
    for (src, n) in &found { dump_nis(src, n); }

    // The kill-volume hunt: every placement with y in the shaft's lower half.
    let lv = nf_formats::level::Level::parse(&data).expect("level parse");
    println!("\n=== all maps: SPs (0xd9) + triggers (3/4/5/12/13) ===");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            let k = p.flags & 0xff;
            let in_shaft = p.translation[0].abs() < 12.0 && p.translation[2].abs() < 12.0 && p.translation[1] > -170.0;
            if k == 0xd9 || (matches!(k, 0x03 | 0x04 | 0x05 | 0x12 | 0x13 | 0xe8 | 0xea | 0xeb | 0xec) && in_shaft) {
                let ps = p.params.iter().map(|(j, v)| format!("p{j}={v:#x}")).collect::<Vec<_>>().join(",");
                println!("map{mi} #{i} kind {k:#04x} ({:.3},{:.3},{:.3}) params [{ps}]",
                    p.translation[0], p.translation[1], p.translation[2]);
            }
        }
    }
    println!("\n=== every kind-7 script in the level + the packs: per-stream events ===");
    let mut all: Vec<(String, u32, nf_formats::nis::Nis)> = Vec::new();
    for e in lb.entries.iter() {
        if e.kind == 7 { if let Ok(n) = Nis::parse(e.hash(), lb.data(&data, e)) { all.push(("level".into(), e.hash(), n)); } }
    }
    for e in plb.entries.iter() {
        if e.kind == 7 { if let Ok(n) = Nis::parse(e.hash(), plb.data(&pack, e)) { all.push(("pack8".into(), e.hash(), n)); } }
    }
    for e in plb2.entries.iter() {
        if e.kind == 7 { if let Ok(n) = Nis::parse(e.hash(), plb2.data(&pack2, e)) { all.push(("pack9".into(), e.hash(), n)); } }
    }
    all.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    for (src, h, n) in &all {
        for (si, s) in n.streams.iter().enumerate() {
            let cmds = s.decode();
            let evs: Vec<_> = cmds.iter().filter_map(|c| match c {
                Cmd::Event { kind, params } => Some(format!("ev{kind:#04x}[{}]", params.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(","))),
                Cmd::SubScript { hash, stream, mode } => Some(format!("SUB[{hash:08x} s{stream} m{mode}]")),
                _ => None,
            }).collect();
            if !evs.is_empty() { println!("{src} {h:08x} s{si} (len {}): {}", n.length, evs.join(" ")); }
        }
    }
}
