//! level-14 probe: what does the level's own nav web think connects the vent area to the
//! facility? Print the nav route the AI would take. Prints only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::nav_route;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_nav_web() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    for (label, from, to) in [
        ("vent start -> mezzanine", [40.8f32, 2.9, 65.0], [40.7, 2.6, 39.0]),
        ("vent start -> facility mid", [40.8, 2.9, 65.0], [40.7, 0.9, 30.0]),
        ("east duct -> facility", [45.3, 2.9, 65.0], [45.0, 0.9, 30.0]),
    ] {
        match nav_route(sim, from, to) {
            Some(r) => {
                let pts: Vec<String> = r.iter().map(|p| format!("({:.1},{:.1},{:.1})", p[0], p[1], p[2])).collect();
                eprintln!("{label}: {} hops [{}]", r.len(), pts.join(" "));
            }
            None => eprintln!("{label}: NO NAV ROUTE"),
        }
    }
}
