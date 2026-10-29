//! level-12-audit: the ScriptPlayers' channel writes on 07000012, through the real load path
//! (`nf_view::load`). Pins `Script_EventHandler` 0x1d94c0 case 0xd (`switch_channels[p0] = p1`)
//! against the disc's own script bytes [D dump_07000012_script_events]:
//! - the skybridge intro (SP #660, script 0x06000530, mode 0x40 preloaded) writes ch 0x3c from its
//!   stream 8 (the intro plays from the load — the mode-0x40 SP runs from the mount, the fusebox
//!   wire precedent [M 0x1dbc08:46-54]);
//! - the shaft-fall death (SP #767, script 0x0600098a, trigger ch 0x7C [D param 6]) writes ch 0x62
//!   (the death restart) once its trigger rises — stream 6's event 0xD [D].
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load() -> nf_view::scene::Scene {
    nf_view::load(iso().expect("NF_ISO").as_str(), "07000012.bin", false).expect("load 07000012")
}

#[test]
fn the_parked_intro_sp_writes_nothing_until_its_activation_beat() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load();
    let sim = scene.sim.as_mut().unwrap();
    let idle = Input::default();
    // SP #660's mask word p5 = 2 [D census]: `SP_CreateScriptPlayer` parks a mask != 0 SP in
    // state 0 [M 0x1db9a8:74-86 — the 07000006 #808 precedent, nis.rs fix-searchlight-alarm],
    // so the intro does NOT run from the load and its stream-8 ch 0x3c write never fires head-
    // lessly. The activation beat (SP_Hit/the level-start path) is not derivable headlessly —
    // the same open live check as 07000006's courtyard boot.
    let mut seen = false;
    for _ in 0..700 {
        sim.step_bond(idle, false, false, false);
        if sim.drones.channels.on(0x3c) { seen = true; break; }
    }
    assert!(!seen, "the mask-parked intro (SP #660, p5=2) writes no channel before activation [M 0x1db9a8:74-86]");
    assert!(!sim.drones.channels.on(0x89), "the sub-script's 0x89 pulse is parked with it [D 06000A25]");
}

#[test]
fn the_shaft_fall_sp_writes_the_death_channel_on_its_trigger() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load();
    let sim = scene.sim.as_mut().unwrap();
    let idle = Input::default();
    for _ in 0..60 { sim.step_bond(idle, false, false, false); }
    assert!(!sim.drones.channels.on(0x62), "sister: the death channel is off before the fall [D]");
    // The trigger plates #788-791 write ch 0x7C [D the level doc §4]; raise it the way they do.
    sim.drones.channels.set_script(0x7c, 1, 0);
    let mut seen = false;
    for _ in 0..200 {
        sim.step_bond(idle, false, false, false);
        if sim.drones.channels.on(0x62) { seen = true; break; }
    }
    assert!(seen, "SP #767 (trigger ch 0x7C) runs 0x0600098a whose stream 6 writes ch 0x62 [M 0x1d94c0 case 0xd; D the script bytes]");
}
