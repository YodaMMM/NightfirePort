//! Scratch (level-06-every-objective): standable-surface map across the gallery → dressing-room
//! gap (the walkreal_07000006 header's OPEN LEG), the level's AI points, and the gap region's
//! triggers. Ray-casts straight down through the port's collision world, one row per z.
use nf_game::world::CollisionWorld;

fn probe(sim: &CollisionWorld, top: f32, bot: f32, x0: f32, x1: f32, z0: f32, z1: f32, step: f32) {
    let cols = ((x1 - x0) / step).round() as i32;
    let xs: String = (0..=cols)
        .map(|i| {
            let x = x0 + step * i as f32;
            if i % 4 == 0 { format!("{:5.0}", x) } else { format!("{:5}", "") }
        })
        .collect();
    println!("   x:      {xs}");
    let rows = ((z1 - z0) / step).round() as i32;
    for k in 0..=rows {
        let z = z0 + step * k as f32;
        let mut row = format!("z {:7.1}: ", z);
        for i in 0..=cols {
            let x = x0 + step * i as f32;
            match sim.segment_hit([x, top, z], [x, bot, z]) {
                Some((t, _)) => {
                    let y = top + t * (bot - top);
                    row.push_str(&format!("{:5.1}", y));
                }
                None => row.push_str("    ."),
            }
        }
        println!("{row}");
    }
}

fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let scene = nf_view::load(&path, "07000006.bin", false).expect("load 07000006");
    let sim = scene.sim.as_ref().unwrap();

    println!("== AI points (kind, pos, yaw, param) [D placements 0xE9]");
    for (i, a) in sim.drones.ai_points.iter().enumerate() {
        println!("  ai {i}: kind {} at {:?} yaw {:.2} param {}", a.kind, a.pos, a.yaw, a.param);
    }

    println!("== gap-region triggers (centre in x 20..45, z −76..−40) with kind/out/mask");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let c = t.centre();
        if c[0] >= 20.0 && c[0] <= 45.0 && c[2] >= -76.0 && c[2] <= -40.0 {
            println!("  trig {i}: kind {:#x} out {} mask {:#x} centre {c:?}", t.kind, t.out, t.mask);
        }
    }

    println!("== doors near the gap (x 25..45, z −76..−40)");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        let p = d.pos;
        if p[0] >= 25.0 && p[0] <= 45.0 && p[2] >= -76.0 && p[2] <= -40.0 {
            println!("  door {i}: placement {} at {p:?} state {} ch_lock {} ch_open {}",
                d.placement, d.state, d.ch_lock, d.ch_open);
        }
    }

    // The gallery (y≈50.1, z −65..−70) → the dressing room (LoadPoint #746 centre ≈ (35.5, 44.5,
    // −54.1); hurt #765's volume x 31.12..36.96, y 42.22..46.79, z −59.06..−48.87).
    let (x0, x1, z0, z1) = (24.0, 42.0, -72.0, -46.0);
    println!("== BAND A 57→50.3: above the gallery (a catwalk? the AI path's 54.67 level?) x {x0}..{x1} z {z0}..{z1}");
    probe(&sim.world, 57.0, 50.3, x0, x1, z0, z1, 1.0);
    println!("== BAND B 50.3→46.0: the gallery deck (50.1) and the room ceiling (~46.8)");
    probe(&sim.world, 50.3, 46.0, x0, x1, z0, z1, 1.0);
    println!("== BAND C 46.0→38.0: the room floor (44.5/42.5), the tunnel, the ground");
    probe(&sim.world, 46.0, 38.0, x0, x1, z0, z1, 1.0);

    // FINE pass over the deck's north edge where the walk wedged (a parapet can hide between
    // the 1.0 z samples above): x 30..38, z −68..−61 at 0.25.
    println!("== FINE A 54→50.3 step 0.25 x 30..38 z −68..−61");
    probe(&sim.world, 54.0, 50.3, 30.0, 38.0, -68.0, -61.0, 0.25);
    println!("== FINE B 50.3→46.0 step 0.25 (the deck edge / the drop zone)");
    probe(&sim.world, 50.3, 46.0, 30.0, 38.0, -68.0, -61.0, 0.25);
    println!("== FINE C 46.0→41.0 step 0.25 (the shelf's south rim)");
    probe(&sim.world, 46.0, 41.0, 30.0, 38.0, -68.0, -61.0, 0.25);

    // Horizontal rays at body heights: where does the deck's north wall stand, per x?
    println!("== horizontal rays west→east... no, south→north: from (x, y, −66.0) to (x, y, −60.0)");
    for &y in &[50.4f32, 49.8, 49.0, 48.0, 47.0, 46.2, 45.5] {
        let mut row = format!("y {y:5.1}: ");
        for i in 0..17 {
            let x = 30.0 + 0.5 * i as f32;
            match sim.world.segment_hit([x, y, -66.0], [x, y, -60.0]) {
                Some((t, _)) => {
                    let z = -66.0 + t * 6.0;
                    row.push_str(&format!("{:5.1}", z));
                }
                None => row.push_str("    ."),
            }
        }
        println!("{row}");
    }
    println!("   (columns are x 30.0, 30.5, .. 38.0; values are the z where the ray hits, dots = clear)");

    // Dense sweep of the exact wedge spot: x 34.0..36.0 (0.25), y 51.5 down to 45.0 (0.25),
    // ray from (x, y, −66.0) to (x, y, −62.0) — every hit is the blocker.
    println!("== dense sweep x 34.0..36.0 y 51.5..45.0 (hits = z of the blocker)");
    for k in 0..27 {
        let y = 51.5 - 0.25 * k as f32;
        let mut row = format!("y {y:5.2}: ");
        for i in 0..9 {
            let x = 34.0 + 0.25 * i as f32;
            match sim.world.segment_hit([x, y, -66.0], [x, y, -62.0]) {
                Some((t, _)) => { let z = -66.0 + t * 4.0; row.push_str(&format!("{:6.2}", z)); }
                None => row.push_str("     ."),
            }
        }
        println!("{row}");
    }
    println!("   (columns x 34.00, 34.25, .. 36.00)");

    // FULL-edge fine sweep: every blocker on the deck's north edge, x 30..38 at 0.1, y 52..44
    // at 0.25, ray (x, y, −66.0) → (x, y, −63.5). Print only the rows that hit something.
    println!("== full-edge sweep x 30..38 step 0.1, y 52..44 step 0.25 (only rows with hits)");
    for k in 0..33 {
        let y = 52.0 - 0.25 * k as f32;
        let mut row = String::new();
        let mut any = false;
        for i in 0..81 {
            let x = 30.0 + 0.1 * i as f32;
            match sim.world.segment_hit([x, y, -66.0], [x, y, -63.5]) {
                Some((t, _)) => { let z = -66.0 + t * 2.5; row.push_str(&format!("{x:.1}:{z:.2} ")); any = true; }
                None => {}
            }
        }
        if any { println!("y {y:5.2}: {row}"); }
    }

    // The west approach: does the room (41.0 floor at x 24..28, z −61..−48) connect to the
    // searchlight courtyard (42.0)? Map x 6..30, z −66..−44.
    println!("== WEST BAND A 54→50.3 x 6..30 z −66..−44 (any upper passage west of the gallery?)");
    probe(&sim.world, 54.0, 50.3, 6.0, 30.0, -66.0, -44.0, 1.0);
    println!("== WEST BAND B 50.3→46.0");
    probe(&sim.world, 50.3, 46.0, 6.0, 30.0, -66.0, -44.0, 1.0);
    println!("== WEST BAND C 46.0→38.0 (the room's 41/43 floors vs the courtyard 42)");
    probe(&sim.world, 46.0, 38.0, 6.0, 30.0, -66.0, -44.0, 1.0);

    // The deck's EAST end and behind locked door #46: x 34..42, z −72..−60, fine bands.
    println!("== EAST A 56→49.5 x 34..42 z −72..−60");
    probe(&sim.world, 56.0, 49.5, 34.0, 42.0, -72.0, -60.0, 0.5);
    println!("== EAST B 49.5→46.0");
    probe(&sim.world, 49.5, 46.0, 34.0, 42.0, -72.0, -60.0, 0.5);
    println!("== EAST C 46.0→39.0");
    probe(&sim.world, 46.0, 39.0, 34.0, 42.0, -72.0, -60.0, 0.5);
}
