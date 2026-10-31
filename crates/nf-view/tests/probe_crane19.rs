//! probe19: the crane glists' collision shapes — which hashes the world holds, what entity kinds
//! the rotate script creates, and where the walkway's static 12.13 comes from. Print-only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe19: the crane shapes; run with --ignored --nocapture"]
fn probe19_shapes() {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();
    for h in [0x0200_0a9cu32, 0x0200_0abb, 0x0200_0a80] {
        eprintln!("[p19] shape {h:08x} present {}", sim.world.has_shape(h));
    }
    // The script objects at load: the crane's entities (hash, kind, translation).
    for (oi, o) in sim.drones.scripts.objects.list.iter().enumerate() {
        if let Some(nf_game::nis::Obj::Entity { hash, kind, matrix, .. }) = o {
            if matrix[3][0] > 150.0 && matrix[3][0] < 220.0 && matrix[3][2] < -230.0 {
                eprintln!("[p19] obj{oi} {hash:08x} kind {kind:#x} at ({:.1},{:.1},{:.1})",
                    matrix[3][0], matrix[3][1], matrix[3][2]);
            }
        }
    }
    // The rotate script's entity create ops (which hashes + kinds it makes).
    {
        let mut sim_m = w.scene.sim.as_mut().unwrap();
        sim_m.bond.pos = [199.6, 13.4, -263.9];
        sim_m.bond.vel = [0.0; 3];
    }
    w.wait(10);
    assert!(w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 30 }), "E #450"));
    w.wait(60);
    w.wait(700); // the swing completes
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The full END-pose map along the axis and perpendicular.
        let (dx2, dz2) = ((-3.334f32).sin(), (-3.334f32).cos());
        for r in [4.0f32, 5.0, 5.5, 6.0, 6.5, 7.0, 8.0, 9.0, 9.5, 10.0, 11.0, 12.0, 13.0] {
            for off in [-1.0f32, -0.4, 0.4] {
                let (x, z) = (200.48 + r * dx2 + off * dz2, -262.37 + r * dz2 - off * dx2);
                let mut row = String::new();
                let mut top = 19.0f32;
                for _ in 0..6 {
                    match sim.world.segment_hit([x, top, z], [x, -1.0, z]) {
                        Some((t, _)) => { let y = top - (top + 1.0) * t; row.push_str(&format!("{y:.2} ")); top = y - 0.05; }
                        None => { row.push_str("-- "); break; }
                    }
                }
                if row != "-- " { eprintln!("[p19] map r{r} off{off:+.1} ({x:.2},{z:.2}): {row}"); }
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The END-pose walkway strip: bearing -3.334, r 6.5..8, off -0.6..+0.2.
        let (dx, dz) = (0.1925f32, -0.981f32);
        for r in [6.5f32, 7.0, 7.5, 8.0] {
            for off in [-0.6f32, -0.4, -0.2, 0.0, 0.2] {
                let (x, z) = (200.48 + r * dx + off * dz, -262.37 + r * dz - off * dx);
                let hit = sim.world.segment_hit([x, 13.5, z], [x, 10.0, z]).map(|(t, _)| 13.5 - 3.5 * t);
                eprintln!("[p19] endpose r{r} off{off:+.1} ({x:.2},{z:.2}) top {hit:?}");
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The dynamic objects' boxes near the crane after the swing.
        for (id, lo, hi) in sim.world.dynamic_boxes() {
            if lo[0] > 180.0 && lo[0] < 215.0 && lo[2] < -255.0 && lo[2] > -280.0 {
                eprintln!("[p19] dyn id {id:#x} lo ({:.1},{:.1},{:.1}) hi ({:.1},{:.1},{:.1})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
            }
        }
        // The raw shape bounds.
        for h in [0x0200_0a9cu32, 0x0200_0abb] {
            if let Some(t) = sim.world.shape(h) {
                let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
                for tri in t { for v in tri { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
                eprintln!("[p19] shape {h:08x} {} tris bounds lo ({:.1},{:.1},{:.1}) hi ({:.1},{:.1},{:.1})",
                    t.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
            }
        }
    }
    for k in 0..12 {
        w.wait(60);
        let sim = w.scene.sim.as_ref().unwrap();
        let r0 = sim.drones.scripts.entity_matrix((0x0600_0a08, 0)).map(|m| m[0][2].atan2(m[0][0]));
        let r1 = sim.drones.scripts.entity_matrix((0x0600_0a08, 1)).map(|m| m[0][2].atan2(m[0][0]));
        let m1 = sim.drones.scripts.entity_matrix((0x0600_0a08, 1));
        let t1 = m1.map(|m| (m[3][0], m[3][1], m[3][2]));
        eprintln!("[p19] t{} rot0 {r0:?} rot1 {r1:?} t1 {t1:?}", (k + 1) * 60);
    }
}
