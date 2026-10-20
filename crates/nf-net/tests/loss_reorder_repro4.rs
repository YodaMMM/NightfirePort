//! net-soak-faults repro (small): 4 machines, real UDP loopback, 5% loss +
//! 8% reorder on ALL FOUR from tick 100 — the soak's phase 1 shape without
//! sims. The lockstep must reach tick 600 with equal hashes on the simmed
//! pair (host + the first client).

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

fn input(t: u32, slot: u8) -> nf_net::Input {
    let mut i = NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 41) as f32 / 41.0;
    i.shot = t % 29 == slot as u32;
    i
}

#[test]
fn four_machines_under_loss_and_reorder_stay_in_sync() {
    let port = 27976;
    let mut host = Session::new_host(cfg(), UdpTransport::bind(&format!("127.0.0.1:{port}")).unwrap(),
        MatchSettings::default(), 0x0BED_0BAD);
    let mut clients: Vec<Session<UdpTransport>> = (0..3)
        .map(|k| {
            let mut c = Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
                format!("127.0.0.1:{port}").parse().unwrap());
            c.set_display_name(&format!("c{k}"));
            c
        })
        .collect();
    let mut ms: u64 = 0;
    let deadline = Instant::now() + Duration::from_secs(20);
    while host.agents().len() < 4 {
        ms += 5;
        let _ = host.poll(ms);
        for c in clients.iter_mut() { let _ = c.poll(ms); }
        assert!(Instant::now() < deadline, "handshake");
        std::thread::sleep(Duration::from_millis(1));
    }
    host.set_host_match(MatchSettings::default(), 0x0BED_0BAD);
    host.start_match(0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while clients.iter().any(|c| !c.is_started()) {
        ms += 5;
        let _ = host.poll(ms);
        for c in clients.iter_mut() { let _ = c.poll(ms); }
        assert!(Instant::now() < deadline, "GO");
        std::thread::sleep(Duration::from_millis(1));
    }

    const TICKS: u32 = 600;
    let mut next = [0u32; 4];
    let mut faults_on = false;
    let stall_report = Instant::now() + Duration::from_secs(45);
    while next.iter().any(|&n| n < TICKS) {
        let t = *next.iter().min().unwrap();
        if !faults_on && t >= 100 {
            faults_on = true;
            host.inject_link_fault(0x50A1, 50, 80);
            for (k, c) in clients.iter_mut().enumerate() {
                c.inject_link_fault(0x50A2 + k as u64, 50, 80);
            }
        }
        if next[0] == t {
            host.local_input(t, input(t, 0));
        }
        for (k, c) in clients.iter_mut().enumerate() {
            if next[k + 1] == t {
                c.local_input(t, input(t, (k + 1) as u8));
            }
        }
        loop {
            ms += 5;
            for e in host.poll(ms) {
                if let Event::Desync { .. } = e { panic!("host desync: {e:?}"); }
            }
            for c in clients.iter_mut() {
                for e in c.poll(ms) {
                    if let Event::Desync { .. } = e { panic!("client desync: {e:?}"); }
                }
            }
            let host_ready = host.ready_inputs(t).is_some();
            let clients_ready = clients.iter_mut().all(|c| c.ready_inputs(t).is_some());
            if host_ready && clients_ready {
                break;
            }
            assert!(Instant::now() < stall_report,
                "tick {t} stuck (logical ms {ms}); host ready {host_ready} clients ready {clients_ready}");
            std::thread::sleep(Duration::from_millis(1));
        }
        let hrow = host.ready_inputs(t).unwrap();
        host.submit_hash(t, row_hash(t, &hrow));
        next[0] = t + 1;
        for (k, c) in clients.iter_mut().enumerate() {
            let crow = c.ready_inputs(t).unwrap();
            if k == 0 {
                assert_eq!(crow, hrow, "tick {t}: the simmed pair holds the same row");
            }
            c.submit_hash(t, row_hash(t, &crow));
            next[k + 1] = t + 1;
        }
    }
}
