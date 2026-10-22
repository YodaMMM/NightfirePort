//! fix-searchlight-spotray (task 934): the spot ray IS the game's mask-0x18 static ray.
//!
//! The game casts `Collide_RayIntersect(head pos, 100-unit aim, head cel, head obj, NULL, hits, 0,
//! 0x18)` [M Searchlight_Update 0x1cb900:201-208, disasm 0x1cbdb0-0x1cbe2c]; mask 0x18's bit 0x10
//! makes Collide_Pick skip the per-cel DYNAMIC-object walk and the ForcedList [M 0x1eb320:314-328,
//! 370-374; disasm 0x1eb480-0x1eb49c] — the ray tests ONLY map-static nodes and the cel's intrinsic
//! geometry. The searchlight parts are dynamic objects [M 0x1cb4f8:73-76 obj+0xD8 = the placement's
//! glist; control_link_object_to_cel 0x1c9680 -> 0x1346b8:16-24], so the parts' own meshes never
//! block the ray, and NOTHING excludes the level's static housing — the ray legitimately lands on
//! it (the port's static world measures the first hit ~0.68 units below every 07000005 head). The
//! old own-housing box skip (the pre-934 stand-in) stood in for a rule the game does not have: its
//! PickObj refusal (0x1eae80:57-59) is unreachable under mask 0x10, its world-space slab form once
//! pushed the skip 57 units out (the re-lit head's spot stayed at −10000), and its local-slab form
//! divided a ray FRACTION by 100 (a 100x scale no-op). This task deletes it; the spot lands on the
//! static segment's first hit exactly like the game's ray.
//!
//! Real path (`nf_view::load` + `Sim::step_bond`), ISO-gated.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The static segment's first hit is the spot's landing — no housing exclusion, no box skip.
#[test]
fn the_spot_lands_on_the_static_segments_first_hit() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&path, "07000006.bin", false).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..300 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    // Wait for a sweeping head with a linked spot (07000006's courtyard heads all carry one).
    let mut head = None;
    for _ in 0..600 {
        sim.step_bond(Default::default(), false, false, false);
        head = sim.drones.searchlights.parts.iter().position(|p| p.kind == 1 && p.state == 0 && p.links[2].is_some());
        if head.is_some() { break; }
    }
    let h = head.expect("a sweeping head with a linked spot");
    let (hp, yaw, pitch) = {
        let hd = &sim.drones.searchlights.parts[h];
        (hd.pos, hd.yaw, hd.pitch)
    };
    // The same ray the update casts (the sweep's aim, 100 units).
    let dir = nf_game::searchlight::spherical(100.0, yaw, -pitch);
    let end = [hp[0] + dir[0], hp[1] + dir[1], hp[2] + dir[2]];
    let expect = sim.world.segment_surface(hp, end, false);
    let Some((et, en, _)) = expect else { eprintln!("sweep aim missed the static world this tick — not a pinned case"); return };
    let want = [hp[0] + (end[0] - hp[0]) * et + en[0] * 0.01, hp[1] + (end[1] - hp[1]) * et + en[1] * 0.01, hp[2] + (end[2] - hp[2]) * et + en[2] * 0.01];
    // One more tick lands the spot at that hit (the update reads the state the tick set).
    sim.step_bond(Default::default(), false, false, false);
    let spot = &sim.drones.searchlights.parts[sim.drones.searchlights.parts[h].links[2].unwrap()];
    // The head may have re-swept on that tick — accept either the pinned aim's landing or the new
    // aim's, but NEVER a box-skip displacement: check BOTH the pinned aim and a re-computed aim.
    let hd = &sim.drones.searchlights.parts[h];
    let dir2 = nf_game::searchlight::spherical(100.0, hd.yaw, -hd.pitch);
    let end2 = [hd.pos[0] + dir2[0], hd.pos[1] + dir2[1], hd.pos[2] + dir2[2]];
    let near = |a: [f32; 3], b: [f32; 3]| (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
    if hd.state == 0 && hd.links[2].is_some() {
        if let Some((t2, n2, _)) = sim.world.segment_surface(hd.pos, end2, false) {
            let want2 = [hd.pos[0] + (end2[0] - hd.pos[0]) * t2 + n2[0] * 0.01, hd.pos[1] + (end2[1] - hd.pos[1]) * t2 + n2[1] * 0.01, hd.pos[2] + (end2[2] - hd.pos[2]) * t2 + n2[2] * 0.01];
            assert!(near(spot.pos, want) < 1e-3 || near(spot.pos, want2) < 1e-3,
                "the spot sits on the static segment's first hit [M 0x1cb900:201-248]: got {:?}, want {:?} (pinned aim) or {:?} (re-swept aim)",
                spot.pos, want, want2);
            return;
        }
    }
    assert!(near(spot.pos, want) < 1e-3, "the spot sits on the static segment's first hit: got {:?}, want {:?}", spot.pos, want);
}

/// Sister: the parts' own meshes are NOT in the static ray world — the game's parts are dynamic
/// objects (obj+0xD8 = their glist, linked into the cel's dynamic list [M 0x1cb4f8:73-76,
/// 0x1c9680]) and the spot ray's mask 0x18 skips the dynamic walk [M 0x1eb320:314-328]. A short
/// segment through each part's own volume hits nothing; if the part meshes ever join the static
/// world (or a box skip eats the ray at the origin), this fails.
#[test]
fn the_parts_own_meshes_never_block_the_ray() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let scene = nf_view::load(&path, "07000006.bin", false).expect("load 07000006");
    let sim = scene.sim.as_ref().unwrap();
    let w = &sim.world;
    let mut checked = 0;
    for part in &sim.drones.searchlights.parts {
        // A ±0.2-unit cross through the part's own centre: its own gfx box spans ~±0.27
        // [L ee_start_05.bin head 016dfe70], so a mesh present in the static world would wall it.
        let a = [part.pos[0] + 0.2, part.pos[1], part.pos[2]];
        let b = [part.pos[0] - 0.2, part.pos[1], part.pos[2]];
        assert!(w.segment_surface(a, b, false).is_none(),
            "kind {} part at {:?}: its own mesh must not be static geometry the spot ray tests [M 0x1eb320:314-328]", part.kind, part.pos);
        checked += 1;
    }
    assert!(checked >= 4, "the level's searchlight parts were enumerated");
}
