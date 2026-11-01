//! probe3: the 0700000d crane-bay vertical profiles + the nav web, printed only
//! (opt-in NF_ISO, --ignored). Marching ray-down that records EVERY surface from y 24
//! to y 0, so roofs, crane rails, the beam and the real floors separate. No assertions.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_warm() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

/// All surfaces down a vertical line: march a 2 m ray from y 24, restart 0.05 below each hit.
fn column(sim: &nf_view::scene::Sim, x: f32, z: f32) -> Vec<f32> {
    let mut ys = Vec::new();
    let mut top = 24.0f32;
    for _ in 0..14 {
        if top <= 0.05 { break; }
        match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
            Some((t, _)) => {
                let y = top - top * t;
                ys.push(y);
                top = y - 0.05;
            }
            None => break,
        }
    }
    ys
}

#[test]
#[ignore = "probe3: vertical profiles; run with --ignored --nocapture"]
fn probe3_the_profiles() {
    let w = load_warm();
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("== profile along z at the beam rest centre x 137.5 ==");
    for z in [-256.0f32, -260.0, -264.0, -268.0, -270.0, -272.0, -276.0, -280.0, -284.0, -288.0, -292.0, -296.0, -300.0, -304.0, -308.0] {
        eprintln!("[probe3] x 137.5 z {z:>7.1}: {:?}", column(sim, 137.5, z));
    }
    eprintln!("== profile along z at the beam east reach x 175 ==");
    for z in [-256.0f32, -260.0, -264.0, -268.0, -270.0, -272.0, -276.0, -280.0, -284.0, -288.0, -292.0, -296.0, -300.0, -304.0, -308.0] {
        eprintln!("[probe3] x 175.0 z {z:>7.1}: {:?}", column(sim, 175.0, z));
    }
    eprintln!("== profile along x at the beam's z -287.11 ==");
    for x in [124.0f32, 128.0, 132.0, 136.0, 140.0, 144.0, 148.0, 152.0, 156.0, 160.0, 164.0, 168.0, 172.0, 176.0, 180.0, 184.0, 188.0, 192.0, 196.0, 200.0, 204.0] {
        eprintln!("[probe3] z -287.1 x {x:>5.1}: {:?}", column(sim, x, -287.11));
    }
    eprintln!("== profile along x at z -271 (the trench north) ==");
    for x in [124.0f32, 128.0, 132.0, 136.0, 140.0, 144.0, 148.0, 152.0, 156.0, 160.0, 164.0, 168.0, 172.0, 176.0, 180.0, 184.0, 188.0, 192.0, 196.0, 200.0, 204.0] {
        eprintln!("[probe3] z -271.0 x {x:>5.1}: {:?}", column(sim, x, -271.0));
    }
    eprintln!("== the warehouse start area (the 11.09 floor and its south edge) ==");
    for z in [-206.0f32, -210.0, -214.0, -218.0, -222.0, -226.0, -230.0] {
        for x in [172.0f32, 178.0, 184.0, 190.0, 196.0] {
            eprintln!("[probe3] start z {z:>6.1} x {x:>5.1}: {:?}", column(sim, x, z));
        }
    }
}

#[test]
#[ignore = "probe3: the nav web around the crane bay; run with --ignored --nocapture"]
fn probe3_the_nav_web() {
    let w = load_warm();
    let sim = w.scene.sim.as_ref().unwrap();
    // Nodes whose xz falls in the crane bay (x 120..230, z -200..-320).
    eprintln!("== nav nodes in the bay ==");
    for (pi, path) in sim.nav.paths.iter().enumerate() {
        for (ni, n) in path.nodes.iter().enumerate() {
            let p = n.pos;
            if p[0] > 120.0 && p[0] < 230.0 && p[2] > -322.0 && p[2] < -198.0 {
                eprintln!("[probe3] nav path{pi} node{ni} ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
            }
        }
    }
    // What does the game's own router say from the dock to the yard?
    for (from, to, label) in [
        ([189.0f32, 9.9, -288.7], [140.0, 5.5, -276.0], "dock -> yard (the stand-in spot)"),
        ([189.0, 9.9, -288.7], [136.0, 4.5, -308.5], "dock -> the ST-3 plate"),
        ([179.5, 11.1, -205.4], [189.0, 9.9, -288.7], "start -> dock"),
        ([189.0, 9.9, -288.7], [200.5, 13.4, -262.4], "dock -> rotate cab"),
    ] {
        match walker::nav_route(&sim, from, to) {
            Some(pts) => {
                eprintln!("[probe3] nav {label}: {} pts", pts.len());
                for p in pts.iter().take(24) { eprintln!("[probe3]   ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
            }
            None => eprintln!("[probe3] nav {label}: NO ROUTE"),
        }
    }
}
