//! Scratch probe (real-health-walks-0b-0c-0d-03): what hurts Bond in the exec-floor corridor
//! (x −1.0..−1.4, z −5.8..+5.3, the real-health walk's 42 world hits)? Teleport into the stretch
//! and step: print health drops, the sim's explosion records, the fusebox/gt state.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_0b_corridor_damage() {
    let w = Walker::new(nf_view::load(&iso(), "0700000b.bin", false).expect("load 0b"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[p] hurts {} fuseboxes {} gt {} drones {}", sim.hurts.len(), sim.drones.fuseboxes.len(), sim.drones.gt_turrets.len(), sim.drones.list.len());
        for (i, f) in sim.drones.fuseboxes.iter().enumerate() { eprintln!("[p] fusebox {i} at {:?}", f.pos); }
        for (i, t) in sim.drones.gt_turrets.iter().enumerate() { eprintln!("[p] gt {i} at {:?}", t.pos); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let hurt_boxes: Vec<([f32; 3], [f32; 3])> = sim.hurts.iter().map(|h| (h.lo, h.hi)).collect();
    drop(w);
    let mut w = Walker::new(nf_view::load(&iso(), "0700000b.bin", false).expect("load 0b"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-1.0, 47.3, -6.0];
        sim.bond.vel = [0.0; 3];
    }
    for t in 0..600u32 {
        let sim = w.scene.sim.as_mut().unwrap();
        let h0 = sim.drones.health.health;
        let p0 = sim.bond.pos;
        sim.step_bond(Input::default(), false, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.health.health < h0 - 0.01 {
            eprintln!("[p] t {t} -{:.2} at ({:.1},{:.1},{:.1}) hits {:?}",
                h0 - sim.drones.health.health, p0[0], p0[1], p0[2],
                sim.drones.player_hits.iter().rev().take(2).map(|h| (h.0, h.1, h.2)).collect::<Vec<_>>());
            for e in sim.drones.explosions.iter().rev().take(4) {
                eprintln!("[p]   expl ({:.1},{:.1},{:.1}) r {:.1} dmg {:.1} owner {} kind {}",
                    e.0[0], e.0[1], e.0[2], e.1, e.2, e.3 as isize, e.4);
            }
        }
        if t == 60 {
            eprintln!("[p] hurt boxes at load: {hurt_boxes:?}");
        }
        if sim.drones.health.health <= 0.0 { break; }
    }
}
