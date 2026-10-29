//! walkreal probe #4 (level-0c-door2turbine-3): the band crossing, part 2 — the AI nav graph IS
//! the game's own connectivity data [D]: dump every nav node + link in the crossing box, run the
//! game's A* (`Route::to_position`, the port's [M] port of AINetwork_CalcRoute) from the band
//! west of the void to the resumed band east of it, and fine-map the strip's landing edges at
//! 0.25 resolution. Diagnostic only, run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn down(sim: &nf_view::scene::Sim, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    sim.world.segment_hit([x, y0, z], [x, y1, z]).map(|(t, _)| y0 - (y0 - y1) * t)
}

#[test]
fn probe_band_crossing_nav() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();

    // 1. The nav nodes in the crossing box, with their links inside the box.
    eprintln!("=== nav nodes in the box x 42..80, z -216..-194 ===");
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        for (ni, n) in p.nodes.iter().enumerate() {
            if n.pos[0] > 42.0 && n.pos[0] < 80.0 && n.pos[2] > -216.0 && n.pos[2] < -194.0 {
                let links: Vec<String> = p.adj[ni].iter()
                    .map(|&li| {
                        let l = &p.links[li as usize];
                        let other = if l.a as usize == ni { l.b as usize } else { l.a as usize };
                        let o = &p.nodes[other];
                        format!("->n{} ({:.1},{:.1},{:.1})", other, o.pos[0], o.pos[1], o.pos[2])
                    })
                    .collect();
                eprintln!("path{pi}({}) n{} ({:.2},{:.2},{:.2}) flags {:#x} {}", p.name, ni, n.pos[0], n.pos[1], n.pos[2], n.flags, links.join(" "));
            }
        }
    }

    // 2. The game's own A* from the band west of the void to the resumed band east of it.
    for (label, from, to) in [
        ("band(52,-207.5) -> resume(70,-208)", [52.0f32, 10.3, -207.5], [70.0f32, 10.3, -208.0]),
        ("band(52,-207.5) -> strip east(66,-205.7)", [52.0f32, 10.3, -207.5], [66.0f32, 10.85, -205.7]),
        ("band(52,-207.5) -> walkway(66,-203)", [52.0f32, 10.3, -207.5], [66.0f32, 8.98, -203.0]),
    ] {
        match walker::nav_route(sim, from, to) {
            Some(pts) => eprintln!("A* {label}: {} nodes: {}", pts.len(),
                pts.iter().map(|p| format!("({:.1},{:.1},{:.1})", p[0], p[1], p[2])).collect::<Vec<_>>().join(" ")),
            None => eprintln!("A* {label}: NO ROUTE"),
        }
    }

    // 3. The strip landing fine map (0.25 grid, x 52..64, z -206.6..-204.6, y 12 -> 8).
    eprintln!("=== strip landing fine map (0.25 grid) ===");
    eprintln!("        x: {}", (0..49).map(|i| format!("{:>6}", 52.0 + 0.25 * i as f32)).collect::<String>());
    for zi in 0..9 {
        let z = -206.6f32 + 0.25 * zi as f32;
        let mut row = format!("z {z:7.2}: ");
        for i in 0..49 {
            let x = 52.0f32 + 0.25 * i as f32;
            match down(sim, x, 12.0, 8.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }

    // 4. The pit and the south-east slope fine map (0.5 grid, x 52..68, z -212..-206, y 12 -> 3).
    eprintln!("=== pit / slope map (y 12 -> 3) ===");
    eprintln!("        x: {}", (0..33).map(|i| format!("{:>6}", 52.0 + 0.5 * i as f32)).collect::<String>());
    for zi in 0..13 {
        let z = -212.0f32 + 0.5 * zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..33 {
            let x = 52.0f32 + 0.5 * i as f32;
            match down(sim, x, 12.0, 3.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }

    // 5. Where does ch 0x3d go? Every trigger / door / drone that reads or writes 0x3d.
    eprintln!("=== ch 0x3d readers/writers (whole level) ===");
    for t in sim.drones.triggers.iter() {
        if t.out == 0x3d { eprintln!("trig kind {:#x} OUT 0x3d mask {:#x} at ({:.2},{:.2},{:.2})", t.kind, t.mask, t.centre()[0], t.centre()[1], t.centre()[2]); }
    }
    // The doors' ch_open / ch_lock that mention 0x3d.
    for d in sim.drones.doors.list.iter() {
        if d.ch_open == 0x3d || d.ch_lock == 0x3d { eprintln!("door kind {} at ({:.2},{:.2},{:.2}) ch_open {:#x} ch_lock {:#x}", d.kind, d.pos[0], d.pos[1], d.pos[2], d.ch_open, d.ch_lock); }
    }

    eprintln!("ok");
}
