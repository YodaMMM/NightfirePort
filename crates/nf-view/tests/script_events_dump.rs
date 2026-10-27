//! script-events-3-4 probe: dump every level archive's kind-7 scripts' Event 3/4 uses with timing [D].
//! Opt-in (NF_ISO); prints only. Also prints the full timed decode of 06000959 and 060006d7.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::{Cmd, Nis};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(img: &IsoImage, bin_lba: u64, entries: &[nf_formats::filelist::FileEntry], archive: &str) -> Option<Vec<u8>> {
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive))?;
    Some(img.read_bytes(bin_lba * 2048 + fe.offset as u64, fe.size as u64).expect("read"))
}

/// Timed decode: walk the stream's commands, tracking `05`-waits; yield (fire_tick, cmd).
fn timed(stream: &nf_formats::nis::Stream) -> Vec<(u32, Cmd)> {
    let mut out = Vec::new();
    let mut wait = 0u32;
    for c in stream.decode() {
        match &c {
            Cmd::Wait(t) => wait = *t as u32,
            _ => {}
        }
        out.push((wait, c));
    }
    out
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_script_event_3_4_users() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let bin_lba = bin.lba as u64;

    // The two focus scripts first.
    for (archive, want) in [("07900013.bin", vec![0x060006d7]), ("07000013.bin", vec![0x06000959, 0x0600006f])] {
        let Some(data) = read_archive(&img, bin_lba, &entries, archive) else { println!("== {archive}: absent =="); continue };
        let lb = LevelBin::parse(&data).expect("levelbin");
        for se in lb.entries.iter().filter(|e| e.kind == 7) {
            if !want.contains(&se.hash()) { continue; }
            let nis = Nis::parse(se.hash(), lb.data(&data, se)).expect("nis");
            println!("== script {:08x} ({}): len {} loop {} streams {} ==", nis.hashcode, archive, nis.length, nis.loop_start, nis.streams.len());
            for (si, s) in nis.streams.iter().enumerate() {
                println!(" stream {si}: kind? keys {}..{}", s.first_key, s.last_key);
                for (t, c) in timed(s) {
                    match &c {
                        Cmd::Entity { hash, parent, spline, obj_type, params } =>
                            println!("   t{t:>4}: Entity glist {hash:08x} parent {parent:?} spline {spline} type {obj_type:02x} params {params:?}"),
                        Cmd::Event { kind, params } =>
                            println!("   t{t:>4}: Event kind {kind:#04x} params {params:?}"),
                        Cmd::Wait(t2) => println!("   t{t:>4}: Wait -> {t2}"),
                        other => println!("   t{t:>4}: {other:?}"),
                    }
                }
            }
        }
    }

    // The sweep: every level archive on the disc, every kind-7 script (main bin + the 078/079/07A packs).
    println!("\n==== SWEEP: Event 3 / Event 4 users across every level archive ====");
    let mut total = 0usize;
    for e in entries.iter().filter(|e| {
        let n = e.name.to_ascii_uppercase();
        n.starts_with("07") && n.ends_with(".BIN") && n.len() == 12
    }) {
        let Ok(data) = std::panic::catch_unwind(|| read_archive(&img, bin_lba, &entries, &e.name).expect("read")) else { continue };
        let Ok(lb) = std::panic::catch_unwind(|| LevelBin::parse(&data).expect("levelbin")) else { continue };
        let mut level_hits: Vec<String> = Vec::new();
        for se in lb.entries.iter().filter(|e| e.kind == 7) {
            let Ok(nis) = Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
            for (si, s) in nis.streams.iter().enumerate() {
                for (t, c) in timed(s) {
                    if let Cmd::Event { kind: kind @ (3 | 4), params } = &c {
                        level_hits.push(format!("script {:08x} stream {si} t{t} kind {kind} params {params:?}", nis.hashcode));
                    }
                }
            }
        }
        if !level_hits.is_empty() {
            println!("{}:", e.name);
            for h in &level_hits { println!("  {h}"); total += 1; }
        }
    }
    println!("==== total Event 3/4 command sites: {total} ====");
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_06000959_keys_and_sp427() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let data = read_archive(&img, bin.lba as u64, &entries, "07000013.bin").expect("07000013");
    let lb = LevelBin::parse(&data).expect("levelbin");
    for se in lb.entries.iter().filter(|e| e.kind == 7 && se_hash_is(e, 0x06000959)) {
        let nis = Nis::parse(se.hash(), lb.data(&data, se)).expect("nis");
        println!("== 06000959 keys: {} ==", nis.keys.len());
        for (i, k) in nis.keys.iter().enumerate() {
            println!(" key {i}: pos ({:.4},{:.4},{:.4},{:.4}) quat ({:.5},{:.5},{:.5},{:.5}) scale {:.4} time {:.2}",
                k.pos[0], k.pos[1], k.pos[2], k.pos[3], k.quat[0], k.quat[1], k.quat[2], k.quat[3], k.scale, k.time);
        }
    }
    let lv = Level::parse(&data).expect("level");
    let mm = lv.main_map().unwrap_or(0);
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if p.flags & 0xff == 0xd9 {
                println!("SP m{mi} #{pi} kind d9 params {:?} @({:.4},{:.4},{:.4}) flags {:#x} hash {:08x}",
                    p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect::<Vec<_>>(), p.translation[0], p.translation[1], p.translation[2], p.flags, p.hashcode);
            }
        }
    }
}
fn se_hash_is(e: &nf_formats::levelbin::BinEntry, h: u32) -> bool { e.hash() == h }
