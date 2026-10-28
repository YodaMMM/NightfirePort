//! drone-bump-collision-5 scratch probe (assertion-free, --nocapture): map the west mouth's
//! passable band. Walks Bond to the hall mouth, then steps him east in 0.25 m increments and
//! radial-scans the world every step (24 directions, 3 m), printing dr22 and the door state —
//! the geometry a player reads when she squeezes past the parked bot.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_the_gap() {
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
        ("the hall west mouth", [76.6, 43.6, -50.7]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    // Step east from the mouth through the door band, scanning every step.
    for step in 0..22u32 {
        let x = 76.6 + step as f32 * 0.25;
        for _ in 0..240 {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if (p[0] - x).abs() < 0.12 { break; }
            let desired = (x - p[0]).atan2(-50.7 - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 0.6 }, ..Default::default() };
            w.tick(input, false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let bond = sim.bond.pos;
        let mut scan = String::new();
        for k in 0..24 {
            let a = k as f32 * std::f32::consts::TAU / 24.0;
            let (dx, dz) = (a.sin(), a.cos());
            let clear = sim.world.segment([bond[0], 43.5, bond[2]], [bond[0] + dx * 3.0, 43.5, bond[2] + dz * 3.0]).is_none();
            scan.push(if clear { '.' } else { '#' });
        }
        let bots: Vec<String> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && !d.corpse && d.enabled
                && (d.pos[0] - bond[0]).abs() < 3.5 && (d.pos[2] - bond[2]).abs() < 3.5)
            .map(|(i, d)| format!("dr{i}({:.2},{:.2})", d.pos[0], d.pos[2]))
            .collect();
        let doors: Vec<String> = sim.drones.doors.list.iter()
            .filter(|d| (d.pos[0] - bond[0]).abs() < 3.0 && (d.pos[2] - bond[2]).abs() < 3.0)
            .map(|d| format!("door({:.2},{:.2})s{:#x}p{:.2}", d.pos[0], d.pos[2], d.state, d.pos01))
            .collect();
        eprintln!("[x{:.2}] bond ({:.2},{:.2},{:.2}) scan[+z..cw] [{scan}] {} {}",
            x, bond[0], bond[1], bond[2], bots.join(" "), doors.join(" "));
    }
}
