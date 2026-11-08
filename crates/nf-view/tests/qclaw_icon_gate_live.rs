//! The Q-Claw's fire gate [M Player_WeaponFiring 0x1a5460:515-528]: on the fire-held arm a 0x50/0x51
//! record RETURNS before `Player_RoundToFire` unless the player's third-icon context byte
//! `BLData+0x95F` reads 2 (a type-2 icon's arm) or the match is multiplayer (`MPSettings+0x180`).
//! 07000014 carries one type-2 icon on the trench floor under each grapple point [D icon census:
//! (45.27, 0.41, 62.94) under GRAPPLEPOINT_SkyBridge (45.29, 5.10, 62.94)]. Fails on the old code,
//! which latched from anywhere the bolt ray reached.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const POINT: [f32; 3] = [45.2888, 5.0961, 62.9362]; // [D placement; the port's resolution]

/// Stand Bond at `at`, draw the claw, aim the head ray at the SkyBridge point and hold fire.
/// Returns (the context byte before the press, latched).
fn press_at(at: [f32; 3]) -> (u8, bool) {
    let mut scene = nf_view::load(&iso(), "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    sim.bond.pos = at;
    sim.bond.pre_collide = at;
    sim.bond.vel = [0.0; 3];
    sim.bond.crouched = false;
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    for _ in 0..120 {
        sim.step_bond(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false, false, false);
        if sim.gun.weapon == 0x50 && sim.gun.switch_state == 0 { break; }
    }
    assert_eq!(sim.gun.weapon, 0x50, "the Q-Claw is drawn");
    let h = sim.bond.head();
    let (dx, dy, dz) = (POINT[0] - h[0], POINT[1] - h[1], POINT[2] - h[2]);
    sim.bond.yaw = dx.atan2(dz);
    sim.bond.pitch = dy.atan2((dx * dx + dz * dz).sqrt()) / std::f32::consts::FRAC_PI_2;
    sim.step_bond(Default::default(), false, false, false);
    let ctx = sim.bond.icon_ctx;
    let mut latched = false;
    for _ in 0..20 {
        sim.step_bond(nf_game::player::Input { fire: true, ..Default::default() }, true, false, false);
        if sim.bond.grapple.is_some() { latched = true; break; }
    }
    (ctx, latched)
}

#[test]
fn the_claw_fires_from_the_icon_arm() {
    // The trench floor beside the icon (1.7 u off its centre, inside reach 1.28·1.5 + 0.55).
    let (ctx, latched) = press_at([44.37, 0.45, 62.84]); // between the grating's slats: the bolt clears them [the walled-pick stance scan]
    assert_eq!(ctx, 2, "the trench floor under the SkyBridge point is the type-2 icon's arm [D]");
    assert!(latched, "the claw latches the SkyBridge point from the icon's arm [M 0x1a5460:515-528]");
}

#[test]
fn the_claw_does_not_fire_outside_the_icon_arm() {
    // The corridor floor 3.5 u east of the icon: the bolt ray still reaches the point, but no icon
    // arms here, so the press returns before the pull.
    let (ctx, latched) = press_at([48.8, 0.45, 62.8]);
    assert_eq!(ctx, 0xff, "no third icon arms on the corridor floor");
    assert!(!latched, "outside a type-2 icon's arm the claw does not fire [M 0x1a5460:515-528]");
}
