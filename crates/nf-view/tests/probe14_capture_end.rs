//! level-14-capture-placement probe: fire the kiko scene (SP #382, 0x0600062F) and the capture
//! scene (SP #407, 0x06000848) through their real trigger touches, and print Bond's position and
//! the script entity objects after each scene end. Assertion-free diagnostic.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_capture_scene_end_positions() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    let free = |sim: &mut nf_view::scene::Sim, ticks: u32| -> bool {
        for _ in 0..ticks { sim.step_bond(nf_game::player::Input::default(), false, false, false); if !sim.disabled { return true; } }
        !sim.disabled
    };
    let dump = |sim: &nf_view::scene::Sim, tag: &str| {
        let b = &sim.bond;
        eprintln!("[probe] {tag}: bond ({:.2},{:.2},{:.2}) disabled {} grounded {}", b.pos[0], b.pos[1], b.pos[2], sim.disabled, b.grounded);
        for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
            if let Some(nf_game::nis::Obj::Entity { hash, matrix, .. }) = o {
                eprintln!("[probe] {tag}: entity obj {i} hash {hash:08x} t=({:.2},{:.2},{:.2})",
                    matrix[3][0], matrix[3][1], matrix[3][2]);
            }
        }
        for (pi, p) in sim.drones.scripts.players.iter().enumerate() {
            eprintln!("[probe] {tag}: script player {pi} placement {} cur script {:08x} playing {}", p.placement, p.scripts[p.cur], p.state);
        }
    };
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    dump(sim, "boot");
    // Stage 1 (walk test): the vent drop.
    sim.bond.pos = [40.7, 2.6, 39.0];
    let mut ch47 = false;
    for _ in 0..1700u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if sim.drones.channels.on(0x47) { ch47 = true; break; }
    }
    eprintln!("[probe] vent scene ch47 {ch47}");
    dump(sim, "after vent scene");
    free(sim, 300);
    // Stage 2: Kiko's walk — TouchOnce #381.
    sim.bond.pos = [-49.2, 0.9, 38.4];
    let mut ch48 = false;
    for _ in 0..1500u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if sim.drones.channels.on(0x48) { ch48 = true; break; }
    }
    eprintln!("[probe] kiko scene ch48 {ch48}");
    dump(sim, "after kiko scene (event 6 = the hash-0 marker entity)");
    free(sim, 300);
    // Stage 4: the capture — the touch #456 with 0x8C off here (the probe does not sabotage);
    // fire SP #407 directly instead: its trigger channel is 9 (SEQ_AND #445 [0x8C, 0x0C] → 0x09).
    // Raise 0x09 the way the game's SEQ_AND would to reach the scene on the scripted path.
    sim.drones.channels.set(0x09, 1, sim.drones.frame);
    sim.bond.pos = [-33.1, 0.9, 26.8];
    let mut ch4a = false;
    for _ in 0..1200u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if sim.drones.channels.on(0x4a) { ch4a = true; break; }
    }
    eprintln!("[probe] capture scene ch4a {ch4a}");
    dump(sim, "after capture scene");
    free(sim, 300);
    dump(sim, "after capture scene + control");
}
