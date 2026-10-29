//! The doors' and interactables' shared colour entry stays white after a guard dies.
//!
//! The palette's colour table is laid out at build: [characters][attachments][pickups][searchlight
//! parts][col_one], `col_one` being the 1.0 entry every door and untinted interactable draws with
//! (`Scene::skin_layout`). A guard's death drops his gun as a NEW pickup (`DroneWeap_DropWeapon` →
//! `Pickup_Create` 0x18dda0 [M]), so the sim's pickup list outgrows the build's pickup section. The
//! palette used to write that pickup's ambient multiplier one slot past the section — on 07000014
//! (no searchlights) straight into `col_one`, dimming every door and interactable to the cel ambient
//! at the drop (the level's dark blue 28/36/78). The drop itself draws as its attachment, so the
//! extra pickup has no slot of its own here.
//!
//! Opt-in via NF_ISO.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::Walker;

#[test]
fn a_dropped_gun_leaves_the_shared_colour_white() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap());
    for _ in 0..4000 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
    for _ in 0..60 { w.tick(Default::default(), false); }
    let (layout, _) = w.scene.skin_layout();
    let one_ok = |s: &nf_view::scene::Scene| {
        let (_, cols) = s.skin_palette(&layout, None, 0.0, nf_view::scene::ViewOpts::default(), None);
        cols[layout.col_one as usize]
    };
    assert_eq!(one_ok(&w.scene), [1.0; 4], "col_one is white at the start");
    let n_pick0 = w.scene.sim.as_ref().unwrap().drones.pickups.len();
    let di = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        (0..sim.drones.list.len())
            .filter(|&i| { let d = &sim.drones.list[i]; d.present && d.enabled && !d.corpse && d.health > 0.0 })
            .min_by(|&a, &b| {
                let f = |p: [f32; 3]| (p[0] - bp[0]).powi(2) + (p[2] - bp[2]).powi(2);
                f(sim.drones.list[a].pos).partial_cmp(&f(sim.drones.list[b].pos)).unwrap()
            })
            .expect("an armed guard on 07000014")
    };
    // Test setup, not the path under test: stand 3 units from the guard so the real fire path kills him.
    { let sim = w.scene.sim.as_mut().unwrap(); let dp = sim.drones.list[di].pos; sim.bond.pos = [dp[0] - 3.0, dp[1], dp[2]]; }
    assert!(w.shoot(di, "the nearest guard"), "the guard dies on the real fire path");
    for _ in 0..60 { w.tick(Default::default(), false); }
    let n_pick1 = w.scene.sim.as_ref().unwrap().drones.pickups.len();
    assert!(n_pick1 > n_pick0, "the death drops a gun as a new pickup ({n_pick0} -> {n_pick1}) [M 0x18dda0]");
    assert_eq!(one_ok(&w.scene), [1.0; 4], "the doors' shared colour entry stays white after the drop");
}

/// Sister: the same overflow on the OLD write (unbounded by the section) does reach `col_one` —
/// the check above can fail.
#[test]
fn an_unbounded_pickup_write_would_reach_col_one() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let scene = nf_view::load_with_carry(&iso, "07000014.bin", false, None).unwrap();
    let (layout, _) = scene.skin_layout();
    let n_pick = layout.col_sl - layout.col_pick;
    let n_sl = layout.col_one - layout.col_sl;
    assert_eq!(n_sl, 0, "07000014 has no searchlight colour slots [D]");
    assert_eq!(layout.col_pick + n_pick, layout.col_one, "the first extra pickup index lands on col_one");
}
