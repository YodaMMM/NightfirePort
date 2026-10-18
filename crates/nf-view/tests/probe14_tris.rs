//! level-14 probe 5: the wedge triangles — the port world's static triangles inside the duct
//! wedge box (x 39.5..42.0, y 2.4..4.2, z 58.0..60.2), to identify the z≈59.3 blocker's shape.
//! Prints only.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_wedge_tris() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let tris = sim.world.tris_near([39.9, 2.4, 58.2], [41.8, 4.2, 60.0]);
    eprintln!("tris in the wedge box: {}", tris.len());
    for (i, t) in tris.iter().enumerate() {
        // Only triangles whose z extent crosses the 58.6..59.5 band and whose y reaches above 2.5
        // (the blocker's y range) — the floor/roof slabs are constant-z sheets and would flood.
        let zmin = t[0][2].min(t[1][2]).min(t[2][2]);
        let zmax = t[0][2].max(t[1][2]).max(t[2][2]);
        let ymax = t[0][1].max(t[1][1]).max(t[2][1]);
        let ymin = t[0][1].min(t[1][1]).min(t[2][1]);
        let xmin = t[0][0].min(t[1][0]).min(t[2][0]);
        let xmax = t[0][0].max(t[1][0]).max(t[2][0]);
        if zmin < 58.80 || zmax > 59.00 || xmin > 41.44 || xmax < 40.15 { continue; }
        eprintln!("t{i} z[{zmin:.2}..{zmax:.2}] y[{ymin:.2}..{ymax:.2}]: ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
}
