//! walkreal probe for 07000011: the corridor's south leg (the ledge z 5.5-8.5, north of door #13)
//! where the route test's stand-in teleport sits. The route test's comment says the direct
//! northward walk "wedges on the corridor's static machinery row (6000 ticks at z ≈ 5.7)". This
//! probe maps the machinery row's blocking faces, the well strip's open floor, and the nav graph's
//! own route around (n12 → n45 → n46/n47), then reproduces the wedge with a lane walk. Diagnostic
//! only — run with --nocapture.

#[path = "common/walker.rs"]
mod walker;

use walker::{wrap_pi, dxz};
use nf_game::world::V3;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_07000011_corridor_south_leg() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000011.bin", false).expect("load 07000011");
    // The static geometry does not depend on the intro: probe straight after the intro tail.
    for _ in 0..4200u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 07000011 corridor south leg (the ledge z 5.5-8.5, x 8-15) ===");
    eprintln!("bond at ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);

    // 1. Floor grid: where is there floor between the corridor ledge (z 5.5-8.5) and the north
    //    strip (z −3.5..−5.4)?  The well strip z 1.5-4.5 is expected open except at x ~1.
    eprintln!("--- floor probes x 8..15 z 7.5..-6 (floor y under (x,48,z)) ---");
    let mut z = 7.5f32;
    while z >= -6.0 {
        let mut row = String::new();
        let mut x = 8.0f32;
        while x <= 15.0 {
            let f = sim.world.floor_below([x, 48.0, z], 8.0);
            row.push_str(&format!(" {:5}", match f { Some(y) => format!("{y:.1}"), None => "open".into() }));
            x += 1.0;
        }
        eprintln!("z {z:5.1}:{row}");
        z -= 1.0;
    }

    // 2. Northbound ray fans (z decreasing) at the route-test's own x rows, body heights: the
    //    machinery row's faces and the gaps.
    eprintln!("--- northbound rays (x,y,z 7.2) -> (x,y,-6.5): first hit ---");
    for &x in &[9.0f32, 10.22, 11.5, 13.0, 13.82, 14.5] {
        let mut row = String::new();
        for &y in &[46.4f32, 47.0, 47.6] {
            let hit = sim.world.segment([x, y, 7.2], [x, y, -6.5]);
            row.push_str(&format!(" y{y}={}", match hit {
                Some(t) => format!("z{:.2}", 7.2 + t * (-6.5 - 7.2)),
                None => "clear".into(),
            }));
        }
        eprintln!("x {x:5.2}:{row}");
    }

    // 3. East-west ray fans at the corridor's z rows: the machinery row's x extents.
    eprintln!("--- east-west rays (8.5,y,z) -> (15.5,y,z) at y 47.0 ---");
    for &z in &[6.6f32, 5.7, 4.8, 3.5, 2.5, 1.5, 0.5, -0.5, -1.5, -2.5, -3.6, -4.4, -5.2] {
        let hit = sim.world.segment([8.5, 47.0, z], [15.5, 47.0, z]);
        eprintln!("z {z:5.1}: {}", match hit {
            Some(t) => format!("hit x {:.2}", 8.5 + t * (15.5 - 8.5)),
            None => "clear".into(),
        });
    }

    // 4. The machinery row's height: a vertical ray under the first northbound hit row.
    eprintln!("--- vertical rays (x,44.0,z) -> (x,50.0,z) at the wedge row ---");
    for &(x, z) in &[(10.22f32, 5.7), (13.0, 5.7), (10.22, 4.8), (13.0, 4.8), (10.22, 3.5), (10.22, 2.5)] {
        let hit = sim.world.segment([x, 44.0, z], [x, 50.0, z]);
        eprintln!("x {x:5.2} z {z:4.1}: {}", match hit {
            Some(t) => format!("hit y {:.2}", 44.0 + t * 6.0),
            None => "clear (no ceiling blocker)".into(),
        });
    }

    // 5. Nav nodes in the corridor/stair region with their links: is there a game route from the
    //    corridor to the stair chain (n94 at (11.8,-6.3))?
    eprintln!("--- nav nodes with x 7..17, z -7.5..8.5, y 44..49 (per path) ---");
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        for (ni, n) in p.nodes.iter().enumerate() {
            if n.pos[0] >= 7.0 && n.pos[0] <= 17.0 && n.pos[2] >= -7.5 && n.pos[2] <= 8.5
                && n.pos[1] >= 44.0 && n.pos[1] <= 49.0 {
                let links: Vec<String> = p.links.iter()
                    .filter(|l| l.a as usize == ni || l.b as usize == ni)
                    .map(|l| format!("{}(len {:.1})", if l.a as usize == ni { l.b } else { l.a }, l.len))
                    .collect();
                eprintln!("path[{pi}] kind={:#x} node[{ni}] flags={:#x} at=({:.2},{:.2},{:.2}) links=[{}]",
                    p.kind, n.flags, n.pos[0], n.pos[1], n.pos[2], links.join(", "));
            }
        }
    }

    // 6. The nav's own answer: does a route exist corridor (13.82,47.4,6.0) -> the stairwell
    //    door's front tile (10.22,47.4,-3.6)?  And ledge-west -> the front tile?
    eprintln!("--- nav_route corridor->front tile and ledge-west->front tile ---");
    {
        let r1 = walker::nav_route(sim, [13.82, 47.4, 6.0], [10.22, 47.4, -3.6]);
        eprintln!("nav_route (13.82,6.0)->(10.22,-3.6): {:?}", r1);
        let r2 = walker::nav_route(sim, [2.2, 47.4, 6.6], [10.22, 47.4, -3.6]);
        eprintln!("nav_route (2.2,6.6)->(10.22,-3.6): {:?}", r2);
    }

    // 7. The lane-walk repro: teleport Bond into the corridor just north of door #13 and steer at
    //    the stairwell door's front tile exactly as walk_to does. Diagnostic (a probe may
    //    teleport) — the wedge this produces is the route test's documented stand-in cause.
    eprintln!("--- lane-walk repro: (13.82,47.4,6.2) steering at (10.22,47.4,-3.6) ---");
    drop(sim);
    {
        let sim = scene.sim.as_mut().unwrap();
        // The 4200 idle intro ticks leave Bond dead on this level (guards/cut-scene — the walkreal
        // tests run with top_up_health for the same reason): revive + teleport.
        sim.drones.health.health = 100.0;
        sim.bond.pos = [13.82, 47.4, 6.2];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::PI;
    }
    let target: V3 = [10.22, 47.4, -3.6];
    for t in 0..2400u32 {
        let (p, err, grounded) = {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            (p, wrap_pi(desired - sim.bond.yaw), sim.bond.grounded)
        };
        if t == 0 {
            let sim = scene.sim.as_ref().unwrap();
            eprintln!("lane repro start: disabled={} substate={:#x} health={:.1} exit_hold={:?}",
                sim.disabled, sim.bond.substate, sim.drones.health.health, sim.drones.exit_hold.is_some());
        }
        let d = dxz(p, target);
        if d < 1.3 { eprintln!("lane repro: ARRIVED at tick {t} pos ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); break; }
        let input = nf_game::player::Input {
            turn: err.clamp(-0.07, 0.07),
            forward: if d > 2.2 { 1.0 } else { 0.45 },
            ..Default::default()
        };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        if t % 60 == 0 {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            eprintln!("lane repro t{t}: pos ({:.2},{:.2},{:.2}) yaw {:.2} grounded {grounded} floor {:?}",
                p[0], p[1], p[2], sim.bond.yaw, sim.world.floor_below(p, 8.0));
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    eprintln!("lane repro end: pos ({:.2},{:.2},{:.2}) floor {:?}", p[0], p[1], p[2],
        sim.world.floor_below(p, 8.0));
}
