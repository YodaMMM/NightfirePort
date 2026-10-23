//! 979 diagnostic: the 0a west doorway floor profile and the 01 shoji threshold sweep.
#[path = "common/walker.rs"]
mod walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
#[test]
#[ignore = "dump"]
fn probe_floors() {
    // 0a: the west doorway floor profile (x across the doorway at z 4.0..5.5).
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0a");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 0a west doorway: floor_below from y 2, depth 4, rows z 3.5..5.75 step 0.25, cols x −20..−16.5 step 0.25 ===");
    let mut z = 3.5f32;
    while z <= 5.75 {
        let mut row = format!("z {:5.2}: ", z);
        let mut x = -20.0f32;
        while x <= -16.5 {
            match sim.world.floor_below([x, 2.0, z], 4.0) {
                Some(fy) => row.push_str(&format!("{:+.3} ", fy)),
                None => row.push_str("  ---- "),
            }
            x += 0.5;
        }
        eprintln!("{row}");
        z += 0.25;
    }
    eprintln!("fine z 4.4..5.0 at x −18.5, −18.0, −17.5:");
    for x in [-18.5f32, -18.0, -17.5] {
        let mut z = 4.4f32;
        while z <= 5.0 {
            let f = sim.world.floor_below([x, 2.0, z], 4.0);
            eprintln!("  x {x:+.2} z {z:+.2} → {f:?}");
            z += 0.1;
        }
    }
    // 01: the courtyard shoji threshold sweep (the 941 numbers).
    let scene1 = nf_view::load(&iso(), "07000001.bin", false).expect("load 01");
    let sim1 = scene1.sim.as_ref().unwrap();
    eprintln!("=== 01 shoji threshold: floor_below from y 6.0, depth 8.0, rows z 8.4..11.2 step 0.2, cols x 4.0..6.4 step 0.2 ===");
    let mut z = 8.4f32;
    while z <= 11.2 {
        let mut row = format!("z {:5.2}: ", z);
        let mut x = 4.0f32;
        while x <= 6.4 {
            match sim1.world.floor_below([x, 6.0, z], 8.0) {
                Some(fy) => row.push_str(&format!("{:+.3} ", fy)),
                None => row.push_str("  ---- "),
            }
            x += 0.2;
        }
        eprintln!("{row}");
        z += 0.2;
    }
}

#[test]
#[ignore = "dump"]
fn probe_door4_leaf_tris() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0a");
    let mut w = walker::Walker::new(scene);
    w.wait(5); // the doors join the dynamic set at the drones' tick
    // Reach the wedge spot and cycle the door to the pos01-0 OPEN pose (the walk's state).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-17.87, 1.12, 4.87];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
    }
    for _ in 0..6 { w.tick(nf_game::player::Input::default(), true); }  // the E toggle
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    // The dynamic (door) tris near the wedge spot.
    let tris = sim.world.dynamic_tris_near([-21.0, -1.0, 3.0], [-16.5, 3.0, 6.5]);
    eprintln!("=== door #4's dynamic tris near the doorway: {} ===", tris.len());
    let mut xs = [(f32::MAX, f32::MIN); 3];
    for t in &tris {
        for v in t { for k in 0..3 { xs[k].0 = xs[k].0.min(v[k]); xs[k].1 = xs[k].1.max(v[k]); } }
    }
    eprintln!("leaf bbox x {:.2}..{:.2} y {:.2}..{:.2} z {:.2}..{:.2}", xs[0].0, xs[0].1, xs[1].0, xs[1].1, xs[2].0, xs[2].1);
    for t in tris.iter().take(24) {
        eprintln!("tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
    let d = &sim.drones.doors.list[4];
    eprintln!("door4 state {:#x} pos01 {:.2} sph {:?}", d.state, d.pos01, d.sph);
}

#[test]
#[ignore = "dump"]
fn probe_all_movers_near() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0a");
    let mut w = walker::Walker::new(scene);
    // Walk Bond to the wedge spot by real input (the reach teleport, then settle).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-17.87, 1.12, 4.87];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
    }
    w.wait(10);
    // Face west and walk into the doorway for 120 ticks (toward the leaf).
    for _ in 0..120 { w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    eprintln!("bond at ({:.4},{:.4},{:.4}) g={}", p[0], p[1], p[2], sim.bond.grounded);
    // Every dynamic object's tris within 3 units.
    let tris = sim.world.dynamic_tris_near([p[0] - 3.0, p[1] - 3.0, p[2] - 3.0], [p[0] + 3.0, p[1] + 3.0, p[2] + 3.0]);
    eprintln!("dynamic tris within 3: {}", tris.len());
    let mut boxes: Vec<([f32; 3], [f32; 3])> = Vec::new();
    for t in &tris {
        for v in t {
            let mut placed = false;
            for b in boxes.iter_mut() {
                // Grow a box if the vertex is within 0.05 of it (the same mover).
                if (0..3).all(|k| v[k] >= b.0[k] - 0.05 && v[k] <= b.1[k] + 0.05) {
                    for k in 0..3 { b.0[k] = b.0[k].min(v[k]); b.1[k] = b.1[k].max(v[k]); }
                    placed = true; break;
                }
            }
            if !placed { boxes.push(([v[0], v[1], v[2]], [v[0], v[1], v[2]])); }
        }
    }
    for b in &boxes {
        eprintln!("mover box x {:+.2}..{:+.2} y {:+.2}..{:+.2} z {:+.2}..{:+.2}", b.0[0], b.1[0], b.0[1], b.1[1], b.0[2], b.1[2]);
    }
    // The static tris that overlap Bond's capsule band (y 0.6..1.5) within 1 unit.
    let stat = sim.world.tris_near([p[0] - 1.2, 0.5, p[2] - 1.2], [p[0] + 1.2, 1.6, p[2] + 1.2]);
    eprintln!("static tris in the capsule band within 1.2: {}", stat.len());
    for t in stat.iter().take(20) {
        eprintln!("stri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
}

#[test]
#[ignore = "dump"]
fn probe_lip_map() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0a");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== the lip: floor y ×1000 at z 4.8..5.5 step 0.1, x −19.6..−17.8 step 0.1 ===");
    let mut z = 4.8f32;
    while z <= 5.5 {
        let mut row = format!("z {z:+.1}: ");
        let mut x = -19.6f32;
        while x <= -17.8 {
            match sim.world.floor_below([x, 2.0, z], 4.0) {
                Some(fy) => row.push_str(&format!("{:+.3}", fy * 1000.0).replace("+0.000", "  .  ").replace("+0.001", " +1  ")),
                None => row.push_str(" ----"),
            }
            row.push(' ');
            x += 0.1;
        }
        eprintln!("{row}");
        z += 0.1;
    }
}

#[test]
#[ignore = "dump"]
fn probe_ai_nodes_south() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0a");
    let sim = scene.sim.as_ref().unwrap();
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        for (ni, n) in p.nodes.iter().enumerate() {
            let q = n.pos;
            if (-14.0..=12.0).contains(&q[0]) && (-26.0..=-12.0).contains(&q[2]) {
                eprintln!("path{} node{} ({:.2},{:.2},{:.2})", pi, ni, q[0], q[1], q[2]);
            }
        }
    }
    // The locked office door's leaf box (dynamic tris) at its state.
    let tris = sim.world.dynamic_tris_near([-12.5, -0.5, -21.0], [-8.5, 2.5, -17.0]);
    eprintln!("dynamic tris near the locked door: {}", tris.len());
    if !tris.is_empty() {
        let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
        for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
        eprintln!("leaf box x {:+.2}..{:+.2} y {:+.2}..{:+.2} z {:+.2}..{:+.2}", lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]);
    }
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if (d.pos[0] + 10.6).abs() < 0.6 && (d.pos[2] + 19.3).abs() < 0.6 {
            eprintln!("door[{i}] at ({:.2},{:.2},{:.2}) kind {} state {:#x} pos01 {:.2} ch_lock {:#x} sph {:?}", d.pos[0], d.pos[1], d.pos[2], d.kind, d.state, d.pos01, d.ch_lock, d.sph);
        }
    }
}

#[test]
#[ignore = "dump"]
fn probe_ai_nodes_west() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0a");
    let sim = scene.sim.as_ref().unwrap();
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        for (ni, n) in p.nodes.iter().enumerate() {
            let q = n.pos;
            if (-24.0..=-13.0).contains(&q[0]) && (-4.0..=10.0).contains(&q[2]) {
                eprintln!("path{} node{} ({:.2},{:.2},{:.2})", pi, ni, q[0], q[1], q[2]);
            }
        }
    }
    // The tables/desks in the west block (the static tris' boxes at y 0.6..1.0).
    let tris = sim.world.tris_near([-22.0, 0.6, -2.0], [-13.0, 1.1, 8.0]);
    eprintln!("tris y 0.6..1.1 in the west block: {}", tris.len());
    let mut boxes: Vec<([f32; 3], [f32; 3])> = Vec::new();
    for t in &tris {
        for v in t {
            let mut placed = false;
            for b in boxes.iter_mut() {
                if (0..3).all(|k| v[k] >= b.0[k] - 0.15 && v[k] <= b.1[k] + 0.15) {
                    for k in 0..3 { b.0[k] = b.0[k].min(v[k]); b.1[k] = b.1[k].max(v[k]); }
                    placed = true; break;
                }
            }
            if !placed { boxes.push(([v[0], v[1], v[2]], [v[0], v[1], v[2]])); }
        }
    }
    for b in &boxes {
        if b.1[0] - b.0[0] > 0.3 && b.1[2] - b.0[2] > 0.3 {
            eprintln!("furn box x {:+.2}..{:+.2} y {:+.2}..{:+.2} z {:+.2}..{:+.2}", b.0[0], b.1[0], b.0[1], b.1[1], b.0[2], b.1[2]);
        }
    }
}

#[test]
#[ignore = "dump"]
fn probe_mouth_tris() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0a");
    let sim = scene.sim.as_ref().unwrap();
    let tris = sim.world.tris_near([-12.3, -0.5, -20.0], [-10.0, 2.5, -17.8]);
    eprintln!("static tris at the mouth: {}", tris.len());
    for t in &tris {
        eprintln!("tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
    // The floor profile along the lane z −18.6.
    for x in [-11.5f32, -11.2, -11.0, -10.6, -10.2, -9.9] {
        let f = sim.world.floor_below([x, 2.0, -18.6], 4.0);
        eprintln!("floor ({:+.2}, −18.6) = {:?}", x, f);
    }
}

#[test]
#[ignore = "dump"]
fn probe_disc_mouth_floor() {
    use nf_formats::iso9660::IsoImage;
    let img = IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000a.bin")).expect("0700000a.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    // All disc collision tris (any map, any placement) in the mouth hole box.
    let mut count = 0usize;
    for (mi, m) in lv.maps.iter().enumerate() {
        for p in &m.placements {
            let t = p.translation;
            if !(t[0] - 16.0..=t[0] + 16.0).contains(&-10.8) { continue; }
            if !(-36.0..=14.0).contains(&t[2]) { continue; }
            if !(-6.0..=8.0).contains(&t[1]) { continue; }
            let scale = p.scale();
            let sc = [scale[0].abs(), scale[1].abs(), scale[2].abs()];
            let yaw = p.rotation[1];
            let (s, c) = (yaw.sin(), yaw.cos());
            let Some(col) = m.objects.get(p.index as usize).and_then(|o| o.collision.as_ref()) else { continue };
            for tri in col.triangles() {
                let w = tri.map(|v| [
                    t[0] + v[0] * sc[0] * c + v[2] * sc[2] * s,
                    t[1] + v[1] * sc[1],
                    t[2] - v[0] * sc[0] * s + v[2] * sc[2] * c,
                ]);
                if w.iter().all(|v| (-11.6..=-10.3).contains(&v[0]) && (-0.6..=0.6).contains(&v[1]) && (-19.6..=-17.9).contains(&v[2])) {
                    count += 1;
                    eprintln!("map{mi} plc{} tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
                        p.index, w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2]);
                }
            }
        }
    }
    eprintln!("total disc tris in the mouth hole box: {count}");
}

#[test]
#[ignore = "dump"]
fn probe_nonmain_cels() {
    use nf_formats::iso9660::IsoImage;
    let img = IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000a.bin")).expect("0a");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let main = lv.main_map();
    for (mi, m) in lv.maps.iter().enumerate() {
        if Some(mi) == main { continue; }
        let cels: Vec<_> = m.placements.iter().filter(|p| p.creates_cel()).collect();
        if cels.is_empty() { continue; }
        eprintln!("map{}: {} placements, {} create cels", mi, m.placements.len(), cels.len());
        for p in cels.iter().take(6) {
            let t = p.translation;
            eprintln!("  plc{} flags {:#x} at ({:.2},{:.2},{:.2}) byhash {}", p.index, p.flags, t[0], t[1], t[2], p.by_hash());
        }
    }
}
