//! 07000012's SkyBridge grapple point, through the real input path end to end.
//!
//! The chain [M/D]:
//! - The level's only type-2 third icon, #680 3RD_Person_Icon (-9.79, 1.17, 32.78) [D params
//!   0:2 1:0x3c], sits on the west end of the sky bridge. Its param 1 is the on-channel: it writes
//!   nothing until ch 0x3C is on, then latches enabled [M ThirdIcon_Create 0x197070:27-32,
//!   ThirdIcon_Update 0x196e80:18-20], and while Bond's walk pick meets its sphere it writes 2 into
//!   BLData+0x95F [M 0x196e80:36-52].
//! - The claw fires only with +0x95F == 2 (or in MP) and no bolt in flight (+0x160) [M
//!   Player_WeaponFiring 0x1a5460:515-528]. Its bolt dying on the class-'Q' grapple object calls
//!   `Player_SetGrapplePoint` [M Bullet_Delete 0x12e660:42-55 → 0x195da0], substate 2, phase 1.
//! - `Player_SSGrapple` 0x192860 reels at 0.05/tick up to 1.5 (SP) and hangs under 2.0 [M :91-104];
//!   the hang holds while the fire is held [M :56-62].
//! - ch 0x3C has one writer: the sky bridge's break script (SP #660, script 0x06000530, stream 8
//!   event 0xD near its end [D the level audit]). The touches #670/#781 at the bridge ends raise
//!   ch 0x25, which releases the copter #707; its rounds start the break and wear down the four
//!   holds (p7 = 120 each [M SP_Update 0x1dbda8 tail]); the deck falls and then ch 0x3C comes on.
//!   So the icon arms only once the bridge is gone, and from the west stub the bolt line to the
//!   point over the gap is open. With the bridge still standing, its glass tube is in the way.
//!
//! The walk: from the start lift east of the bridge onto it at the east end (#670), across it to
//! the west stub inside #680's arm. Bond waits there (health held) while the copter breaks the
//! bridge. Then: the claw drawn with the switch input, aimed with turn/look, the fire held.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp, wrap_pi};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const POINT: [f32; 3] = [11.475247, 9.535462, 30.155592]; // GRAPPLEPOINT_SkyBridge1 #130 [D; the port's sphere]
const STUB: Wp = Wp { at: [-9.0, 1.1, 33.0], act: walker::Act::None, arrive: 0.8 };

fn bridge(w: &Walker) -> &nf_game::nis::Player {
    w.scene.sim.as_ref().unwrap().drones.scripts.players.iter().find(|p| p.placement == 660).expect("SP #660 [D]")
}

fn draw_claw(w: &mut Walker) {
    for _ in 0..200 {
        w.tick(Input { switch_to: Some(0x50), ..Default::default() }, false);
        let g = &w.scene.sim.as_ref().unwrap().gun;
        if g.weapon == 0x50 && g.switch_state == 0 { return; }
    }
    panic!("the Q-Claw (0x50) never came up through the switch input");
}

fn aim(w: &mut Walker, at: [f32; 3]) {
    for _ in 0..120 {
        let b = &w.scene.sim.as_ref().unwrap().bond;
        let h = b.head();
        let to = [at[0] - h[0], at[1] - h[1], at[2] - h[2]];
        let horiz = (to[0] * to[0] + to[2] * to[2]).sqrt().max(1e-4);
        let want_yaw = to[0].atan2(to[2]);
        let want_look = (to[1] / horiz).atan() / std::f32::consts::FRAC_PI_2;
        let inp = Input {
            turn: wrap_pi(want_yaw - b.yaw).clamp(-0.08, 0.08),
            look: (want_look - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        w.tick(inp, false);
    }
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

#[test]
fn the_skybridge_point_latches_from_the_west_stub_once_the_bridge_falls() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    for (label, wp) in [
        ("the shaft lobby", Wp::new(3.0, 1.0, 10.0).r(1.2)),
        ("lane north (6, 14)", Wp::new(6.0, 1.0, 14.0).r(1.2)),
        ("the lane gap (9.5, 20.5)", Wp::new(9.5, 1.0, 20.5).r(1.2)),
        ("the NE strip (10, 24)", Wp::new(10.0, 1.0, 24.0).r(1.2)),
        ("the bridge's east end (#670)", Wp::new(11.0, 1.0, 29.0).r(1.0)),
        ("the deck east", Wp::new(6.0, 1.1, 33.0).r(1.0)),
        ("the deck middle", Wp::new(0.0, 1.1, 33.5).r(1.0)),
        ("the west stub", STUB),
    ] {
        assert!(w.try_walk_to(wp, label), "{label}: the leg walks");
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x25), "the bridge-end touch released the copter [D #670/#781 out 0x25]");
        assert!(!sim.drones.channels.on(0x3c), "ch 0x3C is off while the bridge stands [D its one writer is the break's end]");
        assert_ne!(sim.bond.icon_ctx, 2, "sister: the icon does not arm before its channel");
    }
    // Sister: with the bridge standing the claw does not fire here (the gate reads +0x95F).
    draw_claw(&mut w);
    aim(&mut w, POINT);
    for _ in 0..30 { w.tick_fire(Input::default()); }
    assert!(w.scene.sim.as_ref().unwrap().bond.grapple.is_none(), "sister: no latch before the icon arms [M 0x1a5460:515-528]");
    // The copter breaks the bridge; Bond holds the stub (the walker keeps his health up).
    let mut broke = None;
    for t in 0..30000u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x3c) { broke = Some(t); break; }
    }
    let t = broke.unwrap_or_else(|| panic!("the copter never broke the bridge (state {}, damage {})", bridge(&w).state, bridge(&w).damage));
    eprintln!("[skybridge] the break script raised ch 0x3c {t} ticks after Bond reached the stub");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] > 0.5, "Bond still stands on the stub, got {:?}", sim.bond.pos);
        assert_eq!(sim.bond.icon_ctx, 2, "the icon arms once ch 0x3C is on, at {:?}", sim.bond.pos);
    }
    // Draw (the switch input; a no-op when still held), aim, fire.
    draw_claw(&mut w);
    aim(&mut w, POINT);
    let mut latched = None;
    for k in 0..30 {
        w.tick_fire(Input::default());
        if let Some(g) = w.scene.sim.as_ref().unwrap().bond.grapple.as_ref() { latched = Some((k, g.point)); break; }
    }
    let (_, at) = latched.expect("the claw latches the SkyBridge point from the west stub");
    assert!(dist(at, POINT) < 0.01, "it latched GRAPPLEPOINT_SkyBridge1, got {at:?}");
    // The reel: 0.05/tick to the 1.5 cap, then the hang under 2.0 while the fire is held.
    let mut prev = w.scene.sim.as_ref().unwrap().bond.pos;
    let mut hung = false;
    for _ in 0..400 {
        w.tick_fire(Input::default());
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let g = sim.bond.grapple.as_ref().expect("the held fire stays on the rope [M 0x192860:56-62]");
        assert!(dist(p, prev) <= 1.5001, "a reel step past the 1.5 cap: {:.3}", dist(p, prev));
        prev = p;
        if g.phase == 2 { assert!(dist(p, POINT) < 2.5, "the hang sits by the point, got {p:?}"); hung = true; break; }
    }
    assert!(hung, "Bond reels across the gap and hangs at the point");
    for _ in 0..30 {
        w.tick_fire(Input::default());
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grapple.as_ref().is_some_and(|g| g.phase == 2) && dist(sim.bond.pos, POINT) < 2.5,
            "the hang holds while the fire is held, got {:?}", sim.bond.pos);
    }
}
