//! level-05-every-objective-4: replicate the -3 test's terrace placement through the port and
//! interrogate the port's own collision at the settle point. The test places Bond at
//! (22.0, 49.2, −68.0) and he settles at (21.805, 50.035, −68.098); the disc's static world
//! there is the 49.001 floor only (probe_terrace05). What does the port's world say? Scratch
//! probe — mirrors the walkreal test's flow (nf_view::load + sim.step_bond).

use nf_game::player::Input;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let mut sim = scene.sim.expect("sim");

    sim.bond.pos = [22.0, 50.1, -68.0];
    sim.bond.vel = [0.0; 3];
    sim.bond.yaw = std::f32::consts::FRAC_PI_2;
    println!("placed: pos {:?} cc {}", sim.bond.pos, sim.bond.cc);
    println!("bond.cel initial {:?}", sim.bond.cel);

    for k in 0..15 {
        sim.step_bond(Input::default(), false, false, false);
        println!("tick {:2}: pos {:?} grounded {} ny {:.3} cel {:?} vel {:?} mode {} sub {}",
            k, sim.bond.pos, sim.bond.grounded, sim.bond.floor_ny, sim.bond.cel, sim.bond.vel,
            sim.bond.mode, sim.bond.substate);
    }

    let p = sim.bond.pos;
    let cel = sim.bond.cel;
    // The port's own floor ray shape: from pos down cc+0.6 (Player_FeetOnPoint 0x194188).
    let ray = sim.bond.cc + 0.6;
    let b = [p[0], p[1] - 1.05, p[2]]; // approx capsule B (upright: pos.y - cc)
    let to = [b[0], b[1] - ray, b[2]];
    println!("== the port's floor ray at the settle point: from {:?} to {:?} ==", p, to);
    match sim.world.segment_hit_scoped(cel, p, to) {
        Some((t, n)) => println!("scoped hit t {:.4} y {:.4} normal {:?}", t, p[1] + t * (to[1] - p[1]), n),
        None => println!("scoped hit NONE"),
    }
    match sim.world.segment_hit(p, to) {
        Some((t, n)) => println!("flat hit   t {:.4} y {:.4} normal {:?}", t, p[1] + t * (to[1] - p[1]), n),
        None => println!("flat hit   NONE"),
    }
    // A long ray straight down from high above the settle column.
    let hi = [p[0], 55.0, p[2]];
    let lo = [p[0], 30.0, p[2]];
    match sim.world.segment_hit(hi, lo) {
        Some((t, n)) => println!("long flat down-ray y {:.4} normal {:?}", 55.0 + t * (30.0 - 55.0), n),
        None => println!("long flat down-ray NONE"),
    }
    match sim.world.segment_hit_scoped(cel, hi, lo) {
        Some((t, n)) => println!("long scoped down-ray y {:.4} normal {:?}", 55.0 + t * (30.0 - 55.0), n),
        None => println!("long scoped down-ray NONE"),
    }
    println!("find_cel(settle) {:?}", sim.world.find_cel(p));
    println!("find_cel(placed) {:?}", sim.world.find_cel([22.0, 49.2, -68.0]));
    // Neighbouring columns through the port's own world.
    for (x, z) in [(22.0f32, -68.0f32), (21.8, -68.1), (23.0, -68.0), (25.0, -68.0), (21.0, -68.0)] {
        let hi = [x, 55.0, z];
        let lo = [x, 30.0, z];
        let h = sim.world.segment_hit(hi, lo).map(|(t, n)| (55.0 + t * (30.0 - 55.0), n));
        println!("port col ({x}, {z}): {:?}", h.map(|(y, n)| (y, n[1])));
    }
}
