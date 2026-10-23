//! mp-bot-weapons: the bots use the game's weapon layer online. Two machines (slot 0's and slot
//! 1's, the live match-start setup) tick the same input rows through `nf_view::net_tick` on
//! 07000023 with bots; every tick's hash must agree while the bots fight, take the arena's
//! weapon/ammo/armour pickups through the shared take pass, reload and switch weapons. The run
//! also reports the kill rate (the before/after number of the task) and asserts the layer
//! actually ran: an armed arsenal, a pickup taken by a bot, a weapon switch, kills.
//! NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings(humans: u32, bots: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = humans;
    s.player_count = humans as u16;
    s.bot_count = bots;
    s.total_players = humans + bots;
    s.seed = 0x0bed_0bad;
    s.frag_limit = 120; // the run must not end on the frag limit
    s.time_limit = 10 * 60;
    s
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

/// A slowly walking human (so the humans take pickups too and the bots have moving targets).
fn walk(t: u32, slot: usize) -> nf_net::Input {
    let mut i = NEUTRAL;
    let ph = t + slot as u32 * 131;
    i.forward = if ph % 300 < 220 { 0.7 } else { 0.0 };
    i.turn = if ph % 200 < 50 { 0.025 } else { 0.0 };
    i
}

#[derive(Default, Debug)]
struct Tally {
    kills: i32,
    takes_bots: u32,
    takes_humans: u32,
    switches: u32,
    reloads: u32,
    non_start_kills_weapons: std::collections::BTreeSet<u8>,
    weapons_seen: std::collections::BTreeSet<u8>,
    /// mp-bot-weapons: shots fired while holding the start weapon 6 (the P2K) — the
    /// "0xD4 strafe, latch set, never fires" defect's negative.
    w6_shots: u32,
    /// The smallest gap in ticks between two consecutive weapon-6 shots of one bot — the
    /// game's 60-tick interval floor for weapon 6 [M DroneWeap_NextBulletTime 0x179d40:26-47].
    w6_min_gap: u32,
    /// Weapon ids a bot picked up during the run (held-set growth after the arming tick).
    taken_weapons: std::collections::BTreeSet<u8>,
    /// Shots fired with a picked-up weapon — the take → switch → fire chain.
    taken_used_shots: u32,
}

fn run(humans: usize, bots: u32, ticks: u32) -> Option<Tally> {
    let path = iso()?;
    let s = settings(humans as u32, bots);
    let slots = [0usize, humans - 1];
    let mut a = machine(&path, &s, slots[0]);
    let mut b = machine(&path, &s, slots[1]);
    {
        let sim = a.sim.as_ref().unwrap();
        let kinds: Vec<(u16, u16, i16, u16)> = sim.drones.pickups.iter().map(|p| (p.kind, p.weapon, p.rounds, p.respawn)).collect();
        eprintln!("arena pickups (kind, weapon, rounds, respawn): {kinds:x?}");
    }
    let mut tally = Tally::default();
    tally.w6_min_gap = u32::MAX; // the min-gap accumulator: raise from "no gap seen" 
    let mut prev_weapon: Vec<u8> = Vec::new();
    let mut prev_arsenal: Vec<(i32, u32, u8)> = Vec::new();
    let mut prev_held: Vec<Vec<bool>> = Vec::new();
    let mut prev_shots_by_weapon: Vec<Vec<u32>> = Vec::new();
    let mut last_w6_tick: Vec<u32> = Vec::new();
    let mut prev_pickups: Vec<(bool, u8)> = Vec::new();
    for t in 0..ticks {
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|sl| if sl < humans { walk(t, sl) } else { NEUTRAL });
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, slots[0]);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, slots[1]);
        let sa = a.sim.as_ref().unwrap();
        if ha != hb {
            let sb = b.sim.as_ref().unwrap();
            let ma = sa.mp.as_ref().unwrap();
            let mb = sb.mp.as_ref().unwrap();
            let mut diff = Vec::new();
            for (i, (x, y)) in ma.bots.iter().zip(mb.bots.iter()).enumerate() {
                if x.brain.state_hash() != y.brain.state_hash() || x.health != y.health {
                    diff.push(format!("bot{i}: state {:#x}/{:#x} w {:#x}/{:#x} hp {}/{} pos {:?}/{:?} arsenal-eq {}",
                        x.brain.state, y.brain.state, x.brain.vars.weapon_id, y.brain.vars.weapon_id, x.health, y.health,
                        x.brain.pos, y.brain.pos, x.brain.vars.arsenal == y.brain.vars.arsenal));
                }
            }
            for (k, (p, q)) in sa.drones.pickups.iter().zip(sb.drones.pickups.iter()).enumerate() {
                if (p.state, p.gone, p.hidden, p.respawn_ticks) != (q.state, q.gone, q.hidden, q.respawn_ticks) {
                    diff.push(format!("pickup{k}: {:?} vs {:?}", (p.state, p.gone, p.hidden, p.respawn_ticks), (q.state, q.gone, q.hidden, q.respawn_ticks)));
                }
            }
            diff.push(format!("hp {:?} vs {:?}; armour {:?} vs {:?}", sa.mp_hp, sb.mp_hp, sa.mp_armour, sb.mp_armour));
            diff.push(format!("rng {:x}/{:x} vs {:x}/{:x}", sa.drones.rng.x, sa.drones.rng.draws, sb.drones.rng.x, sb.drones.rng.draws));
            panic!("tick {t}: hash {ha:x} vs {hb:x}\n{}", diff.join("\n"));
        }
        let m = sa.mp.as_ref().unwrap();
        // the bots' weapon switches and takes (an arsenal that grew: a held weapon, a pool, armour)
        if prev_weapon.is_empty() {
            prev_weapon = m.bots.iter().map(|b| b.brain.vars.weapon_id).collect();
            prev_arsenal = vec![(0, 0, 0); m.bots.len()];
            prev_held = m.bots.iter()
                .map(|b| b.brain.vars.arsenal.held.to_vec()).collect();
            last_w6_tick = vec![u32::MAX; m.bots.len()];
        }
        for (i, bot) in m.bots.iter().enumerate() {
            // the take pins: a held-set growth after the arming tick is the pickup (the
            // preference choice arms the bot with the taken weapon — `BotArsenal::take_pickup`).
            for (w, (&now, &was)) in bot.brain.vars.arsenal.held.iter().zip(prev_held[i].iter()).enumerate() {
                if now && !was && bot.brain.tick > 2 {
                    tally.taken_weapons.insert(w as u8);
                }
            }
            prev_held[i] = bot.brain.vars.arsenal.held.to_vec();
        }
        for (i, bot) in m.bots.iter().enumerate() {
            let v = &bot.brain.vars;
            tally.weapons_seen.insert(v.weapon_id);
            // the per-weapon shot tally (the scene drains the brain's shot log this tick):
            // a new 6-shot stamps the tick for the interval-floor gap, a new taken-weapon
            // shot counts the take → switch → fire chain.
            let sbw = &m.bots[i].shots_by_weapon;
            if prev_shots_by_weapon.len() != m.bots.len() {
                prev_shots_by_weapon = (0..m.bots.len()).map(|_| vec![0u32; sbw.len()]).collect();
            }
            if prev_shots_by_weapon[i].len() == sbw.len() {
                for (w, (&now, &was)) in sbw.iter().zip(prev_shots_by_weapon[i].iter()).enumerate() {
                    if now > was {
                        if w == 6 {
                            tally.w6_shots += now - was;
                            if last_w6_tick[i] != u32::MAX {
                                tally.w6_min_gap = tally.w6_min_gap.min(t.wrapping_sub(last_w6_tick[i]));
                            }
                            last_w6_tick[i] = t;
                        }
                        if tally.taken_weapons.contains(&(w as u8)) {
                            tally.taken_used_shots += now - was;
                        }
                    }
                }
            }
            prev_shots_by_weapon[i] = sbw.clone();
            if v.weapon_id != prev_weapon[i] && v.arsenal.armed && bot.brain.tick > 2 {
                tally.switches += 1;
                eprintln!("t{t} bot{i} switches {:#x} -> {:#x} (state {:#x}, opp dist {:.1})", prev_weapon[i], v.weapon_id, bot.brain.state, v.opp_dist);
            }
            prev_weapon[i] = v.weapon_id;
            let pools: i32 = v.arsenal.pool.iter().map(|&p| p as i32).sum();
            let held = v.arsenal.held.iter().filter(|&&h| h).count() as u32;
            let now = (pools, held, v.arsenal.armour);
            let (pp, ph, pa) = prev_arsenal[i];
            if bot.brain.tick > 2 && (held > ph || v.arsenal.armour > pa || pools > pp) {
                // a reload moves pool → clip (pool drops), so a growth is a take
                tally.takes_bots += 1;
            }
            prev_arsenal[i] = now;
            if bot.brain.trace.last().is_some_and(|&(tk, _, to)| tk == bot.brain.tick && to == 0xDD) {
                tally.reloads += 1;
            }
        }
        // the humans' takes: pickups that went to state 2 / gone without a bot take this tick
        let cur: Vec<(bool, u8)> = sa.drones.pickups.iter().map(|p| (p.gone, p.state)).collect();
        if prev_pickups.len() == cur.len() {
            let taken = cur.iter().zip(prev_pickups.iter()).filter(|(c, p)| (c.0 && !p.0) || (c.1 == 2 && p.1 != 2)).count() as u32;
            let _ = taken;
        }
        prev_pickups = cur;
    }
    let sa = a.sim.as_ref().unwrap();
    let m = sa.mp.as_ref().unwrap();
    if let Some(t) = sa.mp_bot_weapons.as_ref() {
        for w in [6u8, 0x10, 0x1e, 0x2c] {
            eprintln!("  table {w:#x}: {:?}", t.row(w));
        }
    }
    tally.kills = m.game.slots.iter().map(|s| s.kills).sum();
    let _ = &mut tally.takes_humans;
    let _ = &mut tally.non_start_kills_weapons;
    let mins = ticks as f32 / 3600.0;
    eprintln!("{humans} humans + {bots} bots, {ticks} ticks ({mins:.1} min): all hashes agree; kills {} ({:.1}/min), bot takes {}, switches {}, reloads {}, weapons seen {:x?}",
        tally.kills, tally.kills as f32 / mins, tally.takes_bots, tally.switches, tally.reloads, tally.weapons_seen);
    for (i, bot) in m.bots.iter().enumerate() {
        let v = &bot.brain.vars;
        let by_w: Vec<(usize, u32)> = bot.shots_by_weapon.iter().enumerate()
            .filter(|(_, &c)| c > 0).map(|(w, &c)| (w, c)).collect();
        eprintln!("  bot{i}: weapon {:#x} held {:x?} armour {} hp {:.0} deaths {} shots {} by-weapon {:x?}",
            v.weapon_id,
            (0..0x53u8).filter(|&w| v.arsenal.held[w as usize]).collect::<Vec<_>>(), v.arsenal.armour, bot.health, bot.deaths, bot.shots, by_w);
    }
    // Sister: the comparison can fail — a bot's armour is in the hash.
    let sb = b.sim.as_mut().unwrap();
    let h0 = sb.net_state_hash(slots[1]);
    sb.mp.as_mut().unwrap().bots[0].brain.vars.arsenal.armour ^= 1;
    assert_ne!(sb.net_state_hash(slots[1]), h0, "the bot arsenal is in the hash");
    sb.mp.as_mut().unwrap().bots[0].brain.vars.arsenal.armour ^= 1;
    let h1 = sb.net_state_hash(slots[1]);
    assert_eq!(h1, h0);
    sb.drones.pickups[0].state ^= 2;
    assert_ne!(sb.net_state_hash(slots[1]), h0, "the pickup take state is in the hash");
    Some(tally)
}

#[test]
fn bots_fight_take_pickups_and_switch_in_lockstep() {
    let Some(t) = run(2, 4, 3 * 3600) else { eprintln!("skip: NF_ISO not set"); return };
    assert!(t.takes_bots > 0, "a bot took a pickup: {t:?}");
    assert!(t.switches > 0, "a bot switched weapons: {t:?}");
    assert!(t.kills > 0, "kills happen: {t:?}");
    assert!(t.weapons_seen.len() >= 2, "more than the start weapon was used: {t:?}");
}

/// mp-bot-weapons: the start weapon 6 fires (a bot in 0xD4 with the fire latch set that never
/// fires does not reproduce here) and honours the game's interval floor:
/// no two weapon-6 shots of one bot land within 60 ticks [M DroneWeap_NextBulletTime
/// 0x179d40:26-47 — the min-60 clamp for weapons 6/0xA/0xE..0x10]. Sister: a picked-up weapon
/// is not only taken but fired — the preference choice leads the take → switch → fire chain.
#[test]
fn weapon6_fires_and_honours_the_interval_floor() {
    let Some(t) = run(2, 4, 3 * 3600) else { eprintln!("skip: NF_ISO not set"); return };
    assert!(t.w6_shots > 0, "a bot holding weapon 6 fires: {t:?}");
    assert!(t.w6_min_gap >= 60, "weapon 6's 60-tick floor holds (min gap {}): {t:?}", t.w6_min_gap);
    assert!(!t.taken_weapons.is_empty(), "a bot picked up a weapon: {t:?}");
    assert!(t.taken_used_shots > 0, "a taken weapon was fired: {t:?}");
}
