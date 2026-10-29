//! mp-perf-16 (agent13): the 16-participant online tick's cost, and the proof that the
//! optimisations left the simulation bit-identical.
//!
//! Two machines (slot 0's and slot 1's, the live match-start setup) tick the SAME fixed input
//! rows (2 walking, shooting humans + 14 bots on 07000023) through `nf_view::net_tick` — the live
//! loop's tick shape. Every tick's hash must agree between the machines, and the fold of all the
//! hashes must equal the digest pinned from the code BEFORE the perf work (commit 420ecfe8): any
//! change to the simulated result (a cached LOS answer that went stale, a re-ordered route query,
//! a hash that skipped a field) changes the digest. The run also prints the per-tick wall time
//! (average / p99 / worst) — `NF_PROFILE=1` adds the per-section breakdown.
//! NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// The digest of the 3600-tick fixed run's per-tick hashes, measured on the pre-optimisation
/// code (420ecfe8 + profiling scopes only) [L this test, 2026-10-18].
// Re-pinned on the merged MP branch (agents 8/9/10/11/12/14 changed the simulated result on purpose:
// bot objectives, shared world interactions, shared rounds, bot brains in the hash; agent 14 arms
// the bots with the weapon set's first gun and carries the lobby's bot records); agent 13's
// optimisations were proven result-identical against its own base (0xe5ac_fd39_a20a_ac80), and the
// lazy sight line again on the merge (lazy_sight false + the precomputed bit2: the same digest).
// Re-pinned for bot-nav-pit-ramp (0x001c_313c_9be0_b2e1, kills 90 over 3600 ticks): the bots'
// follower is the game's `FollowRoute` link creep now (bots.rs `steer_to`/`follow_route`
// [M 0x15b110/0x159810/0x159f28/0x15a2c8]) — routes end at different ticks, bots die differently;
// the two machines' hashes still agree every tick.
// Re-pinned for net-desync-detail (700w2): the state hash is now the fixed-order fold of named component
// hashes, so its value changed while the simulation did not (the same run: kills 90, as bot-nav-pit-ramp's
// pin; repeatable).
// Re-pinned for mp-bot-weapons (776): the weapon layer's fire fix makes the bots actually fire —
// the port cleared the fire latch in the 0xD0/backoff combat arms (the game never writes +0x3b
// there [M 0x16f390/0x16fc30]), so every re-entry was a fresh `DroneWeap_Fire` whose
// `NextBulletTime` re-pushed the bullet deadline (≥ 60 ticks for weapon 6, [M 0x179d40:26-47])
// beyond the 1-in-3 exit roll's stay [M 0x16f5e8:48-52]: latch set, bots never fired. With the
// latch persisting the deadline survives and the bots shoot while they move (35→38 kills in the
// pinned 3600-tick run, the per-tick hashes still agree on both machines).
// Re-pinned for bot-weapons-finish-3 (0xe371_90cd_8fd7_4271, kills 36 over 3600 ticks): 776b's
// fire fix lands on the merged tree and the bots' sight lines follow the game's aim — the shot
// targets the opponent's position [M 0x125b30/0x178f68; the old −0.3 chest guess is gone], the
// scan's lazy sight runs the perception ack's own chain (segment + vis ≥ 0.7), and the site row
// lost its +0.3 compensation. Bots pick opponents a little more strictly (38→36 kills); the two
// machines' hashes still agree every tick.
// Re-pinned for mp-pickups-visible (700y1): the arena's set pickups now carry the match weapon set's guns
// [M Pickup_CreateFromSet 0x18f1b0] and spin, so the bots take different weapons — kills 36, repeatable; the two
// machines' per-tick hashes still agree.
// Re-pinned for standins-scene-930: an online bot's armour pickup now runs Pickup_Handler's class-2 case 3
// (BOT_vars+0xA4 / the armour byte += amount, capped 50 [M 0x18e650:119-133]) instead of the old +20 health,
// so the fights run differently — kills 37; the two machines' per-tick hashes still agree.
// bond-hitbox-pose-2: back to the master-lineage pin. The -pose branch re-pinned to
// 0xb270_68a8_2582_fec6 because its crouch toggle moved Bond::pos[1] −0.4847 (the [L]-backed
// settled root drop) and this run's scripted humans press crouch (`i.crouch = ph % 307 == 0`),
// so their roots rode the stance — its commit abb44f33 bisected the digest move to the
// crouch-pose commit. bond-hitbox-pose-2 moved the root offset out of `pos` into the hit-box
// palette (bond_boxes; the boxes do not feed this digest), so the sim positions are master's
// again and the digest returns to the pre-crouch-pose value 0x89cb_fcb6_ae45_c2ac — the same
// value master reproduces [the capsule-cull-11 report's master crosscheck].
const PINNED_DIGEST: u64 = 0x89cb_fcb6_ae45_c2ac;

const TICKS: u32 = 3600;

/// A walking, shooting player (the mp_bots_moving_live walk + a shot every 12 ticks on the
/// handgun in the middle third of every 600 ticks).
fn walk(t: u32, slot: usize) -> nf_net::Input {
    let mut i = NEUTRAL;
    let ph = t + slot as u32 * 97;
    i.forward = if ph % 240 < 200 { 1.0 } else { -0.5 };
    i.strafe = if ph % 150 < 30 { 0.8 } else if ph % 150 < 45 { -0.8 } else { 0.0 };
    i.turn = if ph % 180 < 60 { 0.03 } else if ph % 180 < 90 { -0.02 } else { 0.0 };
    i.look = if ph % 120 < 20 { 0.01 } else if ph % 120 < 40 { -0.01 } else { 0.0 };
    i.jump = ph % 211 == 0;
    i.crouch = ph % 307 == 0;
    i.held = 2;
    i.shot = (200..400).contains(&(t % 600)) && t % 12 == slot as u32;
    i
}

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 14;
    s.total_players = 16;
    s.seed = 0x0bed_0bad;
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

fn fold(d: u64, h: u64) -> u64 {
    (d ^ h).wrapping_mul(0x0000_0100_0000_01b3).rotate_left(7)
}

#[test]
fn sixteen_participants_fixed_run_digest_is_pinned() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    assert_eq!(a.sim.as_ref().unwrap().mp_bot_count(), 14);
    let mut digest = 0xcbf2_9ce4_8422_2325u64;
    let mut hashes = Vec::with_capacity(TICKS as usize);
    let mut ta = Vec::with_capacity(TICKS as usize);
    for t in 0..TICKS {
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|sl| if sl < 2 { walk(t, sl) } else { NEUTRAL });
        let t0 = std::time::Instant::now();
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        ta.push(t0.elapsed().as_secs_f64() * 1000.0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
        assert_eq!(ha, hb, "tick {t}: the two machines disagree");
        digest = fold(digest, ha);
        hashes.push(ha);
    }
    let sa = a.sim.as_ref().unwrap();
    let kills: i32 = sa.mp.as_ref().unwrap().game.slots.iter().map(|s| s.kills).sum();
    let mut sorted = ta.clone();
    sorted.sort_by(|x, y| x.total_cmp(y));
    let avg = ta.iter().sum::<f64>() / ta.len() as f64;
    eprintln!(
        "[perf16] {TICKS} ticks, 2 humans + 14 bots: net_tick avg {avg:.3} ms, p50 {:.3}, p99 {:.3}, max {:.3}; kills {kills}; digest {digest:#018x}",
        sorted[sorted.len() / 2], sorted[sorted.len() * 99 / 100], sorted[sorted.len() - 1]
    );
    assert!(kills > 0, "the run exercises the combat/death paths");
    // Sister 1: the digest sees every tick — dropping one tick's hash changes it.
    let skipped = hashes.iter().enumerate().filter(|(i, _)| *i != 1800).fold(0xcbf2_9ce4_8422_2325u64, |d, (_, h)| fold(d, *h));
    assert_ne!(skipped, digest);
    // Sister 2: a body's yaw is hashed (a hair's turn of one replica changes the tick hash).
    let sb = b.sim.as_mut().unwrap();
    let h0 = sb.net_state_hash(1);
    if let Some(r) = sb.mp_bonds[0].as_mut() { r.yaw += 1e-3; }
    assert_ne!(sb.net_state_hash(1), h0);
    if PINNED_DIGEST != 0 {
        assert_eq!(digest, PINNED_DIGEST, "the optimisations changed the simulated result");
    }
}

/// The early-out sight predicates answer exactly what the full nearest-hit searches' `is_none()`
/// answered, on the real arena: 20000 segments between points around the bot bodies' walk (the
/// fixed run's positions, jittered), long and short, blocked and clear.
#[test]
fn early_out_segments_match_the_full_search() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut pts: Vec<[f32; 3]> = Vec::new();
    for t in 0..900 {
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|sl| if sl < 2 { walk(t, sl) } else { NEUTRAL });
        nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
        if t % 30 == 0 { pts.extend(a.sim.as_ref().unwrap().mp_bot_positions()); }
    }
    let w = &a.sim.as_ref().unwrap().world;
    let mut x = 0x1234_5678u32;
    let mut rnd = || { x ^= x << 13; x ^= x >> 17; x ^= x << 5; x };
    let (mut blocked, mut blocked_b) = (0, 0);
    for _ in 0..20000 {
        let p = pts[rnd() as usize % pts.len()];
        let q = pts[rnd() as usize % pts.len()];
        let j = |r: u32| (r % 2000) as f32 / 1000.0 - 1.0;
        let from = [p[0] + j(rnd()), p[1] + j(rnd()), p[2] + j(rnd())];
        let to = [q[0] + j(rnd()), q[1] + j(rnd()) - 0.4, q[2] + j(rnd())];
        let full = w.segment(from, to).is_none();
        assert_eq!(w.segment_clear(from, to), full, "segment {from:?} -> {to:?}");
        assert_eq!(w.line_of_sight(from, to, nf_game::world::LOS_GUARD_VISION), full);
        let full_b = w.segment_surface_bullet(from, to).is_none();
        assert_eq!(w.segment_bullet_clear(from, to), full_b, "bullet segment {from:?} -> {to:?}");
        blocked += (!full) as u32;
        blocked_b += (!full_b) as u32;
    }
    eprintln!("[perf16] 20000 segments: {blocked} blocked (bullet class {blocked_b})");
    // Sister: the sample exercises both answers.
    assert!(blocked > 1000 && blocked < 19000, "blocked {blocked}");
}
