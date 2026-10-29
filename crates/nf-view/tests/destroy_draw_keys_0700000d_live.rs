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
