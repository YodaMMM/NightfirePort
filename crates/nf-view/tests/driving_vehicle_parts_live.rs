//! Every drawn vehicle in the driving view poses its own moving parts (nf_view::driving::CarRig on
//! the traffic cars and on each hostile palette slot): the rotors (kind 0xfc / 0xfb, `R(v/256 ·
//! frame °)` [M FUN_001bf3d8 / FUN_001bf458]), the camera-facing item (0xee [M FUN_001bfc20]) and
//! the `Skel` wheels from that car's own spin, steer and springs (`FUN_001d2cb0`, as the player's).
//! Opt-in: NF_ISO.

use nf_formats::carmesh::Attach;
use nf_formats::carpose::{own_matrix, Mat4};
use nf_view::driving::CarRig;

fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut o = [[0.0f32; 4]; 4];
    for i in 0..4 { for j in 0..4 { o[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum(); } }
    o
}

/// The inverse of a rigid row-vector matrix (orthonormal rows 0-2, translation in row 3).
fn rigid_inv(m: &Mat4) -> Mat4 {
    let mut o = [[0.0f32; 4]; 4];
    for i in 0..3 { for j in 0..3 { o[i][j] = m[j][i]; } }
    for j in 0..3 { o[3][j] = -(0..3).map(|k| m[3][k] * o[k][j]).sum::<f32>(); }
    o[3][3] = 1.0;
    o
}

fn kind(r: &CarRig, pi: usize) -> u8 {
    match r.mesh.parts[pi].attach {
        Attach::Skel(_) => 0xfe,
        Attach::Other(k, _) => k,
        Attach::Fixed => 0xff,
    }
}

fn summary(r: &CarRig) -> String {
    let mut k: Vec<u8> = r.parts.iter().map(|&(pi, _)| kind(r, pi)).collect();
    k.sort();
    k.dedup();
    format!("{} parts, kinds {:02x?}", r.parts.len(), k)
}

/// The part-local matrix the palette holds for `part_slot` on a body drawn at `body`.
fn local(pal: &[Mat4], body: &Mat4, part_slot: u32) -> Mat4 { mul(&pal[part_slot as usize], &rigid_inv(body)) }

fn angle_deg(a: [f32; 3], b: [f32; 3], axis: [f32; 3]) -> f32 {
    // Signed angle from a to b about axis.
    let c = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let s = c[0] * axis[0] + c[1] * axis[1] + c[2] * axis[2];
    let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    s.atan2(d).to_degrees()
}

#[test]
fn driving_vehicle_parts_live() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    let mut rotor_checked = 0usize;
    let mut rotor_models = std::collections::BTreeMap::<String, usize>::new();
    let mut wheel_checked = 0usize;
    let mut hostile_wheels = std::collections::BTreeMap::<String, usize>::new();
    // Each mission driven with a scripted throttle; then, where the track has enemy helicopters,
    // staged: the first helicopter element released (`EActivateAIElement`'s HELD clear) and the
    // player car held beside its spawn point so the range manager spawns it [S: a test stage,
    // not a route the mission drives].
    let staged = [("MIS3", "openheli"), ("MIS4", "subcopter"), ("MIS11", "helimine"), ("MIS13A", "jungleheli")];
    let runs = ["MIS01", "MIS3", "MIS4", "MIS11", "MIS13A", "MIS13B", "MIS13C", "RACE"].map(|v| (v, None)).into_iter().chain(staged.map(|(v, m)| (v, Some(m))));
    for (viv, stage) in runs {
        let (scene, mut d) = nf_view::driving::load_driving_scene(&iso, viv).expect("load");
        // A rotor's blades reach 5.57 from its hub [D openheli.crp]: its long triangles are drawn.
        if let Some(m) = stage.filter(|m| *m != "helimine") {
            let h = d.hostiles.as_ref().unwrap();
            let p = h.pools.iter().find(|p| p.model.eq_ignore_ascii_case(m)).expect("pool");
            let r = p.rigs[0].as_ref().expect("rig");
            let mut longest = 0.0f32;
            for &(pi, ps) in &r.parts {
                if !matches!(r.mesh.parts[pi].attach, Attach::Other(0xfc | 0xfb, _)) { continue; }
                let set: std::collections::HashSet<[u32; 3]> = scene.char_verts.iter().filter(|v| v.attachment == Some(ps)).map(|v| v.pos.map(f32::to_bits)).collect();
                for t in scene.indices.chunks(3) {
                    let q = [0, 1, 2].map(|k| scene.vertices[t[k] as usize].pos);
                    if !q.iter().all(|x| set.contains(&x.map(f32::to_bits))) { continue; }
                    for (a, b) in [(0, 1), (1, 2), (0, 2)] {
                        longest = longest.max((0..3).map(|k| (q[a][k] - q[b][k]).powi(2)).sum::<f32>().sqrt());
                    }
                }
            }
            eprintln!("{viv} {m}: longest rotor triangle edge {longest:.2}");
            assert!(longest > 5.0, "{viv} {m}: the rotor blades' long triangles are drawn ({longest})");
        }
        let hold = stage.map(|m| {
            let h = d.hostiles.as_mut().expect("hostiles");
            let (i, p) = h.world.db.elements.iter().enumerate().find(|(_, e)| e.rec.name.eq_ignore_ascii_case(m)).map(|(i, e)| (i, e.rec.pos)).expect("a helicopter element");
            h.world.activate(i);
            [p[0] + 40.0, p[1] + 5.0, p[2]]
        });
        let traffic: Vec<String> = d.ai_rigs.iter().map(|r| r.as_ref().map(summary).unwrap_or_else(|| "no rig".into())).collect();
        let pools: Vec<String> = d.hostiles.as_ref().map(|h| h.pools.iter().map(|p| format!("{} x{}: {}", p.model, p.slots.len(), p.rigs[0].as_ref().map(summary).unwrap_or_else(|| "no rig".into()))).collect()).unwrap_or_default();
        eprintln!("{viv}: traffic {traffic:?}; hostiles {pools:?}");
        assert_eq!(d.ai_rigs.len(), d.ai_cars.len(), "{viv}: a rig entry per traffic car");
        // Each posed part has its own slot carrying vertices, distinct from its body slot.
        for (k, r) in d.ai_rigs.iter().enumerate() {
            let Some(r) = r else { continue };
            for &(_, ps) in &r.parts {
                assert_ne!(ps, d.ai_cars[k].1, "{viv}: a posed part has its own slot");
                assert!(scene.char_verts.iter().any(|v| v.attachment == Some(ps)), "{viv}: slot {ps} has the part's vertices");
            }
        }

        let mut prev: Option<(u32, Vec<Mat4>)> = None;
        let mut rotor_here = 0usize;
        let mut wheel_here = 0usize;
        for t in 0..if hold.is_some() { 300 } else { 2400 } {
            let throttle = if hold.is_some() { 0.0 } else if t % 600 < 450 { 0.7 } else { 0.0 };
            if let Some(p) = hold { d.car.pos = p; d.car.vel = [0.0; 3]; }
            d.step(throttle, 0.0, 0.0, false);
            let mut pal = vec![[[1.0f32; 4]; 4]; scene.attachments.len()];
            let eye = d.chase_camera().0;
            d.apply_to_palette(0, &mut pal);
            if let Some((pt, pp)) = &prev {
                assert_eq!(*pt + 1, d.ticks);
                // Hostiles: each drawn element's rotors turn by v/256 degrees per tick about their
                // own axis; the 0xee item faces the camera.
                for (dr, slot) in d.hostile_draws() {
                    let h = d.hostiles.as_ref().unwrap();
                    let Some(pool) = h.pools.iter().find(|p| p.slots.contains(&slot)) else { continue };
                    let si = pool.slots.iter().position(|&s| s == slot).unwrap();
                    let Some(r) = pool.rigs[si].as_ref() else { continue };
                    let body = pal[slot as usize];
                    let pbody = pp[slot as usize];
                    if pbody == [[0.0; 4]; 4] { continue; }
                    for &(pi, ps) in &r.parts {
                        let part = &r.mesh.parts[pi];
                        match part.attach {
                            Attach::Other(k @ (0xfc | 0xfb), v) => {
                                let axis = if k == 0xfc { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
                                let own = own_matrix(part);
                                let inv_own = rigid_inv(&own);
                                // R = local · own⁻¹ (local = R · own); compare one row turned between ticks.
                                let r0 = mul(&local(&pal, &body, ps), &inv_own);
                                let r1 = mul(&local(pp, &pbody, ps), &inv_own);
                                let probe = 0;
                                let a = [r1[probe][0], r1[probe][1], r1[probe][2]];
                                let b = [r0[probe][0], r0[probe][1], r0[probe][2]];
                                let want = (v as i16) as f32 / 256.0;
                                let got = angle_deg(a, b, axis);
                                let wrap = |x: f32| (x + 540.0).rem_euclid(360.0) - 180.0;
                                assert!((wrap(got - want)).abs() < 0.05, "{viv} t{t}: {} element {} rotor {pi} kind {k:02x} turned {got:.3}° in a tick, want {want:.3}°", dr.model, dr.element);
                                rotor_here += 1;
                                *rotor_models.entry(format!("{viv} {}", dr.model)).or_default() += 1;
                            }
                            Attach::Other(0xee, _) if part.flags & 0x10 == 0 => {
                                // Local +z, taken in world space without the car's rotation (the
                                // game's rule [M]), points from the camera through the item, level.
                                let l = local(&pal, &body, ps);
                                let o = [part.origin[0] + body[3][0] - eye[0], 0.0, part.origin[2] + body[3][2] - eye[2]];
                                let n = (o[0] * o[0] + o[2] * o[2]).sqrt();
                                let z = [l[2][0], l[2][1], l[2][2]];
                                let dot = (z[0] * o[0] + z[2] * o[2]) / n;
                                assert!(dot > 0.98, "{viv} t{t}: 0xee faces the camera ({dot})");
                            }
                            Attach::Skel(_) => {
                                // A moving hostile car's wheels turn with it.
                                let v = h.world.live.iter().find(|v| v.element == dr.element).map(|v| v.vel()).unwrap_or([0.0; 3]);
                                if (v[0] * v[0] + v[2] * v[2]).sqrt() < 2.0 { continue; }
                                let a = local(&pal, &body, ps);
                                let b = local(pp, &pbody, ps);
                                let diff: f32 = (0..3).flat_map(|i| (0..3).map(move |j| (i, j))).map(|(i, j)| (a[i][j] - b[i][j]).abs()).sum();
                                if diff > 1e-3 { *hostile_wheels.entry(format!("{viv} {}", dr.model)).or_default() += 1; }
                            }
                            _ => {}
                        }
                    }
                }
                // Traffic: a moving car's wheels turn between ticks.
                for (k, (drv, slot)) in d.ai_cars.iter().enumerate() {
                    let Some(r) = d.ai_rigs[k].as_ref() else { continue };
                    let speed = (drv.car.vel[0].powi(2) + drv.car.vel[2].powi(2)).sqrt();
                    if speed < 2.0 { continue; }
                    let body = pal[*slot as usize];
                    let pbody = pp[*slot as usize];
                    let mut changed = 0;
                    let mut wheels = 0;
                    for &(pi, ps) in &r.parts {
                        if !matches!(r.mesh.parts[pi].attach, Attach::Skel(_)) { continue; }
                        wheels += 1;
                        let a = local(&pal, &body, ps);
                        let b = local(pp, &pbody, ps);
                        let diff: f32 = (0..3).flat_map(|i| (0..3).map(move |j| (i, j))).map(|(i, j)| (a[i][j] - b[i][j]).abs()).sum();
                        if diff > 1e-3 { changed += 1; }
                    }
                    if wheels > 0 {
                        assert!(changed > 0, "{viv} t{t}: traffic car {k} at {speed:.1} u/s: no wheel turned");
                        wheel_here += 1;
                    }
                }
            }
            prev = Some((d.ticks, pal));
        }
        eprintln!("{viv}{}: {rotor_here} rotor tick checks, {wheel_here} moving traffic wheel checks", stage.map(|m| format!(" (staged {m})")).unwrap_or_default());
        rotor_checked += rotor_here;
        wheel_checked += wheel_here;
    }
    eprintln!("rotor tick checks by model: {rotor_models:?}");
    eprintln!("hostile wheel turns by model: {hostile_wheels:?}");
    assert!(rotor_checked > 0, "a rotor was checked");
    // The helicopter elements (docs/systems/driving-ai.md §11: MIS3 openheli, MIS13A jungleheli).
    for k in ["MIS3 openheli", "MIS13A jungleheli"] {
        assert!(rotor_models.get(k).is_some_and(|&n| n > 0), "{k}: the enemy helicopter's rotor turned by its documented angle");
    }
    assert!(wheel_checked > 0, "a moving traffic car's wheels were checked");
    assert!(hostile_wheels.get("MIS4 supersnow").is_some_and(|&n| n > 0), "a chasing hostile's wheels turned");
}

/// The camera-facing item: the MIS01 helicopter's one 0xee part [D car-model.md §11.2], posed
/// through `CarRig::write_parts` on a car drawn with no rotation, turns its +z away from the
/// camera, level, wherever the camera stands.
#[test]
fn driving_vehicle_parts_live_camera_facing_item() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&path).expect("open NF_ISO");
    let e = iso.find("/DRIVING/MIS01.VIV").expect("MIS01.VIV");
    let g = nf_formats::disc::read_bigf(&iso, e).expect("bigf");
    let x = g.entries.iter().find(|x| x.name.eq_ignore_ascii_case(r"data\car\model\helicopter.crp")).expect("helicopter.crp");
    let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).expect("read");
    let crp = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
    let mesh = nf_formats::carmesh::car_mesh(&crp).expect("mesh");
    let parts: Vec<(usize, u32)> = mesh.parts.iter().enumerate().filter(|(_, p)| matches!(p.attach, Attach::Other(0xee, _))).map(|(i, _)| (i, i as u32)).collect();
    assert_eq!(parts.len(), 1, "one 0xee item");
    let pose = nf_formats::carpose::CarPose::rest(&mesh);
    let rig = CarRig {
        rig: nf_formats::carpose::WheelRig::new(&mesh, false),
        attrs: nf_formats::carpose::WheelAttrs { rest: 0.0, limit_draw: 0.0, wheel_radius: 0.4, is_sub: false },
        mem: Default::default(),
        pose,
        spin: [0.0; 2],
        parts: parts.clone(),
        mesh,
    };
    let (pi, ps) = parts[0];
    let origin = rig.mesh.parts[pi].origin;
    let car = [100.0f32, 5.0, -40.0];
    let car_m: Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [car[0], car[1], car[2], 1.0]];
    for (k, eye) in [[110.0f32, 9.0, -60.0], [80.0, 2.0, -40.0], [100.0, 30.0, 0.0], [130.0, 5.0, -10.0]].into_iter().enumerate() {
        let mut pal = vec![[[0.0f32; 4]; 4]; rig.mesh.parts.len()];
        rig.write_parts(&car_m, 17.0 + k as f32, eye, 0, &mut pal);
        let m = pal[ps as usize];
        let item = [origin[0] + car[0], origin[1] + car[1], origin[2] + car[2]];
        let d = [item[0] - eye[0], item[2] - eye[2]];
        let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let z = [m[2][0], m[2][1], m[2][2]];
        let dot = (z[0] * d[0] + z[2] * d[1]) / n;
        assert!(dot > 0.999 && z[1].abs() < 1e-4, "eye {eye:?}: +z points from the camera through the item, level ({z:?}, dot {dot})");
        assert!((0..3).all(|i| (m[3][i] - item[i]).abs() < 1e-3), "the item stays at its origin on the car");
    }
}
