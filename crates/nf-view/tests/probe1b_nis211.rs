//! level-1b-drake-fight probe: drive SP#211's gate (ch 30 — console-1's UI-script write [D
//! 0x060007e2 stream 1]) and watch the fly-by NIS (0x060007fd → 0x060008f0) play out: Bond's
//! position/substate each 30 ticks, the chain channels (30/46/75/10/15), and the drained
//! player_events (PlayerDisable/PlayerEnable matrices). Assertion-free.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn drive_sp211_gate_and_watch_the_fly_by() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.scene.sim.as_mut().unwrap().bond.substate = nf_game::player::SS_WALK;
    w.wait(120);
    // Walk to the slit mouth (the mapped deck route — real input).
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        println!("[start] bond pos {:?} substate {}", sim.bond.pos, sim.bond.substate);
    }
    // The disc's own activation: console-1 writes ch 30 (SP#211's gate) [D]; the missile-disarm
    // chain writes ch 46 = SP#211's out, whose mask bit 0 IS the start trigger [M 0x1dbda8 case 0].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1e, 1, sim.drones.frame);
        sim.drones.channels.set(0x2e, 1, sim.drones.frame);
    }
    w.skip_cutscenes = false;
    for t in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t % 150 == 0 {
            let ch: Vec<String> = [0x1eu32, 0x2e, 0x4b, 0x4a, 0x0a, 0x0f]
                .iter().map(|c| format!("{c:#x}={}", sim.drones.channels.on(*c) as u8)).collect();
            println!("[t {t}] pos {:?} sub {} enabled {} ch {}",
                sim.bond.pos, sim.bond.substate, !sim.disabled, ch.join(" "));
        }
        for ev in &sim.drones.player_events {
            println!("[t {t}] event {ev:?}");
        }
        if sim.drones.mission.status() == 6 { println!("[t {t}] MISSION ENDED"); break; }
    }
}
