//! net-internet: the lockstep over a simulated internet link — real 127.0.0.1 UDP sockets with
//! a seeded fault line in front of each socket (one-way delay = ping/2, uniform jitter, loss,
//! the reordering jitter causes), driven on a logical millisecond clock so a 30-second match
//! runs in well under a second and every run is reproducible from its seed.
//!
//! Each machine runs the app's frame loop shape (`nf-view` lib.rs, the online tick): poll once
//! per frame, the match clock asks for `elapsed · 60` ticks, at most 4 run per frame, a tick
//! runs only when `ready_inputs` has every input, the local input sampled while running tick
//! `t` is scheduled ahead (`Session::submit_local`). Measured per machine: stall frames (a
//! frame the clock wanted a tick and the gate said no), catch-up frames (more than one tick
//! in one 60 Hz frame — the visible fast-forward), the match lag behind the clock, and the
//! bytes each socket sent per second (+28 B UDP/IPv4 header per datagram).

use std::cell::{Cell, RefCell};
use std::net::{SocketAddr, UdpSocket};
use std::rc::Rc;

use nf_net::net::Transport;
use nf_net::session::{Event, Session};
use nf_net::{MatchSettings, NEUTRAL};

/// The simulated internet between the sockets: every datagram that survives the loss dice
/// waits `delay + uniform(0..=jitter)` ms in one shared queue and is released by whichever
/// machine touches the network next at or after that time (released from the SENDER's socket,
/// so the receiver sees the right source) — the path delivers continuously, not on the
/// sender's frame.
struct Wire {
    queue: Vec<(u64, usize, SocketAddr, Vec<u8>)>,
    socks: Vec<UdpSocket>,
    clock: Rc<Cell<u64>>,
}

impl Wire {
    fn pump(&mut self) {
        let now = self.clock.get();
        let mut i = 0;
        while i < self.queue.len() {
            if self.queue[i].0 <= now {
                let (_, from, to, d) = self.queue.swap_remove(i);
                let _ = self.socks[from].send_to(&d, to);
            } else {
                i += 1;
            }
        }
    }
}

/// One machine's socket on the [`Wire`]: drop at send, then the shared delay queue.
struct Link {
    sock: UdpSocket,
    idx: usize,
    wire: Rc<RefCell<Wire>>,
    rng: u64,
    drop_per_mille: u32,
    delay_ms: u64,
    jitter_ms: u64,
    scratch: Vec<u8>,
    bytes: Rc<Cell<u64>>,
    datagrams: Rc<Cell<u64>>,
}

impl Link {
    fn bind(addr: &str, wire: &Rc<RefCell<Wire>>, seed: u64, drop_per_mille: u32, delay_ms: u64, jitter_ms: u64) -> Link {
        let sock = UdpSocket::bind(addr).unwrap();
        sock.set_nonblocking(true).unwrap();
        let idx = {
            let mut w = wire.borrow_mut();
            w.socks.push(sock.try_clone().unwrap());
            w.socks.len() - 1
        };
        Link {
            sock,
            idx,
            wire: wire.clone(),
            rng: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5DEE_CE66,
            drop_per_mille,
            delay_ms,
            jitter_ms,
            scratch: vec![0u8; 2048],
            bytes: Rc::new(Cell::new(0)),
            datagrams: Rc::new(Cell::new(0)),
        }
    }
    fn rand(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl Transport for Link {
    fn send(&mut self, to: SocketAddr, data: &[u8]) {
        self.bytes.set(self.bytes.get() + data.len() as u64 + 28);
        self.datagrams.set(self.datagrams.get() + 1);
        if self.drop_per_mille > 0 && self.rand() % 1000 < self.drop_per_mille as u64 {
            return;
        }
        let j = if self.jitter_ms > 0 { self.rand() % (self.jitter_ms + 1) } else { 0 };
        let mut w = self.wire.borrow_mut();
        let at = w.clock.get() + self.delay_ms + j;
        w.queue.push((at, self.idx, to, data.to_vec()));
        w.pump();
    }
    fn try_recv(&mut self) -> Option<(SocketAddr, Vec<u8>)> {
        self.wire.borrow_mut().pump();
        match self.sock.recv_from(&mut self.scratch) {
            Ok((n, from)) => Some((from, self.scratch[..n].to_vec())),
            Err(_) => None,
        }
    }
    fn local_addr(&self) -> SocketAddr {
        self.sock.local_addr().unwrap()
    }
}

/// The scripted controls: sticks and mouse change every tick (the worst case for bandwidth —
/// a player always turning), buttons now and then.
fn agent_input(slot: u8, tick: u32) -> nf_net::Input {
    let mut i = NEUTRAL;
    let x = (slot as u32 + 1).wrapping_mul(0x9E37_79B9) ^ tick.wrapping_mul(0x85EB_CA6B);
    i.forward = if (tick / 90 + slot as u32) % 3 == 0 { 0.0 } else { 1.0 };
    i.strafe = if (tick / 50) % 4 == 1 { 0.5 } else { 0.0 };
    i.turn = ((x & 0xFF) as f32 - 128.0) / 4096.0;
    i.look = (((x >> 8) & 0xFF) as f32 - 128.0) / 8192.0;
    i.fire = (tick / 20) % 5 == 0;
    i.trigger = i.fire;
    i.jump = tick % 137 == slot as u32;
    i.held = 2;
    i
}

#[derive(Clone, Debug, Default)]
struct MachineStats {
    frames: u32,
    stall_frames: u32,
    catchup_frames: u32,
    max_ticks_in_frame: u32,
    lag_sum_ticks: u64,
    max_lag_ticks: u32,
    executed: u32,
    bytes: u64,
    datagrams: u64,
    final_delay: u32,
}

struct Machine {
    sess: Session<Link>,
    clock: Rc<Cell<u64>>,
    bytes: Rc<Cell<u64>>,
    datagrams: Rc<Cell<u64>>,
    started_ms: Option<u64>,
    executed: u32,
    prev_hash: u64,
    hashes: Vec<u64>,
    stats: MachineStats,
    pacer: nf_net::pace::TickPacer,
}

#[derive(Clone, Copy, Debug)]
struct Scenario {
    peers: usize,
    ping_ms: u64,
    jitter_ms: u64,
    loss_per_mille: u32,
    frame_hz: u32,
    secs: u32,
    seed: u64,
    port: u16,
    /// The session's adaptive input delay (else the fixed 3-tick lead).
    adaptive: bool,
    /// The app's stall smoothing (`nf_net::pace::TickPacer`; else `min(backlog, 4)`).
    pacer: bool,
}

fn sim_hash(tick: u32, inputs: &[nf_net::Input; nf_net::MAX_AGENTS], prev: u64) -> u64 {
    let mut h = prev ^ 0xC0FF_EE00 ^ tick as u64;
    for (s, i) in inputs.iter().enumerate() {
        h = h.rotate_left(17) ^ (i.turn.to_bits() as u64).wrapping_add(s as u64 + 1);
        h = h.rotate_left(13) ^ (i.fire as u64).wrapping_mul(31) ^ (i.jump as u64) << 7;
        h = h.rotate_left(7) ^ i.forward.to_bits() as u64;
    }
    h
}

fn cfg(adaptive: bool) -> nf_net::Config {
    let red = REDUNDANCY.with(|r| r.get());
    let mut c = nf_net::Config { hash_every: 15, adaptive_delay: adaptive, ..Default::default() };
    if red > 0 {
        c.redundancy = red;
    }
    c
}

thread_local! {
    static REDUNDANCY: Cell<u32> = const { Cell::new(0) };
}

fn run_with(sc: Scenario, redundancy: u32) -> Vec<MachineStats> {
    REDUNDANCY.with(|r| r.set(redundancy));
    let st = run(sc);
    REDUNDANCY.with(|r| r.set(0));
    st
}

/// Run one scenario; returns every machine's stats (host first). Panics on a desync.
fn run(sc: Scenario) -> Vec<MachineStats> {
    let clock = Rc::new(Cell::new(0u64));
    let wire = Rc::new(RefCell::new(Wire { queue: Vec::new(), socks: Vec::new(), clock: clock.clone() }));
    let one_way = sc.ping_ms / 2;
    let jit = sc.jitter_ms / 2;
    let mk = |addr: &str, seed: u64| Link::bind(addr, &wire, seed, sc.loss_per_mille, one_way, jit);
    let host_link = mk(&format!("127.0.0.1:{}", sc.port), sc.seed);
    let mut machines: Vec<Machine> = Vec::new();
    let (hb, hd) = (host_link.bytes.clone(), host_link.datagrams.clone());
    machines.push(Machine {
        sess: Session::new_host(cfg(sc.adaptive), host_link, MatchSettings::default(), 0x1234),
        clock: clock.clone(),
        bytes: hb,
        datagrams: hd,
        started_ms: None,
        executed: 0,
        prev_hash: 0,
        hashes: Vec::new(),
        stats: MachineStats::default(),
        pacer: nf_net::pace::TickPacer::new(),
    });
    let host_addr: SocketAddr = format!("127.0.0.1:{}", sc.port).parse().unwrap();
    for i in 1..sc.peers {
        let l = mk("127.0.0.1:0", sc.seed.wrapping_add(i as u64 * 7919));
        let (b, d) = (l.bytes.clone(), l.datagrams.clone());
        machines.push(Machine {
            sess: Session::new_client(cfg(sc.adaptive), l, host_addr),
            clock: clock.clone(),
            bytes: b,
            datagrams: d,
            started_ms: None,
            executed: 0,
            prev_hash: 0,
            hashes: Vec::new(),
            stats: MachineStats::default(),
            pacer: nf_net::pace::TickPacer::new(),
        });
    }
    // Lobby: every client gets a slot; then a second of lobby time (pings settle).
    let mut ms = 0u64;
    let mut lobby_ms = 0u64;
    loop {
        ms += 5;
        clock.set(ms);
        for m in machines.iter_mut() {
            m.sess.poll(ms);
        }
        let all = machines.iter().skip(1).all(|m| m.sess.local_slot() != 0);
        if all {
            lobby_ms += 5;
            if lobby_ms >= 1000 {
                break;
            }
        }
        assert!(ms < 60_000, "handshake must complete");
    }
    machines[0].sess.start_match(0).unwrap();
    machines[0].started_ms = Some(ms);
    prefill(&mut machines[0]);
    let frame_ms = 1000.0 / sc.frame_hz as f64;
    let start_ms = ms;
    let end_ms = ms + sc.secs as u64 * 1000 + 2000;
    let measure_from = ms + 2000; // the first 2 s settle (delay adaption, clock alignment)
    // Each machine's frames run at its own phase (machines are not frame-locked).
    let mut next_frame: Vec<f64> =
        (0..machines.len()).map(|i| start_ms as f64 + frame_ms * i as f64 / machines.len() as f64).collect();
    let mut base: Vec<(u64, u64)> = vec![(0, 0); machines.len()];
    let mut based = false;
    let mut now = ms;
    while now < end_ms {
        now += 1;
        clock.set(now);
        wire.borrow_mut().pump();
        if !based && now >= measure_from {
            based = true;
            for (i, m) in machines.iter().enumerate() {
                base[i] = (m.bytes.get(), m.datagrams.get());
            }
        }
        for (i, m) in machines.iter_mut().enumerate() {
            if (now as f64) < next_frame[i] {
                continue;
            }
            next_frame[i] += frame_ms;
            let evs = m.sess.poll(now);
            for e in evs {
                match e {
                    Event::MatchStarted { .. } => {
                        m.started_ms = Some(now);
                        prefill(m);
                    }
                    Event::Desync { tick, .. } => panic!("desync at tick {tick}"),
                    _ => {}
                }
            }
            let Some(st) = m.started_ms else { continue };
            let target = ((now - st) as f64 * 60.0 / 1000.0) as u32;
            let budget = if sc.pacer {
                m.pacer.ticks_for_frame(target, m.executed, frame_ms / 1000.0, 4)
            } else {
                target.saturating_sub(m.executed).min(4)
            };
            let mut ran = 0;
            let mut stalled = false;
            for _ in 0..budget {
                match m.sess.ready_inputs(m.executed) {
                    None => {
                        stalled = true;
                        if now >= measure_from && std::env::var_os("INTERNET_DBG").is_some() {
                            eprintln!("  stall m{i} at {} ms tick {} target {} delay {} | {}", now - start_ms, m.executed, target, m.sess.input_delay(), m.sess.debug_state());
                        }
                        break;
                    }
                    Some(inputs) => {
                        let slot = m.sess.local_slot();
                        let inp = agent_input(slot, m.executed);
                        if sc.adaptive {
                            m.sess.submit_local(m.executed, inp);
                        } else {
                            schedule(&mut m.sess, m.executed, inp);
                        }
                        let h = sim_hash(m.executed, &inputs, m.prev_hash);
                        m.sess.submit_hash(m.executed, h);
                        m.prev_hash = h;
                        m.hashes.push(h);
                        m.executed += 1;
                        ran += 1;
                    }
                }
            }
            // The app pumps once more after its ticks (the inputs just scheduled leave now).
            flush(&mut m.sess, now);
            if now >= measure_from {
                let s = &mut m.stats;
                s.frames += 1;
                if stalled {
                    s.stall_frames += 1;
                }
                // A burst: more ticks in one frame than the frame's own share + 1 (60 Hz: 3+;
                // 240 Hz: 2+) — the world visibly jumps.
                let share = (60.0 / sc.frame_hz as f64).ceil() as u32;
                if ran > share + 1 {
                    s.catchup_frames += 1;
                }
                s.max_ticks_in_frame = s.max_ticks_in_frame.max(ran);
                let lag = target.saturating_sub(m.executed);
                s.lag_sum_ticks += lag as u64;
                s.max_lag_ticks = s.max_lag_ticks.max(lag);
            }
        }
    }
    let span_s = (end_ms - measure_from) as f64 / 1000.0;
    // Hash agreement over the common prefix.
    let common = machines.iter().map(|m| m.hashes.len()).min().unwrap();
    for m in machines.iter().skip(1) {
        assert_eq!(&m.hashes[..common], &machines[0].hashes[..common], "per-tick hashes agree");
    }
    machines
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut s = m.stats.clone();
            s.executed = m.executed;
            s.bytes = ((m.bytes.get() - base[i].0) as f64 / span_s) as u64;
            s.datagrams = ((m.datagrams.get() - base[i].1) as f64 / span_s) as u64;
            s.final_delay = m.sess.input_delay();
            s
        })
        .collect()
}

fn prefill(m: &mut Machine) {
    let lead = m.sess.input_delay();
    for t in 0..lead {
        m.sess.local_input(t, NEUTRAL);
    }
}

/// The app's scheduling of the local input sampled while running `tick`.
fn schedule(sess: &mut Session<Link>, tick: u32, input: nf_net::Input) {
    let lead = sess.input_delay();
    sess.local_input(tick + lead, input);
}

fn flush(sess: &mut Session<Link>, now: u64) {
    sess.flush(now);
}

fn summary(label: &str, sc: &Scenario, st: &[MachineStats]) -> String {
    let worst = st.iter().map(|s| s.stall_frames as f64 / s.frames.max(1) as f64).fold(0.0, f64::max);
    let catch = st.iter().map(|s| s.catchup_frames as f64 / s.frames.max(1) as f64).fold(0.0, f64::max);
    let lag = st.iter().map(|s| s.lag_sum_ticks as f64 / s.frames.max(1) as f64).fold(0.0, f64::max);
    let maxlag = st.iter().map(|s| s.max_lag_ticks).max().unwrap_or(0);
    let maxrun = st.iter().map(|s| s.max_ticks_in_frame).max().unwrap_or(0);
    let exec = st.iter().map(|s| s.executed).min().unwrap_or(0);
    let delay = st.iter().map(|s| s.final_delay).max().unwrap_or(0);
    format!(
        "{label}{}: peers {} ping {} jitter {} loss {}‰ {} Hz | stall frames {:.1}% burst frames {:.1}% \
         mean lag {:.1} ticks max lag {} max ticks/frame {} | ran {} ticks | delay {} | host {} B/s ({} pkt/s) client {} B/s ({} pkt/s)",
        if sc.adaptive { " [adaptive+pacer]" } else { " [fixed 3]" }, sc.peers, sc.ping_ms, sc.jitter_ms, sc.loss_per_mille, sc.frame_hz,
        worst * 100.0, catch * 100.0, lag, maxlag, maxrun, exec, delay,
        st[0].bytes, st[0].datagrams,
        st.get(1).map_or(0, |s| s.bytes), st.get(1).map_or(0, |s| s.datagrams),
    )
}

/// The worst machine's stall-frame share, burst-frame share and the fewest ticks any machine
/// ran (the clock asked for `secs + 2` seconds of 60 Hz ticks).
fn worst(st: &[MachineStats]) -> (f64, f64, u32) {
    let stall = st.iter().map(|s| s.stall_frames as f64 / s.frames.max(1) as f64).fold(0.0, f64::max);
    let burst = st.iter().map(|s| s.catchup_frames as f64 / s.frames.max(1) as f64).fold(0.0, f64::max);
    (stall, burst, st.iter().map(|s| s.executed).min().unwrap_or(0))
}

/// Stall rate vs ping, before (the fixed 3-tick lead, the old `min(backlog, 4)` budget) and
/// after (the adaptive delay + the pacer), at 60 and 240 Hz frames, 1 % loss each way.
#[test]
fn stall_rate_vs_ping() {
    let mut port = 28160u16;
    for (adaptive, hz) in [(false, 60), (true, 60), (true, 240)] {
        for &(ping, jitter) in &[(0u64, 0u64), (50, 10), (100, 20), (150, 30)] {
            let sc = Scenario { peers: 3, ping_ms: ping, jitter_ms: jitter, loss_per_mille: 10, frame_hz: hz, secs: 20, seed: 7, port, adaptive, pacer: adaptive };
            port = if port >= 28163 { 28160 } else { port + 1 };
            if let Ok(f) = std::env::var("INTERNET_ONLY") {
                if f != format!("{ping}/{hz}/{adaptive}") { continue; }
            }
            let st = run(sc);
            eprintln!("{}", summary("ping", &sc, &st));
            let (stall, burst, ran) = worst(&st);
            let asked = (sc.secs + 2) * 60;
            if adaptive {
                assert!(stall <= 0.03, "adaptive delay: stalls stay rare at {ping} ms ({:.1}% of frames)", stall * 100.0);
                assert!(burst <= 0.01, "the pacer: no visible fast-forward bursts ({:.1}%)", burst * 100.0);
                assert!(ran as f64 >= asked as f64 * 0.97, "the match keeps real time at {ping} ms: {ran} of {asked} ticks");
            } else if ping >= 100 {
                // Sister: the fixed 3-tick lead cannot hold a 100 ms link — the harness sees it.
                assert!((ran as f64) < asked as f64 * 0.6, "the fixed lead stalls at {ping} ms: {ran} of {asked}");
            }
        }
    }
}

/// 5 % loss each way at 100 ms: the measured loss raises the redundancy (+1 at ≥ 4 %
/// round-trip loss); the configured 3 for comparison.
#[test]
fn heavy_loss() {
    for (i, &(loss, red)) in [(50u32, 2u32), (50, 3)].iter().enumerate() {
        let sc = Scenario { peers: 3, ping_ms: 100, jitter_ms: 20, loss_per_mille: loss, frame_hz: 60, secs: 20, seed: 5, port: 28164 + i as u16, adaptive: true, pacer: true };
        let st = run_with(sc, red);
        eprintln!("{} | redundancy {red}", summary("loss", &sc, &st));
        let (stall, _, ran) = worst(&st);
        assert!(stall <= 0.10, "5 % loss: stalls {:.1}%", stall * 100.0);
        assert!(ran as f64 >= (sc.secs + 2) as f64 * 60.0 * 0.95, "5 % loss keeps time: {ran}");
    }
}

/// Bytes per second at 16 participants (15 clients), every player turning every tick (the
/// worst case for the compact inputs), 60 ms ping, 1 % loss.
#[test]
fn bandwidth_16_participants() {
    let sc = Scenario { peers: 16, ping_ms: 60, jitter_ms: 10, loss_per_mille: 10, frame_hz: 60, secs: 20, seed: 11, port: 28166, adaptive: true, pacer: true };
    let st = run(sc);
    eprintln!("{}", summary("bandwidth", &sc, &st));
    let client_max = st.iter().skip(1).map(|s| s.bytes).max().unwrap();
    assert!(st[0].bytes < 600_000, "host upload at 16 participants: {} B/s", st[0].bytes);
    assert!(client_max < 16_000, "client upload: {client_max} B/s");
    let (stall, _, ran) = worst(&st);
    assert!(stall <= 0.05, "16 participants: stalls {:.1}%", stall * 100.0);
    assert!(ran as f64 >= (sc.secs + 2) as f64 * 60.0 * 0.97, "16 participants keep time: {ran}");
}
