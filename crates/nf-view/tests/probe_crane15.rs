//! probe15: walk to the plate spot, then a manual per-tick E with per-tick SP #492 prints.
//! Print-only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;

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
#[ignore = "probe15: manual per-tick E; run with --ignored --nocapture"]
fn probe15_manual_e() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let ok = w.try_walk_to(Wp { at: [198.93, 14.16, -264.15], act: Act::None, arrive: 0.8 }, "to the plate spot");
    eprintln!("[probe15] walk ok {ok} at {:?}", pos(&w));
    // Manual E: face the plate (199.53, 13.31, -263.81), pulse E every 6 ticks, 300 ticks.
    for k in 0..300 {
        let press = k % 6 == 0;
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let desired = (199.53f32 - sim.bond.pos[0]).atan2(-263.81 - sim.bond.pos[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            sim.step_bond(Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, press);
        }
        w.total_ticks += 1;
        if k % 20 == 0 || k < 30 {
            let sim = w.scene.sim.as_ref().unwrap();
            let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492).unwrap();
            let sc = sp.info[sp.cur].as_ref();
            eprintln!("[probe15] k{} ch0xa {} state {} clock {:?} play {:?} bond {:?}",
                k, sim.drones.channels.on(0xa), sp.state, sc.map(|s| s.clock), sc.map(|s| s.play), pos(&w));
        }
        if sp_state_5(&w) { break; }
    }
    // Then watch the swing carry him: position + the stream-0 matrix + the floor below.
    for k in 0..40 {
        w.wait(20);
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 0));
        let b = pos(&w);
        let floor = sim.world.segment_hit([b[0], b[1] + 0.5, b[2]], [b[0], b[1] - 30.0, b[2]])
            .map(|(t, _)| b[1] + 0.5 - (b[1] + 0.5 - (b[1] - 30.0)) * t);
        eprintln!("[probe15] swing k{} rot {:?} bond {:?} floor {:?} sp {:?}",
            k * 20, m.map(|m| [m[0][0], m[0][2], m[2][0], m[2][2]]), b, floor,
            sim.drones.scripts.players.iter().find(|p| p.placement == 492).map(|p| p.state));
    }
}

fn sim_has(f: fn(&Walker) -> bool) -> impl Fn(&Walker) -> bool { f }

fn sp_state_5(w: &Walker) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.scripts.players.iter().find(|p| p.placement == 492).map(|p| p.state) == Some(5)
}
