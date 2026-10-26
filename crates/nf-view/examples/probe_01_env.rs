//! level-0a-rain-indoors-2 probe: does 07000001 build drops (rain) in the port?
//! The game's 07000001 snapshot says DropType=1 (rain), CurrentMax=0x40 [L ee_start_07000001.bin
//! 0x30d788/0x30d784] — the port must carry the same env so its shared-stream consumption has the
//! game's rain shape (3 FRands per AddDrop, ≤21 respawns/frame [M 0x18c278/0x18c580]).
use nf_game::player::Input;
use nf_view::load;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = load(&iso, "07000001.bin", false).expect("load 07000001");
    for t in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        match sim.drones.env.as_ref() {
            Some(env) if t % 10 == 0 => {
                let alive = env.alive.iter().filter(|&&a| a).count();
                println!("t={t}: kind={} fall={} len={} boxes={} alive={alive} centre={:?}",
                    env.kind, env.fall, env.drops.len(), env.boxes.len(), env.centre);
            }
            None if t % 10 == 0 => println!("t={t}: env NONE"),
            _ => {}
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    match sim.drones.env.as_ref() {
        Some(env) => println!("final: env kind={} alive={}", env.kind, env.alive.iter().filter(|&&a| a).count()),
        None => println!("final: env NONE — the port sees NO weather on 07000001"),
    }
}
