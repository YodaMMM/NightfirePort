//! level-13-audit: the 07000013 main-map data web, pinned off the disc.
//! Pins the census the audit doc (docs/levels/07000013-audit.md) was written from: the per-kind
//! placement counts, the writer/read channel web, and the script ids the main-map SPs reference.
//! Evidence class [D]: every assertion is a disc byte read through the level parser.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::Placement;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn param(p: &Placement, k: u32) -> u32 {
    p.params.iter().find(|x| x.0 == k).map_or(0, |x| x.1)
}

#[test]
fn the_07000013_main_map_data_web_matches_the_disc() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set; skipping"); return };
    let data = read_archive(&iso, "07000013.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    let ps = &lv.maps[mm].placements;
    assert_eq!(ps.len(), 173, "main-map placement count [D census]");

    let kind = |p: &Placement| p.flags & 0xff;
    let count = |k: u32| ps.iter().filter(|p| kind(p) == k).count();
    // per-kind counts [D census_07000013 dump]
    assert_eq!(count(0x0f), 12, "drones");
    assert_eq!(count(0x19), 48, "statics");
    assert_eq!(count(0x20), 33, "glass");
    assert_eq!(count(0x24), 1, "MP start");
    assert_eq!(count(0x2a), 4, "sky");
    assert_eq!(count(0x31), 2, "param-less fusebox art (SCRIPT_SHUTTER_*)");
    assert_eq!(count(0x33), 2, "hints");
    assert_eq!(count(0x47), 2, "zone cels");
    assert_eq!(count(0x71), 1, "rotor/hologram");
    assert_eq!(count(0xd9), 2, "ScriptPlayers");
    assert_eq!(count(0xdb), 2, "doors");
    assert_eq!(count(0xe2), 1, "SimpleScript");
    assert_eq!(count(0xe5), 19, "cover corner");
    assert_eq!(count(0xe6), 18, "cover low");
    assert_eq!(count(0xe8), 1, "load-level trigger");
    assert_eq!(count(0xe9), 6, "AI points");
    assert_eq!(count(0xea), 8, "TouchOnce triggers");
    assert_eq!(count(0xf0), 7, "pickups");
    assert_eq!(count(0xf1), 2, "drone spawners");
    assert_eq!(count(0xf9), 2, "sound triggers");

    // the two 0x31s are param-less art records: the game's SP refuses p0==0 [M 0x1db9a8 L18]
    for p in ps.iter().filter(|p| kind(p) == 0x31) {
        assert_eq!(param(p, 0), 0, "0x31 #{} p0", p.index);
    }

    // ScriptPlayers: the intro (mode 1) and the elevator SP (mode 0x40, mask 4, out ch 0x23) [D]
    let sps: Vec<_> = ps.iter().filter(|p| kind(p) == 0xd9).collect();
    let intro = sps.iter().find(|p| param(p, 2) == 1).expect("intro SP");
    assert_eq!(param(intro, 0), 0x0600_06d7, "intro script [D 07900013.bin]");
    let elev = sps.iter().find(|p| param(p, 2) == 0x40).expect("elevator SP");
    assert_eq!(param(elev, 0), 0x0600_0959, "elevator script [D 07000013.bin]");
    assert_eq!(param(elev, 5), 0x4, "mask 4 = E-usable [M 0x1a5170:121-124]");
    assert_eq!(param(elev, 6), 0x23, "out ch 0x23");

    // the TouchOnce out-channel web [D]
    let mut outs: Vec<u32> = ps.iter().filter(|p| kind(p) == 0xea).map(|p| param(p, 0)).collect();
    outs.sort();
    assert_eq!(outs, vec![0x28, 0x4a, 0x50, 0x56, 0x56, 0x64, 0x64, 0x64], "TouchOnce out chs");
    // all 8 are player-only contact masks [D]
    for p in ps.iter().filter(|p| kind(p) == 0xea) {
        assert_eq!(param(p, 1), 0x8, "TouchOnce #{} mask", p.index);
    }

    // ch 0x23's only writer is the elevator SP's completion [D scan]: no trigger/door/SS/pickup
    // writes it, and the gated readers are exactly SS #415 (ch p3) + hint (enable p3) [D]
    let ss: Vec<_> = ps.iter().filter(|p| kind(p) == 0xe2).collect();
    assert_eq!(ss.len(), 1);
    assert_eq!(param(ss[0], 3), 0x23, "Shutters2 trigger ch = the elevator SP's out ch");
    let hints: Vec<u32> = ps.iter().filter(|p| kind(p) == 0x33).map(|p| param(p, 3)).collect();
    assert!(hints.contains(&0x23), "hint #424 gated on 0x23 [D]");

    // the alarm chain: drone #326's alarm link p6=0x57 feeds both sound triggers' gate ch [D]
    let alarms: Vec<u32> = ps.iter().filter(|p| kind(p) == 0x0f && param(p, 6) != 0).map(|p| param(p, 6)).collect();
    assert_eq!(alarms, vec![0x57], "exactly one alarm-link drone");
    for p in ps.iter().filter(|p| kind(p) == 0xf9) {
        assert_eq!(param(p, 0), 0x57, "sound trigger gate = the alarm link ch");
        assert_eq!(param(p, 2), 0x289, "the alarm klaxon id");
    }

    // spawners: group 2 start ch 0x29 / group 1 start ch 0x2a; the ch-0x2a waiter drone #326 [D]
    let sp_chs: Vec<u32> = ps.iter().filter(|p| kind(p) == 0xf1).map(|p| param(p, 4)).collect();
    assert!(sp_chs.contains(&0x29) && sp_chs.contains(&0x2a), "spawner start chs");
    assert!(ps.iter().any(|p| kind(p) == 0x0f && param(p, 3) == 0x2a), "a drone waits on 0x2a");

    // the exit trigger: level 0x07300013, gate ch 0x4E (no writer in the level's data [D scan])
    let e8: Vec<_> = ps.iter().filter(|p| kind(p) == 0xe8).collect();
    assert_eq!(e8.len(), 1);
    assert_eq!(param(e8[0], 0), 0x0730_0013, "the load target");
    assert_eq!(param(e8[0], 2), 0x4e, "the gate ch");

    // doors: the stairwell door locks on ch 0xc8 (no writer — locked forever in the game's data),
    // the security-room door has none [D]
    let locks: Vec<u32> = ps.iter().filter(|p| kind(p) == 0xdb).map(|p| param(p, 3)).collect();
    assert!(locks.contains(&0xc8) && locks.contains(&0x0), "door lock chs");

    // glass: all type-2 PSI glass; the filters {0x30, 0x0} [D census]
    let mut filters: Vec<u32> = ps.iter().filter(|p| kind(p) == 0x20).map(|p| param(p, 2)).collect();
    filters.sort();
    assert_eq!(filters.first(), Some(&0x0), "the bullet-proof panes");
    assert_eq!(filters.last(), Some(&0x30), "the security-room pane's filter");
}
