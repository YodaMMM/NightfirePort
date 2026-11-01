//! probe11: the rotate SP #492's full timeline from before the E — state, clock matrix
//! rotation, every 30 ticks. Print-only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_real() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    w
}

fn pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

#[test]
#[ignore = "probe11: the rotate SP timeline; run with --ignored --nocapture"]
fn probe11_rotate_timeline() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    // Pre-E baseline.
    for k in 0..3 {
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 1));
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492);
        eprintln!("[probe11] pre{k} rot {:?} sp {:?}", m.map(|m| [m[0][0], m[0][2], m[2][0], m[2][2]]), sp.map(|p| p.state));
        w.wait(10);
    }
    // E with per-30-tick prints DURING the use hold.
    let start = w.total_ticks;
    let mut pressed = false;
    loop {
        if w.total_ticks - start > 1200 { break; }
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let desired = (199.53f32 - sim.bond.pos[0]).atan2(-263.81 - sim.bond.pos[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let press = !pressed && err.abs() < 0.3;
            if press { pressed = true; }
            sim.step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, press, false, false);
        }
        w.total_ticks += 1;
        let t = w.total_ticks - start;
        if t % 30 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 1));
            let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492);
            eprintln!("[probe11] t{t} ch0xa {} rot {:?} sp {:?}", sim.drones.channels.on(0xa),
                m.map(|m| [m[0][0], m[0][2], m[2][0], m[2][2]]), sp.map(|p| p.state));
        }
        if pressed && w.scene.sim.as_ref().unwrap().drones.channels.on(0xa) && w.total_ticks - start > 900 { break; }
    }
    // Post-E watch.
    for k in 0..30 {
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 1));
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492);
        eprintln!("[probe11] post{} rot {:?} sp {:?} bond {:?}", k * 30,
            m.map(|m| [m[0][0], m[0][2], m[2][0], m[2][2]]), sp.map(|p| p.state), pos(&w));
    }
}
