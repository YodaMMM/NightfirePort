//! mp-join-in-progress: the app-level late-join live test — two sims playing
//! over real 127.0.0.1 UDP, a third joining at tick N: it loads the arena
//! from the host's handover, fast-forwards the recorded run log headless
//! (`mp_late_join_catchup`), verifies the snapshot hash, confirms, and the
//! three sims run on with all per-tick state hashes agreeing — the joined
//! slot's agent spawned at the announced tick the game's own MP way on every
//! machine. NF_ISO-gated (the arena loads from the disc like
//! `net_match_live`); serial (three full arena sims per test, a 3 GB test machine
//! OOMs on parallel runs).

use std::time::Instant;

use nf_game::mp::MpSettings;
use nf_net::net::UdpTransport;
use nf_net::session::{Event, LeaveReason, Session};
use nf_net::{MatchSettings, NEUTRAL};

use nf_view::mpnet::{mp_to_match, match_to_mp, to_player_input};

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn scripted_input(t: u32, slot: u8) -> nf_net::Input {
    let mut i = NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 97) as f32 / 97.0;
    i.strafe = if t % 11 == 0 { -0.5 } else { 0.0 };
    i.turn = ((t % 13) as f32 - 6.0) * 0.002;
    i.look = if t % 29 == 0 { 0.25 } else { 0.0 };
    i.jump = t % 61 == 0;
    i.crouch = t % 43 == 0;
    i.fire = t % 23 == 0;
    i
}

fn cfg() -> nf_net::session::Config {
    nf_net::session::Config { hash_every: 15, ..Default::default() }
}


struct Peer {
    sess: Session<UdpTransport>,
    sim: Option<nf_view::scene::Scene>,
    /// The machine's own net tick (the app's `net_tick`): the last tick the
    /// sim RAN + 1. The joiner's starts at the snapshot + 1 (its sim was
    /// fast-forwarded there).
    next_tick: u32,
    last_hash: u64,
    /// mp-join-in-progress: the announced (slot, join tick) — the machine
    /// spawns the slot's agent at that tick (the app's `net_join_pending`).
    join: Option<(u8, u32)>,
}

impl Peer {
    fn host(port: u16) -> Peer {
        Peer {
            sess: Session::new_host(cfg(), UdpTransport::bind(&format!("127.0.0.1:{port}")).unwrap(),
                MatchSettings::default(), 0),
            sim: None, next_tick: 0, last_hash: 0, join: None,
        }
    }
    fn client(port: u16, dial: u16) -> Peer {
        Peer {
            sess: Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
                format!("127.0.0.1:{dial}").parse().unwrap()),
            sim: None, next_tick: 0, last_hash: 0, join: None,
        }
    }
    /// One frame: sample this machine's input for its next tick, pump, run
    /// the ready tick through the sim driver, record (host) + submit the
    /// hash. `sample` = whether the machine produces its own input for this
    /// tick (the joiner does not before its join tick — its slot is
    /// truthfully NEUTRAL there, the session's join_from gate agrees).
    fn step(&mut self, ms: &mut u64, sample: bool) -> Vec<Event> {
        let t = self.next_tick;
        if sample {
            self.sess.local_input(t, scripted_input(t, self.sess.local_slot()));
        }
        let evs = self.sess.poll(*ms);
        if let Some(inputs) = self.sess.ready_inputs(t) {
            let s = self.sim.as_mut().expect("the machine's sim").sim.as_mut().expect("sim");
            // The announced join spawn: every machine creates the slot's
            // agent at the same tick (the shared-stream pick is part of the
            // deterministic tick — `Sim::mp_join_spawn`, the app's hook).
            if let Some((js, jt)) = self.join {
                if t == jt {
                    assert!(s.mp_join_spawn(js as usize), "the join spawns a body at tick {jt}");
                }
            }
            self.last_hash = nf_view::net_slot0_tick(s, &inputs);
            self.sess.record_run_tick(t, inputs);
            self.sess.submit_hash(t, self.last_hash);
            self.next_tick = t + 1;
        }
        evs
    }
}

fn settings_2p() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.seed = 0x0bed_0bad;
    s
}

/// Arm the match and run `ticks` honest ticks on host + client, recording
/// the transcript on the host.
fn run_two(h: &mut Peer, c1: &mut Peer, s: &MpSettings, ticks: u32, ms: &mut u64, path: &str) {
    h.sess.set_host_match(mp_to_match(s), s.seed);
    h.sess.start_match(0).unwrap();
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    while !c1.sess.is_started() {
        *ms += 5;
        let _ = h.sess.poll(*ms);
        let _ = c1.sess.poll(*ms);
        assert!(Instant::now() < deadline, "GO");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let arena = format!("{:08x}.bin", s.level_code);
    h.sim = Some(nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("host sim loads"));
    c1.sim = Some(nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("client sim loads"));
    // The live match start's setup (slot 0's machine shape — the replay starts from the same).
    for p in [&mut *h, &mut *c1] { nf_view::net_slot0_setup(p.sim.as_mut().unwrap().sim.as_mut().unwrap()); }
    let deadline = Instant::now() + std::time::Duration::from_secs(120);
    while h.next_tick < ticks || c1.next_tick < ticks {
        *ms += 5;
        h.step(ms, true);
        c1.step(ms, true);
        assert!(Instant::now() < deadline, "the pre-join ticks complete at {}", h.next_tick);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn late_join_three_sims_hashes_agree_after_the_join() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut h = Peer::host(27950);
    let mut c1 = Peer::client(27951, 27950);
    let mut ms: u64 = 0;
    // Handshake.
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    while !(h.sess.agents().len() == 2 && c1.sess.local_slot() != 0) {
        ms += 5;
        let _ = h.sess.poll(ms);
        let _ = c1.sess.poll(ms);
        assert!(Instant::now() < deadline, "handshake");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let s = settings_2p();
    run_two(&mut h, &mut c1, &s, 300, &mut ms, &path);
    assert_eq!(h.last_hash, c1.last_hash, "the two sims agree before the join");

    // A third machine dials the running match (the match runs on while the
    // host serves the log).
    let mut j = Peer::client(27952, 27950);
    let mut handover: Option<(MpSettings, u32, u64, Vec<[nf_net::Input; nf_net::MAX_AGENTS]>)> = None;
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    while handover.is_none() {
        ms += 5;
        h.step(&mut ms, true);
        c1.step(&mut ms, true);
        for e in j.sess.poll(ms) {
            if let Event::LateJoinHandover { slot, seed, settings, first_tick: _, snapshot_tick, snapshot_hash, log } = e {
                assert_eq!(slot, 2, "the joiner takes the first free slot");
                handover = Some((match_to_mp(&settings, seed), snapshot_tick, snapshot_hash, log));
            }
        }
        assert!(Instant::now() < deadline, "the handover completes");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let (j_settings, snapshot_tick, snapshot_hash, log) = handover.unwrap();
    assert_eq!(j_settings.seed, s.seed, "the handover carries the match seed");
    assert_eq!(log.len(), (snapshot_tick + 1) as usize, "one row per tick, inclusive");

    // The headless fast-forward: load + replay the log, verify the hash.
    let t0 = Instant::now();
    let (mut j_scene, j_hash) =
        nf_view::mp_late_join_catchup(&path, j_settings.clone(), &log, snapshot_tick)
            .expect("the joiner's catch-up loads and replays");
    let catchup_ms = t0.elapsed().as_millis() as f64;
    let ticks_s = (snapshot_tick as f64 + 1.0) / (catchup_ms / 1000.0);
    eprintln!(
        "[late-join] catch-up: {} ticks in {catchup_ms:.0} ms ({ticks_s:.0} ticks/s) — a 20-minute match (72000 ticks) back-fills in ~{} s",
        snapshot_tick + 1, (72000.0 / ticks_s) as u64
    );
    assert_eq!(j_hash, snapshot_hash, "the replay reproduces the host's snapshot hash");
    j.next_tick = snapshot_tick + 1;
    j.sim = Some(j_scene);
    j.sess.confirm_late_join();

    // The join tick is announced to all three; the joiner catches up to it
    // through the normal relay while the match runs on.
    let mut jt_announced: Option<u32> = None;
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    while jt_announced.is_none() {
        ms += 5;
        h.step(&mut ms, true);
        c1.step(&mut ms, true);
        j.step(&mut ms, false); // replaying/catching up: its slot is NEUTRAL yet
        let mut take = |e: &Event| if let Event::LateJoinSlot { slot: 2, join_tick } = e { Some(*join_tick) } else { None };
        for e in h.sess.poll(ms) { if let Some(jt) = take(&e) { jt_announced = jt_announced.or(Some(jt)); h.join = Some((2, jt)); } }
        for e in c1.sess.poll(ms) { if let Some(jt) = take(&e) { jt_announced = jt_announced.or(Some(jt)); c1.join = Some((2, jt)); } }
        for e in j.sess.poll(ms) { if let Some(jt) = take(&e) { jt_announced = jt_announced.or(Some(jt)); j.join = Some((2, jt)); } }
        assert!(Instant::now() < deadline, "the join tick is announced to all three");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let join_tick = jt_announced.unwrap();
    assert!(join_tick > snapshot_tick, "the join tick is past the snapshot");

    // All three run on: the joiner samples its own slot from the join tick,
    // the slot-2 agent spawned on every machine at that tick (the
    // shared-stream pick — the game's MP_Start shape), hashes agree.
    let end = join_tick + 90;
    let deadline = Instant::now() + std::time::Duration::from_secs(60);
    while h.next_tick < end || c1.next_tick < end || j.next_tick < end {
        ms += 5;
        h.step(&mut ms, true);
        c1.step(&mut ms, true);
        j.step(&mut ms, j.next_tick >= join_tick);
        assert!(Instant::now() < deadline, "the post-join run completes");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(h.last_hash, c1.last_hash, "host and client agree after the join");
    assert_eq!(h.last_hash, j.last_hash, "the joiner's sim agrees after the join");
    // The joined agent's body exists at the same place on every machine.
    let hp = h.sim.as_ref().unwrap().sim.as_ref().unwrap().mp.as_ref().unwrap().game.slots[2].pos;
    let cp = c1.sim.as_ref().unwrap().sim.as_ref().unwrap().mp.as_ref().unwrap().game.slots[2].pos;
    let jp = j.sim.as_ref().unwrap().sim.as_ref().unwrap().mp.as_ref().unwrap().game.slots[2].pos;
    assert!(hp.is_some() && cp.is_some() && jp.is_some(), "the joined slot has a body everywhere");
    assert_eq!(hp, cp, "the host and client spawned the joiner identically");
    assert_eq!(hp, jp, "the joiner's own spawn pick agrees");
}

/// A joiner whose replay disagrees with the host's snapshot is refused with
/// a clear reason: the joiner leaves `JoinVerifyFailed` and the host reports
/// exactly that — the match never sees the failed joiner.
#[test]
fn late_join_replay_mismatch_is_refused_with_a_clear_reason() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut h = Peer::host(27960);
    let mut c1 = Peer::client(27961, 27960);
    let mut ms: u64 = 0;
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    while !(h.sess.agents().len() == 2 && c1.sess.local_slot() != 0) {
        ms += 5;
        let _ = h.sess.poll(ms);
        let _ = c1.sess.poll(ms);
        assert!(Instant::now() < deadline, "handshake");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let s = settings_2p();
    run_two(&mut h, &mut c1, &s, 60, &mut ms, &path);
    // The joiner's handover arrives; one input byte of the log is corrupted
    // (a diverged replay's symptom — transfer corruption, a different build
    // or a poked sim all surface the same way: the hash check fails).
    let mut j = Peer::client(27962, 27960);
    let mut handover: Option<(MpSettings, u32, u64, Vec<[nf_net::Input; nf_net::MAX_AGENTS]>)> = None;
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    while handover.is_none() {
        ms += 5;
        h.step(&mut ms, true);
        c1.step(&mut ms, true);
        for e in j.sess.poll(ms) {
            if let Event::LateJoinHandover { slot, seed, settings, first_tick: _, snapshot_tick, snapshot_hash, mut log } = e {
                assert_eq!(slot, 2);
                let last = log.len() - 1;
                log[last][0].forward += 1.0; // one flipped input
                handover = Some((match_to_mp(&settings, seed), snapshot_tick, snapshot_hash, log));
            }
        }
        assert!(Instant::now() < deadline, "the handover completes");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let (j_settings, snapshot_tick, snapshot_hash, log) = handover.unwrap();
    let (_, j_hash) = nf_view::mp_late_join_catchup(&path, j_settings, &log, snapshot_tick)
        .expect("the replay runs");
    assert_ne!(j_hash, snapshot_hash, "the corrupted log fails verification");
    // The clear refusal: the joiner leaves with the explicit reason and the
    // host reports it — no slot flip, no spawn, the match runs on.
    j.sess.leave_with(LeaveReason::JoinVerifyFailed);
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    loop {
        ms += 5;
        let mut left = false;
        for e in h.step(&mut ms, true) {
            if matches!(e, Event::ClientLeft { slot: 2, reason: LeaveReason::JoinVerifyFailed }) {
                left = true;
            }
        }
        c1.step(&mut ms, true);
        if left {
            break;
        }
        assert!(Instant::now() < deadline, "the host reports the clear reason");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(h.sess.agents().len(), 2, "the failed joiner is gone");
}
