//! walkreal probe 12b: the north-wall gap's exact geometry (x 4.2..7.2, z 7.4..10.6).
//! Run: NF_ISO=… cargo test -p nf-view --test walkreal_probe12b -- --ignored --nocapture --test-threads=1

use nf_game::world::V3;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe: gap scan; run explicitly"]
fn scan_the_north_wall_gap() {
    let iso = iso();
    let scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;

    println!("==== north rays (y 0.9) from z 8.55 → 10.4, x 4.0..7.4 step 0.1 ====");
    for xi in 0..35 {
        let x = 4.0 + xi as f32 * 0.1;
        let line = match world.segment_surface([x, 0.9, 8.55], [x, 0.9, 10.4], true) {
            Some((t, _n, cls)) => format!("x {x:4.1}: hit z {:.2} class {:#04x}", 8.55 + 1.85 * t, cls),
            None => format!("x {x:4.1}: CLEAR"),
        };
        println!("{line}");
    }
    println!("==== south rays (y 0.9) from z 10.4 → 8.55 (the wall's north face) ====");
    for xi in 0..35 {
        let x = 4.0 + xi as f32 * 0.1;
        let line = match world.segment_surface([x, 0.9, 10.4], [x, 0.9, 8.55], true) {
            Some((t, _n, cls)) => format!("x {x:4.1}: hit z {:.2} class {:#04x}", 10.4 - 1.85 * t, cls),
            None => format!("x {x:4.1}: CLEAR"),
        };
        println!("{line}");
    }
    println!("==== the gap in depth: west/east rays across x at z 8.7, 9.0, 9.3 (y 0.9) ====");
    for z in [8.7f32, 9.0, 9.3] {
        for y in [0.5f32, 0.9, 1.4] {
            let w = match world.segment_surface([4.0, y, z], [7.4, y, z], true) {
                Some((t, _n, cls)) => format!("hit x {:.2} class {:#04x}", 4.0 + 3.4 * t, cls),
                None => "CLEAR".into(),
            };
            let e = match world.segment_surface([7.4, y, z], [4.0, y, z], true) {
                Some((t, _n, cls)) => format!("hit x {:.2} class {:#04x}", 7.4 - 3.4 * t, cls),
                None => "CLEAR".into(),
            };
            println!("z {z} y {y}: →east {w} | →west {e}");
        }
    }
    let _ = std::marker::PhantomData::<V3>;
}
