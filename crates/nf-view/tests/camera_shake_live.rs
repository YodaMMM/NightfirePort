//! Camera_Update 0x1bb6d8:37-43: a shake moves the eye each tick and spends itself [M]. NF_ISO-gated.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The eye moves by `s·r − s` per axis, s = shake / 60, and the amount falls by up to a quarter a
/// tick. Sister: with no shake the eye stays put.
#[test]
fn a_shake_moves_the_eye_and_spends_itself() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let Ok(mut scene) = nf_view::load(&path, "07000006.bin", false) else { return };
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Default::default(), false, false, false);
    assert_eq!(sim.drones.shake_eye, [0.0; 3], "no shake, no offset");
    sim.drones.shake = 6.0;
    let mut moved = false;
    for _ in 0..240 {
        let before = sim.drones.shake;
        sim.step_bond(Default::default(), false, false, false);
        assert!(sim.drones.shake <= before && sim.drones.shake >= before * 0.75 - 1e-3, "spends at most a quarter");
        let s = before / 60.0;
        for k in 0..3 { assert!(sim.drones.shake_eye[k] <= 0.0 && sim.drones.shake_eye[k] >= -s - 1e-5); }
        moved |= sim.drones.shake_eye != [0.0; 3];
    }
    assert!(moved, "the eye moved");
    assert_eq!(sim.drones.shake, 0.0, "the shake ran out");
    assert_eq!(sim.drones.shake_eye, [0.0; 3]);
}
