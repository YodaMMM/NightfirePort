//! `DroneVision_ProcessDroneSight` 0x176a28 skips any drone without `+0x4F8 & 0x100` (enabled)
//! in all three loops [M :81, :165, :188]: a disabled guard with the vision bit 0x10000 still
//! standing (`NDrone2_Enable(0)` 0x149dd0's mask 0xffe7fe5f leaves it) takes the third loop's
//! lost arm whatever its `+0x228 & 2` LOS flag holds [M :195-202] — no sight counter, no seen
//! ack. And `NDrone2_DSTATE_CivilianPatrol` 0x15d610's ENTER: `+0x9F2 == 0` (no route) goes
//! straight to Idle 4 [M :20-25].
use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// A guard switched off in play (`+0x4F8 & 0x100` clear, the vision bit 0x10000 left standing,
/// as `NDrone2_Enable(0)` 0x149dd0's mask 0xffe7fe5f leaves it) with its LOS flag forced up every
/// tick (the stale `+0x228 & 2` the game keeps across Enable(0)) gains no sight tick and no seen
/// ack, and counts unseen ticks. The port has no in-play per-drone Enable(0) path today (the
/// scripted Drone_EnableAll(0) also stops the whole sight pass), so the state is built by hand.
#[test]
fn disabled_guards_get_no_sight_on_0700000d() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut s = nf_view::load(&iso, "0700000d.bin", false).expect("load");
    let sim = s.sim.as_mut().unwrap();
    for _ in 0..4000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    // Bond 1.5 u from a living team-1 guard with a clear line from its head (the close-range
    // visibility probe under 2 u [M 0x175af0:24-33]); a few ticks enabled so the guard's opponent
    // distance is the near one, then the switch-off.
    let mut pick = None;
    'g: for (i, x) in sim.drones.list.iter().enumerate() {
        if !(x.present && !x.corpse && x.health > 0.0 && x.vision_on && x.enabled && x.team == 1) { continue; }
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::FRAC_PI_4;
            let p = [x.pos[0] + 1.5 * a.cos(), x.pos[1], x.pos[2] + 1.5 * a.sin()];
            let head = x.head();
            if sim.world.line_of_sight(head, [p[0], p[1] + 0.6, p[2]], nf_game::world::LOS_GUARD_VISION) {
                pick = Some((i, p)); break 'g;
            }
        }
    }
    let (gi, p) = pick.expect("a team-1 guard with a clear spot beside it on 0700000d");
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, p, 0.0);
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    let off = vec![gi];
    for &i in &off { sim.drones.list[i].enabled = false; }
    let unseen0: Vec<u32> = off.iter().map(|&i| sim.drones.list[i].unseen_ticks).collect();
    let mut ticks = 0u32;
    for _ in 0..90 {
        for &i in &off { sim.drones.list[i].los = true; }
        sim.step_bond(Input::default(), false, false, false);
        ticks += 1;
        for &i in &off {
            let x = &sim.drones.list[i];
            assert_eq!(x.seen_ticks, 0, "drone {i}: a disabled guard gains no sight tick [M 0x176a28:188]");
            assert!(!x.seen, "drone {i}: no seen ack while disabled");
        }
    }
    for (k, &i) in off.iter().enumerate() {
        assert!(sim.drones.list[i].unseen_ticks >= unseen0[k] + ticks,
            "drone {i}: the lost arm counts +0x274 up every tick [M 0x176a28:198-201]");
    }
}

/// CivilianPatrol entered without a route lands in Idle the same tick; with a route it stays.
#[test]
fn civilian_patrol_without_a_route_goes_idle() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut s = nf_view::load(&iso, "0700000d.bin", false).expect("load");
    let sim = s.sim.as_mut().unwrap();
    for _ in 0..4000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    let i = sim.drones.list.iter().position(|x| x.present && x.enabled && !x.corpse && x.health > 0.0 && x.team != 2 && x.started)
        .expect("an enabled guard on 0700000d");
    sim.drones.list[i].route = None;
    sim.drones.list[i].set_state(state::CIVILIAN_PATROL);
    sim.step_bond(Input::default(), false, false, false);
    let x = &sim.drones.list[i];
    assert_eq!(x.state, state::IDLE, "no route nodes: CivilianPatrol's ENTER sets Idle 4 [M 0x15d610:20-25]");
}
