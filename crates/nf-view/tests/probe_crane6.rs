//! probe6: the wire jump-grab at ladder #435's top, the hand-over-hand crossing to the
//! rotate crane, the cab, E #450, and where the rotation carries a rider.
//! Print-only, no assertions.

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
#[ignore = "probe6: the wire jump-grab and the rotate ride; run with --ignored --nocapture"]
fn probe6_wire_to_rotate() {
    let mut w = load_real();
    // The proven ground approach + ladder climb (probe5).
    w.walk_to(Wp::new(178.0, 11.2, -206.5).r(1.0), "leg1 the platform");
    let ok = w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the drop to the ground");
    eprintln!("[probe6] drop ok {ok} at {:?}", pos(&w));
    for (at, r, label) in [
        ([190.0f32, -3.0, -266.0], 2.0, "g1"),
        ([200.0, -1.0, -256.0], 2.0, "g2"),
        ([208.0, 0.0, -246.0], 2.0, "g3"),
        ([214.0, 0.5, -238.0], 1.5, "g4"),
        ([214.5, 1.0, -235.0], 1.2, "g5"),
        ([213.2, 1.5, -234.8], 0.8, "g6"),
    ] {
        let ok = w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label);
        eprintln!("[probe6] {label} ok {ok} at {:?}", pos(&w));
    }
    let climbed = w.try_walk_to(Wp::at(212.47, 16.0, -234.51, Act::Ladder { top_y: 15.6 }), "climb #435");
    eprintln!("[probe6] climb ok {climbed} at {:?}", pos(&w));
    w.wait(20);
    eprintln!("[probe6] at the top {:?} icon_ctx {}", pos(&w),
        w.scene.sim.as_ref().unwrap().bond.icon_ctx);

    // The jump-grab: with the wire icon in context, jump grabs the nearest wire
    // [M Player_HandleJump 0x1916b0:118-147]. Face along the wire first (toward A,
    // (199.78, 17.04, -259.04) from (212.6, -234.5): direction (-12.8, -24.5) normalized).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let desired = (199.78f32 - sim.bond.pos[0]).atan2(-259.04 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        for _ in 0..20 {
            let sim = w.scene.sim.as_mut().unwrap();
            let e = walker::wrap_pi(desired - sim.bond.yaw);
            sim.step_bond(Input { turn: e.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
    }
    eprintln!("[probe6] faced A, yaw {:.2} at {:?}", w.scene.sim.as_ref().unwrap().bond.yaw, pos(&w));
    // Jump → the grab replaces the jump [M].
    for k in 0..40 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(Input { jump: k == 0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if sim.bond.substate == nf_game::player::SS_WIRE {
            eprintln!("[probe6] WIRE GRABBED at tick +{k} at {:?}", pos(&w));
            break;
        }
    }
    eprintln!("[probe6] after jump-grab {:?} substate {}", pos(&w), w.scene.sim.as_ref().unwrap().bond.substate);

    // Hand-over-hand toward A with strafe −1.
    let start = w.total_ticks;
    while w.total_ticks - start < 2400 {
        let sim = w.scene.sim.as_ref().unwrap();
        let on = sim.bond.substate;
        if on != nf_game::player::SS_WIRE && on != nf_game::player::SS_ZIPLINE {
            eprintln!("[probe6] ride ENDED at +{} at {:?}", w.total_ticks - start, pos(&w));
            break;
        }
        w.tick(Input { strafe: -1.0, ..Default::default() }, false);
        if (w.total_ticks - start) % 240 == 0 {
            eprintln!("[probe6] ride t{} {:?}", w.total_ticks - start, pos(&w));
        }
    }
    for _ in 0..40 { w.wait(10); }
    eprintln!("[probe6] landed {:?}", pos(&w));

    // Explore the rotate crane: walk toward the cab plate #450 (199.53, 13.31, -263.81).
    for (at, r, label) in [
        ([199.5f32, 14.0, -261.0], 1.5, "c1 the cab area"),
        ([199.6, 13.5, -263.5], 1.0, "c2 the plate"),
    ] {
        let ok = w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label);
        eprintln!("[probe6] {label} ok {ok} at {:?}", pos(&w));
    }
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E #450");
    w.wait(10);
    eprintln!("[probe6] E #450 ok {used} ch0xa {} at {:?}",
        w.scene.sim.as_ref().unwrap().drones.channels.on(0xa), pos(&w));
    // Hold still through the rotate; print Bond + the crane matrix.
    for k in 0..14 {
        w.wait(60);
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 1));
        let c = m.map(|m| [m[3][0], m[3][1], m[3][2]]);
        eprintln!("[probe6] rotate t{} bond {:?} crane-origin {:?}", (k + 1) * 60, pos(&w), c);
    }
    for f in &w.report.falls { eprintln!("[probe6] FALL {f}"); }
}
