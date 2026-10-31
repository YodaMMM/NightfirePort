//! Probe: the 07000016 launch pit's standable surfaces vs HurtPoly-ShuttleLaunch's tris.
//! For every 0.5 m cell over the volume's footprint, every collision surface between y −26 and
//! −17 (repeated downward casts) is tested with the port's player hurt capsule (feet → +1.8,
//! r 0.55). Prints the hurt tri bands, then a map: ' ' no surface, '#' every surface touched,
//! a digit = a SAFE surface exists (the digit = its height above −24 in 0.5 m steps).
//! Opt-in via NF_ISO.
fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let scene = nf_view::load_with_carry(&iso, "07000016.bin", false, None).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for (i, h) in sim.hurts.iter().enumerate() {
        eprintln!("hurt[{i}] placement {} gate {:#x} dmg {} tris {} lo {:?} hi {:?}", h.placement, h.gate, h.damage, h.tris.len(), h.lo, h.hi);
        let mut ys: Vec<i32> = h.tris.iter().map(|t| ((t[0][1] + t[1][1] + t[2][1]) / 3.0 * 100.0).round() as i32).collect();
        ys.sort(); ys.dedup();
        for y in ys {
            let band: Vec<_> = h.tris.iter().filter(|t| (((t[0][1] + t[1][1] + t[2][1]) / 3.0 * 100.0).round() as i32) == y).collect();
            let mut lo = [f32::MAX; 3]; let mut hi = [-f32::MAX; 3];
            for t in &band { for v in t.iter() { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
            eprintln!("   band y {:.2}: {} tris x {:.2}..{:.2} y {:.2}..{:.2} z {:.2}..{:.2}", y as f32 / 100.0, band.len(), lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]);
        }
    }
    let hurt = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("the ch-0xf3 volume");
    let (x0, x1, z0, z1) = (-32.0f32, 36.0f32, -198.0f32, -133.0f32);
    let mut safe_list = Vec::new();
    let mut z = z1;
    while z >= z0 {
        let mut row = String::new();
        let mut x = x0;
        while x <= x1 {
            let mut top = -16.5f32;
            let mut ch = ' ';
            let mut best: Option<f32> = None;
            let mut touched = false;
            for _ in 0..8 {
                let Some(y) = sim.world.floor_below([x, top - 0.5, z], (top - 0.5) + 26.0 - 0.5) else { break };
                if y < -26.0 { break; }
                // A standable surface needs 1.8 m of headroom.
                let clear = sim.world.segment([x, y + 0.05, z], [x, y + 1.8, z]).is_none();
                if clear {
                    if hurt.contains([x, y, z], [x, y + 1.8, z], 0.55) { touched = true; } else if best.map_or(true, |b| y > b) { best = Some(y); }
                }
                top = y - 0.05;
            }
            if let Some(y) = best {
                let d = (((y + 24.0) / 0.5).floor().clamp(0.0, 9.0)) as u8;
                ch = (b'0' + d) as char;
                if y < -17.0 { safe_list.push((x, y, z)); }
            } else if touched { ch = '#'; }
            row.push(ch);
            x += 0.5;
        }
        println!("{z:8.1} |{row}|");
        z -= 0.5;
    }
    println!("x from {x0} step 0.5");
    for (x, y, z) in safe_list.iter().filter(|p| p.1 < -19.0) { println!("safe low surface ({x:.1},{y:.2},{z:.1})"); }
}
