//! drone-bump-collision-4 scratch probe (assertion-free, --nocapture): walk the hall line and
//! print every drone within 6 of Bond around the wedge point (79.19,−51.21) — enabled, kind_464,
//! state, pos — so the crowd filter's misses are visible.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_the_crowd_at_the_wedge() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    w.crowd_aware = true;
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    for (name, at) in [
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    // Manual walk into the hall floor with per-60-tick drone prints; at t=900 hold still 300
    // ticks (does any bot move?), then steer at the bypass point beside the nearest armed bot.
    let target = [83.1f32, 43.6, -50.5];
    for t in 0..2400u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let mut input = Input { turn: 0.0, forward: 1.0, ..Default::default() };
        if (900..1200).contains(&t) {
            input = Input::default();
        } else if t >= 1200 {
            // steer at the bypass: nearest armed bot within 3.4, aim 1.8 beside it
            let (gx, gz) = (target[0] - p[0], target[2] - p[2]);
            let gl = (gx * gx + gz * gz).sqrt();
            let (ux, uz) = (gx / gl, gz / gl);
            let mut best: Option<[f32; 2]> = None;
            for d in sim.drones.list.iter() {
                if !d.present || d.corpse || !d.enabled || d.kind_464 != 0x800 { continue; }
                let (rx, rz) = (d.pos[0] - p[0], d.pos[2] - p[2]);
                let r = (rx * rx + rz * rz).sqrt();
                if !(0.5..3.4).contains(&r) { continue; }
                if rx * ux + rz * uz < 0.0 { continue; }
                best = Some([d.pos[0], d.pos[2]]);
            }
            if let Some(bp) = best {
                let (rx, rz) = (bp[0] - p[0], bp[1] - p[2]);
                let side = rx * -uz + rz * ux;
                let s = if side >= 0.0 { -1.0f32 } else { 1.0f32 };
                let aim = [bp[0] + -uz * s * 1.8, bp[1] + ux * s * 1.8];
                let desired = (aim[0] - p[0]).atan2(aim[1] - p[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.2 { 0.0 } else { 1.0 }, ..Default::default() };
            }
        }
        w.tick(input, false);
        if t % 60 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let seg = sim.world.segment(p, target);
            println!("[t{t}] bond ({:.2},{:.2},{:.2}) seg_to_target {seg:?}", p[0], p[1], p[2]);
            for (i, d) in sim.drones.list.iter().enumerate() {
                let r = walker::dxz(d.pos, p);
                if r < 6.0 && d.present && !d.corpse {
                    println!("  dr{i} ({:.2},{:.2},{:.2}) r={r:.2} enabled {} kind {:#x} state {:#x} mode {:#x}",
                        d.pos[0], d.pos[1], d.pos[2], d.enabled as u8, d.kind_464, d.state, d.mode);
                }
            }
        }
        if let Some(sim) = w.scene.sim.as_ref() {
            if sim.bond.pos[0] > 82.0 { println!("[t{t}] PAST the wedge point"); break; }
        }
    }
    let _ = Act::None; let _ = Wp::new(0.0, 0.0, 0.0);
}
