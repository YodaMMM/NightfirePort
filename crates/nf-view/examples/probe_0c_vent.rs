//! Scratch (level-0c-display-hatch): the collision tris around the display vent (0700000c).
//! Regions: the duct (x 2.5..9, y 12.5..19.5, z −114.5..−108), the crates (x 1.5..7.5, y 10.5..17.5, z −117.5..−112.5),
//! the case interior (x 1.5..9.5, y 5.5..15.5, z −115.5..−110.5).
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let scene = nf_view::load(&path, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_ref().unwrap();
    let regions: [(&str, [f32; 3], [f32; 3]); 3] = [
        ("DUCT", [2.5, 12.5, -114.5], [9.0, 19.5, -108.0]),
        ("CRATES", [1.5, 10.5, -117.5], [7.5, 17.5, -112.5]),
        ("CASE", [1.5, 5.5, -115.5], [9.5, 15.5, -110.5]),
    ];
    for (name, lo, hi) in regions {
        println!("== {name} box ({}, {}, {})..({}, {}, {})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        let mut tris = sim.world.tris_near(lo, hi);
        tris.sort_by(|a, b| a[0][1].partial_cmp(&b[0][1]).unwrap());
        for t in tris {
            let (a, b, c) = (t[0], t[1], t[2]);
            let (e1, e2) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
            let cc = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0, (a[2] + b[2] + c[2]) / 3.0];
            println!("tri ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) ny {:.2} c ({:.2},{:.2},{:.2})",
                a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2], n[1] / len, cc[0], cc[1], cc[2]);
        }
    }
}
