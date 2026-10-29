//! The level-objects census (task level-objects-uncited): walks every 07* archive in FILES.BIN
//! through the port's own placement parser and pins, per level, how many dynamic-object
//! placements of the task's kinds exist — the disc evidence for which levels reach each
//! function family [D].
//!
//! Kinds (all dispatched from `parsemap_create_dynamic_objects` 0x1d04f0 [M]):
//! - 0x2F → `Copter_Create` 0x17d340 (Copter_Update 0x17dbb8 → Copter_FireMissile 0x17d9f0)
//! - 0x36 → `Sub_Create` 0x1ad6c0 (class 74: Sub_Update 0x1ad200 …)
//! - 0x37 → `MiniSub_Create` 0x182130 (class 77: MiniSub_Update 0x181f40 …)
//! - 0x38 → `Create_SpaceLaser` 0x1aa8b8 (SpaceLaser_Update 0x1aa3f8 → SpaceLaser_Register 0x1aa9e8)
//! - 0xE0 → `Creature_Create` 0x134ab8 (class 35: Creature_Update 0x135268 …)
//!
//! The 0xE0 pin is the dead-data proof for the whole Creature family: the only creator of a
//! class-0x23 object is Creature_Create [M 0x134ab8:63; exhaustive obj+0xff-writer sweep],
//! and its only caller is this kind-0xE0 arm [M 0x1d04f0:351-357] — so if no level ships
//! kind 0xE0, Creature_* is unreachable on retail data.
//!
//! Opt-in like the goldens: set `NF_ISO`; without it every test prints "skip" and passes.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, levelbin, map, placement};
use std::collections::BTreeMap;
use std::sync::OnceLock;

fn census() -> Option<&'static BTreeMap<(String, u8), u64>> {
    static C: OnceLock<Option<BTreeMap<(String, u8), u64>>> = OnceLock::new();
    let c = C.get_or_init(|| {
        let path = std::env::var_os("NF_ISO")?;
        Some(census_impl(&IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"))))
    });
    c.as_ref()
}

/// (level file name, placement kind) → placement count.
fn census_impl(iso: &IsoImage<std::fs::File>) -> BTreeMap<(String, u8), u64> {
    let (files, bin) = disc::files_bin_entries(iso).unwrap();
    let mut c: BTreeMap<(String, u8), u64> = BTreeMap::new();
    for f in files.iter().filter(|f| {
        f.name.len() >= 10 && f.name[..2].eq_ignore_ascii_case("07") && f.name.to_ascii_lowercase().ends_with(".bin")
    }) {
        let data = iso
            .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64)
            .unwrap();
        let lb = levelbin::LevelBin::parse(&data).unwrap();
        for e in lb.entries.iter() {
            if !matches!(e.kind, 0 | 1 | 2 | 11) {
                continue;
            }
            let bytes = lb.data(&data, e);
            let Ok(mf) = map::MapFile::parse(bytes) else { continue };
            for block in mf.all(map::id::MAP_DATA_STATIC) {
                for p in placement::parse(block.payload(bytes)).unwrap() {
                    if p.flags & 0xa000 != 0 {
                        continue; // [M 0x1d1b68 — cels/templates are not created]
                    }
                    let kind = (p.flags & 0xff) as u8;
                    if matches!(kind, 0x2f | 0x36 | 0x37 | 0x38 | 0xe0) {
                        *c.entry((f.name.to_ascii_uppercase(), kind)).or_default() += 1;
                    }
                }
            }
        }
    }
    c
}

/// The pinned disc truth, from the first walk (2026-10-26, slim ISO):
/// - kind 0x2F (copter): 07000005 ×5, 07000008 ×3, 0700000B ×3, 07000012 ×2, 07000046 ×1.
///   `Copter_Create` 0x17d340:42-45 turns EVERY kind-0x2F placement into a path node and only
///   param-0==0 records into copter objects — the docs' "2 copters on 05 / 1 on 08" [D
///   docs/levels/07000005.md:137, docs/levels/07000008.md:45] are the param-0==0 subset.
/// - kind 0x38 (space laser): 0700001B ×1 [D docs/levels/0700001b.md:73].
/// - kinds 0x36 (Sub) / 0x37 (MiniSub) / 0xE0 (Creature): zero rows on every 07* archive,
///   driving archives included — the creators and both update families are unreachable on
///   retail data [D; the walking filter is the same one script_census.rs uses].
fn pinned() -> Vec<((String, u8), u64)> {
    vec![
        (("07000005.BIN".to_string(), 0x2f), 5),
        (("07000008.BIN".to_string(), 0x2f), 3),
        (("0700000B.BIN".to_string(), 0x2f), 3),
        (("07000012.BIN".to_string(), 0x2f), 2),
        (("0700001B.BIN".to_string(), 0x38), 1),
        (("07000046.BIN".to_string(), 0x2f), 1),
    ]
}

#[test]
fn placement_kinds_match_the_disc() {
    let Some(c) = census() else { eprintln!("skip: NF_ISO not set"); return };
    let mut actual: Vec<((String, u8), u64)> = c.iter().map(|(k, v)| (k.clone(), *v)).collect();
    actual.sort();
    let expected = pinned();
    if actual != expected {
        panic!(
            "census drift — update `pinned()` and docs/systems/level-objects-uncited.md together.\nactual: {:?}\nexpected: {:?}",
            actual, expected
        );
    }
}

/// The Creature dead-data proof as a number: zero kind-0xE0 placements in every shipped level
/// [D; the creator+class sweep is cited in crates/nf-game/src/worldobj.rs].
#[test]
fn creature_kind_0xe0_never_ships() {
    let Some(c) = census() else { eprintln!("skip: NF_ISO not set"); return };
    assert!(
        !c.iter().any(|((_, k), _)| *k == 0xe0),
        "a level now ships placement kind 0xe0 — Creature_* is reachable and must be ported"
    );
}
