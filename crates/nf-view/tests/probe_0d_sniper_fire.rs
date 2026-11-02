//! probe (assertion-free, `--ignored`): the port's per-shot behaviour for the 0d weapon-0x1e
//! snipers at the real-damage walk's death spot. Loads 0700000d, teleports Bond next to the
//! walk's death spot (186.3, −3.4, −266), ticks up to N, and logs for every drone with skin
//! 0x05000087 / weapon 0x1e: spawn tick, each shot (frame, dist, seen_ticks, lost_sight, state,
//! anim class, raw roll, bullet damage) and the sight-streak resets. Run:
//! NF_ISO=… cargo test -p nf-view --test probe_0d_sniper_fire -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_sniper_fire_at_the_death_spot() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [186.3, -3.36, -266.0];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = 2.5;
    }
    let mut spawned_at: std::collections::HashMap<usize, u32> = Default::default();
    let mut last_seen: std::collections::HashMap<usize, (u32, u32)> = Default::default();
    for tick in 0..4000u32 {
        // Bond paces between two spots (moving, like the walk) unless the env says stand.
        let stand = std::env::var_os("NF_PROBE_STAND").is_some();
        let input = if stand { Default::default() } else {
            let t = (tick / 90) % 2;
            nf_game::player::Input { forward: 0.8, turn: if t == 0 { 0.0 } else { 3.0 }, ..Default::default() }
        };
        w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        w.total_ticks += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        let frame = sim.drones.frame;
        for (i, dr) in sim.drones.list.iter().enumerate() {
            if !dr.present { continue; }
            if dr.skin != 0x0500_0087 && dr.skin != 0x0500_0090 { continue; }
            if !spawned_at.contains_key(&i) {
                spawned_at.insert(i, frame);
                eprintln!("[probe] f{}: drone #{i} skin {:#x} weapon {:#x} PRESENT at ({:.1},{:.1},{:.1}) acc {} aggr {} hp {} state {:#x} class {} sight {:.0} engage {:.1}",
                    frame, dr.skin, dr.weapon, dr.pos[0], dr.pos[1], dr.pos[2], dr.accuracy, dr.aggression, dr.health, dr.state, dr.anim.class, dr.sight_range, dr.engage_range);
            }
            let n_shots = dr.shots.len();
            let prev = last_seen.insert(i, (n_shots as u32, frame)).map(|(s, _)| s).unwrap_or(0);
            if n_shots as u32 > prev {
                for k in prev as usize..n_shots {
                    let roll = dr.shot_rolls.get(k).copied();
                    eprintln!("[probe] f{}: drone #{i} SHOT {} at dist {:.1} seen_ticks {} lost {} state {:#x} roll {:?}",
                        frame, k, dr.dist, dr.seen_ticks, dr.lost_sight, dr.state, roll);
                }
            }
        }
        if sim.drones.health.health <= 0.0 { eprintln!("[probe] Bond died at f{frame}"); break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[probe] end f{} bond hp {:.1} pos ({:.1},{:.1},{:.1})", sim.drones.frame, sim.drones.health.health, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    for (i, dr) in sim.drones.list.iter().enumerate() {
        if dr.skin == 0x0500_0087 && dr.present {
            eprintln!("[probe] end: drone #{i} shots {} rolls {} hits-by-owner {:?}", i, dr.shots.len(), sim.drones.player_hits.iter().filter(|(_, o, _)| *o == i).count());
        }
    }
}
