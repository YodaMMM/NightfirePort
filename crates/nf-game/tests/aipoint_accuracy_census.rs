//! The AIPoint accuracy census (guard-goal-arrival-radii): every 07* level's 0xE9 placements
//! through the port's own `AiPoint::from_placement` — param 1 is the goal accuracy the game hands
//! `AINetwork_SetupGoalPosition` at the alarm/run-to point goals [M FindAlarmPoint 0x153008:20-23,
//! FindRunToPoint 0x153168:39-43; the port arrives under it via `goal_arrival`]. The census pins
//! which levels' point goals move when the flat 2.0 stand-in goes. Opt-in like the goldens: set
//! `NF_ISO`; without it the test prints "skip" and passes.

use nf_formats::iso9660::IsoImage;
use std::collections::BTreeMap;

#[test]
fn the_aipoint_accuracy_census() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    // level -> (accuracy value -> count), only the groups the goal paths use (0 run-to team 1,
    // 1, 2 run-to team 3 fallback, 8 alarm panels, 9 run-to team 2).
    let mut census: BTreeMap<String, BTreeMap<u32, usize>> = BTreeMap::new();
    for fe in entries.iter().filter(|e| e.name.len() == 12 && e.name.starts_with("070") && e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(level) = nf_formats::level::Level::parse(&data) else { continue };
        let Some(m) = level.main_map() else { continue };
        let mut accs: BTreeMap<u32, usize> = BTreeMap::new();
        for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0xe9) {
            let point = nf_game::aipoint::AiPoint::from_placement(p);
            *accs.entry(point.accuracy as u32).or_default() += 1;
        }
        if !accs.is_empty() { census.insert(fe.name.to_ascii_uppercase(), accs); }
    }
    // Pin the shape of the table [D]: print every level's distribution and keep a coarse assertion —
    // every accuracy the disc carries is one of the small values the decompile census shows
    // (0.5/1.0/2.0/3.0-class) — a huge raw value would mean the parse broke.
    let mut lines = Vec::new();
    for (lvl, accs) in &census {
        let s = accs.iter().map(|(a, n)| format!("{a}x{n}")).collect::<Vec<_>>().join(" ");
        lines.push(format!("{lvl}: {s}"));
        assert!(accs.keys().all(|a| *a <= 16), "unexpected accuracy {lvl}: {s}");
    }
    eprintln!("== AIPoint accuracy per level (accuracy x count) ==\n{}", lines.join("\n"));
    assert!(!census.is_empty(), "no 0xE9 placements found on any 07* level");
}
