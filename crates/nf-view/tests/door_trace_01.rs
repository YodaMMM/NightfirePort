//! level-14-cutscene-fall-2 diagnostic: traces Bond's position and door 0's state through the
//! intro and the walk to the door on 07000001, to localise what the camera-start base reset
//! changes in the walk. Opt-in via NF_ISO. Run with --nocapture --test-threads=1.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").unwrap_or_else(|_| { eprintln!("NF_ISO not set"); std::process::exit(0) }) }

#[test]
fn trace_door_walk() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if t % 500 == 0 {
            let s = w.scene.sim.as_ref().unwrap();
            eprintln!("[intro t{t}] bond ({:.3},{:.3},{:.3}) door0 {}",
                s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.drones.doors.list[0].state);
        }
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { eprintln!("[intro] ch 0xB at t{t}"); break; }
    }
    w.wait_enabled(6000, "intro tail");
    let dpos = w.scene.sim.as_ref().unwrap().drones.doors.list[0].pos;
    eprintln!("[door0] pos ({:.3},{:.3},{:.3})", dpos[0], dpos[1], dpos[2]);
    // the sister's walk, leg by leg (the walker's own stepping), traced around each leg
    let target = [dpos[0], dpos[1], dpos[2] + 1.6];
    let pts = walker::leg(w.scene.sim.as_ref().unwrap(), w.scene.sim.as_ref().unwrap().bond.pos, target);
    eprintln!("[nav] {} waypoints: {:?}", pts.len(), pts.iter().map(|p| (p[0], p[1], p[2])).collect::<Vec<_>>());
    for (k, p) in pts.iter().enumerate().skip(1) {
        let before = { let s = w.scene.sim.as_ref().unwrap(); (s.bond.pos, s.drones.doors.list[0].state) };
        let ticks = w.walk_to(walker::Wp::new(p[0], p[1], p[2]).r(1.3), &format!("leg{k}"));
        let after = { let s = w.scene.sim.as_ref().unwrap(); (s.bond.pos, s.drones.doors.list[0].state) };
        eprintln!("[leg{k}] +{ticks} ticks bond {:?} -> {:?} door0 {} -> {}", before.0, after.0, before.1, after.1);
    }
    let ticks = w.walk_to(walker::Wp { at: target, act: walker::Act::None, arrive: 1.6 }, "final");
    {
        let s = w.scene.sim.as_ref().unwrap();
        eprintln!("[final] +{ticks} bond ({:.3},{:.3},{:.3}) door0 {}", s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.drones.doors.list[0].state);
    }
    // the sister's 90 idle ticks, traced
    for t in 0..90 {
        w.tick(Default::default(), false);
        let s = w.scene.sim.as_ref().unwrap();
        if s.drones.doors.list[0].state != 2 || t % 30 == 0 {
            eprintln!("[idle t{t}] bond ({:.4},{:.4},{:.4}) door0 {} pos01 {:.4}",
                s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.drones.doors.list[0].state,
                s.drones.doors.list[0].pos01);
        }
    }
    let s = w.scene.sim.as_ref().unwrap();
    eprintln!("[after 90] bond ({:.4},{:.4},{:.4}) door0 {}", s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.drones.doors.list[0].state);
}
