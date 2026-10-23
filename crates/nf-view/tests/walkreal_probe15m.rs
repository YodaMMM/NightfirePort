//! probe: the keypad door #685's leaf sphere and closed tris.
#[path = "common/walker.rs"]
mod walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
#[test]
#[ignore = "dump"]
fn probe_door685() {
    let scene = nf_view::load(&iso(), "07000015.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if (45.0..47.5).contains(&d.pos[0]) && (69.0..71.5).contains(&d.pos[2]) && d.pos[1] > 40.0 {
            let (sc, sr) = d.sph;
            eprintln!("door[{i}] at=({:.2},{:.2},{:.2}) sph=({:.2},{:.2},{:.2}) r={:.3} state={:#x} pos01={:.2}",
                d.pos[0], d.pos[1], d.pos[2], sc[0], sc[1], sc[2], sr, d.state, d.pos01);
        }
    }
    for t in sim.world.tris_near([44.0, 40.0, 71.0], [48.0, 43.5, 74.0]) {
        eprintln!("tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
}
