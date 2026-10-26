//! probe_swim_pocket — diagnostics for swim-height-02-pocket: what stops the hut-pocket swim-out
//! at (-74.01, 3.03, 23.26) with the DEFAULT-arm swim capsule?
//!
//! Replicates ladder_underwater_02_live::hut_pocket_swim_out_passes_the_ladder_like_the_ps2's run
//! (07000002, settle 200, Bond::new([-76.27, 3.17, 22.14], yaw), 168 forward ticks), prints the
//! per-tick state + the capsule push for BOTH capsule arms, then maps the geometry along the path
//! (downward rays + horizontal rays at the capsule's bottom height).
//!
//! Run: NF_SILENT=1 NF_ISO=<iso> cargo run --release -p nf-view --example probe_swim_pocket

use nf_view::scene::Sim;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let yaw: f32 = 0.4;
    let mut sim = load(&iso);
    let (world_start, world_end) = ([-76.27f32, 3.17, 22.14], [-71.14f32, 3.63, 27.94]);
    println!("=== path geometry map (start -> PS2 end), downward rays ===");
    for i in 0..=24 {
        let f = i as f32 / 24.0;
        let x = world_start[0] + (world_end[0] - world_start[0]) * f;
        let z = world_start[2] + (world_end[2] - world_start[2]) * f;
        let from = [x, 5.5, z];
        let to = [x, 0.0, z];
        match sim.world.segment_hit(from, to) {
            Some((t, n)) => {
                let y = 5.5 + t * (0.0 - 5.5);
                println!("  path {i:2} ({x:7.3},{z:7.3}): floor y {y:7.3} normal {:?}", round(n));
            }
            None => println!("  path {i:2} ({x:7.3},{z:7.3}): no floor hit down to y 0"),
        }
    }
    println!("=== lateral map at x = stop_x -1..+1, z = 22..29 (downward) ===");
    for z in [22.0f32, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0] {
        for x in [-75.5f32, -74.5, -74.0, -73.0, -72.0] {
            let from = [x, 5.5, z];
            let to = [x, 0.0, z];
            let s = match sim.world.segment_hit(from, to) {
                Some((t, n)) => format!("y {:7.3} n {:?}", 5.5 + t * -5.5, round(n)),
                None => "   none        ".to_string(),
            };
            print!(" ({x:6.1},{z:4.1}) {s} |");
        }
        println!();
    }

    println!("=== the run, yaw {yaw} ===");
    for _ in 0..200 { step(&mut sim, Default::default()); }
    sim.bond = nf_game::player::Bond::new(world_start, yaw);
    for t in 0..168 {
        step(&mut sim, nf_game::player::Input { forward: 1.0, ..Default::default() });
        if t >= 5 && t <= 60 {
            let b = &sim.bond;
            let (da, db) = b.capsule_seg();
            let (dp, _) = sim.world.capsule_push_scoped(b.cel, da, db, nf_game::player::RADIUS, true, [1.0e9, 1.0e9, 1.0e9]);
            // horizontal rays at the sphere centres' heights, along the stroke and its neighbours
            let (s, c) = b.yaw.sin_cos();
            let mut faces = String::new();
            for (dy, tag) in [(0.275f32, "A"), (0.0, "B"), (-0.5, "low")] {
                for (dx, dz, tag2) in [(s, c, "fwd"), (s*0.707 + c*0.707, c*0.707 - s*0.707, "+45"), (s*0.707 - c*0.707, -c*0.707 - s*0.707, "-45")] {
                    let from = [b.pos[0], b.pos[1] + dy, b.pos[2]];
                    let to = [from[0] + dx * 0.7, from[1], from[2] + dz * 0.7];
                    if let Some((tt, n)) = sim.world.segment_hit_scoped(b.cel, from, to) {
                        faces.push_str(&format!(" {tag}{tag2}@{:.2}n({:.2},{:.2},{:.2})", tt * 0.7, n[0], n[1], n[2]));
                    }
                }
            }
            println!("t {t:3} yaw {:6.3} pos ({:8.3},{:6.3},{:8.3}) push ({:6.3},{:6.3},{:6.3}) g {}{faces}",
                b.yaw, b.pos[0], b.pos[1], b.pos[2], dp[0], dp[1], dp[2], b.grounded as i32);
        }
        if t % 10 != 0 || t < 60 { continue; }
        let b = &sim.bond;
        // DEFAULT arm (the re-landed swim capsule) and the OLD LOW arm, at the same pos.
        let (da, db) = b.capsule_seg();
        let (la, lb) = ([b.pos[0], b.pos[1] + (0.65 - b.cc), b.pos[2]], [b.pos[0], b.pos[1] + (0.45 - b.cc), b.pos[2]]);
        let (dp, dw) = sim.world.capsule_push_scoped(b.cel, da, db, nf_game::player::RADIUS, true, [1.0e9, 1.0e9, 1.0e9]);
        let (lp, lw) = sim.world.capsule_push_scoped(b.cel, la, lb, nf_game::player::RADIUS, false, [1.0e9, 1.0e9, 1.0e9]);
        let floory = |from: [f32; 3], to: [f32; 3]| -> String {
            match sim.world.segment_hit_scoped(b.cel, from, to) {
                Some((tt, n)) => format!("y {:6.3} n {:?}", from[1] + tt * (to[1] - from[1]), round(n)),
                None => "none".to_string(),
            }
        };
        let to = [db[0], db[1] - (b.cc + 0.6), db[2]];
        println!(
            "t {t:3} pos ({:8.3},{:6.3},{:8.3}) vel1 {:6.3} cc {:.3} sub {} g {} fny {:.2} lad {:?}\n      DEF push ({:6.3},{:6.3},{:6.3}) walk {dw} | LOW push ({:6.3},{:6.3},{:6.3}) walk {lw}\n      floor(ray from root) {}",
            b.pos[0], b.pos[1], b.pos[2], b.vel[1], b.cc, b.substate, b.grounded as i32, b.floor_ny, b.ladder,
            dp[0], dp[1], dp[2], lp[0], lp[1], lp[2], floory(b.pos, to));
    }
    println!("final: {:?} sub {}", sim.bond.pos, sim.bond.substate);
}

fn round(n: [f32; 3]) -> [f32; 3] { n.map(|v| (v * 1000.0).round() / 1000.0) }

fn load(iso: &str) -> Sim {
    let mut scene = nf_view::load(iso, "07000002.bin", false).expect("load 07000002");
    scene.sim.take().expect("sim")
}

fn step(sim: &mut Sim, input: nf_game::player::Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, false);
}
