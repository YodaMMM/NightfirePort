// level-14-east-walkway scratch: find the fire spot + aim for the SkyBridge grapple latch.
// For each stance in the nook / west walkway: does a world-clearing ray exist whose closest
// approach to the SkyBridge point (45.29, 5.10, 62.94) is within the latch radius? If not,
// report the first hit of the direct ray (what blocks).
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    let point = [45.2888f32, 5.0961, 62.9362];
    let world = &sim.world;

    // wall tops along x 41.53 / 44.58: vertical down-rays from y 9
    println!("== wall tops (down from y9) ==");
    for z in [62.0f32, 63.0, 64.0, 65.0, 66.0, 67.0] {
        for x in [41.53f32, 44.58] {
            let t = world.segment([x, 9.0, z], [x, -2.5, z]).map(|t| 9.0 - t * 11.5);
            print!("x{x} z{z}: {t:?}  ");
        }
        println!();
    }

    for stance in [
        [40.6f32, 66.7], [40.6, 66.0], [40.8, 65.5], [41.0, 65.0], [41.0, 64.5],
        [41.0, 64.0], [41.0, 63.5], [40.8, 63.0], [40.8, 62.0], [40.8, 61.0], [40.8, 60.0],
    ] {
        // floor at the stance
        let fl = world.segment([stance[0], 3.5, stance[1]], [stance[0], -2.5, stance[1]]).map(|t| 3.5 - t * 6.0);
        let Some(_f) = fl else { println!("stance {:?}: no floor", stance); continue };
        let head = [stance[0], _f + 1.5, stance[1]]; // approx standing head
        let to_p = [point[0] - head[0], point[1] - head[1], point[2] - head[2]];
        let dist = (to_p[0] * to_p[0] + to_p[1] * to_p[1] + to_p[2] * to_p[2]).sqrt();
        let yaw = to_p[0].atan2(to_p[2]);
        let pitch = (to_p[1] / (to_p[0] * to_p[0] + to_p[2] * to_p[2]).sqrt()).asin();
        let (sp, cp) = (pitch.sin(), pitch.cos());
        let (sy, cy) = (yaw.sin(), yaw.cos());
        let dir = [cp * sy, sp, cp * cy];
        let end = [head[0] + dir[0] * dist, head[1] + dir[1] * dist, head[2] + dir[2] * dist];
        match world.segment(head, end) {
            Some(t) => {
                let h = [head[0] + (end[0] - head[0]) * t, head[1] + (end[1] - head[1]) * t, head[2] + (end[2] - head[2]) * t];
                println!("stance {:?} head {:.2}: DIRECT ray blocked at ({:.2}, {:.2}, {:.2}) t {:.2}", stance, head[1], h[0], h[1], h[2], t);
            }
            None => println!("stance {:?} head {:.2}: DIRECT ray CLEAR, dist {:.2}", stance, head[1], dist),
        }
        // also a low-arc scan: pitch from -0.2..0.9 rad around the direct yaw, find ANY clear ray
        // whose closest approach to the point is < 1.2 (a plausible latch sphere)
        let mut found = 0;
        for dpi in -20..=40 {
            let p2 = pitch + dpi as f32 * 0.03;
            if p2 < -0.3 || p2 > 1.2 { continue; }
            let (sp2, cp2) = (p2.sin(), p2.cos());
            let dir2 = [cp2 * sy, sp2, cp2 * cy];
            // closest approach of the ray to the point
            let w = [point[0] - head[0], point[1] - head[1], point[2] - head[2]];
            let proj = w[0] * dir2[0] + w[1] * dir2[1] + w[2] * dir2[2];
            if proj <= 0.0 { continue; }
            let cl = [head[0] + dir2[0] * proj, head[1] + dir2[1] * proj, head[2] + dir2[2] * proj];
            let miss = ((cl[0] - point[0]).powi(2) + (cl[1] - point[1]).powi(2) + (cl[2] - point[2]).powi(2)).sqrt();
            if miss > 1.2 { continue; }
            let end2 = [head[0] + dir2[0] * proj, head[1] + dir2[1] * proj, head[2] + dir2[2] * proj];
            if world.segment_clear(head, end2) {
                if found < 3 { println!("    clear aim: pitch {p2:.2} (direct {pitch:.2}) miss {miss:.2} dist {proj:.2}"); }
                found += 1;
            }
        }
        if found == 0 { println!("    no clear aim within miss 1.2"); }
    }
}
