//! The crawl #35 leg probe: from the ledge walk's end, the E press at the prompt #85 zone
//! (the crawl "Wall Hug Window" #35 at (−34.10, 37.28, −31.65), band z −26.3..−37.0, prompts
//! #84/#85 at z −37.3/−25.5 [D placements]), the forward crawl south, the exit, then the
//! roofed corridor legs (x −36, z −38..−44 [D the band maps]). Scratch probe.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn main() {
    let iso_path = iso();
    let scene = &mut nf_view::load(&iso_path, "07000005.bin", false).expect("load 07000005");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    // Fast-forward: place Bond where the ledge probe ended (probe-grade teleport, not a test leg).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-34.78, 37.109116, -24.845];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::PI;
    }
    // Leg: the prompt 85 zone — walk to (−34.3, −25.4) and read the icon ctx each tick.
    let mut armed = false;
    for t in 0..400u32 {
        let s = scene.sim.as_ref().unwrap();
        let p = s.bond.pos;
        if s.bond.icon_ctx != 0xFF && s.bond.icon_ctx != 0 { armed = true; eprintln!("[t{t}] ctx {:#x} at {:?}", s.bond.icon_ctx, p); break; }
        let want = (-25.4f32 - p[2]).atan2(-34.3 - p[0]);
        let mut err = want - s.bond.yaw;
        if err > std::f32::consts::PI { err -= 2.0 * std::f32::consts::PI; }
        if err < -std::f32::consts::PI { err += 2.0 * std::f32::consts::PI; }
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: if ((p[0] + 34.3).powi(2) + (p[2] + 25.4).powi(2)).sqrt() < 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    let s = scene.sim.as_ref().unwrap();
    eprintln!("pre-press: pos {:?} ctx {:#x} sub {:#x} armed {armed}", s.bond.pos, s.bond.icon_ctx, s.bond.substate);
    // The E press (edge per 6 ticks like the walker's Use).
    let mut crawling = false;
    for t in 0..120u32 {
        let input = Input { jump: t % 6 == 0, ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, t % 6 == 0);
        let _ = input;
        let s = scene.sim.as_ref().unwrap();
        if s.bond.substate == nf_game::player::SS_CREEP { crawling = true; eprintln!("[t{t}] CRAWL entered at {:?}", s.bond.pos); break; }
    }
    let s = scene.sim.as_ref().unwrap();
    eprintln!("after press: pos {:?} ctx {:#x} sub {:#x} crawling {crawling}", s.bond.pos, s.bond.icon_ctx, s.bond.substate);
    if crawling {
        // Forward-crawl south (the #299 precedent: forward pushes toward the wall's A end; yaw 1.57's axis points −z).
        let crawl_tick = 0u32;
        let mut exited = false;
        for t in 0..2400u32 {
            scene.sim.as_mut().unwrap().step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
            let s = scene.sim.as_ref().unwrap();
            if t % 100 == 0 { eprintln!("[crawl t{t}] pos {:?} sub {:#x}", s.bond.pos, s.bond.substate); }
            if s.bond.substate != nf_game::player::SS_CREEP && t > 10 { exited = true; eprintln!("[t{t}] CRAWL EXITED at {:?}", s.bond.pos); break; }
        }
        let s = scene.sim.as_ref().unwrap();
        eprintln!("after crawl: pos {:?} sub {:#x} exited {exited}", s.bond.pos, s.bond.substate);
        // The corridor legs south (x −36, z −38..−44, floors 32.5-36.2 under the 46.0 roof [D]).
        for leg in 0..3 {
            let target = [-35.7f32, -38.6 - 2.7 * leg as f32];
            let mut done = false;
            for t in 0..600u32 {
                let p = { scene.sim.as_ref().unwrap().bond.pos };
                if (p[0] - target[0]).powi(2) + (p[2] - target[1]).powi(2) < 0.5 { done = true; break; }
                let (want_base, yaw) = { let s = scene.sim.as_ref().unwrap(); ((target[1] - p[2]).atan2(target[0] - p[0]), s.bond.yaw) };
                let mut err = want_base - yaw;
                if err > std::f32::consts::PI { err -= 2.0 * std::f32::consts::PI; }
                if err < -std::f32::consts::PI { err += 2.0 * std::f32::consts::PI; }
                let input = Input { turn: err.clamp(-0.07, 0.07), forward: 1.0, ..Default::default() };
                scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
                if t % 100 == 0 { eprintln!("[corridor {leg}] pos {:?}", scene.sim.as_ref().unwrap().bond.pos); }
                if scene.sim.as_ref().unwrap().bond.pos[1] < 25.0 { eprintln!("[corridor {leg}] FELL {:?}", scene.sim.as_ref().unwrap().bond.pos); break; }
            }
            let s = scene.sim.as_ref().unwrap();
            eprintln!("[corridor leg {leg}] done {done} at {:?}", s.bond.pos);
            if !done { break; }
        }
        let s = scene.sim.as_ref().unwrap();
        println!("FINAL: pos {:?} feet {:.2} sub {:#x}", s.bond.pos, s.bond.pos[1] - 1.0847, s.bond.substate);
    } else {
        println!("NO CRAWL: ctx {:#x} at {:?}", s.bond.icon_ctx, s.bond.pos);
    }
}
