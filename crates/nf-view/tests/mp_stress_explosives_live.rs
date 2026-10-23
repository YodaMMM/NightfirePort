//! mp-stress-explosives: lockstep sync must hold under rapid-fire rockets.
//!
//! The worst case the wire allows: a shot is a description in the input row (`Input::shot` +
//! `shot_weapon` + `shot_aim`) that every machine simulates, so a player can send one EVERY tick, past
//! any launcher's own fire rate. 8 humans + 8 bots on 07000023; every human walks and fires an
//! explosive every tick (unguided rockets 0x2D, guided 0x2C, MGL grenades 0x2A, frags 0x34, remote
//! mines 0x37 + the detonator 0x38), aimed around the compass and up/down. Three machines tick the
//! same rows through `nf_view::net_tick` (the live loop's tick shape): slot 0's, slot 1's and slot 5's
//! (a local slot past the first four — GAPS G-053 item 4). Every tick's hash must agree on all three.
//! The run prints the peak live rounds / thrown objects / blasts and the per-tick cost.
//! NF_ISO-gated (the arena loads from the disc). NF_STRESS_TICKS overrides the length.

use nf_game::mp::combat as cb;
use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

const N: usize = nf_net::MAX_AGENTS;
const HUMANS: usize = 8;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = HUMANS as u32;
    s.player_count = HUMANS as u16;
    s.bot_count = 8;
    s.total_players = 16;
    s.seed = 0x0bed_0bad;
    s.time_limit = 60 * 60;
    s.frag_limit = -1;
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

const KIT: [u8; 5] = [0x2d, 0x2c, 0x2a, 0x34, 0x37];

/// Slot `slot`'s input on tick `t`: walking and turning, and an explosive every tick.
fn spam(t: u32, slot: usize) -> nf_net::Input {
    let mut i = NEUTRAL;
    let ph = t + slot as u32 * 97;
    i.forward = if ph % 240 < 200 { 1.0 } else { -0.5 };
    i.strafe = if ph % 150 < 30 { 0.8 } else if ph % 150 < 45 { -0.8 } else { 0.0 };
    i.turn = if ph % 180 < 60 { 0.03 } else if ph % 180 < 90 { -0.02 } else { 0.0 };
    i.jump = ph % 211 == 0;
    i.crouch = ph % 307 == 0;
    // Each human cycles the kit every 300 ticks; every 50th tick a mine thrower presses the detonator.
    let w = if t % 50 == 49 { 0x38 } else { KIT[((t / 300) as usize + slot) % KIT.len()] };
    let a = (t as f32 * 0.37 + slot as f32 * 0.79) % std::f32::consts::TAU;
    let up = ((t as f32 * 0.11 + slot as f32).sin()) * 0.6;
    i.held = w;
    i.shot = true;
    i.shot_weapon = w;
    i.shot_aim = cb::aim_from_dir([a.sin(), up, a.cos()]);
    i
}

#[test]
fn eight_humans_spamming_explosives_every_tick_stay_in_sync() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let ticks: u32 = std::env::var("NF_STRESS_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(7200);
    let s = settings();
    let locals = [0usize, 1, 5];
    let mut ms: Vec<_> = locals.iter().map(|&l| machine(&path, &s, l)).collect();
    assert_eq!(ms[0].sim.as_ref().unwrap().mp_bot_count(), 8);
    let (mut peak_rounds, mut peak_thrown, mut blasts_seen) = (0usize, 0usize, 0usize);
    let mut cost = Vec::with_capacity(ticks as usize);
    for t in 0..ticks {
        let row: [nf_net::Input; N] = std::array::from_fn(|sl| if sl < HUMANS { spam(t, sl) } else { NEUTRAL });
        let t0 = std::time::Instant::now();
        let h0 = nf_view::net_tick(ms[0].sim.as_mut().unwrap(), &row, locals[0]);
        cost.push(t0.elapsed().as_secs_f64() * 1000.0);
        for k in 1..ms.len() {
            let hk = nf_view::net_tick(ms[k].sim.as_mut().unwrap(), &row, locals[k]);
            assert_eq!(h0, hk, "tick {t}: machine {} (local slot {}) disagrees with slot 0's", k, locals[k]);
        }
        let c = &ms[0].sim.as_ref().unwrap().mp_combat;
        peak_rounds = peak_rounds.max(c.rounds.len());
        peak_thrown = peak_thrown.max(c.thrown.len());
        blasts_seen += c.blasts.len();
    }
    let sa = ms[0].sim.as_ref().unwrap();
    let m = sa.mp.as_ref().unwrap();
    let kills: i32 = m.game.slots.iter().map(|s| s.kills).sum();
    let deaths: i32 = m.game.slots.iter().map(|s| s.deaths).sum();
    let mut sorted = cost.clone();
    sorted.sort_by(|x, y| x.total_cmp(y));
    let avg = cost.iter().sum::<f64>() / cost.len() as f64;
    eprintln!(
        "[stress] {ticks} ticks, {HUMANS} humans + 8 bots, an explosive per human per tick: peak rounds {peak_rounds}, peak thrown {peak_thrown}, blasts {blasts_seen}, kills {kills}, deaths {deaths}; net_tick avg {avg:.3} ms, p99 {:.3}, max {:.3}",
        sorted[sorted.len() * 99 / 100], sorted[sorted.len() - 1]
    );
    // mp-shot-gate: the gate applies each slot's shot only at the launcher's real rate with the
    // shared rounds ([M] Player_WeaponFiring 0x1a5460, Player_RoundToFire 0x1a18e0), so the
    // per-tick spam now arrives gated (peak rounds 6, blasts 384, deaths 300 on the pinned run)
    // — the storm is the game's rate, and the machines still agree every tick.
    assert!(peak_rounds >= 5 && blasts_seen > 100 && deaths > 0, "the run really is an explosives storm (peak {peak_rounds}, blasts {blasts_seen}, deaths {deaths})");
    // Sister: a live round is in the hash — nudging one rocket on one machine shows at once.
    let sb = ms[1].sim.as_mut().unwrap();
    let h = sb.net_state_hash(locals[1]);
    if let Some(r) = sb.mp_combat.rounds.first_mut() {
        r.pos[0] += 1e-3;
        assert_ne!(sb.net_state_hash(locals[1]), h, "a rocket's position is hashed");
    }
}
