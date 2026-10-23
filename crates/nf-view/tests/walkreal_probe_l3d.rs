//! level-4a-lift-3 probe d: the post-handback ride/fall trace — y, health, grounded per 50 ticks.
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "diagnostic; --ignored --nocapture"]
fn trace_post_handback() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    for k in 0..84 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
    }
    w.skip_cutscenes = false;
    let mut disabled_seen = false;
    for _ in 0..4200u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.disabled { disabled_seen = true; }
        if disabled_seen && !sim.disabled && sim.bond.pos[1] > -60.0 { break; }
    }
    eprintln!("handback at tick {} pos {:?} health {} grounded {}", w.total_ticks,
        w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.health.health,
        w.scene.sim.as_ref().unwrap().bond.grounded);
    for k in 0..2400u32 {
        w.tick(Default::default(), false);
        if k % 50 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("k{k} pos {:?} hp {} gnd {} ch62 {}", sim.bond.pos, sim.drones.health.health, sim.bond.grounded,
                sim.drones.channels.on(0x62));
        }
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { eprintln!("DEAD at k{k}"); break; }
    }
}
