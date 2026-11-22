//! mp-bot-weapon-gate: online, a listed launcher's rocket on a bot hurts by its blast only, the
//! same on both machines.
//!
//! The game's bots are drone objects: `NDrone2_DealWithObjHit` 0x13fff0 case 5 returns on the
//! round's record id for 0x1b, 0x2a, 0x2b, 0x2e..0x31, 0x34..0x37, 0x3a, 0x46, 0x48, 0x49, 0x4e,
//! 0x50, 0x51 and 0x5f before the team gate, the puff and the hit message [M :153-173]; the
//! rocket's blast still reaches the bot through `Explode_Propagate`'s drone arm (no record
//! test) [M 0x17c518 case 2 → Drone_ExplosiveHit]. Two machines (nf_view::net_tick, arena
//! 07000023, two humans, two bots held at 300 health on both); slot 0 fires the launcher 0x2f
//! (damage 100, radius 8 [D weapon_data]) at the nearest live bot. Every hit takes at most the
//! blast's 100 (before the fix the contact's arm-scaled 100 came on top and the 300-point bot
//! died in one rocket), the two machines' bot health agrees, and the state hashes agree every
//! tick. NF_ISO-gated.

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 2;
    s.total_players = 4;
    s.seed = 0x0bed_0bad;
    s.frag_limit = 120;
    s.time_limit = 10 * 60;
    s
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    for b in sim.mp.as_mut().unwrap().bots.iter_mut() { b.health = 300.0; }
    scene
}

#[test]
fn online_listed_launcher_rocket_hurts_a_bot_by_its_blast_only() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let w = 0x2fu8;
    let mut hits: Vec<f32> = Vec::new();
    let mut last_shot = 0u32;
    let mut shots = 0u32;
    for t in 0..6000u32 {
        let sim = a.sim.as_ref().unwrap();
        let m = sim.mp.as_ref().unwrap();
        let me = sim.bond.head();
        let target = m.bots.iter()
            .filter(|bt| !bt.dead && sim.world.segment(me, bt.brain.pos).is_none())
            .min_by(|p, q| {
                let d = |x: &[f32; 3]| (x[0] - me[0]).powi(2) + (x[1] - me[1]).powi(2) + (x[2] - me[2]).powi(2);
                d(&p.brain.pos).total_cmp(&d(&q.brain.pos))
            })
            .map(|bt| bt.brain.pos);
        let mut row = [NEUTRAL; nf_net::MAX_AGENTS];
        row[0].held = w;
        if let Some(tg) = target {
            row[0].shot = t >= last_shot + 60;
            if row[0].shot { last_shot = t; shots += 1; }
            row[0].shot_weapon = w;
            row[0].shot_aim = nf_game::mp::combat::aim_from_dir([tg[0] - me[0], tg[1] - 0.6 - me[1], tg[2] - me[2]]);
        }
        let hp: Vec<(usize, f32, bool)> = m.bots.iter().map(|bt| (bt.slot, bt.health, bt.dead)).collect();
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the two machines' hashes diverged");
        // Only the ticks a 0x2f rocket blasted count (the bots also shoot each other).
        let rocket = a.sim.as_ref().unwrap().mp_combat.blasts.iter().any(|bl| bl.weapon == w as u16 && bl.owner == 0);
        let (ma, mb) = (a.sim.as_ref().unwrap().mp.as_ref().unwrap(), b.sim.as_ref().unwrap().mp.as_ref().unwrap());
        for (i, bt) in ma.bots.iter().enumerate() {
            assert_eq!(bt.health, mb.bots[i].health, "tick {t}: bot {} health differs between machines", bt.slot);
            let Some(&(_, h0, d0)) = hp.iter().find(|x| x.0 == bt.slot) else { continue };
            if d0 || bt.health >= h0 || !rocket { continue; }
            let loss = h0 - bt.health;
            eprintln!("[bot-gate] t{t} bot {} health {h0} -> {} (dead {})", bt.slot, bt.health, bt.dead);
            hits.push(loss);
        }
        // The bots stay at 300 (equal on both machines), so a hit is never capped by what is left.
        for sim in [a.sim.as_mut().unwrap(), b.sim.as_mut().unwrap()] {
            for bt in sim.mp.as_mut().unwrap().bots.iter_mut() {
                if !bt.dead { bt.health = 300.0; }
            }
        }
        if hits.len() >= 4 { break; }
    }
    eprintln!("[bot-gate] {shots} shots; losses {hits:?}");
    assert!(!hits.is_empty(), "slot 0's rockets never hurt a bot");
    assert!(hits.iter().all(|&l| l <= 100.01), "a 0x2f rocket hurt a bot beyond its blast's 100: {hits:?}");
}
