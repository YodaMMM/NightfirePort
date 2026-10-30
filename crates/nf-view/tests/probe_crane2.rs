//! probe2: the 0700000d crane-route geometry, printed only (opt-in NF_ISO, --ignored).
//! Everything the route needs in one run: the wires' real endpoints (the game's own
//! spherical derivation [M Player_CollWire 0x197130:111-127]), the ladders, the grapple
//! points, the two crane glists' bounds, the SPs, and a broad floor map over the dock /
//! trench / yard / east-ladder ground. No assertions.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_warm() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

fn pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

/// The game's wire line: endpoints = centre ± spherical(r − 2.0, yaw + π/2, pitch)
/// [M Player_CollWire 0x197130:111-127; the port's searchlight::spherical is
/// Vec_Spherical_2_Cartesian 0x1e3948].
fn wire_ends(centre: [f32; 3], radius: f32, yaw: f32, pitch: f32) -> ([f32; 3], [f32; 3]) {
    let s = nf_game::searchlight::spherical(radius - 2.0, yaw + std::f32::consts::FRAC_PI_2, pitch);
    let a = [centre[0] - s[0], centre[1] - s[1], centre[2] - s[2]];
    let b = [centre[0] + s[0], centre[1] + s[1], centre[2] + s[2]];
    (a, b)
}

#[test]
#[ignore = "probe2: the crane geometry dump; run with --ignored --nocapture"]
fn probe2_the_geometry() {
    let w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("== wires ==");
        for (i, wref) in sim.wires.iter().enumerate() {
            let (a, b) = wire_ends(wref.centre, wref.radius, wref.yaw, wref.pitch);
            eprintln!("[probe2] wire#{i} centre ({:.2},{:.2},{:.2}) r {:.2} yaw {:+.3} pitch {:+.3} type {} p0 {}",
                wref.centre[0], wref.centre[1], wref.centre[2], wref.radius, wref.yaw, wref.pitch, wref.wire_type, wref.param0);
            eprintln!("[probe2]   A ({:.2},{:.2},{:.2})  B ({:.2},{:.2},{:.2})", a[0], a[1], a[2], b[0], b[1], b[2]);
        }
        eprintln!("== ladders ==");
        for (i, l) in sim.ladders.iter().enumerate() {
            eprintln!("[probe2] ladder#{i} centre ({:.2},{:.2},{:.2}) r {:.2} yaw {:+.3} ch {} half {:?}",
                l.centre[0], l.centre[1], l.centre[2], l.radius, l.yaw, l.channel, l.half);
        }
        eprintln!("== glists ==");
        for h in [0x0200_0a80u32, 0x0200_0a81, 0x0200_0a9c] {
            match sim.glist_bounds.get(&h) {
                Some((lo, hi)) => eprintln!("[probe2] glist {h:08x} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]),
                None => eprintln!("[probe2] glist {h:08x}: no bounds"),
            }
        }
        eprintln!("== script players ==");
        for pl in sim.drones.scripts.players.iter() {
            eprintln!("[probe2] SP #{} state {} script {:08x}",
                pl.placement, pl.state, pl.scripts[pl.cur]);
        }
        // The beam entity's rest matrix.
        if let Some(m) = sim.drones.scripts.entity_matrix((0x0600_09ef, 1)) {
            eprintln!("[probe2] beam rest origin ({:.2},{:.2},{:.2})", m[3][0], m[3][1], m[3][2]);
        }
        if let Some(m) = sim.drones.scripts.entity_matrix((0x0600_0a08, 1)) {
            eprintln!("[probe2] rotate rest origin ({:.2},{:.2},{:.2})", m[3][0], m[3][1], m[3][2]);
        }
    }
}

#[test]
#[ignore = "probe2: the floor map; run with --ignored --nocapture"]
fn probe2_the_floor_map() {
    let w = load_warm();
    let sim = w.scene.sim.as_ref().unwrap();
    // The dock / trench / yard band, fine in z.
    eprintln!("== floor map A: x 128..200, z -262..-300 ==");
    for z in [-262.0f32, -265.0, -268.0, -271.0, -274.0, -278.0, -282.0, -286.0, -290.0, -294.0, -298.0] {
        let mut row = String::new();
        for x in [128.0f32, 132.0, 136.0, 140.0, 144.0, 148.0, 152.0, 156.0, 160.0, 164.0, 168.0, 172.0, 176.0, 180.0, 184.0, 188.0, 192.0, 196.0, 200.0] {
            let hit = sim.world.segment_hit([x, 20.0, z], [x, 0.0, z]);
            match hit {
                Some((t, _)) => { let y = 20.0 - 20.0 * t; row.push_str(&format!("{y:>5.1} ")); }
                None => row.push_str("   -- "),
            }
        }
        eprintln!("[probe2] z {z:>6.1}: {row}");
    }
    eprintln!("(columns x = 128,132,...,200)");
    // The east strip: the ladder ground (#374 x 217.9 z -227.3; #435 x 212.5 z -234.5).
    eprintln!("== floor map B: x 200..224, z -226..-268 ==");
    for z in [-226.0f32, -232.0, -238.0, -244.0, -250.0, -256.0, -262.0, -268.0] {
        let mut row = String::new();
        for x in [200.0f32, 204.0, 208.0, 212.0, 216.0, 220.0, 224.0] {
            let hit = sim.world.segment_hit([x, 22.0, z], [x, 0.0, z]);
            match hit {
                Some((t, _)) => { let y = 22.0 - 22.0 * t; row.push_str(&format!("{y:>5.1} ")); }
                None => row.push_str("   -- "),
            }
        }
        eprintln!("[probe2] z {z:>6.1}: {row}");
    }
    eprintln!("(columns x = 200,204,...,224)");
    // The warehouse band south to the dock's north edge.
    eprintln!("== floor map C: x 168..204, z -206..-262 ==");
    for z in [-206.0f32, -214.0, -222.0, -230.0, -238.0, -246.0, -254.0, -262.0] {
        let mut row = String::new();
        for x in [168.0f32, 176.0, 184.0, 192.0, 200.0, 204.0] {
            let hit = sim.world.segment_hit([x, 20.0, z], [x, 0.0, z]);
            match hit {
                Some((t, _)) => { let y = 20.0 - 20.0 * t; row.push_str(&format!("{y:>5.1} ")); }
                None => row.push_str("   -- "),
            }
        }
        eprintln!("[probe2] z {z:>6.1}: {row}");
    }
    eprintln!("(columns x = 168,176,184,192,200,204)");
}

/// Real-input route attempt: dock → rotate-cab → E #450 → ride → wire #271 → the beam.
/// Print every leg's outcome and position; no teleports after the dock drop-in (the
/// walkreal's own documented stand-in handles getting TO the dock in the full test).
#[test]
#[ignore = "probe2: the real-input route attempt; run with --ignored --nocapture"]
fn probe2_the_route_walk() {
    let mut w = load_warm();
    // Drop in on the dock (the stand-in equivalent; the walkreal walks this part).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [189.0, 9.9, -287.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    eprintln!("[probe2] dock {:?}", pos(&w));

    // Leg A: E the shipping touchplate #460 first (ch 0x2b starts the beam shuttle).
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -288.7, walker::Act::Use { use_ticks: 240 }), "E #460");
    w.wait(5);
    eprintln!("[probe2] E #460 ok {used} ch0x2b {} at {:?}",
        w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b), pos(&w));

    // Leg B: walk north-east off the dock toward the y 12.1 surface and the rotate cab.
    let up = w.try_walk_to(Wp::new(200.5, 13.4, -262.4), "walk to the rotate cab");
    eprintln!("[probe2] cab walk ok {up} at {:?}", pos(&w));

    // Leg C: E the rotate touchplate #450 (arms ch 0xa + the wire).
    let used2 = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, walker::Act::Use { use_ticks: 240 }), "E #450");
    w.wait(5);
    eprintln!("[probe2] E #450 ok {used2} ch0xa {} at {:?}",
        w.scene.sim.as_ref().unwrap().drones.channels.on(0xa), pos(&w));

    // Leg D: hold still through the rotate (750 ticks) and print.
    for k in 0..13 {
        w.wait(60);
        eprintln!("[probe2] rotate t{} bond {:?}", (k + 1) * 60, pos(&w));
    }

    // Leg E: walk to the wire #271 grab and ride.
    let wire = w.try_walk_to(Wp::at(206.9, 17.1, -246.1, walker::Act::WireRide), "wire #271");
    eprintln!("[probe2] wire ok {wire} at {:?}", pos(&w));
    for _ in 0..10 { w.wait(60); eprintln!("[probe2] after-wire bond {:?}", pos(&w)); }
}
