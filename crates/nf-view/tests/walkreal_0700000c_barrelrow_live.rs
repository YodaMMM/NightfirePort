//! level-0c-secret-door-reach-3: the corridor's north end is CLOSED by a barrel row — the [L
//! 2026-10-31] fresh-boot runs walked the game's Bond south into the corridor (x 53.0..55.6) and
//! the body stopped at z −150.23 in every lane (running jumps fail). The row is NOT a placement —
//! the 0700000c archive's dynamic kinds in that box are cover nodes and drones only [D the
//! corridor dump] — it is the level's own static collision, and the port carries it (this test
//! pins the same stop from the north through the real input path). The strict-south lane's east
//! slide past the row's east end (x ~55.7..61 at z −150..−152) is probe-printed, not asserted —
//! the game's east end was never measured [S, live check filed].
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Walker};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// One lane: teleport to (x, 1.0, −140) — the warehouse side north of the row — and steer south
/// into the corridor mouth on real input; return the resting spot.
fn lane_stop(iso: &str, x: f32) -> [f32; 3] {
    let mut w = Walker::new(nf_view::load_with_carry(iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [x, 1.0, -140.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let start = w.total_ticks;
    let mut best_z = 0.0f32;
    loop {
        let (p, yaw) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.yaw)
        };
        if p[2] < best_z { best_z = p[2]; }
        if p[2] < -170.0 { panic!("lane x {x}: the south walk crossed the row (min z {best_z:.2}) — the pin's premise broke"); }
        if w.total_ticks - start > 400 && p[2] > best_z + 0.05 { break; } // stalled on the wall
        if w.total_ticks - start > 1500 { break; }
        let desired = (54.0f32 - p[0]).atan2(-178.0f32 - p[2]);
        let err = wrap_pi(desired - yaw).clamp(-0.08, 0.08);
        w.tick(Input { turn: err, forward: 1.0, ..Default::default() }, false);
    }
    w.scene.sim.as_ref().unwrap().bond.pos
}

#[test]
fn the_barrel_row_stops_the_corridors_north_end() {
    let iso = iso();
    // The [L] lanes: every lane x 53.0..55.6 stopped at z −150.23. The port's stop per lane
    // (measured in walkreal_probe0c_barrelrow_live): 53.0 → −150.24, 54.3 → −150.23,
    // 55.4 → −150.74 — the same wall, the ±0.5 lane-edge slope.
    for x in [53.0f32, 54.3, 55.4] {
        let p = lane_stop(&iso, x);
        eprintln!("[br] lane x {x}: stopped at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        assert!(p[2] > -151.6 && p[2] < -149.5,
            "lane x {x}: the south walk must stop on the barrel row at z ≈ −150.2 (the game's own stop [L 2026-10-31]; the port's static collision), got ({:.2},{:.2},{:.2})",
            p[0], p[1], p[2]);
    }
    // The west neighbour: x 51 stalls on the row's west end at −150.74 (the probe's print).
    let p = lane_stop(&iso, 51.0);
    eprintln!("[br] lane x 51: stopped at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    assert!(p[2] > -151.6 && p[2] < -149.5,
        "lane x 51: the row's west end must stop the walk too, got ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    eprintln!("[br] the barrel row pinned: the port's static collision stops every lane at z ≈ −150.2, matching [L 2026-10-31]");
}
