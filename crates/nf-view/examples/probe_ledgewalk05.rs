//! The window-ledge lane probe: the approach's west end (x −35..−34, z −19.5, floors 35.9-36.1)
//! onto the window ledge (36.0-36.43, z −19..−25.5) toward crawl prompt #85 (z −25.5) — the lane
//! EAST probes never tried (they walked x −23.7..−32 into the band's north seal; the ledge lane is
//! west of the seal's piers). Raw full-forward real input; a jump pulse if a lip stalls the walk.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn run(scene: &mut nf_view::scene::Scene, start: [f32; 3], aims: &[([f32; 2], &str)], label: &str, jump: bool) {
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = start;
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::PI;
        sim.bond.pitch = 0.0;
    }
    let mut leg = 0usize;
    let mut stall = 0u32;
    let mut last = [start[0], start[2]];
    for t in 0..900u32 {
        if leg >= aims.len() {
            eprintln!("[{label}] all legs done at tick {t}");
            break;
        }
        let (aim, name) = aims[leg];
        let (p, yaw, g) = { let s = scene.sim.as_ref().unwrap(); (s.bond.pos, s.bond.yaw, s.bond.grounded) };
        let dxz = ((p[0] - aim[0]).powi(2) + (p[2] - aim[1]).powi(2)).sqrt();
        if dxz < 0.45 {
            eprintln!("[{label}] leg '{name}' done at tick {t} pos {:?}", p);
            leg += 1;
            stall = 0;
            continue;
        }
        let want = (aim[0] - p[0]).atan2(aim[1] - p[2]);
        let mut err = want - yaw;
        if err > std::f32::consts::PI { err -= 2.0 * std::f32::consts::PI; }
        if err < -std::f32::consts::PI { err += 2.0 * std::f32::consts::PI; }
        let jp = jump && t % 6 == 0;
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: 1.0, jump: jp, ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if t % 15 == 0 {
            eprintln!("[{label} t{t} leg {name}] pos {:?} feet {:.2} g {} ny {:.2} bump {}",
                s.bond.pos, s.bond.pos[1] - 1.0847, s.bond.grounded, s.bond.floor_ny, s.bond.head_bump);
        }
        if (s.bond.pos[0] - last[0]).abs() + (s.bond.pos[2] - last[1]).abs() < 1e-3 { stall += 1; } else { stall = 0; }
        last = [s.bond.pos[0], s.bond.pos[2]];
        if stall > 240 { eprintln!("[{label}] STALL on leg '{name}' at {:?}", s.bond.pos); break; }
        if s.bond.pos[1] < 25.0 { eprintln!("[{label}] FELL at {:?}", s.bond.pos); break; }
    }
    let s = scene.sim.as_ref().unwrap();
    println!("{label}: end pos {:?} feet {:.2} legs {}/{} g {} ny {:.2}",
        s.bond.pos, s.bond.pos[1] - 1.0847, leg, aims.len(), s.bond.grounded, s.bond.floor_ny);
}

fn main() {
    let iso_path = iso();
    let scene = &mut nf_view::load(&iso_path, "07000005.bin", false).expect("load 07000005");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    // The ledge lane: the approach west end → the ledge's lip (z −20) → south to the prompt #85 zone.
    let aims = [
        ([-35.1f32, -19.6], "the approach west end"),
        ([-35.2, -20.6], "the lip"),
        ([-35.3, -22.2], "the ledge south 1"),
        ([-35.3, -24.0], "the ledge south 2"),
        ([-34.6, -25.2], "the prompt 85 approach"),
    ];
    run(scene, [-34.6, 36.0 + 1.0847, -18.6], &aims, "E ledge walk", false);
    let scene2 = &mut nf_view::load(&iso_path, "07000005.bin", false).expect("reload");
    for _ in 0..30 { scene2.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    run(scene2, [-34.6, 36.0 + 1.0847, -18.6], &aims, "F ledge walk+jump", true);
}
