//! level-0b-waiting-room-doors probe (assertion-free diagnostic, run with --ignored): the
//! waiting-room exit's two doors on 0700000b — the port's list #3 (13.71, 46.78, 19.44) and #0
//! (13.82, 46.78, 7.09). The walkreal stand-in says the doors E-open (state 0x2 → 0x4) but the
//! walker wedges short of #3 and #0's doorway still blocks. This probe dumps each door's disc
//! anim (the 0x19 keys), the leaf shape's world AABB at every interp end, the doorway crossing
//! segments before/after an E, and whether the blocker is the leaf (dynamic) or a static cel.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn aabb(tris: &[[[f32; 3]; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for t in tris {
        for p in t {
            for c in 0..3 {
                lo[c] = lo[c].min(p[c]);
                hi[c] = hi[c].max(p[c]);
            }
        }
    }
    (lo, hi)
}

#[test]
#[ignore]
fn probe_0b_waiting_room_doors() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000b.bin", false).expect("load"));
    w.stop_on_exit = false;
    for _ in 0..3000u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            if sim.disabled { sim.drones.scripts.skip_request = true; }
        }
        w.tick(Default::default(), false);
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [8.0, 46.8, 24.8];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);

    // 1. The exec-floor doors: identity + params + pose state.
    eprintln!("--- doors near the exec floor (|y-47.5|<5) ---");
    let mut targets: Vec<usize> = Vec::new();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            if (d.pos[1] - 47.5).abs() < 5.0 {
                eprintln!("door #{i} placement {} pos ({:.2},{:.2},{:.2}) kind {} group {} ch_open {} ch_lock {} flags {:#x} frames {} glist {:#x} sph ({:.2},{:.2},{:.2}) r {:.2} state {:#x} pos01 {:.2} speed {:.4} swing {:.1} dir {:.1} m_t ({:.2},{:.2},{:.2})",
                    d.placement, d.pos[0], d.pos[1], d.pos[2], d.kind, d.group, d.ch_open, d.ch_lock, d.flags, d.frames, d.glist,
                    d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1, d.state, d.pos01, d.speed, d.swing, d.dir,
                    d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]);
                if (d.pos[0] - 13.71).abs() < 0.6 && (d.pos[2] - 19.44).abs() < 0.6 { targets.push(i); }
                if (d.pos[0] - 13.82).abs() < 0.6 && (d.pos[2] - 7.09).abs() < 0.6 { targets.push(i); }
            }
        }
    }
    eprintln!("--- probe targets: {targets:?} ---");

    // 2. For each target: the disc anim's keys and the leaf shape's world AABB at the interp ends.
    for &i in &targets {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[i];
        eprintln!("=== door #{i} (glist {:#x}) keys ===", d.glist);
        match (&d.keys, sim.world.shape(d.glist)) {
            (Some(keys), Some(shape)) => {
                for (k, key) in keys.iter().enumerate() {
                    eprintln!("  key[{k}] pos ({:.3},{:.3},{:.3}) quat ({:.4},{:.4},{:.4},{:.4})",
                        key.pos[0], key.pos[1], key.pos[2], key.quat[0], key.quat[1], key.quat[2], key.quat[3]);
                }
                eprintln!("  leaf shape: {} tris", shape.len());
                for &t in &[0.0f32, 0.25, 0.5, 0.75, 1.0] {
                    if let Some(m) = nf_game::door::interp(keys, 0, d.frames, t * d.frames as f32, true) {
                        let tris: Vec<[[f32; 3]; 3]> = shape.iter()
                            .map(|tri| tri.map(|p| {
                                let v = [p[0], p[1], p[2], 1.0];
                                let mut o = [0.0f32; 3];
                                for c in 0..3 { o[c] = (0..4).map(|r| m[c][r] * v[r]).sum(); }
                                o
                            })).collect();
                        let (lo, hi) = aabb(&tris);
                        eprintln!("  t {t:.2}: leaf AABB lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})  m_t ({:.2},{:.2},{:.2})",
                            lo[0], lo[1], lo[2], hi[0], hi[1], hi[2], m[3][0], m[3][1], m[3][2]);
                    } else {
                        eprintln!("  t {t:.2}: interp None");
                    }
                }
            }
            (keys, shape) => eprintln!("  keys {:?} shape {}", keys.is_some(), shape.map(|s| s.len()).unwrap_or(0)),
        }
    }

    // 3. The doorway crossings before any E: the walker-leg lines and axis rays at body heights.
    let crossings: Vec<(&str, [f32; 3], [f32; 3])> = vec![
        ("door3 waiting-room→east room", [12.4, 47.4, 19.5], [15.0, 47.4, 19.4]),
        ("door3 east-room→waiting room", [15.0, 47.4, 19.4], [12.4, 47.4, 19.5]),
        ("door0 east-room→ring", [14.0, 47.4, 7.0], [10.21, 47.18, 8.2]),
        ("door0 ring→east-room", [10.21, 47.18, 8.2], [14.0, 47.4, 7.0]),
    ];
    eprintln!("--- crossings BEFORE the E ---");
    for (label, a, b) in &crossings {
        let hit = w.scene.sim.as_ref().unwrap().world.segment(*a, *b);
        eprintln!("{label}: {}->{} t {}", fmt(a), fmt(b), hit.map(|t| format!("{t:.3}")).unwrap_or("clear".into()));
    }
    // The dynamic shapes near each doorway (what the leaf contributes).
    for &i in &targets {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.doors.list[i].pos;
        let (lo, hi) = ([p[0] - 3.0, p[1] - 3.0, p[2] - 3.0], [p[0] + 3.0, p[1] + 3.0, p[2] + 3.0]);
        let tris = sim.world.dynamic_tris_near(lo, hi);
        let (alo, ahi) = aabb(&tris);
        eprintln!("door #{i}: dynamic tris within 3 m: {} (AABB lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2}))",
            tris.len(), alo[0], alo[1], alo[2], ahi[0], ahi[1], ahi[2]);
    }

    // 4. The E repro: stand on the room side, press E, wait for the swing, re-dump + re-cross.
    for &i in &targets {
        let (dpos, label) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.drones.doors.list[i].pos;
            (p, format!("door #{i}"))
        };
        // Bond stands 1.2 m north of the door (the waiting-room/east-room side per the door's z).
        let stand = [dpos[0], dpos[1] + 0.6, dpos[2] + 1.2];
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.disabled = false;
            sim.bond.pos = stand;
            sim.bond.vel = [0.0; 3];
        }
        w.wait(8);
        w.use_door(i, &label);
        for k in 0..20 {
            w.wait(6);
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.doors.list[i];
            eprintln!("{label} +{} ticks: state {:#x} pos01 {:.3} dir {:.1} flags {:#x} busy {}", (k + 1) * 6,
                d.state, d.pos01, d.dir, d.flags, d.flags & 0x10 != 0);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[i];
        eprintln!("--- after E: {label} state {:#x} pos01 {:.3} dir {:.1} m_t ({:.2},{:.2},{:.2})",
            d.state, d.pos01, d.dir, d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]);
        for (clabel, a, b) in &crossings {
            let hit = sim.world.segment(*a, *b);
            eprintln!("{clabel}: t {}", hit.map(|t| format!("{t:.3}")).unwrap_or("clear".into()));
        }
        // Which dynamic triangles sit in the doorway plane now?
        let (lo, hi) = ([dpos[0] - 2.0, dpos[1] - 2.0, dpos[2] - 2.0], [dpos[0] + 2.0, dpos[1] + 2.0, dpos[2] + 2.0]);
        let tris = sim.world.dynamic_tris_near(lo, hi);
        let (alo, ahi) = aabb(&tris);
        eprintln!("{label}: dynamic tris within 2 m after E: {} (AABB lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2}))",
            tris.len(), alo[0], alo[1], alo[2], ahi[0], ahi[1], ahi[2]);
    }

    // 5. The approach lane into door #3 from the waiting room (the 2.5-4.3 m wedge): a fine
    //    ray fan along the lane at body heights so the blocker's extent shows.
    eprintln!("--- the door #3 approach lane rays (waiting room → the door), y 46.9/47.4/47.9 ---");
    for &z in &[24.0f32, 23.0, 22.0, 21.0, 20.5] {
        let mut row = String::new();
        for &y in &[46.9f32, 47.4, 47.9] {
            let hit = w.scene.sim.as_ref().unwrap().world.segment([10.0, y, z], [13.6, y, z]);
            row.push_str(&format!(" y{y}={}", hit.map(|t| format!("x{:.2}", 10.0 + t * 3.6)).unwrap_or("clear".into())));
        }
        eprintln!("z {z:5.1}: {row}");
    }

    // 6. The arrival recess (the cab step-off, ~(6.7, 46.5, 24.8)): how does the walker get out?
    //    The nav's own route to the waiting room's node 0.55 (9.18, 21.33), and a wall-ray fan
    //    around the recess at body heights.
    eprintln!("--- the arrival recess exit ---");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("nav_route (6.7,46.6,24.8)->(9.18,21.33): {:?}",
            walker::nav_route(sim, [6.7, 46.6, 24.8], [9.18, 47.18, 21.33]));
        eprintln!("nav_route (6.7,46.6,24.8)->(10.5,22.5): {:?}",
            walker::nav_route(sim, [6.7, 46.6, 24.8], [10.5, 47.4, 22.5]));
        for &(a, b) in &[
            ([7.5f32, 24.9], [10.5f32, 22.5]),
            ([7.5, 24.9], [9.2, 21.3]),
            ([7.5, 24.9], [8.0, 22.0]),
            ([7.5, 24.9], [6.0, 21.5]),
            ([7.5, 24.9], [5.0, 23.5]),
            ([7.5, 24.9], [10.0, 25.5]),
        ] {
            let mut row = String::new();
            for &y in &[46.7f32, 47.0, 47.4] {
                let hit = sim.world.segment([a[0], y, a[1]], [b[0], y, b[1]]);
                row.push_str(&format!(" y{y}={}", hit.map(|t| format!("({:.2},{:.2})",
                    a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1]))).unwrap_or("clear".into())));
            }
            eprintln!("recess ({:.1},{:.1})->({:.1},{:.1}):{row}", a[0], a[1], b[0], b[1]);
        }
        // The floor under the recess neighbourhood (is the recess a pit with one low lip?).
        for z in [26.0f32, 25.0, 24.0, 23.0, 22.0] {
            let mut row = String::new();
            for x in [5.0f32, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0] {
                let f = sim.world.floor_below([x, 48.0, z], 8.0);
                row.push_str(&format!(" {:5}", match f { Some(y) => format!("{y:.1}"), None => "open".into() }));
            }
            eprintln!("floor z {z:4.1}:{row}");
        }
    }
}

fn fmt(a: &[f32; 3]) -> String { format!("({:.2},{:.2},{:.2})", a[0], a[1], a[2]) }
