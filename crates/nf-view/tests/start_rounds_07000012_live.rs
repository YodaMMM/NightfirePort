//! 07000012's idle start, round by round: every guard round fired at Bond in the first 480
//! ticks (shooter, tick, the accuracy roll, landed or not, health lost), read from the sim's
//! own records (`Drone::shots` / `shot_accuracy`, `Drones::player_hits`). No game state is
//! written; Bond stands still at the spawn on difficulty 1.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[derive(Debug, Clone)]
struct Round { shooter: usize, tick: u32, weapon: u8, dist: f32, pct: f32, rolled_hit: bool, landed: Option<(u32, f32)> }

fn idle_rounds(ticks: u32) -> (Vec<Round>, Vec<(u32, usize, f32)>, f32) {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000012.bin", false, None, 1).expect("load 07000012");
    let n = scene.sim.as_ref().unwrap().drones.list.len();
    let mut seen_shots = vec![0usize; n];
    let mut seen_acc = vec![0usize; n];
    let mut seen_hits = 0usize;
    let mut rounds: Vec<Round> = Vec::new();
    let mut hits = Vec::new();
    let mut states = vec![u8::MAX; n];
    for t in 0..=ticks {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones;
        for (i, g) in dr.list.iter().enumerate() {
            if i >= seen_shots.len() { seen_shots.push(0); seen_acc.push(0); states.push(u8::MAX); }
            if g.state != states[i] && !g.shots.is_empty() {
                eprintln!("[12 state] t{t:>4} guard {i:>2} state {:#04x} at ({:.2}, {:.2}, {:.2}) seen {} unseen {}",
                    g.state, g.pos[0], g.pos[1], g.pos[2], g.seen_ticks, g.unseen_ticks);
            }
            states[i] = g.state;
            for &f in &g.shots[seen_shots[i]..] {
                let acc = g.shot_accuracy.get(seen_acc[i]).copied();
                seen_acc[i] += 1;
                let (pct, rolled_hit) = acc.map_or((f32::NAN, false), |a| (a.1, a.2));
                // One record per `FireWeapon`; the record's +0x18 rounds share the roll.
                let w = g.weapon as u8;
                rounds.push(Round { shooter: i, tick: f, weapon: w, dist: g.dist, pct, rolled_hit, landed: None });
            }
            seen_shots[i] = g.shots.len();
        }
        for &(f, owner, lost) in &dr.player_hits[seen_hits..] {
            hits.push((f, owner, lost));
            // The newest unmatched round of that shooter fired at or before the hit (a guard
            // round crosses the ~10 u to Bond inside a tick or two, and a burst's rounds leave
            // 6 ticks apart).
            if let Some(r) = rounds.iter_mut().rev().find(|r| r.shooter == owner && r.landed.is_none() && r.tick <= f) {
                r.landed = Some((f, lost));
            }
        }
        seen_hits = dr.player_hits.len();
        let _ = t;
    }
    let sim = scene.sim.as_ref().unwrap();
    for w in [0x18usize, 0x19] {
        let r = &sim.weapons.records[w];
        let f = |o: usize| f32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        eprintln!("[12 weapon] {w:#x} damage {} range {} speed {} spread {} rounds {} burst {} interval {} f68 {:#x}",
            f(0x0c), f(0x1c), f(0x20), f(0x24), r[0x18], u16::from_le_bytes([r[0x2c], r[0x2d]]), i16::from_le_bytes([r[0x40], r[0x41]]),
            u32::from_le_bytes(r[0x68..0x6c].try_into().unwrap()));
    }
    eprintln!("[12 drone-hits] {:?}", sim.drones.drone_round_hits);
    for i in [6usize, 7] {
        let g = &sim.drones.list[i];
        for (f, m, a) in &g.shot_aims { eprintln!("[12 aim] guard {i} t{f} muzzle ({:.2}, {:.2}, {:.2}) aim ({:.2}, {:.2}, {:.2})", m[0], m[1], m[2], a[0], a[1], a[2]); }
        eprintln!("[12 guard] {i} accuracy {} aggression {} damage_mul {} wobble {} anim_set {:#x}", g.accuracy, g.aggression, g.damage_mul, g.wobble, g.anim_set);
    }
    let hp = sim.drones.health.health;
    (rounds, hits, hp)
}

/// The log itself: prints every round and the health left at 480. Pins nothing about the band
/// (see `start_pair_07000012_live`); it checks only that the records line up (each landed hit
/// has a round behind it).
#[test]
fn log_every_round_fired_at_bond_in_the_first_480_ticks() {
    let (rounds, hits, hp) = idle_rounds(480);
    for r in &rounds {
        eprintln!("[12 round] t{:>4} guard {:>2} w{:#04x} d{:>6.2} pct {:>5.1} roll {} landed {}",
            r.tick, r.shooter, r.weapon, r.dist, r.pct, if r.rolled_hit { "hit " } else { "miss" },
            r.landed.map_or("no".to_string(), |(f, l)| format!("t{f} -{l:.2}")));
    }
    let lost: f32 = hits.iter().map(|h| h.2).sum();
    eprintln!("[12 round] {} rounds, {} landed, {:.2} lost, health {hp:.2} at 480", rounds.len(), hits.len(), lost);
    assert!(hits.iter().all(|h| rounds.iter().any(|r| r.shooter == h.1 && r.tick <= h.0)), "every hit has a round: {hits:?}");
    // Per hit = weapon_data+0x0C (3.5 for 0x18/0x19 [D]) x Drone+0x100 (1.0 for the 0x18/0x19 class,
    // the captain's DroneCaptain_Mod_BulletDamage 2.0 [M NDrone2_DefaultInit 0x14b300:50/559]) x
    // Plr_DMod_Easy 0.5 on 0x12 [M ReadTuningVars 0x1d3a80], no armour at the start: 1.75 or 3.5.
    assert!(hits.iter().all(|h| (h.2 - 1.75).abs() < 1e-4 || (h.2 - 3.5).abs() < 1e-4), "each hit costs 1.75 or 3.5: {hits:?}");
}
