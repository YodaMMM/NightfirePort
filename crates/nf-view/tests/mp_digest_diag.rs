//! capsule-cull-and-los-masks-5 diagnostic: the 4-human digest pin's per-tick COMPONENT
//! stream. The same session shape as mp_net_slots_post_move_live's
//! `four_human_session_digest_is_pinned` (same ARENA/SEED/TICKS/script/harness), but every
//! tick prints the 7 named component hashes (`net_tick_comps`) so a baseline-vs-tree diff
//! names the first divergent component and tick directly (the pin's dump only covers the
//! bodies/rng/match fields; the moved field is in the un-dumped set). Every 20th tick also
//! prints `net_world_fields` (the world component's field-by-field text). Not a pin: the
//! assertions are the session's own consistency (the two simmed machines agree per tick).

use std::time::{Duration, Instant};

use nf_game::mp::MpSettings;
use nf_net::net::UdpTransport;
use nf_net::session::{Event, Session};
use nf_net::{MatchSettings, NEUTRAL};

use nf_view::mpnet::mp_to_match;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

const PORT_4: u16 = 28361; // distinct from the pin binary's 28351 (parallel binaries)
const ARENA: u32 = 0x0700_004B; // MP_MAP_ROWS[7] = Ravine [D placement]
const TICKS: u32 = 240;
const SEED: u32 = 0x0BED_0BAD;

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
    for _ in 1..humans {
        let c = Peer {
            sess: Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
                format!("127.0.0.1:{port}").parse().unwrap()),
        };
        peers.push(c);
        let deadline = Instant::now() + Duration::from_secs(20);
        while peers[0].sess.agents().len() < peers.len() || peers.last().unwrap().sess.agents().len() < 2 {
            ms += 5;
            for p in peers.iter_mut() { let _ = p.sess.poll(ms); }
            assert!(Instant::now() < deadline, "handshake: {} machines up", peers.len());
            std::thread::sleep(Duration::from_millis(1));
        }
    }
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

/// The component stream of the host machine (slot 0) over the same 240-tick session the
/// pin asserts. Output: one `diag]` line per tick with the 7 component hashes in the fixed
/// order (frame_rng bodies vitals combat match bots world), plus the world fields every
/// 20th tick.
#[test]
fn four_human_component_stream() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let HUMANS = 4usize;
    let (_s, mut peers, mut simmed) = bring_up(&path, PORT_4, HUMANS, &[0, 2]);
    let mut ms: u64 = 0;
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
        peers[0].sess.submit_hash(t, h0);
        for p in peers.iter_mut().skip(1) {
            if p.sess.local_slot() == 2 { p.sess.submit_hash(t, h2); }
        }
        eprintln!("diag] t {t} {h0:016x}");
    }
    eprintln!("[diag] done");
}
