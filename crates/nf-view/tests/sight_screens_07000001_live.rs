//! sight-screens-01: the three guards beyond the paper screens on 07000001 must not see Bond
//! standing at (11.43, 6.47, -3.51).
//!
//! The game's result [L audits live-1108 §6, PCSX2: fresh load of 07000001, intro skipped with
//! the skip button, Bond placed at (11.43, 6.47, -3.51) and held 600 frames, each drone's state
//! (obj+0xF4) polled every frame]: the guards at (6.95, 6.34, 2.20), (6.98, 6.34, 8.48) and
//! (5.42, 6.34, 14.44) stayed in state 0x01 (`NDrone2_DSTATE_WaitSwitch`) for all 600 frames.
//!
//! The port's run: the same load, the intro skipped through the game's skip request, the same
//! placement (`Bond::stand_at`, the game's `Player_StandAtNewPosition`), 600 ticks standing
//! still, each of the three guards' state recorded every tick.

use nf_game::player::Bond;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const BOND_AT: [f32; 3] = [11.43, 6.47, -3.51];
const GUARDS_AT: [[f32; 3]; 3] = [[6.95, 6.34, 2.20], [6.98, 6.34, 8.48], [5.42, 6.34, 14.44]];

#[test]
fn guards_beyond_the_screens_stay_in_wait_switch_for_600_ticks() {
    let mut scene = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    // The level-start intro (script 0x06000087) skipped the game's way: the skip request each tick
    // until the player is back in control.
    let intro = 0x0600_0087u32;
    let mut skipped = false;
    for t in 0..1200u32 {
        let sim = scene.sim.as_mut().unwrap();
        let pending = sim.drones.scripts.players.iter().any(|p| p.scripts[0] == intro
            && (matches!(p.state, 1 | 2 | 7) || p.info[p.cur].as_ref().is_some_and(|s| s.play & 1 != 0)));
        if t >= 2 && !pending && !sim.disabled { skipped = true; break; }
        sim.drones.scripts.skip_request = true;
        sim.step_bond(Default::default(), false, false, false);
    }
    assert!(skipped, "the intro never handed control back");

    // The three guards by position (the nearest present drone to each PCSX2 position).
    let idx: Vec<usize> = {
        let sim = scene.sim.as_ref().unwrap();
        GUARDS_AT.iter().map(|g| {
            let (i, d) = sim.drones.list.iter().enumerate().filter(|(_, d)| d.present)
                .map(|(i, d)| (i, ((d.pos[0] - g[0]).powi(2) + (d.pos[1] - g[1]).powi(2) + (d.pos[2] - g[2]).powi(2)).sqrt()))
                .min_by(|a, b| a.1.total_cmp(&b.1)).expect("a drone");
            let dr = &sim.drones.list[i];
            eprintln!("guard {g:?} -> drone#{i} at ({:.2},{:.2},{:.2}) off {d:.2} state {:#04x} team {} enabled {} yaw {:.3}",
                dr.pos[0], dr.pos[1], dr.pos[2], dr.state, dr.team, dr.enabled, dr.yaw);
            assert!(d < 0.5, "no port drone within 0.5 of the game's guard at {g:?} (nearest #{i} {d:.2} off)");
            i
        }).collect()
    };

    {
        let sim = scene.sim.as_mut().unwrap();
        let yaw = sim.bond.yaw;
        sim.bond = Bond::stand_at(&sim.world, BOND_AT, yaw);
    }

    let mut log: Vec<Vec<u8>> = vec![Vec::new(); idx.len()];
    let mut changes: Vec<String> = Vec::new();
    for t in 0..600u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0;
        sim.step_bond(Default::default(), false, false, false);
        for (k, &i) in idx.iter().enumerate() {
            let s = sim.drones.list[i].state;
            if log[k].last().is_some_and(|&p| p != s) || (t == 0 && s != 0x01) {
                changes.push(format!("t{t} drone#{i} {:?} -> {s:#04x} seen {} los {}",
                    log[k].last().map(|p| format!("{p:#04x}")), sim.drones.list[i].seen, sim.drones.list[i].los));
            }
            log[k].push(s);
        }
    }
    {
        // Diagnostic only: what holds the guards, and the port's guard-vision line to Bond's head.
        let sim = scene.sim.as_ref().unwrap();
        let head = [BOND_AT[0], BOND_AT[1] + 1.6, BOND_AT[2]];
        for &i in &idx {
            let d = &sim.drones.list[i];
            eprintln!("drone#{i} switch_channel {:#x} enabled {} los-to-bond-head clear {} hit {:?}", d.switch_channel, d.enabled,
                sim.world.line_of_sight(d.head(), head, nf_game::world::LOS_GUARD_VISION),
                sim.world.line_of_sight_hit(d.head(), head, nf_game::world::LOS_GUARD_VISION));
        }
    }
    let b = scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("bond after 600 ticks ({:.2},{:.2},{:.2})", b[0], b[1], b[2]);
    for c in &changes { eprintln!("CHANGE {c}"); }
    assert!((b[0] - BOND_AT[0]).abs() < 0.3 && (b[2] - BOND_AT[2]).abs() < 0.3, "Bond did not hold the spot");
    for (k, &i) in idx.iter().enumerate() {
        assert!(log[k].iter().all(|&s| s == 0x01),
            "drone#{i} (the game's guard at {:?}) left WAIT_SWITCH 0x01 — the game's stayed 0x01 all 600 frames [L live-1108 §6]: {changes:?}",
            GUARDS_AT[k]);
    }
}
