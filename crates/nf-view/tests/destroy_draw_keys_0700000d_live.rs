//! The Destroy leaves' draw-side keys: `Scene::destroy_centres` pairs each drawn 0xe1 leaf with its
//! PLACEMENT index, and the per-frame draw gates (the dead leaf's hide, the hit-flash tint — lib.rs
//! `dead_destroy_centres` / `destroy_tints`) look the leaf up in `props.destroy` BY PLACEMENT. The old
//! gate indexed `props.destroy` with the placement number as a slot, so on 0700000d (11 Destroy
//! records, the C5 leaves at placements 93/96) it never found a leaf: the dead C5 door kept drawing
//! and the doorway never opened.
//!
//! Opt-in via NF_ISO.

#[test]
fn every_drawn_destroy_leaf_resolves_by_placement() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    let sim = scene.sim.as_ref().unwrap();
    assert!(!scene.destroy_centres.is_empty(), "0700000d draws Destroy leaves [D placements kind 0xe1]");
    for (_, pl) in &scene.destroy_centres {
        assert!(sim.drones.props.destroy.iter().any(|d| d.placement == *pl),
            "drawn leaf placement {pl} has a Destroy record");
    }
    assert!(scene.destroy_centres.iter().any(|(_, pl)| *pl == 96), "bigdoor02 #96 (the C5 leaf) is drawn [D]");
}

/// Sister: the old slot lookup cannot find the C5 leaf — placement 96 is past the record list, so
/// a gate keyed that way never fires.
#[test]
fn the_c5_placement_is_not_a_slot_index() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.props.destroy.get(96).is_none_or(|d| d.placement != 96),
        "slot 96 is not bigdoor02 ({} records)", sim.drones.props.destroy.len());
}

/// The C5 smash sets off the game's TWO fireballs: `Destroy_Smash` 0x12bb60:LAB_0012bc50 runs
/// `Explode_Create` 0x17cbd0 for EVERY leaf whose type byte is non-zero — bigdoor02's kill and
/// the same tick's channel-chain smash of bigdoor01 [M 0x12c4f8:29-33] both qualify (types 6
/// and 5 [D #96/#93 p2]), so both script-0x6000052 blasts exist. The twin's magnitude is the
/// leaf's parsed object sphere [M 0x12bcdc `lwc1 $f12, 140($16)`; 0x1d0fb0 obj+0x8c] — the
/// port's #93 object record carries the parse's ±1e7 sentinel bounds [D #93 obj 91], so its
/// sphere/scale read ~0 (the fireball art plays invisible there; live check: the game's
/// obj+0x8c for #93). The draw holds BLAST_COPIES slot sets per blast script
/// (concurrent_blasts_draw_live), so the twin no longer hides the door's fireball.
#[test]
fn the_c5_smash_sets_off_one_fireball_at_the_door() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    let f = sim.drones.frame;
    sim.drones.channels.set(0x1f, 1, f); // the C5 leaves' trigger channel [D p1 0x1f]
    sim.step_bond(Default::default(), false, false, false);
    let blasts: Vec<_> = sim.drones.blasts.iter().filter(|b| b.script_hash == 0x0600_0052).collect();
    assert_eq!(blasts.len(), 2, "the game's two same-tick C5 blasts [M 0x12bb60 + 0x12c4f8:29-33], got {:?}",
        blasts.iter().map(|b| (b.pos, b.radius)).collect::<Vec<_>>());
    let big = blasts.iter().find(|b| b.radius > 1.0).expect("the door blast has a real size");
    let p = big.pos;
    assert!((p[2] + 269.5).abs() < 1.0 && (p[0] - 134.4).abs() < 3.0, "the blast sits at the C5 door, got {p:?}");
    assert!((big.radius - 3.144_156 * 3.0).abs() < 0.01, "the door blast's radius is the leaf sphere·3 [M 0x12bcdc], got {}", big.radius);
}
