//! copter-script-draw (user report 2026-10-21: "heli is missing from the exchange"): the 07000008
//! extraction helicopter has no glist of its own — `Copter_Create` 0x17d340 hands its look to the
//! script player made from the placement's params 4/5 (0x06000051 the assembly, 0x06000050 the
//! flight [D #45]). The flight script's body entity 0x02000427 (type 0x2F) needs a draw slot,
//! and once the gate channel (ch 5 [D #45 param]) opens the flight, that entity is posed by the
//! stream and moves.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn the_copter_flight_body_has_a_draw_slot_and_flies() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load 07000008");
    assert!(scene.attachments.iter().any(|a| matches!(a.script, Some((0x0600_0050, 0, 0x0200_0427, _)))),
        "the flight script's body 0x02000427 (stream 0) has a script draw slot");
    // sister: a parented part of the flight (stream 1, 0x0200035E) too
    assert!(scene.attachments.iter().any(|a| matches!(a.script, Some((0x0600_0050, 1, 0x0200_035E, _)))),
        "the flight's parented part 0x0200035E has a slot");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.script_entity((0x0600_0050, 0), 0x0200_0427).is_none(), "no flight before the gate (ch 5 off)");
    let f = sim.drones.frame;
    sim.drones.channels.set(5, 1, f);
    let mut first = None;
    let mut moved = false;
    for _ in 0..200 {
        sim.step_bond(Default::default(), false, false, false);
        if let Some(m) = sim.script_entity((0x0600_0050, 0), 0x0200_0427) {
            let p = [m[3][0], m[3][1], m[3][2]];
            if first.is_none() { eprintln!("FLIGHT START {p:?} frame {}", sim.drones.frame); }
            match first { None => first = Some(p), Some(q) => if (0..3).any(|k| (p[k] - q[k]).abs() > 1.0) { moved = true; } }
        }
    }
    assert!(first.is_some(), "ch 5 starts the flight: the body entity is live");
    assert!(moved, "the body flies (moved > 1 unit along the flight's spline)");
}
