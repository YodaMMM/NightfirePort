//! walkreal probe (level-0c-secret-door-reach-3): the barrel-row stop. The fresh-boot
//! live run [L 2026-10-31]: walking SOUTH into the corridor (x 53.0..55.6) stops at z −150.23 in
//! every lane (a barrel row; running jumps fail), while the west walk opens at z −146/−141 (Bond
//! reaches x 50.4). This probe walks the same lanes on real input in the PORT and prints where each
//! lane stops — the [L] comparison row for the port's static collision. Diagnostic, --nocapture.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Walker};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_barrel_row_lanes() {
    let isopath = iso();
    for lane_x in [51.0f32, 52.2, 53.0, 54.3, 55.4, 56.2] {
        let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
        w.wait_enabled(3000, "the intro");
        w.wait(30);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [lane_x, 1.0, -140.0];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(20);
        let start = w.total_ticks;
        let mut stopped: Option<([f32; 3], String)> = None;
        for _ in 0..1200u32 {
            let (p, grounded) = {
                let sim = w.scene.sim.as_ref().unwrap();
                (sim.bond.pos, sim.bond.grounded)
            };
            if p[2] < -170.0 { stopped = Some((p, "reached -170".to_string())); break; }
            // Stalled: the z hasn't improved for 90 ticks while pushing south.
            if w.total_ticks - start > 240 && stopped.is_none() {
                stopped = Some((p, "stalled".to_string()));
                break;
            }
            let desired = (54.0f32 - p[0]).atan2(-178.0f32 - p[2]);
            let err = wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
            w.tick(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
        }
        // The min z reached while pushing.
        let (p, why) = stopped.unwrap_or((w.scene.sim.as_ref().unwrap().bond.pos, "timeout".into()));
        let under = w.scene.sim.as_ref().unwrap().world
            .segment_hit([p[0], p[1] + 0.5, p[2]], [p[0], p[1] - 3.0, p[2]])
            .map(|(tt, _)| p[1] + 0.5 - 3.0 * tt);
        eprintln!("[br] lane x {lane_x}: stopped at ({:.2},{:.2},{:.2}) {why} grounded {} under {under:?}",
            p[0], p[1], p[2], w.scene.sim.as_ref().unwrap().bond.grounded);
    }
    // The z -150.23 pin lane: walk strictly south (no steering) at x 54.3 and record the z wall.
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [54.3, 1.0, -144.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let mut min_z = 0.0f32;
    for _ in 0..600u32 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[2] < min_z { min_z = p[2]; }
        if p[2] < -170.0 { break; }
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[br] strict-south x 54.3: min z {min_z:.3}, final ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
}
