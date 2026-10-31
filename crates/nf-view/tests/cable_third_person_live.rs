//! The cable's third-person view (07000005): on the wire the game switches to its tracking camera and
//! draws Bond's own body hanging from the cable.
//!
//! [M] `Player_CollWire` 0x197130 sets `SetCamMode(0xB)` with viewer+0x1F0 = 4.0 and hangs the player
//! object 1.0207 below the cable; `Player_PositionCamera` 0x18f5d8 case 0xb draws the body in all
//! views and hides the view model, and `camera_tracking` 0x1ba5e0 places the camera 4.0 from the
//! tracked point (the cable line lowered 1.0) at pitch pi - t*pi/6. [L PCSX2 07000005, the cable at
//! (-23.58, 34.0, 2.65): third person, Bond's hands on the cable - audits/render-1029-cable p-stand.]
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn hang() -> Option<nf_view::scene::Scene> {
    let iso = std::env::var("NF_ISO").ok()?;
    let mut scene = nf_view::load(&iso, "07000005.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        // The jump grab at the cable icons [M Player_HandleJump 0x1916b0:118-147; fix_wire2_live].
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [-23.58, 34.0, 2.65], 0.0);
        for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
        sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
        for _ in 0..250 { sim.step_bond(Input::default(), false, false, false); }
    }
    Some(scene)
}

#[test]
fn hanging_on_the_cable_is_third_person_with_bond_drawn() {
    let Some(scene) = hang() else { return };
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.bond.substate, nf_game::player::SS_WIRE, "Bond hangs on the cable");
    let (cam, _, _, _) = sim.wire_camera().expect("the wire's tracking camera runs in cam mode 0xB");
    let w = &sim.wires[sim.bond.wire.unwrap()];
    // The tracked point is on the cable line lowered 1.0; the camera sits 4.0 from it [M 0x1ba5e0:149-154].
    let off = nf_game::searchlight::spherical(w.radius, w.yaw + std::f32::consts::FRAC_PI_2, w.pitch);
    let (a, b) = ([w.origin[0] + off[0], w.origin[1] + off[1] - 1.0, w.origin[2] + off[2]], [w.origin[0] - off[0], w.origin[1] - off[1] - 1.0, w.origin[2] - off[2]]);
    let tracked = nf_game::wire::Wire::closest_point(a, b, sim.bond.pos);
    // 07000005's cable carries param 0 = 2 [D]: the position comes from ThirdPersonCameras[2] at t
    // (`ThirdCam_GetPos` 0x1bc590 [M 0x1ba5e0:171-179]) - the placed camera spline, not the 4.0 sphere.
    assert_eq!(w.param0, 2, "07000005's cable names ThirdPersonCameras[2] [D placement param 0]");
    let tc = sim.drones.worldobjs.thirdcams.iter().find(|tc| tc.cam_id == w.param0).expect("ThirdCam #2 is placed on 07000005 [D]");
    let d2 = |p: [f32; 3], q: [f32; 3]| ((p[0] - q[0]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
    let t = d2(a, tracked) / d2(a, b);
    let m = tc.get_pos(t).expect("the spline position");
    assert!((cam[0] - m[3][0]).abs() < 1e-4 && (cam[1] - m[3][1]).abs() < 1e-4 && (cam[2] - m[3][2]).abs() < 1e-4,
        "the camera rides ThirdCam #2's spline at t = {t}: {cam:?} vs {:?}", &m[3][..3]);
    assert!(cam[1] > tracked[1], "the camera looks down on Bond [L p-stand]");
    // Bond's body is drawn (posed), and its highest bones - the hands - sit on the cable.
    let ci = scene.bond_body.expect("the single-player body character");
    assert!(!scene.char_hidden(ci, Default::default()), "Bond's body draws in the wire camera");
    let fs = scene.frame_state_for(None, 0.0, Default::default(), true);
    let (_, world, m) = fs.posed[ci].clone().expect("Bond's body is posed by the wire anim");
    let top = world.iter().map(|bm| {
        let p = [bm[3][0], bm[3][1], bm[3][2]];
        p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1] + m[3][1]
    }).fold(f32::MIN, f32::max);
    let cable_y = w.origin[1];
    assert!((top - cable_y).abs() < 0.15, "the hands grip the cable: top bone {top} vs cable {cable_y} [L p-stand]");
}

#[test]
fn on_foot_the_body_stays_hidden_and_the_view_is_first_person() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).unwrap();
    { let sim = scene.sim.as_mut().unwrap(); for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); } }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.wire_camera().is_none(), "no wire camera on foot");
    let ci = scene.bond_body.expect("the body character exists");
    assert!(scene.char_hidden(ci, Default::default()), "the body is hidden in first person");
}

/// Moving along the cable never snaps the body: the root bone draws at the player object (the sim
/// already carries the anims' root motion in Bond's position [M AnimFrameResolve 0x1c35f8]), so a
/// clip switch (turn -> hand-over-hand move) does not jump. Drawing the clip's own root offset on top
/// snapped the body 1.14 in one tick at the switch.
#[test]
fn hand_over_hand_moves_without_snapping() {
    let Some(mut scene) = hang() else { return };
    let ci = scene.bond_body.unwrap();
    let mut prev: Option<[f32; 3]> = None;
    let mut worst = 0.0f32;
    let mut anims = std::collections::BTreeSet::new();
    for _ in 0..200 {
        scene.sim.as_mut().unwrap().step_bond(Input { strafe: 1.0, ..Default::default() }, false, false, false);
        anims.insert(scene.sim.as_ref().unwrap().bond.wire_anim);
        let fs = scene.frame_state_for(None, 0.0, Default::default(), true);
        let (_, world, m) = fs.posed[ci].clone().expect("posed on the cable");
        let p = [world[0][3][0], world[0][3][1], world[0][3][2]];
        let root = [p[0] * m[0][0] + p[1] * m[1][0] + p[2] * m[2][0] + m[3][0], p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1] + m[3][1], p[0] * m[0][2] + p[1] * m[1][2] + p[2] * m[2][2] + m[3][2]];
        if let Some(q) = prev { worst = worst.max(((root[0] - q[0]).powi(2) + (root[1] - q[1]).powi(2) + (root[2] - q[2]).powi(2)).sqrt()); }
        prev = Some(root);
    }
    assert!(anims.len() >= 2, "the ride switches clips (got {anims:x?})");
    assert!(worst < 0.15, "the body never snaps between ticks: worst root step {worst}");
}
