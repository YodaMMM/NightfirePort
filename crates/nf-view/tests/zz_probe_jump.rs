//! Probe (uncommitted): the standing-jump rise on 0700000a's west hall, to compare with the live
//! game's 0.7513 (apex ~23 frames) at (-15, 1.12, 16.3).
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore]
fn probe_jump_rise() {
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    scene.sim.as_mut().unwrap().bond.pos = [-15.0, 1.2, 16.3];
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    let base = scene.sim.as_ref().unwrap().bond.pos[1];
    let mut ys = Vec::new();
    for t in 0..90 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { jump: t == 0, ..Default::default() }, false, false, false);
        ys.push(sim.bond.pos[1] - base);
    }
    let (i, m) = ys.iter().enumerate().fold((0, f32::MIN), |a, (i, &v)| if v > a.1 { (i, v) } else { a });
    eprintln!("port: base {base:.4} rise {m:.4} ticks-to-apex {i}");
    eprintln!("trace {}", ys.iter().take(50).map(|v| format!("{v:.3}")).collect::<Vec<_>>().join(" "));
}
