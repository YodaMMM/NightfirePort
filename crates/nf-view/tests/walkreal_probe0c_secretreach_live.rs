//! walkreal probe (level-0c-secret-door-reach): the WALKABILITY FLOOD-FILL over the sim's own
//! collision for the warehouse/corridor zone. Grid 0.25 over x 8..62, z −192..−148; every
//! column's surfaces from y 18..0; edges between adjacent cells when a step |Δy| ≤ 0.55
//! [S unverified step law] or a same-level/drop move has no wall at knee height; flood from the
//! spots the walk actually stands on (the display ground 6.27, the 11.27 hall, plus the target
//! spots for cross-checking: the corridor 0.90, the mezzanine band by the vent, the warehouse
//! upper floor, the yard). Band maps (ground 0..2, mezzanine 4.5..7, upper 8..11.5) show which
//! component owns which floor. Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use std::collections::{BTreeMap, BTreeSet};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const X0: f32 = 8.0; const X1: f32 = 62.0; const Z0: f32 = -192.0; const Z1: f32 = -115.0;
const STEP: f32 = 0.25;
const NX: usize = ((X1 - X0) / STEP) as usize + 1;
const NZ: usize = ((Z1 - Z0) / STEP) as usize + 1;
const MAX_STEP_UP: f32 = 0.55; // [S] unverified step law — the walk-ride bound

fn idx(x: usize, z: usize, s: usize) -> usize { (z * NX + x) * 8 + s }

#[test]
fn probe_secret_reach_flood() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // 1. Sample every column's surfaces (multi-hit down-rays, y 18 -> 0).
    let mut surf: Vec<Option<[f32; 8]>> = vec![None; NX * NZ];
    let sim = w.scene.sim.as_ref().unwrap();
    for zi in 0..NZ {
        let z = Z0 + STEP * zi as f32;
        for xi in 0..NX {
            let x = X0 + STEP * xi as f32;
            let mut top = 18.0f32;
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

    // 2. Edges: same-level both ways; step-up <= MAX_STEP_UP one way; drops one-way with the
    //    knee-height wall check (the eastclimb7 law).
    let mut adj: BTreeSet<(usize, usize)> = BTreeSet::new();
    for zi in 0..NZ {
        for xi in 0..NX {
            let Some(sa) = surf[zi * NX + xi] else { continue };
            for (si, &ya) in sa.iter().enumerate() {
                if ya.is_nan() { break; }
                for (dx, dz) in [(1usize, 0usize), (0usize, 1usize)] {
                    let nxi = xi + dx; let nzi = zi + dz;
                    if nxi >= NX || nzi >= NZ { continue; }
                    if let Some(sb) = surf[nzi * NX + nxi] {
                        for (sj, &yb) in sb.iter().enumerate() {
                            if yb.is_nan() { break; }
                            let d = yb - ya;
                            let a = idx(xi, zi, si); let b = idx(nxi, nzi, sj);
                            if d.abs() <= 0.02 {
                                adj.insert((a, b)); adj.insert((b, a));
                            } else if d > 0.0 && d <= MAX_STEP_UP {
                                adj.insert((a, b));
                            } else if d < 0.0 && d > -3.5 {
                                let knee = ya + 0.9;
                                let blocked = sim.world.segment_hit(
                                    [X0 + STEP * xi as f32, knee, Z0 + STEP * zi as f32],
                                    [X0 + STEP * nxi as f32, knee, Z0 + STEP * nzi as f32]).is_some();
                                if !blocked { adj.insert((a, b)); }
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("edges: {}", adj.len());

    // 3. Flood per seed (each seed pins an explicit y band).
    let seeds: Vec<(&str, f32, f32, f32, f32)> = vec![
        ("display ground 6.27", 13.2, -120.0, 6.0, 6.6),
        ("hall 11.27", 13.0, -128.2, 11.0, 11.6),
        ("corridor 0.90", 53.0, -173.0, 0.6, 1.2),
        ("mezzanine by the vent", 53.8, -165.0, 5.2, 6.2),
        ("warehouse upper", 50.0, -170.0, 8.5, 10.0),
        ("yard 0.69", 48.0, -180.0, 0.4, 1.0),
    ];
    let mut comp: BTreeMap<usize, usize> = BTreeMap::new();
    let mut seed_comps: Vec<(&str, Option<usize>, f32)> = Vec::new();
    let mut cid = 0usize;
    for (name, sx, sz, ylo, yhi) in &seeds {
        let xi = ((sx - X0) / STEP).round() as usize;
        let zi = ((sz - Z0) / STEP).round() as usize;
        let mut start = None;
        if let Some(sa) = surf[zi * NX + xi] {
            for (si, &y) in sa.iter().enumerate() {
                if y.is_nan() { break; }
                if (ylo..=yhi).contains(&&y) { start = Some(idx(xi, zi, si)); break; }
            }
        }
        let Some(start) = start else { seed_comps.push((name, None, f32::NAN)); continue; };
        if comp.contains_key(&start) {
            let y = surf[(start / 8 / NX) * NX + start / 8 % NX].map(|a| a[start % 8]).unwrap_or(f32::NAN);
            seed_comps.push((name, comp.get(&start).copied(), y));
            continue;
        }
        let mut stack = vec![start];
        comp.insert(start, cid);
        while let Some(node) = stack.pop() {
            let x = (node / 8 % NX) as isize; let z = (node / 8 / NX) as isize;
            for (dx, dz) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if nx < 0 || nz < 0 || nx >= NX as isize || nz >= NZ as isize { continue; }
                for sj in 0..8 {
                    let nb = idx(nx as usize, nz as usize, sj);
                    if adj.contains(&(node, nb)) && !comp.contains_key(&nb) {
                        comp.insert(nb, cid);
                        stack.push(nb);
                    }
                }
            }
        }
        let y = surf[(start / 8 / NX) * NX + start / 8 % NX].map(|a| a[start % 8]).unwrap_or(f32::NAN);
        seed_comps.push((name, Some(cid), y));
        cid += 1;
    }
    eprintln!("=== seed components ===");
    for (name, c, y) in &seed_comps {
        eprintln!("{name}: y {y:.2} -> component {c:?}");
    }

    // 4. Per-component stats: cell count, bbox, y range.
    let mut stats: BTreeMap<usize, (usize, f32, f32, f32, f32, f32, f32)> = BTreeMap::new();
    for (&node, &c) in &comp {
        let x = X0 + STEP * (node / 8 % NX) as f32;
        let z = Z0 + STEP * (node / 8 / NX) as f32;
        let y = surf[(node / 8 / NX) * NX + node / 8 % NX].map(|a| a[node % 8]).unwrap_or(f32::NAN);
        let e = stats.entry(c).or_insert((0, x, x, z, z, y, y));
        e.0 += 1;
        e.1 = e.1.min(x); e.2 = e.2.max(x);
        e.3 = e.3.min(z); e.4 = e.4.max(z);
        e.5 = e.5.min(y); e.6 = e.6.max(y);
    }
    eprintln!("=== component stats (count, x0..x1, z0..z1, y0..y1) ===");
    for (c, s) in &stats {
        eprintln!("comp {c}: {} cells, x {:.1}..{:.1}, z {:.1}..{:.1}, y {:.2}..{:.2}", s.0, s.1, s.2, s.3, s.4, s.5, s.6);
    }

    // 5. Band maps: for each 1.0 m cell, mark the component of a surface in the band.
    //    legend: digits/letters = component id; '.' = surface in band but unflooded; ' ' = none.
    let band_map = |name: &str, ylo: f32, yhi: f32| {
        eprintln!("=== band map: {name} (y {ylo}..{yhi}) — cols x {X0}..{X1} step 1, rows z {Z0}..{Z1} step 1 ===");
        let mut header = String::from("z\\x  ");
        for xi in 0..NX {
            let x = X0 + STEP * xi as f32;
            if (x * 2.0).round() as i32 % 2 == 0 && (x - X0) % 1.0 < STEP {
                header.push_str(&format!("{:.0}", x % 10.0));
            }
        }
        let _ = header;
        for zi in 0..NZ {
            let z = Z0 + STEP * zi as f32;
            if (z - Z0) % 1.0 >= STEP && (z - Z0) % 1.0 < 1.0 - STEP && zi != NZ - 1 { continue; }
            let mut row = format!("{:6.1} ", z);
            for xi in 0..NX {
                let x = X0 + STEP * xi as f32;
                if (x - X0) % 1.0 >= STEP && (x - X0) % 1.0 < 1.0 - STEP && xi != NX - 1 { continue; }
                let ch = match surf[zi * NX + xi] {
                    Some(sa) => {
                        let mut hit = None;
                        for &y in sa.iter() {
                            if y.is_nan() { break; }
                            if (ylo..=yhi).contains(&y) { hit = Some(y); break; }
                        }
                        match hit {
                            None => ' ',
                            Some(_) => {
                                // any surface in the band with a component
                                let mut c = None;
                                for (si, &y) in sa.iter().enumerate() {
                                    if y.is_nan() { break; }
                                    if (ylo..=yhi).contains(&y) {
                                        if let Some(cc) = comp.get(&idx(xi, zi, si)) { c = Some(*cc); }
                                        break;
                                    }
                                }
                                match c {
                                    Some(cc) => std::char::from_digit(cc as u32, 36).unwrap_or('?'),
                                    None => '.',
                                }
                            }
                        }
                    }
                    None => ' ',
                };
                row.push(ch);
            }
            eprintln!("{row}");
        }
    };
    band_map("ground", 0.0, 2.0);
    band_map("mezzanine band", 4.5, 7.0);
    band_map("upper band", 8.0, 11.5);

    // 6. Scan lines: the vent column x 53.8 (z −170..−158) and the row z −165 (x 44..62),
    //    printing every surface, so the vent hole and the mezzanine edges are explicit.
    eprintln!("=== scan: column x 53.8, z −170..−158 (step 0.5) ===");
    let col_scan = |x: f32, z: f32| -> Vec<f32> {
        let xi = ((x - X0) / STEP).round() as usize;
        let zi = ((z - Z0) / STEP).round() as usize;
        match surf[zi * NX + xi] { Some(a) => a.iter().copied().filter(|y| !y.is_nan()).collect(), None => vec![] }
    };
    let mut z = -170.0f32;
    while z <= -157.5 {
        let ys = col_scan(53.8, z);
        let ys: Vec<String> = ys.iter().map(|y| format!("{y:.2}")).collect();
        eprintln!("  x 53.8 z {z:.1}: [{}]", ys.join(", "));
        z += 0.5;
    }
    eprintln!("=== scan: row z −165.0, x 44..62 (step 0.5) ===");
    let mut x = 44.0f32;
    while x <= 61.5 {
        let ys = col_scan(x, -165.0);
        let ys: Vec<String> = ys.iter().map(|y| format!("{y:.2}")).collect();
        eprintln!("  z −165 x {x:.1}: [{}]", ys.join(", "));
        x += 0.5;
    }
    eprintln!("ok");
}
