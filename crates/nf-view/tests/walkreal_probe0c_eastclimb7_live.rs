//! walkreal probe (level-0c-east-climb): the WALKABILITY FLOOD-FILL over the sim's own collision.
//! Grid 0.25 over x 36..100, z −242..−203; every column's surfaces from y 0..17; edges between
//! adjacent cells when a step |Δy| ≤ 0.55 [S unverified step law] or a same-level/drop move has no
//! wall at knee height; flood from the walk's real seeds; report which component reaches the gate
//! zone (x 95..97, z −210.5..−208.5, deck ≥ 10.2). Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use std::collections::{BTreeMap, BTreeSet};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const X0: f32 = 36.0; const X1: f32 = 100.0; const Z0: f32 = -242.0; const Z1: f32 = -203.0;
const STEP: f32 = 0.25;
const NX: usize = ((X1 - X0) / STEP) as usize + 1;
const NZ: usize = ((Z1 - Z0) / STEP) as usize + 1;
const MAX_STEP_UP: f32 = 0.80; // [S] unverified step law — the walk-ride bound (r 0.55 [M 0x194df0])

fn idx(x: usize, z: usize, s: usize) -> usize { (z * NX + x) * 8 + s }

fn main_() {}

#[test]
fn probe_east_climb_flood() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // 1. Sample every column's surfaces (multi-hit down-rays, y 17 -> 0).
    let mut surf: Vec<Option<[f32; 8]>> = vec![None; NX * NZ]; // up to 8 surfaces per column
    let sim = w.scene.sim.as_ref().unwrap();
    for zi in 0..NZ {
        let z = Z0 + STEP * zi as f32;
        for xi in 0..NX {
            let x = X0 + STEP * xi as f32;
            let mut top = 17.0f32;
            let mut n = 0usize;
            let mut arr = [f32::NAN; 8];
            while n < 8 {
                match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
                    Some((t, _)) => {
                        let y = top - top * t;
                        arr[n] = y; n += 1;
                        top = y - 0.05;
                        if top <= 0.05 { break; }
                    }
                    None => break,
                }
            }
            if n > 0 { surf[zi * NX + xi] = Some(arr); }
        }
    }
    let ncols = surf.iter().filter(|s| s.is_some()).count();
    eprintln!("columns sampled: {}/{}", ncols, NX * NZ);

    // 2. Build edges. Same cell: connect consecutive surfaces if the gap is climbable-down (any)
    //    or the upper is within MAX_STEP_UP of the lower (walk-up through the floor boundary is
    //    NOT allowed — surfaces in one column stack vertically; a column's surfaces are separate
    //    levels reachable from neighbours).
    let mut adj: BTreeSet<(usize, usize)> = BTreeSet::new(); // DIRECTED (from, to)
    let mut link = |a: usize, b: usize, adj: &mut BTreeSet<(usize, usize)>| {
        adj.insert((a, b));
    };
    for zi in 0..NZ {
        for xi in 0..NX {
            let Some(sa) = surf[zi * NX + xi] else { continue };
            for (si, &ya) in sa.iter().enumerate() {
                if ya.is_nan() { break; }
                // neighbour east
                if xi + 1 < NX {
                    if let Some(sb) = surf[zi * NX + xi + 1] {
                        for (sj, &yb) in sb.iter().enumerate() {
                            if yb.is_nan() { break; }
                            let d = yb - ya;
                            let zc = Z0 + STEP * zi as f32;
                            let a = idx(xi, zi, si); let b = idx(xi + 1, zi, sj);
                            if d.abs() <= 0.02 {
                                link(a, b, &mut adj); link(b, a, &mut adj);
                            } else if d > 0.0 && d <= MAX_STEP_UP {
                                // walk up: needs headroom over the lower cell — the upper surface
                                // must be the LOWEST surface in that column above ya (true by
                                // construction if yb is the first surface > ya) and no wall: the
                                // multi-hit already exposes the step face as a surface; allow.
                                link(a, b, &mut adj);
                            } else if d < 0.0 && d > -3.5 {
                                // drop: no wall check at knee height of the upper surface
                                let knee = ya + 0.9;
                                let blocked = sim.world.segment_hit([X0 + STEP * xi as f32, knee, zc],
                                    [X0 + STEP * (xi + 1) as f32, knee, zc]).is_some();
                                if !blocked { link(a, b, &mut adj); }
                            }
                        }
                    }
                }
                // neighbour north (z+1)
                if zi + 1 < NZ {
                    if let Some(sb) = surf[(zi + 1) * NX + xi] {
                        for (sj, &yb) in sb.iter().enumerate() {
                            if yb.is_nan() { break; }
                            let d = yb - ya;
                            let z2 = Z0 + STEP * (zi + 1) as f32;
                            let xc = X0 + STEP * xi as f32;
                            let a = idx(xi, zi, si); let b = idx(xi, zi + 1, sj);
                            if d.abs() <= 0.02 {
                                link(a, b, &mut adj); link(b, a, &mut adj);
                            } else if d > 0.0 && d <= MAX_STEP_UP {
                                link(a, b, &mut adj);
                            } else if d < 0.0 && d > -3.5 {
                                let knee = ya + 0.9;
                                let blocked = sim.world.segment_hit([xc, knee, Z0 + STEP * zi as f32],
                                    [xc, knee, z2]).is_some();
                                if !blocked { link(a, b, &mut adj); }
                            }
                        }
                    }
                }
            }
        }
    }
    let nedges = adj.len();
    eprintln!("edges: {nedges}");

    // 3. Flood from the seeds (find the nearest sampled surface to each seed point).
    let seeds: Vec<(&str, f32, f32)> = vec![
        ("west band 10.34", 52.0, -207.6),
        ("corner/corridor 11.09", 39.5, -209.0),
        ("east ground 6.25", 82.0, -210.0),
        ("deep gallery 6.25", 43.0, -237.8),
        ("yard 0.0", 70.0, -220.0),
        ("gate deck (stand-in spot)", 95.0, -209.5),
        ("staircase01 deck", 86.0, -210.0),
        ("south mezzanine", 45.0, -237.5),
        ("room05 east strip", 96.0, -209.7),
    ];
    let mut comp: BTreeMap<usize, usize> = BTreeMap::new();
    let mut parent: BTreeMap<usize, usize> = BTreeMap::new();
    let mut cid = 0usize;
    let mut seed_comps: Vec<(&str, Option<usize>, f32)> = Vec::new();
    for (name, sx, sz) in &seeds {
        let xi = ((sx - X0) / STEP).round() as usize;
        let zi = ((sz - Z0) / STEP).round() as usize;
        let mut best: Option<(usize, f32)> = None;
        if let Some(sa) = surf[zi * NX + xi] {
            for (si, &y) in sa.iter().enumerate() {
                if y.is_nan() { break; }
                // the surface closest below y 13 that is a floor
                best = match best { None => Some((si, y)), Some((_, by)) if y > by && y < 14.0 => Some((si, y)), b => b };
            }
        }
        let Some((si, y)) = best else { seed_comps.push((name, None, f32::NAN)); continue; };
        let start = idx(xi, zi, si);
        if comp.contains_key(&start) {
            seed_comps.push((name, comp.get(&start).copied(), y));
            continue;
        }
        // BFS
        let mut stack = vec![start];
        comp.insert(start, cid);
        while let Some(node) = stack.pop() {
            let _ = &parent;
            let x = (node / 8 % NX) as isize; let z = (node / 8 / NX) as isize;
            for (dx, dz) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if nx < 0 || nz < 0 || nx >= NX as isize || nz >= NZ as isize { continue; }
                for sj in 0..8 {
                    let nb = idx(nx as usize, nz as usize, sj);
                    if adj.contains(&(node, nb)) && !comp.contains_key(&nb) {
                        comp.insert(nb, cid);
                        parent.insert(nb, node);
                        stack.push(nb);
                    }
                }
            }
        }
        seed_comps.push((name, Some(cid), y));
        cid += 1;
    }
    eprintln!("=== seed components ===");
    for (name, c, y) in &seed_comps {
        eprintln!("{name}: y {y:.2} -> component {c:?}");
    }
    let counts = seed_comps.iter().filter_map(|(_, c, _)| *c).collect::<BTreeSet<_>>();
    eprintln!("distinct components among seeds: {:?}", counts);
    // 4. Does any seed component contain the gate deck?
    let gate_xi = ((95.0 - X0) / STEP).round() as usize;
    let gate_zi = ((-209.5 - Z0) / STEP).round() as usize;
    if let Some(sa) = surf[gate_zi * NX + gate_xi] {
        for (si, &y) in sa.iter().enumerate() {
            if y.is_nan() { break; }
            let node = idx(gate_xi, gate_zi, si);
            eprintln!("gate column surface y {y:.2}: component {:?}", comp.get(&node));
        }
    }
    // 5. Trace the path from the east-ground seed to the gate deck + the staircase01 deck.
    let trace = |target: usize, name: &str, parent: &BTreeMap<usize, usize>| {
        let mut path = Vec::new();
        let mut cur = Some(target);
        while let Some(n) = cur {
            path.push(n);
            cur = parent.get(&n).copied();
        }
        path.reverse();
        eprintln!("=== path to {name} ({} nodes) ===", path.len());
        for (i, &n) in path.iter().enumerate() {
            if i > 0 && i < path.len() - 1 { continue; } // ends only... no: print all if short
        }
        for (i, &n) in path.iter().enumerate() {
            let x = X0 + STEP * (n / 8 % NX) as f32;
            let z = Z0 + STEP * (n / 8 / NX) as f32;
            let s = n % 8;
            let y = surf[(n / 8 / NX) * NX + n / 8 % NX].map(|a| a[s]).unwrap_or(f32::NAN);
            if path.len() <= 60 || i % ((path.len() / 40).max(1)) == 0 || i == path.len() - 1 {
                eprintln!("  {i:3}: ({x:.2}, {z:.2}) y {y:.2}");
            }
        }
    };
    // find nodes: east ground seed node and the gate deck node / staircase01 deck node
    let find_node = |sx: f32, sz: f32, ylo: f32, yhi: f32| -> Option<usize> {
        let xi = ((sx - X0) / STEP).round() as usize;
        let zi = ((sz - Z0) / STEP).round() as usize;
        let sa = surf[zi * NX + xi]?;
        for (si, &y) in sa.iter().enumerate() {
            if y.is_nan() { break; }
            if (ylo..=yhi).contains(&y) { return Some(idx(xi, zi, si)); }
        }
        None
    };
    if let Some(t) = find_node(95.0, -209.5, 10.2, 10.5) {
        let mut path = Vec::new();
        let mut cur = Some(t);
        while let Some(n) = cur { path.push(n); cur = parent.get(&n).copied(); }
        path.reverse();
        eprintln!("=== path to the gate deck ({} nodes) ===", path.len());
        let stride = (path.len() / 45).max(1);
        for (i, &n) in path.iter().enumerate() {
            if i % stride != 0 && i != path.len() - 1 { continue; }
            let x = X0 + STEP * (n / 8 % NX) as f32;
            let z = Z0 + STEP * (n / 8 / NX) as f32;
            let s = n % 8;
            let y = surf[(n / 8 / NX) * NX + n / 8 % NX].map(|a| a[s]).unwrap_or(f32::NAN);
            eprintln!("  {i:3}: ({x:.2}, {z:.2}) y {y:.2}");
        }
    }
    if let Some(t) = find_node(86.0, -210.0, 10.2, 10.5) {
        let mut path = Vec::new();
        let mut cur = Some(t);
        while let Some(n) = cur { path.push(n); cur = parent.get(&n).copied(); }
        path.reverse();
        eprintln!("=== path to the staircase01 deck ({} nodes) ===", path.len());
        let stride = 1;
        for (i, &n) in path.iter().enumerate().skip(path.len().saturating_sub(60)) {
            if i % stride != 0 && i != path.len() - 1 { continue; }
            let x = X0 + STEP * (n / 8 % NX) as f32;
            let z = Z0 + STEP * (n / 8 / NX) as f32;
            let s = n % 8;
            let y = surf[(n / 8 / NX) * NX + n / 8 % NX].map(|a| a[s]).unwrap_or(f32::NAN);
            eprintln!("  {i:3}: ({x:.2}, {z:.2}) y {y:.2}");
        }
    }
    eprintln!("ok");
}
