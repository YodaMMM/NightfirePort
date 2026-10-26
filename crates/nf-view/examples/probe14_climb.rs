//! Scratch probe (level-14-lid-and-climb): the Q-Claw climb's geometry at the z 42.16 mouths —
//! whether the aim ray from the corridor stand spots reaches the grapple points #626/#657 through
//! the port's collision (static + dynamic lids), and what floor the hang-release lands on.
//! Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);

    let points: [(&str, [f32; 3]); 2] = [("#626", [45.30, 4.79, 42.86]), ("#657", [40.78, 4.79, 42.86])];
    let stands: [[f32; 3]; 6] = [
        [45.30, -0.60, 43.40],
        [45.30, -0.60, 44.20],
        [45.30, -0.60, 45.00],
        [40.78, -0.60, 43.40],
        [40.78, -0.60, 44.20],
        [43.00, -0.60, 44.60],
    ];
    for (label, gp) in points {
        println!("== grapple {label} at {gp:?} ==");
        for s in stands {
            if (s[0] - gp[0]).abs() > 3.0 {
                continue;
            }
            let from = [s[0], s[1] + 1.0, s[2]]; // the head
            let dir = [gp[0] - from[0], gp[1] - from[1], gp[2] - from[2]];
            let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            let dir = [dir[0] / len, dir[1] / len, dir[2] / len];
            let t = sim.world.segment(from, [gp[0], gp[1], gp[2]]);
            let hit = t.map(|t| {
                [
                    from[0] + (gp[0] - from[0]) * t,
                    from[1] + (gp[1] - from[1]) * t,
                    from[2] + (gp[2] - from[2]) * t,
                ]
            });
            let pick = nf_game::gadget::pick_grapple_point(&sim.drones.grapple_points, from, dir, 60.0);
            println!(
                "  stand ({:.2}, {:.2}, {:.2}): ray len {:.2}; blocked {:?}; pick {:?}",
                s[0], s[1], s[2], len, hit, pick
            );
        }
    }

    // The mezzanine floor around the mouths: down-segs from y 4.2.
    println!("-- floor under the mouth area (from y 4.2 down 5 m) --");
    for z in [41.0, 41.5, 42.16, 42.5, 43.0, 43.6, 44.0, 44.5, 45.0] {
        print!("z {:>5}: ", z);
        for x in [40.2, 40.78, 41.5, 44.6, 45.3, 46.0] {
            let p = [x, 4.2, z];
            let t = sim.world.segment(p, [p[0], p[1] - 5.0, p[2]]);
            match t {
                Some(t) => print!(" x{:.2} y{:.2}", x, p[1] - t * 5.0),
                None => print!(" x{:.2} open", x),
            }
        }
        println!();
    }
    println!("grapple points in the port: {}", sim.drones.grapple_points.len());
    for g in &sim.drones.grapple_points {
        println!("  {:?}", g.pos);
    }
}
