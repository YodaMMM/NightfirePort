//! probe14-ghost: the drop-slot stall at y 2.97 — WHO owns the ghost triangle? After the E-open
//! of lid SP #266 and the drop through the slot, the port wedges Bond on a rest at y 2.97 with
//! `capsule_push` walkable, push ~0, no floor tri within 3 m below and no dynamic tri in the old
//! stall box. This probe attributes every dynamic-triangle owner in the drop lane (per script
//! object hash/kind/matrix + per door/SS part), dumps the two script entities' final poses
//! (0x02000750 the animating entity, 0x02000751 the kind-0x19 entity created at key 61), and
//! dumps the lid placements' own flags/glist words. Assertion-free; run with --ignored.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn tri_box(t: &[[f32; 3]; 3]) -> ([f32; 3], [f32; 3]) {
    let lo = [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])];
    let hi = [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])];
    (lo, hi)
}

/// The world-space tris of script object index `oi` (hash + matrix from the objects list).
fn entity_tris(sim: &nf_view::scene::Sim, oi: usize) -> Option<(u32, u8, Vec<[[f32; 3]; 3]>)> {
    let (hash, kind, m) = match sim.drones.scripts.objects.list.get(oi).and_then(|o| o.as_ref())? {
        nf_game::nis::Obj::Entity { hash, kind, matrix, .. } => (*hash, *kind, *matrix),
        _ => return None,
    };
    let shape = sim.world.shape(hash)?;
    let tris = shape.iter().map(|t| t.map(|p| nf_formats::placement::transform(&m, p))).collect();
    Some((hash, kind, tris))
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_ghost_owners() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.crouched = true;
    }
    for _ in 0..15 { w.tick(nf_game::player::Input::default(), false); }
    w.walk_to(Wp::new(41.0, 2.4, 63.9).r(0.4), "the crawl south to the lip");

    // == A. the CLOSED state: who owns the slot's dynamic tris? ==
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let boxes = sim.world.dynamic_boxes();
        println!("== A. closed state: dynamic objects overlapping the drop lane ==");
        for (id, lo, hi) in &boxes {
            if hi[0] < 39.9 || lo[0] > 41.7 || hi[2] < 61.5 || lo[2] > 64.5 { continue; }
            let (owner, hash) = describe_part(sim, *id);
            println!("  id {id:#x} hash {hash:#010x} owner {owner} box y {:.3}..{:.3} z {:.3}..{:.3} x {:.3}..{:.3}", lo[1], hi[1], lo[2], hi[2], lo[0], hi[0]);
        }
        let slot = sim.world.dynamic_tris_near([40.0, 2.0, 62.1], [41.6, 2.6, 63.7]);
        println!("  slot dyn tris (dynamic_tris_near y 2.0..2.6): {}", slot.len());
    }

    // == B. E-open lid #266 (the same flow as probe14_lid) ==
    let lid_i = w.scene.sim.as_ref().unwrap().drones.scripts.players.iter()
        .position(|p| p.placement == 266).expect("lid SP #266 exists [D]");
    for k in 0..80u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (40.80f32 - sim.bond.pos[0]).atan2(63.66 - sim.bond.pos[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        let st = sim.drones.scripts.players[lid_i].state;
        let pressing = st == 1 && k % 6 == 0;
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, pressing);
        if st != 1 { println!("== B. lid #266 ACTIVATED at k{k} =="); break; }
    }
    for k in 0..120 {
        w.tick(nf_game::player::Input::default(), false);
        if k % 30 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let slot = sim.world.dynamic_tris_near([40.0, 2.0, 62.1], [41.6, 2.6, 63.7]);
            println!("  open k{k}: slot tris {} bond {:?}", slot.len(), sim.bond.pos);
        }
    }

    // == C. the OPEN state: every script entity's hash/kind/matrix in the lane, with its own
    // shape's world tris (the per-owner attribution the old stall-box dump could not give). ==
    {
        let sim = w.scene.sim.as_ref().unwrap();
        println!("== C. open state: script entities with shapes near the drop lane ==");
        for oi in 0..sim.drones.scripts.objects.list.len() {
            let Some((hash, kind, tris)) = entity_tris(sim, oi) else { continue };
            let mut lane = 0usize;
            let (mut ylo, mut yhi) = (f32::MAX, f32::MIN);
            let (mut zlo, mut zhi) = (f32::MAX, f32::MIN);
            for t in &tris {
                let (lo, hi) = tri_box(t);
                if hi[0] < 39.9 || lo[0] > 41.7 || hi[2] < 61.5 || lo[2] > 64.5 { continue; }
                lane += 1;
                ylo = ylo.min(lo[1]); yhi = yhi.max(hi[1]);
                zlo = zlo.min(lo[2]); zhi = zhi.max(hi[2]);
            }
            let m = match sim.drones.scripts.objects.list[oi].as_ref().unwrap() {
                nf_game::nis::Obj::Entity { matrix, .. } => *matrix,
                _ => continue,
            };
            println!("  obj[{oi}] hash {hash:#010x} kind {kind:#04x} trans [{:.3},{:.3},{:.3}] lane-tris {lane}{}",
                m[3][0], m[3][1], m[3][2],
                if lane > 0 { format!(" y {ylo:.3}..{yhi:.3} z {zlo:.3}..{zhi:.3}") } else { String::new() });
        }
        println!("== C2. all dynamic parts in the lane (doors/SS included) ==");
        for (id, lo, hi) in &sim.world.dynamic_boxes() {
            if hi[0] < 39.9 || lo[0] > 41.7 || hi[2] < 61.5 || lo[2] > 64.5 { continue; }
            let (owner, hash) = describe_part(sim, *id);
            println!("  id {id:#x} hash {hash:#010x} owner {owner} box y {:.3}..{:.3} z {:.3}..{:.3}", lo[1], hi[1], lo[2], hi[2]);
        }
    }

    // == D. the drop: walk south through the slot; log Bond's rest and the owners under him. ==
    let mut dropped = false;
    for k in 0..400u32 {
        let (y, z) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos[1], sim.bond.pos[2]) };
        if y < 0.5 && z < 63.0 { dropped = true; break; }
        let desired = std::f32::consts::PI; // south (−z) the whole way — no east turn mid-fall
        let sim = w.scene.sim.as_ref().unwrap();
        let err = wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { forward: 1.0, turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
        if k % 40 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            println!("  drop k{k}: {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        println!("== D. after the drop (fell={dropped}): pos {p:?} grounded {}", sim.bond.grounded);
        let (a, b) = sim.bond.capsule_seg();
        let (push, walkable) = sim.world.capsule_push(a, b, 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        println!("  capsule_push {push:?} walkable {walkable} (a {a:?} b {b:?} crouched {})", sim.bond.crouched);
        println!("  segment 3 m down: {:?}", sim.world.segment(p, [p[0], p[1] - 3.0, p[2]]));
        // == D3. the CONTACT faces at the rest point: replicate capsule_tri_contact's core over
        // static + missile + dynamic tris in the query box and print every face with dist < r.
        {
            let r = 0.55f32;
            let grow = r + 0.1;
            let lo = [0, 1, 2].map(|k| a[k].min(b[k]) - grow);
            let hi = [0, 1, 2].map(|k| a[k].max(b[k]) + grow);
            let sub = |u: [f32; 3], v: [f32; 3]| [u[0] - v[0], u[1] - v[1], u[2] - v[2]];
            let dot = |u: [f32; 3], v: [f32; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
            let cross = |u: [f32; 3], v: [f32; 3]| [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let closest_on_tri = |p: [f32; 3], t: &[[f32; 3]; 3]| -> ([f32; 3], bool) {
                // brute-force: the closest of the face point / the three edges' closest points
                let (mut best, mut on_edge) = (t[0], false);
                let mut bd = dot(sub(p, t[0]), sub(p, t[0]));
                // face interior via barycentric projection
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                let n = cross(e1, e2);
                let nl2 = dot(n, n);
                if nl2 > 1e-12 {
                    let d = dot(sub(p, t[0]), n) / nl2;
                    let q = sub(p, [n[0] * d, n[1] * d, n[2] * d]);
                    // barycentric
                    let (v1, v2, v3) = (e1, e2, sub(q, t[0]));
                    let d00 = dot(v1, v1); let d01 = dot(v1, v2); let d11 = dot(v2, v2); let d20 = dot(v3, v1); let d21 = dot(v3, v2);
                    let den = d00 * d11 - d01 * d01;
                    if den.abs() > 1e-12 {
                        let v = (d11 * d20 - d01 * d21) / den;
                        let ww = (d00 * d21 - d01 * d20) / den;
                        let u = 1.0 - v - ww;
                        if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) && (0.0..=1.0).contains(&ww) {
                            let cand = [t[0][0] + e1[0] * v + e2[0] * ww, t[0][1] + e1[1] * v + e2[1] * ww, t[0][2] + e1[2] * v + e2[2] * ww];
                            let dd = dot(sub(p, cand), sub(p, cand));
                            if dd < bd { bd = dd; best = cand; on_edge = false; }
                        }
                    }
                }
                for (i, j) in [(0usize, 1usize), (1, 2), (2, 0)] {
                    let (s, e) = (t[i], t[j]);
                    let d = sub(e, s);
                    let tt = dot(sub(p, s), d) / dot(d, d).max(1e-12);
                    let tt = tt.clamp(0.0, 1.0);
                    let cand = [s[0] + d[0] * tt, s[1] + d[1] * tt, s[2] + d[2] * tt];
                    let dd = dot(sub(p, cand), sub(p, cand));
                    if dd < bd { bd = dd; best = cand; on_edge = true; }
                }
                (best, on_edge)
            };
            let report = |tag: &str, tris: &[[[f32; 3]; 3]]| {
                for (ti, t) in tris.iter().enumerate() {
                    let tl = [0, 1, 2].map(|k| t[0][k].min(t[1][k]).min(t[2][k]));
                    let th = [0, 1, 2].map(|k| t[0][k].max(t[1][k]).max(t[2][k]));
                    let sl = [0, 1, 2].map(|k| a[k].min(b[k]) - grow);
                    let sh = [0, 1, 2].map(|k| a[k].max(b[k]) + grow);
                    if (0..3).any(|k| th[k] < sl[k] || tl[k] > sh[k]) { continue; }
                    let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
                    let nl = dot(n, n).sqrt();
                    if nl < 1e-12 { continue; }
                    let n = [n[0] / nl, n[1] / nl, n[2] / nl];
                    let ab = sub(b, a);
                    let denom = dot(n, ab);
                    let mut f = 0.0f32;
                    if denom.abs() > 0.0002 {
                        f = (-(dot(n, sub(a, t[0]))) / denom).clamp(0.0, 1.0);
                        let ab_len = dot(ab, ab).sqrt();
                        let nd = denom / ab_len;
                        if (nd > 0.999 && f > 0.5) || (nd < -0.999 && f < 0.5) { continue; }
                    }
                    let p2 = if denom.abs() > 0.0002 {
                        [a[0] + ab[0] * f, a[1] + ab[1] * f, a[2] + ab[2] * f]
                    } else { a };
                    let (q, _on_edge) = closest_on_tri(p2, t);
                    let v = sub(p2, q);
                    let dist = dot(v, v).sqrt();
                    if dist < r + 0.02 {
                        println!("  D3 {tag} tri[{ti}] n [{:.2},{:.2},{:.2}] q [{:.3},{:.3},{:.3}] dist {dist:.4} (r {r}) f {f:.2}",
                            n[0], n[1], n[2], q[0], q[1], q[2]);
                    }
                }
            };
            let stat_tris = sim.world.tris_near(lo, hi);
            let dyn_tris = sim.world.dynamic_tris_near(lo, hi);
            println!("== D3. faces within r+0.02 of the spine at the rest point (stat {} dyn {} in box) ==", stat_tris.len(), dyn_tris.len());
            report("stat", &stat_tris);
            report("dyn", &dyn_tris);
        }
        // every owner with tris within 1.5 m of Bond
        println!("== D2. owners with tris near the rest point ==");
        for (id, lo, hi) in &sim.world.dynamic_boxes() {
            if hi[0] < p[0] - 1.5 || lo[0] > p[0] + 1.5 || hi[2] < p[2] - 1.5 || lo[2] > p[2] + 1.5 { continue; }
            let (owner, hash) = describe_part(sim, *id);
            println!("  id {id:#x} hash {hash:#010x} owner {owner} box y {:.3}..{:.3} z {:.3}..{:.3}", lo[1], hi[1], lo[2], hi[2]);
        }
        for oi in 0..sim.drones.scripts.objects.list.len() {
            let Some((hash, kind, tris)) = entity_tris(sim, oi) else { continue };
            let near: Vec<[f32; 3]> = tris.iter().filter(|t| {
                let (lo, hi) = tri_box(t);
                hi[0] >= p[0] - 1.5 && lo[0] <= p[0] + 1.5 && hi[2] >= p[2] - 1.5 && lo[2] <= p[2] + 1.5
            }).map(|t| [t[0][1], t[1][1], t[2][1]]).collect();
            if !near.is_empty() {
                let ymin = near.iter().fold(f32::MAX, |a, t| a.min(*t.iter().min_by(|a, b| a.total_cmp(b)).unwrap()));
                let ymax = near.iter().fold(f32::MIN, |a, t| a.max(*t.iter().max_by(|a, b| a.total_cmp(b)).unwrap()));
                println!("  obj[{oi}] hash {hash:#010x} kind {kind:#04x}: {} tris near, y {ymin:.3}..{ymax:.3}", near.len());
            }
        }
    }

    // == E. the lid SP placements' own flags (the static-cel question: does #266 carry 0x8000?). ==
    {
        use nf_formats::level::Level;
        let data = {
            use nf_formats::iso9660::IsoImage;
            let img = IsoImage::open(&iso).expect("open iso");
            let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
            let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
            img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
        };
        let lv = Level::parse(&data).expect("parse");
        let (instances, _) = lv.instances(false);
        println!("== E. the vent-lid SP placements ==");
        for (pi, inst) in instances.iter().enumerate() {
            let name = ""; // names live on the object, not the placement; print by position instead
            let _ = name;
            let t = inst.placement.translation;
            if t[2] > 40.0 && t[2] < 70.0 && t[1] > 1.5 && t[1] < 3.5 && (t[0] - 40.8).abs() < 0.6 || ((t[0] - 45.28).abs() < 0.3 && (t[2] - 63.66).abs() < 0.3) {
                println!("  plc #{pi} flags {:#010x} (creates_cel {}) hash {:#010x} trans [{:.3},{:.3},{:.3}] params {:?}",
                    inst.placement.flags, inst.placement.creates_cel(), inst.placement.hashcode, t[0], t[1], t[2],
                    inst.placement.params.iter().take(4).collect::<Vec<_>>());
            }
        }
    }
}

/// Describe a dynamic part id: script object (id<2^40), door (bit 40), SS (bit 41), breakable pane (bit 42).
fn describe_part(sim: &nf_view::scene::Sim, id: u64) -> (String, u32) {
    if id & 0x100_0000_0000 != 0 {
        let k = (id & 0xff_ffff_ffff) as usize;
        return (format!("door[{k}]"), sim.drones.doors.list.get(k).map(|d| d.glist).unwrap_or(0));
    }
    if id & 0x200_0000_0000 != 0 {
        let k = (id & 0xff_ffff_ffff) as usize;
        return (format!("ss[{k}]"), sim.drones.sss.list.get(k).map(|s| s.hash).unwrap_or(0));
    }
    let oi = (id >> 6) as usize;
    match sim.drones.scripts.objects.list.get(oi).and_then(|o| o.as_ref()) {
        Some(nf_game::nis::Obj::Entity { hash, kind, .. }) => (format!("script-obj[{oi}] kind {kind:#04x}"), *hash),
        Some(nf_game::nis::Obj::Anim { skin, .. }) => (format!("script-anim[{oi}] skin {skin:#x}"), 0),
        _ => (format!("script-obj[{oi}] (gone)"), 0),
    }
}
