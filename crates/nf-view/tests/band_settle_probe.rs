//! level-0a-band-settle-7 probe: the port's per-tick glide through the 0700000a west-hall
//! doorway, against the [L 2026-10-26] frames (placed 1.12 → 1.1402 @f5 → 1.1465 @f13 →
//! 1.1285 @f21+, rest 1.1285; BLData+0xcc 0.0 at every sample). Also the raw floor y under
//! the route (the lobby, the doorway, the hall). Assertion-free, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn floor_under(w: &Walker, x: f32, y: f32, z: f32) -> Option<(f32, [f32; 3])> {
    w.scene.sim.as_ref().unwrap().world.segment_hit([x, y, z], [x, y - 3.0, z])
}

fn wrap_pi(mut a: f32) -> f32 {
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}

#[test]
fn probe_glide_per_tick() {
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load"));
    w.wait_enabled(3000, "the party start");
    w.wait(30);
    // The raw floors along the route [D port world tris]
    for (tag, x, y, z) in [
        ("spawn", -3.93f32, 1.2, 10.40),
        ("lobby a", -3.9, 1.2, 13.2),
        ("lobby b", -3.9, 1.2, 15.5),
        ("doorway mouth", -8.0, 1.2, 16.29),
        ("jamb west", -8.5, 1.2, 16.5),
        ("hall entry", -9.3, 1.2, 16.7),
        ("hall mid", -10.0, 1.2, 16.83),
        ("hall deep", -11.0, 1.2, 16.83),
        ("hall deep2", -12.0, 1.2, 16.83),
    ] {
        let f = floor_under(&w, x, y, z);
        eprintln!("[floor] {tag} ({x},{y},{z}) -> {:?}", f.map(|(t, n)| (y - 3.0 * t, n)));
    }
    // The glide walk (the test's route), logging y per tick inside the doorway window.
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    let mut logging = false;
    let start = w.total_ticks;
    // Drive the through-line leg manually so we can print per tick.
    let target = [-10.0f32, 0.4, 16.29];
    loop {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if !logging && p[0] > -7.6 && p[0] < -6.5 && p[2] > 15.0 { logging = true; }
        if logging {
            let f = floor_under(&w, p[0], p[1] + 0.5, p[2]);
            eprintln!("[glide] t+{} pos ({:.4},{:.4},{:.4}) floor {:?}", w.total_ticks - start, p[0], p[1], p[2],
                f.map(|(t, n)| (p[1] + 0.5 - 3.0 * t, n)));
        }
        if (p[0] - target[0]).hypot(p[2] - target[2]) < 0.8 { break; }
        if w.total_ticks - start > 3000 { eprintln!("[glide] cap"); break; }
        let yaw = w.scene.sim.as_ref().unwrap().bond.yaw;
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let input = nf_game::player::Input {
            turn: wrap_pi(desired - yaw).clamp(-0.07, 0.07),
            forward: 1.0,
            ..Default::default()
        };
        w.tick(input, false);
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[glide] end pos ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
    for _ in 0..90 {
        let yaw = w.scene.sim.as_ref().unwrap().bond.yaw;
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let input = nf_game::player::Input {
            turn: wrap_pi(desired - yaw).clamp(-0.07, 0.07),
            forward: 0.0,
            ..Default::default()
        };
        w.tick(input, false);
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[glide] rest pos ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
}
