//! level-0c-vent-crawl probe: the (5.33, 12.49, -110.94) "cannot crawl through the vent" spot.
//! Diagnostics only (eprintln, no asserts) — run with --nocapture.
//!
//! 1. Static map: the trench/bay floors, the north wall (z −110.39) face at body heights, what is
//!    north of it, the south wall plane at the walkway end (the vent opening's real extent), and
//!    the crate-stack route (#443 at 4.65,12.56,−114.24).
//! 2. The real-input route: the walkway → through the open vent → off the lip into the trench →
//!    crouch → try every direction from the rest spot, log each stall.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load_at(pos: [f32; 3], yaw: f32) -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..100 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, pos, yaw);
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

fn burn_display_hinges(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    for k in 0..sim.drones.locks.len() {
        if (14..=17).contains(&sim.drones.locks[k].out) {
            sim.drones.locks[k].burn_through(&mut sim.drones.channels, 1);
        }
    }
}

fn floors(world: &nf_game::world::CollisionWorld, x: f32, z: f32, top: f32, bottom: f32) -> Vec<f32> {
    let mut out = Vec::new();
    let mut y = top;
    for _ in 0..40 {
        let Some((t, hit)) = world.segment_hit([x, y, z], [x, bottom, z]) else { break };
        let hy = y + (bottom - y) * t;
        out.push(hy);
        y = hy - 0.06;
        if y < bottom { break; }
    }
    out
}

/// The trench and its walls: every face a crouched body could press at the rest spot.
#[test]
fn probe_trench_north_map() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = load_at([5.8, 15.8, -107.5], std::f32::consts::PI);
    burn_display_hinges(&mut scene);
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();

    // A. Trench floors along x (z −110.6..−111.9): the 11.84 bay floor and anything lower/north.
    println!("-- A. floors (rows z, cols x), from y 17 down to 5.8 --");
    let xs = [4.2f32, 4.6, 5.0, 5.4, 5.8, 6.2, 6.6, 7.0, 7.4];
    for z in [-110.0f32, -110.4, -110.9, -111.4, -111.9, -112.2] {
        print!("z {:7.1}: ", z);
        for x in xs {
            let fl = floors(&sim.world, x, z, 17.0, 5.8);
            let s: Vec<String> = fl.iter().map(|y| format!("{:.2}", y)).collect();
            print!(" x{:4.1}[{}] ", x, s.join(","));
        }
        println!();
    }

    // B. The north wall (z −110.39): rays along +z from inside the trench at body heights —
    //    where is it solid, where does an opening sit?
    println!("-- B. rays +z from z -111.5 to -109.0 (X = wall hit), rows y, cols x --");
    for y in [11.9f32, 12.2, 12.5, 12.8, 13.1, 13.4, 13.7, 14.0, 14.3, 14.6, 15.0, 15.4, 15.8, 16.2, 16.6, 17.0, 17.4] {
        print!("y {:5.2}: ", y);
        for x in [4.8f32, 5.2, 5.4, 5.8, 6.2, 6.6, 6.9] {
            let hit = sim.world.segment_hit([x, y, -111.5], [x, y, -109.0]);
            print!(" {:4.1}{}", x, if hit.is_some() { "X" } else { "." });
        }
        println!();
    }

    // C. What is north of the wall: floors below at z −109.9 / −109.0 / −108.0.
    println!("-- C. north-of-wall floors (from 17 down to 5.8) --");
    for z in [-109.9f32, -109.0, -108.0, -107.0] {
        print!("z {:7.1}: ", z);
        for x in [4.8f32, 5.4, 5.8, 6.2, 6.9] {
            let fl = floors(&sim.world, x, z, 17.0, 5.8);
            let s: Vec<String> = fl.iter().map(|y| format!("{:.2}", y)).collect();
            print!(" x{:4.1}[{}] ", x, s.join(","));
        }
        println!();
    }

    // D. The south wall plane at the walkway end (z −112.25): rays along −z from the walkway
    //    (z −111.5) southward at walkway/vent heights — where does the walkway actually end and
    //    what does a walking body cross?
    println!("-- D. rays -z from z -111.0 to -113.6 (X = hit), rows y, cols x --");
    for y in [14.6f32, 14.8, 15.0, 15.2, 15.5, 15.8, 16.2, 16.6, 17.0, 17.4, 17.8] {
        print!("y {:5.2}: ", y);
        for x in [4.8f32, 5.4, 5.8, 6.4, 6.9] {
            let hit = sim.world.segment_hit([x, y, -111.0], [x, y, -113.6]);
            print!(" {:4.1}{}", x, if hit.is_some() { "X" } else { "." });
        }
        println!();
    }

    // F. The side walls: rays along −x/+x from inside the trench at body heights — is the
    //    glass-roof ledge open to the case (6.27) at any body height, or walled at 4.78/6.74?
    println!("-- F. rays -x from x 6.4 to x 3.8 (rows y, cols z) / +x from x 5.0 to x 7.6 --");
    for y in [11.9f32, 12.2, 12.5, 12.8, 13.1, 13.4, 13.7, 14.0, 14.3] {
        print!("y {:5.2} -x: ", y);
        for z in [-110.6f32, -111.0, -111.4, -111.8] {
            let hit = sim.world.segment_hit([6.4, y, z], [3.8, y, z]);
            print!(" z{:7.1}{}", z, if hit.is_some() { "X" } else { "." });
        }
        print!("  | +x:");
        for z in [-110.6f32, -111.0, -111.4, -111.8] {
            let hit = sim.world.segment_hit([5.0, y, z], [7.6, y, z]);
            print!(" z{:7.1}{}", z, if hit.is_some() { "X" } else { "." });
        }
        println!();
    }

    // E. The crate stack route (#443 at 4.65,12.56,−114.24): tops and the space between crates
    //    and the trench's south face.
    println!("-- E. crate columns (from 16 down to 10.8) --");
    for (x, z) in [(4.65f32, -114.24f32), (4.2, -113.8), (5.2, -113.8), (5.8, -113.6), (4.65, -113.4), (5.4, -112.8)] {
        let fl = floors(&sim.world, x, z, 16.0, 10.8);
        let s: Vec<String> = fl.iter().map(|y| format!("{:.2}", y)).collect();
        println!("  ({:5.2},{:7.2}): [{}]", x, z, s.join(","));
    }
}

/// The real-input route: the walkway → through the open vent → the trench → crouch → try every
/// direction from the rest spot (the complaint state).
#[test]
fn probe_real_walk_into_trench_and_tries() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = load_at([5.8, 15.8, -107.5], std::f32::consts::PI);
    burn_display_hinges(&mut scene);
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    // Walk south along the walkway through the open vent and off the lip (the trench-exit walk's
    // real-input leg, reproduced here to log every step).
    for tick in 1..=700 {
        let (p, yaw) = { let s = scene.sim.as_ref().unwrap(); (s.bond.pos, s.bond.yaw) };
        let desired = (5.78f32 - p[0]).atan2(-112.9f32 - p[2]);
        let err = ((desired - yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI);
        let inp = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 0.6 { 0.0 } else { 1.0 }, ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
        if tick % 50 == 0 {
            let s = scene.sim.as_ref().unwrap();
            let p = s.bond.pos;
            eprintln!("south tick {tick}: ({:.2}, {:.2}, {:.2}) gnd {} ch1b {}", p[0], p[1], p[2], s.bond.grounded, s.drones.channels.on(0x1b));
        }
        if tick > 60 {
            let s = scene.sim.as_ref().unwrap();
            if s.bond.pos[2] < -112.0 && s.bond.grounded { break; }
        }
    }
    {
        let s = scene.sim.as_ref().unwrap();
        let p = s.bond.pos;
        eprintln!("landed in the trench: ({:.2}, {:.2}, {:.2}) gnd {}", p[0], p[1], p[2], s.bond.grounded);
    }
    // Crouch at the rest (the complaint spot) — then try each direction with real input.
    scene.sim.as_mut().unwrap().step_bond(Input { crouch: true, ..Default::default() }, false, false, false);
    for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let s = scene.sim.as_ref().unwrap();
        let p = s.bond.pos;
        eprintln!("crouched at: ({:.3}, {:.3}, {:.3}) crouched {}", p[0], p[1], p[2], s.bond.crouched);
    }
    // Direction tries: yaw = north (+z, 0), south (π), east (π/2), west (−π/2). 240 ticks each,
    // log the start and end.
    for (name, yaw) in [("north", 0.0f32), ("south", std::f32::consts::PI), ("east", -std::f32::consts::FRAC_PI_2), ("west", std::f32::consts::FRAC_PI_2)] {
        // face the direction first (turn in place)
        for _ in 0..120 {
            let s = scene.sim.as_ref().unwrap();
            let err = ((yaw - s.bond.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI);
            let inp = Input { turn: err.clamp(-0.08, 0.08), ..Default::default() };
            scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
            if err.abs() < 0.01 { break; }
        }
        let start = scene.sim.as_ref().unwrap().bond.pos;
        for _ in 0..240 {
            scene.sim.as_mut().unwrap().step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        }
        let end = scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("crouch-walk {}: ({:.3},{:.3},{:.3}) -> ({:.3},{:.3},{:.3}) gnd {}",
            name, start[0], start[1], start[2], end[0], end[1], end[2],
            scene.sim.as_ref().unwrap().bond.grounded);
        // jump attempt too (the 0.78 apex)
        for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Input { forward: 1.0, jump: true, ..Default::default() }, false, false, false); }
        let j = scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("  +jump: ({:.3},{:.3},{:.3})", j[0], j[1], j[2]);
    }
    // Rays from the final crouched position: every blocking face within 1.2 m at body heights.
    {
        let s = scene.sim.as_ref().unwrap();
        let (x, y, z) = { let p = s.bond.pos; (p[0], p[1], p[2]) };
        eprintln!("-- rays from ({:.2},{:.2},{:.2}) at body heights (dy above root) --", x, y, z);
        for dy in [-0.4f32, 0.0, 0.4, 0.8, 1.1] {
            for (dx, dz, lbl) in [(0.0, 1.2, "+z N"), (0.0, -1.2, "-z S"), (1.2, 0.0, "+x"), (-1.2, 0.0, "-x")] {
                let hit = s.world.segment_hit([x, y + dy, z], [x + dx, y + dy, z + dz]);
                if let Some((t, _)) = hit { eprintln!("  dy {dy:+.1} {lbl}: HIT at t {:.2}", t); }
            }
        }
    }
}
