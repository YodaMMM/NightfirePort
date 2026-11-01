//! probe8: the full crane chain — ladder #435 → jump-grab wire #271 → hand-over-hand to A
//! → land on the rotate crane → E #450 → the rotation ride → where Bond ends up.
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

/// The proven approach: spawn → the platform edge → the ground → east → the ladder → the top.
fn approach_top(w: &mut Walker) {
    w.walk_to(Wp::new(178.0, 11.2, -206.5).r(1.0), "leg1 the platform");
    let ok = w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the drop");
    eprintln!("[probe8] drop ok {ok}");
    for (at, r, label) in [
        ([190.0f32, -3.0, -266.0], 2.0, "g1"),
        ([200.0, -1.0, -256.0], 2.0, "g2"),
        ([208.0, 0.0, -246.0], 2.0, "g3"),
        ([214.0, 0.5, -238.0], 1.5, "g4"),
        ([214.5, 1.0, -235.0], 1.2, "g5"),
        ([213.2, 1.5, -234.8], 0.8, "g6"),
    ] {
        let ok = w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label);
        if !ok { eprintln!("[probe8] {label} GAVE UP at {:?}", pos(w)); }
    }
    let climbed = w.try_walk_to(Wp::at(212.47, 16.0, -234.51, Act::Ladder { top_y: 15.6 }), "climb #435");
    eprintln!("[probe8] climb ok {climbed} at {:?}", pos(w));
}

#[test]
#[ignore = "probe8: the wire crossing and the rotate ride; run with --ignored --nocapture"]
fn probe8_wire_crossing_and_rotate() {
    let mut w = load_real();
    approach_top(&mut w);
    // Face along the wire toward A and walk a step (grounded), then jump-grab.
    let sim = w.scene.sim.as_mut().unwrap();
    let desired = (199.78f32 - sim.bond.pos[0]).atan2(-259.04 - sim.bond.pos[2]);
    for _ in 0..30 {
        let sim = w.scene.sim.as_mut().unwrap();
        let e = walker::wrap_pi(desired - sim.bond.yaw);
        sim.step_bond(Input { turn: e.clamp(-0.08, 0.08), forward: 0.3, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    let mut grabbed = false;
    for k in 0..80 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(Input { jump: (60..=63).contains(&k), ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if sim.bond.substate == nf_game::player::SS_WIRE { grabbed = true; eprintln!("[probe8] grabbed at +{k}"); break; }
    }
    eprintln!("[probe8] grab {grabbed} at {:?}", pos(&w));
    if !grabbed { return; }

    // Hand-over-hand toward A: strafe +1 = mode 7 = faces A-ward (wire.yaw + 3π/2
    // [M 0x1977c0-0x1979d0]) and the b4/b5 root motion (+z local [D track spans]) carries
    // A-ward. The ride is ~24 m at ~0.0175 m/tick ≈ 1370 ticks, inside the 1800-tick
    // auto-drop [M 0x197748-0x197754].
    let start = w.total_ticks;
    while w.total_ticks - start < 2600 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate != nf_game::player::SS_WIRE {
            eprintln!("[probe8] ride ENDED at +{} at {:?}", w.total_ticks - start, pos(&w));
            break;
        }
        w.tick(Input { strafe: 1.0, ..Default::default() }, false);
        if (w.total_ticks - start) % 200 == 0 {
            eprintln!("[probe8] ride t{} {:?}", w.total_ticks - start, pos(&w));
        }
    }
    for _ in 0..50 { w.wait(10); }
    eprintln!("[probe8] landed {:?}", pos(&w));

    // Walk to the rotate plate #450 (199.53, 13.31, -263.81) and E it.
    for (at, r, label) in [
        ([199.5f32, 14.0, -261.5], 1.5, "c1"),
        ([199.6, 13.5, -263.5], 1.0, "c2"),
    ] {
        let ok = w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label);
        eprintln!("[probe8] {label} ok {ok} at {:?}", pos(&w));
    }
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E #450");
    w.wait(10);
    eprintln!("[probe8] E #450 ok {used} ch0xa {} at {:?}",
        w.scene.sim.as_ref().unwrap().drones.channels.on(0xa), pos(&w));
    // Hold still through the rotate; print Bond + the crane matrix every 60.
    for k in 0..16 {
        w.wait(60);
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 1));
        let c = m.map(|m| [m[3][0], m[3][1], m[3][2]]);
        eprintln!("[probe8] rot t{} bond {:?} crane {:?}", (k + 1) * 60, pos(&w), c);
    }
    for f in &w.report.falls { eprintln!("[probe8] FALL {f}"); }
}
