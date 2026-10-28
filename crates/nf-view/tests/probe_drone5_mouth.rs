//! drone-bump-collision-5 scratch probe (assertion-free, --nocapture): the west-mouth anatomy.
//!
//! 1. The disc nav nodes + flags around the hall's west mouth (x 72..90, z −58..−42) — every
//!    path, every node, so the mouth node (79.75,−50.45 [D the nav dump]) and its door/wait
//!    flags are visible [D MAPNODE+4].
//! 2. Every drone's route (path, nodes, each node's pos+flags) — whose route carries the mouth.
//! 3. The armed mouth bot (state 0x16) tracked from the boot with NO walk: does its route
//!    advance when nobody stands near it, and where does it park?

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_the_mouth_routes() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== nav nodes near the west mouth (x 72..90, z −58..−42) ===");
        for (pi, path) in sim.nav.paths.iter().enumerate() {
            for (ni, n) in path.nodes.iter().enumerate() {
                if (72.0..90.0).contains(&n.pos[0]) && (-58.0..-42.0).contains(&n.pos[2]) {
                    eprintln!("path{pi} node{ni} ({:.2},{:.2},{:.2}) flags {:#x}", n.pos[0], n.pos[1], n.pos[2], n.flags);
                }
            }
        }
        eprintln!("=== every drone's route ===");
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present { continue; }
            let r = match &d.route {
                Some(r) => {
                    let nodes = r.nodes.iter().map(|n| {
                        let np = &sim.nav.paths[r.path].nodes[*n as usize];
                        format!("{}({:.1},{:.1},f{:#x})", n, np.pos[0], np.pos[2], np.flags)
                    }).collect::<Vec<_>>().join(" -> ");
                    format!("path{} idx{} [{}]", r.path, r.index, nodes)
                }
                None => "none".to_string(),
            };
            eprintln!("dr{i} ({:.2},{:.2}) state {:#x} mode {:#x} started {} enabled {} route {}",
                d.pos[0], d.pos[2], d.state, d.mode, d.started as u8, d.enabled as u8, r);
        }
    }
    // Track the mouth bots from the boot with nobody walking: 1200 ticks, print every 120.
    for t in 0..1200u32 {
        w.tick(Input::default(), false);
        if t % 120 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            for (i, d) in sim.drones.list.iter().enumerate() {
                if !d.present || d.corpse { continue; }
                if matches!(i, 20 | 21 | 22) || (d.enabled && (76.0..82.0).contains(&d.pos[0]) && (-53.0..-47.0).contains(&d.pos[2])) {
                    let route = match &d.route {
                        Some(r) => format!("path{} idx{}/{} target({:.2},{:.2}) mode {:?} nodes{}",
                            r.path, r.index, r.nodes.len(), r.target[0], r.target[2],
                            std::mem::discriminant(&r.mode), r.nodes.len()),
                        None => "none".to_string(),
                    };
                    eprintln!("[t{t}] dr{i} ({:.2},{:.2}) state {:#x} mode {:#x} route {}",
                        d.pos[0], d.pos[2], d.state, d.mode, route);
                }
            }
        }
    }
    // Now walk the wake legs (the -4 anchors to the north row) and dump the mouth bots' routes
    // once the group plates fire, then track dr22 while standing at the mouth.
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
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== path3/4/5 links ===");
        for pi in [3usize, 4, 5] {
            for l in &sim.nav.paths[pi].links {
                eprintln!("path{pi} link {}->{} len {:.2}", l.a, l.b, l.len);
            }
        }
        eprintln!("=== mouth bots after the wake walk ===");
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !matches!(i, 20 | 21 | 22) { continue; }
            let r = match &d.route {
                Some(r) => {
                    let nodes = r.nodes.iter().map(|n| {
                        let np = &sim.nav.paths[r.path].nodes[*n as usize];
                        format!("{}({:.2},{:.2},f{:#x})", n, np.pos[0], np.pos[2], np.flags)
                    }).collect::<Vec<_>>().join(" -> ");
                    format!("path{} idx{}/{} mode {:?} [{}]", r.path, r.index, r.nodes.len(), r.mode, nodes)
                }
                None => "none".to_string(),
            };
            eprintln!("dr{i} ({:.2},{:.2}) state {:#x} mode {:#x} route {}", d.pos[0], d.pos[2], d.state, d.mode, r);
        }
    }
    for t in 0..1800u32 {
        w.tick(Input::default(), false);
        if t % 150 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            for (i, d) in sim.drones.list.iter().enumerate() {
                if !matches!(i, 20 | 21 | 22) || !d.present || d.corpse { continue; }
                let route = match &d.route {
                    Some(r) => format!("path{} idx{}/{} target({:.2},{:.2})", r.path, r.index, r.nodes.len(), r.target[0], r.target[2]),
                    None => "none".to_string(),
                };
                eprintln!("[w{t}] dr{i} ({:.2},{:.2}) state {:#x} mode {:#x} route {}", d.pos[0], d.pos[2], d.state, d.mode, route);
            }
        }
    }
    // The -4 probe's manual drive woke dr22: straight south from the north row east. Replicate it
    // until dr22 wakes (state != 0x1), then STEP ASIDE (north-east, out of the mouth band) and
    // watch her play the route: the door cycle, the crossing, the settle.
    {
        let target = [76.6f32, 43.6, -50.7];
        for t in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            let d22 = &sim.drones.list[22];
            if d22.enabled && d22.state != 0x1 && d22.state != 0x0 {
                eprintln!("[wake t{t}] dr22 state {:#x} at ({:.2},{:.2})", d22.state, d22.pos[0], d22.pos[2]);
                break;
            }
            let p = sim.bond.pos;
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, false);
            if t % 300 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[drive t{t}] bond ({:.2},{:.2}) dr22 state {:#x} ({:.2},{:.2})",
                    sim.bond.pos[0], sim.bond.pos[2], sim.drones.list[22].state, sim.drones.list[22].pos[0], sim.drones.list[22].pos[2]);
            }
        }
        // Step aside: back north-east out of the mouth band, then hold.
        for t in 0..2200u32 {
            w.tick(Input::default(), false);
            if t % 100 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[22];
                let route = match &d.route {
                    Some(r) => format!("path{} idx{}/{} target({:.2},{:.2}) creep a({:.2},{:.2}) b({:.2},{:.2}) step{}/{}",
                        r.path, r.index, r.nodes.len(), r.target[0], r.target[2],
                        r.creep.a[0], r.creep.a[2], r.creep.b[0], r.creep.b[2], r.creep.step, r.creep.steps),
                    None => "none".to_string(),
                };
                let door = sim.drones.doors.list.iter().enumerate()
                    .find(|(_, d)| dxz_door(d.pos, [80.0, -49.9]) < 1.5);
                let door_s = door.map(|(i, d)| format!("door{i} at({:.2},{:.2}) state {:#04x}", d.pos[0], d.pos[2], d.state)).unwrap_or_default();
                // What does the world say about the move toward her target?
                let sim2 = w.scene.sim.as_ref().unwrap();
                let seg_t = sim2.world.segment(d.pos, [d.route.as_ref().map(|r| r.target[0]).unwrap_or(77.48), d.pos[1], d.route.as_ref().map(|r| r.target[2]).unwrap_or(-49.77)]);
                // The move probe's own test (drone.rs probe_move): LOS_PROBE_8 rays at y and y+0.5.
                let probe = |from: [f32; 3], to: [f32; 2]| {
                    let ox = 0.4 * (to[0] - from[0]).signum();
                    let oz = 0.4 * (to[1] - from[2]).signum();
                    let ray = |y: f32| sim2.world.line_of_sight([from[0], y, from[2]], [to[0] + ox, y, to[1] + oz], nf_game::world::LOS_PROBE_8);
                    (ray(from[1]), ray(from[1] + 0.5))
                };
                let np = |n: u16| { let p = &sim2.nav.paths[3].nodes[n as usize].pos; [p[0], p[2]] };
                let (f11, f11h) = probe(d.pos, np(11));
                let (f13, f13h) = probe(d.pos, np(13));
                let (f14, f14h) = probe(d.pos, np(14));
                eprintln!("[aside t{t}] dr22 ({:.2},{:.2},{:.2}) state {:#x} route {} {} seg_t {seg_t:?} probe y/y+.5: n11 {f11}/{f11h} n13 {f13}/{f13h} n14 {f14}/{f14h}",
                    d.pos[0], d.pos[1], d.pos[2], d.state, route, door_s);
            }
        }
    }
}

/// dxz against a fixed [x, z] point.
fn dxz_door(a: [f32; 3], b: [f32; 2]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[1]).powi(2)).sqrt() }
