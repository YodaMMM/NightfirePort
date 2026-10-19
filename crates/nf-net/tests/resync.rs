//! net-desync-resync: the resync handshake at session level on the
//! deterministic shim (logical ms, seeded loss + latency — the same harness
//! as `drop_rejoin.rs`, whose reconnect test is the template):
//!
//! - a client whose state hash diverges mid-match stays in the match: its
//!   session asks (the new `Packet::Resync`, tag 19), the host re-serves the
//!   late-join family (JOIN_BEGIN + JOIN_LOG, the same `run_log` a late
//!   joiner gets) ON THE LIVE CONNECTION, the client replays the host's rows
//!   (the fake chain — the same shape the app's `mp_late_join_catchup_joins`
//!   runs), verifies the snapshot hash, confirms, and the re-announce
//!   (`LateJoinSlot`) arms its slot again on every machine. The gap runs
//!   NEUTRAL for the resyncer's slot on BOTH sides (the host's
//!   `host_slot_required` drop after the serve; the resyncer's own
//!   `join_from` marker at arm time) and no `join_history` entry is added
//!   (the body never left the match). The other machines keep playing.
//! - sisters: no corruption → nothing resyncs; a second corruption inside
//!   the 60 s window is fatal (`DesyncFatal` — the lobby-message event).

use std::net::SocketAddr;

use nf_net::shim::{Shim, ShimEndpoint};
use nf_net::session::{Config, Event, Session};
use nf_net::{Input, MatchSettings, NEUTRAL, MAX_AGENTS};

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

fn agent_input(slot: u8, tick: u32) -> Input {
    let mut inp = NEUTRAL;
    let x = (slot as u32).wrapping_mul(0x9E37_79B9) ^ tick.wrapping_mul(0x85EB_CA6B);
    let f = ((x & 0xFF) as i32 - 128) as f32 / 128.0;
    inp.forward = if f == 0.0 { 0.5 } else { f };
    inp.turn = (((x >> 8) & 0xFF) as i32 - 128) as f32 / 128.0;
    inp.fire = x & 0x100 != 0;
    inp.jump = x & 0x200 != 0;
    inp.weapon_cycle = (x & 0x0F) as i8 - 8;
    inp
}

fn sim_hash(tick: u32, inputs: &[Input; MAX_AGENTS], prev: u64) -> u64 {
    let mut h = prev ^ 0xC0FF_EE00;
    for (s, i) in inputs.iter().enumerate() {
        h = h.rotate_left(17) ^ (i.forward.to_bits() as u64).wrapping_add(s as u64 + 1);
        h = h.rotate_left(13) ^ (i.fire as u64).wrapping_mul(31);
        h = h.rotate_left(7) ^ (i.weapon_cycle as u8 as u64);
        h = h.rotate_left(11) ^ ((i.jump as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    }
    h ^ (tick as u64)
}

/// One machine: its session, its fake-sim chain, every row it ran.
struct Machine {
    sess: Session<ShimEndpoint>,
    next_tick: u32,
    prev_hash: u64,
    rows: Vec<[Input; MAX_AGENTS]>,
    hashes: Vec<u64>,
    alive: bool,
    lead: u32,
    /// The tick from which this machine's HASH submissions carry one flipped
    /// bit (the test's "a body's x moved on this machine only" — persistent
    /// state, so every later exchange diverges too; the machine's own chain
    /// stays true). Hash packets are one-shot (no retransmit), so a persistent
    /// divergence is what guarantees the mismatch reaches the host through the
    /// shim's seeded 5% loss.
    corrupt_from: Option<u32>,
}

impl Machine {
    fn new(sess: Session<ShimEndpoint>) -> Machine {
        Machine { sess, next_tick: 0, prev_hash: 0, rows: Vec::new(), hashes: Vec::new(), alive: true, lead: 0, corrupt_from: None }
    }
    fn run(&mut self, t: u32, row: [Input; MAX_AGENTS]) -> u64 {
        assert_eq!(self.next_tick, t, "ticks run sequentially");
        self.prev_hash = sim_hash(t, &row, self.prev_hash);
        if self.rows.len() == t as usize {
            self.rows.push(row);
            self.hashes.push(self.prev_hash);
        } else {
            while self.rows.len() < t as usize { self.rows.push([NEUTRAL; MAX_AGENTS]); self.hashes.push(0); }
            self.rows.push(row);
            self.hashes.push(self.prev_hash);
        }
        self.next_tick = t + 1;
        self.prev_hash
    }
    fn frame(&mut self, ms: u64, host: bool) -> Vec<Event> {
        if !self.alive {
            return Vec::new();
        }
        let t = self.next_tick;
        let slot = self.sess.local_slot();
        for k in t..=t + self.lead {
            self.sess.local_input(k, agent_input(slot, k));
        }
        let mut evs = self.sess.poll(ms);
        if let Some(row) = self.sess.ready_inputs(t) {
            let mut h = self.run(t, row);
            if self.corrupt_from.map_or(false, |f| t >= f) {
                h ^= 1; // the machine's state hash diverges from this tick on
            }
            if host { self.sess.record_run_tick(t, row); }
            self.sess.submit_hash(t, h);
        }
        evs.extend(self.sess.poll(ms));
        evs
    }
}

fn lossy() -> Shim {
    let shim = Shim::new(0xD2_0952);
    shim.set_loss(5);
    shim.set_latency(30, 20);
    shim
}

fn start_match(shim: &Shim, base: u16, names: &[&str], ms: &mut u64) -> (Machine, Vec<Machine>) {
    let host = Session::new_host(test_cfg(), shim.endpoint(addr(base)), MatchSettings::default(), 0x5EED);
    let mut host = Machine::new(host);
    let mut clients: Vec<Machine> = Vec::new();
    for (i, n) in names.iter().enumerate() {
        let mut c = Session::new_client(test_cfg(), shim.endpoint(addr(base + 1 + i as u16)), addr(base));
        c.set_display_name(n);
        let mut c = Machine::new(c);
        let deadline = *ms + 4000;
        while c.sess.local_slot() == 0 {
            *ms += 10;
            shim.advance(10);
            let _ = host.sess.poll(*ms);
            let _ = c.sess.poll(*ms);
            for o in clients.iter_mut() { let _ = o.sess.poll(*ms); }
            assert!(*ms < deadline, "handshake {n}");
        }
        assert_eq!(c.sess.local_slot() as usize, i + 1, "{n} takes slot {}", i + 1);
        clients.push(c);
    }
    host.sess.start_match(0).unwrap();
    let deadline = *ms + 4000;
    while clients.iter().any(|c| !c.sess.is_started()) {
        *ms += 10;
        shim.advance(10);
        let _ = host.sess.poll(*ms);
        for c in clients.iter_mut() { let _ = c.sess.poll(*ms); }
        assert!(*ms < deadline, "GO reaches everyone");
    }
    (host, clients)
}

fn frame_all(shim: &Shim, host: &mut Machine, clients: &mut [Machine], ms: &mut u64) -> Vec<(usize, Event)> {
    *ms += 10;
    shim.advance(10);
    let mut out: Vec<(usize, Event)> = host.frame(*ms, true).into_iter().map(|e| (0, e)).collect();
    for (i, c) in clients.iter_mut().enumerate() {
        out.extend(c.frame(*ms, false).into_iter().map(|e| (i + 1, e)));
    }
    out
}

fn assert_same_run(a: &Machine, b: &Machine, from: u32, what: &str) {
    let n = a.next_tick.min(b.next_tick) as usize;
    assert!(n > from as usize, "{what}: the machines ran past tick {from}");
    for t in from as usize..n {
        assert_eq!(a.rows[t], b.rows[t], "{what}: tick {t} ran with different inputs");
        assert_eq!(a.hashes[t], b.hashes[t], "{what}: tick {t}'s hash differs");
    }
}

/// The main test: slot 2's hash diverges at tick 120; that machine resyncs
/// over the live connection (the host serves the log, the replay verifies,
/// the re-announce arms the slot), the gap runs NEUTRAL on both sides, no
/// `join_history` entry is added, and all three agree to the end of the run.
#[test]
fn a_diverged_machine_resyncs_over_the_live_connection_and_the_match_runs_on() {
    let shim = lossy();
    let mut ms = 0u64;
    let (mut host, mut clients) = start_match(&shim, 9160, &["Alpha", "Bravo", "Charlie"], &mut ms);
    while host.next_tick < 100 || clients.iter().any(|c| c.next_tick < 100) {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < 60_000, "the opening run");
    }
    assert_same_run(&host, &clients[0], 0, "pre-corruption host vs Alpha");
    assert_same_run(&host, &clients[1], 0, "pre-corruption host vs Bravo");
    // Charlie's state diverges at tick 120 (a hash-exchange tick).
    let desync_tick = 120;
    clients[1].corrupt_from = Some(desync_tick);
    // The desync fires on the diverged machine (its local vs the host's hash);
    // Alpha — in sync with the host — sees nothing. The HOST may or may not see
    // the event: hash packets are one-shot (no retransmit) and the diverged
    // machine stops sending them once it arms the resync — the host's
    // authoritative signal is the RESYNC request, which the serve answers.
    let mut charlie_desync = None;
    let deadline = ms + 20_000;
    while charlie_desync.is_none() {
        for (who, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::Desync { tick, local, remote } = e {
                assert_ne!(local, remote, "the two hashes differ");
                if who == 2 { charlie_desync = Some(tick); }
                assert_ne!(who, 1, "Alpha (in sync with the host) sees no mismatch");
            }
        }
        assert!(ms < deadline, "the desync fires ({charlie_desync:?})");
    }
    assert!(charlie_desync.unwrap() >= desync_tick, "Charlie saw its own mismatch");
    assert!(clients[0].sess.is_started(), "Alpha never even paused");
    // The serve + the handover arrive on the LIVE session (no re-dial).
    let mut handover = None;
    let mut served = false;
    let deadline = ms + 20_000;
    while handover.is_none() {
        for (who, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            match e {
                Event::LateJoinStarted { slot, .. } if who == 0 => {
                    assert_eq!(slot, 2, "the host serves Charlie's slot");
                    served = true;
                }
                Event::LateJoinHandover { slot, first_tick, snapshot_tick, snapshot_hash, log, .. } => {
                    assert_eq!(who, 2, "the handover lands on the diverged machine");
                    assert_eq!(slot, 2);
                    eprintln!("[dbg] handover: first {first_tick} snapshot {snapshot_tick} log {} rows charlie_next {} | log row 100 slot2 NEUTRAL {} host run_log row 100 slot2 NEUTRAL {}", log.len(), clients[1].next_tick, log.get(100).map_or(false, |r| r[2] == NEUTRAL), host.rows.get(100).map_or(false, |r| r[2] == NEUTRAL));
                    handover = Some((first_tick, snapshot_tick, snapshot_hash, log));
                }
                Event::DesyncFatal { tick } => {
                    for k in 0..MAX_AGENTS {
                        }
                    {
                        let n = host.rows.len().min(clients[1].rows.len());
                        let mut first = None;
                        for t in 0..n {
                            if host.rows[t] != clients[1].rows[t] {
                                first = Some(t);
                                break;
                            }
                        }
                        if let Some(t) = first {
                            for k in 0..MAX_AGENTS {
                                if host.rows[t][k] != clients[1].rows[t][k] {
                                    eprintln!("[dbg] first row diff at tick {t} slot {k}: host fwd {:?} charlie fwd {:?} (host slot2==NEUTRAL: {}, charlie: {})",
                                        host.rows[t][k].forward, clients[1].rows[t][k].forward,
                                        host.rows[t][2] == NEUTRAL, clients[1].rows[t][2] == NEUTRAL);
                                }
                            }
                        } else {
                            eprintln!("[dbg] rows identical to tick {n} — the CHAIN diverged without a row diff?!");
                        }
                        let hn = host.hashes.len().min(clients[1].hashes.len());
                        let mut hdiff = None;
                        for t in 0..hn {
                            if host.hashes[t] != clients[1].hashes[t] { hdiff = Some(t); break; }
                        }
                        eprintln!("[dbg] first HASH divergence: {hdiff:?} (host hashes {}, charlie {})", host.hashes.len(), clients[1].hashes.len());
                        eprintln!("[dbg] chain(240): host {:016x} charlie {:016x}", host.hashes.get(240).copied().unwrap_or(0), clients[1].hashes.get(240).copied().unwrap_or(0));
                    }
                    panic!("the first resync must not be fatal (machine {who}, tick {tick})");
                }
                _ => {}
            }
        }
        assert!(ms < deadline, "the resync handover completes (served: {served})");
    }
    assert!(served, "the host announced the serve");
    let (first_tick, snapshot_tick, snapshot_hash, log) = handover.unwrap();
    assert_eq!(first_tick, 0, "the resync replays from the match start");
    assert!(snapshot_tick < desync_tick, "the snapshot is an AGREEING tick ({} < {desync_tick})", snapshot_tick);
    // The replay: the same chain over the host's rows reproduces the snapshot
    // hash (the fake chain stands in for `mp_late_join_catchup_joins`).
    clients[1].corrupt_from = None; // the replay REPLACES the perturbed state
    clients[1].next_tick = first_tick;
    clients[1].prev_hash = 0;
    clients[1].rows.clear();
    clients[1].hashes.clear();
    for (i, row) in log.iter().enumerate() {
        let t = first_tick + i as u32;
        if t > snapshot_tick { break; }
        let h = clients[1].run(t, *row);
        // The app's replayed sim re-submits every re-run tick's hash
        // (`net_tick`'s `submit_hash(self.net_tick, …)`) — the session's
        // local_hashes must follow the replayed state, not the diverged one.
        clients[1].sess.submit_hash(t, h);
    }
    assert_eq!(clients[1].prev_hash, snapshot_hash, "the replay verifies against the host's snapshot");
    clients[1].sess.confirm_late_join();
    // The re-announce arms the slot on every machine; the resyncing body never
    // left, so no join_history entry was added (checked via the host's own record).
    let mut jt = None;
    let deadline = ms + 20_000;
    while jt.is_none() {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::LateJoinSlot { slot: 2, join_tick } = e { jt = Some(join_tick); }
        }
        assert!(ms < deadline, "the re-announce lands");
    }
    let join_tick = jt.unwrap();
    assert!(join_tick > snapshot_tick, "the rejoin is ahead of the replay");
    // The gap ran NEUTRAL for slot 2 on the host (from the serve to the join
    // tick) and Charlie's real input resumes exactly at the join tick.
    let serve_gap = (desync_tick as usize..host.rows.len()).find(|t| host.rows[*t][2] == NEUTRAL)
        .expect("slot 2 rode NEUTRAL during the resync gap");
    assert!(serve_gap < join_tick as usize, "the gap opened before the rejoin");
    let end = join_tick + 150;
    let deadline = ms + 40_000;
    while host.next_tick < end || clients.iter().any(|c| c.next_tick < end) {
        for (who, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::DesyncFatal { tick } = e {
                let n = host.rows.len().min(clients[1].rows.len());
                let mut first = None;
                for t in 0..n { if host.rows[t] != clients[1].rows[t] { first = Some(t); break; } }
                let hn = host.hashes.len().min(clients[1].hashes.len());
                let mut hdiff = None;
                for t in 0..hn { if host.hashes[t] != clients[1].hashes[t] { hdiff = Some(t); break; } }
                let t0 = first.unwrap_or(0);
                eprintln!("[dbg] t0 {t0}: host rowslot2 NEUTRAL {} charlie {}", host.rows[t0][2] == NEUTRAL, clients[1].rows[t0][2] == NEUTRAL);
                let (hr, cr) = (&host.rows[t0], &clients[1].rows[t0]);
                let slots: Vec<String> = (0..4).map(|k| format!("slot {k}: host fwd {:?} fire {} | charlie fwd {:?} fire {}", hr[k].forward, hr[k].fire, cr[k].forward, cr[k].fire)).collect();
                panic!("no fatal after the resync ({who}: tick {tick}) — first row diff {first:?}, first hash diff {hdiff:?}, rows {}/{} :: {}",
                    host.rows.len(), clients[1].rows.len(), slots.join(" | "));
            }
        }
        assert!(ms < deadline, "the post-resync run (host {} c1 {} charlie {} of {end}; ms {ms})", host.next_tick, clients[0].next_tick, clients[1].next_tick);
    }
    assert_eq!(host.rows[join_tick as usize][2], agent_input(2, join_tick), "slot 2 runs Charlie's input from the join tick");
    assert_eq!(host.rows[join_tick as usize - 1][2], NEUTRAL, "slot 2 was NEUTRAL just before the rejoin");
    assert_same_run(&host, &clients[0], 0, "host vs Alpha to the end");
    assert_same_run(&host, &clients[1], snapshot_tick + 1, "host vs the resynced Charlie");
    // No join_history entry: a FUTURE late joiner's JOIN_BEGIN lists only joins
    // at or before its snapshot and none were added — the host served no new
    // history (the body never left). observable: the session's agents count is
    // unchanged and no LateJoinStarted for a NEW slot fired (the serve carried
    // slot 2's own).
    assert_eq!(host.sess.agents().len(), 3, "host + Alpha + Charlie, nobody left");
}

/// Sister: with no corruption nothing resyncs — no desync, no serve, no
/// handover, no re-announce, no fatal, for the whole run.
#[test]
fn without_a_divergence_nothing_resyncs() {
    let shim = lossy();
    let mut ms = 0u64;
    let (mut host, mut clients) = start_match(&shim, 9170, &["Alpha", "Bravo"], &mut ms);
    let end = 300;
    while host.next_tick < end || clients.iter().any(|c| c.next_tick < end) {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            assert!(!matches!(e, Event::Desync { .. } | Event::DesyncFatal { .. }), "a clean run never desyncs: {e:?}");
            assert!(!matches!(e, Event::LateJoinStarted { .. } | Event::LateJoinHandover { .. } | Event::LateJoinSlot { .. }), "a clean run never resyncs: {e:?}");
        }
        assert!(ms < 90_000, "the clean run");
    }
    assert_same_run(&host, &clients[0], 0, "clean host vs Alpha");
    assert_same_run(&host, &clients[1], 0, "clean host vs Bravo");
}

/// Sister 2: a SECOND divergence inside the 60 s window is fatal — the
/// machine falls back to ending the match (the pre-resync behaviour, the
/// event the app's lobby message rides).
#[test]
fn a_second_divergence_within_the_window_is_fatal() {
    let shim = lossy();
    let mut ms = 0u64;
    let (mut host, mut clients) = start_match(&shim, 9180, &["Alpha", "Bravo"], &mut ms);
    while host.next_tick < 100 || clients.iter().any(|c| c.next_tick < 100) {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < 60_000, "the opening run");
    }
    // First divergence → the resync completes (the main test's flow).
    clients[1].corrupt_from = Some(120);
    let mut handover = false;
    let deadline = ms + 30_000;
    while !handover {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if matches!(e, Event::LateJoinHandover { .. }) { handover = true; }
        }
        assert!(ms < deadline, "the first resync's handover");
    }
    let (mut first_tick, mut snapshot_tick, mut snapshot_hash, mut log) = (0u32, 0u32, 0u64, Vec::new());
    let deadline = ms + 30_000;
    while snapshot_tick == 0 {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::LateJoinHandover { first_tick: f, snapshot_tick: s, snapshot_hash: h, log: l, .. } = e {
                (first_tick, snapshot_tick, snapshot_hash, log) = (f, s, h, l);
            }
        }
        assert!(ms < deadline, "the handover captured");
    }
    clients[1].corrupt_from = None; // the replay replaces the perturbed state
    clients[1].next_tick = first_tick;
    clients[1].prev_hash = 0;
    clients[1].rows.clear();
    clients[1].hashes.clear();
    for (i, row) in log.iter().enumerate() {
        let t = first_tick + i as u32;
        if t > snapshot_tick { break; }
        let h = clients[1].run(t, *row);
        clients[1].sess.submit_hash(t, h); // the app re-submits the replayed hashes
    }
    assert_eq!(clients[1].prev_hash, snapshot_hash, "the first replay verifies");
    clients[1].sess.confirm_late_join();
    // The second divergence, well inside 60 s of the resync request.
    let mut rejoined = None;
    let deadline = ms + 30_000;
    while rejoined.is_none() {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::LateJoinSlot { slot: 1, join_tick } = e { rejoined = Some(join_tick); }
        }
        assert!(ms < deadline, "the first resync's re-announce");
    }
    let join_tick = rejoined.unwrap();
    let end = join_tick + 60;
    let deadline = ms + 40_000;
    while host.next_tick < end || clients[1].next_tick < end {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < deadline, "the resyncer rejoins");
    }
    clients[1].corrupt_from = Some(host.next_tick + 30);
    let mut fatal = None;
    let deadline = ms + 30_000;
    while fatal.is_none() {
        for (who, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::DesyncFatal { tick } = e {
                assert_eq!(who, 1, "the fatal lands on the twice-diverged machine");
                fatal = Some(tick);
            }
        }
        assert!(ms < deadline, "the second divergence is fatal");
    }
    assert!(ms - 0 < 60_000, "the whole test stayed inside the 60 s window (the harness's clock)");
    eprintln!("[resync] the second divergence within the window is fatal at tick {:?}", fatal);
}
