//! mp-net-slots-post-move (GAPS G-053 item 4): every active slot takes its post-move
//! position in the same place, with no fixed 4. The live online tick and
//! `nf_view::net_tick` write all of `0..MAX_AGENTS` after the movement
//! (lib.rs:4721-4726, lib.rs:509-515 [M code]) — the fixed-4 writer the GAPS item
//! described (net-match-host's `for sl in 0..4`, still there at mp-16-slots' e4f7cbf7)
//! left at 30a3cf69 (docs/systems/net-slot-positions.md §3). What master still lacked
//! is the proof through the real path: a 6-human loopback session whose machines agree
//! on every tick while slot 5 moves, each human slot's match record equal to its
//! body's post-move position on BOTH machines every tick, and the sister 4-human
//! digest pin. Harness shape: mp_16_slots_live's direct `Session`s over 127.0.0.1 —
//! 2 simmed machines (one arena load is ~875 MB measured; 4 concurrent loads OOM a
//! 3 GB test machine — as in mp_16_slots_live), the other four sessions still feed real inputs and
//! gate every `ready_inputs`; the replay pass then runs each remaining local slot alone
//! over the recorded rows through the same `nf_view::net_tick` (the app's own
//! late-join verification shape, `mp_late_join_catchup`) so all six machines' hashes
//! are checked every tick within the RAM. NF_ISO-gated (the arena loads from the disc).

use std::time::{Duration, Instant};

use nf_game::mp::MpSettings;
use nf_net::net::UdpTransport;
use nf_net::session::{Event, Session};
use nf_net::{MatchSettings, NEUTRAL};

use nf_view::mpnet::mp_to_match;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// Distinct per test binary: the binaries' tests run in parallel threads.
const PORT_6: u16 = 28341;
const PORT_4: u16 = 28351;
const ARENA: u32 = 0x0700_004B; // MP_MAP_ROWS[7] = Ravine [D placement] — the lightest multi-scene arena
const TICKS: u32 = 240;
const SEED: u32 = 0x0BED_0BAD;

/// Every human moves every tick (mp_16_slots_live's script — forward, strafe, turn,
/// look, jump, crouch all cycle, so a slot's position never stands still for long).
fn scripted_input(t: u32, slot: u8) -> nf_net::Input {
    let mut i = NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 89) as f32 / 89.0;
    i.strafe = if t % 13 == 0 { -0.4 } else { 0.0 };
    i.turn = ((t % 17) as f32 - 8.0) * 0.002;
    i.look = if t % 31 == 0 { 0.2 } else { 0.0 };
    i.jump = t % 67 == 0;
    i.crouch = t % 47 == 0;
    i
}

fn cfg() -> nf_net::Config {
    nf_net::Config { hash_every: 15, ..Default::default() }
}

struct Peer {
    sess: Session<UdpTransport>,
}

/// Bring up `humans` real sessions over 127.0.0.1 (the host listens on `port`, the
/// clients dial one at a time so the join order is stable), hand out the host's match
/// (all humans, no bots), and load the sims for `sim_slots` — one arena load at a time
/// (the test machine's RAM), each with its per-machine generator (the online start seeds it
/// by slot) and its own slot swapped in as the local body. Returns the settings, the
/// peers (peers[0] is the host) and the (slot, scene) pairs.
fn bring_up(
    path: &str,
    port: u16,
    humans: usize,
    sim_slots: &[usize],
) -> (MpSettings, Vec<Peer>, Vec<(usize, nf_view::scene::Scene)>) {
    let mut ms: u64 = 0;
    let mut peers = vec![Peer {
        sess: Session::new_host(cfg(), UdpTransport::bind(format!("127.0.0.1:{port}").as_str()).unwrap(),
            MatchSettings::default(), SEED),
    }];
    // The clients dial one at a time; the host's agent count is the join book.
    for _ in 1..humans {
        let c = Peer {
            sess: Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
                format!("127.0.0.1:{port}").parse().unwrap()),
        };
        peers.push(c);
        let deadline = Instant::now() + Duration::from_secs(20);
        // The host sees every machine (agents() includes self) and the new client sees
        // the host — the next dial starts only after this one is fully welcomed.
        while peers[0].sess.agents().len() < peers.len() || peers.last().unwrap().sess.agents().len() < 2 {
            ms += 5;
            for p in peers.iter_mut() { let _ = p.sess.poll(ms); }
            assert!(Instant::now() < deadline, "handshake: {} machines up", peers.len());
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    // Every human slot is taken exactly once (the bodies and the inputs are per SLOT).
    let mut slots: Vec<u8> = peers.iter().map(|p| p.sess.local_slot()).collect();
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(slots.len(), humans, "each of the {humans} humans holds its own slot");
    assert_eq!(slots.first().copied(), Some(0), "the host is slot 0");
    assert_eq!(*slots.last().unwrap() as usize, humans - 1, "the slots cover 0..{humans}");

    let mut s = MpSettings::deathmatch(ARENA);
    s.human_count = humans as u32;
    s.player_count = humans as u16;
    s.total_players = humans as u32;
    s.seed = SEED;
    peers[0].sess.set_host_match(mp_to_match(&s), s.seed);
    peers[0].sess.start_match(0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while peers.iter().skip(1).any(|p| !p.sess.is_started()) {
        ms += 5;
        for p in peers.iter_mut() { let _ = p.sess.poll(ms); }
        assert!(Instant::now() < deadline, "GO reaches all {humans}");
        std::thread::sleep(Duration::from_millis(1));
    }
    // The sims, one load at a time (the RAM bound).
    let mut simmed = Vec::new();
    for &slot in sim_slots {
        let arena = format!("{ARENA:08x}.bin");
        let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone())
            .expect("the arena loads");
        {
            let sim = scene.sim.as_mut().expect("the match sim");
            sim.swap_local_slot(slot);
            sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
        }
        simmed.push((slot, scene));
    }
    (s, peers, simmed)
}

/// A slot's body position on this machine (the local slot's body is `bond`).
fn body_pos(sim: &nf_view::scene::Sim, s: usize) -> [f32; 3] {
    if s == sim.mp_local_slot { sim.bond.pos } else { sim.mp_bonds[s].as_ref().expect("the slot's body").pos }
}

/// The GAPS G-053 property, per machine, per tick: every active human slot's match
/// record equals its body's position — the body's POST-move position, written for
/// every slot in one place (lib.rs:4721-4726 / lib.rs:509-515 [M code]). On the
/// fixed-4 writer this fails for any slot past the first four on every machine that
/// does not own the slot, from the first tick the body moves.
fn assert_match_records_track_bodies(sim: &nf_view::scene::Sim, humans: usize, what: &str) {
    let m = sim.mp.as_ref().expect("the match");
    for s in 0..humans {
        assert_eq!(m.game.slots[s].pos, Some(body_pos(sim, s)),
            "{what}: slot {s}'s match record is not its body's post-move position");
    }
}

/// 6 humans over 127.0.0.1 (real sessions): the host's machine and slot 5's machine
/// tick the same rows through `nf_view::net_tick` (with `net_rng` set, the live
/// start's per-machine generator) — the hashes agree on EVERY tick, every human
/// slot's match record equals its body's post-move position on BOTH machines every
/// tick, slot 5 really travels, and no session reports a desync. The remaining four
/// locals replay the recorded rows alone through the same tick and must reproduce the
/// recorded hash at every tick — all six machines' states and hashes agree.
#[test]
fn six_human_loopback_slot5_moves_hashes_agree_every_tick() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    const HUMANS: usize = 6;
    let (s, mut peers, mut simmed) = bring_up(&path, PORT_6, HUMANS, &[0, 5]);
    assert_eq!(simmed[0].0, 0, "the host's sim is slot 0's machine");
    assert_eq!(simmed[1].0, 5, "the second sim is slot 5's machine (a local past the first four)");

    let mut ms: u64 = 0;
    let mut log_rows: Vec<[nf_net::Input; nf_net::MAX_AGENTS]> = Vec::with_capacity(TICKS as usize);
    let mut log_hash: Vec<u64> = Vec::with_capacity(TICKS as usize);
    let mut desyncs: Vec<String> = Vec::new();
    let mut first_pos5 = [0.0f32; 3];
    for t in 0..TICKS {
        for p in peers.iter_mut() {
            let slot = p.sess.local_slot();
            p.sess.local_input(t, scripted_input(t, slot));
        }
        // Pump until every one of the 6 sessions can run tick t (the lockstep never guesses).
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            ms += 5;
            for p in peers.iter_mut() { let _ = p.sess.poll(ms); }
            if peers.iter_mut().all(|p| p.sess.ready_inputs(t).is_some()) { break; }
            assert!(Instant::now() < deadline, "tick {t} readiness (all {HUMANS} humans)");
            std::thread::sleep(Duration::from_millis(1));
        }
        let rows = peers[0].sess.ready_inputs(t).expect("host ready");
        // The lockstep handed every machine the same row.
        for (k, p) in peers.iter_mut().enumerate().skip(1) {
            let r = p.sess.ready_inputs(t).expect("client ready");
            assert_eq!(&rows[..HUMANS], &r[..HUMANS], "tick {t}: machine {k}'s input row differs");
        }
        let slot_a = simmed[0].0;
        let slot_b = simmed[1].0;
        let h0 = nf_view::net_tick(simmed[0].1.sim.as_mut().unwrap(), &rows, slot_a);
        let h5 = nf_view::net_tick(simmed[1].1.sim.as_mut().unwrap(), &rows, slot_b);
        assert_eq!(h0, h5, "tick {t}: slot 5's machine's hash disagrees with the host's");
        assert_match_records_track_bodies(simmed[0].1.sim.as_ref().unwrap(), HUMANS, "host");
        assert_match_records_track_bodies(simmed[1].1.sim.as_ref().unwrap(), HUMANS, "slot 5's machine");
        if t == 0 { first_pos5 = body_pos(simmed[1].1.sim.as_ref().unwrap(), 5); }
        peers[0].sess.submit_hash(t, h0);
        for p in peers.iter_mut().skip(1) {
            if p.sess.local_slot() == 5 { p.sess.submit_hash(t, h5); }
        }
        for p in peers.iter_mut() {
            desyncs.extend(p.sess.poll(ms).into_iter().filter_map(|e| match e {
                Event::Desync { tick, .. } => Some(format!("tick {tick}")),
                _ => None,
            }));
        }
        log_rows.push(rows);
        log_hash.push(h0);
    }
    assert!(desyncs.is_empty(), "no desync over the honest 6-human run: {desyncs:?}");
    // The property is not vacuous: slot 5 really left its spawn.
    let end_pos5 = body_pos(simmed[1].1.sim.as_ref().unwrap(), 5);
    let travel = ((end_pos5[0] - first_pos5[0]).powi(2) + (end_pos5[2] - first_pos5[2]).powi(2)).sqrt();
    assert!(travel > 5.0, "slot 5 moved only {travel} over {TICKS} ticks — the input script stalled");
    eprintln!("[slots] 6 humans {TICKS} ticks: slot 5 travelled {travel:.1}, final hash {:016x}", log_hash[TICKS as usize - 1]);

    // The other four machines, one resident sim at a time: each replays the exact rows
    // through the same `nf_view::net_tick` (the late-join replay IS a machine —
    // mp_late_join_catchup) and must land on the recorded hash every tick.
    drop(simmed);
    for k in 1..HUMANS - 1 {
        let arena = format!("{:08x}.bin", s.level_code);
        let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s.clone())
            .expect("the replay machine loads");
        {
            let sim = scene.sim.as_mut().expect("the replay sim");
            sim.swap_local_slot(k);
            sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (k as u32).wrapping_mul(0x9e37_79b9)));
        }
        for t in 0..TICKS {
            let h = nf_view::net_tick(scene.sim.as_mut().unwrap(), &log_rows[t as usize], k);
            assert_eq!(h, log_hash[t as usize],
                "replayed machine {k} tick {t}: its hash disagrees with the live run's");
        }
        eprintln!("[slots] replayed machine {k}: {TICKS} ticks agree with the live run");
    }
}

/// Sister: the 4-human session's digest is pinned. All four slots sit inside the old
/// fixed-4 window, so this shape never had the defect — the digest must not move
/// while the 16-slot rule is (re)proved around it. Pinned on the pre-task tree
/// (master e5638828); a mismatch means the 4-human path moved: re-pin consciously,
/// with the reason in the report.
///
/// RE-PINNED (capsule-cull-and-los-masks-6): the final digest moved when the game's
/// head-on cull went unconditional [M Intersect_CylGeom 0x109da0:321-322 — the skip
/// `((0.999 < n·d̂) && (0.5 < f)) || ((n·d̂ < -0.999) && (f < 0.5))`, f the plane-crossing
/// fraction from A clamped :310-320, the spine NORMALIZED before the loop :109f74
/// Vec_Normalise 0x1e3000, walk A = HITTEST+0x20 = BLData+0x760 = pos + up·0.275 (the TOP),
/// B = HITTEST+0x30 = BLData+0x770 = pos + up·(0.55 − cc) [M Player_Collision 0x194df0
/// decompile :301/:333]]. The old final (0xfb7b…) was pinned on a tree whose capsule
/// contact LACKED that cull — the pre-cull tree pushed bodies the game passes through,
/// so the old expectation baked in a non-game rule and the TEST was wrong (fixed here,
/// per the task rule, with the address). Bisect evidence (mp_net_slots_post_move_live
/// runs, NF_NET_HASHDBG): 82362142 (batch-1026d) GREEN; 24af5915 (the cull's flag flip,
/// first capsule commit) already reads 0xbc5e_2155_a378_6e9c; with the cull's const
/// flipped back OFF on that same tree the pin is GREEN again — the cull is the sole
/// mover; the
/// LOS masks (the same commit's other flip, unconditional now) are digest-inert here; the
/// 4a155ddf merge's master-side content and its resolution are exonerated (4a155ddf^1 =
/// ca2e7389, capsule-3 tip, reads the same 0xbc5e… before any of that content).
/// First divergence: session t142, component bodies, slot 3 only — x +0.00122,
/// y −0.00235, z/yaw/pitch/substate/crouched identical (slot 3 is airborne off the
/// t134 scripted jump, pressed against the z −206.70874 wall); tick0 is UNCHANGED.
#[test]
fn four_human_session_digest_is_pinned() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    // The measured digests: tick0 unchanged from the original pin (master e5638828);
    // final re-pinned for the head-on cull (see the block above).
    const PINNED_TICK0: u64 = 0x5c0b_b60a_8cdf_8938;
    const PINNED_FINAL: u64 = 0xbc5e_2155_a378_6e9c;
    const HUMANS: usize = 4;
    let (_s, mut peers, mut simmed) = bring_up(&path, PORT_4, HUMANS, &[0, 2]);
    let mut ms: u64 = 0;
    let mut first = 0u64;
    let mut last = 0u64;
    for t in 0..TICKS {
        for p in peers.iter_mut() {
            let slot = p.sess.local_slot();
            p.sess.local_input(t, scripted_input(t, slot));
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            ms += 5;
            for p in peers.iter_mut() { let _ = p.sess.poll(ms); }
            if peers.iter_mut().all(|p| p.sess.ready_inputs(t).is_some()) { break; }
            assert!(Instant::now() < deadline, "tick {t} readiness (all {HUMANS} humans)");
            std::thread::sleep(Duration::from_millis(1));
        }
        let rows = peers[0].sess.ready_inputs(t).expect("host ready");
        let slot_a = simmed[0].0;
        let slot_b = simmed[1].0;
        let h0 = nf_view::net_tick(simmed[0].1.sim.as_mut().unwrap(), &rows, slot_a);
        let h2 = nf_view::net_tick(simmed[1].1.sim.as_mut().unwrap(), &rows, slot_b);
        assert_eq!(h0, h2, "tick {t}: the 4-human machines disagree");
        assert_match_records_track_bodies(simmed[0].1.sim.as_ref().unwrap(), HUMANS, "host");
        assert_match_records_track_bodies(simmed[1].1.sim.as_ref().unwrap(), HUMANS, "slot 2's machine");
        peers[0].sess.submit_hash(t, h0);
        for p in peers.iter_mut().skip(1) {
            if p.sess.local_slot() == 2 { p.sess.submit_hash(t, h2); }
        }
        if t == 0 { first = h0; }
        last = h0;
    }
    eprintln!("[slots] 4-human digest: tick0 {first:#018x} final {last:#018x}");
    assert_eq!(first, PINNED_TICK0, "the 4-human session's tick-0 digest moved (measured {first:#018x})");
    assert_eq!(last, PINNED_FINAL, "the 4-human session's final digest moved (measured {last:#018x})");
}
