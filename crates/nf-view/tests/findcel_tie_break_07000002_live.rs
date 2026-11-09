//! findcel-tie-break on 07000002: the 13 kind-0xD8 placements whose create point overlaps more
//! than one room cel resolve through the game's `build_FindCel` 0x1c8cb0 collision-ray tie-break,
//! not the port's old first-parse-order stand-in.
//!
//! The game's create pick [M]: `RB_Create` 0x1d6bf0:81 → `build_LinkToRoom` 0x1c9680 →
//! `Control_BuildWorldSph` 0x133070 (the glist centre +0x10 through the object matrix, obj+0x80)
//! → `build_FindCel` 0x1c8cb0 at that point: the containing room cels (`cel+0x3c & 0x40000`,
//! `cel+0x40..0x58` inclusive bbox [M 0x1c8cb0:106-109]); when MORE than one contains the point,
//! per candidate (in `gbl_room_cels` order) an UP ray p→p+(0,len,0) and a DOWN ray
//! p→p−(0,min(up,len),0), both `Collide_RayIntersect` (mask 0x90) scoped to the candidate's OWN
//! geometry; the closer hit's distance (HITDATA+0xc) wins while it stays under the running best
//! (256.0 first, strict <, [M 0x1c8cb0:170-229 disasm]); no candidate wins → the FIRST candidate
//! stands ([M 0x1c8cb0's tail returns CelList_153[0]]); one candidate returns directly, none
//! returns NULL (the object stays unlinked, `build_LinkToRoom` 0 → no obj+0x20).
//!
//! The port: `Rbodies::create_scoped(level, Some(&world))` (the scene build passes the scoped
//! world) runs the same rule through `CollisionWorld::find_cel_report` and stores the picked
//! scope cell in `RbObj::cel`; the wind/buoyancy fields derive from the picked cel.
//!
//! [D] facts are read from the disc level bin in this file; the load goes through
//! `nf_view::load` — the real path (no teleport before the pin's ticks run).

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The main map's room-cel boxes in placement order (the game's `gbl_room_cels` candidates are
/// the flag-0x40000 cels; on the disc that is exactly the flags16 0xc023/0xc047 set [M
/// parseentity_fixup_entity 0x1d00c0's flag words: 0xc023 → 0x804c023, 0xc047 → 0x804c047]) —
/// the raw glist bbox each (the cel's +0x40..0x58 [M parsemap_block_map_data_static 0x1d1998]).
fn room_boxes(level: &Level) -> Vec<([f32; 3], [f32; 3])> {
    let main = level.main_map().unwrap_or(0);
    level.maps[main].placements.iter()
        .filter(|p| p.creates_cel() && matches!((p.flags & 0xffff) as u16, 0xc023 | 0xc047))
        .filter_map(|p| {
            let r = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
            r.and_then(|(mm, oi)| level.maps[mm].objects.get(oi)).map(|o| (o.min, o.max))
        }).collect()
}

fn contains(lo: &[f32; 3], hi: &[f32; 3], p: &[f32; 3]) -> bool {
    (0..3).all(|k| lo[k] <= p[k] && p[k] <= hi[k])
}

#[test]
fn the_multi_cel_placements_pick_the_game_s_tie_break_cel() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    // The level bin, for the [D] scan (placements, room boxes, object names).
    let img = IsoImage::open(&iso).unwrap();
    let (entries, files) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let e = entries.iter().find(|e| e.name == "07000002.bin").expect("07000002.bin on the disc");
    let data = img.read_bytes(files.lba as u64 * 2048 + e.offset as u64, e.size as u64).unwrap();
    let level = Level::parse(&data).expect("parse 07000002");
    let main = level.main_map().unwrap();
    let boxes = room_boxes(&level);

    // The real load path (nf_view::load → Scene::build → the scoped world → attach_level_world).
    let scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_ref().expect("sim");
    let rbs = &sim.drones.rbodies.list;
    assert_eq!(rbs.len(), 21, "the census: 21 kind-0xD8 placements on 07000002 [D]");

    // The parse-wide scope-cell counter replay (every creates_cel() placement counts, sub-maps
    // first, main last) — maps a scope cell back to its main-map room-cel ordinal for the print.
    let mut cell_to_room: std::collections::HashMap<u32, usize> = Default::default();
    let mut counter = 0usize;
    for (mi, map) in level.maps.iter().enumerate() {
        for p in &map.placements {
            if !p.creates_cel() { continue; }
            if mi == main && matches!((p.flags & 0xffff) as u16, 0xc023 | 0xc047) {
                cell_to_room.insert(counter as u32, cell_to_room.len());
            }
            counter += 1;
        }
    }

    // The scan ([D]): each 0xD8 placement's candidate count at the translation (the old
    // measurement — the task's 13) and at the world-sphere centre (the game's pick point
    // [M Control_BuildWorldSph 0x133070 through build_LinkToRoom 0x1c9680:41-45]).
    let mut multi_at_translation: Vec<(u16, usize)> = Vec::new();
    let mut multi_at_centre: Vec<(u16, usize)> = Vec::new();
    for (pi, p) in level.maps[main].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xd8) {
        let r = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
        let Some((mm, oi)) = r else { continue };
        let Some(o) = level.maps[mm].objects.get(oi) else { continue };
        let centre = nf_formats::placement::transform(&nf_formats::placement::object_matrix(p.rotation, p.translation), o.centre);
        let nt = boxes.iter().filter(|(lo, hi)| contains(lo, hi, &p.translation)).count();
        let nc = boxes.iter().filter(|(lo, hi)| contains(lo, hi, &centre)).count();
        if nt > 1 { multi_at_translation.push((pi as u16, nt)); }
        if nc > 1 { multi_at_centre.push((pi as u16, nc)); }
    }
    println!("07000002 [D scan]: {} multi-cel placements at the translation {multi_at_translation:?}; {} at the sphere centre {multi_at_centre:?}",
        multi_at_translation.len(), multi_at_centre.len());
    assert_eq!(multi_at_translation.len(), 13, "the task's [D] count: 13 placements overlap more than one cel at the translation");
    assert_eq!(rbs.iter().filter(|o| o.cel.is_some()).count(), rbs.len(), "every body resolved a create cel [M build_LinkToRoom 0x1c9680 links on a FindCel hit]");

    // The before/after print the task asks for: per multi-cel placement, the cel the OLD port
    // rule picked (first parse-order room cel containing the TRANSLATION) vs the cel the port
    // picks now (the game's tie-break at the world-sphere centre, `RbObj::cel`).
    for (pi, p) in level.maps[main].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xd8) {
        let r = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
        let Some((mm, oi)) = r else { continue };
        let Some(o) = level.maps[mm].objects.get(oi) else { continue };
        let centre = nf_formats::placement::transform(&nf_formats::placement::object_matrix(p.rotation, p.translation), o.centre);
        let nt = boxes.iter().filter(|(lo, hi)| contains(lo, hi, &p.translation)).count();
        let nc = boxes.iter().filter(|(lo, hi)| contains(lo, hi, &centre)).count();
        let old = boxes.iter().position(|(lo, hi)| contains(lo, hi, &p.translation));
        let body = rbs.iter().find(|b| b.placement == pi as u16);
        let new_room = body.and_then(|b| b.cel).and_then(|c| cell_to_room.get(&c).copied());
        if nt > 1 || nc > 1 {
            println!("rb #{:3} {:28} trans {:?} centre {:?} cand(t)={} cand(c)={} old_room={:?} new_room={:?} (scope cell {:?})",
                pi, o.name, p.translation, centre, nt, nc, old, new_room, body.and_then(|b| b.cel));
        }
    }

    // The picked cel's fields feed the body: a resolved cel always carries the buoyancy pair
    // ([M parseentity_fixup_entity 0x1d00c0's cel+0x90 rule] the create reads); the wind struct
    // rides the cel that a 0xDF placement landed in ([D] 07000002 does carry 0xDF placements —
    // the winds list is non-empty; the pans' room-12 pick changes which cel's wind they feel).
    println!("07000002 [D]: {} wind struct(s)", sim.drones.rbodies.winds.len());
    for b in rbs {
        assert_eq!(b.buoy.is_some(), b.cel.is_some(), "body #{}: buoyancy fields iff the cel resolved", b.placement);
    }

    // The multi-cel placements pin the game's rule outcome: the tie-break cel is one of the
    // candidates ([M the winner comes from CelList]) and the port's stored cell is the same
    // rule `CollisionWorld::find_cel_report` answers at the centre (the real-path world).
    for (pi, nc) in &multi_at_centre {
        let b = rbs.iter().find(|b| b.placement == *pi).expect("the body exists");
        let c = b.cel.expect("a multi-cel placement resolves");
        assert!(*nc > 1, "the scan agrees the placement is multi-cel");
        assert!(cell_to_room.contains_key(&c), "the picked cell {} is a main-map room cel", c);
    }

    // Real play: 240 settled ticks — the create pick holds (a Rot body never translates [M
    // EulerEvolve's type-0 position arm], so the game's obj+0x20 never re-links [M
    // control_handle_cel_change 0x133288 runs on the movement handler's moved pass]), the rest
    // state stays (no wind [D], the buoyancy gate shut at these cels), and the collision pass
    // (per-pass `find_cel`) leaves the rest bits alone.
    let mut w = Walker::new(scene);
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let before: Vec<(u16, Option<u32>, [f32; 3])> = {
        let s = w.scene.sim.as_ref().unwrap();
        s.drones.rbodies.list.iter().map(|b| (b.placement, b.cel, b.st.trans)).collect()
    };
    w.wait(240);
    {
        let s = w.scene.sim.as_ref().unwrap();
        for (b, (p0, c0, t0)) in s.drones.rbodies.list.iter().zip(&before) {
            assert_eq!(b.placement, *p0);
            assert_eq!(b.cel, *c0, "body #{}'s create cel holds through real play", b.placement);
            assert!(b.st.trans.iter().zip(t0.iter()).all(|(a, b)| (a - b).abs() < 1e-4),
                "body #{} never translates at rest [M 0x1d7810's type-0-only position arm]", b.placement);
            assert!(b.st.matrix.iter().flatten().zip([1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0].iter())
                .all(|(a, b)| (a - b).abs() < 1e-6), "body #{} drifted at rest", b.placement);
        }
    }
}
