//! 07000014 fusebox: the laser must cut the VISIBLE cable run, which sits 1.5 m above the wire
//! entity's translation.
//!
//! [D] the box's wires are `OBJECT_GreenWire1` (hash 0x02000aa1) and `OBJECT_BlueWire1` (0x02000aa4);
//! their script entity matrix translation is (-89.3055, -2.2213, 30.787) while the wire geometry's local
//! y 1.49..1.72 puts the visible cables at world y -0.73..-0.50 — 1.5 m ABOVE the entity origin.
//! [M] the game collides the round against the wire OBJECT's geometry (`Collide_PickObj` 0x1eae80 →
//! obj+0xD0 → `FuseBox_Update` 0x1adfe8:88-123); the port used a 0.5-radius sphere at the entity
//! translation, so a beam on the cables never touched anything.
//!
//! The test stands at the reported spot, aims at the middle of the visible cable and holds the laser
//! watch: the box must lose wires. Pristine: the sphere is 1.6 m below the aim, so nothing is cut.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn wrap(mut a: f32) -> f32 {
    let t = std::f32::consts::PI * 2.0;
    while a > std::f32::consts::PI { a -= t; }
    while a < -std::f32::consts::PI { a += t; }
    a
}

#[test]
fn the_laser_cuts_the_visible_cables() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    // the reported spot and the visible cable's middle (the wire object's world bounds)
    let spot = [-89.71f32, -1.10, 32.49];
    // The muzzle sits 0.48 m below the eye [measured: from.y -1.327 vs eye -0.85], so a beam parallel to
    // the view lands 0.48 m low: the player puts the BEAM on the cable, i.e. the crosshair above it.
    let target = [-89.61f32, -0.485, 30.89];
    let wires0 = {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..120 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        sim.bond.pos = spot;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        sim.bond.pos = spot;
        sim.step_bond(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false, false, false);
        for _ in 0..200 { sim.step_bond(nf_game::player::Input::default(), false, false, false); if sim.gun.weapon == 0x4e { break; } }
        let fb = sim.drones.fuseboxes.iter().find(|f| (f.pos[0] + 89.31).abs() < 2.0).expect("the box");
        fb.alive_wires().count()
    };
    assert!(wires0 > 0, "the box's script registered wires at load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..240 {
        let b = &sim.bond;
        let eye = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        let yaw = d[0].atan2(d[2]);
        let pitch = (d[1] / len).asin() / std::f32::consts::FRAC_PI_2;
        let ye = wrap(yaw - b.yaw).clamp(-0.08, 0.08);
        let pe = (pitch - b.pitch).clamp(-0.08, 0.08);
        sim.step_bond(nf_game::player::Input { turn: ye, look: pe, aiming: true, ..Default::default() }, true, false, false);
    }
    let fb = sim.drones.fuseboxes.iter().find(|f| (f.pos[0] + 89.31).abs() < 2.0).unwrap();
    let wires1 = fb.alive_wires().count();
    assert!(wires1 < wires0,
        "the held laser aimed at the visible cable (world y -0.73..-0.50, 1.5 m above the wire entity's \
         translation) must cut: the contact is the wire object's geometry [M Collide_PickObj 0x1eae80], \
         not the 0.5 sphere at the entity origin; alive wires {wires0} -> {wires1}");
}
