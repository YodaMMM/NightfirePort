//! probe13: SP #492 per-tick after the E press — state, clock, play, stay. Print-only.

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

#[test]
#[ignore = "probe13: per-tick rotate SP; run with --ignored --nocapture"]
fn probe13_rotate_pertick() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E #450");
    eprintln!("[probe13] E ok {used}");
    for k in 0..24 {
        let sim = w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492).unwrap();
        let sc = sp.info[sp.cur].as_ref();
        eprintln!("[probe13] k{} state {} clock {:?} play {:?} stay {} cur {}",
            k, sp.state, sc.map(|s| s.clock), sc.map(|s| s.play), sp.stay, sp.cur);
        if sp.state == 5 { break; }
        w.wait(1);
    }
}
