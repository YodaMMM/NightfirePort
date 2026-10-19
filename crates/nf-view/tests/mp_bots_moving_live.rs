//! The two-window "desync when I move one" report (2026-10-18): host + joiner on 07000023 with
//! bots, a player moving. Headless: two sims (slot 0's machine and slot 1's machine, the live
//! match-start setup — the per-machine private generator, the joiner's `swap_local_slot`) tick
//! the SAME input rows through `nf_view::net_tick` (the live loop's tick shape: the local slot's
//! full `step_bond`, the remotes, the online combat, the slot positions, the hash). The humans
//! walk/turn the whole run; every tick's hash must agree. On the first mismatch the test names the
//! diverging fields (the NF_NET_HASHDBG fields plus the bots' and the pickups' state).
//! NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// A walking player: forward most of the time, turning, strafing now and then, a jump/crouch.
fn walk(t: u32, slot: usize) -> nf_net::Input {
    let mut i = NEUTRAL;
    let ph = t + slot as u32 * 97;
    i.forward = if ph % 240 < 200 { 1.0 } else { -0.5 };
    i.strafe = if ph % 150 < 30 { 0.8 } else if ph % 150 < 45 { -0.8 } else { 0.0 };
    i.turn = if ph % 180 < 60 { 0.03 } else if ph % 180 < 90 { -0.02 } else { 0.0 };
    i.look = if ph % 120 < 20 { 0.01 } else if ph % 120 < 40 { -0.01 } else { 0.0 };
    i.jump = ph % 211 == 0;
    i.crouch = ph % 307 == 0;
    i
}

fn settings(humans: u32, bots: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = humans;
    s.player_count = humans as u16;
    s.bot_count = bots;
    s.total_players = humans + bots;
    s.seed = 0x0bed_0bad;
    s.time_limit = 10 * 60; // the live host's 10 minutes (deathmatch()'s raw 10 ends in 10 s)
    s
}

/// The live match start's per-machine setup (`swap_in_scene`'s net block).
fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

/// The state a desync hunt compares, field by field (by SLOT, whatever the machine's local slot).
fn fields(sim: &nf_view::scene::Sim) -> Vec<(String, String)> {
    let mut v = Vec::new();
    v.push(("frame".into(), format!("{}", sim.drones.frame)));
    v.push(("rng".into(), format!("{:x}/{:x}/{}", sim.drones.rng.x, sim.drones.rng.y, sim.drones.rng.draws)));
    for s in 0..nf_net::MAX_AGENTS {
        let b = if s == sim.mp_local_slot { Some(&sim.bond) } else { sim.mp_bonds[s].as_ref() };
        if let Some(b) = b {
            v.push((format!("body{s}"), format!("{:?} yaw {} pitch {} ss {} cr {}", b.pos, b.yaw, b.pitch, b.substate, b.crouched)));
        }
        v.push((format!("hp{s}"), format!("{} dead {}", sim.mp_hp[s], sim.mp_dead_ticks[s])));
    }
    if let Some(m) = &sim.mp {
        v.push(("game".into(), format!("state {} clock {} limit {}", m.game.state, m.game.clock, m.game.limit)));
        for (i, s) in m.game.slots.iter().enumerate() {
            v.push((format!("slot{i}"), format!("{:?} {:?} {} {} {} {}", s.pos, s.timers, s.score, s.kills, s.deaths, s.dead)));
        }
        for (i, b) in m.bots.iter().enumerate() {
            v.push((format!("bot{i}"), format!("pos {:?} yaw {} state {:#x} hp {} dead {} opp {:?}", b.brain.pos, b.brain.yaw, b.brain.state, b.health, b.dead, b.brain.vars.opponent)));
        }
    }
    v.push(("pickups".into(), format!("{}", sim.drones.pickups.len())));
    v.push(("bot_pickups".into(), format!("{:?}", sim.mp_bot_pickups.iter().map(|p| (p.pos, p.weapon_id, p.mp_class)).collect::<Vec<_>>())));
    v
}

fn run(humans: usize, bots: u32, ticks: u32, moving: bool) -> u32 {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return 0 };
    let s = settings(humans as u32, bots);
    // Two machines: the host (slot 0) and the last human's (the joiner).
    let slots = [0usize, humans - 1];
    let mut a = machine(&path, &s, slots[0]);
    let mut b = machine(&path, &s, slots[1]);
    assert_eq!(a.sim.as_ref().unwrap().mp_bot_count(), bots as usize);
    let mut moved = 0.0f32;
    let mut deaths = 0u32; // human deaths seen (the dead-body freeze is the fixed path)
    let mut was_dead = [false; nf_net::MAX_AGENTS];
    let start = a.sim.as_ref().unwrap().bond.pos;
    for t in 0..ticks {
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|sl| {
            if moving && sl < humans { walk(t, sl) } else { NEUTRAL }
        });
        let sa = a.sim.as_mut().unwrap();
        let ha = nf_view::net_tick(sa, &row, slots[0]);
        let sb = b.sim.as_mut().unwrap();
        let hb = nf_view::net_tick(sb, &row, slots[1]);
        if ha != hb {
            let fa = fields(a.sim.as_ref().unwrap());
            let fb = fields(b.sim.as_ref().unwrap());
            let diff: Vec<String> = fa.iter().zip(fb.iter()).filter(|(x, y)| x.1 != y.1)
                .map(|(x, y)| format!("{}: machine{} {} | machine{} {}", x.0, slots[0], x.1, slots[1], y.1)).collect();
            panic!("tick {t}: hash {ha:x} vs {hb:x}; diverging fields:\n{}", diff.join("\n"));
        }
        for h in 0..humans.min(nf_net::MAX_AGENTS) {
            let d = a.sim.as_ref().unwrap().mp_dead_ticks[h] > 0;
            if d && !was_dead[h] { deaths += 1; }
            was_dead[h] = d;
        }
        let p = a.sim.as_ref().unwrap().bond.pos;
        moved = moved.max(((p[0] - start[0]).powi(2) + (p[2] - start[2]).powi(2)).sqrt());
    }
    eprintln!("{humans} humans + {bots} bots, moving {moving}: {ticks} ticks agree; slot 0 got {moved:.1} from its spawn; {deaths} human deaths");
    if moving { assert!(moved > 2.0, "the scripted walk moves the player ({moved})"); }
    // Sister: the comparison can fail — one machine's replica of the other human turned a hair.
    let sb = b.sim.as_mut().unwrap();
    let h0 = sb.net_state_hash(slots[1]);
    if let Some(r) = sb.mp_bonds[slots[0]].as_mut() { r.yaw += 1e-3; }
    assert_ne!(sb.net_state_hash(slots[1]), h0, "a body's yaw is in the hash");
    deaths
}

#[test]
fn two_humans_2_bots_moving_hashes_agree() {
    run(2, 2, 7200, true);
}

#[test]
fn two_humans_14_bots_moving_hashes_agree() {
    // The bots kill the walking humans: before the fix the dead human's replica kept walking on
    // the other machine (the first diverging field: body1, tick 682, two ticks after the kill);
    // with that fixed, the replica's stale collision cel (no per-tick relink) split body1 by
    // 0.004 at tick 2738 on a crouch-walk along a wall.
    let deaths = run(2, 14, 3600, true);
    if iso().is_some() { assert!(deaths > 0, "the run exercises a human death while moving"); }
}

#[test]
fn two_humans_14_bots_idle_hashes_agree() {
    run(2, 14, 1800, false);
}
