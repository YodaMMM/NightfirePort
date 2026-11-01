//! walkreal probe (level-0c-secret-door-reach-3): the annex-vent chain, the corridor's REAL route
//! candidate. The vent SS #180 `ventmesh` + the blocking poly #184 hang in the corridor's ceiling
//! at (53.8, 5.5, −163.59) with the Q-Laser latch #182 `VentLatch` at (53.80, 5.90, −165.36) —
//! all above the mezzanine deck (5.44..6.03, x 52..56, z −176..−152 [D reach-1 scans]); ch 0x54
//! opens them [D #182 p5 = 0x54]. This probe: stand on the mezzanine, burn the latch through the
//! real held-fire path, watch ch 0x54 + the SS sweep, drop into the corridor (floor 0.90 [D
//! secretreach]) and walk south to the plates #558/#559 (ch 3/ch 2) → the E-arm mux → ch 8 → the
//! door sweeps. The mezzanine REACH itself is the documented stand-in (the port's statics seal
//! every link; live check filed). Diagnostic, --nocapture.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{dxz, wrap_pi, Walker};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The walkreal's held-fire aim loop (the same shape walkreal_0700000c_live uses for the
/// photograph): steer the yaw/pitch onto the target, tap the trigger on-target.
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
            let want_pitch = (dy / dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let input = if on_target {
                Input { aiming: true, ..Default::default() }
            } else {
                Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), aiming: true, ..Default::default() }
            };
            drop(sim);
            w.tick(input, false);
        }
    }
}

#[test]
fn probe_vent_route() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // The stand-in: onto the mezzanine deck (5.44..6.03 [D reach-1]) above the corridor.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [54.0, 5.8, -158.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let under = sim.world.segment_hit([p[0], p[1] + 0.5, p[2]], [p[0], p[1] - 4.0, p[2]])
            .map(|(tt, _)| p[1] + 0.5 - 4.0 * tt);
        eprintln!("[vr] the mezzanine stand: ({:.2},{:.2},{:.2}) grounded {} under {under:?} (want ~5.44..6.03 [D])",
            p[0], p[1], p[2], sim.bond.grounded);
    }
    // Walk to the latch spot (the latch #182 at (53.80, 5.90, −165.36) [D]).
    w.walk_to(walker::Wp::new(54.0, 5.8, -160.8).r(0.8), "the deck south");
    // The deck's internal 0.56 step-down around z −161 holds the walker (the game's own lip-hold
    // [L 2026-10-31 walkway]); cross it with a manual full-speed push (real input, no pokes).
    {
        let start = w.total_ticks;
        let mut jumped = false;
        for _ in 0..400u32 {
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            if p[2] < -164.5 && w.scene.sim.as_ref().unwrap().bond.grounded { break; }
            let desired = (53.8f32 - p[0]).atan2(-165.5f32 - p[2]);
            let err = wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
            let jump = !jumped && w.scene.sim.as_ref().unwrap().bond.grounded && p[2] < -160.4;
            if jump { jumped = true; }
            w.tick(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, jump, ..Default::default() }, false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[vr] the step-down crossed (+{} ticks): ({:.2},{:.2},{:.2}) grounded {}",
            w.total_ticks - start, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded);
    }
    w.walk_to(walker::Wp::new(53.8, 5.8, -164.8).r(0.6), "under the latch");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[vr] at the latch spot ({:.2},{:.2},{:.2}) ch54 {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.drones.channels.on(0x54));
    }
    // The Q-Laser burn through the real held-fire path (the hinge-lock pattern [M Lock_Update
    // 0x1ae3d8 case 0]).
    w.tick(Input { switch_to: Some(0x4e), ..Default::default() }, false);
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
        w.tick(Input::default(), false);
    }
    eprintln!("[vr] laser held = {}", w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e);
    let latch = w.scene.sim.as_ref().unwrap().drones.locks.iter()
        .find(|l| l.out as u32 == 0x54).map(|l| l.pos)
        .expect("the ch-0x54 VentLatch #182 lock [D]");
    eprintln!("[vr] the ch-0x54 lock pos: {latch:?}");
    // The walkreal's hinge-burn pattern: the aim re-converges per tick (turn/look) and the
    // trigger HOLDS via tick_fire [M Lock_Update 0x1ae3d8:82-101].
    let mut burned = false;
    let burn_start = w.total_ticks;
    while w.total_ticks - burn_start <= 900 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.channels.on(0x54) { burned = true; break; }
        }
        let fire_input = {
            let sim = w.scene.sim.as_ref().unwrap();
            let h = sim.bond.head();
            let d = [latch[0] - h[0], latch[1] - h[1], latch[2] - h[2]];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let yaw_err = wrap_pi(d[0].atan2(d[2]) - sim.bond.yaw);
            let pitch_err = (d[1] / l).clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI - sim.bond.pitch;
            Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.08, 0.08), ..Default::default() }
        };
        w.tick_fire(fire_input);
    }
    eprintln!("[vr] ch 0x54 burned = {burned} (+{} ticks)", w.total_ticks - burn_start);
    // The SS sweep: watch the ch-0x54 SS state.
    let mut swept = false;
    for _ in 0..300u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(ss) = sim.drones.sss.list.iter().find(|s| s.channel == 0x54) {
            if ss.state == 0 { swept = true; break; }
        }
    }
    eprintln!("[vr] the vent SS swept open = {swept}");
    // The drop: walk onto the vent hole (the mesh at (53.8, 5.5, −163.59)) and fall through.
    w.walk_to(walker::Wp::new(53.8, 5.8, -163.6).r(0.5), "onto the vent hole");
    let mut landed = false;
    for _ in 0..300u32 {
        w.tick(Input { forward: 0.3, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.pos[1] < 3.0 {
            landed = true;
            let under = sim.world.segment_hit([sim.bond.pos[0], sim.bond.pos[1] + 0.5, sim.bond.pos[2]],
                [sim.bond.pos[0], sim.bond.pos[1] - 4.0, sim.bond.pos[2]])
                .map(|(tt, _)| sim.bond.pos[1] + 0.5 - 4.0 * tt);
            eprintln!("[vr] DROPPED into the corridor at ({:.2},{:.2},{:.2}) under {under:?} (want 0.90 [D secretreach])",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
            break;
        }
    }
    if !landed { eprintln!("[vr] never dropped: {:?}", w.scene.sim.as_ref().unwrap().bond.pos); }
    // The corridor south to the plates.
    if landed {
        w.walk_to(walker::Wp::new(53.5, 1.0, -167.0).r(1.0), "the corridor south 1");
        w.walk_to(walker::Wp::new(53.2, 1.0, -171.0).r(1.0), "the corridor south 2");
        w.walk_to(walker::Wp::new(52.9, 1.0, -173.3).r(0.7), "the plates");
        w.wait(20);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            eprintln!("[vr] at the plates ({:.2},{:.2},{:.2}) ch2 {} ch3 {}",
                p[0], p[1], p[2], sim.drones.channels.on(2), sim.drones.channels.on(3));
        }
        for _ in 0..120 {
            w.tick(Input::default(), true);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(8) { break; }
        }
        eprintln!("[vr] ch8 = {}", w.scene.sim.as_ref().unwrap().drones.channels.on(8));
        for _ in 0..900 {
            w.tick(Input::default(), true);
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(d) = sim.drones.sss.list.iter().find(|s| s.channel == 8) {
                if d.state == 0 { eprintln!("[vr] the door swept open"); break; }
            }
        }
    }
    eprintln!("[vr] done");
}
