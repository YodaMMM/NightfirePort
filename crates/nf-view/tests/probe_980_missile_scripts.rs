//! 980-space-missile-timelines probe: re-derives the 0700001b SpaceMissile script decode from the
//! disc — the kind-0x39 placements' p0/p1/p2 script hashes (Create_SpaceMissile's param map
//! [M 0x1a9ef0]) and each script's clock-annotated Event commands (the kind-0x11 channel writes
//! [M Script_EventHandler 0x1d94c0], the kind-3 exit) + the End marker, per stream.
//! Scratch evidence for space_missile.rs's [D] constants — remove or #[ignore] once pinned.
//! Run: `NF_ISO=… cargo test -p nf-view --test probe_980_missile_scripts -- --nocapture`

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::{Cmd, Nis};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn param(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0)
}

#[test]
fn probe_missile_scripts() {
    let iso = iso();
    let data = read_archive(&iso, "0700001b.bin");
    let lb = LevelBin::parse(&data).expect("levelbin");
    let lib: std::collections::BTreeMap<u32, Nis> = lb.entries.iter()
        .filter(|e| e.kind == 7)
        .filter_map(|e| Nis::parse(e.hash(), lb.data(&data, e)).ok().map(|n| (e.hash(), n)))
        .collect();
    println!("script bank: {} NIS entries", lib.len());

    let level = Level::parse(&data).expect("parse");
    let m = level.main_map().expect("main map");
    let mm = &level.maps[m];
    let mut scripts_of = Vec::new();
    for (pi, p) in mm.placements.iter().enumerate() {
        if p.flags & 0xff != 0x39 { continue; }
        let p0 = param(p, 0); let p1 = param(p, 1); let p2 = param(p, 2);
        println!("MISSILE #{pi}: index p4={} p3={} p5(fail)={} p6(done)={} p7(gate)={} p0={p0:08x} p1={p1:08x} p2={p2:08x}",
            param(p, 4), param(p, 3) as i32, param(p, 5), param(p, 6), param(p, 7));
        scripts_of.push((format!("#pi{pi} p0"), p0));
        scripts_of.push((format!("#pi{pi} p1"), p1));
        scripts_of.push((format!("#pi{pi} p2"), p2));
    }

    // Who else touches ch 99 (0x63)? Any placement carrying 99 as a small param (a gate/out/in
    // channel), any kind — the [D] sweep for "the mission reader is Mission_Update, not a trigger".
    for (pi, p) in mm.placements.iter().enumerate() {
        if p.params.iter().any(|&(_, v)| v == 99) {
            println!("CH99-REF #{pi} kind {:02x} params {:x?}", p.flags & 0xff, p.params);
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    for (tag, hash) in &scripts_of {
        if !seen.insert(*hash) { continue; }
        let Some(n) = lib.get(hash) else { println!("{tag} {hash:08x}: NOT IN BANK"); continue; };
        println!("--- {hash:08x} ({tag}): header length {} camera {} streams {}", n.length, n.camera, n.streams.len());
        for (si, s) in n.streams.iter().enumerate() {
            // `Script_Run`'s op 5 sets the stream's wait TARGET — the arg is an absolute stream
            // time (the port: `s.wait = t`, hold until `clock >= t` [M 0x1d8548; nf-game nis.rs:451]).
            let mut t: u32 = 0;
            let mut out = Vec::new();
            for cmd in s.decode() {
                match cmd {
                    Cmd::Wait(a) => t = a as u32,
                    Cmd::Event { kind, params } => out.push(format!("t{t}: ev kind {kind:#04x} params {params:x?}")),
                    Cmd::End(a) => out.push(format!("t{t}: End({a})")),
                    Cmd::Marker(a) => out.push(format!("t{t}: Marker({a})")),
                    Cmd::SubScript { hash, stream, mode } => out.push(format!("t{t}: sub {hash:08x} s{stream} m{mode}")),
                    _ => {}
                }
            }
            println!("  stream {si}: {}", out.join(" | "));
        }
    }
}
