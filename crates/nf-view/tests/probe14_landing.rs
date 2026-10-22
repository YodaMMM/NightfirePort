//! level-14 probe (level-14-complete-2): the chute descent with the ray-gated grounding —
//! crawl to the lip, then NEUTRAL (no forward): does Bond fall straight down the shaft and
//! land on the facility floor at −2.4 (root ≈ −1.75)? Prints only.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_neutral_landing() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.crouched = true;
        for _ in 0..15 { sim.step_bond(Input::default(), false, false, false); }
        // Crawl south through the grill to the lip — STOP at z=56.45 (the wall push's
        // equilibrium is z≈56.27; past it the crawl goes over the lip).
        for _ in 0..340 {
            if sim.bond.pos[2] <= 56.45 { break; }
            let inp = Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
        }
        eprintln!("at the lip: {:?}", sim.bond.pos);
        // NEUTRAL fall down the shaft.
        let mut traj = Vec::new();
        for t in 0..400 {
            sim.step_bond(Input::default(), false, false, false);
            if t % 30 == 0 { traj.push(sim.bond.pos); }
        }
        traj.push(sim.bond.pos);
        let pts: Vec<String> = traj.iter().map(|p| format!("({:.2},{:.2},{:.2})", p[0], p[1], p[2])).collect();
        eprintln!("neutral fall: {}", pts.join(" "));
        let p = sim.bond.pos;
        eprintln!("final: grounded {} y {:.2} z {:.2}", sim.bond.grounded, p[1], p[2]);
        // Then south along the facility floor.
        let mut traj2 = Vec::new();
        for t in 0..600 {
            let inp = Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
            if t % 60 == 0 { traj2.push(sim.bond.pos); }
        }
        traj2.push(sim.bond.pos);
        let pts: Vec<String> = traj2.iter().map(|p| format!("({:.2},{:.2},{:.2})", p[0], p[1], p[2])).collect();
        eprintln!("south walk: {}", pts.join(" "));
    }
}
