//! walkreal-b2 probe: the 0700000a wedge corner (−17.87, 4.87) — what does the walk contact there,
//! and which faces meet at the corner? Diagnostic — run with --nocapture (NF_CAPS_LOG=1 adds the
//! per-contact [cap] PUSH/CULL lines).

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_carry() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load 0700000a cold");
    let sim = cold.sim.as_ref().unwrap();
    let save = nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels);
    Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load 0700000a warm"))
}

/// The faces around the wedge corner: x −19..−15.5, y −0.5..3.0, z 1..6.5.
#[test]
fn probe_0a_corner_faces() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_ref().unwrap();
    let tris = sim.world.tris_near([-19.0, -0.5, 1.0], [-15.5, 3.0, 6.5]);
    eprintln!("=== corner tris: {} (x −19..−15.5, y −0.5..3, z 1..6.5)", tris.len());
    for t in &tris {
        let n = cross3(sub3(t[1], t[0]), sub3(t[2], t[0]));
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l < 1e-9 { continue; }
        let n = [n[0] / l, n[1] / l, n[2] / l];
        eprintln!("tri ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) n ({:.3},{:.3},{:.3})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2], n[0], n[1], n[2]);
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// The walk itself: the test's legs to the wedge corner, then the game's hold-forward line with
/// per-tick tracing (every 10 ticks) — where does the port wedge?
#[test]
fn probe_0a_corner_walk() {
    let mut w = load_carry();
    for _ in 0..150 {
        let done = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .filter(|s| s.placement == 524 || s.placement == 525)
            .all(|s| s.state == 0 || s.state == 0x40);
        if done { break; }
        w.wait(5);
    }
    for (to, label) in [
        ([-3.9, 0.0, 13.2], "out of the lift"),
        ([-3.9, 0.0, 15.5], "lobby south"),
        ([-10.0, 0.4, 16.0], "the west hall"),
        ([-20.5, 1.3, 16.5], "ring: the west offices' north corridor"),
        ([-20.5, 1.3, 5.5], "ring: down the west office block"),
        ([-17.6, 1.3, 5.5], "ring: the ai node 17 east of the door"),
    ] {
        w.walk_to(Wp::new(to[0], to[1], to[2]).r(0.8), label);
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            if d.pos[0] < -18.0 && d.pos[0] > -21.0 && d.pos[2] > 3.0 && d.pos[2] < 7.0 {
                eprintln!("[door] after '{label}': door #{i} state {:#x} pos01 {:.2}", d.state, d.pos01);
            }
        }
    }
    w.walk_to(Wp::new(-17.87, 1.3, 4.87).r(0.4), "the wedge corner");
    eprintln!("=== corner reached at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The game's hold-forward line: steer toward (−17.5, 2.5) every tick, tracing.
    let target = [-17.5f32, 2.5];
    for tick in 0..400u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let d = ((target[0] - p[0]).powi(2) + (target[1] - p[2]).powi(2)).sqrt();
        if d < 0.35 {
            eprintln!("[corner] tick {tick}: ARRIVED d={d:.3} at ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
            return;
        }
        let desired = (target[0] - p[0]).atan2(target[1] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: err.clamp(-0.07, 0.07),
            forward: if d > 2.2 { 1.0 } else { 0.45 },
            ..Default::default()
        };
        if tick % 10 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let (a, bb) = b.capsule_seg();
            eprintln!("[corner] tick {tick}: pos ({:.4},{:.4},{:.4}) yaw {:.3} grounded {} fny {:.3} ref ({:.3},{:.3},{:.3}) capsule a ({:.3},{:.3},{:.3}) b ({:.3},{:.3},{:.3})",
                p[0], p[1], p[2], b.yaw, b.grounded, b.floor_ny, b.pick_ref[0], b.pick_ref[1], b.pick_ref[2], a[0], a[1], a[2], bb[0], bb[1], bb[2]);
        }
        w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        w.total_ticks += 1;
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[corner] 400-tick cap: wedged at ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
}

/// What stands at (−18.4..−18.5, z ≈ 4.55)? The doors and the dynamic boxes near the corner.
#[test]
fn probe_0a_corner_objects() {
    let scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== doors within 6 m of (−18.5, 4.6) at LOAD (no ticks):");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        let dx = d.pos[0] - (-18.5); let dz = d.pos[2] - 4.6;
        if dx * dx + dz * dz < 36.0 {
            eprintln!("door #{i} pos ({:.3},{:.3},{:.3}) state {:#x} pos01 {:.2} sph ({:.2},{:.2},{:.2}) r {:.2}",
                d.pos[0], d.pos[1], d.pos[2], d.state, d.pos01, d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1);
        }
    }
    eprintln!("--- after 200 idle ticks:");
    let mut scene = scene; let sim = scene.sim.as_mut().unwrap();
    for _ in 0..200 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        let dx = d.pos[0] - (-18.5); let dz = d.pos[2] - 4.6;
        if dx * dx + dz * dz < 36.0 {
            eprintln!("door #{i} pos ({:.3},{:.3},{:.3}) state {:#x} pos01 {:.2} sph ({:.2},{:.2},{:.2}) r {:.2}",
                d.pos[0], d.pos[1], d.pos[2], d.state, d.pos01, d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1);
        }
    }
    eprintln!("=== dynamic boxes near the corner:");
    for (id, lo, hi) in sim.world.dynamic_boxes() {
        if lo[0] > -20.0 && lo[0] < -15.0 && lo[2] > 2.0 && lo[2] < 7.0 && lo[1] < 3.0 {
            eprintln!("dyn {id:#x} lo ({:.3},{:.3},{:.3}) hi ({:.3},{:.3},{:.3})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        }
    }
    eprintln!("=== static tris in the tight corner box x −18.6..−17.3, y 0.4..1.6, z 4.0..5.2:");
    for t in sim.world.tris_near([-18.6, 0.4, 4.0], [-17.3, 1.6, 5.2]) {
        let n = cross3(sub3(t[1], t[0]), sub3(t[2], t[0]));
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l < 1e-9 { continue; }
        let n = [n[0] / l, n[1] / l, n[2] / l];
        eprintln!("tri ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) n ({:.3},{:.3},{:.3})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2], n[0], n[1], n[2]);
    }
}
