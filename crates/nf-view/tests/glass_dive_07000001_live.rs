//! Double Cross Part 1 (07000001): the opening Yakuza dives through the hall window and the pane
//! shatters. `Break_Update` 0x12bfd8's drone arm — on 0x7000001 only, a drone in a glass pane's
//! contact list drains it by `Vec_SqDist3D(obj+0x30, obj+0x40) * 20` [M 0x12bfd8:20-27] — breaks
//! placement #130 as the guard's PlayScript dive carries him through it (PCSX2 frame 2175 after the
//! level start: the window in shards, the guard crouched inside [L 2026-10-26]). The port drained
//! panes only from bullets, so the window stayed whole.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

#[test]
fn the_opening_yakuza_dive_shatters_the_hall_window() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "07000001.bin", false).unwrap();
    let pane = |scene: &nf_view::scene::Scene| scene.sim.as_ref().unwrap().drones.breaks.list.iter()
        .find(|b| b.placement == 130).map(|b| (b.kind, b.dead)).expect("placement #130, the hall window [D]");
    assert_eq!(pane(&scene), (2, false), "#130 is a glass pane (kind 2), whole at load [D]");
    let mut broke_at = None;
    for t in 0..2400u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if broke_at.is_none() && pane(&scene).1 { broke_at = Some(t); }
    }
    let t = broke_at.expect("the dive shatters #130 [M 0x12bfd8:20-27, L PCSX2 frame 2175]");
    assert!((2100..2200).contains(&t), "it breaks as the guard goes through, just after the intro (tick {t})");
}
