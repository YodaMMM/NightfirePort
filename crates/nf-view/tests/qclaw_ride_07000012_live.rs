//! level-12-qclaw-ride: the 07000012 SkyBridge Q-Claw crossing through the real input path.
//!
//! GRAPPLEPOINT_SkyBridge1 #130 (11.49, 9.23, 30.14 [D 07000012-audit §0x3a]) — the port's
//! object sphere resolves it to (11.475, 9.535, 30.156) r 0.899. The level's south-east office
//! strip is ROOFED at y ~2.95 (the deck's underside): every office-floor aim ray to the point
//! is blocked (qclaw_probe12's grid — 0 clear stances on the floor, 0 from the tower2 route
//! legs), and the point hangs over the skybridge deck band: the 3.25 deck (x 4..13, z 29..36)
//! and its 4.58 east parapet. The route WALKS to the strip below the point ((11.77, 1.04,
//! 27.89) with every leg green — probe12d), but the deck entry is not walked here (the open
//! item, the report); the stance stand-in below is the game's own crossing stance class.
//!
//! The ride pins [M Player_SSGrapple 0x192860]: fire with the claw held latches the aim ray's
//! point [M Bullet_Delete 0x12e660:50-55 → Player_SetGrapplePoint 0x195da0 — the port's
//! scene.rs latch]; the reel accumulates 0.05/tick capped at 1.5 single-player [M
//! 0x192860:96-104] and hangs at distance < 2.0 [M 0x192860:91-94]; the hang holds while the
//! fire is HELD and the RELEASE detaches to substate 0 [M 0x192860:56-66]; the drop lands Bond
//! on the deck below the hang (the game's release carries the zeroed +0x10 wish velocity —
//! the straight drop, the wire-grapple atlas §the grapple) and he walks on.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp, wrap_pi};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const POINT: [f32; 3] = [11.475247, 9.535462, 30.155592]; // [D #130; the port's sphere resolution]

/// The ride: select the claw, aim, fire, reel, hang, release, land, walk on.
#[test]
fn the_qclaw_ride_crosses_the_skybridge() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    // The route to the point: the tower2 legs to the shaft lobby, then the north lane through
    // the z 18.5..21 wall gap (x 8..11) into the strip below the deck — every leg walked
    // (probe12d's run: all ok, no stand-ins).
    for (label, wp) in [
        ("the shaft lobby", Wp::new(3.0, 1.0, 10.0).r(1.2)),
        ("lane north (6, 14)", Wp::new(6.0, 1.0, 14.0).r(1.2)),
        ("the lane gap (9.5, 20.5)", Wp::new(9.5, 1.0, 20.5).r(1.2)),
        ("the NE strip (10, 24)", Wp::new(10.0, 1.0, 24.0).r(1.2)),
        ("the strip below the point (11.5, 27.5)", Wp::new(11.5, 1.0, 27.5).r(1.2)),
    ] {
        assert!(w.try_walk_to(wp, label), "{label}: the route leg walks [D the level's floor]");
    }
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!((p[0] - 11.5).abs() < 1.5 && (p[2] - 27.5).abs() < 1.8,
            "the route reaches the strip below the grapple point, got {p:?}");
    }
    // The crossing stance: the deck's east parapet at (11, 28), surface 4.58 — the game's deck
    // entry is the open item (the report); the stand-in reaches the stance, everything after it
    // is the real input path.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [11.0, 4.63, 28.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        assert!(b.grounded && b.pos[1] > 4.4 && b.pos[1] < 5.8,
            "Bond stands on the parapet stance, got {:?} grounded {}", b.pos, b.grounded);
        assert!(sim.drones.inventory.owned.get(0x50).copied().unwrap_or(false),
            "the fresh 07000012 kit owns the Grapple 0x50 [M 0x1a7ef8 case 0x7000012 -> the fresh_11 kit]");
    }
    // Select the claw through the real weapon-switch input.
    let mut have = false;
    for _ in 0..120 {
        w.tick(Input { switch_to: Some(0x50), ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x50 { have = true; break; }
    }
    assert!(have, "the Q-Claw (0x50) is held [L gadget is_grapple: 0x50/0x51; +0x70 & 0x40]");
    // Aim at the point with the real look inputs (turn/look, the walkreal-14 pattern).
    let want_yaw = (POINT[0] - 11.0).atan2(POINT[2] - 28.0);
    let head_y = w.scene.sim.as_ref().unwrap().bond.pos[1] + 0.5;
    let to = [POINT[0] - 11.0, POINT[1] - head_y, POINT[2] - 28.0];
    let horiz = (to[0] * to[0] + to[2] * to[2]).sqrt().max(1e-4);
    let want_look = (to[1] / horiz).atan() / std::f32::consts::FRAC_PI_2;
    for _ in 0..120 {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let inp = Input {
            turn: wrap_pi(want_yaw - b.yaw).clamp(-0.08, 0.08),
            look: (want_look - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
    // Fire to latch [M Player_SetGrapplePoint 0x195da0 — the port's scene.rs latch gate].
    let mut latched = false;
    let mut reel_speeds: Vec<f32> = Vec::new();
    let mut prev = w.scene.sim.as_ref().unwrap().bond.pos;
    for _ in 0..30 {
        w.tick_fire(Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { latched = true; break; }
    }
    assert!(latched, "the Q-Claw fire latches the SkyBridge point [M Bullet_Delete 0x12e660:50-55; D #130]");
    // The reel: pull accumulates 0.05/tick to the 1.5 SP cap [M 0x192860:96-104]; every per-tick
    // displacement toward the point is <= the cap.
    let mut hung = false;
    for _ in 0..400 {
        w.tick_fire(Input::default());
        let sim = w.scene.sim.as_ref().unwrap();
        let (b, p) = (&sim.bond, sim.bond.pos);
        if let Some(g) = &b.grapple {
            if g.phase == 2 {
                let d = ((p[0] - g.point[0]).powi(2) + (p[1] - g.point[1]).powi(2) + (p[2] - g.point[2]).powi(2)).sqrt();
                assert!(d < 2.5, "the hang sits within 2.0 of the point [M 0x192860:91-94], got {d:.2}");
                hung = true;
                break;
            }
            let step = ((p[0] - prev[0]).powi(2) + (p[1] - prev[1]).powi(2) + (p[2] - prev[2]).powi(2)).sqrt();
            reel_speeds.push(step);
        }
        prev = p;
    }
    assert!(hung, "the reel reaches the phase-2 hang [M 0x192860:91-94]");
    if let Some(&m) = reel_speeds.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
        assert!(m <= 1.5001, "the per-tick pull step never exceeds the 1.5 SP cap [M 0x192860:96-104], got {m:.3}");
    }
    // The hang holds while the fire is HELD [M 0x192860:56-66].
    for _ in 0..10 {
        w.tick_fire(Input::default());
        assert!(w.scene.sim.as_ref().unwrap().bond.grapple.is_some(), "held fire stays in the hang [M 0x192860:56-62]");
    }
    // The RELEASE detaches [M 0x192860:64-66] and Bond drops onto the deck below the hang.
    for _ in 0..400 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grapple.is_none() && sim.bond.grounded { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        assert!(b.grapple.is_none() && b.substate == nf_game::player::SS_WALK,
            "the release detaches to the walk substate [M 0x192860:64-66]");
        assert!(b.grounded && b.pos[1] > 3.9 && b.pos[1] < 5.0,
            "the drop lands Bond on the 3.25 deck below the hang (the game's straight drop — the wire-grapple atlas), got {:?} grounded {}",
            b.pos, b.grounded);
        assert!(b.pos[2] > 28.4 && b.pos[2] < 31.4 && b.pos[0] > 9.4 && b.pos[0] < 13.6,
            "the landing is the deck under the point, got {:?}", b.pos);
    }
    // He can walk on from there: the deck runs west-north (x 4..13, z 29..36 [D probe grid]).
    assert!(w.try_walk_to(Wp::new(6.0, 3.25, 33.0).r(1.2), "walk on across the deck"),
        "Bond walks on across the deck from the landing");
    // Sister: aiming the opposite way from the stance latches nothing.
    let mut w2 = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w2.wait_enabled(3000, "the intro");
    {
        let sim = w2.scene.sim.as_mut().unwrap();
        sim.bond.pos = [11.0, 4.63, 28.0];
        sim.bond.vel = [0.0; 3];
    }
    w2.wait(20);
    for _ in 0..40 { w2.tick(Input { switch_to: Some(0x50), ..Default::default() }, false); }
    for _ in 0..60 {
        let b = &w2.scene.sim.as_ref().unwrap().bond;
        w2.tick(Input { turn: wrap_pi(want_yaw + std::f32::consts::PI - b.yaw).clamp(-0.08, 0.08), ..Default::default() }, false);
    }
    for _ in 0..30 { w2.tick_fire(Input::default()); }
    assert!(w2.scene.sim.as_ref().unwrap().bond.grapple.is_none(),
        "sister: aiming away from the point latches nothing [M the pick needs the ray on the sphere]");
}
