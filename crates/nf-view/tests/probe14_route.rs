//! level-14 probe: the vent topology — where does the duct system go? A floor-height map of the
//! region around the store room (ray down from y=6.5 at every 0.5 grid point), plus horizontal
//! blockage scans along the candidate duct lines. Prints only; feeds the walkreal route.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_vent_topology() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }

    // 1) Floor-height map: ' ' = no floor hit within y 1..6 from above (open shaft or blocked),
    //    digits = floor height bucket. Region: the store room + everything south/east.
    eprintln!("floor height map (x 36..50 by 0.75, z 34..72 by 0.75); '.'=no hit, digit=y bucket:");
    let hdr: String = (36u32..50).map(|i| format!("{}", i % 10)).collect();
    eprintln!("z\\x  {hdr}");
    let mut z = 72.0f32;
    while z >= 33.5 {
        let mut row = String::new();
        let mut x = 36.0f32;
        while x <= 50.0 {
            let hit = sim.world.segment([x, 6.5, z], [x, 0.5, z]);
            match hit {
                None => row.push('.'),
                Some(t) => {
                    let y = 6.5 - (6.0 - 0.5) * t;
                    if y < 1.2 { row.push('a'); }        // ~0..1.2 : facility lower floor
                    else if y < 2.2 { row.push('b'); }   // ~1.2..2.2
                    else if y < 2.7 { row.push('c'); }   // ~2.2..2.7 : the vent floor 2.40
                    else if y < 4.0 { row.push('d'); }   // ~2.7..4
                    else { row.push('e'); }              // 4+ : skybridge/upper
                }
            }
            x += 0.75;
        }
        eprintln!("{z:5.1} {row}");
        z -= 0.75;
    }

    // 2) Horizontal scans: along the south duct centre (x=40.8, y=2.8) from z=64 to z=36 — where
    //    does the first block sit, and is there anything past it?
    for (label, from, to) in [
        ("south duct x=40.8 y=2.8 northward", [40.8f32, 2.8, 64.0], [40.8, 2.8, 36.0]),
        ("east duct z=63.66 y=2.8 eastward", [41.5, 2.8, 63.66], [47.0, 2.8, 63.66]),
        ("east duct z=63.66 y=2.8 westward", [41.5, 2.8, 63.66], [38.0, 2.8, 63.66]),
        ("north duct x=40.8 y=2.8 northward", [40.8, 2.8, 66.0], [40.8, 2.8, 71.0]),
        ("grate->mouth x=45.3 y=2.8 southward", [45.3, 2.8, 62.0], [45.3, 2.8, 42.0]),
        ("mouth row z=42.2 y=2.8", [39.0, 2.8, 42.2], [47.0, 2.8, 42.2]),
    ] {
        let len = ((to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2) + (to[2] - from[2]).powi(2)).sqrt();
        let hits: Vec<(f32, f32)> = {
            let mut v = Vec::new();
            let mut t = 0.0f32;
            while t <= len {
                let p = [from[0] + (to[0] - from[0]) * t / len, from[1] + (to[1] - from[1]) * t / len, from[2] + (to[2] - from[2]) * t / len];
                // A 0.55-radius sphere feel: 6 rays from the centre outward.
                let blocked = [
                    sim.world.segment(p, [p[0] + 0.5, p[1], p[2]]).map(|f| f * 0.5),
                    sim.world.segment(p, [p[0] - 0.5, p[1], p[2]]).map(|f| f * 0.5),
                    sim.world.segment(p, [p[0], p[1], p[2] + 0.5]).map(|f| f * 0.5),
                    sim.world.segment(p, [p[0], p[1], p[2] - 0.5]).map(|f| f * 0.5),
                    sim.world.segment(p, [p[0], p[1] + 0.5, p[2]]).map(|f| f * 0.5),
                    sim.world.segment(p, [p[0], p[1] - 0.5, p[2]]).map(|f| f * 0.5),
                ];
                if blocked.iter().any(|b| matches!(b, Some(d) if *d < 0.35)) { v.push((t, p[2])); }
                t += 0.25;
            }
            v
        };
        // Compress to ranges.
        let mut ranges: Vec<(f32, f32)> = Vec::new();
        for (t, z) in hits {
            match ranges.last_mut() { Some(l) if t - l.1 <= 0.3 => l.1 = t, _ => ranges.push((t, t)) }
        }
        let rs = ranges.iter().map(|(a, b)| format!("{a:.1}-{b:.1}")).collect::<Vec<_>>().join(", ");
        eprintln!("{label}: blocked@ [{rs}] (scan len {len:.1})");
    }
    let _ = &sim;
}
