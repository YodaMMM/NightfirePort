//! net-internet: three real `NetCore`s (the app's session glue) over 127.0.0.1 with the
//! session's fault line on every core's sends — 50 ms one-way (100 ms ping), ±10 ms jitter,
//! 1 % loss — playing 12 s of real time in the app's online frame shape: poll once per frame,
//! the match clock's budget through `nf_net::pace::TickPacer`, `ready_inputs` gates each tick,
//! the local input goes through `NetCore::submit_local` (the adaptive delay), `flush` after the
//! ticks. Checks: every core used the same input delay at every run tick (the host's
//! announcement switched them at the same boundary), the delay adapted to the ping, stalls
//! stay rare after the first seconds, the per-tick hashes agree, and the bytes each core sent.
//! No disc needed (a hash chain stands in for the sim — the session layer is under test).

use std::time::{Duration, Instant};

use nf_view::mpnet::NetCore;

const PORT: u16 = 28169;

fn input(slot: usize, tick: u32) -> nf_net::Input {
    let mut i = nf_net::NEUTRAL;
    let x = (slot as u32 + 1).wrapping_mul(0x9E37_79B9) ^ tick.wrapping_mul(0x85EB_CA6B);
    i.forward = 1.0;
    i.turn = ((x & 0xFF) as f32 - 128.0) / 4096.0;
    i.jump = tick % 53 == slot as u32;
    i.held = 2;
    i
}

struct Core {
    core: std::rc::Rc<std::cell::RefCell<NetCore>>,
    clock: Option<Instant>,
    tick: u32,
    pacer: nf_net::pace::TickPacer,
    hash: u64,
    hashes: Vec<u64>,
    used: Vec<u32>,
    frames: u32,
    stalls: u32,
    bytes_at: Option<(u64, Instant)>,
}

#[test]
fn three_cores_over_a_100ms_link_adapt_and_agree() {
    let host = NetCore::host(PORT).expect("bind the host port");
    let clients: Vec<_> = (0..2).map(|_| NetCore::connect(&format!("127.0.0.1:{PORT}")).expect("client core")).collect();
    for (i, c) in std::iter::once(&host).chain(clients.iter()).enumerate() {
        c.borrow_mut().inject_link_delay(0x51ED + i as u64, 10, 45, 10);
    }
    let all: Vec<_> = std::iter::once(host.clone()).chain(clients.iter().cloned()).collect();
    // Lobby: every client takes a slot, then a second of pings.
    let deadline = Instant::now() + Duration::from_secs(15);
    while clients.iter().any(|c| c.borrow().local_slot() == 0) {
        for c in &all {
            c.borrow_mut().poll();
        }
        std::thread::sleep(Duration::from_millis(2));
        assert!(Instant::now() < deadline, "handshake over the slow link");
    }
    let lobby_end = Instant::now() + Duration::from_millis(1200);
    while Instant::now() < lobby_end {
        for c in &all {
            c.borrow_mut().poll();
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let mut s = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    s.human_count = 3;
    s.player_count = 3;
    s.total_players = 3;
    s.seed = 0x1e7_5eed;
    host.borrow_mut().start_match(&s);
    let mut cores: Vec<Core> = all
        .iter()
        .map(|c| Core { core: c.clone(), clock: None, tick: 0, pacer: Default::default(), hash: 0, hashes: vec![], used: vec![], frames: 0, stalls: 0, bytes_at: None })
        .collect();
    let t0 = Instant::now();
    let frame = Duration::from_micros(16_667);
    let mut next = t0;
    while t0.elapsed() < Duration::from_secs(12) {
        next += frame;
        while Instant::now() < next {
            std::thread::sleep(Duration::from_millis(1));
        }
        let measuring = t0.elapsed() > Duration::from_secs(4);
        for m in cores.iter_mut() {
            m.core.borrow_mut().poll();
            m.core.borrow_mut().take_app_events();
            if m.clock.is_none() && m.core.borrow().match_started().is_some() {
                m.clock = Some(Instant::now());
                let lead = m.core.borrow().input_delay();
                for t in 0..lead {
                    m.core.borrow_mut().local_input(t, nf_net::NEUTRAL);
                }
            }
            let Some(c) = m.clock else { continue };
            let target = (c.elapsed().as_secs_f64() * 60.0) as u32;
            let n = m.pacer.ticks_for_frame(target, m.tick, frame.as_secs_f64(), 4);
            let mut stalled = false;
            for _ in 0..n {
                let ready = m.core.borrow_mut().ready_inputs(m.tick);
                let Some(row) = ready else {
                    stalled = true;
                    break;
                };
                let slot = m.core.borrow().local_slot();
                m.used.push(m.core.borrow().delay_at(m.tick));
                m.core.borrow_mut().submit_local(m.tick, input(slot, m.tick));
                for (k, i) in row.iter().enumerate() {
                    m.hash = m.hash.rotate_left(9) ^ (i.turn.to_bits() as u64).wrapping_add(k as u64) ^ (i.jump as u64) << 3;
                }
                m.hash ^= m.tick as u64;
                m.core.borrow_mut().submit_hash(m.tick, m.hash);
                m.hashes.push(m.hash);
                m.tick += 1;
            }
            m.core.borrow_mut().flush();
            if measuring {
                m.frames += 1;
                if stalled {
                    m.stalls += 1;
                }
                if m.bytes_at.is_none() {
                    m.bytes_at = Some((m.core.borrow().net_stats().2, Instant::now()));
                }
            }
        }
    }
    let common = cores.iter().map(|m| m.hashes.len()).min().unwrap();
    assert!(common >= 600, "the match ran ({common} ticks in 12 s)");
    for (i, m) in cores.iter().enumerate() {
        let (rtt, delay, bytes, pkts) = m.core.borrow().net_stats();
        let (b0, at) = m.bytes_at.unwrap();
        let bps = (bytes - b0) as f64 / at.elapsed().as_secs_f64();
        eprintln!(
            "core {i}: ran {} ticks, stall frames {}/{} ({:.1}%), rtt {:?} ms, delay {delay}, schedule {:?}, {:.0} B/s payload ({pkts} pkts total)",
            m.tick, m.stalls, m.frames, m.stalls as f64 * 100.0 / m.frames.max(1) as f64, rtt.map(|r| r.round()), m.core.borrow().delay_schedule(), bps
        );
        assert_eq!(&m.hashes[..common], &cores[0].hashes[..common], "core {i}: per-tick hashes agree");
        assert_eq!(&m.used[..common], &cores[0].used[..common], "core {i}: the same delay at every run tick");
        assert!(m.stalls as f64 <= m.frames as f64 * 0.05, "core {i}: stalls stay rare after the start ({} of {})", m.stalls, m.frames);
        assert!(bps < 20_000.0, "core {i}: {bps:.0} B/s");
    }
    let d = cores[0].used[common - 1];
    // 100 ms ping + frame pacing: the measured need is ~7 ticks (see net-lockstep.md); the
    // fixed default was 3, which a 100 ms link cannot hold (nf-net internet_link's sister).
    assert!(d >= 6 && d <= 12, "the delay adapted to the 100 ms link: {d}");
    for c in &all {
        c.borrow_mut().close();
    }
}
