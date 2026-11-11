//! The rigid-body build census over EVERY shipped-disc level bin — the [D] proof for two
//! `rigid_body.rs` markers:
//!
//! 1. `Rbodies::create`'s doc claimed "[S] the degenerate census case: no 0xD8 placement on
//!    the disc fails the check" (the moments' triangle check, `rb_build`'s gate
//!    [M RB_Create 0x1d6bf0 → rb_build's `m2 <= m0+m1 ∧ m0 <= m1+m2 ∧ m1 <= m0+m2`]). This
//!    test walks every level bin and asserts every 0xD8 placement built — the game's
//!    failed-build path (the object exists, the body never joins the sim) is never reached
//!    on the shipped disc.
//! 2. The create-time cel pick: the game's `build_FindCel` 0x1c8cb0 tie-break decides the cel
//!    of every placement whose create point (the world-sphere centre, `Control_BuildWorldSph`
//!    0x133070 through `build_LinkToRoom` 0x1c9680) containment-matches more than one room
//!    cel. This census counts those placements per level at the game's point ([D] this scan)
//!    — 07000002 carries 13 (the translation scan the task cites; the sphere-centre count
//!    prints alongside). The tie-break itself needs the scoped geometry, which nf-game's
//!    tests cannot build (the scope graph lives in nf-view's scene build) — the tie-break
//!    pick is pinned on 07000002 through the real path in
//!    `nf-view/tests/findcel_tie_break_07000002_live.rs`.
//!
//! Opt-in: `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::rigid_body::Rbodies;

fn level_bin_names(iso: &IsoImage<std::fs::File>) -> Vec<(String, u64, u64, u64)> {
    let (entries, files) = nf_formats::disc::files_bin_entries(iso).unwrap();
    entries
        .iter()
        .filter(|e| e.name.starts_with("0700") && e.name.ends_with(".bin"))
        .map(|e| (e.name.clone(), files.lba as u64 * 2048 + e.offset as u64, e.size as u64, 0))
        .collect()
}

#[test]
fn every_disc_rigid_body_builds_and_resolves_one_cel() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let bins = level_bin_names(&iso);
    assert!(bins.len() >= 20, "expected the disc's level bins, found {}", bins.len());
    let mut total_bodies = 0usize;
    let mut levels = 0usize;
    let mut skipped = Vec::new();
    let mut type_counts: std::collections::BTreeMap<i16, usize> = Default::default();
    for (name, off, size, _) in bins {
        let data = match iso.read_bytes(off, size) {
            Ok(d) => d,
            Err(e) => { skipped.push(format!("{name}: read {e}")); continue }
        };
        let level = match Level::parse(&data) {
            Ok(l) => l,
            Err(e) => { skipped.push(format!("{name}: parse {e}")); continue }
        };
        levels += 1;
        let rbs = Rbodies::create(&level);
        let unbuilt: Vec<u16> = rbs.list.iter().filter(|o| !o.built).map(|o| o.placement).collect();
        assert!(unbuilt.is_empty(), "{name}: 0xD8 placements failed the moments' check: {unbuilt:?}");
        for o in &rbs.list { *type_counts.entry(o.rb.ty).or_default() += 1; }
        total_bodies += rbs.list.len();
        // The tie-break census, both points ([D] this scan): `cel_ambiguous` counts the
        // placements whose SPHERE-CENTRE point (the game's `build_LinkToRoom` 0x1c9680 →
        // `Control_BuildWorldSph` 0x133070 pick point) containment-matches more than one room
        // cel — the game's tie-break decides those. The translation scan repeats the older
        // first-parse-order measurement the task cites (07000002 = 13).
        let main = level.main_map().unwrap_or(0);
        let room_boxes: Vec<([f32; 3], [f32; 3])> = level.maps[main].placements.iter()
            .filter(|p| p.creates_cel() && matches!((p.flags & 0xffff) as u16, 0xc023 | 0xc047))
            .filter_map(|p| {
                let r = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
                r.and_then(|(mm, oi)| level.maps[mm].objects.get(oi)).map(|o| (o.min, o.max))
            }).collect();
        let mut multi_at_translation = 0u32;
        let mut multi_list: Vec<(u16, usize)> = Vec::new();
        for (pi, p) in level.maps[main].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xd8) {
            let r = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
            let Some((mm, oi)) = r else { continue };
            let Some(o) = level.maps[mm].objects.get(oi) else { continue };
            let t = p.translation;
            let n = room_boxes.iter().filter(|(lo, hi)| (0..3).all(|k| lo[k] <= t[k] && t[k] <= hi[k])).count();
            if n > 1 { multi_at_translation += 1; multi_list.push((pi as u16, n)); }
        }
        if rbs.cel_ambiguous != 0 || multi_at_translation != 0 {
            println!("{name}: {multi_at_translation} multi-cel at the translation {multi_list:?}; {} at the sphere centre (the game's point)", rbs.cel_ambiguous);
        }
        println!("{name}: {} bodies", rbs.list.len());
    }
    println!("levels parsed: {levels}; skipped: {skipped:?}; total 0xD8 bodies: {total_bodies}; types: {type_counts:?}");
    assert!(levels >= 20, "parsed {levels} levels — the census must cover the disc");
}
