//! lock census part 4: every map (not just the main one) of the candidate levels, every writer
//! placement whose channel params name a kind-0 lock's out channel. Opt-in NF_ISO; prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_all_map_lock_channel_writers() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    for a in ["0700000a.bin", "0700000b.bin", "07000011.bin", "07000012.bin", "07000014.bin", "07000015.bin", "07000046.bin"] {
        let data = read_archive(&iso, a);
        let lv = Level::parse(&data).expect("parse");
        let mm = lv.main_map().expect("main map");
        let mut chans: Vec<u32> = Vec::new();
        for p in lv.maps[mm].placements.iter() {
            if p.flags & 0xff != 0x2e { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            if param(0) == 0 && param(5) != 0 { chans.push(param(5)); }
        }
        chans.sort(); chans.dedup();
        if chans.is_empty() { println!("== {a}: none =="); continue; }
        println!("== {a}: lock out channels {chans:02x?} ==");
        let mut hits = 0;
        for (mi, map) in lv.maps.iter().enumerate() {
            for (i, p) in map.placements.iter().enumerate() {
                let k = p.flags & 0xff;
                let param = |kk: u32| p.params.iter().find(|(j, _)| *j == kk).map_or(0, |&(_, v)| v);
                // (kind, [param slots that are channel words]) per the port's creators.
                let slots: &[(u32, u8)] = match k {
                    0xdb => &[(2, b'o'), (3, b'l')],          // door: ch_open / ch_lock
                    0x28 => &[(3, b'o'), (4, b'd'), (8, b'm')], // sensor: out / disable / monitor
                    0x29 => &[(0, b'c')],                      // switch: ch
                    0x25 => &[(3, b'c')],                      // SS: channel
                    0xf9 => &[(0, b'n'), (4, b'f')],           // sound trigger: on / off
                    0x31 => &[(0, b'o')],                      // fusebox: out
                    0xEA | 0xEB => &[(1, b'g'), (3, b'f'), (4, b's'), (6, b'k')], // trigger family params
                    _ => &[],
                };
                for &(slot, tag) in slots {
                    let v = param(slot);
                    if v != 0 && v < 256 && chans.contains(&v) {
                        hits += 1;
                        println!("  map{mi} #{i} kind {k:#04x} param{slot}({tag}) = {v:#04x}  params [{}]",
                            p.params.iter().map(|(x, y)| format!("{x}:{y:#x}")).collect::<Vec<_>>().join(" "));
                    }
                }
            }
        }
        if hits == 0 { println!("  (no writer placement on ANY map names a lock channel)"); }
    }
}
