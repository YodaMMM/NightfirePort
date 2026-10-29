//! walkreal probe #8 (level-0c-every-objective-blockers): the second pass.
//! (a) the display ground -> annex route map (x 0..52, z -126..-158);
//! (b) the pit hole in the lower walkway (x 0..14, z -148..-158, y 13 -> 5);
//! (c) the west floors x -12..8, z -214..-226 (the wire[2]/wire[0] landing ledges);
//! (d) every trigger in the level (kind/out/mask/centre) — the ch-0x54 writer hunt;
//! (e) locks near the annex; (f) the annex east-edge openness; (g) the vent underside.
//! Diagnostic only, run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn down(sim: &nf_view::scene::Sim, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    sim.world.segment_hit([x, y0, z], [x, y1, z]).map(|(t, _)| y0 - (y0 - y1) * t)
}

#[test]
fn probe_blockers2() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();

    // (a) the display ground -> annex route map, y 9 -> 1.
    eprintln!("=== display->annex map (y 9 -> 1) x 0..52 step 2, z -126..-158 step 2 ===");
    eprintln!("        x: {}", (0..27).map(|i| format!("{:>5}", i * 2)).collect::<String>());
    for zi in 0..17 {
        let z = -126.0f32 - zi as f32 * 2.0;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..27 {
            let x = i as f32 * 2.0;
            match down(sim, x, 9.0, 1.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2}")),
                None => row.push_str("   --"),
            }
            row.push(' ');
        }
        eprintln!("{row}");
    }

    // (b) the pit hole in the lower walkway.
    eprintln!("=== the pit (y 13 -> 5) x 0..14 step 1, z -148..-158 step 1 ===");
    eprintln!("        x: {}", (0..15).map(|i| format!("{:>5}", i)).collect::<String>());
    for zi in 0..11 {
        let z = -148.0f32 - zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for x in 0..15 {
            match down(sim, x as f32, 13.0, 5.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2}")),
                None => row.push_str("   --"),
            }
            row.push(' ');
        }
        eprintln!("{row}");
    }

    // (c) the west floors (the wire[2]/wire[0] landings).
    eprintln!("=== west floors (y 14 -> 8) x -12..8 step 1, z -214..-226 step 1 ===");
    eprintln!("        x: {}", (0..21).map(|i| format!("{:>6}", i as i32 - 12)).collect::<String>());
    for zi in 0..13 {
        let z = -214.0f32 - zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..21 {
            let x = (i as i32 - 12) as f32;
            match down(sim, x, 14.0, 8.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }

    // (d) every trigger in the level.
    eprintln!("=== all triggers (kind out mask centre) ===");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let c = t.centre();
        eprintln!("trig[{i}] kind {} out {:#04x} mask {} ({:.2},{:.2},{:.2}) tris {}",
            t.kind, t.out, t.mask, c[0], c[1], c[2], t.tris.len());
    }

    // (e) locks near the annex (x 48..60, z -172..-156).
    eprintln!("=== locks in the annex box ===");
    for (i, l) in sim.drones.locks.iter().enumerate() {
        let p = l.pos;
        if p[0] > 44.0 && p[0] < 62.0 && p[2] > -174.0 && p[2] < -152.0 {
            eprintln!("lock[{i}] out {:#x} pos ({:.2},{:.2},{:.2})", l.out, p[0], p[1], p[2]);
        }
    }
    // also all locks, short form
    eprintln!("=== all locks (out pos) ===");
    for (i, l) in sim.drones.locks.iter().enumerate() {
        eprintln!("lock[{i}] out {:#x} ({:.2},{:.2},{:.2})", l.out, l.pos[0], l.pos[1], l.pos[2]);
    }

    // (f) the annex east edge: east-west rays at y 4.5, z -160..-168, from x 54 -> 59.
    eprintln!("=== annex east edge rays (x 54 -> 59) ===");
    for z in [-160.0f32, -162.0, -164.0, -166.0, -168.0] {
        for y in [4.0f32, 4.7, 5.5] {
            let hit = sim.world.segment_hit([54.0, y, z], [59.0, y, z]).map(|(t, _)| 54.0 + 5.0 * t);
            eprintln!("z {z:7.1} y {y:4.1}: first face at x {:?}", hit);
        }
    }

    // (g) the vent underside: y 5.2 -> 0 (below the closed lid 5.43).
    eprintln!("=== the vent underside (y 5.2 -> 0) x 52..56 step 0.5, z -162..-166 step 0.5 ===");
    eprintln!("        x: {}", (0..9).map(|i| format!("{:>5}", 104 + i)).collect::<String>());
    for zi in 0..9 {
        let z = -162.0f32 - zi as f32 * 0.5;
        let mut row = format!("z {z:7.2}: ");
        for i in 0..9 {
            let x = 52.0f32 + i as f32 * 0.5;
            match down(sim, x, 5.2, 0.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2}")),
                None => row.push_str("   --"),
            }
            row.push(' ');
        }
        eprintln!("{row}");
    }

    eprintln!("ok");
}
