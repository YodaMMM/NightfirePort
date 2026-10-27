// level-14-full-walk scratch 7b: every ladder on 07000014. Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let lads: Vec<([f32; 3], f32, f32)> = sim.ladders.iter()
        .map(|l| (l.centre, l.top(), l.bottom())).collect();
    println!("== ladders: {} ==", lads.len());
    for (p, top, bot) in &lads {
        println!("  ladder at ({:.2}, {:.2}, {:.2}) top {:.2} bottom {:.2}", p[0], p[1], p[2], top, bot);
    }
}
