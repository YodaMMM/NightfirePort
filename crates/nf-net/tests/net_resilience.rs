//! net-internet: deterministic (shim, logical ms) checks of the session's internet-link
//! machinery — the ack that must flow when a peer has nothing new to send (the loopback
//! wedge), and the adaptive input delay: every machine switches at the same tick boundary,
//! a rise fills the gap without repeating a one-shot press, a fall carries the dropped
//! sample's presses into the next tick.

use std::net::SocketAddr;

use nf_net::pace::TickPacer;
use nf_net::session::{Config, Event, Session};
use nf_net::shim::{Shim, ShimEndpoint};
use nf_net::{Input, MatchSettings, NEUTRAL, MAX_AGENTS};

fn addr(port: u16) -> SocketAddr {
    format!("10.0.0.1:{port}").parse().unwrap()
}

fn cfg(adaptive: bool) -> Config {
    Config { hash_every: 15, adaptive_delay: adaptive, timeout_ms: 8000, ..Default::default() }
}

/// Handshake `clients` with the host over the shim (5 ms steps), bounded.
fn connect(shim: &Shim, host: &mut Session<ShimEndpoint>, clients: &mut [Session<ShimEndpoint>], lobby_ms: u64) -> u64 {
    let mut ms = 0u64;
    let mut settled = 0u64;
    loop {
        ms += 5;
        shim.advance(5);
        host.poll(ms);
        for c in clients.iter_mut() {
            c.poll(ms);
        }
        if clients.iter().all(|c| c.local_slot() != 0) {
            settled += 5;
            if settled >= lobby_ms {
                return ms;
            }
        }
        assert!(ms < 20_000, "handshake");
    }
}

/// The wedge's mechanism (loopback_e2e, 3–6 of 40 runs): a peer whose own inputs are all
/// acked has nothing new to send, and protocol 4's acks rode only on data packets — so once
/// its last ack-carrying packet was lost (or a reordered older one rewound the sender's view),
/// the sender never learned what it held. Here the client schedules ticks 0..=10 only and
/// then goes quiet; the host's view of the client's ack must still reach tick 10.
#[test]
fn an_idle_peer_still_acks() {
    let shim = Shim::new(41);
    shim.set_latency(10, 0);
    let mut host = Session::new_host(cfg(false), shim.endpoint(addr(1)), MatchSettings::default(), 9);
    let mut clients = vec![Session::new_client(cfg(false), shim.endpoint(addr(2)), addr(1))];
    let mut ms = connect(&shim, &mut host, &mut clients, 50);
    host.start_match(0).unwrap();
    for t in 0..=200 {
        host.local_input(t, NEUTRAL);
    }
    let mut c = clients.pop().unwrap();
    let (mut he, mut ce) = (0u32, 0u32);
    let mut scheduled = false;
    // Lose every client→host datagram for a stretch right after the client's last inputs
    // went out (its acks of the host's later relay die with them).
    for step in 0..400 {
        ms += 16;
        shim.advance(16);
        host.poll(ms);
        let evs = c.poll(ms);
        if !scheduled && (c.is_started() || evs.iter().any(|e| matches!(e, Event::MatchStarted { .. }))) {
            for t in 0..=10 {
                c.local_input(t, NEUTRAL);
            }
            scheduled = true;
        }
        while host.ready_inputs(he).is_some() {
            he += 1;
        }
        while c.ready_inputs(ce).is_some() {
            ce += 1;
        }
        if (20..40).contains(&step) {
            shim.drop_pending_to(addr(1), 64);
        }
    }
    assert_eq!(he, 11, "the host ran every tick the client scheduled");
    assert_eq!(ce, 11, "the client ran them too");
    let ack = host.peer_acks().into_iter().find(|(s, _)| *s == 1).and_then(|(_, a)| a);
    assert!(ack >= Some(10), "the idle client's ack reached the host: {ack:?}");
}

/// Per-machine state of the adaptive-delay run.
struct M {
    sess: Session<ShimEndpoint>,
    started: Option<u64>,
    executed: u32,
    pacer: TickPacer,
    /// The delay `submit_local` used for each run tick.
    used: Vec<u32>,
    /// Every executed tick's inputs.
    rows: Vec<[Input; MAX_AGENTS]>,
    /// Run ticks whose sample carried a press / a fired round.
    jumps: Vec<u32>,
    shots: Vec<u32>,
}

fn sample(slot: u8, run_tick: u32) -> Input {
    let mut i = NEUTRAL;
    i.forward = 1.0;
    i.turn = 0.001 * (slot as f32 + 1.0);
    i.jump = run_tick % 37 == slot as u32;
    i.shot = run_tick % 41 == 20 + slot as u32;
    i.held = 2;
    // A one-shot lane on EVERY sample: a fill tick must add 0, a carried sample must add its
    // 1 to the next — the executed sum equals the sample count exactly.
    i.weapon_cycle = 1;
    i
}

/// The adaptive delay over a link whose latency changes: 20 ms ping, then 160 ms, then 20 ms
/// again. Every machine must use the same delay for every run tick (the announced boundary),
/// the delay must rise and fall, the lockstep must agree, and every press / fired round must
/// land exactly once (the rise's fill never repeats one, the fall's drop carries it).
#[test]
fn adaptive_delay_agreed_at_tick_boundaries() {
    let shim = Shim::new(77);
    shim.set_latency(10, 2);
    let mut host = Session::new_host(cfg(true), shim.endpoint(addr(11)), MatchSettings::default(), 5);
    let mut cl: Vec<Session<ShimEndpoint>> =
        (0..2).map(|i| Session::new_client(cfg(true), shim.endpoint(addr(12 + i)), addr(11))).collect();
    let mut ms = connect(&shim, &mut host, &mut cl, 1000);
    host.start_match(0).unwrap();
    let mut ms_list = vec![M { sess: host, started: Some(ms), executed: 0, pacer: TickPacer::new(), used: vec![], rows: vec![], jumps: vec![], shots: vec![] }];
    for c in cl {
        ms_list.push(M { sess: c, started: None, executed: 0, pacer: TickPacer::new(), used: vec![], rows: vec![], jumps: vec![], shots: vec![] });
    }
    let prefill = |m: &mut M| {
        let lead = m.sess.input_delay();
        for t in 0..lead {
            m.sess.local_input(t, NEUTRAL);
        }
    };
    prefill(&mut ms_list[0]);
    let t0 = ms;
    let end = t0 + 34_000;
    while ms < end {
        ms += 1;
        shim.advance(1);
        let phase = ms - t0;
        if phase == 6000 {
            shim.set_latency(80, 6);
        } else if phase == 16_000 {
            shim.set_latency(10, 2);
        }
        if ms % 17 != 0 {
            continue; // ~60 Hz frames
        }
        for m in ms_list.iter_mut() {
            for e in m.sess.poll(ms) {
                match e {
                    Event::MatchStarted { .. } => {
                        m.started = Some(ms);
                        let lead = m.sess.input_delay();
                        for t in 0..lead {
                            m.sess.local_input(t, NEUTRAL);
                        }
                    }
                    Event::Desync { tick, .. } => panic!("desync at {tick}"),
                    _ => {}
                }
            }
            let Some(st) = m.started else { continue };
            let target = ((ms - st) * 60 / 1000) as u32;
            let n = m.pacer.ticks_for_frame(target, m.executed, 0.017, 4);
            for _ in 0..n {
                let Some(row) = m.sess.ready_inputs(m.executed) else { break };
                let slot = m.sess.local_slot();
                let s = sample(slot, m.executed);
                if s.jump {
                    m.jumps.push(m.executed);
                }
                if s.shot {
                    m.shots.push(m.executed);
                }
                m.used.push(m.sess.delay_at(m.executed));
                m.sess.submit_local(m.executed, s);
                m.rows.push(row);
                m.executed += 1;
            }
            m.sess.flush(ms);
        }
    }
    let common = ms_list.iter().map(|m| m.rows.len()).min().unwrap();
    assert!(common > 1500, "the match ran ({common} ticks)");
    for m in ms_list.iter().skip(1) {
        assert_eq!(&m.rows[..common], &ms_list[0].rows[..common], "lockstep inputs agree");
        assert_eq!(&m.used[..common], &ms_list[0].used[..common], "every machine used the same delay at every run tick");
    }
    let used = &ms_list[0].used[..common];
    let lo = *used[..300].iter().max().unwrap();
    let hi = *used.iter().max().unwrap();
    let last = used[common - 1];
    assert!(hi >= lo + 4, "the delay rose with the 160 ms ping: {lo} -> {hi}");
    assert!(last < hi, "and fell again when the ping dropped: {hi} -> {last}");
    eprintln!("delays used: start {lo}, peak {hi}, end {last}; changes at run ticks {:?}",
        (1..common).filter(|t| used[*t] != used[*t - 1]).map(|t| (t, used[t])).collect::<Vec<_>>());
    // One-shots: each machine's presses land exactly once in its slot's executed stream.
    for m in ms_list.iter() {
        let slot = m.sess.local_slot() as usize;
        let horizon = common as u32;
        let landed_jumps: Vec<u32> = (0..horizon).filter(|t| ms_list[0].rows[*t as usize][slot].jump).collect();
        let landed_shots: Vec<u32> = (0..horizon).filter(|t| ms_list[0].rows[*t as usize][slot].shot).collect();
        let sure = |v: &Vec<u32>| v.iter().filter(|r| **r + 20 < horizon).count();
        let all = |v: &Vec<u32>| v.iter().filter(|r| **r < horizon).count();
        assert!(landed_jumps.len() >= sure(&m.jumps) && landed_jumps.len() <= all(&m.jumps),
            "slot {slot}: every press lands once ({} landed, {}..={} sampled)", landed_jumps.len(), sure(&m.jumps), all(&m.jumps));
        assert!(landed_jumps.windows(2).all(|w| w[1] - w[0] >= 20), "slot {slot}: no press repeated: {landed_jumps:?}");
        assert!(landed_shots.len() >= sure(&m.shots) && landed_shots.len() <= all(&m.shots),
            "slot {slot}: every fired round lands once ({} landed, {}..={} sampled)", landed_shots.len(), sure(&m.shots), all(&m.shots));
        assert!(landed_shots.windows(2).all(|w| w[1] - w[0] >= 20), "slot {slot}: no round repeated: {landed_shots:?}");
        // Every sample's one-shot lane: the run ticks 0..R sampled R of them; they land on
        // ticks up to R + delay. Count the executed sum over the ticks whose samples are all
        // in (every tick up to `horizon` came from run ticks < horizon): it equals the number
        // of run ticks whose sample landed below `horizon`.
        let sum: i64 = (0..horizon).map(|t| ms_list[0].rows[t as usize][slot].weapon_cycle as i64).sum();
        let lead_in = ms_list[0].rows.iter().take(horizon as usize).position(|r| r[slot].weapon_cycle != 0).unwrap_or(0) as u32;
        let sampled_landed = (0..horizon).filter(|r| r + m.used[*r as usize] < horizon).count() as i64;
        assert!((sum - sampled_landed).abs() <= 1,
            "slot {slot}: the one-shot lane sums to the samples that landed (sum {sum}, samples {sampled_landed}, lead-in {lead_in})");
    }
}
