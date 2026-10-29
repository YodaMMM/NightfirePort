// level-14-full-walk scratch 3: the game's own nav web around the vent room / stairwell /
// mezzanine — where does the AI's graph connect the lower levels to the vent lanes (2.40)
// and the mezzanine? Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);

    for (pi, path) in sim.nav.paths.iter().enumerate() {
        let inband: Vec<(usize, [f32; 3])> = path.nodes.iter().enumerate()
            .filter(|(_, n)| n.pos[0] > 36.0 && n.pos[0] < 52.0 && n.pos[2] > 34.0 && n.pos[2] < 70.0)
            .map(|(i, n)| (i, n.pos)).collect();
        if inband.is_empty() { continue; }
        println!("== path {pi}: {} nodes in the vent band ==", inband.len());
        for (i, p) in &inband {
            println!("  node {i}: ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);
        }
        // links between in-band nodes
        let idxs: Vec<usize> = inband.iter().map(|(i, _)| *i).collect();
        for (li, link) in path.links.iter().enumerate() {
            let a = link.a as usize;
            let b = link.b as usize;
            if idxs.contains(&a) && idxs.contains(&b) {
                let pa = path.nodes[a].pos;
                let pb = path.nodes[b].pos;
                println!("  link {li}: {a} -> {b}  dy {:.2}  d {:.2}", pb[1] - pa[1],
                    ((pa[0] - pb[0]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt());
            }
        }
    }
}
