//! probe12: SP #492's script clock during/after the E — the clock, the play byte, the
//! stream-0 matrix, every 10 ticks. Print-only.

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
#[ignore = "probe12: the rotate script clock; run with --ignored --nocapture"]
fn probe12_rotate_clock() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492).unwrap();
        let sc = sp.info[sp.cur].as_ref();
        eprintln!("[probe12] pre: state {} clock {:?} play {:?}",
            sp.state, sc.map(|s| s.clock), sc.map(|s| s.play));
    }
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E #450");
    w.wait(5);
    eprintln!("[probe12] E ok {used} ch0xa {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0xa));
    for k in 0..80 {
        w.wait(10);
        let sim = w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492).unwrap();
        let sc = sp.info[sp.cur].as_ref();
        if k % 3 == 0 || sp.state != 2 {
            eprintln!("[probe12] t{} state {} clock {:?} play {:?} bond {:?}",
                (k + 1) * 10, sp.state, sc.map(|s| s.clock), sc.map(|s| s.play), pos(&w));
        }
        if sp.state == 5 && k > 6 { break; }
    }
}
