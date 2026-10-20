//! net-soak-faults repro (small): 2 machines, real UDP loopback, 5% loss +
//! 8% reorder on BOTH machines from tick 100 — the soak's phase 1 in
//! miniature. The lockstep must reach tick 400 with equal hashes.

use std::time::{Duration, Instant};

use nf_net::hash::Hasher;
use nf_net::net::{Transport, UdpTransport};
use nf_net::session::{Event, Session};
use nf_net::{Config, MatchSettings, NEUTRAL};

fn cfg() -> Config {
    Config { hash_every: 15, ..Default::default() }
}

fn row_hash(t: u32, row: &[nf_net::Input; nf_net::MAX_AGENTS]) -> u64 {
    let mut h = Hasher::new();
    h.u32(t);
    for i in row {
        h.f32(i.forward);
        h.f32(i.turn);
    }
    h.finish()
}

fn input(t: u32) -> nf_net::Input {
    let mut i = NEUTRAL;
    i.forward = ((t % 41) as f32) / 41.0;
    i.shot = t % 29 == 0;
    i
}

#[test]
fn five_percent_loss_and_reorder_keeps_the_lockstep_alive() {
    let port = 27975;
    let mut host = Session::new_host(cfg(), UdpTransport::bind(&format!("127.0.0.1:{port}")).unwrap(),
        MatchSettings::default(), 0x0BED_0BAD);
    let mut client = Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
        format!("127.0.0.1:{port}").parse().unwrap());
    let mut ms: u64 = 0;
    let deadline = Instant::now() + Duration::from_secs(20);
    while host.agents().len() < 2 {
        ms += 5;
        let _ = host.poll(ms);
        let _ = client.poll(ms);
        assert!(Instant::now() < deadline, "handshake");
        std::thread::sleep(Duration::from_millis(1));
    }
    host.set_host_match(MatchSettings::default(), 0x0BED_0BAD);
    host.start_match(0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !client.is_started() {
        ms += 5;
        let _ = host.poll(ms);
        let _ = client.poll(ms);
        assert!(Instant::now() < deadline, "GO");
        std::thread::sleep(Duration::from_millis(1));
    }

    const TICKS: u32 = 400;
    let mut host_next = 0u32;
    let mut client_next = 0u32;
    let mut host_hash = 0u64;
    let mut client_hash = 0u64;
    let stall_report = Instant::now() + Duration::from_secs(30);
    while host_next < TICKS || client_next < TICKS {
        if host_next == 100 && client_next >= 100 {
            // The soak's phase 1: 5% loss + 8% reorder, both machines' sends.
            host.inject_link_fault(0x50A1, 50, 80);
            client.inject_link_fault(0x50A2, 50, 80);
        }
        let t = host_next.min(client_next);
        if host_next == t {
            host.local_input(t, input(t));
        }
        if client_next == t {
            client.local_input(t, input(t));
        }
        loop {
            ms += 5;
            for e in host.poll(ms) {
                if let Event::Desync { .. } = e { panic!("host desync: {e:?}"); }
            }
            for e in client.poll(ms) {
                if let Event::Desync { .. } = e { panic!("client desync: {e:?}"); }
            }
            let hr = host.ready_inputs(t).is_some();
            let cr = client.ready_inputs(t).is_some();
            if hr && cr {
                break;
            }
            assert!(Instant::now() < stall_report,
                "tick {t} stuck (logical ms {ms}); host ready {hr} client ready {cr}");
            std::thread::sleep(Duration::from_millis(1));
        }
        let hrow = host.ready_inputs(t).unwrap();
        host_hash = row_hash(t, &hrow);
        host.submit_hash(t, host_hash);
        host_next = t + 1;
        let crow = client.ready_inputs(t).unwrap();
        client_hash = row_hash(t, &crow);
        client.submit_hash(t, client_hash);
        client_next = t + 1;
    }
    assert_eq!(host_hash, client_hash, "equal hashes to the end");
}
