//! PROBE (level-0c-plates-reach): the game's own route from the display ground toward the
//! secret-door plates. The route is mapped and WALKED (real input) from the display ground to
//! the 2.37 pocket — the ONE link this window could not close is the pocket→yard descent
//! (the slab face z −155.77 spans x −4.5..15.25 at y 2.37..6.15 [D coll-scan], and the
//! pocket's west edge (x −1.8) hangs over void). #[ignore]d until the descent is mapped:
//! a failing probe must not go in the suite's pass/fail set (it is a route record, not a pin).
use walker::{Walker, Wp};

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The crouch toggle (the walker lacks Act::CrouchOn — the walkreal_07000014 pattern).
fn set_crouch(w: &mut Walker, on: bool) {
    w.scene.sim.as_mut().unwrap().bond.crouched = on;
    for _ in 0..15 { w.tick(nf_game::player::Input::default(), false); }
}

#[test]
#[ignore = "the route record stops at the 2.37 pocket: the pocket→yard descent is the one unmapped link (the walk_to off the pocket edge wedges at the slab face z −155.77) — re-enable when the descent is mapped"]
fn probe_the_low_route_to_the_plates() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    // Leg-3's end state (the walk's real leg 3 ends here on the display ground).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.4, 7.35, -113.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    // 1. East along the display ground's known-good lane, then south to the stair's west
    //    flank and the south edge (the shaft is x 11.5..12.3, east of this lane).
    w.walk_to(Wp::new(10.5, 7.4, -113.4).r(0.9), "the display ground west [D the walk's leg 3]");
    w.walk_to(Wp::new(10.5, 7.0, -115.8).r(0.8), "to the south edge (z ≈ −116.5 [D probe])");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] at the edge: grounded {} pos {:?}", sim.bond.grounded, sim.bond.pos);
    }
    // 2. The proven crouch walk-off: the walker steers but won't cross a floor edge, so the
    //    walk-off is raw forward ticks (the crossing session's probe shape), then the walker
    //    resumes on the lane.
    set_crouch(&mut w, true);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let yaw = (10.5f32 - sim.bond.pos[0]).atan2(-121.0f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(yaw - sim.bond.yaw);
        drop(sim);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
    }
    let mut landed_lane = false;
    for _ in 0..420 {
        w.tick(nf_game::player::Input { forward: 1.0, crouch: true, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.pos[1] < 4.0 && sim.bond.pos[2] < -118.0 { landed_lane = true; break; }
    }
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    set_crouch(&mut w, false);
    assert!(landed_lane, "the crouch walk-off must land on the 2.18 lane, got {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] on the lane: grounded {} pos {:?}", sim.bond.grounded, sim.bond.pos);
        assert!(sim.bond.grounded && sim.bond.pos[1] < 4.0, "landed on the 2.18 room, got {:?}", sim.bond.pos);
    }
    // 3. West-south through the storage room (one 2.18 floor [D room_storage01]).
    w.walk_to(Wp::new(8.0, 3.3, -127.0).r(1.2), "the room south-west 1");
    w.walk_to(Wp::new(5.0, 3.3, -132.0).r(1.2), "the room west 2");
    w.walk_to(Wp::new(2.0, 3.3, -140.0).r(1.2), "the room west 3");
    w.walk_to(Wp::new(1.0, 3.3, -147.5).r(1.0), "the room south end");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] at the room's south edge: grounded {} pos {:?}", sim.bond.grounded, sim.bond.pos);
        assert!(sim.bond.grounded, "the storage room walk stays grounded");
    }
    // 4. The 2.37 pocket south of the storage room (a floor at 2.37 continues to z ≈ −155.7;
    //    the slab face spans x 2.5..15.25 only, so at x ≈ 1.2 the pocket ends in the drop to
    //    the yard).
    w.walk_to(Wp::new(1.2, 2.6, -153.5).r(1.0), "the pocket south 1");
    // 4b. THE DESCENT TRY: run west off the pocket's west edge (x −1.8) with a jump — the
    //     yard floor (y ≈ 0) spans x −5.65..−0.34 at z −158.5..−162.17 [D coll-scan]; the
    //     fall needs ~4 units of south drift. Print where the body actually lands.
    {
        w.walk_to(Wp::new(-1.3, 3.4, -154.0).r(0.6), "the pocket's west edge");
        let sim = w.scene.sim.as_ref().unwrap();
        let yaw = (-4.5f32 - sim.bond.pos[0]).atan2(-159.5f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(yaw - sim.bond.yaw);
        drop(sim);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
        let mut landed = None;
        for i in 0..300 {
            w.tick(nf_game::player::Input { forward: 1.0, jump: if i == 6 { true } else { false }, ..Default::default() }, false);
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.grounded && sim.bond.pos[1] < 3.0 {
                landed = Some((sim.bond.pos, sim.bond.fall_damage));
                break;
            }
        }
        eprintln!("[probe] the west-edge jump-off landed at {:?} (fall_damage {:?})",
            landed.as_ref().map(|(p, _)| *p), landed.as_ref().map(|(_, d)| *d));
    }
    // 5. The yard: south then east over the bridge into the corridor.
    w.walk_to(Wp::new(1.5, 0.9, -163.0).r(1.2), "the yard south 1");
    w.walk_to(Wp::new(1.7, 0.9, -172.0).r(1.2), "the yard south 2");
    w.walk_to(Wp::new(4.0, 0.9, -179.5).r(1.2), "the yard south-east");
    w.walk_to(Wp::new(8.5, 0.9, -179.0).r(1.2), "the bridge west");
    w.walk_to(Wp::new(14.5, 0.9, -175.0).r(1.2), "the bridge east");
    for (x, y, z, label) in [
        (20.0f32, 0.9f32, -174.0f32, "the corridor 1"),
        (27.0, 0.9, -174.0, "the corridor 2"),
        (34.0, 0.9, -174.0, "the corridor 3"),
        (41.0, 1.2, -174.0, "the corridor 4 (the 0.38 step)"),
        (47.0, 1.2, -174.0, "the corridor 5"),
        (50.5, 1.5, -173.8, "the strip approach"),
        (51.6, 1.5, -173.6, "the strip at the door"),
    ] {
        w.walk_to(Wp::new(x, y, z).r(0.9), label);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] on the strip: grounded {} pos {:?}", sim.bond.grounded, sim.bond.pos);
        assert!(sim.bond.grounded, "the strip stand is grounded");
        eprintln!("[probe] ch2 {} ch3 {} ch8 {}", sim.drones.channels.on(2),
            sim.drones.channels.on(3), sim.drones.channels.on(8));
    }
    // 6. Face east and E the plates (the use sphere reaches the box from the strip).
    for i in 0..240 {
        w.tick(nf_game::player::Input { turn: 0.05, ..Default::default() }, i % 16 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(8) {
            eprintln!("[probe] ch 8 rose at tick {i}");
            break;
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[probe] final: pos {:?} ch2 {} ch3 {} ch8 {}", sim.bond.pos,
        sim.drones.channels.on(2), sim.drones.channels.on(3), sim.drones.channels.on(8));
    let door = sim.drones.sss.list.iter().find(|s| s.channel == 8);
    eprintln!("[probe] door #544 state {:?} pos {:?}", door.map(|d| d.state), door.map(|d| d.pos()));
    assert!(sim.drones.channels.on(8), "the plates' AND must raise ch 8 from the real route");
}

#[test]
fn probe_the_nav_web_across_the_west_side() {
    let iso = iso();
    let w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    let sim = w.scene.sim.as_ref().unwrap();
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        let in_band = |pos: [f32; 3]| pos[2] < -140.0 && pos[2] > -195.0 && pos[0] > -16.0 && pos[0] < 60.0;
        let band: Vec<usize> = (0..p.nodes.len()).filter(|&i| in_band(p.nodes[i].pos)).collect();
        if band.len() < 2 { continue; }
        eprintln!("[nav] path {pi} name {:?} kind {} nodes {} band {}", p.name, p.kind, p.nodes.len(), band.len());
        // Links with one end in the band and the other OUTSIDE it (z > −140 or x > 60 or x < −16):
        // the connections in/out of the west-side low band.
        for l in &p.links {
            let (a, b) = (l.a as usize, l.b as usize);
            if a >= p.nodes.len() || b >= p.nodes.len() { continue; }
            let (pa, pb) = (p.nodes[a].pos, p.nodes[b].pos);
            let (ia, ib) = (in_band(pa), in_band(pb));
            if ia != ib {
                eprintln!("[nav]   cross link {} {:?} ({:.1},{:.1},{:.1}) <-> {} ({:.1},{:.1},{:.1})",
                    l.a, if ia {"in"} else {"OUT"}, pa[0], pa[1], pa[2], l.b, pb[0], pb[1], pb[2]);
            }
        }
        // Also the band's north-south spine: nodes sorted by z with their y, every 4th.
        let mut band = band; band.sort_by_key(|&i| (p.nodes[i].pos[2] * 10.0) as i32);
        for (k, &i) in band.iter().enumerate() {
            if k % 4 == 0 {
                let n = p.nodes[i].pos;
                eprintln!("[nav]   node {} ({:.1},{:.1},{:.1})", i, n[0], n[1], n[2]);
            }
        }
    }
}
