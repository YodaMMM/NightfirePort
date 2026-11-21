//! G-016: how `parsemap_block_portal_data` (0x1d15f8) resolves a block-0x21 record's two cel
//! endpoints, pinned on every level of `nf_view::dev::LEVELS` with `Level::parse` [D] against the
//! disassembly [M].
//!
//! The resolver [M 0x1d15f8, disasm 0x1d160c-0x1d1780]: `s3` (celA) and `s4` (celB) are zeroed ONCE
//! before the record loop (0x1d160c `move s4,zero`, 0x1d1614 `move s3,zero`); per record the walk
//! over `glb_world`'s cel chain (`cel+0x14`) runs `movz s3, cel, (cel+0x38 ^ (pCelList + a*0x50))`
//! and the same for `s4`/b (0x1d1768-0x1d177c). Nothing resets the two registers between records,
//! so an endpoint that names no runtime cel KEEPS the previous record's cel. Because
//! `build_alloc_cel` 0x1c93f8 prepends (`cel+0x14 = old head`), the walk meets the newest cel first
//! and the last `movz` match is the OLDEST cel holding that record (first parsed wins).
//!
//! On the disc [D] only 07000012 has records naming no cel: its main-map records 35-52 (18 of 53)
//! give both endpoints as record 552, the "Sky bridge" script player (a 0xd9 placement, no cel
//! bit), so the game hangs all 18 quads on record 34's pair, `corridors_a_8` (#203) and `Office-C`
//! (#550). The port used to drop them.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::scene;

fn levels() -> Option<Vec<(u32, Level)>> {
    let iso = std::env::var("NF_ISO").ok()?;
    let img = IsoImage::open(&iso).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).ok()?;
    let mut out = Vec::new();
    for &lv in nf_view::dev::LEVELS {
        let name = format!("{lv:08x}.bin");
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else { continue };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
        out.push((lv, Level::parse(&data).unwrap_or_else(|e| panic!("{name}: {e:?}"))));
    }
    Some(out)
}

/// Every block-0x21 record becomes a portal edge on every level: the stale-register rule leaves
/// none unresolved on the disc (the first record of each block always resolves), and every cel
/// placement resolves its record (by index or through the hash).
#[test]
fn every_portal_record_becomes_an_edge_on_every_level() {
    let Some(levels) = levels() else { eprintln!("NF_ISO not set, skipping"); return };
    for (lv, level) in &levels {
        let (_, g) = scene::parse_cel_graph(level);
        let records: usize = level.maps.iter().map(|m| m.portals.len()).sum();
        eprintln!("{lv:08x}: {records} records, {} edges, {} dropped", g.portals.len(), g.dropped_portals);
        assert_eq!(g.dropped_portals, 0, "{lv:08x}: {} portal records dropped", g.dropped_portals);
        assert_eq!(g.portals.len(), records, "{lv:08x}: edges != block-0x21 records");
        // No cel placement fails to resolve its object record (the static parse's `index <=
        // MapEntityCount` / hashtable paths [M 0x1d1998]).
        for (mi, m) in level.maps.iter().enumerate() {
            for p in m.placements.iter().filter(|p| p.creates_cel()) {
                let ok = if p.by_hash() { level.by_hash.contains_key(&p.hashcode) } else { (p.index as usize) < m.objects.len() };
                assert!(ok, "{lv:08x} map {mi}: a cel placement resolves no record");
            }
        }
    }
}

/// 07000012's records 35-52 ride record 34's cel pair (the stale `s3`/`s4`), each with its own quad.
#[test]
fn twelve_sky_bridge_records_ride_the_previous_records_cels() {
    let Some(levels) = levels() else { eprintln!("NF_ISO not set, skipping"); return };
    let Some((_, level)) = levels.iter().find(|(lv, _)| *lv == 0x0700_0012) else { return };
    let main = &level.maps[level.main_map().unwrap()];
    assert_eq!(main.portals.len(), 53);
    for q in &main.portals[35..53] { assert_eq!((q.cel_a, q.cel_b), (552, 552), "[D] the Sky bridge records"); }
    assert_eq!((main.portals[34].cel_a, main.portals[34].cel_b), (129, 456));
    let (_, g) = scene::parse_cel_graph(level);
    let corridor = g.own_cells_main[203].expect("#203 corridors_a_8 is a cel") as usize;
    let office = g.own_cells_main[550].expect("#550 Office-C is a cel") as usize;
    assert_eq!((g.portals[34].cel_a, g.portals[34].cel_b), (corridor, office));
    for k in 35..53 {
        assert_eq!((g.portals[k].cel_a, g.portals[k].cel_b), (corridor, office), "record {k}: the stale pair");
        assert_eq!(g.portals[k].corners, main.portals[k].corners, "record {k}: its own quad");
    }
}
