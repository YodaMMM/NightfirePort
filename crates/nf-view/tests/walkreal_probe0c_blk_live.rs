//! walkreal probe #7 (level-0c-every-objective-blockers): the four stand-in regions in one run.
//! (a) blocker 2 — the north display case / plate #274: the walkway's north end, the case mouth,
//!     the bay interior, the case's south wall openings onto the lower walkway;
//! (b) blocker 3 — the display ground -> warehouse/yard descent: the annex/deck/strip floors, the
//!     ch-0x54 vent SS #184 (keys), the triggers writing 0x54, the shaft floors;
//! (c) blocker 4 — the ring's west/south gaps: every wire's derived segments, the type-3 icons,
//!     the floors at the ends, the grab-reach from the arc/west edge;
//! (d) blocker 5 — the doorway-ledge -> yard descent: the ledge's south edge walls and floors.
//! Diagnostic only, run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn down(sim: &nf_view::scene::Sim, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    sim.world.segment_hit([x, y0, z], [x, y1, z]).map(|(t, _)| y0 - (y0 - y1) * t)
}

fn ray(sim: &nf_view::scene::Sim, a: [f32; 3], b: [f32; 3]) -> Option<(f32, f32)> {
    // returns (hit height, hit z or x along the ray) — caller interprets
    sim.world.segment_hit(a, b).map(|(t, _)| {
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
        (p[1], if a[2] != b[2] { p[2] } else { p[0] })
    })
}

#[test]
fn probe_blockers() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();

    // ============ (c) FIRST: the wires + icons (blocker 4) ============
    eprintln!("=== wires (all) ===");
    for (i, wr) in sim.wires.iter().enumerate() {
        let (ga, gb) = wr.grab_segment();
        eprintln!("wire[{i}] type {} ch {} centre ({:.2},{:.2},{:.2}) r {:.2} yaw {:.3} pitch {:.3}",
            wr.wire_type, wr.channel, wr.centre[0], wr.centre[1], wr.centre[2], wr.radius, wr.yaw, wr.pitch);
        eprintln!("         grab A ({:.2},{:.2},{:.2}) B ({:.2},{:.2},{:.2})", ga[0], ga[1], ga[2], gb[0], gb[1], gb[2]);
        let (ta, tb) = wr.travel_segment();
        eprintln!("         travel A ({:.2},{:.2},{:.2}) B ({:.2},{:.2},{:.2})", ta[0], ta[1], ta[2], tb[0], tb[1], tb[2]);
        for (lbl, p) in [("grabA", ga), ("grabB", gb)] {
            let f = down(sim, p[0], p[1] + 1.0, p[1] - 8.0, p[2]);
            eprintln!("         floor under {lbl}: {:?}", f);
        }
    }
    eprintln!("=== icons (all) ===");
    for (i, ic) in sim.icons.iter().enumerate() {
        eprintln!("icon[{i}] ctx {} on_ch {} pos ({:.2},{:.2},{:.2}) centre ({:.2},{:.2},{:.2}) r {:.2}",
            ic.ctx, ic.on_channel, ic.pos[0], ic.pos[1], ic.pos[2], ic.centre[0], ic.centre[1], ic.centre[2], ic.radius);
    }
    // Floors across the west gap and the south gap at walkway heights, fine grid.
    eprintln!("=== west gap floors (y 14 -> 8) x 0..15 step 1, z -208..-222 step 1 ===");
    eprintln!("        x: {}", (0..16).map(|i| format!("{:>6}", i)).collect::<String>());
    for zi in 0..15 {
        let z = -208.0f32 - zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for x in 0..16 {
            let xf = x as f32;
            match down(sim, xf, 14.0, 8.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("=== south gap floors (y 14 -> 8) x 26..42 step 1, z -212..-224 step 1 ===");
    eprintln!("        x: {}", (0..17).map(|i| format!("{:>6}", 26 + i)).collect::<String>());
    for zi in 0..13 {
        let z = -212.0f32 - zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..17 {
            let x = 26.0f32 + i as f32;
            match down(sim, x, 14.0, 8.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    // Grab reach: for the type-0 wires near the ring, is the wire sphere hit by a body standing
    // at candidate spots (capsule spine pos+0.275..pos-0.485, r ignored by sphere_hits)?
    eprintln!("=== grab tests: body capsule at candidate spots vs each wire's sphere ===");
    for (lbl, p) in [
        ("west edge (4,13.6,-207.5)", [4.0f32, 13.6, -207.5]),
        ("west edge S (4,13.6,-208.5)", [4.0, 13.6, -208.5]),
        ("arc W (14,12.3,-218)", [14.0, 12.3, -218.0]),
        ("arc E (27,12.3,-218)", [27.0, 12.3, -218.0]),
        ("arc E2 (28,12.3,-217.5)", [28.0, 12.3, -217.5]),
        ("ledge W (38.9,12.64,-217.6)", [38.9, 12.64, -217.6]),
    ] {
        let a = [p[0], p[1] + 0.275, p[2]];
        let b = [p[0], p[1] - 0.485, p[2]];
        for (i, wr) in sim.wires.iter().enumerate() {
            if wr.sphere_hits(a, b) {
                eprintln!("{lbl} -> wire[{i}] (type {}) sphere HIT", wr.wire_type);
            }
        }
    }
    // What does Player_Wire's ride do: read the decompile-side pins is out of scope here; the
    // ride direction test happens in the leg.

    // ============ (a) blocker 2: the display case / plate #274 ============
    eprintln!("=== the bay: floors (y 16 -> 11) x 2..10 step 0.5, z -108..-116 step 0.5 ===");
    eprintln!("        x: {}", (0..17).map(|i| format!("{:>5}", 20 + i)).collect::<String>());
    for zi in 0..17 {
        let z = -108.0f32 - zi as f32 * 0.5;
        let mut row = format!("z {z:7.2}: ");
        for i in 0..17 {
            let x = 2.0f32 + i as f32 * 0.5;
            match down(sim, x, 16.0, 11.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("=== the case mouth: north-south rays at x 5.5 (z -109 -> -117) at body heights ===");
    for y in [12.5f32, 13.4, 14.2, 15.0, 15.8] {
        let hit = ray(sim, [5.5, y, -108.5], [5.5, y, -117.0]);
        eprintln!("y {y:5.1}: first face at {:?}", hit);
    }
    eprintln!("=== the case south wall: east-west rays at z -111.2 (x 2 -> 10) ===");
    for y in [12.5f32, 13.0, 13.4, 13.9, 14.4, 15.0] {
        let hit = ray(sim, [2.0, y, -111.2], [10.0, y, -111.2]);
        eprintln!("y {y:5.1}: first face at x {:?}", hit);
    }
    eprintln!("=== triggers near the case (centre in x 2..10, z -114..-108) ===");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let c = t.centre();
        if c[0] > 2.0 && c[0] < 10.0 && c[2] > -114.0 && c[2] < -108.0 {
            eprintln!("trig[{i}] kind {} out {:#x} mask {} centre ({:.2},{:.2},{:.2}) tris {}",
                t.kind, t.out, t.mask, c[0], c[1], c[2], t.tris.len());
        }
    }
    eprintln!("=== SS movers with centre in x -2..12, z -120..-100 ===");
    for s in sim.drones.sss.list.iter() {
        let p = s.pos();
        if p[0] > -2.0 && p[0] < 12.0 && p[2] > -120.0 && p[2] < -100.0 {
            eprintln!("ss ch {} state {} mask {:#x} pos ({:.2},{:.2},{:.2}) keys {} key0y {:.2} keyNy {:.2} rate {}",
                s.channel, s.state, s.mask, p[0], p[1], p[2], s.keys.len(),
                s.keys.first().map(|k| k.pos[1]).unwrap_or(f32::NAN),
                s.keys.last().map(|k| k.pos[1]).unwrap_or(f32::NAN), s.rate);
        }
    }

    // ============ (b) blocker 3: the display ground -> warehouse/yard descent ============
    eprintln!("=== the vent region floors (y 8 -> 0) x 48..58 step 1, z -158..-170 step 1 ===");
    eprintln!("        x: {}", (0..11).map(|i| format!("{:>6}", 48 + i)).collect::<String>());
    for zi in 0..13 {
        let z = -158.0f32 - zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..11 {
            let x = 48.0f32 + i as f32;
            match down(sim, x, 8.0, 0.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("=== the display ground east floors (y 9 -> 0) x 30..50 step 2, z -148..-162 step 2 ===");
    eprintln!("        x: {}", (0..11).map(|i| format!("{:>6}", 30 + i * 2)).collect::<String>());
    for zi in 0..8 {
        let z = -148.0f32 - zi as f32 * 2.0;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..11 {
            let x = 30.0f32 + i as f32 * 2.0;
            match down(sim, x, 9.0, 0.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("=== SS movers with centre in x 40..70, z -175..-150 ===");
    for s in sim.drones.sss.list.iter() {
        let p = s.pos();
        if p[0] > 40.0 && p[0] < 70.0 && p[2] > -175.0 && p[2] < -150.0 {
            eprintln!("ss ch {:#x} state {} mask {:#x} pos ({:.2},{:.2},{:.2}) keys {}", s.channel, s.state, s.mask, p[0], p[1], p[2], s.keys.len());
            for (ki, k) in s.keys.iter().enumerate() {
                eprintln!("   key[{ki}] ({:.2},{:.2},{:.2})", k.pos[0], k.pos[1], k.pos[2]);
            }
        }
    }
    eprintln!("=== triggers out 0x54 or centre near the vent (x 48..60, z -170..-158) ===");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let c = t.centre();
        let near = c[0] > 44.0 && c[0] < 64.0 && c[2] > -172.0 && c[2] < -154.0;
        if t.out == 0x54 || near {
            eprintln!("trig[{i}] kind {} out {:#x} mask {} centre ({:.2},{:.2},{:.2}) tris {}",
                t.kind, t.out, t.mask, c[0], c[1], c[2], t.tris.len());
        }
    }

    // ============ (d) blocker 5: the doorway-ledge -> yard descent ============
    eprintln!("=== the doorway-ledge region floors (y 5 -> -0.5) x 52..60 step 0.5, z -172..-180 step 0.5 ===");
    eprintln!("        x: {}", (0..17).map(|i| format!("{:>5}", 104 + i)).collect::<String>());
    for zi in 0..17 {
        let z = -172.0f32 - zi as f32 * 0.5;
        let mut row = format!("z {z:7.2}: ");
        for i in 0..17 {
            let x = 52.0f32 + i as f32 * 0.5;
            match down(sim, x, 5.0, -0.5, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("=== the ledge south edge: north-south rays at x 55.2, 56.5 (z -174 -> -178) ===");
    for x in [54.5f32, 55.2, 56.5, 57.5] {
        for y in [1.5f32, 2.2, 2.9] {
            let hit = ray(sim, [x, y, -174.0], [x, y, -178.0]);
            eprintln!("x {x:5.1} y {y:4.1}: first face at z {:?}", hit);
        }
    }

    eprintln!("ok");
}
