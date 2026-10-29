//! walkreal probe #3 (level-0c-door2turbine-3): the band x 53..67 crossing. The -2 session left
//! the band east walk wedged: the band floor 10.34 runs x 44..53 (z −209..−206.5), resumes
//! x 67..69+, and x 55..65 is void at every probed z (−209.5..−206.0) [D its probe], while a
//! 10.85 strip runs z −206.0..−205.2 (x 47.5..53 AND x 61..69, gap x 55..59) and the east
//! walkway descends north of it (x 61..69, z −204..−202). This probe maps the whole crossing
//! box densely (three-slice column map), fires the horizontal rays the step/jump questions
//! need, and dumps every dynamic object / trigger / door / pickup in the box twice (300 ticks
//! apart) so a moving bridge shows as Δpos. Everything [D] from the port's disc-built world —
//! diagnostic only, run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn down(sim: &nf_view::scene::Sim, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    sim.world.segment_hit([x, y0, z], [x, y1, z]).map(|(t, _)| y0 - (y0 - y1) * t)
}

fn hray(sim: &nf_view::scene::Sim, from: [f32; 3], to: [f32; 3]) -> Option<[f32; 3]> {
    sim.world.segment_hit(from, to).map(|(t, _)| {
        [from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t, from[2] + (to[2] - from[2]) * t]
    })
}

fn fmt(v: Option<f32>) -> String {
    match v { Some(y) => format!("{y:5.2}"), None => "   --".into() }
}

#[test]
fn probe_band_crossing() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);

    // 1. The three-slice column map over the crossing box: top surface (y 18->12), mid
    //    (12->6), low (6->1). A bridge at ANY height shows; the · separates the slices.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== crossing column map (x 50..72 step 1, z -212..-196 step 1; top/mid/low) ===");
        eprintln!("        x: {}", (0..23).map(|i| format!("{:>6}", 50 + i)).collect::<String>());
        for zi in 0..17 {
            let z = -212.0f32 + zi as f32;
            let mut row = format!("z {z:7.1}: ");
            for i in 0..23 {
                let x = 50.0f32 + i as f32;
                let top = down(sim, x, 18.0, 12.0, z);
                let mid = down(sim, x, 12.0, 6.0, z);
                let low = down(sim, x, 6.0, 1.0, z);
                row.push_str(&format!("{}·{}·{} ", fmt(top), fmt(mid).trim(), fmt(low).trim()));
            }
            eprintln!("{row}");
        }
        eprintln!("(each cell top·mid·low; -- = void in that slice)");
    }

    // 2. The band's east edge fine rows (x 44..57 step 0.5, z -209.5..-206.0 step 0.5, y 12->8).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== band east edge fine map (y 12 -> 8) ===");
        eprintln!("        x: {}", (0..27).map(|i| format!("{:>6}", 44.0 + 0.5 * i as f32)).collect::<String>());
        for zi in 0..8 {
            let z = -209.5f32 + 0.5 * zi as f32;
            let mut row = format!("z {z:7.1}: ");
            for i in 0..27 {
                let x = 44.0f32 + 0.5 * i as f32;
                row.push_str(&format!("{} ", fmt(down(sim, x, 12.0, 8.0, z)).trim_start()));
            }
            eprintln!("{row}");
        }
    }

    // 3. The strip fine rows (z -206.5..-204.5 step 0.25, x 46..70 step 0.5, y 12->8) — the
    //    10.85 strip's exact z extent and its x 55..59 gap.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== strip fine map (y 12 -> 8) ===");
        eprintln!("        x: {}", (0..49).map(|i| format!("{:>6}", 46.0 + 0.5 * i as f32)).collect::<String>());
        for zi in 0..9 {
            let z = -206.5f32 + 0.25 * zi as f32;
            let mut row = format!("z {z:7.1}: ");
            for i in 0..49 {
                let x = 46.0f32 + 0.5 * i as f32;
                row.push_str(&format!("{} ", fmt(down(sim, x, 12.0, 8.0, z)).trim_start()));
            }
            eprintln!("{row}");
        }
    }

    // 4. The horizontal rays: band->strip step-up (north rays at band height), the strip's
    //    eastward run (the x 55..59 gap at strip height), the strip->band resume drop (south
    //    rays at strip height east of 65).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== horizontal rays ===");
        for (label, from, to) in [
            ("band->strip north x47 y10.6", [47.0f32, 10.6, -206.8], [47.0f32, 10.6, -204.8]),
            ("band->strip north x50 y10.6", [50.0f32, 10.6, -206.8], [50.0f32, 10.6, -204.8]),
            ("band->strip north x52 y10.6", [52.0f32, 10.6, -206.8], [52.0f32, 10.6, -204.8]),
            ("band->strip north x53 y10.6", [53.0f32, 10.6, -206.8], [53.0f32, 10.6, -204.8]),
            ("strip east y10.9 z-205.6", [52.5f32, 10.9, -205.6], [70.0f32, 10.9, -205.6]),
            ("strip east y11.3 z-205.6", [52.5f32, 11.3, -205.6], [70.0f32, 11.3, -205.6]),
            ("strip east y10.5 z-205.6", [52.5f32, 10.5, -205.6], [70.0f32, 10.5, -205.6]),
            ("strip->resume south x66", [66.0f32, 10.7, -205.4], [66.0f32, 10.7, -208.5]),
            ("strip->resume south x68", [68.0f32, 10.7, -205.4], [68.0f32, 10.7, -208.5]),
            ("strip->resume south x63", [63.0f32, 10.7, -205.4], [63.0f32, 10.7, -208.5]),
            ("band level east z-207.6", [52.0f32, 10.6, -207.6], [70.0f32, 10.6, -207.6]),
            ("band level east z-208.5", [52.0f32, 10.6, -208.5], [70.0f32, 10.6, -208.5]),
            ("north of strip x56 y10.9", [56.0f32, 10.9, -206.2], [56.0f32, 10.9, -203.0]),
            ("north of strip x58 y10.9", [58.0f32, 10.9, -206.2], [58.0f32, 10.9, -203.0]),
            ("walkway north x65 y9.0", [65.0f32, 9.0, -204.5], [65.0f32, 9.0, -201.0]),
            ("walkway north x68 y9.0", [68.0f32, 9.0, -204.5], [68.0f32, 9.0, -201.0]),
        ] {
            let hit = hray(sim, from, to);
            match hit {
                Some(p) => eprintln!("{label}: HIT at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]),
                None => eprintln!("{label}: CLEAR"),
            }
        }
    }

    // 5. The dynamic objects in the box, twice, 300 ticks apart — a moving bridge shows as Δpos.
    let dump_box = |w: &Walker, tag: &str| {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== drones in the crossing box ({tag}) ===");
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || d.corpse { continue; }
            if d.pos[0] > 47.0 && d.pos[0] < 76.0 && d.pos[2] > -215.0 && d.pos[2] < -193.0 {
                eprintln!("g{i} skin {:#x} team {} at ({:.2},{:.2},{:.2}) yaw {:.2}", d.skin, d.team, d.pos[0], d.pos[1], d.pos[2], d.yaw);
            }
        }
        eprintln!("=== triggers in the box ({tag}) ===");
        for t in sim.drones.triggers.iter() {
            if t.tris.is_empty() { continue; }
            let c = t.centre();
            if c[0] > 47.0 && c[0] < 76.0 && c[2] > -215.0 && c[2] < -193.0 {
                eprintln!("trig kind {:#x} out {:#x} mask {:#x} at ({:.2},{:.2},{:.2})", t.kind, t.out, t.mask, c[0], c[1], c[2]);
            }
        }
        eprintln!("=== doors in the box ({tag}) ===");
        for d in sim.drones.doors.list.iter() {
            if d.pos[0] > 47.0 && d.pos[0] < 76.0 && d.pos[2] > -215.0 && d.pos[2] < -193.0 {
                eprintln!("door kind {} at ({:.2},{:.2},{:.2}) state {:#x} ch_open {}", d.kind, d.pos[0], d.pos[1], d.pos[2], d.state, d.ch_open);
            }
        }
        eprintln!("=== pickups in the box ({tag}) ===");
        for p in sim.drones.pickups.iter() {
            if p.gone { continue; }
            let q = [p.m[3][0], p.m[3][1], p.m[3][2]];
            if q[0] > 47.0 && q[0] < 76.0 && q[2] > -215.0 && q[2] < -193.0 {
                eprintln!("pickup kind {:#x} at ({:.2},{:.2},{:.2})", p.kind, q[0], q[1], q[2]);
            }
        }
    };
    dump_box(&w, "t0");
    w.wait(300);
    dump_box(&w, "t+300");

    eprintln!("ok");
}
