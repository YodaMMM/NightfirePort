//! 07000003's `HOTOBJECT` (map 50 placement 495, a 0x20 breakable whose glist carries 0x400) is
//! drawn only under the x-ray goggles: `View_AddObjects` 0x1e62d8:38-47 skips a 0x400 or 0x10
//! object in the normal view. Its breakable slot must stay empty; the x-ray pass draws it.
//!
//! Opt-in via NF_ISO.

#[test]
fn the_hotobject_frame_is_not_drawn_in_the_normal_view() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_ref().unwrap();
    let hidden: Vec<usize> = sim.drones.breaks.list.iter().filter(|b| b.draw_hidden).map(|b| b.placement).collect();
    assert_eq!(hidden.len(), 1, "exactly one x-ray-only breakable on 03 [D the all-level check]: {hidden:?}");
    let k = sim.drones.doors.list.len() + sim.drones.sss.list.len()
        + sim.drones.breaks.list.iter().position(|b| b.draw_hidden).unwrap();
    let m = scene.door_slot_matrix(sim, k);
    assert!(m[3][1] < -1000.0, "its breakable slot collapses (not drawn): {:?}", m[3]);
}
