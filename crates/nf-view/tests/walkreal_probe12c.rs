//! walkreal probe 12c: the SS positions at rest + the gap rays AFTER the console E / call button.
//! Run: NF_ISO=… cargo test -p nf-view --test walkreal_probe12c -- --ignored --nocapture --test-threads=1

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe: ss/gap scan; run explicitly"]
fn ss_positions_and_post_e_gap() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);

    {
        let sim = w.scene.sim.as_ref().unwrap();
        println!("==== ALL SS at rest ====");
        for s in &sim.drones.sss.list {
            let p = s.matrix[3];
            println!("ss plc {} ch {:#x} pos ({:.2}, {:.2}, {:.2}) hash {:#x}", s.placement, s.channel, p[0], p[1], p[2], s.hash);
        }
    }

    // Press the console (ch 0x49 + 0x59) and the call button (ch 0x58) like the route does.
    w.scene.sim.as_mut().unwrap().bond.pos = [-1.55, 0.5, 7.2];
    w.wait(10);
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), true); }
    w.scene.sim.as_mut().unwrap().bond.pos = [1.1, 0.9, 7.4];
    w.wait(10);
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), true); }
    for _ in 0..180 { w.tick(nf_game::player::Input::default(), false); }

    {
        let sim = w.scene.sim.as_ref().unwrap();
        println!("ch 0x49 {} 0x58 {} 0x59 {}", sim.drones.channels.on(0x49), sim.drones.channels.on(0x58), sim.drones.channels.on(0x59));
        println!("==== SS after the presses ====");
        for s in &sim.drones.sss.list {
            let p = s.matrix[3];
            println!("ss plc {} ch {:#x} pos ({:.2}, {:.2}, {:.2})", s.placement, s.channel, p[0], p[1], p[2]);
        }
        let world = &sim.world;
        println!("==== gap north rays (y 0.9) x 4.6..7.2 step 0.1 AFTER the presses ====");
        for xi in 0..27 {
            let x = 4.6 + xi as f32 * 0.1;
            let line = match world.segment_surface([x, 0.9, 8.55], [x, 0.9, 10.4], true) {
                Some((t, _n, cls)) => format!("x {x:4.1}: hit z {:.2} class {:#04x}", 8.55 + 1.85 * t, cls),
                None => format!("x {x:4.1}: CLEAR"),
            };
            println!("{line}");
        }
        println!("==== gap across rays (y 0.9) at z 8.7/9.0/9.3 AFTER the presses ====");
        for z in [8.7f32, 9.0, 9.3] {
            let e = match world.segment_surface([4.0, 0.9, z], [7.4, 0.9, z], true) {
                Some((t, _n, cls)) => format!("hit x {:.2} class {:#04x}", 4.0 + 3.4 * t, cls),
                None => "CLEAR".into(),
            };
            let we = match world.segment_surface([7.4, 0.9, z], [4.0, 0.9, z], true) {
                Some((t, _n, cls)) => format!("hit x {:.2} class {:#04x}", 7.4 - 3.4 * t, cls),
                None => "CLEAR".into(),
            };
            println!("z {z}: →east {e} | →west {we}");
        }
    }
}
