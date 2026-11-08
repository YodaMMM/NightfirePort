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

/// The game's second pass [L live-1108 §9, PCSX2]: intro skipped, Bond at the start spot
/// (-8.486, 6.47, -9.782), ch 0x5 and 0xe re-raised every frame for a 900-frame wait, then Bond
/// at (11.43, 6.47, -3.51) yaw 0 for 600 frames.
/// - The guard at (6.98, 6.34, 8.48) left 0x01 at +748 of the wait (0x58 -> 0x5a -> 0x5f) and later
///   sat in CombatWait 0x5e, walking from z 8.48 to 3.86 toward Bond.
/// - The guard at (6.95, 6.34, 2.20) stayed 0x01 through the wait, left it for Combat 0x58 five
///   frames after the placement, reached AimStandFire 0x68 at +53 and sat in 0x68 for 547 of the
///   600 frames, turning from yaw 1.57 to 2.27; 24 rounds from a muzzle at (7.83, 6.47, 1.73),
///   11 reaching Bond.
///
/// The release itself is a race in the game run: ch 0x5 is the OUT copy of ch 0xd (trigger #5,
/// list [4, 5], rewritten every frame [M Trigger_Update 0x1b1a78 OUT arm]), so a WaitSwitch
/// drone only sees a between-frames poke when its update lands before that copy. The port
/// steps whole ticks, so the test releases each guard at the game's own release frame instead:
/// its switch channel is pointed at ch 0xe (held on), which takes the same WaitSwitch update
/// arm [M NDrone2_DSTATE_WaitSwitch 0x15b960 msg 3] to the drone's start state.
#[test]
fn released_guards_see_and_fire_at_bond_beyond_the_screens() {
    let mut scene = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    let intro = 0x0600_0087u32;
    for t in 0..1200u32 {
        let sim = scene.sim.as_mut().unwrap();
        let pending = sim.drones.scripts.players.iter().any(|p| p.scripts[0] == intro
            && (matches!(p.state, 1 | 2 | 7) || p.info[p.cur].as_ref().is_some_and(|s| s.play & 1 != 0)));
        if t >= 2 && !pending && !sim.disabled { break; }
        sim.drones.scripts.skip_request = true;
        sim.step_bond(Default::default(), false, false, false);
    }
    let idx: Vec<usize> = {
        let sim = scene.sim.as_ref().unwrap();
        GUARDS_AT.iter().map(|g| sim.drones.list.iter().enumerate().filter(|(_, d)| d.present)
            .min_by(|a, b| { let f = |p: [f32; 3]| (p[0] - g[0]).powi(2) + (p[2] - g[2]).powi(2); f(a.1.pos).total_cmp(&f(b.1.pos)) })
            .map(|(i, _)| i).unwrap()).collect()
    };
    let (near, mid) = (idx[0], idx[1]);
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!((sim.drones.list[near].switch_channel, sim.drones.list[mid].switch_channel), (5, 5));
        sim.bond = Bond::stand_at(&sim.world, [-8.486, 6.47, -9.782], sim.bond.yaw);
    }
    let mut last: Vec<u8> = idx.iter().map(|&i| scene.sim.as_ref().unwrap().drones.list[i].state).collect();
    let log = |scene: &nf_view::scene::Scene, phase: &str, t: u32, last: &mut Vec<u8>| {
        let sim = scene.sim.as_ref().unwrap();
        for (k, &i) in idx.iter().enumerate() {
            let d = &sim.drones.list[i];
            if d.state != last[k] {
                eprintln!("{phase} t{t} drone#{i} {:#04x} -> {:#04x} pos ({:.2},{:.2},{:.2}) yaw {:.2} seen {} los {}",
                    last[k], d.state, d.pos[0], d.pos[1], d.pos[2], d.yaw, d.seen, d.los);
                last[k] = d.state;
            }
        }
    };
    let step = |scene: &mut nf_view::scene::Scene| {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.set_channel(0x5);
        sim.drones.set_channel(0xe);
        sim.drones.health.health = 100.0;
        sim.step_bond(Default::default(), false, false, false);
    };
    for t in 0..900u32 {
        if t == 747 { scene.sim.as_mut().unwrap().drones.list[mid].switch_channel = 0xe; }
        step(&mut scene);
        log(&scene, "wait", t, &mut last);
        if t < 747 { assert_eq!(scene.sim.as_ref().unwrap().drones.list[mid].state, 0x01); }
    }
    assert_eq!(scene.sim.as_ref().unwrap().drones.list[near].state, 0x01, "the near guard waits through the wait [L §9]");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = Bond::stand_at(&sim.world, BOND_AT, 0.0);
    }
    let shots0 = scene.sim.as_ref().unwrap().drones.list[near].shot_aims.len();
    let mut first_leave = None;
    let mut first_aim = None;
    let mut aim_ticks = 0u32;
    let mut hit_ticks = 0u32;
    for t in 0..600u32 {
        if t == 4 { scene.sim.as_mut().unwrap().drones.list[near].switch_channel = 0xe; }
        step(&mut scene);
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.health.health < 100.0 { hit_ticks += 1; }
        let s = sim.drones.list[near].state;
        if first_leave.is_none() && s != 0x01 { first_leave = Some((t, s)); }
        if s == 0x68 { aim_ticks += 1; if first_aim.is_none() { first_aim = Some(t); } }
        log(&scene, "held", t, &mut last);
    }
    let sim = scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[near];
    let shots = &d.shot_aims[shots0..];
    for (f, m, a) in shots.iter().take(30) { eprintln!("SHOT frame {f} muzzle ({:.2},{:.2},{:.2}) aim ({:.2},{:.2},{:.2})", m[0], m[1], m[2], a[0], a[1], a[2]); }
    eprintln!("near guard: first leave {first_leave:?}, first 0x68 {first_aim:?}, 0x68 ticks {aim_ticks}, end yaw {:.2} pos {:?}, rounds {}, damage ticks {hit_ticks}",
        d.yaw, d.pos, shots.len());
    // Pins (the game's run [L live-1108 §9]):
    // - release goes to Combat 0x58: Attack 0x164210:71-87 compares the engage range with the
    //   distance frozen while the drone was disabled (`Drone_GetOpponentInfo` 0x138f68:16-18 needs
    //   +0x4F8 & 0x100) — 17.5 from the load spot, out of the 15.7 range — not AimStand 0x67;
    assert_eq!(first_leave, Some((4, 0x58)), "the released guard enters Combat 0x58 as the game's did");
    // - he aims and fires at Bond: AimStandFire 0x68 within the game's +53 and held most of the
    //   600 frames (the game's 547; the port's route differs in detail, see the report);
    assert!(first_aim.is_some_and(|t| t <= 60), "AimStandFire 0x68 by +60 (game +53): {first_aim:?}");
    assert!(aim_ticks >= 400, "0x68 for most of the 600 frames (game 547): {aim_ticks}");
    // - his rounds are aimed at Bond and reach him through the opening (11 of 24 in the game's
    //   6 s window).
    assert!(shots.len() >= 10 && shots.iter().all(|(_, _, a)| (a[0] - BOND_AT[0]).abs() < 0.5 && (a[2] - BOND_AT[2]).abs() < 0.5),
        "the guard fires at Bond: {} rounds", shots.len());
    assert!(hit_ticks >= 1, "at least one round reaches Bond (game: 11)");
}
