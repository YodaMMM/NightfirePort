//! level-14 probe: the crawl directions — real Bond, crouched, driven along each candidate duct
//! with the real player collision; prints the trajectory and where each direction wedges. Also
//! E-activates the lid SPs (the port's mask-bit-2 arm [M Player_Activate 0x1a5170:121-122 as
//! useobj.rs documents]) and watches the lid script's effects. Prints only.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn scene_yaw(scene: &nf_view::scene::Scene) -> f32 { scene.sim.as_ref().unwrap().bond.yaw }

fn wrap_angle(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}

fn crawl(scene: &mut nf_view::scene::Scene, dir: [f32; 2], ticks: u32, label: &str) {
    let mut traj: Vec<(u32, [f32; 3])> = Vec::new();
    let target_yaw = dir[0].atan2(dir[1]);
    let mut stuck_since: Option<u32> = None;
    for t in 0..ticks {
        let inp = Input {
            forward: 1.0,
            turn: wrap_angle(target_yaw - scene.sim.as_ref().unwrap().bond.yaw).clamp(-0.06, 0.06),
            ..Default::default()
        };
        scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
        let p = scene.sim.as_ref().unwrap().bond.pos;
        if t % 30 == 0 { traj.push((t, p)); }
        let moved = traj.last().map(|q| ((p[0]-q.1[0]).powi(2)+(p[2]-q.1[2]).powi(2)).sqrt() > 1e-3);
        stuck_since = match (moved, stuck_since) {
            (Some(false), None) if t > 60 => Some(t),
            (Some(true), _) => None,
            (s, k) => k.or(s.and(None).map(|_: ()| k.unwrap_or(0)).filter(|_| false)).or(k),
        };
        if let Some(s0) = stuck_since { if t - s0 >= 90 { traj.push((t, p)); break; } }
    }
    let pts: Vec<String> = traj.iter().map(|(t, p)| format!("t{t}({:.2},{:.2},{:.2})", p[0], p[1], p[2])).collect();
    eprintln!("{label}: {}", pts.join(" "));
}

#[test]
fn probe_crawl_directions() {
    let iso = iso();

    // 1) SOUTH (the known wedge).
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.crouched = true;
    }
    for _ in 0..20 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    crawl(&mut scene, [0.0, -1.0], 900, "SOUTH crawl");

    // 2) EAST from the room (z 63.66 the east duct line).
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    scene.sim.as_mut().unwrap().bond.crouched = true;
    for _ in 0..20 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    crawl(&mut scene, [1.0, 0.0], 500, "EAST crawl");

    // 3) NORTH.
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    scene.sim.as_mut().unwrap().bond.crouched = true;
    for _ in 0..20 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    crawl(&mut scene, [0.0, 1.0], 300, "NORTH crawl");

    // 4) E-activate the south-mouth lid SP (mask 0x4, the port's use arm) and watch its state.
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        let n = sim.drones.scripts.players.len();
        eprintln!("script players mounted: {n}");
        for (i, p) in sim.drones.scripts.players.iter().enumerate() {
            if p.scripts[0] == 0x0600_0a3b {
                eprintln!("  SP {i}: script 06000a3b state {} activated {} mask {:#x} placement {}", p.state, p.activated, p.mask, p.placement);
            }
        }
    }
    // Face the south mouth lid (40.80, 2.37, 63.66) from the start and press E.
    {
        let sim = scene.sim.as_mut().unwrap();
        let dx = 40.80 - sim.bond.pos[0];
        let dz = 63.66 - sim.bond.pos[2];
        sim.bond.yaw = dx.atan2(dz);
        sim.bond.pitch = 0.0;
    }
    for t in 0..200 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), t == 5, false, false);
        if t == 10 || t == 50 || t == 150 {
            let sim = scene.sim.as_ref().unwrap();
            let st: Vec<String> = sim.drones.scripts.players.iter().enumerate()
                .filter(|(_, p)| p.scripts[0] == 0x0600_0a3b)
                .map(|(i, p)| format!("SP{i}:st{} act{}", p.state, p.activated))
                .collect();
            eprintln!("t{t}: lid SPs [{}]", st.join(" "));
        }
    }
}
