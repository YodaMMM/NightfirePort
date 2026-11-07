//! hurt46_probe: what 07000046's seven flying hurt volumes are [D disc].
//! Dumps every 0xE4 placement: params, raw +0x40/42/44 (count/rate/path), the target object's
//! name/cel, and the referenced 0x19 path blocks' keys. Assertion-free, `--ignored --nocapture`.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn hurt46_probe() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000046.bin");
    let level = Level::parse(&data).expect("parse 07000046");
    let paths = level.path_blocks();
    println!("path_blocks: {}", paths.len());
    for (i, b) in paths.iter().enumerate() {
        let n = b.len() / 28;
        let keys: Vec<String> = (0..n).map(|k| {
            let p = &b[k * 28..];
            let w = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
            format!("({:.2},{:.2},{:.2})", w(0), w(4), w(8))
        }).collect();
        println!("  path[{i}] keys={n}: {}", keys.join(" "));
    }
    let m = level.main_map().expect("main map");
    for (i, p) in level.maps[m].placements.iter().enumerate() {
        if p.flags & 0xff != 0xe4 { continue; }
        let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
        // raw words: the placement record's +0x40/+0x42/+0x44 — the port's Placement carries
        // the params map only; the raw words live in p.extra (see nf-formats placement doc).
        let target = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|(mm, oi)| (format!("by-hash {:08x}", p.hashcode), *mm, *oi)) }
            else if (p.index as usize) < level.maps[m].objects.len() { Some((format!("idx {}", p.index), m, p.index as usize)) } else { None };
        let objname = target.and_then(|(_, om, oi)| level.maps[om].objects.get(oi).map(|o| o.name.clone()));
        println!(
            "hurt #{i:x}: obj={:?} p0={} p1={} p2(gate)={} p3(sound)={} p4(who)={:#x} p5={} count={:?} rate={:?} path={:?} extra={:?}",
            objname, param(0), param(1), param(2), param(3), param(4), param(5),
            p.extra[7] & 0xffff, p.extra[7] >> 16, p.extra[8] & 0xffff, p.extra
        );
    }
    println!("LEVELS idx: {:?}", nf_view::dev::LEVELS.iter().position(|&l| l == 0x0700_0046));
    println!("label: {}", nf_view::dev::LEVEL_LABELS[nf_view::dev::LEVELS.iter().position(|&l| l == 0x0700_0046).unwrap_or(0)]);
}
