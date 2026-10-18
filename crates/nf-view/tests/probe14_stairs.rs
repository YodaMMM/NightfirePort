//! level-14 probe: the stairwell descent — from the duct floor south of the grill, try the ramp
//! standing and crouched, printing the trajectory. Prints only.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn descend(crouch: bool, label: &str) {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.crouched = true;
        for _ in 0..15 { sim.step_bond(Input::default(), false, false, false); }
        // Crawl south through the grill to the stair head.
        for _ in 0..340 {
            let inp = Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
        }
        let p0 = sim.bond.pos;
        sim.bond.crouched = crouch;
        for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
        let mut traj = vec![p0];
        for t in 0..500 {
            let inp = Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
            if t % 40 == 0 { traj.push(sim.bond.pos); }
        }
        traj.push(scene.sim.as_ref().unwrap().bond.pos);
        let pts: Vec<String> = traj.iter().map(|p| format!("({:.2},{:.2},{:.2})", p[0], p[1], p[2])).collect();
        eprintln!("{label}: {}", pts.join(" "));
    }
}

#[test]
fn probe_stair_descent() {
    descend(true, "crouched descent");
    descend(false, "standing descent");
}

#[test]
fn probe_stair_blocker() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    // From the stuck point, cast the walk ray south and print the hit.
    for y in [0.7f32, 0.9, 1.05, 1.3, 1.6, 2.0, 2.4, 2.8] {
        for z0 in [56.6f32, 56.3, 56.0] {
            if let Some((t, n)) = sim.world.segment_hit([40.8, y, z0], [40.8, y, 54.8]) {
                let p = [40.8, y + (54.8 - y) * 0.0, z0 + (54.8 - z0) * t];
                eprintln!("y {y:.2} z0 {z0:.2}: hit at t {t:.3} pos ({:.2},{:.2},{:.2}) n ({:.2},{:.2},{:.2})",
                    p[0], p[1], p[2], n[0], n[1], n[2]);
            } else {
                eprintln!("y {y:.2} z0 {z0:.2}: CLEAR to z=54.8");
            }
        }
    }
}

#[test]
fn probe_trench_drop() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    // Long down-rays through the trench footprint + the south wall band at low y.
    for (x, z) in [(40.8f32, 56.27), (40.8, 55.9), (40.8, 55.72), (40.5, 55.6), (41.1, 55.6), (40.8, 55.5), (40.8, 55.2), (40.8, 54.9)] {
        let hit = sim.world.segment([x, 1.6, z], [x, -3.5, z]);
        match hit {
            Some(t) => eprintln!("({x:.2},{z:.2}): floor at y {:.2}", 1.6 - 5.1 * t),
            None => eprintln!("({x:.2},{z:.2}): OPEN to y=-3.5"),
        }
    }
    // Southward rays at very low y (under the wall?).
    for y in [0.5f32, 0.3, 0.1, -0.2] {
        let hit = sim.world.segment([40.8, y, 55.9], [40.8, y, 54.5]);
        eprintln!("southward y {y:.2} from z 55.9: {}", if hit.is_some() { "BLOCKED" } else { "CLEAR" });
    }
}

#[test]
fn probe_shaft_drop() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = [40.8, 1.2, 55.95];
        sim.bond.crouched = true;
        let mut traj = Vec::new();
        for t in 0..300 {
            sim.step_bond(Input::default(), false, false, false);
            if t % 40 == 0 { traj.push(sim.bond.pos); }
        }
        traj.push(sim.bond.pos);
        let pts: Vec<String> = traj.iter().map(|p| format!("({:.2},{:.2},{:.2})", p[0], p[1], p[2])).collect();
        eprintln!("shaft drop (crouched, in the slot): {}", pts.join(" "));
        let p = sim.bond.pos;
        eprintln!("final: grounded {} y {:.2}", sim.bond.grounded, p[1]);
    }
}

#[test]
fn probe_duct_floor_hole() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    // Scan the west duct floor (x 40.3..41.3) for holes: ray down from 2.2 (just under the floor
    // surface 2.32/2.40) — where does it find the under-vent floor (~0.3) vs the floor slab?
    let mut z = 67.0f32;
    while z >= 57.5 {
        let mut row = String::new();
        let mut x = 40.2f32;
        while x <= 41.4 {
            let hit = sim.world.segment([x, 2.25, z], [x, -0.5, z]);
            match hit {
                None => row.push('?'),
                Some(t) => {
                    let y = 2.25 - 2.75 * t;
                    row.push(if y < 1.9 { 'H' } else if y < 2.36 { '.' } else { '#' });
                }
            }
            x += 0.2;
        }
        eprintln!("z {z:5.2} {row}");
        z -= 0.25;
    }
    eprintln!("legend: . = duct floor slab, H = opening (under-floor seen), # = raised");
}

#[test]
fn probe_east_floor_hole() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    let mut z = 67.0f32;
    while z >= 56.5 {
        let mut row = String::new();
        let mut x = 44.5f32;
        while x <= 46.2 {
            let hit = sim.world.segment([x, 2.6, z], [x, -0.5, z]);
            match hit {
                None => row.push('?'),
                Some(t) => {
                    let y = 2.6 - 3.1 * t;
                    row.push(if y < 1.9 { 'H' } else if y < 2.45 { '.' } else { '#' });
                }
            }
            x += 0.2;
        }
        eprintln!("z {z:5.2} {row}");
        z -= 0.25;
    }
    eprintln!("legend: . = duct floor slab (~2.4), H = opening (floor below 1.9), # = raised, ? = no hit");
}

#[test]
fn probe_ledge_climb() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = [40.8, 1.4, 56.27];
        sim.bond.crouched = true;
        let mut traj = Vec::new();
        for t in 0..420 {
            // Face east, push forward — try to climb the trench's side ledge into the middle void.
            let inp = Input { forward: 1.0, turn: if t < 30 { 0.05 } else { 0.0 }, ..Default::default() };
            sim.step_bond(inp, false, false, false);
            if t % 40 == 0 { traj.push(sim.bond.pos); }
        }
        traj.push(sim.bond.pos);
        let pts: Vec<String> = traj.iter().map(|p| format!("({:.2},{:.2},{:.2})", p[0], p[1], p[2])).collect();
        eprintln!("ledge climb east: {}", pts.join(" "));
        let p = scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("final: ({:.2},{:.2},{:.2}) grounded {}", p[0], p[1], p[2], scene.sim.as_ref().unwrap().bond.grounded);
    }
}
