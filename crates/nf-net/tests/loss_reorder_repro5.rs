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
fn four_machines_pads_and_late_joiners_under_loss_reorder() {
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
    // 10 pads join slots 2..=11 then leave (the soak's reclaim handles).
    let mut pads: Vec<Session<UdpTransport>> = Vec::new();
    for slot in 4..=13u8 {
        let mut d = Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
            format!("127.0.0.1:{port}").parse().unwrap());
        d.set_display_name(&format!("pad-{slot}"));
        let dl = Instant::now() + Duration::from_secs(20);
        loop {
            ms += 5;
            let _ = host.poll(ms);
            for c in clients.iter_mut() { let _ = c.poll(ms); }
            let _ = d.poll(ms);
            assert!(Instant::now() < dl, "pad {slot} joins");
            if d.local_slot() == slot { break; }
            std::thread::sleep(Duration::from_millis(1));
        }
        pads.push(d);
    }
    for (i, mut d) in pads.drain(..).enumerate() {
        d.leave();
        let dl = Instant::now() + Duration::from_secs(20);
        loop {
            ms += 5;
            let _ = host.poll(ms);
            for c in clients.iter_mut() { let _ = c.poll(ms); }
            let _ = d.poll(ms);
            assert!(Instant::now() < dl, "pad leaves");
            if host.agents().len() == 13 - i { break; }
            std::thread::sleep(Duration::from_millis(1));
        }
        drop(d);
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

    // Two late joiners reclaim slots 5 and 11 mid-match.
    struct Joiner { sess: Option<Session<UdpTransport>>, armed: bool }
    let mut j5 = Joiner { sess: None, armed: false };
    let mut j11 = Joiner { sess: None, armed: false };
    let mut dial = |j: &mut Joiner, name: &str, ms: u64| {
        let mut s = Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
            format!("127.0.0.1:{port}").parse().unwrap());
        s.set_display_name(name);
        let _ = s.poll(ms);
        j.sess = Some(s);
    };
    const TICKS: u32 = 600;
    let mut next = [0u32; 4];
    let mut faults_on = false;
    let stall_report = Instant::now() + Duration::from_secs(45);
    while next.iter().any(|&n| n < TICKS) {
        let t = *next.iter().min().unwrap();
        if t == 150 && j5.sess.is_none() { dial(&mut j5, "pad-5", ms); }
        if t == 300 && j11.sess.is_none() { dial(&mut j11, "pad-11", ms); }
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
        for j in [&mut j5, &mut j11] {
            if j.armed {
                if let Some(s) = j.sess.as_mut() {
                    let slot = s.local_slot();
                    s.local_input(t, input(t, slot));
                }
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
            for j in [&mut j5, &mut j11] {
                if let Some(s) = j.sess.as_mut() {
                    for e in s.poll(ms) {
                        if let Event::Desync { .. } = e { panic!("joiner desync: {e:?}"); }
                        if matches!(e, Event::LateJoinHandover { .. }) { s.confirm_late_join(); }
                        if matches!(e, Event::LateJoinSlot { .. }) { j.armed = true; }
                    }
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
        host.record_run_tick(t, hrow);
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
