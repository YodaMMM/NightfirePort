//! probe7: diagnose the wire jump-grab at the ladder top — per-tick player state around
//! the jump press. Print-only.

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
#[ignore = "probe7: the jump-grab diagnosis; run with --ignored --nocapture"]
fn probe7_jump_grab_diagnosis() {
    let mut w = load_real();
    w.walk_to(Wp::new(178.0, 11.2, -206.5).r(1.0), "leg1 the platform");
    let ok = w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the drop");
    for (at, r, label) in [
        ([190.0f32, -3.0, -266.0], 2.0, "g1"),
        ([200.0, -1.0, -256.0], 2.0, "g2"),
        ([208.0, 0.0, -246.0], 2.0, "g3"),
        ([214.0, 0.5, -238.0], 1.5, "g4"),
        ([214.5, 1.0, -235.0], 1.2, "g5"),
        ([213.2, 1.5, -234.8], 0.8, "g6"),
    ] {
        let _ = w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label);
    }
    let climbed = w.try_walk_to(Wp::at(212.47, 16.0, -234.51, Act::Ladder { top_y: 15.6 }), "climb #435");
    eprintln!("[probe7] climb ok {climbed} at {:?}", pos(&w));
    // Face along the wire toward A, then walk one step toward A (grounded ticks).
    let sim = w.scene.sim.as_mut().unwrap();
    let desired = (199.78f32 - sim.bond.pos[0]).atan2(-259.04 - sim.bond.pos[2]);
    for k in 0..30 {
        let sim = w.scene.sim.as_mut().unwrap();
        let e = walker::wrap_pi(desired - sim.bond.yaw);
        sim.step_bond(Input { turn: e.clamp(-0.08, 0.08), forward: 0.3, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    // Diagnose 90 ticks; jump pressed at k 60..63.
    for k in 0..90 {
        let jump = (60..=63).contains(&k);
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(Input { jump, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        let b = &sim.bond;
        if k >= 55 && k <= 75 {
            eprintln!("[probe7] t{k} jump{jump} grounded {} gh {} jumpvar {} ctx {} sub {} headbump {} pos {:?}",
                b.grounded, b.grounded_history, b.jump, b.icon_ctx, b.substate, b.head_bump, b.pos);
        }
        if b.substate == nf_game::player::SS_WIRE {
            eprintln!("[probe7] GRABBED at t{k}");
            break;
        }
    }
    eprintln!("[probe7] final sub {} at {:?}", w.scene.sim.as_ref().unwrap().bond.substate, pos(&w));
}
