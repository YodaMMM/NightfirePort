//! walkreal probe 12d: the lobby floor grid (cold) — where is there floor at all?
//! Run: NF_ISO=… cargo test -p nf-view --test walkreal_probe12d -- --ignored --nocapture --test-threads=1

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe: lobby floor grid; run explicitly"]
fn lobby_floor_grid() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    let sim = w.scene.sim.as_ref().unwrap();
    let world = &sim.world;
    println!("==== floor rays down (from y 1.2 to −2) x 0.5..6.5 step 0.5, z 9.4..13.4 step 0.4 ====");
    for zi in 0..11 {
        let z = 9.4 + zi as f32 * 0.4;
        let mut row = String::new();
        for xi in 0..13 {
            let x = 0.5 + xi as f32 * 0.5;
            match world.segment_surface([x, 1.2, z], [x, 1.2, -2.0], true) {
                Some((t, _n, cls)) => row.push_str(&format!("{:.2}/{:02x} ", 1.2 - 3.2 * t, cls)),
                None => row.push_str("----     "),
            }
        }
        println!("z {z:4.1}: {row}");
    }
    println!("(height/class; ---- = no floor down to −2)");
    // Also: what does the DYNAMIC set hold near the lobby? (doors/SS leaves/platforms)
    println!("==== dynamic objects near the lobby (from the door + ss lists) ====");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        let p = d.pos;
        if p[0] > -2.0 && p[0] < 8.0 && p[2] > 8.0 && p[2] < 15.0 {
            println!("door[{i}] plc {} state {:#x} pos {:.2?}", d.placement, d.state, p);
        }
    }
    for s in &sim.drones.sss.list {
        let p = s.matrix[3];
        if p[0] > -2.0 && p[0] < 8.0 && p[2] > 8.0 && p[2] < 15.0 {
            println!("ss plc {} ch {:#x} pos {:.2?}", s.placement, s.channel, p);
        }
    }
}
