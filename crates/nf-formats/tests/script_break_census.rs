//! The disc census of `Script_CreateEntity` 0x1d8968's obj_type 0x20 creates (`Break_Create`
//! 0x12c230 — the script-created breakables): every kind-7 NIS script in every level bin AND
//! in every loadable pack (the 078/079/07A bins the level's kind-0xC records stream [M
//! LoaderProcess 0x1d4880 case 0xC; LoadableLoad 0x1d69e8 — the port loads them in
//! nf-view/src/lib.rs "loadable cut-scene packs"]), every stream decoded, every Entity command
//! with obj_type 0x20 listed with its level, script hash and stream. The pin is the [D] census:
//! the port's script-pane drain (drone.rs's per-tick 0x20 sync) must see every row this lists.
//! Opt-in like the goldens: set `NF_ISO`; without it the test prints "skip".

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, levelbin, nis};
use std::collections::BTreeMap;

#[test]
fn the_disc_census_of_script_created_breakables() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let (files, bin) = disc::files_bin_entries(&iso).unwrap();

    // EVERY archive on the disc that parses as a level bin — the SP levels (0700xxxx.bin),
    // the MP maps and any other .bin: the census must not miss an archive class. The
    // loadable packs (078/079/07A) are reached through the kind-0xC records below.
    let mut census: BTreeMap<String, Vec<(u32, u8, Option<u8>, u32, Vec<u32>)>> = BTreeMap::new();
    let mut scanned: Vec<String> = Vec::new();
    for f in &files {
        if !(f.name.len() == 12 && f.name.to_ascii_lowercase().ends_with(".bin")) { continue; }
        let data = match iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64) {
            Ok(d) => d,
            Err(e) => { eprintln!("[census] {}: read failed: {e}", f.name); continue; }
        };
        let lb = match levelbin::LevelBin::parse(&data) { Ok(l) => l, Err(e) => { eprintln!("[census] {}: parse failed: {e}", f.name); continue; } };
        scanned.push(f.name.clone());
        let mut hit = |name: &str, lb: &levelbin::LevelBin, data: &[u8]| {
            for e in &lb.entries {
                if e.kind != 7 { continue; }
                let Ok(n) = nis::Nis::parse(e.hash(), lb.data(data, e)) else { continue };
                for (si, s) in n.streams.iter().enumerate() {
                    for cmd in s.decode() {
                        if let nis::Cmd::Entity { hash, parent, obj_type: 0x20, params, .. } = cmd {
                            census.entry(name.to_string()).or_default()
                                .push((e.hash(), si as u8, parent, hash, params));
                        }
                    }
                }
            }
        };
        hit(&f.name, &lb, &data);
        // The loadable packs: the level bin's kind-0xC records name them (flag+7 nibble at bits
        // 20-23 of the CURRENT level's id [M LoadableLoad 0x1d69e8]).
        let level_id = u32::from_str_radix(&f.name[..8], 16).unwrap_or(0);
        for e in &lb.entries {
            if e.kind != 0xC { continue; }
            let r = lb.data(&data, e);
            if r.len() < 5 { continue; }
            let flag = r[4];
            if flag == 0 || flag > 3 { continue; }
            let pack_name = format!("{:08x}.bin", (level_id & 0xff0f_ffff) | ((flag as u32 + 7) << 20));
            let Some(pf) = files.iter().find(|f| f.name.eq_ignore_ascii_case(&pack_name)) else { continue };
            let Ok(pdata) = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + pf.offset as u64, pf.size as u64) else { continue };
            let Ok(plb) = levelbin::LevelBin::parse(&pdata) else { continue };
            hit(&pack_name, &plb, &pdata);
        }
    }
    eprintln!("[census] scanned {} level bins", scanned.len());
    let mut total = 0usize;
    for (name, rows) in &census {
        for (script, stream, parent, hash, params) in rows {
            eprintln!("[census] {name} script {script:08X} stream {stream} parent {:?} glist {hash:08X} params {params:?}", parent);
            total += 1;
        }
    }
    eprintln!("[census] TOTAL script-created 0x20 entities: {total}");

    // The [D] census pin: the whole disc creates script breakables in exactly TWO levels.
    assert_eq!(total, 34, "the disc's script-created 0x20 entities [D census, 99 archives]");
    let mut keys: Vec<&String> = census.keys().collect();
    keys.sort();
    assert_eq!(keys, ["07000008.bin", "07000012.bin"], "the two levels [D census]");

    // 07000008: the gondola's 16 cabin windows — ride script 0x0600004C, streams 7-18/23-26,
    // every one parented to stream 2 (the carriage), params [2, 1, 0] (kind 2 glass, health
    // 1.0, filter 0) [D decode; pinned per-stream in gondola_windows_script.rs].
    let g = &census["07000008.bin"];
    assert_eq!(g.len(), 16, "the gondola's 16 windows [D]");
    assert!(g.iter().all(|&(s, st, p, _, ref pa)| s == 0x0600_004C && p == Some(2) && pa == &[2, 1, 0]
        && (7..=18).contains(&st) || (23..=26).contains(&st)), "the gondola rows' shape [D]");
    assert_eq!(g.iter().map(|r| r.1).collect::<std::collections::BTreeSet<_>>().len(), 16, "16 distinct streams [D]");

    // 07000012: the second and only other instance — script 0x06000530, streams 10-27 (18
    // panes), params [2,1,0] except streams 15 and 26 which carry [1,1,0] (kind 1) [D census].
    let i = &census["07000012.bin"];
    assert_eq!(i.len(), 18, "07000012's 18 panes [D]");
    assert!(i.iter().all(|&(s, _, _, _, _)| s == 0x0600_0530), "one script [D]");
    assert_eq!(i.iter().map(|r| r.1).collect::<std::collections::BTreeSet<_>>(),
        (10..=27).collect::<std::collections::BTreeSet<_>>(), "streams 10-27 [D]");
    assert_eq!(i.iter().filter(|r| r.4 == vec![1, 1, 0]).map(|r| r.1).collect::<Vec<_>>(), vec![15, 26],
        "the two kind-1 panes [D census]");
    assert!(i.iter().filter(|r| r.4 == vec![2, 1, 0]).count() == 16, "the 16 kind-2 panes [D]");
}
