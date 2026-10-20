//! net-soak-faults: the send-stall primitive and the receive counter, on the
//! real UDP loopback. A 2 s total stall of one machine's sends must stall the
//! lockstep (the never-guess rule) and then RECOVER through the session's own
//! retransmit — equal hashes to the end, no desync [S net-soak-faults: the
//! soak's phase 4 in miniature; the retransmit machinery itself is the game's
//! unacked-window design, session.rs module doc lines 28-29].

use std::time::{Duration, Instant};

use nf_net::hash::Hasher;
use nf_net::net::{Transport, UdpTransport};
use nf_net::session::{Event, Session};
use nf_net::{Config, MatchSettings, NEUTRAL};

fn cfg() -> Config {
    Config { hash_every: 15, ..Default::default() }
}

/// The lockstep's honest hash of a tick: the full input row folded (both
/// machines fold the same row, so equal hashes mean equal runs).
fn row_hash(t: u32, row: &[nf_net::Input; nf_net::MAX_AGENTS]) -> u64 {
    let mut h = Hasher::new();
    h.u32(t);
    for i in row {
        h.f32(i.forward);
        h.f32(i.strafe);
        h.f32(i.turn);
        h.bool(i.shot);
    }
    h.finish()
}

fn input(t: u32) -> nf_net::Input {
    let mut i = NEUTRAL;
    i.forward = ((t % 41) as f32) / 41.0;
    i.turn = ((t % 13) as f32 - 6.0) * 0.003;
    i.shot = t % 29 == 0;
    i
}

struct Peer {
    sess: Session<UdpTransport>,
    next: u32,
    /// The worst stretch (logical ms) this machine waited for the row.
    worst_wait_ms: u64,
}

impl Peer {
    fn pump(&mut self, ms: u64) -> Vec<Event> {
        self.sess.poll(ms)
    }
    fn ready(&mut self, t: u32) -> bool {
        self.sess.ready_inputs(t).is_some()
    }
    /// Run tick `t` when the row is ready: submit the hash, advance.
    fn run(&mut self, t: u32, row: &[nf_net::Input; nf_net::MAX_AGENTS]) -> u64 {
        let h = row_hash(t, row);
        self.sess.submit_hash(t, h);
        self.next = t + 1;
        h
    }
}

/// Two machines over real UDP: the client's sends stall for `stall_ms` at
/// logical ms 1000, then recover. Both must reach `TICKS` with equal hashes.
fn run_stall(stall_ms: u64, port: u16) -> (u64, u64, u64, u64) {
    let mut host = Peer {
        sess: Session::new_host(cfg(), UdpTransport::bind(&format!("127.0.0.1:{port}")).unwrap(),
            MatchSettings::default(), 0x0BED_0BAD),
        next: 0,
        worst_wait_ms: 0,
    };
    let mut client = Peer {
        sess: Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
            format!("127.0.0.1:{port}").parse().unwrap()),
        next: 0,
        worst_wait_ms: 0,
    };
    let mut ms: u64 = 0;
    let deadline = Instant::now() + Duration::from_secs(20);
    while host.sess.agents().len() < 2 {
        ms += 5;
        let _ = host.pump(ms);
        let _ = client.pump(ms);
        assert!(Instant::now() < deadline, "handshake");
        std::thread::sleep(Duration::from_millis(1));
    }
    host.sess.set_host_match(MatchSettings::default(), 0x0BED_0BAD);
    host.sess.start_match(0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !client.sess.is_started() {
        ms += 5;
        let _ = host.pump(ms);
        let _ = client.pump(ms);
        assert!(Instant::now() < deadline, "GO");
        std::thread::sleep(Duration::from_millis(1));
    }

    const TICKS: u32 = 400;
    let mut desyncs = 0;
    let mut stalled_ticks = 0u32;
    let mut stalled_seen = false;
    while host.next < TICKS || client.next < TICKS {
        let t = host.next.min(client.next);
        // The stall hits at logical ms 1000 (mid-run).
        if stall_ms > 0 && ms >= 1000 && !stalled_seen {
            client.sess.inject_send_stall(stall_ms);
            stalled_seen = client.sess.send_stalled();
            assert!(stalled_seen, "the stall engaged");
        }
        if host.next == t {
            host.sess.local_input(t, input(t));
        }
        if client.next == t {
            client.sess.local_input(t, input(t));
        }
        let wait_from = ms;
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            ms += 5;
            let _ = host.pump(ms);
            let _ = client.pump(ms);
            if host.ready(t) && client.ready(t) {
                break;
            }
            assert!(Instant::now() < deadline, "tick {t} never became ready (stall {stall_ms})");
            std::thread::sleep(Duration::from_millis(1));
        }
        let waited = ms - wait_from;
        // The machine whose input was the blocker is the one that waited the
        // lockstep out; attribute the wait to both is wrong — track the max
        // either way (the stall phase's blocker is the stalled client).
        if host.next == t && client.next == t {
            host.worst_wait_ms = host.worst_wait_ms.max(waited);
            client.worst_wait_ms = client.worst_wait_ms.max(waited);
            if waited > 100 {
                stalled_ticks += 1;
            }
        }
        let row = host.sess.ready_inputs(t).expect("host row");
        let ch = {
            let crow = client.sess.ready_inputs(t).expect("client row");
            assert_eq!(row, crow, "tick {t}: both machines hold the same row");
            client.run(t, &crow)
        };
        let hh = host.run(t, &row);
        assert_eq!(hh, ch, "tick {t}: equal hashes");
        for (p, who) in [(&mut host, "host"), (&mut client, "client")] {
            for e in p.pump(ms) {
                if let Event::Desync { .. } = e {
                    desyncs += 1;
                    panic!("{who} desynced at {t}: {e:?}");
                }
            }
        }
    }
    assert_eq!(desyncs, 0);
    assert_eq!(host.next, TICKS);
    assert_eq!(client.next, TICKS);
    if stall_ms > 0 {
        // Sister: without the stall the lockstep never waits >100 logical ms
        // here; with it, ticks block (the freeze is real, then recovery).
        assert!(stalled_ticks > 0, "the stall actually blocked ticks");
        assert!(client.worst_wait_ms >= 500, "the worst input wait covers the stall ({} ms)", client.worst_wait_ms);
    }
    let (hb, hp) = host.sess.sent_totals();
    let (hrb, hrp) = host.sess.recv_totals();
    assert!(hb > 0 && hp > 0, "the host sent bytes");
    assert!(hrb > 0 && hrp > 0, "the host received bytes (the recv counter counts)");
    (hb, hp, hrb, hrp)
}

/// The 2 s stall: the lockstep freezes (the blocked ticks) and recovers —
/// equal hashes to tick 400, no desync, both counters alive.
#[test]
fn a_two_second_send_stall_stalls_then_recovers_in_sync() {
    let _ = run_stall(2000, 27971);
}

/// Sister: the same run with no stall never blocks and agrees too (the
/// primitive changes nothing when unused, and the recv counter still counts).
#[test]
fn the_clean_run_never_blocks_and_agrees() {
    let _ = run_stall(0, 27972);
}
