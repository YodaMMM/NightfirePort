//! level-13-guards-start-alerted probe: fresh 07000013 load — every lobby guard's AI state and
//! alert meter at tick 0 and over the first seconds, Bond hidden at the spawn. Assertion-free
//! probe (the pin lands in the real test once the game's rule is confirmed).
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_guard_start_states() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000013.bin", false).expect("load 07000013");
    let sim = scene.sim.as_mut().unwrap();
    let dump = |tag: &str, sim: &nf_view::scene::Sim| {
        eprintln!("--- {tag} (frame {}) ---", sim.drones.frame);
        for (i, dr) in sim.drones.list.iter().enumerate() {
            if !dr.present { continue; }
            eprintln!(
                "guard[{i}] placement #{:>3} skin {:08x} state {:#04x} prev {:#04x} alert {:.3} mode {:#x} dtype {} waitch {:#x} chon {} sighted {} group {}",
                dr.placement, dr.skin, dr.state, dr.prev_state, dr.alert, dr.mode, dr.dtype,
                dr.switch_channel, dr.channel_on as u8, dr.sighted as u8, dr.group
            );
        }
    };
    // tick 0: one step so the first ENTER is delivered, then dump.
    sim.step_bond(Input::default(), false, false, false);
    dump("tick 1", sim);
    for t in [30u32, 120, 360, 600] {
        while sim.drones.frame < t { sim.step_bond(Input::default(), false, false, false); }
        dump(&format!("tick {t}"), sim);
    }
    eprintln!("npc_enabled at end: {}", sim.drones.npc_enabled);
    // channel census: is the wait channel 0x28 on (the spawn touch)?
    eprintln!("ch 0x28 on: {}", sim.drones.channels.on(0x28));
    eprintln!("ch 0x2a on: {}", sim.drones.channels.on(0x2a));
    eprintln!("bond at {:?}", sim.bond.pos);
}

#[test]
fn probe_first_sighting() {
    // After the settle, walk Bond into the lobby open and watch the first guard reaction.
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000013.bin", false).expect("load 07000013");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    eprintln!("after settle: frame {}", sim.drones.frame);
    for (i, dr) in sim.drones.list.iter().enumerate() {
        if dr.present { eprintln!("guard[{i}] placement #{:>3} state {:#04x} alert {:.3} sighted {}", dr.placement, dr.state, dr.alert, dr.sighted as u8); }
    }
    // Track state changes and sight flips with positions.
    let mut last: Vec<(u8, bool)> = Vec::new();
    for t in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if t % 30 == 0 { eprintln!("t={t} npc_enabled {} disabled {}", sim.drones.npc_enabled, sim.disabled); }
        let cur: Vec<(u8, bool)> = sim.drones.list.iter().map(|d| (d.state, d.sighted)).collect();
        if cur != last {
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if dr.present && (last.get(i) != Some(&cur[i])) {
                    let dx = dr.pos[0] - sim.bond.pos[0]; let dy = dr.pos[1] - sim.bond.pos[1]; let dz = dr.pos[2] - sim.bond.pos[2];
                    let dist = (dx*dx + dy*dy + dz*dz).sqrt();
                    eprintln!("t={t} guard[{i}] #{:>3} -> {:#04x} sighted {} @({:.2},{:.2},{:.2}) yaw {:.2} toBond {:.2} dist {:.2} range {:.1} los {} vis {:.2}",
                        dr.placement, dr.state, dr.sighted as u8, dr.pos[0], dr.pos[1], dr.pos[2], dr.yaw, dr.yaw_to_opp, dist, dr.sight_range, dr.los as u8, dr.visibility);
                }
            }
            last = cur;
        }
    }
    for (i, dr) in sim.drones.list.iter().enumerate() {
        if dr.present { eprintln!("final guard[{i}] placement #{:>3} state {:#04x} alert {:.3} sighted {}", dr.placement, dr.state, dr.alert, dr.sighted as u8); }
    }
}
