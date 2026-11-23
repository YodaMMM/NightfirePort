//! mp-bot-hit-fx: online, a human's round on a bot shows the bot's flesh puff on BOTH machines.
//!
//! The game's bots are drone objects: a round's contact reaches `NDrone2_DealWithObjHit` 0x13fff0
//! case 5, which puffs `Gas_Init(3.0, 6.0, ..., 0x20008bc, 5, ...)` at the contact for a weapon
//! with damage [M :243-249] before the hit message reaches `BOT_handlePain` 0x125ed8 [M :306;
//! NDrone2_HitDamage 0x145558:15-17]. Two machines (nf_view::net_tick, arena 07000023, two humans
//! and two bots); slot 0 tracks the nearest live bot with the shot description and fires. On the
//! tick a bot's health drops, each machine holds a new blood puff at the bot, the two machines'
//! puffs sit at the same place, and the state hashes agree every tick (the puff is cosmetic: it
//! never touches the shared stream). NF_ISO-gated.

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

const BLOOD: u32 = 0x0200_08bc;

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
    scene
}

fn blood(sim: &nf_view::scene::Sim) -> Vec<[f32; 3]> {
    sim.drones.gas.puffs.iter().filter(|g| g.mesh == BLOOD).map(|g| g.pos).collect()
}

#[test]
fn online_human_round_on_a_bot_puffs_on_both_machines() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let w6 = 6u8;
    let mut seen = 0u32;
    for t in 0..1200u32 {
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
        row[0].held = w6;
        if let Some(tg) = target {
            row[0].shot = t % 12 == 0;
            row[0].shot_weapon = w6;
            row[0].shot_aim = nf_game::mp::combat::aim_from_dir([tg[0] - me[0], tg[1] - 0.2 - me[1], tg[2] - me[2]]);
        }
        let hp: Vec<(usize, f32)> = m.bots.iter().map(|bt| (bt.slot, bt.health)).collect();
        let (pa, pb) = (blood(a.sim.as_ref().unwrap()).len(), blood(b.sim.as_ref().unwrap()).len());
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the two machines' hashes diverged");
        let (sa, sb) = (a.sim.as_ref().unwrap(), b.sim.as_ref().unwrap());
        let hurt: Vec<[f32; 3]> = sa.mp.as_ref().unwrap().bots.iter()
            .filter(|bt| !bt.dead && hp.iter().any(|&(sl, h)| sl == bt.slot && bt.health < h))
            .map(|bt| bt.brain.pos).collect();
        if hurt.is_empty() { continue; }
        let (ba, bb) = (blood(sa), blood(sb));
        eprintln!("[bot-fx] t{t} bot hurt at {hurt:?}: blood puffs a {pa} -> {}, b {pb} -> {}", ba.len(), bb.len());
        assert!(ba.len() > pa && bb.len() > pb, "tick {t}: the hit spawned no flesh puff on one machine");
        let at = *ba.last().unwrap();
        assert_eq!(at, *bb.last().unwrap(), "tick {t}: the two machines' puffs sit apart");
        assert!(hurt.iter().any(|p| ((p[0] - at[0]).powi(2) + (p[2] - at[2]).powi(2)).sqrt() < 1.5), "tick {t}: the puff at {at:?} is not at the hit bot");
        seen += 1;
        if seen >= 3 { break; }
    }
    assert!(seen > 0, "slot 0's rounds never hurt a bot");
}
