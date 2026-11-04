// real-health-14-00agent-5 scratch: how does the walk get past the mezzanine west lip?
//
// The driver's fresh boot [L]: Bond placed crouched at (45.6, 3.05, 39.87) facing west stops at
// x 45.134 and stays — the same number the port's fallback holds (the -4 probe). So the hold is
// the GAME's own behaviour and the -4 honest_ref "release" was a divergence, dropped. The
// question this probe answers: what gets the walk past this spot — the route's own legs (south
// first, then west), a jump, or the stance?
//
// Arms:
//   A  fresh-boot repro: crouched west crawl at z 39.87 — expect the ~45.134 pin [L repro].
//   B  the walk's route: standing, the lane south (45.0, 40.0) → (44.9, 39.2) → west across the
//      connector to (40.7, 39.0), the walk_to steering law (turn ±0.07, the d>2.2 forward law) —
//      where does root x cross 45.134 (the z at the crossing), does it reach the #301 band?
//   C  the lip's z extent: crouched west crawls from (45.6, *, z) for z 39.87/39.6/39.3/39.0/
//      38.7 — at which z does the hold end (the pass z band)?
//   D  jump: crouched at the pin, forward + a hop every 30 grounded ticks — does the hop clear?
//   E  standing west crawl at the pin — hold or pass?
//
// Success signal for the westward legs: root x < 44.0 (well past the hold) or the #301
// TouchOnce fires (sim.disabled — the cut-scene grabs Bond inside the band).
use nf_game::player::Input;

fn wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}

const HOLD_X: f32 = 45.134;

fn settle(sim: &mut nf_view::scene::Sim, cap: u32) {
    for _ in 0..cap {
        if sim.bond.grounded && sim.bond.vel[1].abs() < 0.01 { return; }
        sim.step_bond(Input::default(), false, false, false);
    }
}

/// Steer to a target with the walker's law. Returns (reached, final pos). Logs the tick/root-x
/// first crosses below HOLD_X (the crossing z is the interesting number).
fn steer_to(
    sim: &mut nf_view::scene::Sim, target: [f32; 2], arrive: f32, max_ticks: u32, label: &str,
    cross_log: &mut Option<(u32, f32, f32)>, verbose: bool,
) -> (bool, [f32; 3]) {
    let start_x = sim.bond.pos[0];
    for t in 0..max_ticks {
        let p = sim.bond.pos;
        let d = ((target[0] - p[0]).powi(2) + (target[1] - p[2]).powi(2)).sqrt();
        if d < arrive {
            if verbose { println!("  [{label}] arrived t {t} at ({:.3},{:.3},{:.3})", p[0], p[1], p[2]); }
            return (true, p);
        }
        if cross_log.is_none() && start_x >= HOLD_X && p[0] < HOLD_X {
            *cross_log = Some((t, p[0], p[2]));
            println!("  [{label}] x crossed below {HOLD_X} at t {t}: ({:.3},{:.3},{:.3})", p[0], p[1], p[2]);
        }
        let desired = (target[0] - p[0]).atan2(target[1] - p[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        let input = Input {
            turn: err.clamp(-0.07, 0.07),
            forward: if d > 2.2 { 1.0 } else { 0.45 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        if t % 100 == 0 && verbose {
            let p = sim.bond.pos;
            println!("  [{label}] t {t}: ({:.3},{:.3},{:.3}) grounded {}", p[0], p[1], p[2], sim.bond.grounded);
        }
    }
    let p = sim.bond.pos;
    println!("  [{label}] CAP at ({:.3},{:.3},{:.3})", p[0], p[1], p[2]);
    (false, p)
}

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");

    // ---- Arm A: the fresh-boot repro (crouched west crawl at z 39.87) ----
    {
        let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [45.6, 3.05, 39.87];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.bond.crouched = true;
        sim.bond.cc = 0.65;
        sim.bond.vel = [0.0; 3];
        settle(sim, 120);
        println!("== A fresh-boot repro: settled pos=({:.4},{:.4},{:.4}) yaw={:.4} crouched={}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.yaw, sim.bond.crouched);
        for t in 0..600u32 {
            sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
            if t % 100 == 0 || t == 599 {
                let p = sim.bond.pos;
                println!("  A t {t:>3}: ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
            }
        }
        println!("  A final x {:.4} (the fresh boot [L] pins 45.134)", sim.bond.pos[0]);
    }

    // ---- Arm B: the walk's route (standing, south first, then west) ----
    {
        let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
        let sim = scene.sim.as_mut().unwrap();
        // The release lands Bond standing on the lane (~(45.4, *, 42.2)); pose close to it.
        sim.bond.pos = [45.35, 3.45, 42.2];
        sim.bond.crouched = false;
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        settle(sim, 200);
        println!("== B the walk's route (standing): settled pos=({:.4},{:.4},{:.4}) grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded);
        let mut cross = None;
        let (r1, _) = steer_to(sim, [45.0, 40.0], 0.8, 1200, "lane south past the mouth", &mut cross, true);
        let (r2, _) = steer_to(sim, [44.9, 39.2], 0.8, 1200, "lane south to the connector", &mut cross, true);
        let (r3, p3) = steer_to(sim, [40.7, 39.0], 1.2, 2400, "mezzanine west across the connector", &mut cross, true);
        let in_band = (39.68..41.75).contains(&p3[0]) && (37.62..40.40).contains(&p3[2]);
        println!("  B legs reached {r1}/{r2}/{r3}; crossing log {cross:?}; final ({:.3},{:.3},{:.3}) #301 band {in_band} disabled {}",
            p3[0], p3[1], p3[2], sim.disabled);
    }

    // ---- Arm C: the lip's z extent (crouched west crawls) ----
    for z in [39.87f32, 39.4, 39.0, 38.6, 38.25, 37.9] {
        let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [45.6, 3.3, z];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.bond.crouched = true;
        sim.bond.cc = 0.65;
        sim.bond.vel = [0.0; 3];
        settle(sim, 200);
        let settled = sim.bond.pos;
        for _ in 0..600u32 {
            sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        }
        let p = sim.bond.pos;
        println!("== C z {z}: settled ({:.3},{:.3},{:.3}) -> 600 west ticks ({:.4},{:.4},{:.4}) {}",
            settled[0], settled[1], settled[2], p[0], p[1], p[2],
            if p[0] < 44.0 { "PAST" } else if (p[0] - HOLD_X).abs() < 0.05 { "HELD" } else { "partial" });
    }

    // ---- Arm D: the jump candidate (crouched at the pin, hop while crawling west) ----
    {
        let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [45.6, 3.05, 39.87];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.bond.crouched = true;
        sim.bond.cc = 0.65;
        sim.bond.vel = [0.0; 3];
        settle(sim, 120);
        println!("== D jump: settled pos=({:.4},{:.4},{:.4})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
        let mut hops = 0;
        for t in 0..900u32 {
            let grounded = sim.bond.grounded;
            sim.step_bond(Input { forward: 1.0, jump: grounded && t % 30 == 0, ..Default::default() }, false, false, false);
            if grounded && t % 30 == 0 { hops += 1; }
            if t % 150 == 0 {
                let p = sim.bond.pos;
                println!("  D t {t:>3}: ({:.4},{:.4},{:.4}) hops {hops}", p[0], p[1], p[2]);
            }
            if sim.bond.pos[0] < 44.0 { println!("  D t {t}: PAST the lip (x < 44) after {hops} hops"); break; }
        }
        let p = sim.bond.pos;
        println!("  D final ({:.4},{:.4},{:.4}) hops {hops} {}", p[0], p[1], p[2],
            if p[0] < 44.0 { "PAST" } else { "held" });
    }

    // ---- Arm E: standing west crawl at the pin ----
    {
        let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [45.6, 3.5, 39.87];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.bond.crouched = false;
        sim.bond.vel = [0.0; 3];
        settle(sim, 200);
        println!("== E standing: settled pos=({:.4},{:.4},{:.4}) grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded);
        for t in 0..600u32 {
            sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
            if t % 150 == 0 || t == 599 {
                let p = sim.bond.pos;
                println!("  E t {t:>3}: ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
            }
            if sim.bond.pos[0] < 44.0 { println!("  E t {t}: PAST the lip (x < 44)"); break; }
        }
    }
}
