//! The gondola's windows, from the disc [D]: 07000008's ride script 0x0600004C (a kind-7 entry
//! of the LEVEL BIN, not a pack) carries the carriage's parts as op-7 Entity streams — 16 of
//! them obj_type 0x20, i.e. `Script_CreateEntity` 0x1d8968:120-123's `Break_Create` arm — each
//! parented to stream 2 (the carriage's own parent chain; stream 1 is the `OBJECT_Carriage`
//! 0x0200038a glist entity, obj_type 0x31) and carrying params [2, 1, 0]: kind 2 (glass),
//! health (f32)(i32)1 = 1.0 [M 0x12c230 disasm 0x12c2a8-0x12c2c4 — the cvt.s.w], filter param 0
//! → 0 [M 0x12c230:29-35]. These are the cabin windows the player shoots out on the ride; the
//! port's script-pane breakables (crates/nf-game/src/breakable.rs `script_pane`) decode exactly
//! these params. Opt-in like the goldens: set `NF_ISO`; without it the test prints "skip".

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, levelbin, nis};

#[test]
fn the_gondola_ride_script_carries_sixteen_parented_glass_entities() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let (files, bin) = disc::files_bin_entries(&iso).unwrap();
    let f = files.iter().find(|f| f.name.eq_ignore_ascii_case("07000008.bin"))
        .expect("07000008.bin in FILES.BIN [D]");
    let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64).unwrap();
    let lb = levelbin::LevelBin::parse(&data).unwrap();
    let e = lb.entries.iter().find(|e| e.hash() == 0x0600_004c).expect("entry 0x0600004C [D]");
    assert_eq!(e.kind, 7, "the ride script is a kind-7 NIS entry [D]");
    let n = nis::Nis::parse(e.hash(), lb.data(&data, e)).unwrap();

    // The script's shape [D]: 53 streams, 1420 ticks (docs/levels/07000008.md §2 step 4).
    assert_eq!(n.streams.len(), 53, "the ride script's stream count [D]");
    assert_eq!(n.length, 1420, "the ride script's length [D stream 46's t1410 write]");

    // Stream 1: the carriage itself — Entity 0x0200038a, obj_type 0x31, no parent [D].
    let car = n.streams[1].decode().into_iter().find_map(|c| match c {
        nis::Cmd::Entity { hash, parent, obj_type, .. } => Some((hash, parent, obj_type)),
        _ => None,
    }).expect("stream 1 carries the carriage Entity [D]");
    assert_eq!(car, (0x0200_038a, None, 0x31), "stream 1 = OBJECT_Carriage 0x0200038a, obj_type 0x31, no parent [D]");

    // The windows: every obj_type-0x20 Entity stream — 16, all parented to stream 2, all
    // params [2, 1, 0] (kind 2 glass, health 1.0, filter 0) [D].
    let mut panes: Vec<(u8, u32, u8, Vec<u32>)> = Vec::new(); // (stream, hash, parent, params)
    for (si, s) in n.streams.iter().enumerate() {
        for cmd in s.decode() {
            if let nis::Cmd::Entity { hash, parent, obj_type: 0x20, params, .. } = cmd {
                panes.push((si as u8, hash, parent.unwrap_or(0xff), params));
            }
        }
    }
    eprintln!("[gondola] window streams: {panes:?}");
    assert_eq!(panes.len(), 16, "the cabin's 16 windows [D decode]");
    for &(si, hash, parent, ref params) in &panes {
        assert_eq!(parent, 2, "window on stream {si}: parented to stream 2 (the ride's carrier) [D]");
        assert_eq!(*params, vec![2, 1, 0],
            "window on stream {si}: params kind 2 / health 1 / filter 0 [D decode; M 0x1d8d78:80-87]");
        assert_ne!(hash, 0, "window on stream {si}: a glist hash [D]");
    }
    // The west-face/east-face split is the carriage's own parenting; the hashes are the panes'
    // glists (`hashtable_hashcode_to_celglist` [M 0x1d8968:121]).
    let mut hashes: Vec<u32> = panes.iter().map(|p| p.1).collect();
    hashes.sort_unstable();
    hashes.dedup();
    eprintln!("[gondola] distinct window glists: {hashes:?}");
    assert_eq!(hashes.len(), 16, "16 distinct window glists across the 16 streams [D decode]");
}

#[test]
fn the_ride_scripts_keep_objects_event_and_window_stream_ends() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let (files, bin) = disc::files_bin_entries(&iso).unwrap();
    let f = files.iter().find(|f| f.name.eq_ignore_ascii_case("07000008.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64).unwrap();
    let lb = levelbin::LevelBin::parse(&data).unwrap();
    let e = lb.entries.iter().find(|e| e.hash() == 0x0600_004c).unwrap();
    let n = nis::Nis::parse(e.hash(), lb.data(&data, e)).unwrap();
    // Every Event command with its stream + the streams' first/last key ticks.
    for (si, s) in n.streams.iter().enumerate() {
        let events: Vec<(u8, Vec<u32>)> = s.decode().into_iter().filter_map(|c| match c {
            nis::Cmd::Event { kind, params } => Some((kind, params)),
            _ => None,
        }).collect();
        if !events.is_empty() {
            eprintln!("[gondola] stream {si} keys {}..{} events {events:?}", s.first_key, s.last_key);
        }
    }
    // Stream 7's command sequence (a window stream): what ends it?
    let cmds: Vec<String> = n.streams[7].decode().into_iter().map(|c| format!("{c:?}")).collect();
    eprintln!("[gondola] stream 7 commands: {cmds:?}");
}
