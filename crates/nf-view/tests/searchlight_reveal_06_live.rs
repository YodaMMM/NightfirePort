//! light-glow: the 07000006 courtyard searchlights' reveal beat.
//!
//! The reveal NIS 0x06000066 (stream 0 raises the heads' on-channels 64/63/61/62 at waits 1/59/116/167
//! [D 07000006 kind-7 NIS]) is owned by SP placement #808 (p0 = 0x06000066, p5 mask = 1,
//! p6 = ch 0x34) [D]. `SP_Hit` 0x1dc470:16-18 raises bit 1 while `switch_channels[tag+0x30]` (p6) is
//! on and `SP_Update` 0x1dbda8 case 0 plays the script once `bits & tag+0x2E` is nonzero
//! [M 0x1dbda8:64-106]. Ch 0x34 is written by the courtyard TouchOnce plates #524/#525/#526/#661-#664
//! [D trigger census]. So the lights stay dark until Bond steps on a plate.
//!
//! The visible symptom this pins: Bond placed in the courtyard without crossing a plate (the PS2
//! capture at 7.24, 42.03, -39.31) sees four dark heads — no beams, no coronas — where the port used
//! to boot the script at level start and draw two full-size coronas over the wall.
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// 07000006 after its intro, Bond free.
fn load_06(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000006.bin", false).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!sim.disabled, "Bond is free after the intro");
    scene
}

fn heads(sim: &nf_view::scene::Sim) -> Vec<&nf_game::searchlight::Part> {
    sim.drones.searchlights.parts.iter().filter(|p| p.kind == 1).collect()
}

/// Corona parts drawn this tick.
fn coronas_drawn(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.searchlights.parts.iter().filter(|p| p.kind == 4 && p.drawn).count()
}

fn place(sim: &mut nf_view::scene::Sim, p: [f32; 3], yaw: f32) {
    sim.bond = nf_game::player::Bond::new(p, yaw);
}

#[test]
fn courtyard_without_a_plate_stays_dark() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load_06(&iso);
    let sim = scene.sim.as_mut().unwrap();
    // The PS2 capture's spot, held there: no plate under Bond.
    place(sim, [7.24, 42.03, -39.31], 10.161);
    for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x34), "no ch-0x34 plate crossed");
    let h = heads(sim);
    assert_eq!(h.len(), 4, "07000006 has four searchlight heads");
    for p in &h {
        assert_eq!(p.state, 3, "head at {:?} stays dark (state 3) until the reveal script runs", p.pos);
        assert!(!sim.drones.channels.on(p.on as u32), "head on-channel {} still off", p.on);
    }
    assert_eq!(coronas_drawn(sim), 0, "no corona drawn while the heads are dark");
}

#[test]
fn entry_plate_reveals_all_four() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load_06(&iso);
    let sim = scene.sim.as_mut().unwrap();
    // Plate #525 [D trigger census: box (7.13, 41.71, -12.18)-(9.64, 41.71, -10.67), out ch 0x34]:
    // stand on its centre.
    place(sim, [8.38, 41.9, -11.42], 3.14);
    let mut lit_at = None;
    for t in 0..60 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.channels.on(0x34) { lit_at = Some(t); break; }
    }
    assert!(lit_at.is_some(), "standing on plate #525 raises ch 0x34");
    // The reveal stages the four on-channels over 167 ticks [D NIS 0x06000066]; give it 240.
    for _ in 0..240 { sim.step_bond(Input::default(), false, false, false); }
    for p in heads(sim) {
        assert!(sim.drones.channels.on(p.on as u32), "head on-channel {} raised by the reveal", p.on);
        assert_ne!(p.state, 3, "head at {:?} lit after the reveal", p.pos);
    }
}

/// Sister check: the dark-state assertion can fail — ch 0x34 set by hand (what a plate does) lights
/// the same heads at the same spot, so `courtyard_without_a_plate_stays_dark` is not vacuous.
#[test]
fn sister_channel_by_hand_lights_them() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load_06(&iso);
    let sim = scene.sim.as_mut().unwrap();
    place(sim, [7.24, 42.03, -39.31], 10.161);
    sim.drones.channels.set(0x34, 1, 0);
    for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    let lit = heads(sim).iter().filter(|p| p.state != 3).count();
    assert_eq!(lit, 4, "all four heads leave state 3 once ch 0x34 rises");
}
