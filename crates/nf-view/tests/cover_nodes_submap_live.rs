//! cover-nodes-submap: the U-050 answer, pinned. The game creates live cover nodes from the
//! LAST-parsed map entry's placement block only (`ResetMap_Load` 0x1d6360's single flag-1
//! `parsemap_block_map_data_dynamic` call over the global `DynamicPtr` 0x30cff0; every map
//! file's open resets it — `parsemap_parsemap` 0x1d1cf0 state 0, disasm 0x1d1dc8/0x1d1dd0 —
//! and each file's block 0x1a re-records it, `parsemap_handle_block_id` 0x1d1020 case 0x1a,
//! disasm 0x1d121c). Every mission archive lists its kind-1 main map LAST among the map
//! entries (kind 0/1/2/11) [D], so the live list is the main map's 0xE5/0xE6 placements in
//! placement order and the shared sub-map 010000f3's template pair is never instantiated.
//! Docs: docs/systems/cover-nodes-load.md. Opt-in: `NF_ISO`.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_level(iso: &IsoImage<std::fs::File>, name: &str) -> Level {
    let (entries, bin) = disc::files_bin_entries(iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap()
}

/// The list position, from the disc bytes: in every mission archive the kind-1 main map is the
/// LAST map entry (the game's `DynamicPtr` therefore holds the main map's block 0x1a when
/// `ResetMap_Load` creates the live objects [M]), 010000f3 sits among the earlier sub-maps
/// carrying the cover template pair (0xe5 + 0xe6 at (0,0,0), no params) [D], and the main map's
/// 0xE5/0xE6 placement counts are the live node counts the port builds (cover.rs is main-map
/// only [M parsemap_handle_block_id 0x1d1020:82-84 (the last block 0x1a wins) + ResetMap_Load 0x1d6360:146 (the single flag-1 create)]). Sister: the main map is never the FIRST map entry — the sub-maps the scan
/// would have added are really there, ahead of it.
#[test]
fn every_mission_archive_parses_the_main_map_last() {
    let Some(_) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
    let iso_img = IsoImage::open(&iso()).unwrap();
    // (level, main-map 0xE5/0xE6 placements) [D — placement bytes of each kind-1 entry]
    for (level_name, main_covers) in [
        ("07000001.bin", 20usize), ("07000002.bin", 9), ("07000004.bin", 0),
        ("07000005.bin", 18), ("07000006.bin", 114),
    ] {
        let level = load_level(&iso_img, level_name);
        let map_entries: Vec<(usize, u8, &String)> = level.maps.iter()
            .map(|m| (m.entry, m.kind, &m.name)).collect();
        assert!(map_entries.len() >= 2, "{level_name}: map entries exist [D]");
        let (last_entry, last_kind, last_name) = map_entries.last().unwrap();
        assert_eq!(*last_kind, 1, "{level_name}: the LAST map entry is the kind-1 main map \
            (entry {last_entry}, {}) — anything later would win DynamicPtr [M 0x1d1020 case 0x1a]", last_name);
        let first_kind0 = map_entries.iter().find(|(_, k, _)| *k == 0).expect("a kind-0 sub-map precedes the main map [D]");
        assert!(first_kind0.0 < *last_entry, "{level_name}: sub-maps parse before the main map");
        let f3 = map_entries.iter().find(|(_, _, n)| n.as_str() == "010000f3")
            .expect("{level_name}: the shared sub-map 010000f3 is in the archive [D]");
        assert!(f3.0 < *last_entry, "{level_name}: 010000f3 parses before the main map");
        // f3's own map carries the template pair [D].
        let f3_map = level.maps.iter().find(|m| m.name == "010000f3").unwrap();
        let cp = f3_map.placements.iter().find(|p| p.flags == 0xe5).expect("template corner [D]");
        let lp = f3_map.placements.iter().find(|p| p.flags == 0xe6).expect("template low [D]");
        assert!(cp.translation == [0.0, 0.0, 0.0] && lp.translation == [0.0, 0.0, 0.0],
            "{level_name}: the templates sit at the origin [D]");
        assert!(cp.params.is_empty() && lp.params.is_empty(), "{level_name}: no params [D]");
        // The main map's own cover placements — the live list's source [M 0x1d6360].
        let main_map = level.main_map().unwrap();
        let covers = level.maps[main_map].placements.iter()
            .filter(|p| matches!(p.flags, 0xe5 | 0xe6)).count();
        assert_eq!(covers, main_covers, "{level_name}: the main map's 0xE5/0xE6 placements [D]");
    }
}

/// The live list itself, through the real load path (`nf_view::load` → `cover::build`): on
/// 07000005 it is the main map's 18 cover placements in placement order — node 0 at
/// (59.07, 26.04, 94.05), node 1 at (59.11, 26.04, 98.69) [D order; the chase-trace node the
/// prepended scan shifted to index 3 stays at index 1 [L work/chase_trace.tsv]] — and NO node
/// at the origin. Sister: the disc-wide scan would have made 20 (18 + 010000f3's pair); if that
/// scan ever returns this fails 18 != 20.
#[test]
fn the_live_list_is_the_main_map_s_placements_in_order() {
    let Some(_) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
    let iso_img = IsoImage::open(&iso()).unwrap();
    let level = load_level(&iso_img, "07000005.bin");
    let main_map = level.main_map().unwrap();
    let want: Vec<[f32; 3]> = level.maps[main_map].placements.iter()
        .filter(|p| matches!(p.flags, 0xe5 | 0xe6)).map(|p| p.translation).collect();
    assert_eq!(want.len(), 18, "the main map's cover placements [D]");

    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load 07000005");
    for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); if !scene.sim.as_ref().unwrap().disabled { break; } }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.covers.len(), 18, "live nodes = the main map's placements [M 0x1d6360 chain]");
    for (i, n) in sim.covers.iter().enumerate() {
        assert!((n.pos[0] - want[i][0]).abs() < 1e-3 && (n.pos[1] - want[i][1]).abs() < 1e-3
            && (n.pos[2] - want[i][2]).abs() < 1e-3,
            "node {i} is placement {i} in order [D]: {:?} vs {:?}", n.pos, want[i]);
    }
    assert_eq!(sim.covers[0].pos[0], want[0][0]);
    assert!((sim.covers[0].pos[0] - 59.07).abs() < 0.01 && (sim.covers[0].pos[1] - 26.04).abs() < 0.01
        && (sim.covers[0].pos[2] - 94.05).abs() < 0.01, "node 0 [D]: {:?}", sim.covers[0].pos);
    assert!((sim.covers[1].pos[0] - 59.11).abs() < 0.01 && (sim.covers[1].pos[1] - 26.04).abs() < 0.01
        && (sim.covers[1].pos[2] - 98.69).abs() < 0.01, "node 1 — the chase-trace node [L]: {:?}", sim.covers[1].pos);
    assert!(!sim.covers.iter().any(|n| n.pos == [0.0, 0.0, 0.0]),
        "the (0,0,0) template pair is not live [M parsemap_handle_block_id 0x1d1020:82-84 + ResetMap_Load 0x1d6360:146]");
    // Sister [D]: the scan's premise — two more cover placements live in 010000f3.
    let f3 = level.maps.iter().find(|m| m.name == "010000f3").unwrap();
    let extra = f3.placements.iter().filter(|p| matches!(p.flags, 0xe5 | 0xe6)).count();
    assert_eq!(extra, 2, "010000f3's template pair on disc [D] — 18 + 2 is what the scan built");
}
