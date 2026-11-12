//! walkreal probe (level-0c-vent-crawl-2): the display-case vent route walked END TO END on real
//! input from the level load — no position writes anywhere: the walkway, the burned hatch (the
//! four hinge locks #139-142), the duct, off the sill lip into the shaft (plate #274 fires
//! mid-fall, objective 1), the crouch, the crawl SOUTH out under the sill into the hall behind
//! the storage crates, the stand, and on toward the next objective (the staircase descent and the
//! camera photograph, objective 2).
//!
//! Every end-of-leg assert pins the live numbers [L PCSX2 fresh boot 0700000c, 2026-11-11]:
//! - the shaft floor standing spot (5.6, 12.95, -111.2) is the start of the exit legs;
//! - standing south stops at z -111.395, north (standing and crouched) at z -110.94, east/west
//!   at x 6.187 / 5.332 (the shaft's wedge map);
//! - the crawl south out ends at (5.6, 11.882, -113.54) — asserted at 0.1 tolerance (the port's
//!   from-placement crawl ends at (5.6, 11.924, -113.540), inside it).
//! The game's own route: [D probes] the walkway floor 14.61; the hinges #139-142 → AND #505 →
//! ch 18 → SS #138/#179 sweep; plate #274 (kind 2, out 0x1b) at (5.79, 14.09, -111.10) fires
//! mid-fall; the crawl out passes ventblocker_01 #457 (z -112.363, surface 0x40, normal south —
//! a body from the north is behind it and passes [M Dist2Tri 0x109770 code 99;
//! Intersect_CylGeom 0x109da0 skips the face on 99]); the crouch is the duck action with the
//! 60-tick lockout [M Player_SSCrouch 0x1924c0:84-105, Player_ChangeSubState 0x190630:47-48].
//! The post-crawl legs replay walkreal0c's own route to objective 2, so the exit point the live
//! game reaches is proven to connect to the objective route. No draw changes.
#[path = "common/walkreal0c.rs"]
mod walker;

use walker::{wrap_pi, Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Fire the held weapon at a static point through the real trigger path — copied from
/// common/walkreal0c.rs's local `shoot_at` (kept private there) so this file adds no shared
/// helper change. Same aim loop, same tick shape.
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks { return false; }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            if (dx * dx + dy * dy + dz * dz).sqrt() > range { return false; }
            let desired = dx.atan2(dz);
            let err = wrap_pi(desired - sim.bond.yaw);
            let dxz = ((bp[0] - target[0]) * (bp[0] - target[0]) + (bp[2] - target[2]) * (bp[2] - target[2])).sqrt();
            let want_pitch = (dy / dxz.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
            let input = if trigger {
                Input { aiming: true, ..Default::default() }
            } else {
                Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), aiming: true, ..Default::default() }
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        }
        w.total_ticks += 1;
        w.after_step();
    }
}

/// Hold one compass direction standing for `ticks` and report where the body stopped (the
/// wedge map [L 2026-11-11]: S z -111.395, N z -110.94, E x 6.187, W x 5.332).
fn hold_direction_standing(w: &mut Walker, dir: [f32; 2], ticks: u32, label: &str) -> [f32; 3] {
    let desired = dir[0].atan2(dir[1]);
    for _ in 0..ticks {
        let err = wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let input = Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[vent-exit] hold {label}: stopped at ({:.3}, {:.3}, {:.3})", p[0], p[1], p[2]);
    p
}

/// Hold one compass direction standing at an analog speed until a position gate opens (real
/// held input, no writes; used to walk the body onto a measured start spot the way a player
/// would).
fn hold_direction_gated(w: &mut Walker, dir: [f32; 2], speed: f32, max_ticks: u32, done: impl Fn(&[f32; 3]) -> bool, label: &str) -> [f32; 3] {
    let desired = dir[0].atan2(dir[1]);
    let mut p = w.scene.sim.as_ref().unwrap().bond.pos;
    for _ in 0..max_ticks {
        if done(&p) { break; }
        let err = wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let input = Input { turn: err.clamp(-0.08, 0.08), forward: speed, ..Default::default() };
        w.tick(input, false);
        p = w.scene.sim.as_ref().unwrap().bond.pos;
    }
    eprintln!("[vent-exit] hold {label} (gated): at ({:.3}, {:.3}, {:.3})", p[0], p[1], p[2]);
    p
}

#[test]
fn the_vent_route_walks_out_through_the_crawl_on_real_input() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 8.98).abs() < 2.0 && (p[1] - 15.07).abs() < 2.0 && (p[2] + 149.65).abs() < 2.0,
            "the walkway start #1 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);

    // The walkway north (the walkreal0c legs verbatim — real input only).
    w.walk_to(Wp::new(9.0, 15.7, -140.0).r(1.5), "the walkway north 1");
    w.walk_to(Wp::new(12.0, 15.7, -118.0).r(1.5), "the walkway north 2 (east of the raised block)");
    w.walk_to(Wp::new(7.61, 15.71, -113.39).r(0.8), "the inner balcony repro spot [L 991]");
    // The laser out through the real switch input.
    w.tick(Input { switch_to: Some(0x4e), ..Default::default() }, false);
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Laser equips [M 0x1a7ef8]");
    // The burned hatch: the four hinge locks under held laser fire [M 0x1ae3d8:82-101].
    for out in [14u32, 15, 16, 17] {
        let target = w.scene.sim.as_ref().unwrap().drones.locks.iter()
            .find(|l| l.out as u32 == out).map(|l| l.pos)
            .unwrap_or_else(|| panic!("the out-{out} hinge lock exists [D #139-142]"));
        let mut burned = false;
        let start = w.total_ticks;
        while w.total_ticks - start <= 500 {
            let fire_input = {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(out) { burned = true; break; }
                let h = sim.bond.head();
                let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let yaw_err = wrap_pi(d[0].atan2(d[2]) - sim.bond.yaw);
                let pitch_err = (d[1] / l).clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI - sim.bond.pitch;
                Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.08, 0.08), ..Default::default() }
            };
            w.tick_fire(fire_input);
        }
        assert!(burned, "hinge out {out} never burned under the held laser fire [M 0x1ae3d8:82-101]");
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(18), "the AND mux #505 raised ch 18 [D #505]");
    // The SSs sweep after ch 18; completion = the y-16.3 ray crossing the open vent
    // [M the SS end poses].
    let mut open = false;
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().world.segment_hit([5.78, 16.3, -111.7], [5.78, 16.3, -113.0]).is_none() { open = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(open, "the open vent never let the y-16.3 ray through [M the SS end poses]");
    // Through the open vent and off the sill lip into the shaft — the fall crosses plate #274.
    w.walk_to(Wp::new(5.78, 15.7, -112.3).r(0.6), "through the open vent");
    let mut fired_at = None;
    let start = w.total_ticks;
    let landed = loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let (p, grounded) = (sim.bond.pos, sim.bond.grounded);
        if grounded && fired_at.is_some() { break true; }
        if w.total_ticks - start > 400 { break false; }
        if !grounded && fired_at.is_none() && sim.drones.channels.on(0x1b) {
            fired_at = Some(w.total_ticks - start);
        }
        let desired = (5.78f32 - p[0]).atan2(-110.8f32 - p[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        let input = Input {
            turn: err.clamp(-0.08, 0.08),
            forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        w.tick(input, false);
    };
    // Leg end — the shaft floor [L 2026-11-11]: the standing spot (5.6, 12.95, -111.2). The two
    // live drop measurements (2026-10-26 hatch-2 (5.42, 12.8773, -111.38); 2026-11-11 (5.6,
    // 12.95, -111.2)) differ by 0.18 in x/z on the game's own floor, so the landing pin keeps
    // 0.35 of approach slack in x/z and 0.15 in y; the crawl exit below is the 0.1 pin.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(landed && fired_at.is_some() && sim.bond.grounded,
            "the vent drop must fire plate #274 mid-fall (ch 0x1b [D #274; L hatch-2]) and land, got pos {p:?} fired {fired_at:?}");
        assert!((p[0] - 5.6).abs() < 0.35 && (p[1] - 12.95).abs() < 0.15 && (p[2] + 111.2).abs() < 0.35,
            "the drop must land standing on the shaft floor (5.6, 12.95, -111.2 [L 2026-11-11]), got {p:?}");
        assert_eq!(sim.drones.mission.states[11][0], 4, "objective 1 completes on the #274 drop [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[11][1], 3, "ch 0x1b reveals objective 2 [L record 11 obj 1]");
    }
    eprintln!("[vent-exit] landed on the shaft floor at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);

    // The wedge map from the landed spot, standing [L 2026-11-11]: S z -111.395, N z -110.94,
    // E x 6.187, W x 5.332. The stops are wall planes, so they hold from any inside start.
    let p = hold_direction_standing(&mut w, [0.0, -1.0], 90, "south (standing)");
    assert!((p[2] + 111.395).abs() < 0.1, "standing south stops at z -111.395 [L 2026-11-11], got {:.3}", p[2]);
    let p = hold_direction_standing(&mut w, [0.0, 1.0], 90, "north (standing)");
    assert!((p[2] + 110.94).abs() < 0.1, "standing north stops at z -110.94 [L 2026-11-11], got {:.3}", p[2]);
    let p = hold_direction_standing(&mut w, [1.0, 0.0], 90, "east (standing)");
    assert!((p[0] - 6.187).abs() < 0.1, "east stops at x 6.187 [L 2026-11-11], got {:.3}", p[0]);
    let p = hold_direction_standing(&mut w, [-1.0, 0.0], 90, "west (standing)");
    assert!((p[0] - 5.332).abs() < 0.1, "west stops at x 5.332 [L 2026-11-11], got {:.3}", p[0]);
    // Back to the live crawl start the way a player walks there: south to the sill stop, then
    // east to the floor's middle x — full speed to 5.45, then a slow stick that stops just west
    // of the middle (real held input, gated on the position, no writes; the port's crawl is
    // laterally divergent east of x 5.6, so the approach parks the body just west of it).
    let p = hold_direction_gated(&mut w, [0.0, -1.0], 1.0, 120, |p: &[f32; 3]| p[2] <= -111.35, "south to the sill stop");
    assert!(p[2] <= -111.35, "the south hold never reached the sill stop, got {:.3}", p[2]);
    let p = hold_direction_gated(&mut w, [1.0, 0.0], 1.0, 120, |p: &[f32; 3]| p[0] >= 5.45, "east fast");
    let p = hold_direction_gated(&mut w, [1.0, 0.0], 0.2, 200, |p: &[f32; 3]| p[0] >= 5.575, "east slow to the crawl start");
    assert!(p[0] >= 5.57 && p[0] <= 5.6, "the crawl start must park just west of the floor's middle x 5.6 [L 2026-11-11], got {:.3}", p[0]);
    eprintln!("[vent-exit] at the crawl start ({:.3}, {:.3}, {:.3})", p[0], p[1], p[2]);

    // The crawl out: the only way out is crouched crawling SOUTH under the sill [L 2026-11-11].
    // Crouch once (the toggle; the 60-tick lockout eats the repeat [M 0x1924c0:84-105,
    // 0x190630:47-48]), hold south, stall-detect the stop. Live end: (5.6, 11.882, -113.54).
    for _ in 0..3 {
        w.tick(Input { crouch: true, ..Default::default() }, false);
    }
    assert!(w.scene.sim.as_ref().unwrap().bond.crouched, "the crouch toggle crouched [M 0x1924c0]");
    let mut end = None;
    let mut last_z = f32::MAX;
    let mut stall = 0u32;
    for t in 0..500 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if p[2] < -113.0 && sim.bond.grounded {
            if (p[2] - last_z).abs() < 0.003 { stall += 1; if stall > 25 { end = Some((t, p)); break; } }
            else { stall = 0; }
            last_z = p[2];
        }
        let err = wrap_pi(std::f32::consts::PI - sim.bond.yaw);
        let input = Input {
            turn: err.clamp(-0.08, 0.08),
            forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        w.tick(input, false);
    }
    // Leg end — the crawl exit, the 0.1 pin [L 2026-11-11] (port's from-placement crawl
    // (5.6, 11.924, -113.540) sits inside it).
    {
        let p = end.map(|(_, p)| p).unwrap_or_else(|| w.scene.sim.as_ref().unwrap().bond.pos);
        assert!(end.is_some(), "the crawl never stalled out in the hall, pos {p:?}");
        assert!((p[0] - 5.6).abs() < 0.1 && (p[1] - 11.882).abs() < 0.1 && (p[2] + 113.54).abs() < 0.1,
            "the crawl south out must end at (5.6, 11.882, -113.54) in the hall behind the storage \
             crates (0.1 [L 2026-11-11]), got ({:.3}, {:.3}, {:.3})", p[0], p[1], p[2]);
        eprintln!("[vent-exit] the crawl out ended at ({:.3}, {:.3}, {:.3}) after {} crawl ticks",
            p[0], p[1], p[2], end.map(|(t, _)| t).unwrap_or(0));
    }
    // Stand up (the fresh press + clear 0.95 LOS [M 0x1924c0:84-105]).
    for _ in 0..3 { w.tick(Input { crouch: true, ..Default::default() }, false); }
    for _ in 0..40 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.bond.crouched && sim.bond.grounded, "the stand-up toggle failed after the crawl [M 0x1924c0]");
        eprintln!("[vent-exit] stood up at {:?}", sim.bond.pos);
    }

    // On toward the next objective (objective 2, the photograph) — walkreal0c's own route legs
    // from the crawl exit: the hall east, the upper east lane, the staircase descent, the
    // display eye.
    w.walk_to(Wp::new(8.0, 12.4, -113.2).r(1.0), "the hall east (south of the x 6..7 pillars)");
    w.walk_to(Wp::new(13.5, 12.4, -115.0).r(1.2), "the upper east lane");
    w.walk_to(Wp::new(13.2, 12.4, -124.0).r(1.0), "the lane south to the stair head");
    w.walk_to(Wp::new(13.0, 12.4, -128.2).r(0.8), "the stair approach");
    w.walk_to(Wp::new(11.2, 12.35, -128.1).r(0.6), "west onto the stair landing");
    for (x, z, label) in [
        (11.2f32, -126.5f32, "the ramp 1"),
        (11.2, -125.0, "the ramp 2"),
        (11.2, -123.5, "the ramp 3"),
        (11.3, -122.0, "the ramp 4"),
        (11.4, -120.8, "the ramp 5"),
    ] {
        w.walk_to(Wp::new(x, 7.0, z).r(0.9), label);
    }
    w.walk_to(Wp::new(12.2, 7.4, -120.3).r(0.7), "the stair bottom east");
    w.walk_to(Wp::new(13.2, 7.4, -120.0).r(0.8), "onto the ledge");
    w.walk_to(Wp::new(13.2, 7.4, -116.0).r(0.9), "the ledge north 1");
    w.walk_to(Wp::new(13.2, 7.4, -113.4).r(0.8), "the ledge north 2");
    w.walk_to(Wp::new(10.5, 7.4, -113.4).r(0.9), "the display ground west");
    w.walk_to(Wp::new(5.4, 7.35, -113.0).r(0.7), "the display eye");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 7.41).abs() < 0.5,
            "Bond stands on the display ground at the eye spot (6.27 floor [D probe rays]), got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
    }
    eprintln!("[vent-exit] the staircase descent walked — the display eye at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    // The photograph through the real camera switch and the real trigger path [M 0x1bc668:14-24].
    let subject = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1c)
            .map(|s| (s.lo, s.hi)).expect("the jetpack subjects [D #414/#454]")
    };
    for _ in 0..400 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.weapon == 0x54 { break; }
        let press = sim.gun.switch_request != 0x54;
        w.tick(Input { switch_to: press.then_some(0x54), ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x54, "the camera is held [M 0x1a7ef8]");
    let sc = [(subject.0[0] + subject.1[0]) / 2.0, (subject.0[1] + subject.1[1]) / 2.0, (subject.0[2] + subject.1[2]) / 2.0];
    let photographed = shoot_at(&mut w, sc, 30.0, 900, &|sim| sim.drones.channels.on(0x1c));
    assert!(photographed, "the photograph never wrote ch 0x1c [M 0x1bc668:14-24]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..2 { sim.step_bond(Input::default(), false, false, false); }
        assert_eq!(sim.drones.mission.states[11][1], 4, "objective 2 completes [M 0x1827d8:88-99]");
    }
    eprintln!("[vent-exit] objective 2 complete — the vent route walked end to end on real input");
}
