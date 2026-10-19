//! Loopback late-join tests (mp-join-in-progress).
//!
//! Deterministic throughout: logical milliseconds, the seeded shim network
//! (loss + latency — the transfer must survive a real lossy link), fixed
//! 60 Hz steps. The "sim" is the same fake deterministic hash chain the
//! lockstep tests use: the late joiner's replay hash is the chain over the
//! received log, so a bit-flip anywhere in the transfer cannot verify.

use std::net::SocketAddr;

use nf_net::net::Transport;
use nf_net::shim::{Shim, ShimEndpoint};
use nf_net::session::{Config, Event, LeaveReason, RefuseReason, Session};
use nf_net::{Input, MatchSettings, NEUTRAL};

fn addr(port: u16) -> SocketAddr {
    format!("127.0.0.1:{port}").parse().unwrap()
}

fn test_cfg() -> Config {
    Config {
        build_id: 77,
        input_delay: 3,
        redundancy: 4,
        hash_every: 30,
        timeout_ms: 5000,
        keepalive_ms: 100,
        hello_retry_ms: 50,
        resend_ms: 20,
        ..Default::default()
    }
}

/// Deterministic per-agent "controls" for (slot, tick).
fn agent_input(slot: u8, tick: u32) -> Input {
    let mut inp = NEUTRAL;
    let x = (slot as u32).wrapping_mul(0x9E37_79B9) ^ tick.wrapping_mul(0x85EB_CA6B);
    inp.forward = ((x & 0xFF) as i32 - 128) as f32 / 128.0;
    inp.turn = (((x >> 8) & 0xFF) as i32 - 128) as f32 / 128.0;
    inp.fire = x & 0x100 != 0;
    inp.jump = x & 0x200 != 0;
    inp.weapon_cycle = (x & 0x0F) as i8 - 8;
    inp
}

/// The fake deterministic sim hash (the lockstep tests' chain): same inputs
/// on every machine = same hash; a one-bit difference flips it.
fn sim_hash(tick: u32, inputs: &[Input; 4], prev: u64) -> u64 {
    let mut h = prev ^ 0xC0FF_EE00;
    for (s, i) in inputs.iter().enumerate() {
        h = h.rotate_left(17) ^ (i.forward.to_bits() as u64).wrapping_add(s as u64 + 1);
        h = h.rotate_left(13) ^ (i.fire as u64).wrapping_mul(31);
        h = h.rotate_left(7) ^ (i.weapon_cycle as u8 as u64);
        h = h.rotate_left(11) ^ ((i.jump as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    }
    h ^ (tick as u64)
}

/// One machine's fake-sim bookkeeping: the next tick to run and the hash
/// chain. The host records what it runs; the joiner replays the received log
/// through the SAME chain, so `prev_hash` agrees everywhere by construction.
struct Sim {
    next_tick: u32,
    prev_hash: u64,
    marks: Vec<(u32, u64)>,
}

impl Sim {
    fn new() -> Sim {
        Sim { next_tick: 0, prev_hash: 0, marks: Vec::new() }
    }
    /// Run `tick` with `inputs`: fold the chain, return the hash.
    fn run(&mut self, tick: u32, inputs: &[Input; 4]) -> u64 {
        assert_eq!(self.next_tick, tick, "ticks run sequentially");
        self.prev_hash = sim_hash(tick, inputs, self.prev_hash);
        self.next_tick += 1;
        if self.marks.last().map_or(true, |(t, _)| *t != tick) {
            self.marks.push((tick, self.prev_hash));
        }
        self.prev_hash
    }
}

fn handshake<'a>(
    shim: &Shim,
    host: &'a mut Session<ShimEndpoint>,
    client: &'a mut Session<ShimEndpoint>,
    ms: &mut u64,
) {
    let deadline = *ms + 4000;
    loop {
        *ms += 10;
        shim.advance(10);
        let _ = host.poll(*ms);
        let _ = client.poll(*ms);
        if host.agents().len() == 2 && client.local_slot() == 1 {
            return;
        }
        assert!(*ms < deadline, "handshake within the logical deadline");
    }
}

/// Start the match and run `ticks` ticks on host + client (slots 0/1),
/// recording + hashing every tick on the host (the late-join transcript).
/// Each machine runs at ITS OWN next tick (`ready_inputs` answers for any
/// tick whose inputs exist — re-asking is legal, re-running is not).
fn run_two(
    shim: &Shim,
    host: &mut Session<ShimEndpoint>,
    client: &mut Session<ShimEndpoint>,
    host_sim: &mut Sim,
    client_sim: &mut Sim,
    ticks: u32,
    ms: &mut u64,
) {
    host.start_match(0).unwrap();
    let deadline = *ms + 4000;
    while !client.is_started() {
        *ms += 10;
        shim.advance(10);
        let _ = host.poll(*ms);
        let _ = client.poll(*ms);
        assert!(*ms < deadline, "GO arrives");
    }
    while host_sim.next_tick < ticks || client_sim.next_tick < ticks {
        *ms += 10;
        shim.advance(10);
        let ht = host_sim.next_tick;
        host.local_input(ht, agent_input(0, ht));
        let ct = client_sim.next_tick;
        client.local_input(ct, agent_input(1, ct));
        let _ = host.poll(*ms);
        let _ = client.poll(*ms);
        if let Some(inputs) = host.ready_inputs(ht) {
            let h = host_sim.run(ht, &inputs);
            host.record_run_tick(ht, inputs);
            host.submit_hash(ht, h);
        }
        if let Some(inputs) = client.ready_inputs(ct) {
            let h = client_sim.run(ct, &inputs);
            client.submit_hash(ct, h);
        }
        assert!(*ms < 60000, "ticks complete (ms={ms})");
    }
}

/// A lossy link (5‰ drop, 30 ms latency, 20 ms jitter) — the transfer and the
/// ongoing match both ride it.
fn shim() -> Shim {
    let shim = Shim::new(0x0A11_CED5);
    shim.set_loss(5);
    shim.set_latency(30, 20);
    shim
}

/// The whole flow: host + client playing 120 ticks, a third joins at tick N,
/// replays the log, verifies, and plays on — the hashes agree everywhere.
#[test]
fn late_join_transfer_verify_and_hashes_agree() {
    let shim = shim();
    let mut ms: u64 = 0;
    let mut host = Session::new_host(test_cfg(), shim.endpoint(addr(8800)), MatchSettings::default(), 0x0BED_0BAD);
    let mut c1 = Session::new_client(test_cfg(), shim.endpoint(addr(8801)), addr(8800));
    handshake(&shim, &mut host, &mut c1, &mut ms);
    let mut host_sim = Sim::new();
    let mut c1_sim = Sim::new();
    run_two(&shim, &mut host, &mut c1, &mut host_sim, &mut c1_sim, 120, &mut ms);

    // The joiner dials a GO'd host: the late-join handover begins.
    let mut joiner = Session::new_client(test_cfg(), shim.endpoint(addr(8802)), addr(8800));
    let mut handover: Option<Event> = None;
    let mut host_saw_started = false;
    let deadline = ms + 6000;
    while handover.is_none() {
        ms += 10;
        shim.advance(10);
        for e in host.poll(ms) {
            if matches!(e, Event::LateJoinStarted { slot: 2, .. }) {
                host_saw_started = true;
            }
        }
        let _ = c1.poll(ms);
        for e in joiner.poll(ms) {
            if matches!(e, Event::LateJoinHandover { .. }) {
                handover = Some(e);
            }
        }
        assert!(ms < deadline, "the log transfer completes");
    }
    // The host announced the serving; the slot stays non-human so far.
    assert!(host_saw_started, "the host reports the late-join serving");
    // Slot 2 rides NEUTRAL pre-join: the last fully-run tick (119) is still
    // runnable without the joiner's input (a required-but-absent slot would
    // stall — never guess).
    assert!(host.ready_inputs(119).is_some(), "slot 2 rides NEUTRAL: no stall pre-join");

    let Event::LateJoinHandover { slot, seed, settings, first_tick, snapshot_tick, snapshot_hash, log } =
        handover.unwrap()
    else {
        unreachable!()
    };
    assert_eq!(slot, 2, "the joiner takes the first free slot");
    assert_eq!(seed, 0x0BED_0BAD, "the seed rides the handover");
    assert_eq!(settings, MatchSettings::default(), "the settings ride the handover");
    assert_eq!(first_tick, 0, "the log covers the match from tick 0");
    assert_eq!(snapshot_tick, 119, "the snapshot is the host's verified head");
    assert_eq!(log.len(), 120, "one log row per tick, inclusive");

    // The replay: the joiner folds the received log through the same chain —
    // it must reproduce the host's snapshot hash bit-exactly (a flipped byte
    // anywhere in the transfer breaks this).
    let mut j_sim = Sim::new();
    for (t, row) in log.iter().enumerate() {
        let h = j_sim.run(t as u32, row);
        if t as u32 == snapshot_tick {
            assert_eq!(h, snapshot_hash, "the replay hash matches the snapshot");
        }
    }
    assert_eq!(j_sim.prev_hash, snapshot_hash, "the chain ends at the snapshot hash");

    // Verified: confirm; every machine arms slot 2 at the same join tick.
    joiner.confirm_late_join();
    let mut join_ticks: Vec<u32> = Vec::new();
    let deadline = ms + 4000;
    while join_ticks.len() < 3 {
        ms += 10;
        shim.advance(10);
        for e in host.poll(ms) {
            if let Event::LateJoinSlot { slot: 2, join_tick } = e {
                join_ticks.push(join_tick);
            }
        }
        for e in c1.poll(ms) {
            if let Event::LateJoinSlot { slot: 2, join_tick } = e {
                join_ticks.push(join_tick);
            }
        }
        for e in joiner.poll(ms) {
            if let Event::LateJoinSlot { slot: 2, join_tick } = e {
                join_ticks.push(join_tick);
            }
        }
        assert!(ms < deadline, "the join tick is announced to all three");
    }
    assert_eq!(join_ticks[0], join_ticks[1], "the host and client agree on the join tick");
    assert_eq!(join_ticks[1], join_ticks[2], "the joiner agrees on the join tick");
    let join_tick = join_ticks[0];
    assert!(join_tick > snapshot_tick, "the join tick is past the snapshot");
    assert_eq!(join_tick % 1, 0, "the join tick is an absolute tick boundary");
    assert_eq!(joiner.local_slot(), 2, "the joiner owns slot 2");

    // All three play on. The joiner's slot requires real input from
    // join_tick; everyone keeps running the same fake sim and hash chain —
    // no Desync may fire on any machine. Each machine runs at its OWN
    // next tick (re-asking ready_inputs is legal, re-running is not).
    let mut desyncs: Vec<String> = Vec::new();
    let end = join_tick + 60;
    let deadline = ms + 20000;
    while host_sim.next_tick < end || c1_sim.next_tick < end || j_sim.next_tick < end {
        ms += 10;
        shim.advance(10);
        let ht = host_sim.next_tick;
        host.local_input(ht, agent_input(0, ht));
        let ct = c1_sim.next_tick;
        c1.local_input(ct, agent_input(1, ct));
        let jt = j_sim.next_tick;
        // The joiner samples its own slot from the join tick (its app's
        // controls only exist once its agent spawned); before it the slot
        // rides NEUTRAL on every machine.
        if jt >= join_tick {
            joiner.local_input(jt, agent_input(2, jt));
        }
        let _ = host.poll(ms);
        let _ = c1.poll(ms);
        let _ = joiner.poll(ms);
        if let Some(inputs) = host.ready_inputs(ht) {
            let h = host_sim.run(ht, &inputs);
            host.record_run_tick(ht, inputs);
            host.submit_hash(ht, h);
        }
        if let Some(inputs) = c1.ready_inputs(ct) {
            let h = c1_sim.run(ct, &inputs);
            c1.submit_hash(ct, h);
        }
        if let Some(inputs) = joiner.ready_inputs(jt) {
            let h = j_sim.run(jt, &inputs);
            joiner.submit_hash(jt, h);
        }
        desyncs.extend(host.poll(ms).into_iter().filter_map(|e| match e {
            Event::Desync { tick, .. } => Some(format!("host {tick}")),
            _ => None,
        }));
        desyncs.extend(c1.poll(ms).into_iter().filter_map(|e| match e {
            Event::Desync { tick, .. } => Some(format!("c1 {tick}")),
            _ => None,
        }));
        desyncs.extend(joiner.poll(ms).into_iter().filter_map(|e| match e {
            Event::Desync { tick, .. } => Some(format!("joiner {tick}")),
            _ => None,
        }));
        assert!(ms < deadline, "the post-join run completes");
    }
    assert_eq!(host_sim.prev_hash, c1_sim.prev_hash, "host and client hashes agree after the join");
    assert_eq!(host_sim.prev_hash, j_sim.prev_hash, "the joiner's hash agrees after the join");
    assert!(desyncs.is_empty(), "no desync was reported: {desyncs:?}");
}

/// The match is full (4 agents = the game's MP limit, humans 0..4 [M
/// MP_Start 0x183a50]): a late HELLO is refused with `Full`.
#[test]
fn late_join_refused_when_full() {
    let shim = Shim::new(7);
    let mut ms: u64 = 0;
    let mut host = Session::new_host(test_cfg(), shim.endpoint(addr(8810)), MatchSettings::default(), 1);
    let mut clients: Vec<Session<ShimEndpoint>> = (0..3)
        .map(|i| Session::new_client(test_cfg(), shim.endpoint(addr(8811 + i)), addr(8810)))
        .collect();
    for i in 0..3 {
        let deadline = ms + 4000;
        loop {
            ms += 10;
            shim.advance(10);
            let _ = host.poll(ms);
            let mut done = clients[i].local_slot() == (i as u8 + 1);
            for e in clients[i].poll(ms) {
                let _ = e;
            }
            if done {
                break;
            }
            assert!(ms < deadline, "client {i} handshake");
        }
    }
    host.start_match(0).unwrap();
    let deadline = ms + 4000;
    while clients.iter().any(|c| !c.is_started()) {
        ms += 10;
        shim.advance(10);
        let _ = host.poll(ms);
        for c in clients.iter_mut() {
            let _ = c.poll(ms);
        }
        assert!(ms < deadline, "GO reaches everyone");
    }
    // A fourth machine dials: refused, full.
    let mut joiner = Session::new_client(test_cfg(), shim.endpoint(addr(8814)), addr(8810));
    let deadline = ms + 4000;
    loop {
        ms += 10;
        shim.advance(10);
        let _ = host.poll(ms);
        let mut refused = false;
        for e in joiner.poll(ms) {
            if matches!(e, Event::Refused { reason: RefuseReason::Full }) {
                refused = true;
            }
        }
        if refused {
            break;
        }
        assert!(ms < deadline, "the full host refuses the late joiner");
    }
    assert_eq!(host.agents().len(), 4, "the refusal left the roster untouched");
}

/// A joiner whose replay does NOT verify leaves with `JoinVerifyFailed` —
/// the host reports the clear reason — and the slot stays free for a retry.
#[test]
fn late_join_verify_failure_reports_clear_reason_and_slot_stays_free() {
    let shim = Shim::new(11);
    let mut ms: u64 = 0;
    let mut host = Session::new_host(test_cfg(), shim.endpoint(addr(8820)), MatchSettings::default(), 2);
    let mut c1 = Session::new_client(test_cfg(), shim.endpoint(addr(8821)), addr(8820));
    handshake(&shim, &mut host, &mut c1, &mut ms);
    let mut host_sim = Sim::new();
    let mut c1_sim = Sim::new();
    run_two(&shim, &mut host, &mut c1, &mut host_sim, &mut c1_sim, 60, &mut ms);

    // First joiner: takes the transfer, then reports the failed verification.
    let mut joiner = Session::new_client(test_cfg(), shim.endpoint(addr(8822)), addr(8820));
    let deadline = ms + 6000;
    loop {
        ms += 10;
        shim.advance(10);
        let _ = host.poll(ms);
        let _ = c1.poll(ms);
        let got = joiner.poll(ms).into_iter().any(|e| matches!(e, Event::LateJoinHandover { .. }));
        if got {
            break;
        }
        assert!(ms < deadline, "the first transfer completes");
    }
    // The replay disagrees (the app's hash check failed): leave with the
    // explicit reason — the host surfaces it verbatim.
    joiner.leave_with(LeaveReason::JoinVerifyFailed);
    let deadline = ms + 4000;
    loop {
        ms += 10;
        shim.advance(10);
        let mut left = false;
        for e in host.poll(ms) {
            if matches!(e, Event::ClientLeft { slot: 2, reason: LeaveReason::JoinVerifyFailed }) {
                left = true;
            }
        }
        let _ = c1.poll(ms);
        let _ = joiner.poll(ms);
        if left {
            break;
        }
        assert!(ms < deadline, "the host reports the clear reason");
    }
    assert_eq!(host.agents().len(), 2, "the failed joiner is gone");

    // The slot is free again: a retry takes it and the transfer restarts.
    let mut retry = Session::new_client(test_cfg(), shim.endpoint(addr(8823)), addr(8820));
    let deadline = ms + 6000;
    loop {
        ms += 10;
        shim.advance(10);
        let _ = host.poll(ms);
        let _ = c1.poll(ms);
        let got = retry.poll(ms).into_iter().any(|e| matches!(e, Event::LateJoinHandover { slot: 2, .. }));
        if got {
            break;
        }
        assert!(ms < deadline, "the retry transfer completes");
    }
}
