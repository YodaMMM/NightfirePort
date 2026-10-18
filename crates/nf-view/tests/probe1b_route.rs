//! level-1b-complete probe: thread the hub crossing. The walkreal_0700001b walk fell into the
//! pit (y −15.4) and `follow_nav_route` finds no nav graph (the station's drones hover — no
//! ground AI paths over the hub). This probe BFSes `floor_below` at a fine step from the spawn
//! platform to the Drake arena and prints a walkable waypoint chain. Assertion-free.

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }


#[test]
fn probe_column_scans() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load");
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let pts: &[(&str, f32, f32)] = &[
        ("spawn", 22.98, 3.13), ("hub-mid", 14.0, 1.5), ("lane?", 5.0, 0.0),
        ("arena#", 2.75, 0.0), ("drake", -7.32, 0.06), ("arena720a", -5.81, 0.14),
        ("pit", 0.0, -4.0), ("west", -10.0, 0.0),
    ];
    for (name, x, z) in pts.iter() {
        let mut row = format!("{:>9} ({:6.2},{:6.2}):", name, x, z);
        let mut y = 12.0f32;
        while y > -17.0 {
            match sim.world.floor_below([*x, y, *z], 0.9) {
                Some(f) => { row.push_str(&format!(" {:7.2}", f)); y = f - 0.9; }
                None => y -= 0.9,
            }
        }
        eprintln!("{row}");
    }
}


#[test]
fn probe_south_west_map() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load");
    for _ in 0..320 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    // The south/west band: x -12..28, z -16..2, step 1.0; '#' = floor in [-13,-0.6], '.' = deeper floor.
    eprintln!("=== south/west band (rows z 2..-16, cols x -12..28, step 1.0) ===");
    let mut z = 2.0f32;
    while z >= -16.0 {
        let mut row = format!("z {:6.1}: ", z);
        let mut x = -12.0f32;
        while x <= 28.0 {
            match sim.world.floor_below([x, 0.5, z], 18.0) {
                Some(f) if (-13.0..=-0.6).contains(&f) => row.push('#'),
                Some(_) => row.push('.'),
                None => row.push(' '),
            }
            x += 1.0;
        }
        eprintln!("{row}");
        z -= 1.0;
    }
}

#[test]
fn probe_hub_route_bfs() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load");
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();

    let step = 0.75f32;
    let x0 = -10.0f32; let x1 = 24.0f32; let z0 = -6.0f32; let z1 = 8.0f32;
    let plane = 0.0f32; // sample from above every deck; the floor band picks the route lane
    let depth = 6.0f32;  // the floor within 6 below y=0: the hub route band (root = floor + cc)

    let nx = ((x1 - x0) / step) as usize + 1;
    let nz = ((z1 - z0) / step) as usize + 1;
    let idx = |ix: usize, iz: usize| iz * nx + ix;
    let mut walk = vec![false; nx * nz];
    let mut fy = vec![0f32; nx * nz];
    for iz in 0..nz {
        for ix in 0..nx {
            let x = x0 + ix as f32 * step;
            let z = z0 + iz as f32 * step;
            if let Some(y) = sim.world.floor_below([x, plane, z], depth) {
                if (-4.2..=-0.6).contains(&y) { walk[idx(ix, iz)] = true; fy[idx(ix, iz)] = y; }
            }
        }
    }
    // Print the map ('#' walkable, '.' floor deeper, ' ' void) for the record.
    eprintln!("=== hub walk map at plane {plane} (rows z {z1}..{z0}, cols x {x0}..{x1}, step {step}) ===");
    for iz in (0..nz).rev() {
        let mut row = format!("z {:6.2}: ", z0 + iz as f32 * step);
        for ix in 0..nx {
            row.push(if walk[idx(ix, iz)] { '#' } else if sim.world.floor_below([x0 + ix as f32 * step, plane, z0 + iz as f32 * step], 16.0).is_some() { '.' } else { ' ' });
        }
        eprintln!("{row}");
    }

    // BFS 8-connected from the spawn cell to the arena cell.
    let sx = ((22.98 - x0) / step) as usize;
    let sz = ((3.13 - z0) / step) as usize;
    let tx_x = ((-7.32 - x0) / step) as usize;
    let tx_z = ((0.06 - z0) / step) as usize;
    let (sn, tn) = (idx(sx, sz), idx(tx_x, tx_z));
    if !walk[sn] { eprintln!("spawn cell not walkable! ({sx},{sz})"); }
    if !walk[tn] { eprintln!("arena cell not walkable! ({tx_x},{tx_z})"); }
    let mut prev = vec![usize::MAX; nx * nz];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(sn);
    prev[sn] = sn;
    while let Some(c) = queue.pop_front() {
        if c == tn { break; }
        let (cx, cz) = (c % nx, c / nx);
        for dx in -1i32..=1 {
            for dz in -1i32..=1 {
                if dx == 0 && dz == 0 { continue; }
                let (nxp, nzp) = (cx as i32 + dx, cz as i32 + dz);
                if nxp < 0 || nzp < 0 || nxp >= nx as i32 || nzp >= nz as i32 { continue; }
                let n = idx(nxp as usize, nzp as usize);
                if walk[n] && prev[n] == usize::MAX { prev[n] = c; queue.push_back(n); }
            }
        }
    }
    if prev[tn] == usize::MAX {
        eprintln!("NO BFS ROUTE from the spawn to the arena");
        // Where DOES the walkable set reach? Print the BFS frontier extent.
        let mut reach = vec![false; nx * nz];
        for i in 0..nx * nz { if prev[i] != usize::MAX { reach[i] = true; } }
        eprintln!("=== the reachable set ===");
        for iz in (0..nz).rev() {
            let mut row = format!("z {:6.2}: ", z0 + iz as f32 * step);
            for ix in 0..nx { row.push(if reach[idx(ix, iz)] { 'o' } else if walk[idx(ix, iz)] { '#' } else { '.' }); }
            eprintln!("{row}");
        }
        return;
    }
    let mut path = vec![tn];
    let mut c = tn;
    while prev[c] != c { c = prev[c]; path.push(c); }
    path.reverse();
    eprintln!("=== BFS route: {} cells, {} m ===", path.len(), path.len() as f32 * step);
    for (k, &c) in path.iter().enumerate() {
        if k % 6 == 0 || k == path.len() - 1 {
            let (cx, cz) = (c % nx, c / nx);
            eprintln!("wp{k:3}: ({:.2}, {:.2}, {:.2}) floor {:.2}", x0 + cx as f32 * step, fy[c], z0 + cz as f32 * step, fy[c]);
        }
    }
}
