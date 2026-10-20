//! mp-online-failures: the session's failure paths on the deterministic shim (logical ms, seeded
//! loss + latency — the same harness as `late_join.rs`):
//!
//! - a client dropping mid-match (graceful LEAVE or a crash → timeout) never desyncs the
//!   machines that stay: the host keeps relaying the leaver's REAL inputs for the ticks it
//!   already ran, NEUTRAL only from its drop tick on, and a client takes every other slot from
//!   that relay (before this, the host relayed NEUTRAL for already-run ticks and a client behind
//!   the host ran them differently — a desync on every drop);
//! - the leaver reconnects into the running match and takes ITS slot back (the codename match),
//!   not the lowest free one, and plays on in sync;
//! - the host's LEAVE reaches a client at once (`HostLeft`), not after the 5 s timeout;
//! - a finished match (`close_joins`) answers no new HELLO — the rematch re-dial waits for the
//!   next lobby instead of late-joining the match that just ended.

use std::net::SocketAddr;

use nf_net::shim::{Shim, ShimEndpoint};
use nf_net::session::{Config, Event, LeaveReason, Session};
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

/// Deterministic per-agent controls for (slot, tick) — never NEUTRAL (forward is never 0 for
/// these inputs' x & 0xFF != 128), so a NEUTRAL substitution always shows in the chain.
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

/// One machine: its session, the next tick its fake sim runs, the hash chain and every row it
/// ran (the comparison is row-by-row: what each tick RAN with must agree on every machine).
struct Machine {
    sess: Session<ShimEndpoint>,
    next_tick: u32,
    prev_hash: u64,
    rows: Vec<[Input; MAX_AGENTS]>,
    hashes: Vec<u64>,
    /// Whether this machine feeds its own slot this frame (a crashed / gone machine does not).
    alive: bool,
    /// The late joiner's first own-input tick (None = a match-start human).
    sample_from: Option<u32>,
    /// Inputs are scheduled this many ticks ahead of the running tick (the app's input lead —
    /// the controls are deterministic here, so any lead is honest).
    lead: u32,
    /// A machine that keeps pumping its socket but runs no tick (a slow frame): the others run
    /// ahead of it by up to the lead.
    hold: bool,
}

impl Machine {
    fn new(sess: Session<ShimEndpoint>) -> Machine {
        Machine { sess, next_tick: 0, prev_hash: 0, rows: Vec::new(), hashes: Vec::new(), alive: true, sample_from: None, lead: 0, hold: false }
    }
    fn run(&mut self, t: u32, row: [Input; MAX_AGENTS]) -> u64 {
        assert_eq!(self.next_tick, t, "ticks run sequentially");
        self.prev_hash = sim_hash(t, &row, self.prev_hash);
        if self.rows.len() == t as usize {
            self.rows.push(row);
            self.hashes.push(self.prev_hash);
        } else {
            // A late joiner's rows start at its replayed head.
            while self.rows.len() < t as usize { self.rows.push([NEUTRAL; MAX_AGENTS]); self.hashes.push(0); }
            self.rows.push(row);
            self.hashes.push(self.prev_hash);
        }
        self.next_tick = t + 1;
        self.prev_hash
    }
    /// One frame: feed the own input, pump, run the next tick when ready (record + hash).
    fn frame(&mut self, ms: u64, host: bool) -> Vec<Event> {
        if !self.alive {
            return Vec::new();
        }
        let t = self.next_tick;
        let slot = self.sess.local_slot();
        for k in t..=t + self.lead {
            if self.sample_from.map_or(true, |f| k >= f) {
                self.sess.local_input(k, agent_input(slot, k));
            }
        }
        let mut evs = self.sess.poll(ms);
        if self.hold {
            return evs;
        }
        if let Some(row) = self.sess.ready_inputs(t) {
            let h = self.run(t, row);
            if host { self.sess.record_run_tick(t, row); }
            self.sess.submit_hash(t, h);
        }
        evs.extend(self.sess.poll(ms));
        evs
    }
}

fn lossy() -> Shim {
    let shim = Shim::new(0xD2_0951);
    shim.set_loss(5);
    shim.set_latency(30, 20);
    shim
}

/// Host + `n` clients (codenames by slot), handshaken and started.
fn start_match(shim: &Shim, base: u16, names: &[&str], ms: &mut u64) -> (Machine, Vec<Machine>) {
    let mut host = Session::new_host(test_cfg(), shim.endpoint(addr(base)), MatchSettings::default(), 0x5EED);
    host.set_display_name("Host");
    let mut host = Machine::new(host);
    let mut clients: Vec<Machine> = Vec::new();
    // One at a time, so the slots are 1, 2, 3 in name order.
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

/// One frame for everyone; returns the events (machine index 0 = host, 1.. = clients).
fn frame_all(shim: &Shim, host: &mut Machine, clients: &mut [Machine], ms: &mut u64) -> Vec<(usize, Event)> {
    *ms += 10;
    shim.advance(10);
    let mut out: Vec<(usize, Event)> = host.frame(*ms, true).into_iter().map(|e| (0, e)).collect();
    for (i, c) in clients.iter_mut().enumerate() {
        out.extend(c.frame(*ms, false).into_iter().map(|e| (i + 1, e)));
    }
    out
}

/// Every tick both machines ran: the same row, the same hash.
fn assert_same_run(a: &Machine, b: &Machine, from: u32, what: &str) {
    let n = a.next_tick.min(b.next_tick) as usize;
    assert!(n > from as usize, "{what}: the machines ran past tick {from}");
    for t in from as usize..n {
        assert_eq!(a.rows[t], b.rows[t], "{what}: tick {t} ran with different inputs");
        assert_eq!(a.hashes[t], b.hashes[t], "{what}: tick {t}'s hash differs");
    }
}

/// The drop boundary on the host: slot 2 ran its real inputs up to some tick, NEUTRAL after.
/// Returns the first NEUTRAL tick (the drop tick) — the sister check that the comparison above
/// is not vacuous: the leaver's last real row is NOT NEUTRAL, so a machine that substituted
/// NEUTRAL there would have run a different row (the old bug).
fn drop_tick(host: &Machine, slot: usize, after: u32) -> u32 {
    let t = (after as usize..host.rows.len()).find(|t| host.rows[*t][slot] == NEUTRAL).expect("the slot went NEUTRAL") as u32;
    assert!(t > 0 && host.rows[t as usize - 1][slot] != NEUTRAL, "the leaver's last run tick carried its real input");
    t
}

fn run_drop_variant(crash: bool, base: u16) {
    let shim = lossy();
    let mut ms = 0u64;
    let (mut host, mut clients) = start_match(&shim, base, &["Alpha", "Bravo"], &mut ms);
    // 150 honest ticks.
    while host.next_tick < 150 || clients.iter().any(|c| c.next_tick < 150) {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < 60_000, "the pre-drop run");
    }
    // The race the drop must survive: Alpha falls BEHIND (a slow frame — it keeps pumping its
    // socket but runs no tick) while the host runs ahead on the inputs scheduled 20 ticks
    // ahead; Bravo goes while Alpha still has those ticks to run. Alpha then runs ticks the
    // host ran with Bravo's REAL inputs — it must run them the same way.
    for m in std::iter::once(&mut host).chain(clients.iter_mut()) { m.lead = 20; }
    clients[0].hold = true;
    let held_at = clients[0].next_tick;
    let deadline = ms + 20_000;
    while host.next_tick < held_at + 15 {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < deadline, "the host runs ahead of the held client");
    }
    // Bravo (slot 2) goes: a graceful LEAVE (the app's close / quit) or a crash (silence).
    let leave_from = host.next_tick;
    if crash {
        clients[1].alive = false;
    } else {
        clients[1].sess.leave();
        clients[1].alive = false;
    }
    let mut host_saw = None;
    let mut c1_saw = None;
    let mut alpha_at_news: Option<u32> = None;
    let end = leave_from + if crash { 450 } else { 200 };
    let deadline = ms + 120_000;
    while host.next_tick < end || clients[0].next_tick < end {
        for (who, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::ClientLeft { slot: 2, reason } = e {
                if who == 0 { host_saw = Some(reason); } else if who == 1 { c1_saw = Some(reason); }
            }
            // Alpha resumes only once the departure reached it (it is still behind the host).
            if who == 1 && c1_saw.is_some() && clients[0].hold { clients[0].hold = false; alpha_at_news = Some(clients[0].next_tick); }
            assert!(!matches!(e, Event::Desync { .. }), "no desync after the drop ({who}: {e:?})");
        }
        assert!(ms < deadline, "the match runs on after the drop (host at {}, Alpha at {}, host saw {host_saw:?}, Alpha saw {c1_saw:?})",
            host.next_tick, clients[0].next_tick);
    }
    let want = if crash { LeaveReason::Timeout } else { LeaveReason::Graceful };
    assert_eq!(host_saw, Some(want), "the host reports the drop with its reason");
    assert_eq!(c1_saw, Some(want), "the staying client hears it relayed");
    // The staying machines ran exactly the same rows — through the drop boundary.
    assert_same_run(&host, &clients[0], 0, "host vs Alpha");
    let dt = drop_tick(&host, 2, held_at);
    // Non-vacuous: Alpha heard of the drop while ticks the host ran with Bravo's real input
    // were still ahead of it (the old session ran them NEUTRAL there — a desync).
    let a = alpha_at_news.expect("Alpha heard of the drop");
    assert!(a < dt && host.rows[a as usize][2] != NEUTRAL, "Alpha was at tick {a}, behind the drop tick {dt}");
    assert!(dt + 50 < host.next_tick, "the match ran well past the drop tick {dt}");
    assert!(host.rows[dt as usize..].iter().all(|r| r[2] == NEUTRAL), "slot 2 rides NEUTRAL from its drop tick on");
}

/// A graceful LEAVE mid-match: the others run on in lockstep, row-identical through the drop.
#[test]
fn a_client_leaving_mid_match_never_desyncs_the_others() {
    run_drop_variant(false, 9100);
}

/// A crash (the leaver goes silent): the others stall for the timeout, then run on NEUTRAL —
/// row-identical through the drop.
#[test]
fn a_client_crashing_mid_match_never_desyncs_the_others() {
    run_drop_variant(true, 9120);
}

/// The leaver reconnects into the running match: it takes ITS slot back (Bravo → 2) although a
/// lower slot (Alpha's 1) is free too, replays the host's log, verifies, and plays on — every
/// machine's rows agree from the reconnect on.
#[test]
fn a_reconnect_takes_its_own_slot_back_and_plays_on_in_sync() {
    let shim = lossy();
    let mut ms = 0u64;
    let (mut host, mut clients) = start_match(&shim, 9140, &["Alpha", "Bravo", "Charlie"], &mut ms);
    while host.next_tick < 100 || clients.iter().any(|c| c.next_tick < 100) {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < 60_000, "the opening run");
    }
    // Alpha and Bravo both leave.
    clients[0].sess.leave();
    clients[0].alive = false;
    clients[1].sess.leave();
    clients[1].alive = false;
    let resume = host.next_tick + 60;
    while host.next_tick < resume || clients[2].next_tick < resume {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < 90_000, "the run after the two drops");
    }
    assert_eq!(host.sess.agents().len(), 2, "host + Charlie remain");
    // Bravo dials again (a fresh socket — the app's new session).
    let mut back = Session::new_client(test_cfg(), shim.endpoint(addr(9150)), addr(9140));
    back.set_display_name("Bravo");
    let mut back = Machine::new(back);
    back.sample_from = Some(u32::MAX); // no own input before the announced join tick
    let mut handover = None;
    let deadline = ms + 10_000;
    while handover.is_none() {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        for e in back.sess.poll(ms) {
            if let Event::LateJoinHandover { slot, first_tick, snapshot_tick, snapshot_hash, log, .. } = e {
                handover = Some((slot, first_tick, snapshot_tick, snapshot_hash, log));
            }
        }
        assert!(ms < deadline, "the reconnect's handover completes");
    }
    let (slot, first_tick, snapshot_tick, snapshot_hash, log) = handover.unwrap();
    assert_eq!(slot, 2, "Bravo takes its own slot back, not the lowest free one (1)");
    assert_eq!(back.sess.local_slot(), 2);
    // The replay: the same chain over the host's log reproduces its snapshot.
    // (The last JOIN_LOG chunk may carry rows past the snapshot — the host kept running while it
    // served; the replay stops at the snapshot like `mp_late_join_catchup`.)
    for (i, row) in log.iter().enumerate() {
        let t = first_tick + i as u32;
        if t > snapshot_tick { break; }
        back.run(t, *row);
    }
    assert_eq!(back.prev_hash, snapshot_hash, "the replay verifies");
    assert_eq!(back.next_tick, snapshot_tick + 1);
    back.sess.confirm_late_join();
    // The join tick arms on all three.
    let mut jt = None;
    let deadline = ms + 10_000;
    while jt.is_none() {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            if let Event::LateJoinSlot { slot: 2, join_tick } = e { jt = Some(join_tick); }
        }
        let _ = back.frame(ms, false);
        assert!(ms < deadline, "the join tick is announced");
    }
    let join_tick = jt.unwrap();
    back.sample_from = Some(join_tick);
    let end = join_tick + 120;
    let deadline = ms + 30_000;
    while host.next_tick < end || clients[2].next_tick < end || back.next_tick < end {
        for (who, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            assert!(!matches!(e, Event::Desync { .. }), "no desync after the reconnect ({who}: {e:?})");
        }
        for e in back.frame(ms, false) {
            assert!(!matches!(e, Event::Desync { .. }), "the reconnected machine never desyncs: {e:?}");
        }
        assert!(ms < deadline, "the post-reconnect run");
    }
    assert_same_run(&host, &clients[2], 0, "host vs Charlie");
    assert_same_run(&host, &back, snapshot_tick + 1, "host vs the reconnected Bravo");
    // Non-vacuous: Bravo's real input drives slot 2 again from the join tick (sister: before it
    // the slot was NEUTRAL, so a wrong slot pick or a missing arm would show as a row mismatch).
    assert_eq!(host.rows[join_tick as usize][2], agent_input(2, join_tick), "slot 2 runs Bravo's input from the join tick");
    assert_eq!(host.rows[join_tick as usize - 1][2], NEUTRAL, "slot 2 was NEUTRAL just before");
    assert_eq!(host.sess.agents().len(), 3, "host + Charlie + Bravo");
}

/// The host's LEAVE reaches a client at once as `HostLeft` (not after the 5 s `HostTimeout`).
#[test]
fn the_host_leaving_is_reported_at_once() {
    let shim = lossy();
    let mut ms = 0u64;
    let (mut host, mut clients) = start_match(&shim, 9160, &["Alpha"], &mut ms);
    while host.next_tick < 60 || clients[0].next_tick < 60 {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < 30_000, "the opening run");
    }
    host.sess.leave();
    host.alive = false;
    let left_at = ms;
    let mut got = None;
    while got.is_none() {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            match e {
                Event::HostLeft => got = Some("left"),
                Event::HostTimeout => got = Some("timeout"),
                _ => {}
            }
        }
        assert!(ms < left_at + 10_000, "the client learns the host is gone");
    }
    assert_eq!(got, Some("left"), "the LEAVE is reported as HostLeft, not a timeout");
    assert!(ms - left_at < 1000, "at once (the link's latency), not the {} ms timeout: {} ms", test_cfg().timeout_ms, ms - left_at);
    assert!(clients[0].sess.host_left(), "the session remembers it");
}

/// A finished match answers no new HELLO (the rematch re-dial waits for the next lobby); the
/// known peers are unaffected. Sister: the same dial before `close_joins` begins a late join.
#[test]
fn a_finished_match_ignores_new_hellos() {
    let shim = lossy();
    let mut ms = 0u64;
    let (mut host, mut clients) = start_match(&shim, 9180, &["Alpha"], &mut ms);
    while host.next_tick < 60 || clients[0].next_tick < 60 {
        frame_all(&shim, &mut host, &mut clients, &mut ms);
        assert!(ms < 30_000, "the opening run");
    }
    host.sess.close_joins();
    let mut late = Session::new_client(test_cfg(), shim.endpoint(addr(9190)), addr(9180));
    let until = ms + 2000;
    let mut answered = false;
    while ms < until {
        for (_, e) in frame_all(&shim, &mut host, &mut clients, &mut ms) {
            assert!(!matches!(e, Event::LateJoinStarted { .. }), "the finished match serves no late join");
            assert!(!matches!(e, Event::ClientLeft { .. }), "the known client stays");
        }
        for e in late.poll(ms) {
            if matches!(e, Event::Connected { .. } | Event::Refused { .. }) { answered = true; }
        }
    }
    assert!(!answered && late.local_slot() == 0, "no WELCOME, no REFUSE: the dialer keeps retrying");
    assert_eq!(host.sess.agents().len(), 2, "the roster is untouched");

    // Sister: a host that has NOT closed joins begins the late join for the same dial.
    let shim2 = lossy();
    let mut ms2 = 0u64;
    let (mut host2, mut clients2) = start_match(&shim2, 9200, &["Alpha"], &mut ms2);
    while host2.next_tick < 60 || clients2[0].next_tick < 60 {
        frame_all(&shim2, &mut host2, &mut clients2, &mut ms2);
        assert!(ms2 < 30_000, "the sister's opening run");
    }
    let mut late2 = Session::new_client(test_cfg(), shim2.endpoint(addr(9210)), addr(9200));
    let until = ms2 + 2000;
    let mut began = false;
    while ms2 < until && !began {
        for (_, e) in frame_all(&shim2, &mut host2, &mut clients2, &mut ms2) {
            if matches!(e, Event::LateJoinStarted { .. }) { began = true; }
        }
        let _ = late2.poll(ms2);
    }
    assert!(began, "an open match answers the same dial with a late join");
}
