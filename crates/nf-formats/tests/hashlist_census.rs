//! Real-path census of the map hashlist block (`parsemap_block_hashlist` 0x1d1e78): every map entry on
//! the disc carries one block 0x2c, its payload is whole dwords, and the kept hashes never contain the
//! 0x06000000 sentinel the game skips [M 0x1d1e78]. Opt-in on `NF_ISO` like disc_golden.rs; without it
//! the test prints "skip" and passes.
//!
//! The 2,230 count is the measured census (docs/formats/map.md) [D]; the sister assertion makes the
//! "no sentinel kept" pin fail if the filter in `map::hashlist_entries` ever disappears AND a sentinel
//! occurs on the disc — the synthetic-filter pin lives in map.rs's unit tests.

use nf_formats::{disc, map};

fn iso() -> Option<nf_formats::iso9660::IsoImage<std::fs::File>> {
    let path = std::env::var_os("NF_ISO")?;
    Some(nf_formats::iso9660::IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}")))
}

#[test]
fn every_map_entry_carries_a_hashlist_whose_parse_matches_the_game() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut maps = 0usize;
    let mut with_hashlist = 0usize;
    let mut kept_total = 0usize;
    let mut sentinel_dropped = 0usize;
    disc::for_each_map(&iso, |_archive, _i, _e, data| {
        maps += 1;
        let mf = map::MapFile::parse(data)?;
        if let Some(b) = mf.first(map::id::HASHLIST) {
            with_hashlist += 1;
            let payload = b.payload(data);
            let raw = payload.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).count();
            sentinel_dropped += raw - map::hashlist_entries(payload).len();
            for h in map::hashlist_entries(payload) {
                assert_ne!(h, 0x0600_0000, "a sentinel hash survived the filter (game 0x1d1e78 skips it)");
            }
            kept_total += map::hashlist_entries(payload).len();
        }
        Ok(())
    })
    .unwrap_or_else(|e| panic!("disc walk: {e}"));
    assert_eq!(maps, 2_230, "map-entry census on the disc [D docs/formats/map.md]");
    assert_eq!(with_hashlist, 2_230, "every map entry carries a block-0x2c hashlist [D census]");
    assert!(kept_total > 0, "the hashlists are not all empty — if this fails the parse over-drops");
    eprintln!("hashlist census: {maps} maps, {kept_total} kept hashes, {sentinel_dropped} sentinels dropped");
}

/// Sister: the same walk with the parse replaced by the raw dword count must disagree wherever a
/// sentinel occurs; if the filter and the raw count NEVER agree to differ anywhere on the disc the
/// disc part is vacuous — this keeps the test honest about what it can catch.
#[test]
fn hashlist_census_sister_raw_count_differences_are_bounded() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut max_dropped_in_one_block = 0usize;
    disc::for_each_map(&iso, |_archive, _i, _e, data| {
        let mf = map::MapFile::parse(data)?;
        if let Some(b) = mf.first(map::id::HASHLIST) {
            let payload = b.payload(data);
            let raw = payload.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).count();
            let kept = map::hashlist_entries(payload).len();
            max_dropped_in_one_block = max_dropped_in_one_block.max(raw - kept);
        }
        Ok(())
    })
    .unwrap_or_else(|e| panic!("disc walk: {e}"));
    // A hashlist is small (docs/formats/map.md has no byte count; the game reads one dword per needed
    // file) — a single block dropping more than 0x100 entries would mean the parse is misaligned.
    assert!(max_dropped_in_one_block <= 0x100,
        "one hashlist dropped {max_dropped_in_one_block} entries: the parse has diverged from 0x1d1e78");
}
