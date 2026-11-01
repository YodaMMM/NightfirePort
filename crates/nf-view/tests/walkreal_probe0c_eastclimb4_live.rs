//! walkreal probe (level-0c-whole-walk-east-route): ALL surfaces under the east climb. For each
//! 0.25 x-column at z −211.0, −210.0, −209.25 the ray walks downward collecting every hit
//! (y 12 -> 0), so under-stair landings hidden below the staircase decks show up. Diagnostic only,
//! --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_east_climb4() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    for z in [-211.0f32, -210.25, -209.25, -208.75] {
        eprintln!("=== z {z} (x 77..89 step 0.25; every surface from y 12 down) ===");
        let row = {
            let mut row = String::new();
            for i in 0..=48 {
                let x = 77.0f32 + 0.25 * i as f32;
                let mut ys: Vec<f32> = Vec::new();
                let mut top = 12.0f32;
                loop {
                    match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
                        Some((t, _)) => {
                            let y = top - top * t;
                            ys.push(y);
                            top = y - 0.05;
                            if top <= 0.05 { break; }
                        }
                        None => break,
                    }
                }
                let s = ys.iter().map(|y| format!("{y:.2}")).collect::<Vec<_>>().join("/");
                row.push_str(&format!("{x:>5.1}[{s}] "));
            }
            row
        };
        let bytes = row.as_bytes();
        let mut start = 0usize;
        while start < bytes.len() {
            let mut cut = (start + 110).min(bytes.len());
            while cut < bytes.len() && bytes[cut] != b' ' { cut += 1; }
            eprintln!("{}", &row[start..cut]);
            start = cut;
        }
    }
    eprintln!("ok");
}
