//! level-14 probe: the openings — from points along the south duct and the room, cast horizontal
//! rays every 5 degrees at the crawl height (y 2.9) and map the FREE distances (a 0.55-radius
//! feel: blocked when a hit sits within 0.5). Also a fine floor map at 0.25 resolution over
//! x 39..48, z 54..71. Prints only.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_openings() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    }

    // Fine floor map at y-slice 2.9: for each cell, is there FREE space for a 0.55 capsule
    // (a down-ray finds floor 2.0..2.6 AND side rays at 2.9 reach 0.55 without hits)?
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("passability at y=2.90 (x 39..48 step 0.25, z 54..71 step 0.25): '#'=blocked/blocked, '.'=crawlable");
    let mut z = 71.0f32;
    while z >= 54.0 {
        let mut row = String::new();
        let mut x = 39.0f32;
        while x <= 48.0 {
            let floor = sim.world.segment([x, 4.5, z], [x, 0.0, z]).map(|t| 4.5 - 4.5 * t);
            let open = matches!(floor, Some(f) if (1.9..2.75).contains(&f))
                && sim.world.segment([x, 2.9, z], [x + 0.55, 2.9, z]).is_none()
                && sim.world.segment([x, 2.9, z], [x - 0.55, 2.9, z]).is_none()
                && sim.world.segment([x, 2.9, z], [x, 2.9, z + 0.55]).is_none()
                && sim.world.segment([x, 2.9, z], [x, 2.9, z - 0.55]).is_none()
                && sim.world.segment([x, 2.9, z], [x, 3.45, z]).is_none();
            row.push(if open { '.' } else { '#' });
            x += 0.25;
        }
        eprintln!("{z:5.2} {row}");
        z -= 0.25;
    }

    // Radial openings from key points: how far can a 0.55 feel go each direction at y 2.9?
    for (label, p) in [
        ("the start", [40.81f32, 2.9, 65.08]),
        ("south of the lid #592", [40.81, 2.9, 63.0]),
        ("mid south duct", [40.81, 2.9, 61.0]),
        ("at the grate", [40.81, 2.9, 59.4]),
        ("east of the room", [42.5, 2.9, 64.5]),
        ("the east duct z=63.66", [44.0, 2.9, 63.66]),
    ] {
        let mut out = Vec::new();
        let mut deg = 0i32;
        while deg < 360 {
            let a = deg as f32 * std::f32::consts::PI / 180.0;
            let dir = [a.sin(), 0.0, a.cos()];
            let mut free = 0.0f32;
            while free < 6.0 {
                let q = [p[0] + dir[0] * (free + 0.55), p[1], p[2] + dir[2] * (free + 0.55)];
                if sim.world.segment(p, q).is_some() { break; }
                free += 0.25;
            }
            if free > 0.6 { out.push(format!("{deg}°:{free:.1}")); }
            deg += 5;
        }
        eprintln!("{label} @({:.1},{:.1},{:.1}): free rays [{}]", p[0], p[1], p[2], out.join(" "));
    }
}
