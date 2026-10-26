//! level-0d-audit pins: the 0700000d placement census [D] and the mission record 12 [L]
//! (work/ee_start_05.bin, file offset = EE address). The census assertions pin the parse path;
//! the record assertions pin c0700000d::DEF against the boot snapshot. Snapshot-dependent parts
//! skip when work/ is absent (CI / fresh clones).
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn ee_start_05() -> Option<Vec<u8>> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../work/ee_start_05.bin")).ok()
}

/// The main-map placement census of 0700000d [D 0700000d.bin]: 568 placements over 34 kinds,
/// with the exact per-kind counts the audit table (docs/levels/0700000d-audit.md) is built from.
#[test]
fn census_0700000d_kind_histogram() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set; skipping"); return };
    let data = read_archive(&iso, "0700000d.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    let placements = &lv.maps[mm].placements;
    assert_eq!(placements.len(), 568, "the main-map placement count [D census 2026-10-26]");
    let mut hist: Vec<(u32, usize)> = Vec::new();
    for p in placements {
        let k = p.flags & 0xff;
        match hist.iter_mut().find(|(kk, _)| *kk == k) {
            Some((_, n)) => *n += 1,
            None => hist.push((k, 1)),
        }
    }
    hist.sort();
    let want: &[u32] = &[0x0f, 0x19, 0x22, 0x24, 0x2a, 0x2d, 0x31, 0x33, 0x34, 0x3a, 0x3b, 0x3e,
        0x3f, 0x47, 0x71, 0xd8, 0xd9, 0xdb, 0xdf, 0xe1, 0xe2, 0xe5, 0xe6, 0xe7, 0xea, 0xeb,
        0xec, 0xf0, 0xf1, 0xf9, 0xfa, 0xfb, 0xfd, 0xff];
    let got: Vec<u32> = hist.iter().map(|(k, _)| *k).collect();
    assert_eq!(got, want, "the kind set [D census]");
    for (k, n) in want.iter().zip([41usize, 260, 3, 1, 2, 1, 12, 1, 1, 3, 5, 3, 1, 15, 1, 3, 3,
        6, 1, 11, 1, 56, 68, 4, 8, 19, 1, 16, 6, 1, 9, 1, 2, 2]) {
        let c = hist.iter().find(|(kk, _)| kk == k).map(|(_, n)| *n).unwrap_or(0);
        assert_eq!(c, n, "kind {k:#04x} count [D census]");
    }
    // The wiring pins the audit's wait-web rows rest on [D]: the dock pair's gate ch equals
    // TouchOnce #513's out ch; the corridor trio's equals #554's.
    let pl = |i: usize| &placements[i];
    let param = |p: &nf_formats::placement::Placement, k: u32|
        p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
    assert_eq!(param(pl(513), 0), 0x2d, "#513's out ch [D]");
    assert_eq!(param(pl(292), 3), 0x2d, "#292 waits on #513's ch [D]");
    assert_eq!(param(pl(554), 0), 0x58, "#554's out ch [D]");
    assert_eq!(param(pl(550), 3), 0x58, "#550 waits on #554's ch [D]");
    // The C5 leaf pair: out ch == watch ch == 0x1f [D] — the pair-smash wiring.
    for i in [93usize, 96] {
        assert_eq!(param(pl(i), 0), 0x1f, "#{i} out ch [D]");
        assert_eq!(param(pl(i), 1), 0x1f, "#{i} watch ch [D]");
    }
    // Every drone's death channel is 0 [D census] — the fail hooks are unreachable.
    for (i, p) in placements.iter().enumerate() {
        if p.flags & 0xff == 0x0f {
            assert_eq!(param(p, 12), 0, "drone #{i} death ch is 0 [D census]");
        }
    }
}

/// MissionData record 12 read from the boot snapshot [L ee_start_05.bin @0x2a4530] — the record
/// lives at 0x2a4530 (0x2a44a0, the level doc's old address, is record 8's count word) — and the
/// three objectives @0x2a40c8, pinned against c0700000d::DEF field-for-field.
#[test]
fn mission_record_12_matches_the_snapshot() {
    let Some(snap) = ee_start_05() else { eprintln!("work/ee_start_05.bin not present; skipping"); return };
    let rd = |addr: usize| -> u32 {
        u32::from_le_bytes(snap[addr..addr + 4].try_into().unwrap())
    };
    let base = 0x2a4530;
    assert_eq!(rd(base), 0x0700000d, "record 12 level hash [L]");
    assert_eq!(rd(base + 4), 0x0700000c, "record 12 BaseMap [L]");
    assert_eq!(rd(base + 8), 1, "record 12 part [L]");
    let objectives = rd(base + 0x0c) as usize;
    assert_eq!(objectives, 0x2a40c8, "record 12 objective ptr [L]");
    assert_eq!(rd(base + 0x10), 3, "record 12 count [L]");
    // the old wrong address really is record 8's count word [L]
    assert_eq!(rd(0x2a44a0), 3, "0x2a44a0 holds record 8's count, not record 12");
    let def = &nf_game::mission::c0700000d::DEF;
    assert_eq!(def.level, rd(base), "DEF.level == the snapshot record [L]");
    assert_eq!(def.mission, rd(base + 4), "DEF.mission [L]");
    assert_eq!(def.part, rd(base + 8), "DEF.part [L]");
    assert_eq!(def.objectives.len(), 3, "DEF count [L]");
    for (j, o) in def.objectives.iter().enumerate() {
        let ob = objectives + j * 0x18;
        assert_eq!(o.text, rd(ob), "objective {j} text [L]");
        assert_eq!(o.fail, rd(ob + 4), "objective {j} fail label [L]");
        assert_eq!(o.hint, rd(ob + 8), "objective {j} hint label [L]");
        assert_eq!(o.channel, snap[ob + 0x0c], "objective {j} complete ch [L]");
        assert_eq!(o.init, snap[ob + 0x0d], "objective {j} init [L]");
        assert_eq!(o.enable, snap[ob + 0x0f], "objective {j} enable ch [L]");
        assert_eq!(o.flags, snap[ob + 0x10], "objective {j} flags [L]");
    }
}
