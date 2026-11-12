//! Scratch probe (level-0b-files-plate-hazard): the real-damage walks die at the files plate to
//! "world" damage — -2.40 every 7 frames, owner usize::MAX, from tick ~13308 at (-1.1, 47.9,
//! -3.4) entering the terminal room. Instrumented run on the walk's own exec-floor lane (the
//! duct route: the waiting room → the ring → the terminal room → the dome floor → the files
//! plate), every health drop logged with the sim's full source state (the GT turrets'
//! state/target/fire timers, the sensors' states, mines, the pending blast/beam queues,
//! explosions, the player_hits tail).
//!
//! Suspects named by the load dump (probe run 1): the exec floor's three GT wall guns #291/292/293
//! (p4 = 28 = ch 0x1C enable, p5 = 78 = 0x4E disable — OR #289: ch 0x1C ← [0x27, 0x28], the two
//! exec-floor cameras' out channels [D audit §GT/§Sensor]). The terminal-room pair #174/#175
//! (p4 = 14 = ch 0x0E ← OR #290 [0x0D, 0x06]) is the dome pair.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump_statics(w: &Walker, tag: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, t) in sim.drones.gt_turrets.iter().enumerate() {
        eprintln!("[{tag}] gt{i} placement {} pos ({:.2},{:.2},{:.2}) state {} facing {} p1 {} p4 {} p5 {} hp {}",
            t.placement, t.pos[0], t.pos[1], t.pos[2], t.state, t.facing, t.param1, t.param4, t.param5, t.hp);
    }
    for (i, s) in sim.drones.sensors.list.iter().enumerate() {
        eprintln!("[{tag}] sensor{i} placement {} pos ({:.2},{:.2},{:.2}) out {} disable {} range {:.1} keys {} state {}",
            s.placement, s.pos[0], s.pos[1], s.pos[2], s.out, s.disable, s.range, s.keys.len(), s.state);
    }
    for (i, m) in sim.drones.mines.iter().enumerate() {
        eprintln!("[{tag}] mine{i} placement {} pos ({:.2},{:.2},{:.2}) dmg {} r {}", m.placement, m.pos[0], m.pos[1], m.pos[2], m.damage, m.radius);
    }
    for (i, h) in sim.hurts.iter().enumerate() {
        if h.damage > 0.0 {
            eprintln!("[{tag}] hurt{i} placement {} box ({:.2},{:.2},{:.2})..({:.2},{:.2},{:.2}) dmg {} kind {} gate {} who {} one_shot {}",
                h.placement, h.lo[0], h.lo[1], h.lo[2], h.hi[0], h.hi[1], h.hi[2], h.damage, h.kind, h.gate, h.who, h.one_shot);
        }
    }
}

fn ch_line(sim: &nf_view::scene::Sim) -> String {
    format!("ch 0x1C {} 0x27 {} 0x28 {} 0x0E {} 0x0D {}",
        sim.drones.channels.on(0x1c), sim.drones.channels.on(0x27), sim.drones.channels.on(0x28),
        sim.drones.channels.on(0x0e), sim.drones.channels.on(0x0d))
}

fn log_drops(w: &Walker, tag: &str, t0: u32, drops: &mut u32) {
    let sim = w.scene.sim.as_ref().unwrap();
    let hits = sim.drones.player_hits.iter().rev().take(3).collect::<Vec<_>>();
    let n_hits = sim.drones.player_hits.len();
    eprintln!("[{tag}] DROP{} tick {} at ({:.2},{:.2},{:.2}) hp {:.2} {} hits tail {:?} (n {n_hits})",
        drops, w.total_ticks - t0, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.drones.health.health, ch_line(sim), hits);
    for (i, t) in sim.drones.gt_turrets.iter().enumerate() {
        eprintln!("[{tag}]   gt{i} placement {} state {} target {} fire_timer {:.2} flash {} hp {}", t.placement, t.state, t.target, t.fire_timer, t.flash, t.hp);
    }
    for (i, s) in sim.drones.sensors.list.iter().enumerate() {
        eprintln!("[{tag}]   sensor{i} placement {} state {} out {}", s.placement, s.state, s.out);
    }
    eprintln!("[{tag}]   pending_blasts {} beam {} explosions {} mines alive {}",
        sim.drones.pending_blasts.len(), sim.drones.pending_beam_blasts.len(), sim.drones.explosions.len(),
        sim.drones.mines.iter().filter(|m| !m.exploded).count());
    for e in sim.drones.explosions.iter().rev().take(3) {
        eprintln!("[{tag}]   expl ({:.1},{:.1},{:.1}) r {:.1} dmg {:.1} owner {} kind {}", e.0[0], e.0[1], e.0[2], e.1, e.2, e.3 as isize, e.4);
    }
    *drops += 1;
}

/// The walk's own exec-floor lane on real input: the ring's east corridor (n11-n13) → the
/// terminal room → the dome floor south → the files plate. Every drop logged.
#[test]
fn probe_plate_hazard_ring_lane() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000b.bin", false).expect("load 0b"));
    dump_statics(&w, "load");
    w.wait(30);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [10.21, 47.3, 8.20];
        sim.bond.vel = [0.0; 3];
    }
    let t0 = w.total_ticks;
    let mut drops = 0;
    let mut last_hp = w.scene.sim.as_ref().unwrap().drones.health.health;
    for (label, target) in [
        ("ring n12", [6.54f32, 47.3, 7.03]),
        ("ring n13", [2.18, 47.3, 7.53]),
        ("terminal room", [-1.0, 47.5, -5.5]),
        ("dome lane mid", [-1.1, 47.5, -9.0]),
        ("the files plate", [-1.15, 47.5, -12.6]),
    ] {
        eprintln!("[lane] leg {label} begin at tick {} ({})", w.total_ticks - t0, ch_line(w.scene.sim.as_ref().unwrap()));
        let start = w.total_ticks;
        while w.total_ticks - start < 900 {
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(p, target) < 1.2 { eprintln!("[lane] leg {label} arrived ({:.2},{:.2},{:.2}) {}", p[0], p[1], p[2], ch_line(w.scene.sim.as_ref().unwrap())); break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = nf_game::player::Input {
                turn: err.clamp(-0.08, 0.08),
                forward: if err.abs() > 0.6 { 0.0 } else { 0.6 },
                ..Default::default()
            };
            w.tick(input, false);
            let sim = w.scene.sim.as_ref().unwrap();
            if w.total_ticks % 120 == 0 {
                eprintln!("[lane] t {} at ({:.2},{:.2},{:.2}) {} sensors {:?}", w.total_ticks - t0, p[0], p[1], p[2], ch_line(sim),
                    sim.drones.sensors.list.iter().map(|s| (s.state, s.out)).collect::<Vec<_>>());
            }
            if sim.drones.health.health < last_hp - 0.01 {
                last_hp = sim.drones.health.health;
                log_drops(&w, "lane", t0, &mut drops);
                if drops > 18 { eprintln!("[lane] drop cap hit"); return; }
            }
        }
    }
    eprintln!("[lane] done: {drops} drops, hp {:.2} at {:?}", last_hp, w.scene.sim.as_ref().unwrap().bond.pos);
}
