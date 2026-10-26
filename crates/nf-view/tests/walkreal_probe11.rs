//! walkreal probe for 07000001: the meet-hall west wall region where Bond's walk stalls at
//! (1.38, 6.7, 11.61) with the contact parked at (6.9, 6.3, 9.1) state 0x20. Static geometry +
//! nav graph + breakables + doors around the stall. Diagnostic only — run with --nocapture.

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_07000001_west_wall() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    // The static geometry does not depend on the fight: probe straight after the intro tail.
    for _ in 0..4200u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();

    eprintln!("=== the meet-hall west wall region ===");
    eprintln!("bond at ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);

    // 1. Ray fan eastward from the west corridor: where does the wall have gaps?
    //    (hip height y=6.6 and knee height y=6.0, from x=1.0 to x=4.0 at z = 6..16 step 0.5)
    eprintln!("--- ray fan: from (0.8,y,z) to (4.2,y,z) — eastbound through the wall plane x~1.9-2.9 ---");
    for z in [6.0f32, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.5, 10.0, 10.5, 11.0, 11.5, 12.0, 12.5, 13.0, 13.5, 14.0, 14.5, 15.0, 15.5, 16.0, 16.5, 17.0] {
        let mut row = String::new();
        for &y in &[6.1f32, 6.6] {
            let hit = sim.world.segment([0.8, y, z], [4.2, y, z]);
            row.push_str(&format!(" y{y}={}", match hit { Some(t) => format!("{t:.2}"), None => "----".into() }));
        }
        eprintln!("z {z:4.1}:{row}");
    }
    eprintln!("--- ray fan westbound from inside the hall: (4.0,y,z) to (0.5,y,z) ---");
    for z in [6.0f32, 8.0, 9.0, 9.7, 10.5, 11.6, 12.5, 13.5, 14.5, 15.5, 16.5] {
        let hit = sim.world.segment([4.0, 6.6, z], [0.5, 6.6, z]);
        eprintln!("z {z:4.1}: t={}", match hit { Some(t) => format!("{t:.2} (hit x {:.2})", 4.0 + t * (0.5 - 4.0)), None => "clear".into() });
    }

    // 2. Where can Bond stand west of the wall — floor probes on a grid.
    eprintln!("--- floor probes x 0.5..3.5 z 6..17 (y of the floor under (x,7.0,z)) ---");
    for z in [6.0f32, 8.0, 10.0, 11.6, 12.5, 14.0, 15.0, 16.0, 17.0] {
        let mut row = String::new();
        for x in [0.5f32, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5] {
            let f = sim.world.floor_below([x, 7.0, z], 6.0);
            row.push_str(&format!(" ({x:.1},{f:?})"));
        }
        eprintln!("z {z:4.1}:{row}");
    }

    // 2b. Fine z-scan of the wall plane at the corridor side (x 1.0 → 3.5, y 6.6): the openings'
    //     exact z extents.
    eprintln!("--- fine z-scan eastbound (1.0,6.6,z)->(3.5,6.6,z), step 0.25 ---");
    let mut z = 5.0f32;
    while z <= 17.5 {
        let hit = sim.world.segment([1.0, 6.6, z], [3.5, 6.6, z]);
        if let Some(t) = hit {
            let hx = 1.0 + t * 2.5;
            eprintln!("z {z:5.2}: hit t={t:.3} x={hx:.2}");
        }
        z += 0.25;
    }

    // 2c. The AI's boundary walls near the region (does the AI's own network seal the west edge?).
    eprintln!("--- nav walls with an endpoint within 9 of (2.0,6.5,11.0) ---");
    for w in &sim.nav.walls {
        let da = ((w.a[0] - 2.0).powi(2) + (w.a[2] - 11.0).powi(2)).sqrt();
        let db = ((w.b[0] - 2.0).powi(2) + (w.b[2] - 11.0).powi(2)).sqrt();
        if da < 9.0 || db < 9.0 {
            eprintln!("wall a=({:.2},{:.2},{:.2}) ha={:.1} b=({:.2},{:.2},{:.2}) hb={:.1} flags={:#x}", w.a[0], w.a[1], w.a[2], w.ha, w.b[0], w.b[1], w.b[2], w.hb, w.flags);
        }
    }

    // 3. Breakables near the region (the meeting_room_glass_doors / meeting_glass2) — by box, not
    //    matrix (a placed pane's matrix row 3 need not carry its centre).
    eprintln!("--- breakables whose box centre is within 12 of (2.0,6.5,11.0) ---");
    for (i, b) in sim.drones.breaks.list.iter().enumerate() {
        let c = [(b.lo[0] + b.hi[0]) * 0.5, (b.lo[1] + b.hi[1]) * 0.5, (b.lo[2] + b.hi[2]) * 0.5];
        let d = ((c[0] - 2.0).powi(2) + (c[1] - 6.5).powi(2) + (c[2] - 11.0).powi(2)).sqrt();
        if d < 12.0 {
            eprintln!("break[{i}] placement={} kind={} health={:.0} dead={} lo=({:.2},{:.2},{:.2}) hi=({:.2},{:.2},{:.2}) d={d:.1}",
                b.placement, b.kind, b.health, b.dead, b.lo[0], b.lo[1], b.lo[2], b.hi[0], b.hi[1], b.hi[2]);
        }
    }

    // 4. Doors (all of them — 5 doors on the level).
    eprintln!("--- doors ---");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        eprintln!("door[{i}] at=({:.2},{:.2},{:.2}) kind={} state={:#x} pos01={:.2} ch_open={:#x} ch_lock={:#x} approach={}",
            d.pos[0], d.pos[1], d.pos[2], d.kind, d.state, d.pos01, d.ch_open, d.ch_lock, d.approach);
    }

    // 5. Nav nodes near the region + their links (per path).
    eprintln!("--- nav nodes within 10 of (2.0,6.5,11.0) ---");
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        for (ni, n) in p.nodes.iter().enumerate() {
            let d3 = ((n.pos[0] - 2.0).powi(2) + (n.pos[1] - 6.5).powi(2) + (n.pos[2] - 11.0).powi(2)).sqrt();
            if d3 < 10.0 {
                let links: Vec<String> = p.links.iter()
                    .filter(|l| l.a as usize == ni || l.b as usize == ni)
                    .map(|l| format!("{}(len {:.1})", if l.a as usize == ni { l.b } else { l.a }, l.len))
                    .collect();
                eprintln!("path[{pi}] kind={:#x} node[{ni}] flags={:#x} at=({:.2},{:.2},{:.2}) links=[{}]",
                    p.kind, n.flags, n.pos[0], n.pos[1], n.pos[2], links.join(", "));
            }
        }
    }

    // 6. move_test across the wall at the sample spots (does the AI's own move check pass it?).
    eprintln!("--- move_test across the wall plane ---");
    for (a, b) in [
        ([1.38f32, 6.7, 11.61], [2.97, 6.34, 9.69]),
        ([1.38, 6.7, 11.61], [6.9, 6.3, 9.1]),
        ([1.57, 6.42, 14.46], [2.97, 6.34, 9.69]),
        ([1.0, 6.5, 14.5], [3.5, 6.4, 14.5]),
        ([1.0, 6.5, 6.3], [3.5, 6.4, 6.3]),
        ([1.57, 6.42, 14.46], [6.9, 6.3, 9.1]),
    ] {
        let (r, hit) = sim.nav.move_test(&sim.world, a, b, false, false);
        let seg = sim.world.segment(a, b);
        eprintln!("move_test {:?} -> {:?}: result {r} hit {hit:?} world_seg {seg:?}", a, b);
    }

    // 6b. The exact stall ray: t + normal (the wall face's orientation).
    {
        let (t, n) = sim.world.segment_hit_scoped(None, [1.38, 6.7, 11.61], [2.97, 6.34, 9.69])
            .expect("the stall ray must hit");
        eprintln!("stall ray (1.38,6.7,11.61)->(2.97,6.34,9.69): t={t:.4} normal=({:.2},{:.2},{:.2})", n[0], n[1], n[2]);
    }

    // 8. The isolated lane-walk repro: teleport Bond to the lane's west end (where the walk's
    //    leg 1 left him: (1.57, 6.37, 13.63)) and steer at the north-opening anchor (3.39, 14.69)
    //    exactly as walk_to does, sampling every 30 ticks. Diagnostic only (a probe may teleport).
    eprintln!("--- isolated lane walk repro ---");
    drop(sim);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [1.57, 6.37, 13.63];
        sim.bond.vel = [0.0; 3];
        // The live leg-1 end faced WEST (its final +6-tick step walked −x); start there.
        sim.bond.yaw = std::f32::consts::PI;
    }
    for t in 0..600u32 {
        let (sim_bond, err) = {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (3.39f32 - p[0]).atan2(14.69 - p[2]);
            (p, walker::wrap_pi(desired - sim.bond.yaw))
        };
        let d = ((3.39f32 - sim_bond[0]).powi(2) + (14.69f32 - sim_bond[2]).powi(2)).sqrt();
        if d < 1.3 { eprintln!("lane repro: ARRIVED at tick {t} pos ({:.2},{:.2},{:.2})", sim_bond[0], sim_bond[1], sim_bond[2]); break; }
        let input = nf_game::player::Input {
            turn: err.clamp(-0.07, 0.07),
            forward: if d > 2.2 { 1.0 } else { 0.45 },
            ..Default::default()
        };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        if t % 30 == 0 {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let seg = sim.world.segment(p, [3.39, 6.40, 14.69]);
            eprintln!("lane repro t{t}: pos ({:.2},{:.2},{:.2}) yaw {:.2} grounded {} seg_to_target {seg:?} floor {:?}",
                p[0], p[1], p[2], sim.bond.yaw, sim.bond.grounded, sim.world.floor_below(p, 6.0));
        }
    }

    // 9. Height scan across the lane: the capsule band (feet ~5.4 to head ~7.3) eastbound at the
    //    lane's z rows.
    eprintln!("--- lane height scan (1.2,y,z)->(3.6,y,z) ---");
    let sim = scene.sim.as_ref().unwrap();
    for z in [13.5f32, 13.75, 14.0, 14.25, 14.5, 14.75] {
        let mut row = String::new();
        for &y in &[5.2f32, 5.6, 6.0, 6.4, 6.8, 7.1, 7.4] {
            let hit = sim.world.segment([1.2, y, z], [3.6, y, z]);
            row.push_str(&format!(" y{y}={}", match hit { Some(t) => format!("{t:.2}"), None => "--".into() }));
        }
        eprintln!("z {z:5.2}:{row}");
    }
}
