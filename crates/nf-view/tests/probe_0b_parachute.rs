//! Probe (scratch, level-0b-parachute-step-up): the disc-side data around the parachute pickup
//! #280 (2.53, 58.45, 42.34 [D]) on 0700000b — the hovering copter's parked pose and its live
//! collision triangles, the static deck triangles, and the pickup's touch bounds. No pins here:
//! evidence gathering for the walked take. Run with --nocapture.

use nf_game::world::V3;
use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump_tris(label: &str, tris: &[[[f32; 3]; 3]], lo: V3, hi: V3) {
    eprintln!("[probe] {label}: {} tris in box", tris.len());
    let mut ys = Vec::new();
    for t in tris {
        for v in t { ys.push(v[1]); }
    }
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if !ys.is_empty() {
        eprintln!("[probe] {label}: y range {:.2}..{:.2}", ys[0], ys[ys.len() - 1]);
    }
    for (i, t) in tris.iter().enumerate() {
        eprintln!("[probe] {label}[{i}] ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
    let _ = (lo, hi);
}

#[test]
fn probe_0b_parachute_data() {
    let mut scene = nf_view::load_with_carry(&iso(), "0700000b.bin", false, None).expect("load");
    // The helicopter's gate: 060004FA stream 14 writes ch 0x2A at 0x990 [D] — set it directly
    // (the probe opens the gate without the cut-scene) and let the copter fly its route.
    scene.sim.as_mut().unwrap().drones.set_channel(0x2a);
    let w = &mut scene;
    for i in 0..3600u32 {
        w.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = w.sim.as_ref().unwrap();
        if i % 300 == 0 {
            let c = sim.drones.copters.iter().find(|c| c.out == 0x66);
            match c {
                Some(c) => eprintln!("[probe] t{i}: copter state {} t {:.3} path {:?} m ({:.2},{:.2},{:.2}) landed {}",
                    c.state, c.t, c.path, c.matrix[3][0], c.matrix[3][1], c.matrix[3][2], c.landed),
                None => eprintln!("[probe] t{i}: no copter with out 0x66 (n={})", sim.drones.copters.len()),
            }
        }
    }
    let sim = w.sim.as_ref().unwrap();
    // The copter's final pose.
    if let Some(c) = sim.drones.copters.iter().find(|c| c.out == 0x66) {
        eprintln!("[probe] FINAL copter state {} t {:.3} matrix rows:", c.state, c.t);
        for r in 0..4 { eprintln!("[probe]   [{:.3} {:.3} {:.3} {:.3}]", c.matrix[r][0], c.matrix[r][1], c.matrix[r][2], c.matrix[r][3]); }
        eprintln!("[probe] body_obj {:?}", c.body_obj);
    }
    // The script entities near the pickup (the copter's body is one).
    for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
        if let Some(o) = o {
            if let nf_game::nis::Obj::Entity { hash, kind, matrix, .. } = o {
                let dx = matrix[3][0] - 2.53; let dy = matrix[3][1] - 58.45; let dz = matrix[3][2] - 42.34;
                if dx * dx + dy * dy + dz * dz < 400.0 || hash & 0x0f00_0000 == 0x0200_0000 {
                    eprintln!("[probe] script-obj{i}: hash {hash:#x} kind {kind} at ({:.2},{:.2},{:.2})",
                        matrix[3][0], matrix[3][1], matrix[3][2]);
                }
            }
        }
    }
    // The pickup's bounds.
    for p in &sim.drones.pickups {
        if p.switch_channel == 0x3f {
            eprintln!("[probe] parachute pickup: m ({:.2},{:.2},{:.2}) bounds centre ({:.2},{:.2},{:.2}) radius {:.2} kind {} weapon {:#x} state {} gone {}",
                p.m[3][0], p.m[3][1], p.m[3][2], p.bounds.centre[0], p.bounds.centre[1], p.bounds.centre[2], p.bounds.radius,
                p.kind, p.weapon, p.state, p.gone);
            eprintln!("[probe]   touch radius (radius+1.1) = {:.2}", p.bounds.radius + 1.1);
        }
    }
    // The static deck + the copter's live collision around the pickup.
    let lo: V3 = [-3.0, 54.0, 36.0];
    let hi: V3 = [8.0, 62.0, 49.0];
    // Focused: the floor profile under and around the pickup (ray down from y 61).
    for z in [38.0f32, 40.0, 41.0, 42.0, 42.34, 43.0, 44.0, 45.0, 46.0] {
        for x in [-2.0f32, 0.0, 1.0, 2.53, 4.0, 6.0] {
            let hit = sim.world.segment([x, 61.0, z], [x, 50.0, z]);
            if let Some(t) = hit {
                let y = 61.0 + t * (50.0 - 61.0);
                eprintln!("[probe] floor@({x:.2},{z:.2}): y {y:.3}");
            } else {
                eprintln!("[probe] floor@({x:.2},{z:.2}): MISS");
            }
        }
    }
    // Horizontal rays at walk heights from the deck toward the pickup target.
    for (y, label) in [(58.07f32, "root"), (57.63, "capsB+0.3"), (57.33, "capsB"), (57.0, "deck")] {
        let hit = sim.world.segment([2.53, y, 36.0], [2.53, y, 45.0]);
        match hit {
            Some(t) => eprintln!("[probe] ray {label} y{y}: hit z {:.3}", 36.0 + t * 9.0),
            None => eprintln!("[probe] ray {label} y{y}: clear"),
        }
    }
    // The tight triangle dump around the pickup's cavity — filtered to the pocket between the
    // stuck pose and the pickup centre.
    let tlo: V3 = [1.0, 56.8, 40.5];
    let thi: V3 = [5.5, 59.0, 45.5];
    let tris = sim.world.tris_near(tlo, thi);
    eprintln!("[probe] tight box: {} tris", tris.len());
    let in_pocket = |t: &[[f32; 3]; 3]| {
        t.iter().all(|v| v[0] >= 1.8 && v[0] <= 4.6 && v[1] >= 56.9 && v[1] <= 59.2 && v[2] >= 41.0 && v[2] <= 43.2)
    };
    for (i, t) in tris.iter().enumerate() {
        if in_pocket(t) {
            eprintln!("[probe] pocket tri{i} ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
                t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
        }
    }
    // The take-ray from the walkthrough's stuck pose to the pickup centre.
    let stuck: V3 = [3.61, 58.07, 41.34];
    let c = [2.53f32, 58.45, 42.34];
    match sim.world.segment_take_scoped(None, stuck, c) {
        Some(t) => eprintln!("[probe] take-ray from stuck pose: BLOCKED t {:.4} at ({:.2},{:.2},{:.2})", t,
            stuck[0] + t * (c[0] - stuck[0]), stuck[1] + t * (c[1] - stuck[1]), stuck[2] + t * (c[2] - stuck[2])),
        None => eprintln!("[probe] take-ray from stuck pose: CLEAR"),
    }
    // The take-region map: for each xz on the deck (root = floor + 1.089) and on the box
    // (root = floor + 2.0), does the distance gate AND the take-ray pass?
    let c = [2.53f32, 58.45, 42.34];
    eprintln!("[probe] take map (X = take, . = no), rows z 45.0->40.5 step 0.5, cols x 1.0->6.0 step 0.5");
    for zi in 0..10 {
        let z = 45.0 - 0.5 * zi as f32;
        let mut row = String::new();
        for xi in 0..11 {
            let x = 1.0 + 0.5 * xi as f32;
            let floor = sim.world.segment([x, 62.0, z], [x, 50.0, z]).map(|t| 62.0 + t * -12.0);
            let root = match floor { Some(f) => f + 1.089, None => { row.push(' '); continue; } };
            let p = [x, root, z];
            let d = ((c[0]-p[0]).powi(2)+(c[1]-p[1]).powi(2)+(c[2]-p[2]).powi(2)).sqrt();
            let los = sim.world.segment_take_scoped(None, p, c).map_or(true, |t| t >= 1.0 - 1e-3);
            row.push(if d <= 1.79 && los { 'X' } else { '.' });
        }
        eprintln!("[probe] z{z:5.1} {row}");
    }
}
