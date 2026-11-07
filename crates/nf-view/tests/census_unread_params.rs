//! census_unread_params: throwaway census for task `unread-params-drones`.
//! For every level archive on the disc (nf_view::dev::LEVELS), count non-zero
//! values of every placement parameter (and the +0x24 quaternion / +0x34 scale
//! words) for the drone-family classes 0x0f (Drone_Create 0x136be0), 0xf1
//! (DroneSpawner_Create 0x1378d0), 0xf5/0xf8 (Drone_AIVolume_Create 0x1376c0),
//! plus the AI-path fields the port's nav build ignores (path params, extra
//! area, node raw words, link words). Assertion-free probe [D level data].
use nf_formats::aipath::AiNetwork;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

const CLASSES: [u32; 4] = [0x0f, 0xf1, 0xf5, 0xf8];

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored --nocapture"]
fn census_unread_params() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    // (class, param) -> (levels with nonzero, total nonzero records, max value)
    let mut pv: std::collections::BTreeMap<(u32, u32), (Vec<String>, u32, u32)> = Default::default();
    // (class, "quat"/"scale") -> same
    let mut xs: std::collections::BTreeMap<(u32, &'static str), (Vec<String>, u32)> = Default::default();
    let mut class_totals: std::collections::BTreeMap<u32, u32> = Default::default();
    // AI path fields: (slot) -> levels/records; node raw word offsets relative to the
    // retained raw (raw[0] = node+0x04, raw[1..] = node+0x1c..); link a/b flag words.
    let mut pparams: std::collections::BTreeMap<usize, (Vec<String>, u32)> = Default::default();
    let mut pextra = (Vec::<String>::new(), 0u32);
    let mut nraw: std::collections::BTreeMap<usize, (Vec<String>, u32)> = Default::default();
    let mut lflags = (Vec::<String>::new(), 0u32);
    let mut path_count = 0u32;
    for &lv in nf_view::dev::LEVELS {
        let name = format!("{lv:08x}.bin");
        let Ok(img) = IsoImage::open(&iso) else { continue };
        let Ok((entries, bin)) = nf_formats::disc::files_bin_entries(&img) else { continue };
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else {
            println!("MISSING ARCHIVE {name}"); continue;
        };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let level = Level::parse(&data).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let tag = format!("{lv:08x}");
        for map in &level.maps {
            for p in &map.placements {
                let c = p.flags & 0xff;
                if !CLASSES.contains(&c) { continue; }
                *class_totals.entry(c).or_default() += 1;
                let e = |m: &mut std::collections::BTreeMap<(u32, u32), (Vec<String>, u32, u32)>, k: (u32, u32), v: u32| {
                    let a = m.entry(k).or_default();
                    if v != 0 { a.0.push(tag.clone()); a.1 += 1; a.2 = a.2.max(v); }
                };
                for (i, v) in &p.params { e(&mut pv, (c, *i), *v); }
                if p.extra[0..4].iter().any(|&w| w != 0) { let a = xs.entry((c, "quat")).or_default(); a.0.push(tag.clone()); a.1 += 1; }
                if p.extra[4..7].iter().any(|&w| w != 0) { let a = xs.entry((c, "scale")).or_default(); a.0.push(tag.clone()); a.1 += 1; }
            }
            if let Some(net) = &map.ai_network {
                for path in &net.paths {
                    path_count += 1;
                    for (s, &v) in path.params.iter().enumerate() {
                        if v != 0 { let a = pparams.entry(s).or_default(); a.0.push(tag.clone()); a.1 += 1; }
                    }
                    if !path.extra.is_empty() { pextra.0.push(tag.clone()); pextra.1 += 1; }
                    for n in &path.nodes {
                        for (w, &v) in n.raw.iter().enumerate() {
                            if v != 0 { let a = nraw.entry(w).or_default(); a.0.push(tag.clone()); a.1 += 1; }
                        }
                    }
                    for l in &path.links {
                        // [M AIPath_Parse 0x1d2598:77-100] link words: +8 s16 target, +10 u16
                        // (node index | flags 0x20/0x40 per nav.rs's folding note).
                        if (l.b & 0x60) != 0 { let a = &mut lflags; a.0.push(tag.clone()); a.1 += 1; }
                    }
                }
            }
        }
    }
    println!("== placement param census (class, param): levels-with-nonzero, nonzero records, max ==");
    for ((c, i), (levels, n, max)) in &pv {
        println!("P class {c:#04x} param {i:<2} nonzero {n:<4} max {max:#010x} levels {}", levels.iter().fold(std::collections::BTreeSet::new(), |mut s: std::collections::BTreeSet<_>, l| { if s.insert(l.clone()) { } s }).len());
    }
    println!("== class totals ==");
    for (c, n) in &class_totals { println!("C class {c:#04x} placements {n}"); }
    println!("== quat/scale words ==");
    for ((c, w), (levels, n)) in &xs { println!("X class {c:#04x} {w} nonzero {n:<4} levels {}", levels.iter().collect::<std::collections::BTreeSet<_>>().len()); }
    println!("== ai path census (paths {path_count}) ==");
    println!("A path params[8] slot nonzero: {:?}", pparams);
    println!("A path extra area present: {} records, levels {}", pextra.1, pextra.0.iter().collect::<std::collections::BTreeSet<_>>().len());
    println!("A node raw words (raw[0]=node+0x04, raw[k]=node+0x18+4k):");
    for (w, (levels, n)) in &nraw { println!("N raw[{w}] nonzero {n:<5} levels {}", levels.iter().collect::<std::collections::BTreeSet<_>>().len()); }
    println!("A link +10 words carrying flags 0x20/0x40: {} records, levels {}", lflags.1, lflags.0.iter().collect::<std::collections::BTreeSet<_>>().len());
}
