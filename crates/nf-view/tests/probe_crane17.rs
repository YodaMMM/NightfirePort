//! probe17 v4: the deck→yard crossing at the REST pose — the static walkway strip (12.13,
//! r 6.5-8 [probe16/probe19: its collision never moves; the swing is the cut-scene's beat]),
//! the stall-jump over the deck's rim gap, the walkway walk, the drop onto the dock, the E #460.
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

fn beam_x(sim: &nf_view::scene::Sim) -> Option<f32> {
    sim.drones.scripts.objects.list.iter().flatten()
        .filter_map(|o| if let nf_game::nis::Obj::Entity { hash, matrix, .. } = o {
            (*hash == 0x0200_0a80).then_some(matrix[3][0])
        } else { None })
        .next()
}

#[test]
#[ignore = "probe17: the rest-pose crossing; run with --ignored --nocapture"]
fn probe17_rest_crossing() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    assert!(w.try_walk_to(Wp { at: [198.93, 14.16, -264.15], act: Act::None, arrive: 0.8 }, "the cab"));
    // NO E — the crossing uses the static walkway; the walkreal's own E beat stays as it is.
    let anchor = [200.48f32, -262.37f32];
    let bb = 1.065f32 - std::f32::consts::PI; // the rest jib bearing
    let (dx, dz) = (bb.sin(), bb.cos());
    eprintln!("[p17] rest jib dir ({dx:.3},{dz:.3})");
    let mut on_walkway = false;
    let mut jumps = 0u32;
    let mut stall = 0u32;
    let mut last = pos(&w);
    for k in 0..500u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        let r = ((b[0] - anchor[0]).powi(2) + (b[2] - anchor[1]).powi(2)).sqrt();
        // The aim: r 7.25 along the axis, 1.0 to the strip's side — the fall's tangential
        // carry drifts +0.25 to the positive side [probe17 v3's +0.126 landing], so the bias
        // overcompensates it into the strip (off -0.75..0).
        let tx = anchor[0] + 7.25 * dx + 1.0 * dz;
        let tz = anchor[1] + 7.25 * dz - 1.0 * dx;
        let desired = (tx - b[0]).atan2(tz - b[2]);
        let grounded = sim.bond.grounded;
        let moved = ((b[0] - last[0]).powi(2) + (b[2] - last[2]).powi(2)).sqrt();
        last = b;
        stall = if grounded && moved < 0.012 && r < 6.2 { stall + 1 } else { 0 };
        let jump = grounded && stall > 12 && jumps < 5;
        if jump { jumps += 1; eprintln!("[p17] JUMP {jumps} at r{r:.2} ({:.2},{:.2},{:.2})", b[0], b[1], b[2]); }
        {
            let mut sim_m = w.scene.sim.as_mut().unwrap();
            sim_m.bond.yaw = desired;
            sim_m.step_bond(Input { forward: 1.0, jump, ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
        let b2 = pos(&w);
        let g2 = w.scene.sim.as_ref().unwrap().bond.grounded;
        if !on_walkway && g2 && b2[1] > 11.0 && b2[1] < 13.5 && b2[0] > 192.0 && b2[0] < 196.5 && b2[2] < -263.5 && b2[2] > -268.0 {
            on_walkway = true;
            eprintln!("[p17] ON THE WALKWAY k{k} ({:.2},{:.2},{:.2})", b2[0], b2[1], b2[2]);
        }
        if k % 40 == 0 {
            eprintln!("[p17] walk k{k} ({:.2},{:.2},{:.2}) r{r:.1} g{grounded} stall{stall} jumps{jumps}", b[0], b[1], b[2]);
        }
        if on_walkway { break; }
    }
    eprintln!("[p17] on_walkway {on_walkway} at {:?}", pos(&w));
    if !on_walkway { eprintln!("[p17] ABORT: no walkway landing"); return; }
    // Walk outward along the axis, off the walkway's end (r 8), down onto the dock (9.5).
    for k in 0..200u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let _ = sim;
        {
            let mut sim_m = w.scene.sim.as_mut().unwrap();
            sim_m.bond.yaw = bb;
            sim_m.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
        if k % 20 == 0 {
            let b = pos(&w);
            eprintln!("[p17] out k{k} ({:.2},{:.2},{:.2})", b[0], b[1], b[2]);
        }
        if pos(&w)[1] < 10.2 { break; }
    }
    w.wait(80);
    let dock = pos(&w);
    eprintln!("[p17] DOCK {dock:?} grounded {}", w.scene.sim.as_ref().unwrap().bond.grounded);
    if dock[1] < 8.5 || dock[1] > 11.0 { eprintln!("[p17] ABORT: not on the dock"); return; }
    // The dock walk to the control corner + E #460.
    for (at, rr, label) in [
        ([196.0f32, 9.7, -277.0], 2.0, "the dock strip"),
        ([192.0, 9.7, -283.0], 1.5, "the dock south"),
        ([189.3, 9.8, -287.2], 1.0, "the control corner"),
    ] {
        if !w.try_walk_to(Wp { at, act: Act::None, arrive: rr }, label) {
            eprintln!("[p17] dock leg stuck: {label} at {:?}", pos(&w));
        }
    }
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -287.6, Act::Use { use_ticks: 60 }), "E #460");
    eprintln!("[p17] E460 {used} ch0x2b {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b));
    // The beam's shuttle (the first 600 ticks).
    for k in 0..10 {
        w.wait(60);
        let sim = w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 115).map(|p| p.state);
        eprintln!("[p17] beam t{} x{:?} sp{sp:?}", (k + 1) * 60, beam_x(&sim));
    }
    for f in &w.report.falls { eprintln!("[p17] FALL {f}"); }
}
