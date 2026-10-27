//! level-12-qclaw-ride probe: the SkyBridge grapple point #130 `GRAPPLEPOINT_SkyBridge1`
//! (11.49, 9.23, 30.14 [D 07000012-audit §0x3a]) — what the office floor can see of it, what is
//! under it (the skybridge deck?), and where the reel's hang would drop Bond. Geometry evidence
//! only; the assertions live in the real ride test.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn floor_at(world: &nf_game::world::CollisionWorld, x: f32, z: f32, from_y: f32) -> Option<f32> {
    world.segment([x, from_y, z], [x, -3.0, z]).map(|t| from_y - t * (from_y + 3.0))
}

#[ignore = "level-12-qclaw-ride: the geometry probe — the findings live in its comments and the ride test"]
fn probe12_grapple_geometry() {
    let iso = iso();
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("== grapple points [D placements 0x3a]: {:?}", sim.drones.grapple_points);
    assert!(!sim.drones.grapple_points.is_empty(), "the SkyBridge grapple point loads [D #130]");
    let gp = sim.drones.grapple_points[0].pos;
    let rad = sim.drones.grapple_points[0].radius;
    eprintln!("== point {gp:?} radius {rad}");
    let world = &sim.world;

    // 1. The deck under the point: down-segments on a grid around its xz.
    eprintln!("== deck under the point (floor y from y 10 down; 'none' = a fall past -3) ==");
    for dz in [-4.0f32, -2.0, 0.0, 2.0, 4.0] {
        let mut row = String::new();
        for dx in [-4.0f32, -2.0, 0.0, 2.0, 4.0] {
            let y = floor_at(world, gp[0] + dx, gp[2] + dz, 10.0);
            row.push_str(&match y { Some(y) => format!("{y:>6.2} "), None => "  none ".to_string() });
        }
        eprintln!("z {:>6.2}: {row}", gp[2] + dz);
    }

    // 2. Wider deck map: x 4..20, z 22..38, floor from y 10 (the skybridge band).
    eprintln!("== wider deck map x 4..20 z 22..38 (floor y from 10; '.....' none) ==");
    for zi in 0..9 {
        let z = 22.0 + zi as f32 * 2.0;
        let mut row = String::new();
        for xi in 0..9 {
            let x = 4.0 + xi as f32 * 2.0;
            let y = floor_at(world, x, z, 10.0);
            row.push_str(&match y { Some(y) => format!("{y:>6.2} "), None => "  .... ".to_string() });
        }
        eprintln!("z {z:>5.1}: {row}");
    }

    // 3. Stance scan on EVERY standable surface (the first hit from y 10 in 0..6): a clear
    //    bolt-legal aim to the point. Head = surface + 1.5; the bolt window is the point's sphere
    //    grown by the bolt radius 1.0 [M Bullet_init 0x12c5a0:92], the range the claw's 50.0 [L].
    eprintln!("== stance scan (all surfaces y 0..6): clear aim rays, closest approach <= rad+1.0, dist <= 50 ==");
    let mut hits: Vec<(f32, f32, f32, f32, f32)> = Vec::new(); // x, z, surf_y, dist, miss
    for zi in 0..30 {
        let z = 16.0 + zi as f32 * 1.0;
        for xi in 0..24 {
            let x = 0.0 + xi as f32 * 1.0;
            let Some(surf) = floor_at(world, x, z, 10.0) else { continue };
            if !(0.0..6.0).contains(&surf) { continue; }
            let head = [x, surf + 1.5, z];
            let to = [gp[0] - head[0], gp[1] - head[1], gp[2] - head[2]];
            let dist = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
            if dist > 50.0 { continue; }
            let yaw = to[0].atan2(to[2]);
            let horiz = (to[0] * to[0] + to[2] * to[2]).sqrt();
            let pitch = (to[1] / horiz.max(1e-4)).atan(); // atan2-style: valid past 45 degrees
            // scan the direct pitch and a little below/above (up to ~79 degrees)
            for dpi in -12..=8 {
                let p2 = pitch + dpi as f32 * 0.03;
                if p2 < -0.3 || p2 > 1.38 { continue; }
                let (sp, cp) = (p2.sin(), p2.cos());
                let (sy, cy) = (yaw.sin(), yaw.cos());
                let dir = [cp * sy, sp, cp * cy];
                let w = to;
                let proj = w[0] * dir[0] + w[1] * dir[1] + w[2] * dir[2];
                if proj <= 0.0 || proj > 50.0 { continue; }
                let cl = [head[0] + dir[0] * proj, head[1] + dir[1] * proj, head[2] + dir[2] * proj];
                let miss = ((cl[0] - gp[0]).powi(2) + (cl[1] - gp[1]).powi(2) + (cl[2] - gp[2]).powi(2)).sqrt();
                if miss > rad + 1.0 { continue; }
                let end = [head[0] + dir[0] * proj, head[1] + dir[1] * proj, head[2] + dir[2] * proj];
                if world.segment_clear(head, end) {
                    hits.push((x, z, surf, dist, miss));
                    break;
                }
            }
        }
    }
    hits.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
    for (x, z, fl, dist, miss) in hits.iter().take(30) {
        eprintln!("stance ({x:.1}, {z:.1}) floor {fl:.2}: dist {dist:.2} miss {miss:.2}");
    }
    eprintln!("== {} clear stances total ==", hits.len());

    // 3b. Blocker diagnosis: for a few representative stances, where does the DIRECT ray stop?
    for (label, x, z, sy) in [
        ("office SE (14, 28)", 14.0f32, 28.0, 0.0f32),
        ("office SE (16, 30)", 16.0, 30.0, 0.0),
        ("office (12, 26)", 12.0, 26.0, 0.0),
        ("deck 3.25 (8, 33)", 8.0, 33.0, 3.25),
        ("deck 3.25 (10, 31)", 10.0, 31.0, 3.25),
        ("band 4.58 (7, 24)", 7.0, 24.0, 4.58),
        ("office mid (12, 20)", 12.0, 20.0, 0.0),
    ] {
        let head = [x, sy + 1.5, z];
        let to = [gp[0] - head[0], gp[1] - head[1], gp[2] - head[2]];
        let dist = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let yaw = to[0].atan2(to[2]);
        let horiz = (to[0] * to[0] + to[2] * to[2]).sqrt();
        let pitch = (to[1] / horiz.max(1e-4)).atan();
        let (sp, cp) = (pitch.sin(), pitch.cos());
        let dir = [cp * yaw.sin(), sp, cp * yaw.cos()];
        let end = [head[0] + dir[0] * dist, head[1] + dir[1] * dist, head[2] + dir[2] * dist];
        match world.segment(head, end) {
            Some(t) => {
                let h = [head[0] + (end[0] - head[0]) * t, head[1] + (end[1] - head[1]) * t, head[2] + (end[2] - head[2]) * t];
                eprintln!("blocker {label} head {:.2}: hit ({:.2}, {:.2}, {:.2}) at t {t:.2} of {dist:.2}", head[1], h[0], h[1], h[2]);
            }
            None => eprintln!("blocker {label} head {:.2}: direct ray CLEAR, dist {dist:.2}", head[1]),
        }
    }

    // 4. The route waypoints' aims specifically (tower2 walk_12's legs).
    for (label, x, z) in [("lobby south", 14.0f32, 6.0), ("floor south 2", 10.0, 2.0),
        ("keypad approach", 9.3, -0.6), ("console", -1.55, 7.2), ("lift button", 1.1, 7.4)] {
        let fl = floor_at(world, x, z, 3.0).unwrap_or(f32::NAN);
        let head = [x, fl + 1.5, z];
        let to = [gp[0] - head[0], gp[1] - head[1], gp[2] - head[2]];
        let dist = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let yaw = to[0].atan2(to[2]);
        let horiz = (to[0] * to[0] + to[2] * to[2]).sqrt();
        let pitch = (to[1] / horiz.max(1e-4)).atan();
        let mut found = 0;
        let mut first = String::new();
        for dpi in -12..=8 {
            let p2 = pitch + dpi as f32 * 0.03;
            if p2 < -0.3 || p2 > 1.38 { continue; }
            let (sp, cp) = (p2.sin(), p2.cos());
            let (sy, cy) = (yaw.sin(), yaw.cos());
            let dir = [cp * sy, sp, cp * cy];
            let proj = to[0] * dir[0] + to[1] * dir[1] + to[2] * dir[2];
            if proj <= 0.0 || proj > 50.0 { continue; }
            let cl = [head[0] + dir[0] * proj, head[1] + dir[1] * proj, head[2] + dir[2] * proj];
            let miss = ((cl[0] - gp[0]).powi(2) + (cl[1] - gp[1]).powi(2) + (cl[2] - gp[2]).powi(2)).sqrt();
            if miss > rad + 1.0 { continue; }
            let end = [head[0] + dir[0] * proj, head[1] + dir[1] * proj, head[2] + dir[2] * proj];
            if world.segment_clear(head, end) {
                found += 1;
                if first.is_empty() { first = format!("pitch {p2:.2} miss {miss:.2} dist {dist:.2}"); }
            }
        }
        eprintln!("route {label} ({x}, floor {fl:.2}, {z}): {found} clear aims; {first}");
    }

    // 5. The hang would sit ~<=2.0 from the point toward the best stance; drop-map under the hang.
    //    Use the closest clear stance found (or straight down if none).
    let hang = if let Some(&(x, z, fl, _, _)) = hits.first() {
        let head = [x, fl + 1.5, z];
        let to = [gp[0] - head[0], gp[1] - head[1], gp[2] - head[2]];
        let dist = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        [gp[0] - to[0] / dist * 1.9, gp[1] - to[1] / dist * 1.9, gp[2] - to[2] / dist * 1.9]
    } else { [gp[0], gp[1] - 1.9, gp[2]] };
    eprintln!("== hang candidate {hang:?} ==");
    for dz in [-2.0f32, -1.0, 0.0, 1.0, 2.0] {
        let mut row = String::new();
        for dx in [-2.0f32, -1.0, 0.0, 1.0, 2.0] {
            let y = floor_at(world, hang[0] + dx, hang[2] + dz, hang[1]);
            row.push_str(&match y { Some(y) => format!("{y:>6.2} "), None => "  none ".to_string() });
        }
        eprintln!("z {:>6.2}: {row}", hang[2] + dz);
    }
}
