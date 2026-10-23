//! copter-script-draw (the helicopter was missing from The Exchange): the 07000008
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
    // The copter's ScriptPlayer (836, Copter_Create 0x17d340 → SP_Create modes 8,8 [M 0x1dc7e0:65-70])
    // plays its loaded script 0x06000051 and is SP_HideObj'd while the gate channel is off [M
    // 0x17d340:73-74]; the gate's first ON tick unhides it [M Copter_Update 0x17dbb8 head]. The
    // flight route itself is CopterPaths, not a script one-shot.
    assert!(scene.attachments.iter().any(|a| matches!(a.script, Some((0x0600_0051, 0, 0x0200_035D, _)))),
        "the SP's playing script 0x06000051 stream 0 (0x0200035D) has a script draw slot");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.script_entity((0x0600_0051, 0), 0x0200_035D).is_none(), "hidden before the gate (ch 5 off) [M 0x17d340:73-74]");
    let f = sim.drones.frame;
    sim.drones.channels.set(5, 1, f);
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.script_entity((0x0600_0051, 0), 0x0200_035D).is_some(), "ch 5 unhides the copter's SP: its entity is live and drawable [M 0x17dbb8 head]");
}
