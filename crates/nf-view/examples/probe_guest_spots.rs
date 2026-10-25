//! One-shot: named render spots — the first present type-13/14 guests' positions near Bond's
//! free spawn (the party crowd), for the report's --bond list. Opt-in: NF_ISO.
use nf_game::player::Input;
fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    for _ in 0..3000 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    println!("bond free pos {:?} yaw {:.3}", sim.bond.pos, sim.bond.yaw);
    let mut n = 0;
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.present && d.enabled && !d.corpse && (d.voice_type == 13 || d.voice_type == 14)
            && d.health > 0.0 && d.skin & 0xffff != 0x22 {
            let yaw = ((sim.bond.pos[0] - d.pos[0]).atan2(sim.bond.pos[2] - d.pos[2]));
            println!("guest d{i} vt {} skin {:08x} pos {:.2?} bond-view yaw {:.2}", d.voice_type, d.skin, d.pos, yaw);
            n += 1;
            if n >= 5 { break; }
        }
    }
}
