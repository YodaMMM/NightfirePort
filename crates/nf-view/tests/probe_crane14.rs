//! probe14: how long does the E leg actually run, and what is the script clock at k0?
//! Print-only.

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

fn sp492(w: &Walker) -> (u16, Option<f32>, Option<u8>) {
    let sim = w.scene.sim.as_ref().unwrap();
    let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492).unwrap();
    (sp.state, sp.info[sp.cur].as_ref().map(|s| s.clock), sp.info[sp.cur].as_ref().map(|s| s.play))
}

#[test]
#[ignore = "probe14: the E leg timing; run with --ignored --nocapture"]
fn probe14_e_leg_timing() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    eprintln!("[probe14] pre-E ticks {} sp {:?}", w.total_ticks, sp492(&w));
    let t0 = w.total_ticks;
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E #450");
    let t1 = w.total_ticks;
    eprintln!("[probe14] E ok {used} leg {} ticks (walk+use) sp {:?}", t1 - t0, sp492(&w));
    // Watch 800 ticks: the clock per tick.
    for k in 0..80 {
        w.wait(10);
        let (st, cl, pl) = sp492(&w);
        if k % 4 == 0 || st == 5 {
            eprintln!("[probe14] +{} state {:?} clock {:?} play {:?}", (k + 1) * 10, st, cl, pl);
        }
        if st == 5 { break; }
    }
    eprintln!("[probe14] bond at {:?}", pos(&w));
}
