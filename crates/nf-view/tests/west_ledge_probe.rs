//! Diagnostic probe for the west-ledge cable route (level-05-west-ledge): what blocks the ledge
//! west of ladder #151's top (the x≈58 stall), and which instances sit there. Diagnostic only —
//! run with --nocapture.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Ray sweep + floor scan from the ladder top.
#[test]
fn probe_what_blocks_the_ledge() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let ladder_top = [61.64f32, 34.62, 89.20];
    eprintln!("=== ray sweep from the ladder top (16 yaws x heights +0.2/+1.0/+1.8, len 24) ===");
    for h in [0.2f32, 1.0, 1.8] {
        for k in 0..16 {
            let yaw = k as f32 * std::f32::consts::TAU / 16.0;
            let dir = [yaw.sin(), 0.0, yaw.cos()];
            let from = [ladder_top[0], ladder_top[1] + h, ladder_top[2]];
            let to = [from[0] + dir[0] * 24.0, from[1], from[2] + dir[2] * 24.0];
            if let Some((f, p)) = sim.world.segment_hit(from, to) {
                eprintln!("h{h:.1} yaw{k:2} ({:5.1} rad) hit f={f:.3} at ({:.2},{:.2},{:.2})", yaw, p[0], p[1], p[2]);
            }
        }
    }
    eprintln!("=== full-band grid (floor in 30..52; x -33..70 step 2, z -6..130 step 2, from y 56 down 26) ===");
    let mut z = 128.0f32;
    while z >= -6.0 {
        let mut row = format!("z{z:6.1} |");
        let mut x = -33.0f32;
        while x <= 70.0 {
            let fl = sim.world.floor_below([x, 56.0, z], 26.0);
            row.push_str(&match fl { Some(f) if (30.0..52.0).contains(&f) => format!("{f:4.0}"), _ => format!("   .") });
            x += 2.0;
        }
        eprintln!("{row}");
        z -= 2.0;
    }
    eprintln!("=== ledge-band grid (floor in 33.5..37.0; x -33..70 step 3, z -6..104 step 3) ===");
    let mut z = 102.0f32;
    while z >= -6.0 {
        let mut row = format!("z{z:6.1} |");
        let mut x = -33.0f32;
        while x <= 70.0 {
            let fl = sim.world.floor_below([x, 40.0, z], 14.0);
            row.push_str(&match fl { Some(f) if (33.5..37.0).contains(&f) => format!("{f:4.1}"), _ => format!("   .") });
            x += 3.0;
        }
        eprintln!("{row}");
        z -= 3.0;
    }
    eprintln!("=== fine floor grid (x 46..64, z 78..100, step 1.5/1.5, from y 40 down 12) ===");
    let mut z = 78.0f32;
    while z <= 100.0 {
        let mut row = format!("z{z:5.1} |");
        let mut x = 46.0f32;
        while x <= 64.0 {
            let fl = sim.world.floor_below([x, 40.0, z], 12.0);
            row.push_str(&match fl { Some(f) => format!("{f:5.1}"), None => format!("    .") });
            x += 1.5;
        }
        eprintln!("{row}");
        z += 1.5;
    }
    eprintln!("=== floor_below along the west lane (z 88.5 and z 94, x 62→40) ===");
    for z in [88.5f32, 94.0] {
        let mut x = 62.0f32;
        while x >= 40.0 {
            let fl = sim.world.floor_below([x, 37.0, z], 10.0);
            eprintln!("z{z:5.1} x{x:5.1} floor {:?}", fl.map(|f| format!("{f:.2}")));
            x -= 1.5;
        }
    }
    eprintln!("=== floor_below along the north lane (x 58, z 90→110) ===");
    for z in [90.0f32, 95.0, 100.0, 105.0, 110.0] {
        let fl = sim.world.floor_below([58.0, 37.0, z], 10.0);
        eprintln!("x58 z{z:5.1} floor {:?}", fl.map(|f| format!("{f:.2}")));
    }
}

/// The instances (placements + collision objects) near the stall region, and which one each
/// ladder-top ray lands in.
#[test]
fn probe_instances_near_the_stall() {
    use nf_formats::disc::files_bin_entries;
    use nf_formats::iso9660::IsoImage;
    use nf_formats::level::Level;
    let path = iso();
    let iso = IsoImage::open(&path).expect("iso");
    let (entries, bin) = files_bin_entries(&iso).expect("FILES.BIN");
    let read = |name: &str| -> Option<Vec<u8>> {
        entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
            .and_then(|fe| iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok())
    };
    let data = read("07000005.bin").expect("07000005.bin");
    let level = Level::parse(&data).expect("parse");
    let m = level.main_map().expect("main map");
    eprintln!("=== instances near the stall (x 40..72, y 20..42, z 70..102) ===");
    let (instances, missing) = level.instances(false);
    eprintln!("instances {} (missing {missing})", instances.len());
    for inst in &instances {
        let t = inst.matrix[3];
        if !(40.0..72.0).contains(&t[0]) || !(20.0..42.0).contains(&t[1]) || !(70.0..102.0).contains(&t[2]) { continue; }
        let obj = &level.maps[inst.map].objects[inst.object];
        let (tris, boxes) = match &obj.collision {
            Some(c) => (c.tris.len(), c.boxes.len()),
            None => (0, 0),
        };
        eprintln!(
            "plc #{} hash {:#x} flags {:#x} at ({:.2},{:.2},{:.2}) obj centre ({:.1},{:.1},{:.1}) r {:.1} coll tris {tris} boxes {boxes}",
            inst.placement.index, inst.placement.hashcode, inst.placement.flags, t[0], t[1], t[2],
            obj.centre[0], obj.centre[1], obj.centre[2], obj.radius);
        for (k, v) in inst.placement.params.iter().take(6) { eprintln!("    param{k} = {v:#x} ({v})"); }
    }
    eprintln!("=== which instance does each ladder-top ray land near? ===");
    let scene = nf_view::load(&path, "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let ladder_top = [61.64f32, 34.62, 89.20];
    for k in [12u32, 13, 14, 15, 0, 1, 2, 3, 4] {
        let yaw = k as f32 * std::f32::consts::TAU / 16.0;
        let dir = [yaw.sin(), 0.0, yaw.cos()];
        let from = [ladder_top[0], ladder_top[1] + 1.0, ladder_top[2]];
        let to = [from[0] + dir[0] * 24.0, from[1], from[2] + dir[2] * 24.0];
        if let Some((f, p)) = sim.world.segment_hit(from, to) {
            // nearest instance translation to the hit point
            let mut best: Option<(f32, u16, u32)> = None;
            for inst in &instances {
                let t = inst.matrix[3];
                let d = ((t[0] - p[0]).powi(2) + (t[1] - p[1]).powi(2) + (t[2] - p[2]).powi(2)).sqrt();
                if best.map_or(true, |(bd, _, _)| d < bd) { best = Some((d, inst.placement.index, inst.placement.hashcode)); }
            }
            eprintln!("yaw{k:2} f={f:.3} hit ({:.2},{:.2},{:.2}) nearest plc {:?} (d {:.1})",
                p[0], p[1], p[2], best.map(|(_, i, h)| format!("#{} {:#x}", i, h)), best.map(|(d, _, _)| d).unwrap_or(-1.0));
        } else {
            eprintln!("yaw{k:2} clear");
        }
    }
}

/// BFS the walkable web from three seeds (ladder top / the wire's west ledge / the courtyard
/// spawn): adjacent cells 1.0 apart, |Δfloor| ≤ 1.2, the mid-height segment between clear. Prints
/// the per-seed reach and the shared ASCII map, plus the game's own route markers (the ledge
/// cut-scene plates, the drones, the icons) so the intended lane is visible.
#[test]
fn probe_walkable_web_bfs() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    // floor cache
    let x0 = -70i32; let x1 = 75i32; let z0 = -80i32; let z1 = 132i32;
    let n = ((x1 - x0 + 1) * (z1 - z0 + 1)) as usize;
    let mut floor: Vec<Option<f32>> = vec![None; n];
    let idx = |x: i32, z: i32| ((z - z0) * (x1 - x0 + 1) + (x - x0)) as usize;
    for z in z0..=z1 { for x in x0..=x1 {
        floor[idx(x, z)] = sim.world.floor_below([x as f32, 50.0, z as f32], 30.0).filter(|f| (26.0..52.0).contains(f));
    }}
    let seeds: [(&str, [f32; 2], usize); 6] = [
        ("ladder-top", [61.6, 89.2], 1),
        ("wire-ledge", [-23.5, 6.3], 2),
        ("west-ledge-scene", [-33.0, -32.0], 3),
        ("zip-door", [41.5, -69.7], 4),
        ("east-west-ledge", [24.9, 67.6], 5),
        ("gatehouse-north", [62.4, 104.1], 6),
    ];
    let mut mark: Vec<u8> = vec![0; n];
    for (_, s, id) in seeds {
        let (sx, sz) = (s[0].round() as i32, s[1].round() as i32);
        let start = idx(sx, sz);
        if floor[start].is_none() {
            // search a 3x3 for a stand cell
            let mut found = false;
            'outer: for dz in -3..=3 { for dx in -3..=3 {
                let (qx, qz) = (sx + dx, sz + dz);
                if qx < x0 || qx > x1 || qz < z0 || qz > z1 { continue; }
                let q = idx(qx, qz);
                if floor[q].is_some() { mark[q] = id as u8; found = true; break 'outer; }
            }}
            eprintln!("seed {id} ({sx},{sz}) stand cell found={found}");
            if !found { continue; }
        } else { mark[start] = id as u8; }
        let mut queue = std::collections::VecDeque::new();
        for i in 0..n { if mark[i] == id as u8 { queue.push_back(i); } }
        while let Some(i) = queue.pop_front() {
            let x = (i as i32 % (x1 - x0 + 1)) + x0;
            let z = (i as i32 / (x1 - x0 + 1)) + z0;
            let f = floor[i].unwrap();
            for (dx, dz) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx; let nz = z + dz;
                if nx < x0 || nx > x1 || nz < z0 || nz > z1 { continue; }
                let j = idx(nx, nz);
                if mark[j] != 0 { continue; }
                let Some(g) = floor[j] else { continue };
                let drop = f - g; // >0 = falling off an edge (real movement: walk off, land lower)
                if drop < -1.2 || drop > 14.0 { continue; }
                let h = f.min(g) + 1.0;
                let a = [x as f32, h, z as f32];
                let b = [nx as f32, h, nz as f32];
                if sim.world.segment(a, b).is_some() { continue; }
                mark[j] = id as u8;
                queue.push_back(j);
            }
        }
        let cells = mark.iter().filter(|&&m| m == id as u8).count();
        let (mut mnx, mut mxx, mut mnz, mut mxz) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
        for i in 0..n { if mark[i] == id as u8 {
            let x = (i as i32 % (x1 - x0 + 1)) + x0;
            let z = (i as i32 / (x1 - x0 + 1)) + z0;
            mnx = mnx.min(x); mxx = mxx.max(x); mnz = mnz.min(z); mxz = mxz.max(z);
        }}
        eprintln!("seed {id} reach {cells} cells bbox x {mnx}..{mxx} z {mnz}..{mxz}");
        // the 12 westernmost, 12 southernmost and 12 northernmost cells with their floors
        let mut cells_v: Vec<(i32, i32, f32)> = (0..n).filter_map(|i| if mark[i] == id as u8 {
            let x = (i as i32 % (x1 - x0 + 1)) + x0;
            let z = (i as i32 / (x1 - x0 + 1)) + z0;
            Some((x, z, floor[i].unwrap()))
        } else { None }).collect();
        cells_v.sort_by_key(|(x, _, _)| *x);
        let west: Vec<String> = cells_v.iter().take(12).map(|(x, z, f)| format!("({x},{z},{f:.1})")).collect();
        eprintln!("  west: {}", west.join(" "));
        cells_v.sort_by_key(|(_, z, _)| *z);
        let south: Vec<String> = cells_v.iter().take(12).map(|(x, z, f)| format!("({x},{z},{f:.1})")).collect();
        eprintln!("  south: {}", south.join(" "));
        let north: Vec<String> = cells_v.iter().rev().take(12).map(|(x, z, f)| format!("({x},{z},{f:.1})")).collect();
        eprintln!("  north: {}", north.join(" "));
    }
    eprintln!("=== web map (1 ladder-top 2 wire-ledge 3 courtyard, X overlap; '.' floor-no-seed, ' ' none) ===");
    let mut z = z1;
    while z >= z0 {
        let mut row = format!("z{z:5} ");
        let mut x = x0;
        while x <= x1 {
            let i = idx(x, z);
            let m = mark[i];
            let c = if m != 0 { std::char::from_digit(m as u32, 10).unwrap() } else if floor[i].is_some() { '.' } else { ' ' };
            row.push(c);
            x += 2;
        }
        eprintln!("{row}");
        z -= 2;
    }
    eprintln!("=== the game's markers in the upper region (y > 28) ===");
    for t in &sim.drones.triggers {
        let c = t.centre();
        if c[1] > 28.0 && !t.tris.is_empty() {
            eprintln!("trig out={:#04x} mask={:#04x} lock={} at ({:.1},{:.1},{:.1})", t.out, t.mask, t.lock, c[0], c[1], c[2]);
        }
    }
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.pos[1] > 28.0 && !d.corpse && d.health > 0.0 {
            eprintln!("drone[{i}] at ({:.1},{:.1},{:.1}) home ({:.1},{:.1},{:.1}) state {:#x}", d.pos[0], d.pos[1], d.pos[2], d.home[0], d.home[1], d.home[2], d.state);
        }
    }
    for g in &sim.drones.grapple_points {
        eprintln!("grapple at ({:.1},{:.1},{:.1})", g.pos[0], g.pos[1], g.pos[2]);
    }
    for w in &sim.wires {
        eprintln!("wire centre ({:.2},{:.2},{:.2}) r {:.2} yaw {:.2} type {}", w.centre[0], w.centre[1], w.centre[2], w.radius, w.yaw, w.wire_type);
        let (a, b) = w.grab_segment();
        eprintln!("  grab segment a ({:.2},{:.2},{:.2}) b ({:.2},{:.2},{:.2})", a[0], a[1], a[2], b[0], b[1], b[2]);
        let (a, b) = w.travel_segment();
        eprintln!("  travel segment a ({:.2},{:.2},{:.2}) b ({:.2},{:.2},{:.2})", a[0], a[1], a[2], b[0], b[1], b[2]);
    }
}

/// Dense sweep + direct island→strip rays: where is the opening in the parapet ring?
#[test]
fn probe_island_openings() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 72-yaw sweep from the island centre (61.6, 34.8+, 89.2), len 8 ===");
    for h in [0.3f32, 0.8, 1.3, 1.8, 2.3] {
        let mut clear_yaw = Vec::new();
        for k in 0..72 {
            let yaw = k as f32 * std::f32::consts::TAU / 72.0;
            let dir = [yaw.sin(), 0.0, yaw.cos()];
            let from = [61.6, 34.8 + h, 89.2];
            let to = [from[0] + dir[0] * 8.0, from[1], from[2] + dir[2] * 8.0];
            match sim.world.segment_hit(from, to) {
                None => clear_yaw.push((k, 8.0)),
                Some((f, _)) if f * 8.0 > 2.0 => clear_yaw.push((k, f * 8.0)),
                _ => {}
            }
        }
        if clear_yaw.is_empty() { eprintln!("h{h:.1}: all blocked within 2"); }
        else {
            let s = clear_yaw.iter().map(|(k, d)| format!("yaw{k}({d:.1})")).collect::<Vec<_>>().join(" ");
            eprintln!("h{h:.1}: clear {s}");
        }
    }
    eprintln!("=== direct segments island→north strip ===");
    for (a, b) in [
        ([61.6f32, 35.6, 90.0], [59.5, 35.0, 92.5]),
        ([61.6, 35.6, 90.0], [61.6, 35.0, 92.5]),
        ([60.5, 35.6, 90.0], [59.5, 35.0, 92.5]),
        ([62.4, 35.6, 90.0], [62.4, 35.0, 92.5]),
        ([61.6, 34.9, 90.2], [61.6, 34.9, 92.5]),
        ([60.9, 34.9, 90.2], [59.5, 34.9, 92.5]),
    ] {
        let f = sim.world.segment(a, b);
        eprintln!("from ({:.1},{:.1},{:.1}) to ({:.1},{:.1},{:.1}): {:?}", a[0], a[1], a[2], b[0], b[1], b[2], f);
    }
    eprintln!("=== vertical profile x 59.0..63.0 (step 0.5) at z 90..96: floor(from y 38, depth 8) ===");
    for z in [90.0f32, 91.0, 92.0, 93.0, 94.0, 95.0, 96.0] {
        let mut row = format!("z{z:4.1} ");
        let mut x = 58.0f32;
        while x <= 64.0 {
            let fl = sim.world.floor_below([x, 38.0, z], 8.0);
            row.push_str(&match fl { Some(f) => format!("{f:5.1}"), None => format!("    .") });
            x += 0.5;
        }
        eprintln!("{row}");
    }
    eprintln!("=== same but from y 34.9 down 4 (what is AT feet level?) ===");
    for z in [90.0f32, 91.0, 92.0, 93.0] {
        let mut row = format!("z{z:4.1} ");
        let mut x = 58.0f32;
        while x <= 64.0 {
            let fl = sim.world.floor_below([x, 34.9, z], 4.0);
            row.push_str(&match fl { Some(f) => format!("{f:5.1}"), None => format!("    .") });
            x += 0.5;
        }
        eprintln!("{row}");
    }
}

/// Manual real-input drive from the ladder top: north through the parapet slot, along the strip.
/// Teleport only TO the ladder top (the walk test itself climbs for real); every step after is
/// real step_bond input.
#[test]
fn probe_drive_north_from_the_ladder_top() {
    use nf_game::player::Input;
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [61.64, 34.72, 89.20];
    sim.bond.yaw = 0.0; // facing +z? the walk uses atan2 conventions — try both
    sim.bond.vel = [0.0; 3];
    let mut input = Input::default();
    // face north (+z): the walk code's desired = atan2(dx, dz); yaw 0 → forward (0,0,+1)?
    for t in 0..600u32 {
        input.forward = 1.0;
        sim.step_bond(input, false, false, false);
        if t % 60 == 0 {
            eprintln!("[drive fwd yaw0 t{t}] bond ({:.2},{:.2},{:.2}) grounded {}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded);
        }
    }
    // now walk back and try the slot at x 61.9 heading north
    sim.bond.pos = [61.9, 34.72, 89.8];
    sim.bond.vel = [0.0; 3];
    for t in 0..600u32 {
        input.forward = 1.0;
        sim.step_bond(input, false, false, false);
        if t % 60 == 0 {
            eprintln!("[drive fwd slot t{t}] bond ({:.2},{:.2},{:.2}) grounded {}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded);
        }
    }
}

/// Fine scan of the x≈62.4 lane the island guard's patrol runs along.
#[test]
fn probe_the_x62_lane() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== floor (from y 38, depth 8) x 61.8..63.4 step 0.1, z 88..108 step 1 ===");
    for z in 88..=108i32 {
        let mut row = format!("z{z:4} ");
        let mut x = 61.8f32;
        while x <= 63.45 {
            let fl = sim.world.floor_below([x, 38.0, z as f32], 8.0);
            row.push_str(&match fl { Some(f) => format!("{f:5.1}"), None => format!("    .") });
            x += 0.1;
        }
        eprintln!("{row}");
    }
    eprintln!("=== walls: segments from (62.4, y, 89.0) north to (62.4, y, 92.0) for y 34.4..37 ===");
    for y in [34.4f32, 34.7, 35.0, 35.3, 35.6, 36.0, 36.4, 36.8] {
        let f = sim.world.segment([62.4, y, 89.0], [62.4, y, 92.0]);
        eprintln!("y{y:.1}: {:?}", f);
    }
}

/// The level's AI points (the guards' nav graph) — the game's own route map over the ledges.
#[test]
fn probe_ai_points() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== ai_points ({}) ===", sim.drones.ai_points.len());
    for (i, a) in sim.drones.ai_points.iter().enumerate() {
        if a.pos[1] > 28.0 {
            eprintln!("ai[{i}] kind {} ({:.1},{:.1},{:.1})", a.kind, a.pos[0], a.pos[1], a.pos[2]);
        }
    }
}

/// Jump the parapet: real input (forward + jump) from the island — where does Bond land?
#[test]
fn probe_jump_the_parapet() {
    use nf_game::player::Input;
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for yaw in [0.0f32, 0.3, -0.3] {
        sim.bond.pos = [61.64, 34.72, 89.20];
        sim.bond.yaw = yaw;
        sim.bond.vel = [0.0; 3];
        let mut input = Input { forward: 1.0, ..Default::default() };
        let mut jumped = false;
        for t in 0..240u32 {
            input.jump = !jumped && t == 20;
            if t == 20 { jumped = true; }
            sim.step_bond(input, false, false, false);
            if t % 20 == 0 {
                eprintln!("[jump yaw{yaw} t{t}] ({:.2},{:.2},{:.2}) grounded {}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded);
            }
        }
        eprintln!("[jump yaw{yaw}] end ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    }
}

/// The icons and creep walls (the game's route mechanics markers).
#[test]
fn probe_icons_and_creepwalls() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== icons ({}) ===", sim.icons.len());
    for ic in &sim.icons {
        eprintln!("icon ctx {} at ({:.2},{:.2},{:.2}) r {:.2} on_ch {}", ic.ctx, ic.pos[0], ic.pos[1], ic.pos[2], ic.radius, ic.on_channel);
    }
    eprintln!("=== creep walls (from the level parse) ===");
    for (i, cw) in sim.creepwalls.iter().enumerate() {
        eprintln!("creep[{i}] at ({:.2},{:.2},{:.2}) yaw {:.2} half_len {:.2}", cw.pos[0], cw.pos[1], cw.pos[2], cw.yaw, cw.half_len);
    }
}

/// The west lane (x −29..−18, z 0..46): floors + what breaks the connection between the wire
/// ledge (z ≤ 4) and the big web's west end (z ≥ 42).
#[test]
fn probe_west_lane() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== west lane floors (x -29..-18 step 1 across, z 46..0 step 1 down; from y 40 depth 16) ===");
    eprintln!("      x: -29 -28 -27 -26 -25 -24 -23 -22 -21 -20 -19 -18");
    for z in (0..=46i32).rev() {
        let mut row = format!("z{z:4} ");
        for x in -29..=-18i32 {
            let fl = sim.world.floor_below([x as f32, 40.0, z as f32], 16.0);
            row.push_str(&match fl { Some(f) if f < 40.0 => format!("{f:5.1}"), _ => format!("    .") });
        }
        eprintln!("{row}");
    }
    eprintln!("=== north-of-wire pieces: floors x -27..-12 step 1, z 8..20 (from y 40 depth 16) ===");
    for z in (8..=20i32).rev() {
        let mut row = format!("z{z:4} ");
        for x in -27..=-12i32 {
            let fl = sim.world.floor_below([x as f32, 40.0, z as f32], 16.0);
            row.push_str(&match fl { Some(f) if f < 40.0 => format!("{f:5.1}"), _ => format!("    .") });
        }
        eprintln!("{row}");
    }
}

/// Pickup positions (is a bonus token on the wire ledge?) + the full real-input drive:
/// island → walk off west → the roofs → the west lane → the wire grab → the ride.
#[test]
fn probe_pickups_and_full_drive() {
    use nf_game::player::Input;
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== pickups ===");
    for (i, pk) in sim.drones.pickups.iter().enumerate() {
        eprintln!("pickup[{i}] at=({:.1},{:.1},{:.1}) kind={:#x} ch={:#x}", pk.m[3][0], pk.m[3][1], pk.m[3][2], pk.kind, pk.switch_channel);
    }
    drop(sim);
    // ── the drive: teleport to the island top (the walk test climbs for real), then all-real input ──
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [61.64, 34.72, 89.20];
    sim.bond.yaw = 0.0;
    sim.bond.vel = [0.0; 3];
    let mut input = Input::default();
    // 1. walk WEST off the island's west edge (yaw π/2 → −x?) — the walk yaw convention: forward
    //    = (sin yaw, 0, cos yaw) per the drive probe (yaw 0 → +z). West = −x → yaw = −π/2? sin(−π/2) = −1 → −x. Yes.
    input.forward = 1.0;
    for t in 0..400u32 {
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.step_bond(input, false, false, false);
        if t % 40 == 0 { eprintln!("[d1 west t{t}] ({:.2},{:.2},{:.2}) g{}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded as u8); }
        if sim.bond.pos[0] < 55.0 { break; }
    }
    // 2. once fallen onto the roofs (y ~28-29), keep walking west to x −28
    for t in 0..1400u32 {
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.step_bond(input, false, false, false);
        if t % 100 == 0 { eprintln!("[d2 roofs t{t}] ({:.2},{:.2},{:.2}) g{}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded as u8); }
        if sim.bond.pos[0] <= -26.5 { break; }
    }
    // 3. south along the west lane to the wire's north grab zone (z ~11)
    for t in 0..1600u32 {
        sim.bond.yaw = std::f32::consts::PI; // forward = (sin π, cos π) = (0, −1) = −z = south
        sim.step_bond(input, false, false, false);
        if t % 100 == 0 { eprintln!("[d3 south t{t}] ({:.2},{:.2},{:.2}) g{}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded as u8); }
        if sim.bond.pos[2] <= 12.0 { break; }
    }
    eprintln!("[d3 end] bond ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
}

/// The jump's real numbers: flat-ground run + jump (apex height, distance), then the island
/// run-up jump at the north parapet.
#[test]
fn probe_jump_numbers() {
    use nf_game::player::Input;
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    // flat ground: the courtyard spawn
    sim.bond.pos = [104.95, 25.4, 108.83];
    sim.bond.yaw = -std::f32::consts::FRAC_PI_2; // west
    sim.bond.vel = [0.0; 3];
    let mut input = Input { forward: 1.0, ..Default::default() };
    let (mut apex, mut start_z, mut jumped) = (0.0f32, sim.bond.pos[2], false);
    for t in 0..200u32 {
        input.jump = t == 40;
        if t == 40 { jumped = true; start_z = sim.bond.pos[2]; }
        sim.step_bond(input, false, false, false);
        apex = apex.max(sim.bond.pos[1]);
        if t % 20 == 0 { eprintln!("[flat t{t}] ({:.2},{:.2},{:.2}) g{}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded as u8); }
    }
    eprintln!("[flat] apex {:.2} (start 25.4, rise {:.2}), jump z-span {:.2}", apex, apex - 25.4, start_z - sim.bond.pos[2]);
    // island: run north + jump at the parapet
    sim.bond.pos = [61.64, 34.72, 86.8];
    sim.bond.yaw = 0.0; // +z north
    sim.bond.vel = [0.0; 3];
    let mut input = Input { forward: 1.0, ..Default::default() };
    let mut apex = 0.0f32;
    for t in 0..300u32 {
        input.jump = t == 14;
        sim.step_bond(input, false, false, false);
        apex = apex.max(sim.bond.pos[1]);
        if t % 15 == 0 { eprintln!("[isl t{t}] ({:.2},{:.2},{:.2}) g{}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded as u8); }
        if sim.bond.pos[2] > 92.0 { break; }
    }
    eprintln!("[isl] apex {:.2} end ({:.2},{:.2},{:.2})", apex, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
}

/// The walking path (BFS with fall edges + parents) from the fall landing to the wire's north
/// grab zone — the waypoint list the walk test drives.
#[test]
fn probe_the_path_to_the_wire() {
    let scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let x0 = -70i32; let x1 = 75i32; let z0 = -80i32; let z1 = 132i32;
    let w = (x1 - x0 + 1) as usize;
    let n = w * ((z1 - z0 + 1) as usize);
    let idx = |x: i32, z: i32| (((z - z0) as usize) * w + ((x - x0) as usize));
    let mut floor: Vec<Option<f32>> = vec![None; n];
    for z in z0..=z1 { for x in x0..=x1 {
        floor[idx(x, z)] = sim.world.floor_below([x as f32, 50.0, z as f32], 30.0).filter(|f| (20.0..52.0).contains(f));
    }}
    let start = idx(59, 89);
    let goal = idx(-24, 13);
    let mut prev: Vec<Option<usize>> = vec![None; n];
    let mut seen = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    seen[start] = true;
    queue.push_back(start);
    'bfs: while let Some(i) = queue.pop_front() {
        let x = (i % w) as i32 + x0;
        let z = (i / w) as i32 + z0;
        let f = floor[i].unwrap();
        for (dx, dz) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let nx = x + dx; let nz = z + dz;
            if nx < x0 || nx > x1 || nz < z0 || nz > z1 { continue; }
            let j = idx(nx, nz);
            if seen[j] { continue; }
            let Some(g) = floor[j] else { continue };
            let drop = f - g;
            if drop < -1.2 || drop > 14.0 { continue; }
            let h = f.min(g) + 1.0;
            let a = [x as f32, h, z as f32];
            let b = [nx as f32, h, nz as f32];
            if sim.world.segment(a, b).is_some() { continue; }
            seen[j] = true;
            prev[j] = Some(i);
            if j == goal { break 'bfs; }
            queue.push_back(j);
        }
    }
    if !seen[goal] { eprintln!("[path] NO PATH from the landing to the wire grab zone"); return; }
    let mut path = Vec::new();
    let mut cur = Some(goal);
    while let Some(c) = cur { path.push(c); cur = prev[c]; }
    path.reverse();
    eprintln!("[path] {} cells", path.len());
    for (k, i) in path.iter().enumerate() {
        if k % 10 == 0 || k == path.len() - 1 {
            let x = (i % w) as i32 + x0;
            let z = (i / w) as i32 + z0;
            eprintln!("[path {k:4}] ({x},{z},{:.1})", floor[*i].unwrap());
        }
    }
}
