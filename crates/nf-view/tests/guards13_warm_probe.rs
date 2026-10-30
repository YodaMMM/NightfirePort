//! level-13-guards-start-alerted-9 probe: the REAL path (07000012 walked → 0700004A spawn drop →
//! the RamSave carry → 07000013) and the player's first seconds of QUIET movement. Assertion-free
//! evidence dump: every lobby guard's state, alert meter, vision bit, sight latch per second, plus
//! the first state transition per guard. The pin lands in the real test once the trigger is known.
use nf_game::player::Input;

#[path = "common/tower2.rs"]
mod tower2;
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump(tag: &str, w: &Walker) {
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("--- {tag} (frame {}) ch28 {} ch2a {} ch4d {} ch4a {} npc_enabled {} ---",
        sim.drones.frame, sim.drones.channels.on(0x28) as u8, sim.drones.channels.on(0x2a) as u8,
        sim.drones.channels.on(0x4d) as u8, sim.drones.channels.on(0x4a) as u8, sim.drones.npc_enabled as u8);
    for (i, dr) in sim.drones.list.iter().enumerate() {
        if !dr.present { continue; }
        eprintln!(
            "guard[{i}] placement #{:>3} skin {:08x} state {:#04x} prev {:#04x} alert {:.3} vision_on {} sighted {} mode {:#x} dtype {} waitch {:#x} chon {} group {} pos ({:.1},{:.1},{:.1})",
            dr.placement, dr.skin, dr.state, dr.prev_state, dr.alert, dr.vision_on as u8, dr.sighted as u8,
            dr.mode, dr.dtype, dr.switch_channel, dr.channel_on as u8, dr.group, dr.pos[0], dr.pos[1], dr.pos[2]);
    }
}

#[test]
fn probe_warm_arrival_guard_states() {
    let iso = iso();
    // ---- 07000012 walked (the shared route — the real approach) ----
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
        tower2::walk_12(&mut w);
        tower2::walk_12_exit(&mut w);
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };

    // ---- 0700004A: the spawn drop onto the 0x4D touch ----
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save)).unwrap());
        w.wait_enabled(3000, "the 4A intro");
        let mut landed = false;
        for _ in 0..600 {
            w.tick(nf_game::player::Input::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { landed = true; break; }
        }
        assert!(landed, "4A: the start #43 drops Bond into touch #277 → ch 0x4D [D]");
        for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };

    // ---- 07000013: the lift arrival (the carry) ----
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000013.bin", false, Some(&save)).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    dump("13 arrival, control regained", &w);
    // The first seconds of QUIET movement: no fire, no use — Bond stands (and a slow walk-in,
    // the way a careful player moves). Log every 30 ticks; record the first transition per guard.
    let mut last: Vec<(usize, u8, f32)> = Vec::new(); // (index, state, alert)
    for t in 0..600u32 {
        w.tick(nf_game::player::Input::default(), false);
        if t % 30 == 29 {
            let sim = w.scene.sim.as_ref().unwrap();
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if !dr.present { continue; }
                let prev = last.iter().find(|(j, ..)| *j == i).map(|(_, s, _)| *s);
                if prev != Some(dr.state) || t == 29 {
                    eprintln!("[warm13] tick {} guard[{}] placement #{} state {:#04x} -> {:#04x} alert {:.3} vision_on {} sighted {}",
                        t + 1, i, dr.placement, prev.unwrap_or(0), dr.state, dr.alert, dr.vision_on as u8, dr.sighted as u8);
                }
            }
            last.clear();
            let sim = w.scene.sim.as_ref().unwrap();
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if dr.present { last.push((i, dr.state, dr.alert)); }
            }
        }
        if t == 149 || t == 299 || t == 599 { dump(&format!("quiet tick {t}"), &w); }
    }
    dump("13 after 600 quiet ticks", &w);
    // Then the slow walk-in through the lobby the way the player moves (no shots).
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13 probe: the lobby east lane");
    dump("13 after the first walk leg", &w);
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13 probe: the lobby west mid");
    dump("13 after the second walk leg", &w);
}
