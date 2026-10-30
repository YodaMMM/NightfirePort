//! level-1b-scripted-carry: the geometry probe — the boundary box around the 1b shaft (the west
//! wall x 4.433, the top slab 16.74, the pit floor −16.56, the end-walls z ±20.40 and their
//! deck-level windows). The probes below are the task's final state: the end-wall map (both ray
//! directions — the segment query is one-sided [M 0x10a8c8:168]) and the window y bands.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let scene = nf_view::load(&iso_path, "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let w = &sim.world;
    println!("== the end-walls at z ±20.40 (both dirs) ==");
    for (x, y) in [(6.5f32, -9.5), (6.5, 0.5), (6.5, 8.5), (10.0, -6.5), (14.0, -3.5), (19.0, 2.5), (23.0, -9.5)] {
        for zs in [20.4f32, -20.4] {
            let (a, b) = if zs > 0.0 { (26.0, 14.0) } else { (-26.0, -14.0) };
            let mut hit = None;
            for (p, q) in [(a, b), (b, a)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z - zs).abs() < 0.6 { hit = Some((z, mat)); break; }
                }
            }
            println!("  ({x:.1},{y:+.1}) z{zs:+.1}: {}", hit.map(|(z, m)| format!("z{z:+.2} m{m:02x}")).unwrap_or_else(|| "GAP".into()));
        }
    }
    println!("== the deck-level windows' exact y bands (0.2 steps, both dirs) ==");
    for x in [7.5f32, 8.0, 9.0, 10.0] {
        let mut band = Vec::new();
        let mut y = -3.0f32;
        while y <= 1.5 {
            let mut hitz = false;
            for (p, q) in [(-26.0f32, -14.0), (-14.0, -26.0)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z + 20.4).abs() < 0.6 { hitz = true; break; }
                }
            }
            if !hitz { band.push(format!("{y:+.1}")); }
            y += 0.2;
        }
        println!("  x{x:.1}: open y = {}", band.join(" "));
    }
    println!("== the west wall's z extent (east->west rays, y 0) ==");
    for z in [19.0f32, 20.0, 20.8, 21.5, 22.5, -19.0, -20.0, -20.8, -21.5, -22.5] {
        let hit = w.segment_surface_flags([5.5, 0.0, z], [3.0, 0.0, z], true);
        println!("  z{z:+.1}: {}", hit.map(|(t, n, m, _)| format!("x {:.2} m{m:02x}", 5.5 - t * 2.5)).unwrap_or_else(|| "NO wall".into()));
    }
}
