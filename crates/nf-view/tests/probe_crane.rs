//! probe: the 0700000d crane route, leg by leg, printed only (opt-in NF_ISO, --ignored).
//! Route to find: the dock → ladder #435 → the rotate crane #492 (E #450) → wire #271 →
//! the shipping crane beam (E #460) → the yard. Each leg prints Bond's position so the
//! next window can route the walk. No assertions.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

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

fn teleport(w: &mut Walker, x: f32, y: f32, z: f32) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [x, y, z];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[probe] teleport settled at {:?}", pos(w));
}

#[test]
#[ignore = "probe: the beam surfaces and the ledge ride; run with --ignored --nocapture"]
fn probe_the_beam_and_ledge() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The beam at rest: ray down its footprint.
        for x in [135.0f32, 137.5, 140.0] {
            for z in [-272.0f32, -280.0, -290.0, -300.0] {
                let hit = sim.world.segment_hit([x, 18.0, z], [x, 0.0, z]);
                let y = hit.map(|(t, _)| 18.0 - 18.0 * t);
                eprintln!("[probe] beam-rest ray ({x},{z}) -> {y:?}");
            }
        }
        // The ledge fine map at z -268..-272, x 160..190.
        for z in [-268.0f32, -270.0, -272.0] {
            let mut row = String::new();
            for x in [160.0f32, 164.0, 168.0, 172.0, 176.0, 180.0, 184.0, 186.0, 188.0] {
                let hit = sim.world.segment_hit([x, 15.0, z], [x, 0.0, z]);
                let y = hit.map(|(t, _)| 15.0 - 15.0 * t);
                row.push_str(&format!("({x:.0},{:.1}) ", y.unwrap_or(-1.0)));
            }
            eprintln!("[probe] ledge z {z}: {row}");
        }
    }
    // E the plate from the dock, then watch the beam pass the ledge.
    teleport(&mut w, 189.0, 10.0, -287.6);
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E #460");
    w.wait(5);
    eprintln!("[probe] E ok {used} ch {:}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b));
    // Stand on the ledge (x 176, z -270) and watch the beam arrive.
    teleport(&mut w, 176.0, 9.0, -270.0);
    w.wait(30);
    eprintln!("[probe] ledge stand {:?}", pos(&w));
    for k in 0..14 {
        w.wait(120);
        eprintln!("[probe] ledge-watch t{} bond {:?}", (k + 1) * 120, pos(&w));
    }
    // Jump onto the beam mid-ride: teleport onto the beam's flange and ride west.
    teleport(&mut w, 174.0, 8.8, -280.0);
    w.wait(20);
    eprintln!("[probe] board attempt {:?}", pos(&w));
    for k in 0..20 {
        w.wait(120);
        eprintln!("[probe] ride t{} bond {:?}", (k + 1) * 120, pos(&w));
    }
}

#[test]
#[ignore = "probe: the dock/ledge floor map; run with --ignored --nocapture"]
fn probe_the_dock_floor_map() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_09ef, 1));
        eprintln!("[probe] beam rest matrix {:?}", m.map(|m| [m[3][0], m[3][1], m[3][2]]));
    }
    // Ray-down floor map over the crane area (static + dynamic): x 168..196, z -262..-296.
    let sim = w.scene.sim.as_ref().unwrap();
    for z in [-262.0f32, -266.0, -270.0, -274.0, -278.0, -282.0, -286.0, -290.0, -294.0] {
        let mut row = String::new();
        for x in [168.0f32, 172.0, 176.0, 180.0, 184.0, 188.0, 192.0, 196.0] {
            let hit = sim.world.segment_hit([x, 18.0, z], [x, 0.0, z]);
            match hit {
                Some((t, n)) => {
                    let y = 18.0 + (0.0 - 18.0) * t;
                    row.push_str(&format!("({x:.0},{y:.1}) "));
                }
                None => row.push_str(&format!("({x:.0},--) ")),
            }
        }
        eprintln!("[probe] z {z}: {row}");
    }
}

#[test]
#[ignore = "probe: the shipping-crane ride; run with --ignored --nocapture"]
fn probe_the_shipping_ride() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for h in [0x0200_0a80u32, 0x0200_0a81, 0x0200_0a9c] {
            if let Some((lo, hi)) = sim.glist_bounds.get(&h) {
                eprintln!("[probe] glist {h:08x} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
            } else { eprintln!("[probe] glist {h:08x}: no bounds"); }
        }
    }
    teleport(&mut w, 189.0, 10.0, -287.6);
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E the shipping touchplate #460");
    w.wait(5);
    let ch_2b = w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b);
    eprintln!("[probe] E #460 ok {used} ch0x2b {ch_2b} bond {:?}", pos(&w));
    // Track the beam entity's world origin every 100 ticks.
    for k in 0..16 {
        w.wait(100);
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_09ef, 1));
        let e = m.map(|m| [m[3][0], m[3][1], m[3][2]]);
        eprintln!("[probe] t{} beam {:?} bond {:?}", (k + 1) * 100, e, pos(&w));
    }
}

#[test]
#[ignore = "probe: prints the crane route legs; run with --ignored --nocapture"]
fn probe_the_crane_route() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (c, top, bot) in walker::ladders(sim) {
            if c[0] > 140.0 && c[0] < 230.0 { eprintln!("[probe] ladder at ({:.2},{:.2},{:.2}) top {top:.2} bottom {bot:.2}", c[0], c[1], c[2]); }
        }
        for pl in sim.drones.scripts.players.iter() {
            eprintln!("[probe] SP #{} state {} cur script {:08x}", pl.placement, pl.state, pl.scripts[pl.cur]);
        }
    }
    // Leg A: the dock near the shipping-crane touchplate, then the ladder to the crane.
    teleport(&mut w, 189.0, 10.0, -287.6);
    // Climb ladder #435 (212.22, 8.14, -234.51) to the crane cab level (~13.3).
    let climbed = w.try_walk_to(Wp::at(212.2, 14.6, -234.5, Act::Ladder { top_y: 14.4 }), "climb ladder #435 to the crane");
    eprintln!("[probe] leg A climb ok {climbed} bond at {:?}", pos(&w));
    // Leg B: walk to the rotate crane's cab platform and E the touchplate #450.
    let to_cab = w.try_walk_to(Wp::at(200.5, 13.4, -262.4, Act::None), "walk to the crane cab");
    eprintln!("[probe] leg B cab ok {to_cab} bond at {:?}", pos(&w));
    // Stand on the arm and press E on #450 (199.53, 13.31, -263.81).
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E the rotate touchplate #450");
    w.wait(10);
    let ch_a = w.scene.sim.as_ref().unwrap().drones.channels.on(0xa);
    eprintln!("[probe] leg B E ok {used} ch0xa {ch_a} bond at {:?}", pos(&w));
    // Ride: hold still, print every 60 ticks for 900 ticks.
    for k in 0..15 {
        w.wait(60);
        eprintln!("[probe] ride t{} bond {:?} ch0xa {}", (k + 1) * 60, pos(&w), w.scene.sim.as_ref().unwrap().drones.channels.on(0xa));
    }
    // Leg C: the wire #271 (206.93, 17.04, -246.13).
    let to_wire = w.try_walk_to(Wp::at(206.9, 17.1, -246.1, Act::WireRide), "walk to wire #271 and ride");
    eprintln!("[probe] leg C wire ok {to_wire} bond at {:?}", pos(&w));
    for _ in 0..6 { w.wait(60); eprintln!("[probe] after wire bond {:?}", pos(&w)); }
    // Leg D: the shipping crane beam — teleport onto its east end and E #460.
    teleport(&mut w, 177.0, 11.2, -287.1);
    let used2 = w.try_walk_to(Wp::at(189.0, 9.9, -288.8, Act::Use { use_ticks: 240 }), "E the shipping touchplate #460");
    w.wait(10);
    let ch_2b = w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b);
    eprintln!("[probe] leg D E ok {used2} ch0x2b {ch_2b} bond at {:?}", pos(&w));
    for k in 0..20 {
        w.wait(120);
        eprintln!("[probe] shipping t{} bond {:?}", (k + 1) * 120, pos(&w));
    }
}
