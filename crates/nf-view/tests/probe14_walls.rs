//! level-14 probe: the junction walls — every static triangle whose centroid sits in the box
//! x 39.5..46.5, y 0..5, z 54..68, printed raw so the duct/room topology can be read off the
//! coordinates. Prints only.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_junction_walls() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    let tris = sim.world.tris_near([39.5, 0.0, 54.0], [46.5, 5.0, 68.0]);
    eprintln!("tris in the junction box: {}", tris.len());
    let mut k = 0;
    for t in &tris {
        let cx = (t[0][0] + t[1][0] + t[2][0]) / 3.0;
        let cy = (t[0][1] + t[1][1] + t[2][1]) / 3.0;
        let cz = (t[0][2] + t[1][2] + t[2][2]) / 3.0;
        k += 1;
        eprintln!("w{k:03} c({cx:6.2},{cy:5.2},{cz:6.2}) | ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
}
