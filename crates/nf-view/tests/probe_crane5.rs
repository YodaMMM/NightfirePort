//! probe5: the crane route's upper legs on REAL input from the warehouse ground:
//! east along the wall → ladder #435 climb → wire #271 hand-over-hand (strafe −1, B→A)
//! → the rotate cab → E #450 → the rotation ride. Print-only, no assertions.

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
#[ignore = "probe5: ground -> ladder -> wire -> rotate cab; run with --ignored --nocapture"]
fn probe5_ground_to_rotate() {
    let mut w = load_real();
    // Re-run the proven ground approach (probe4): off the platform, the descending ground.
    w.walk_to(Wp::new(178.0, 11.2, -206.5).r(1.0), "leg1 the platform");
    let ok = w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the drop to the ground");
    eprintln!("[probe5] drop ok {ok} at {:?}", pos(&w));

    // East along the wall, north-east to the ladder #435 base (212.47, ~0, -234.51).
    let legs: [([f32; 3], f32, &str); 6] = [
        ([190.0, -3.0, -266.0], 2.0, "g1 east along the wall"),
        ([200.0, -1.0, -256.0], 2.0, "g2 north-east"),
        ([208.0, 0.0, -246.0], 2.0, "g3 the ground rises"),
        ([214.0, 0.5, -238.0], 1.5, "g4 east of the ladder"),
        ([214.5, 1.0, -235.0], 1.2, "g5 the ladder approach"),
        ([213.2, 1.5, -234.8], 0.8, "g6 at the base"),
    ];
    for (at, r, label) in legs {
        let ok = w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label);
        eprintln!("[probe5] {label} ok {ok} at {:?}", pos(&w));
    }

    // The ladder climb: walk into the centre from the east and hold forward.
    let climbed = w.try_walk_to(Wp::at(212.47, 16.0, -234.51, Act::Ladder { top_y: 15.6 }), "climb #435");
    eprintln!("[probe5] climb ok {climbed} at {:?}", pos(&w));

    // From the ladder top: step to the wire #271 B end (211.43, 17.04, -238.0) and grab.
    // Hand-over-hand toward A = strafe −1 (the walker's WireRide uses strafe 1 toward B).
    let to_wire = w.try_walk_to(Wp { at: [211.4, 16.9, -237.9], act: Act::None, arrive: 1.0 }, "to the wire B end");
    eprintln!("[probe5] to-wire ok {to_wire} at {:?}", pos(&w));
    // Grab: walk INTO the wire point; the grab scan runs per tick.
    let grabbed = w.try_walk_to(Wp::at(211.43, 17.04, -238.0, Act::WireRide), "grab wire #271");
    eprintln!("[probe5] grab ok {grabbed} substate {} at {:?}",
        w.scene.sim.as_ref().unwrap().bond.substate, pos(&w));
    // Ride with strafe −1 (toward A) until the wire drops us.
    let start = w.total_ticks;
    while w.total_ticks - start < 1800 {
        let sim = w.scene.sim.as_ref().unwrap();
        let on = sim.bond.substate;
        if on != nf_game::player::SS_WIRE && on != nf_game::player::SS_ZIPLINE {
            eprintln!("[probe5] wire ride ENDED at +{} ticks at {:?}", w.total_ticks - start, pos(&w));
            break;
        }
        w.tick(Input { strafe: -1.0, ..Default::default() }, false);
        if (w.total_ticks - start) % 300 == 0 {
            eprintln!("[probe5] ride t{} at {:?}", w.total_ticks - start, pos(&w));
        }
    }
    for _ in 0..30 { w.wait(10); }
    eprintln!("[probe5] after the wire at {:?}", pos(&w));

    // E the rotate touchplate #450 (199.53, 13.31, -263.81) from the cab.
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E #450");
    w.wait(10);
    eprintln!("[probe5] E #450 ok {used} ch0xa {} at {:?}",
        w.scene.sim.as_ref().unwrap().drones.channels.on(0xa), pos(&w));
    // Hold still through the rotate and print.
    for k in 0..14 {
        w.wait(60);
        eprintln!("[probe5] rotate t{} bond {:?}", (k + 1) * 60, pos(&w));
    }
    for f in &w.report.falls { eprintln!("[probe5] FALL {f}"); }
}
