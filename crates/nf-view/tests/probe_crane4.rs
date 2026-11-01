//! probe4: walk the 0700000d warehouse→dock on REAL input from the spawn, printing Bond's
//! position after each short southward leg — the sim's own streamed geometry reveals the
//! route (the cold ray-maps miss un-streamed cels). Print-only, no assertions.

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
#[ignore = "probe4: the real-input warehouse walk; run with --ignored --nocapture"]
fn probe4_walk_the_warehouse() {
    let mut w = load_real();
    eprintln!("[probe4] spawn {:?}", pos(&w));

    // The walkreal's own leg 1 (proven to walk).
    w.walk_to(Wp::new(178.0, 11.2, -206.5).r(1.0), "leg1 the walkreal's stop");
    eprintln!("[probe4] leg1 {:?}", pos(&w));

    // Short southward hops; the sim streams the warehouse as Bond moves. The waypoint y is
    // only a steering hint — walk_to steers on xz and Bond follows the real floor.
    let legs: [([f32; 3], f32, &str); 12] = [
        ([178.0, 11.2, -210.0], 1.2, "s1 z-210"),
        ([179.0, 10.0, -216.0], 1.2, "s2 z-216"),
        ([180.0, 9.0, -222.0], 1.2, "s3 z-222"),
        ([181.0, 7.5, -227.5], 1.5, "s4 the Start Trigger #513"),
        ([182.0, 7.0, -234.0], 1.5, "s5 z-234"),
        ([183.0, 8.0, -242.0], 1.5, "s6 z-242"),
        ([184.0, 9.0, -250.0], 1.5, "s7 z-250"),
        ([185.0, 10.0, -258.0], 1.5, "s8 z-258"),
        ([186.0, 11.0, -264.0], 1.5, "s9 z-264"),
        ([187.0, 10.0, -270.0], 1.5, "s10 z-270"),
        ([188.0, 9.6, -280.0], 1.5, "s11 z-280 the dock"),
        ([189.0, 9.9, -288.0], 1.2, "s12 the touchplate #460"),
    ];
    for (at, r, label) in legs {
        let ok = w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label);
        let p = pos(&w);
        eprintln!("[probe4] {label} ok {ok} pos ({:.2},{:.2},{:.2}) falls {}",
            p[0], p[1], p[2], w.report.falls.len());
        if !ok { eprintln!("[probe4]   (gave up)"); }
    }
    // Where did the falls/stucks happen?
    for f in &w.report.falls { eprintln!("[probe4] FALL {f}"); }
    for s in w.report.stuck_events.iter().take(10) { eprintln!("[probe4] STUCK {s}"); }
}
