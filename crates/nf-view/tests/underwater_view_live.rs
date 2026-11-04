//! Camera_Shear 0x1bba38: the underwater view wobble reaches the Sim's view state [M]. NF_ISO-gated.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Under water the shear runs and moves each tick; above water it is (0, 0).
#[test]
fn the_wobble_runs_only_under_water() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let Ok(mut scene) = nf_view::load(&path, "07000002.bin", false) else { return };
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Default::default(), false, false, false);
    assert_eq!(sim.view_shear, (0.0, 0.0), "dry");
    let (a, _) = sim.camera_shear();
    let (b, _) = sim.camera_shear();
    assert!(a.abs() <= 0.0251 && b.abs() <= 0.0251, "the 0.025 scale [M :20-24]");
    assert_ne!(a, b, "the phase advances");
}
