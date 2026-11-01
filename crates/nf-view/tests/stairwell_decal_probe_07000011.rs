//! Scratch probe: fire from the 07000011 stairwell poses and attribute every new decal to the
//! collision instance it sits on. Assertion-free.
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn tf(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    [
        p[0] * m[0][0] + p[1] * m[1][0] + p[2] * m[2][0] + m[3][0],
        p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1] + m[3][1],
        p[0] * m[0][2] + p[1] * m[1][2] + p[2] * m[2][2] + m[3][2],
    ]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

fn plane_dist(p: [f32; 3], t: &[[f32; 3]; 3]) -> Option<f32> {
    // distance to the triangle if p projects inside it, else None
    let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
    let l = dot(n, n).sqrt();
    if l < 1e-9 { return None; }
    let n = [n[0] / l, n[1] / l, n[2] / l];
    let d = dot(n, sub(p, t[0]));
    let q = [p[0] - n[0] * d, p[1] - n[1] * d, p[2] - n[2] * d];
    for i in 0..3 {
        let a = t[i];
        let b = t[(i + 1) % 3];
        if dot(cross(sub(b, a), sub(q, a)), n) < -1e-4 { return None; }
    }
    Some(d.abs())
}

pub struct Tri { pub w: [[f32; 3]; 3], pub mat: u8, pub inst: usize }

#[test]
#[ignore = "probe"]
fn probe_stairwell_decals() {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).unwrap();
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let (insts, _) = level.instances(true);
    let mut tris = Vec::new();
    for (k, inst) in insts.iter().enumerate() {
        let ob = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &ob.collision else { continue };
        for (t, m) in c.triangles_with_material() {
            let w = t.map(|p| tf(&inst.matrix, p));
            if w.iter().all(|p| p[1] < 10.0 || p[1] > 55.0) { continue; }
            tris.push(Tri { w, mat: m, inst: k });
        }
    }
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).unwrap();
    let poses: [(&str, [f32; 3], f32, f32); 8] = [
        ("020", [22.40, 30.09, 1.42], -1.505, -0.035),
        ("021", [19.94, 24.01, -2.44], -6.643, -0.392),
        ("022", [16.03, 32.17, -2.39], 0.340, -0.101),
        ("023", [19.75, 16.06, -1.37], -13.408, 0.536),
        ("026", [19.22, 32.26, -2.39], -0.192, -0.163),
        ("027", [13.91, 28.17, 3.46], -3.575, -0.327),
        ("028", [16.89, 24.07, -2.44], -6.262, -0.261),
        ("029", [20.50, 24.07, -2.44], -7.020, -0.225),
    ];
    let mut rt: Vec<[[f32; 3]; 3]> = Vec::new();
    for m in &scene.materials {
        if !matches!(m.layer, nf_view::scene::Layer::World) { continue; }
        let mut i = m.first_index;
        while i + 2 < m.first_index + m.index_count {
            let t = [scene.vertices[scene.indices[i as usize] as usize].pos, scene.vertices[scene.indices[i as usize + 1] as usize].pos, scene.vertices[scene.indices[i as usize + 2] as usize].pos];
            if t.iter().any(|p| p[0] > 4.0 && p[0] < 32.0 && p[1] > 8.0 && p[1] < 56.0 && p[2] > -12.0 && p[2] < 12.0) { rt.push(t); }
            i += 3;
        }
    }
    let rdist = |p: [f32; 3]| rt.iter().map(|t| { let q = closest_on_tri(p, t); dot(sub(p, q), sub(p, q)).sqrt() }).fold(f32::INFINITY, f32::min);
    for p in [[23.219f32, 28.226, 5.772]] {
        eprintln!("[pin] {p:?} render {:.3}", rdist(p));
        for t in &tris {
            let q = closest_on_tri(p, &t.w); let d = dot(sub(p, q), sub(p, q)).sqrt();
            if d < 0.6 { let ob = &level.maps[insts[t.inst].map].objects[insts[t.inst].object]; eprintln!("[pin]   coll d {d:.3} inst {} '{}' flags {:#06x} mat {:#04x} v {:?}", t.inst, ob.name.chars().take(30).collect::<String>(), ob.flags, t.mat, t.w); }
        }
        for t in &rt { let q = closest_on_tri(p, t); let d = dot(sub(p, q), sub(p, q)).sqrt(); if d < 0.6 { eprintln!("[pin]   rend d {d:.3} v {t:?}"); } }
    }
    if std::env::var_os("PIN_ONLY").is_some() { return; }
    let mut summary = std::collections::BTreeMap::<String, usize>::new();
    for (name, pos, yaw0, pitch0) in poses {
        for dy in -3i32..=3 {
            for dp in -2i32..=2 {
                let yaw = yaw0 + dy as f32 * 0.08;
                let pitch = pitch0 + dp as f32 * 0.06;
                {
                    let sim = scene.sim.as_mut().unwrap();
                    sim.bond.pos = pos;
                    sim.bond.vel = [0.0; 3];
                    sim.bond.yaw = yaw;
                    sim.bond.pitch = pitch;
                    sim.gun.reload_state = 0;
                    sim.gun.fire_state = false;
                    sim.gun.clip = 12;
                    sim.gun.pending = 0;
                    for c in sim.drones.inventory.clip.iter_mut() { *c = 12; }
                    for c in sim.drones.inventory.carried.iter_mut() { *c = 48; }
                }
                let before = scene.sim.as_ref().unwrap().drones.effects.decals.len();
                let ev0 = scene.sim.as_ref().unwrap().drones.effects.evicted;
                let head = scene.sim.as_ref().unwrap().bond.head();
                let mut input = nf_game::player::Input::default();
                input.fire = true;
                scene.sim.as_mut().unwrap().step_bond(input, true, false, false);
                for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
                let sim = scene.sim.as_ref().unwrap();
                let ev = sim.drones.effects.evicted - ev0;
                let ds = &sim.drones.effects.decals;
                let start = before.saturating_sub(ev as usize).min(ds.len());
                for dcl in &ds[start..] {
                    let p = dcl.pos;
                    // nearest collision triangle (projected inside)
                    let mut best: Option<(f32, &Tri)> = None;
                    for t in &tris {
                        if let Some(d) = plane_dist(p, &t.w) {
                            if best.is_none_or(|(bd, _)| d < bd) { best = Some((d, t)); }
                        }
                    }
                    let (d, t) = match best { Some(b) => b, None => { eprintln!("[decal] {name} no tri p {p:?}"); continue } };
                    let inst = &insts[t.inst];
                    let ob = &level.maps[inst.map].objects[inst.object];
                    let key = format!("{} flags {:#06x} mat {:#04x}", ob.name.chars().take(40).collect::<String>(), ob.flags, t.mat);
                    *summary.entry(key.clone()).or_default() += 1;
                    let rd = rdist(p);
                    if rd > 0.05 { eprintln!("[MID] {name} p ({:.3},{:.3},{:.3}) render {rd:.3} coll {d:.3} inst {}", p[0], p[1], p[2], t.inst); }
                    eprintln!("[decal] {name} yaw {yaw:+.3} pitch {pitch:+.3} head ({:.2},{:.2},{:.2}) p ({:.3},{:.3},{:.3}) n ({:.2},{:.2},{:.2}) d {d:.3} inst {} {key}",
                        head[0], head[1], head[2], p[0], p[1], p[2], dcl.normal[0], dcl.normal[1], dcl.normal[2], t.inst);
                }
            }
        }
    }
    for (k, v) in summary { eprintln!("[summary] {v:5} {k}"); }
}

/// The headless shot's own sequence: Bond placed at a pose, the M32 drawn, the trigger pulled
/// every 4 ticks; list every decal with its collision attribution.
#[test]
#[ignore = "probe"]
fn probe_pose_burst() {
    let pose: Vec<f32> = std::env::var("POSE").unwrap_or("19.94,24.01,-2.44,-6.643,-0.392".into()).split(',').map(|t| t.parse().unwrap()).collect();
    let weapon: usize = std::env::var("WEAPON").ok().map(|s| usize::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()).unwrap_or(0x12);
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::new([pose[0], pose[1], pose[2]], pose[3]);
        sim.bond.pitch = pose[4];
        let wd = sim.weapons.clone();
        let rounds = wd.clip_size(weapon).max(1);
        if sim.drones.inventory.equip_weapon(&wd, weapon, rounds, true) { sim.drones.inventory.equip_ammo(&wd, weapon, rounds * 3); }
        sim.step_bond(nf_game::player::Input { switch_to: Some(weapon as u8), ..Default::default() }, false, false, false);
    }
    let mut last = 0usize;
    for t in 0..320u32 {
        let fire = t >= 80 && t <= 300 && (t - 80) % 4 == 0;
        let input = nf_game::player::Input { fire, ..Default::default() };
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(input, fire, false, false);
        let ds = &sim.drones.effects.decals;
        if ds.len() > last {
            for dcl in &ds[last..] {
                let h = sim.bond.head();
                eprintln!("[burst] t {t} bond ({:.2},{:.2},{:.2}) yaw {:.3} pitch {:.3} head ({:.2},{:.2},{:.2}) decal ({:.3},{:.3},{:.3}) n ({:.2},{:.2},{:.2}) mesh {:#x}",
                    sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.yaw, sim.bond.pitch, h[0], h[1], h[2],
                    dcl.pos[0], dcl.pos[1], dcl.pos[2], dcl.normal[0], dcl.normal[1], dcl.normal[2], dcl.mesh);
            }
            last = ds.len();
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[burst] total decals {} evicted {}", sim.drones.effects.decals.len(), sim.drones.effects.evicted);
}

fn closest_on_tri(p: [f32; 3], t: &[[f32; 3]; 3]) -> [f32; 3] {
    let (a, b, c) = (t[0], t[1], t[2]);
    let ab = sub(b, a); let ac = sub(c, a); let ap = sub(p, a);
    let d1 = dot(ab, ap); let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 { return a; }
    let bp = sub(p, b); let d3 = dot(ab, bp); let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 { return b; }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 { let v = d1 / (d1 - d3); return [a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v]; }
    let cp = sub(p, c); let d5 = dot(ab, cp); let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 { return c; }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 { let w = d2 / (d2 - d6); return [a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w]; }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 { let w = (d4 - d3) / ((d4 - d3) + (d5 - d6)); let bc = sub(c, b); return [b[0] + bc[0] * w, b[1] + bc[1] * w, b[2] + bc[2] * w]; }
    let den = 1.0 / (va + vb + vc); let v = vb * den; let w = vc * den;
    [a[0] + ab[0] * v + ac[0] * w, a[1] + ab[1] * v + ac[1] * w, a[2] + ab[2] * v + ac[2] * w]
}

/// Every stairwell collision triangle's sample points measured against the drawn World mesh:
/// list the collision faces with no drawn surface under them (collision-only faces).
#[test]
#[ignore = "probe"]
fn probe_collision_only_faces() {
    use nf_view::scene::Layer;
    let img = nf_formats::iso9660::IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).unwrap();
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let (insts, _) = level.instances(true);
    let scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).unwrap();
    let mut rt: Vec<[[f32; 3]; 3]> = Vec::new();
    for m in &scene.materials {
        if !matches!(m.layer, Layer::World) { continue; }
        let mut i = m.first_index;
        while i + 2 < m.first_index + m.index_count {
            let t = [scene.vertices[scene.indices[i as usize] as usize].pos, scene.vertices[scene.indices[i as usize + 1] as usize].pos, scene.vertices[scene.indices[i as usize + 2] as usize].pos];
            if t.iter().any(|p| p[0] > 8.0 && p[0] < 28.0 && p[1] > 10.0 && p[1] < 54.0 && p[2] > -10.0 && p[2] < 10.0) { rt.push(t); }
            i += 3;
        }
    }
    eprintln!("[cof] render tris in box {}", rt.len());
    let rdist = |p: [f32; 3]| rt.iter().map(|t| { let q = closest_on_tri(p, t); dot(sub(p, q), sub(p, q)).sqrt() }).fold(f32::INFINITY, f32::min);
    for (k, inst) in insts.iter().enumerate() {
        let ob = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &ob.collision else { continue };
        if ob.flags & 0x20 != 0 { continue; }
        for (t, m) in c.triangles_with_material() {
            let w = t.map(|p| tf(&inst.matrix, p));
            let cen = [(w[0][0] + w[1][0] + w[2][0]) / 3.0, (w[0][1] + w[1][1] + w[2][1]) / 3.0, (w[0][2] + w[1][2] + w[2][2]) / 3.0];
            if !(cen[0] > 12.0 && cen[0] < 26.0 && cen[1] > 12.0 && cen[1] < 52.0 && cen[2] > -8.0 && cen[2] < 8.0) { continue; }
            // samples: centroid + midpoints between the centroid and each vertex
            let mut worst = 0f32;
            let mut samples = vec![cen];
            for v in &w { samples.push([(cen[0] + v[0]) / 2.0, (cen[1] + v[1]) / 2.0, (cen[2] + v[2]) / 2.0]); }
            for s in &samples { worst = worst.max(rdist(*s)); }
            if worst > 0.1 {
                let n = cross(sub(w[1], w[0]), sub(w[2], w[0]));
                let l = dot(n, n).sqrt().max(1e-9);
                eprintln!("[cof] inst {k} '{}' flags {:#06x} mat {m:#04x} worst {worst:.3} cen ({:.2},{:.2},{:.2}) n ({:.2},{:.2},{:.2}) area {:.3} v {:?}",
                    ob.name.chars().take(30).collect::<String>(), ob.flags, cen[0], cen[1], cen[2], n[0] / l, n[1] / l, n[2] / l, l / 2.0,
                    w.map(|p| p.map(|x| (x * 100.0).round() / 100.0)));
            }
        }
    }
}

#[test]
#[ignore = "probe"]
fn probe_pinned_decal_sets() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).unwrap();
    for _ in 0..3 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let p = [23.219f32, 28.226, 5.772];
    let lo = [p[0] - 0.6, p[1] - 0.6, p[2] - 0.6];
    let hi = [p[0] + 0.6, p[1] + 0.6, p[2] + 0.6];
    let sim = scene.sim.as_ref().unwrap(); let dd = sim.data(); let w = dd.world;
    for (id, a, b) in w.dynamic_boxes() {
        if a[0] <= hi[0] && b[0] >= lo[0] && a[1] <= hi[1] && b[1] >= lo[1] && a[2] <= hi[2] && b[2] >= lo[2] {
            eprintln!("[sets] dynamic id {id:#x} lo {a:?} hi {b:?}");
        }
    }
    for t in w.dynamic_tris_near(lo, hi) { let q = closest_on_tri(p, &t); eprintln!("[sets] dyn tri d {:.3} {t:?}", dot(sub(p, q), sub(p, q)).sqrt()); }
    for t in w.missile_tris_near(lo, hi) { let q = closest_on_tri(p, &t); eprintln!("[sets] missile tri d {:.3} {t:?}", dot(sub(p, q), sub(p, q)).sqrt()); }
}

#[test]
#[ignore = "probe"]
fn probe_doors() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).unwrap();
    for _ in 0..3 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    for (k, d) in sim.drones.doors.list.iter().enumerate() {
        eprintln!("[door] {k} placement {} kind {} glist {:#x} ch_open {} ch_lock {} flags {:#x} state {} pos01 {:.2} t ({:.2},{:.2},{:.2})",
            d.placement, d.kind, d.glist, d.ch_open, d.ch_lock, d.flags, d.state, d.pos01, d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]);
    }
}
