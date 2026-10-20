//! level-13 gate probe 3: the fake-lift switch's E sphere, the E → ch 0x23 chain, and the
//! Shutters2 SS pose response. Opt-in NF_ISO; prints only.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_07000013_gate_chain() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    // SP #161's model object (hash 0x020005E2): the E target's world sphere [D the level parse].
    let spot = {
        use nf_formats::iso9660::IsoImage;
        let img = IsoImage::open(&iso).unwrap();
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000013.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let lv = nf_formats::level::Level::parse(&data).unwrap();
        for h in [0x0200_05e2u32, 0x0200_05eb, 0x0200_05ec] {
            if let Some(&(m, o)) = lv.by_hash.get(&h) {
                let obj = &lv.maps[m].objects[o];
                eprintln!("[model {h:#x}] obj {o} \"{}\" centre {:?} r {} min {:?} max {:?}",
                    obj.name.trim(), obj.centre, obj.radius, obj.min, obj.max);
            } else { eprintln!("[model {h:#x}] not in by_hash"); }
        }
        lv.by_hash.get(&0x0200_05e2).map(|&(_, o)| lv.maps[lv.main_map().unwrap()].objects[o].centre).unwrap_or([0.0; 3])
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000013.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (hash, nis) in &sim.drones.scripts.lib {
            if *hash == 0x0600_0959 {
                for (si, st) in nis.streams.iter().enumerate() {
                    eprintln!("[959] stream {si}: first {} last {} cmds {}", st.first_key, st.last_key, st.commands.len());
                    for c in st.decode() { eprintln!("    {c:?}"); }
                }
            }
        }
        for p in &sim.drones.scripts.players {
            eprintln!("[sp] placement {} modes {:?} mask {:#x} out {:#x} state {}", p.placement, p.modes, p.mask, p.out, p.state);
        }
        for s in &sim.drones.sss.list {
            eprintln!("[ss] placement {} ch {:#x} t ({:.2},{:.2},{:.2})",
                s.placement, s.channel, s.matrix[3][0], s.matrix[3][1], s.matrix[3][2]);
        }
    }
    // What supports the spawn tile before the E?
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.world.tris_near([0.0, -371.0, 9.0], [4.5, -368.5, 13.5]);
        eprintln!("[floor pre-E] {} tris", t.len());
    }
    // Control 1: the spawn, NO E. Control 2: the panel area, NO E.
    for (label, pos) in [("spawn", [2.16f32, -369.1, 11.0]), ("panel", [-19.49, -369.1, 23.9])] {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = pos;
            sim.bond.vel = [0.0; 3];
        }
        for k in 0..5 {
            w.wait(40);
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[noE {label}] +{} y {:.2}", (k + 1) * 40, sim.bond.pos[1]);
        }
    }
    // The E targets are the preload's entities (0x02000A2C "SCRIPT_SHUTTER_CLOSE") at the west
    // wall panel [D script 0x06000959 stream 0; obj 70's glist]. Try stands around it.
    let mut ch23 = false;
    'outer: for stand in [[-19.49f32, -369.1, 23.9], [-19.49, -369.1, 24.0], [-18.08, -369.1, 23.9],
                          [-19.49, -369.1, 25.9], [-17.0, -369.1, 25.0], [-19.49, -369.1, 25.0]] {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = stand;
            sim.bond.yaw = 0.0;
            sim.bond.pitch = 0.0;
            sim.bond.vel = [0.0; 3];
        }
        for _ in 0..40 {
            w.tick(nf_game::player::Input::default(), true);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break 'outer; }
        }
    }
    eprintln!("[chain] E at the lift doors → ch 0x23: {ch23}");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.world.tris_near([0.0, -382.0, 9.0], [4.5, -368.5, 13.5]);
        eprintln!("[floor post-E] {} tris (deep box)", t.len());
        for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
            if let Some(nf_game::nis::Obj::Entity { kind, matrix, src, hash, .. }) = o {
                eprintln!("[obj {i}] entity kind {kind:#x} hash {hash:#x} src {src:?} t ({:.2},{:.2},{:.2})",
                    matrix[3][0], matrix[3][1], matrix[3][2]);
            }
        }
    }
    for k in 0..6 {
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(s) = sim.drones.sss.list.iter().find(|s| s.channel == 0x23) {
            eprintln!("[gate] +{} ticks: SS t ({:.2},{:.2},{:.2}) state {:#x} rows [{:.2?} {:.2?} {:.2?}]",
                (k + 1) * 30, s.matrix[3][0], s.matrix[3][1], s.matrix[3][2], s.state,
                s.matrix[0], s.matrix[1], s.matrix[2]);
        } else { eprintln!("[gate] no SS on ch 0x23"); }
    }
    // The walk south: can Bond pass z 65 now?
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "the lobby east lane");
    w.walk_to(Wp::new(3.0, -368.4, 45.0).r(1.5), "the lobby mid");
    w.walk_to(Wp::new(-6.0, -368.2, 60.0).r(1.5), "the lobby south");
    let n = w.try_walk_to(Wp::new(-6.2, -368.0, 66.3).r(0.6), "through the gate");
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[walk] at ({:.2},{:.2},{:.2}) reached={}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], n);
    eprintln!("[walk] ch50={} ch64={}", sim.drones.channels.on(0x50), sim.drones.channels.on(0x64));
}
