//! standins-scene-930: the online bot armour take runs `Pickup_Handler`'s class-2 case 3 whole —
//! the drone's health +20 clamped to BOT_vars+0xA4 AND the armour byte +0x769 += the pickup's
//! amount, capped 50 [M 0x18e650:119-133; `BOTWEAP_setDroneArmour` 0x12af70 is an empty body].
//! The pre-task stand-in applied only the health arm, so the brain's armour pool stayed 0
//! online. The test runs the live two-machine match (nf_view::net_tick, arena 07000023) and
//! asserts a bot that takes an armour pickup ends the take with a non-zero armour pool, while
//! the weapon/ammo neighbour path still takes and the two machines' hashes agree every tick.
//! NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 2;
    s.total_players = 4;
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

/// A slowly walking human (the bots get moving targets and the humans contest the pickups).
fn walk(t: u32, slot: usize) -> nf_net::Input {
    let mut i = NEUTRAL;
    let ph = t + slot as u32 * 131;
    i.forward = if ph % 300 < 220 { 0.7 } else { 0.0 };
    i.turn = if ph % 200 < 50 { 0.025 } else { 0.0 };
    i
}

#[test]
fn bot_armour_take_online_raises_the_brains_armour_pool() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipped"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let mut armour_takes: u32 = 0;
    let mut weapon_takes: u32 = 0;
    let mut armour_take: Option<(i16, Option<u8>)> = None; // (the pickup's amount, the pool after)
    for t in 0..3000u32 {
        let row: [nf_net::Input; nf_net::MAX_AGENTS] =
            std::array::from_fn(|sl| if sl < 2 { walk(t, sl) } else { NEUTRAL });
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the two machines' hashes diverged");
        let sim = a.sim.as_ref().unwrap();
        let takes = sim.net_last_takes.clone();
        for &(k, slot, bot) in &takes {
            if !bot { continue; }
            let Some(p) = sim.drones.pickups.get(k) else { continue };
            let pool = sim.mp.as_ref().and_then(|m| m.bots.iter().find(|bt| bt.slot == slot))
                .map(|bt| bt.brain.vars.arsenal.armour);
            match p.kind {
                nf_game::pickup::kind::ARMOUR => {
                    armour_takes += 1;
                    if armour_take.is_none() { armour_take = Some((p.rounds, pool)); }
                }
                nf_game::pickup::kind::WEAPON | nf_game::pickup::kind::AMMO => weapon_takes += 1,
                _ => {}
            }
        }
        if armour_takes > 0 && weapon_takes > 0 && armour_take.map_or(false, |(_, p)| p.map_or(false, |v| v > 0)) {
            break;
        }
    }
    eprintln!("armour takes {armour_takes}, weapon/ammo takes {weapon_takes}, first armour take {armour_take:?}");
    assert!(weapon_takes > 0, "the weapon/ammo neighbour path took nothing — the harness broke");
    assert!(armour_takes > 0, "no bot armour take in 3000 ticks — the harness broke");
    // The stand-in (health only, no brain take) leaves the pool at 0: this is the assertion
    // that fails on it. The pool byte = 0 + the pickup's amount, capped 50 [M 0x18e650:128-132].
    let (rounds, pool) = armour_take.expect("armour take recorded");
    assert_eq!(pool, Some((rounds.clamp(0, 50)) as u8), "the armour pool byte after the take");
}
