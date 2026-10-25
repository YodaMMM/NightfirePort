//! level-07-party-door-lock probe: what does the port do with door #251 (starts-open, open ch 130,
//! lock ch 200) across the control-taking scene (script 0x06000073 stream 8's event 0x0D [130, 0])?
//! Assertion-free diagnostic, run with `cargo test -p nf-view --test probe_07_door_lock -- --nocapture`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load() -> Option<nf_view::scene::Scene> {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let iso = iso()?;
    Some(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"))
}

fn dump(sim: &nf_view::scene::Sim, tag: &str) {
    let d251 = sim.drones.doors.list.iter().find(|d| d.placement == 251);
    let d245 = sim.drones.doors.list.iter().find(|d| d.placement == 245);
    println!("[{tag}] bond {:?} disabled {} ch130 {} ch200 {} ch0x48 {}",
        sim.bond.pos, sim.disabled as u8,
        sim.drones.channels.on(130) as u8, sim.drones.channels.on(200) as u8,
        sim.drones.channels.on(0x48) as u8);
    if let Some(d) = d251 { println!("[{tag}] door #251 state {:#04x} ch_open {} ch_lock {} flags {:#04x} pos {:?}",
        d.state, d.ch_open, d.ch_lock, d.flags, d.pos); }
    if let Some(d) = d245 { println!("[{tag}] door #245 state {:#04x}", d.state); }
}

#[test]
fn probe_the_door_across_the_scene() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set"); return };
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..400 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        dump(sim, "boot");
    }
    // Teleport onto the hall walk-in plate (the interact style) — the real trigger runs the scene.
    {
        let sim = scene.sim.as_mut().unwrap();
        let c = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && u32::from(t.out) == 0x48)
            .map(|t| t.centre()).expect("TouchOnce #412");
        sim.bond.pos = [c[0], c[1], c[2]];
        dump(sim, "on the plate");
    }
    for t in 0..1400u32 {
        let sim = scene.sim.as_mut().unwrap();
        if !sim.disabled && t > 60 { break; }
        sim.drones.scripts.skip_request = sim.disabled;
        sim.step_bond(Input::default(), false, false, false);
        if t % 200 == 0 { dump(&scene.sim.as_ref().unwrap(), &format!("scene t{t}")); }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        dump(sim, "after the scene");
        assert!(sim.drones.scripts.players.iter().any(|p| p.scripts.contains(&0x0600_0073) && p.state >= 3),
            "the scene ran (probe precondition)");
    }
    // 40 more ticks: the door's channel arm closes it when the player is clear.
    for _ in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        dump(sim, "settled");
    }
    // Walk up to the closed door #251 and press E (the player's own use): refused with the hit
    // sound [M Door_Activate 0x135be0 LAB_00135cf8] — or does it open (the port symptom)?
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [85.5, 43.0, -37.0];
        sim.bond.yaw = 0.0; // face +z? the door is at z −35.12, north of −37 → +z is toward it? see the dump
    }
    for t in 0..90 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, t % 30 == 0);
        if t % 30 == 0 { dump(&scene.sim.as_ref().unwrap(), &format!("E press t{t}")); }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        dump(sim, "after E");
    }
}
