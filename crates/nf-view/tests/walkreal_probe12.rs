//! walkreal probe 12: the shaft-lobby region of 07000012 — what blocks it, what lives in it.
//! Run: NF_ISO=… cargo test -p nf-view --test walkreal_probe12 -- --ignored --nocapture --test-threads=1
//! Evidence for the walkreal_07000012 route's two documented stand-ins (the door[19] pincer and
//! the walled-off shaft lobby). Not part of the pass/fail suite.

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe: dumps the shaft-lobby region; run explicitly"]
fn dump_the_lobby_region() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);

    let sim = w.scene.sim.as_ref().unwrap();
    let region = |p: [f32; 3]| p[0] > -5.0 && p[0] < 9.0 && p[1] > -2.0 && p[1] < 3.5 && p[2] > 4.0 && p[2] < 16.5;

    println!("==== DOORS in x -5..9, z 4..16.5 ====");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if region(d.pos) {
            println!("door[{i}] plc {} kind {} state {:#x} pos01 {:.2} ch_open {} ch_lock {} pos {:.2?} sph r {:.2}",
                d.placement, d.kind, d.state, d.pos01, d.ch_open, d.ch_lock, d.pos, d.sph.1);
        }
    }

    println!("==== SS in region ====");
    for s in &sim.drones.sss.list {
        let p = s.matrix[3];
        if region([p[0], p[1], p[2]]) {
            println!("ss plc {} ch {:#x} pos {:.2?} hash {:#x}", s.placement, s.channel, p, s.hash);
        }
    }

    println!("==== TRIGGERS in region (via drone triggers list) ====");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let mid = [(t.lo[0] + t.hi[0]) / 2.0, (t.lo[1] + t.hi[1]) / 2.0, (t.lo[2] + t.hi[2]) / 2.0];
        if region(mid) {
            println!("trig[{i}] plc {} kind {:#x} out {:#x} mask {:#x} lock {:#x} lo {:.2?} hi {:.2?} gone {}",
                t.placement, t.kind, t.out, t.mask, t.lock, t.lo, t.hi, t.gone);
        }
    }

    println!("==== RAYS from the office floor toward the lobby (class bytes) ====");
    let world = &sim.world;
    // West→east lanes at y 0.6 (feet) and y 1.4 (head), z 8..12.5
    for z in [7.0f32, 8.2, 9.0, 9.5, 10.0, 10.4, 11.0, 12.0, 12.6, 13.0, 13.5] {
        for y in [0.6f32, 1.4] {
            if let Some((t, n, cls)) = world.segment_surface([6.0, y, z], [0.0, y, z], true) {
                let hit = [6.0 + (0.0 - 6.0) * t, y, z];
                println!("ray (6.0,{y},{z})→(0.0) hit t {:.2} at {:.2?} class {:#04x} n {:.1?}", t, hit, cls, n);
            } else {
                println!("ray (6.0,{y},{z})→(0.0) CLEAR");
            }
        }
    }
    // South→north lanes from the console area into the lobby
    for x in [-2.0f32, -0.5, 0.5, 1.2, 2.0, 2.4, 3.2, 4.5, 6.0] {
        for y in [0.6f32, 1.4] {
            if let Some((t, n, cls)) = world.segment_surface([x, y, 8.5], [x, y, 12.5], true) {
                let hit = [x, y, 8.5 + (12.5 - 8.5) * t];
                println!("ray ({x},{y},8.5)→12.5 hit t {:.2} at {:.2?} class {:#04x} n {:.1?}", t, hit, cls, n);
            } else {
                println!("ray ({x},{y},8.5)→12.5 CLEAR");
            }
        }
    }

    println!("==== the door[19] tunnel at the keypad (x 6..8, z -2..1) ====");
    println!("door[19]: kind {} state {:#x} pos01 {:.2} pos {:.2?}", sim.drones.doors.list[19].kind,
        sim.drones.doors.list[19].state, sim.drones.doors.list[19].pos01, sim.drones.doors.list[19].pos);
    for x in [5.4f32, 6.0, 6.4, 6.85, 7.0, 7.14, 7.5, 8.0] {
        for z in [-1.4f32, -1.0, -0.7, -0.4, 0.0] {
            if let Some((t, n, cls)) = world.segment_surface([x, 1.0, z], [x, 1.0, 2.5], true) {
                println!("zray ({x},1.0,{z})→+z hit t {:.2} z {:.2} class {:#04x} n {:.1?}", t, z + (2.5 - z) * t, cls, n);
            }
        }
    }

    println!("==== FLOOR PLAN: occupancy scan y 0.9, x -6..8.5 step 0.5, z 3..14.5 step 0.5 ====");
    println!("(each cell: rays of length 1.2 in the 4 compass directions from (x, 0.9, z); '#' = ≥3 blocked (inside a wall), '+' = 1-2 blocked (near geometry), '.' = all clear)");
    for zi in 0..24 {
        let z = 3.0 + zi as f32 * 0.5;
        let mut row = String::new();
        for xi in 0..30 {
            let x = -6.0 + xi as f32 * 0.5;
            let mut blocked = 0;
            for (dx, dz) in [(1.2f32, 0.0), (-1.2, 0.0), (0.0, 1.2), (0.0, -1.2)] {
                if world.segment_surface([x, 0.9, z], [x + dx, 0.9, z + dz], true).is_some() { blocked += 1; }
            }
            row.push(if blocked >= 3 { '#' } else if blocked > 0 { '+' } else { '.' });
        }
        println!("z {z:5.1} {row}");
    }
    println!("           x: -6   -5   -4   -3   -2   -1   0    1    2    3    4    5    6    7    8");

    println!("==== the pillar at (2.23, z 11): x-scan at z 11 (east→west) ====");
    for z in [10.6f32, 10.8, 11.0, 11.2, 11.4] {
        if let Some((t, _n, cls)) = world.segment_surface([6.0, 0.9, z], [0.0, 0.9, z], true) {
            println!("z {z}: hit x {:.2} class {:#04x}", 6.0 - 6.0 * t, cls);
        } else { println!("z {z}: CLEAR"); }
    }
    println!("==== pillar z-extent: north rays at x 2.0 / 2.5 ====");
    for x in [1.9f32, 2.1, 2.23, 2.4, 2.6] {
        if let Some((t, _n, cls)) = world.segment_surface([x, 0.9, 9.6], [x, 0.9, 13.0], true) {
            println!("x {x}: hit z {:.2} class {:#04x}", 9.6 + 3.4 * t, cls);
        } else { println!("x {x}: CLEAR"); }
    }
}
