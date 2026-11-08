//! The store-room vent on 0700000a: the lid SP #685, the context-2 plate #786, the grapple point
//! #785 and the crawl to the security room through plates #752-754 into the grate SS #663, all on
//! real input. What each test pins [M]:
//! - `Bullet_Delete` 0x12e660:50-55: the latch is the bolt dying ON the class-0x51 object, so a
//!   solid in the way (the closed lid) takes the bolt first.
//! - `Player_SSGrapple` 0x192860:49 runs `Player_Move`, so the hang turns; `Player_Aiming`
//!   0x1a1d28 runs for substates up to 7, so the look works.
//! - `Trigger_Update` 0x1b1a78 case 4: plates 0x7e, 0x7f, 0x80 in order raise ch 0x67.

#[path = "common/walker_b2.rs"]
mod walker;
use nf_game::player::Input;
use walker::{wrap_pi, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// A warm 0700000a (the campaign's entry; the kit carries the claw 0x50 [M Player_InitWeapon 0x1a7ef8]).
fn load() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load cold");
    let save = nf_view::warm_exit_save(cold.sim.as_ref().unwrap());
    Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load warm"))
}

fn stand(w: &mut Walker, pos: [f32; 3]) {
    let sim = w.scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, pos, 0.0);
    w.wait(20);
}

/// The turn/look that puts the view on `at`, and whether it is already there.
fn aim(w: &Walker, at: [f32; 3]) -> (Input, bool) {
    let b = &w.scene.sim.as_ref().unwrap().bond;
    let (dx, dz) = (at[0] - b.pos[0], at[2] - b.pos[2]);
    let eye = b.pos[1] + b.eye - b.crouch_drop;
    let e = wrap_pi(dx.atan2(dz) - b.yaw);
    let p = ((at[1] - eye) / (dx * dx + dz * dz).sqrt().max(0.01)).atan() / std::f32::consts::FRAC_PI_2 - b.pitch;
    (Input { turn: e.clamp(-0.08, 0.08), look: p.clamp(-0.08, 0.08), ..Default::default() }, e.abs() < 0.05 && p.abs() < 0.03)
}

fn claw_out(w: &mut Walker) {
    for _ in 0..40 { w.tick(Input { switch_to: Some(0x50), ..Default::default() }, false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x50, "the Q-Claw is in hand");
}

fn point(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().drones.grapple_points[0].pos }

/// Aim at the grapple point and hold the trigger; true once Bond is on the rope.
fn fire_at_point(w: &mut Walker) -> bool {
    let pt = point(w);
    for _ in 0..200 {
        let (inp, ok) = aim(w, pt);
        if ok { w.tick_fire(inp); } else { w.tick(inp, false); }
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { return true; }
    }
    false
}

/// E on the lid from under it (its art sphere (−11.86, 3.93, −6.74) r 1.03 [D]).
fn open_lid(w: &mut Walker) {
    let lid = [-11.86f32, 3.93, -6.74];
    for i in 0..200 {
        let (inp, ok) = aim(w, lid);
        w.tick(inp, ok && i % 8 == 0);
        if w.scene.sim.as_ref().unwrap().drones.scripts.players.iter().any(|p| p.placement == 685 && (p.activated || p.state != 1)) {
            w.wait(90);
            return;
        }
    }
    panic!("the lid never took the use key");
}

/// The room's floor under the plate, inside `ThirdIcon` #786's reach.
const UNDER_LID: [f32; 3] = [-11.9, 1.2, -7.2];

#[test]
fn the_closed_lid_stops_the_bolt() {
    let mut w = load();
    w.wait(30);
    stand(&mut w, UNDER_LID);
    claw_out(&mut w);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.icon_ctx, 2, "the context-2 plate is live here");
    assert!(!fire_at_point(&mut w), "the point is above the closed lid: the bolt dies on it, no latch");
}

/// The whole vent: lid, claw, hang, turn, crawl, grate, drop, and the ch 0x67 chain.
#[test]
fn the_vent_route_runs_from_the_store_room_to_the_security_room() {
    let mut w = load();
    w.wait(30);
    stand(&mut w, UNDER_LID);
    open_lid(&mut w);
    claw_out(&mut w);
    assert!(fire_at_point(&mut w), "with the lid open the bolt reaches the point from the plate");
    for _ in 0..200 {
        w.tick_fire(Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.as_ref().is_some_and(|g| g.phase == 2) { break; }
    }
    assert!(w.scene.sim.as_ref().unwrap().bond.grapple.as_ref().is_some_and(|g| g.phase == 2), "the reel reaches the hang");
    // The hang: the view moves. From the lid side the duct runs +z, so turn onto it and level off.
    let yaw0 = w.scene.sim.as_ref().unwrap().bond.yaw;
    w.tick_fire(Input { turn: 0.05, look: -0.05, ..Default::default() });
    assert!((w.scene.sim.as_ref().unwrap().bond.yaw - yaw0 - 0.05).abs() < 1e-4, "the stick turns Bond on the rope [M 0x192860:49]");
    for _ in 0..60 {
        let (yaw, pitch) = { let b = &w.scene.sim.as_ref().unwrap().bond; (b.yaw, b.pitch) };
        w.tick_fire(Input { turn: wrap_pi(-yaw).clamp(-0.08, 0.08), look: (-pitch).clamp(-0.08, 0.08), ..Default::default() });
    }
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().bond.pos[2] > -5.2 { break; }
        w.tick_fire(Input { forward: 1.0, ..Default::default() });
    }
    // Let go over the duct mouth and walk on.
    for _ in 0..60 { w.tick(Input { forward: 1.0, ..Default::default() }, false); }
    assert!(w.scene.sim.as_ref().unwrap().bond.pos[1] > 4.5, "Bond lands in the duct, got {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    for t in 0..400u32 {
        if w.scene.sim.as_ref().unwrap().bond.crouched { break; }
        w.tick(Input { crouch: t % 70 == 0, ..Default::default() }, false);
    }
    for (p, l) in [([-11.9f32, 4.3, -3.0], "the duct north"), ([-11.9, 4.3, 0.5], "the bend"), ([-8.0, 4.8, 1.1], "the duct east"), ([-5.6, 4.8, 1.1], "the grate")] {
        assert!(w.try_walk_to(Wp::new(p[0], p[1], p[2]).r(0.7), l), "{l}");
    }
    // E the grate SS #663.
    let g = [-4.6f32, 4.6, 0.4];
    for i in 0..200 {
        let (inp, ok) = aim(&w, g);
        w.tick(inp, ok && i % 8 == 0);
        if w.scene.sim.as_ref().unwrap().drones.sss.list.iter().any(|s| s.placement == 663 && s.state != 0x40) { break; }
    }
    w.wait(120);
    assert!(w.try_walk_to(Wp::new(-4.7, 4.8, 1.1).r(0.3), "over the opening"));
    for _ in 0..400 {
        if w.scene.sim.as_ref().unwrap().bond.pos[1] < 1.5 { break; }
        w.wait(1);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.bond.pos[1] < 1.5, "Bond drops into the security room, got {:?}", sim.bond.pos);
    assert!(sim.drones.channels.on(0x67), "the ordered plates raise ch 0x67 [D #755; M 0x1b1a78 case 4]");
}
