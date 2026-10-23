//! net-desync-resync: a state-hash mismatch no longer ends the match — the
//! diverged machine resyncs (it asks the host, the host re-serves the
//! late-join family on the live connection, the machine replays the host's
//! whole input log headless through the REAL app path
//! (`mp_late_join_catchup_joins`, the same call the `LateJoinHandover` arm
//! runs), verifies the replay's hash, confirms, and rejoins at the
//! re-announced tick) while the others keep playing on the session's
//! existing rules: the resyncing slot rides NEUTRAL (`host_slot_required`
//! false from the serve — the shape a dropped peer gets under
//! `DropPolicy::Neutral`) and any tick the host cannot run stalls the
//! never-guess `ready_inputs` way (no timeout — the 8 s `timeout_ms` only
//! ends a resync that never gets served). Harness: the flag-path three
//! machine loopback of `mp_online_failures.rs` (`SessionProxy` + real
//! 127.0.0.1 UDP + `nf_view::net_slot0_tick`, `net_rng` seeded per
//! `net_slot0_setup`); every machine runs the full slot-0 replica, whose
//! hash (`Sim::net_state_hash`) folds every slot's body — view-independent.
//! Numbers and ids only; port 28210 (distinct from the other live tests).

use std::time::{Duration, Instant};

use nf_view::mpnet::{CoreEvent, SessionProxy};

/// The three tests bind the same host port (28210): run them one at a time (the test harness
/// runs a binary's tests on parallel threads — the suite once failed two on "bind 28210").
static PORT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn sleep() { std::thread::sleep(Duration::from_millis(2)); }

fn scripted_input(t: u32, slot: usize) -> nf_net::Input {
    let mut i = nf_net::NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 89) as f32 / 89.0;
    i.strafe = if (t + slot as u32) % 13 == 0 { -0.4 } else { 0.0 };
    i.turn = ((t % 17) as f32 - 8.0) * 0.002;
    i.jump = (t + slot as u32 * 5) % 71 == 0;
    i
}

/// One machine of the loopback: its core, its scene, its next tick, its hash
/// log, the pending join-spawn (the app's `net_join_pending`).
struct Live {
    p: SessionProxy,
    scene: Option<nf_view::scene::Scene>,
    next: u32,
    hash: u64,
    hashes: Vec<(u32, u64)>,
    join: Option<(u8, u32)>,
    spawned: Option<bool>,
    /// The events the pump loop has not classified yet (Desync / fatal / handovers).
    saw: Saw,
}

#[derive(Default)]
struct Saw {
    desyncs: Vec<(u32, u64, u64)>,
    fatals: Vec<u32>,
    handovers: usize,
    slots: Vec<(u8, u32)>,
}

impl Live {
    fn new(p: SessionProxy) -> Live {
        Live { p, scene: None, next: 0, hash: 0, hashes: Vec::new(), join: None, spawned: None, saw: Saw::default() }
    }
    /// One frame: own input, pump, run the ready tick (the app's net tick as
    /// the slot-0 replica view), report the hash. Returns nothing — the
    /// events land in `saw`.
    fn frame(&mut self) {
        let t = self.next;
        let slot = self.p.0.borrow().local_slot();
        self.p.0.borrow_mut().local_input(t, scripted_input(t, slot));
        self.p.0.borrow_mut().poll();
        let ready = self.p.0.borrow_mut().ready_inputs(t);
        if let (Some(row), Some(scene)) = (ready, self.scene.as_mut()) {
            let s = scene.sim.as_mut().unwrap();
            if let Some((js, jt)) = self.join {
                if t == jt {
                    // net-desync-resync: the re-announce's spawn is a NO-OP for a
                    // resyncing slot (its body never left — `mp_join_spawn` returns
                    // false when the slot holds a body) and the real spawn for a
                    // fresh late joiner; the test pins both.
                    self.spawned = Some(s.mp_join_spawn(js as usize));
                    self.join = None;
                }
            }
            self.hash = nf_view::net_slot0_tick(s, &row);
            if slot == 0 {
                self.p.0.borrow_mut().record_run_tick(t, row);
            }
            self.p.0.borrow_mut().submit_hash(t, self.hash);
            self.hashes.push((t, self.hash));
            self.next = t + 1;
        }
        // Drain into an owned local first: the `for` head's temporaries (the
        // RefMut among them) live across the whole loop body, and the
        // handover arm's `confirm_late_join` borrows the session again
        // (RefCell panic — latent until the replay started succeeding).
        let evs: Vec<CoreEvent> = self.p.0.borrow_mut().take_app_events();
        for ev in evs {
            match ev {
                CoreEvent::Desync { tick, local, remote } => self.saw.desyncs.push((tick, local, remote)),
                CoreEvent::DesyncFatal { tick } => self.saw.fatals.push(tick),
                CoreEvent::LateJoinHandover { settings, snapshot_tick, snapshot_hash, log, joins, .. } => {
                    self.saw.handovers += 1;
                    // The app's real replay path (`mp_late_join_catchup_joins`,
                    // what the LateJoinHandover arm runs): reload the arena, run
                    // the host's log headless, compare the hash, confirm. The
                    // frozen machine's old scene drops first — the replay replaces
                    // it wholesale on success (the app keeps the old scene visible;
                    // this harness does not render, and a 4 GB test machine cannot
                    // hold four MP scenes at once).
                    self.scene = None;
                    let iso_v = iso().expect("NF_ISO");
                    match nf_view::mp_late_join_catchup_joins(&iso_v, *settings.clone(), &log, snapshot_tick, &joins) {
                        Ok((scene, hash)) => {
                            assert_eq!(hash, snapshot_hash, "the replay must reproduce the host's hash at tick {snapshot_tick}");
                            self.scene = Some(scene);
                            self.next = snapshot_tick + 1;
                            self.p.0.borrow_mut().confirm_late_join();
                        }
                        Err(e) => panic!("the resync replay failed: {e}"),
                    }
                }
                CoreEvent::LateJoinSlot { slot, join_tick } => {
                    self.saw.slots.push((slot, join_tick));
                    self.join = Some((slot, join_tick));
                }
                _ => {}
            }
        }
    }
}

fn hash_at(m: &Live, t: u32) -> Option<u64> {
    m.hashes.iter().rev().find(|(x, _)| *x == t).map(|(_, h)| *h)
}

/// Every tick from `from` that both machines ran: the same hash.
fn same_hashes(a: &Live, b: &Live, from: u32, what: &str) {
    let mut n = 0;
    for (t, h) in a.hashes.iter().filter(|(t, _)| *t >= from) {
        if let Some(o) = hash_at(b, *t) {
            assert_eq!(*h, o, "{what}: tick {t}");
            n += 1;
        }
    }
    assert!(n > 20, "{what}: only {n} common ticks compared");
}

/// Host + two clients in a running 3-human deathmatch, all machines loaded and
/// ticking. Returns them in (host, client-1, client-2) order.
fn three_machines(path: &str) -> (Live, Live, Live, nf_game::mp::MpSettings) {
    const PORT: u16 = 28210;
    let hp = SessionProxy::host(0).unwrap();
    hp.0.borrow_mut().rebind_host(PORT).unwrap();
    hp.0.borrow_mut().set_codename("Host");
    let cp1 = SessionProxy::host(0).unwrap();
    cp1.0.borrow_mut().dial(&format!("127.0.0.1:{PORT}")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while cp1.0.borrow().local_slot() == 0 {
        hp.0.borrow_mut().poll();
        cp1.0.borrow_mut().poll();
        sleep();
        assert!(Instant::now() < deadline, "client 1 handshake");
    }
    let cp2 = SessionProxy::host(0).unwrap();
    cp2.0.borrow_mut().dial(&format!("127.0.0.1:{PORT}")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while cp2.0.borrow().local_slot() == 0 {
        hp.0.borrow_mut().poll();
        cp1.0.borrow_mut().poll();
        cp2.0.borrow_mut().poll();
        sleep();
        assert!(Instant::now() < deadline, "client 2 handshake");
    }
    let mut s = nf_game::mp::MpSettings::deathmatch(0x0700_0024);
    s.human_count = 3;
    s.player_count = 3;
    s.total_players = 3;
    s.seed = 0x0bad_5eed;
    hp.0.borrow_mut().start_match(&s);
    let deadline = Instant::now() + Duration::from_secs(10);
    while cp1.0.borrow().match_started().is_none() || cp2.0.borrow().match_started().is_none() {
        hp.0.borrow_mut().poll();
        cp1.0.borrow_mut().poll();
        cp2.0.borrow_mut().poll();
        sleep();
        assert!(Instant::now() < deadline, "the clients get the match");
    }
    let arena = format!("{:08x}.bin", s.level_code);
    let _ = nf_view::level_archive_name(0x0700_0024);
    let mut h = Live::new(hp);
    let mut c1 = Live::new(cp1);
    let mut c2 = Live::new(cp2);
    for m in [&mut h, &mut c1, &mut c2] {
        let mut sc = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("the arena loads");
        nf_view::net_slot0_setup(sc.sim.as_mut().unwrap());
        m.scene = Some(sc);
        m.p.0.borrow_mut().take_app_events();
    }
    (h, c1, c2, s)
}

/// Pump every machine once; a machine whose `ready_inputs` is None (a stall or
/// a resync freeze) just polls. Returns true if any machine advanced.
fn pump_all(machines: &mut [&mut Live]) -> bool {
    let mut before = 0;
    for m in machines.iter() { before += m.next; }
    for m in machines.iter_mut() { m.frame(); }
    let mut after = 0;
    for m in machines.iter() { after += m.next; }
    after > before
}

fn pump_until(machines: &mut [&mut Live], secs: u64, what: &str, mut done: impl FnMut(&mut [&mut Live]) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while !done(machines) {
        pump_all(machines);
        sleep();
        assert!(Instant::now() < deadline, "{what}");
    }
}

/// The main test: a mid-match perturbation of ONE machine's state diverges its
/// hash; that machine resyncs (the session asks, the host re-serves the log,
/// the machine replays through the app's real catch-up, verifies, rejoins),
/// all three agree on every tick after, and the match runs on to the tick
/// target with no fatal event. Sisters in the other two tests.
#[test]
fn a_perturbed_machine_resyncs_and_the_match_runs_on() {
    let _port = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut h, mut c1, mut c2, _s) = three_machines(&path);
    // In sync to tick 200.
    pump_until(&mut [&mut h, &mut c1, &mut c2], 90, "in sync to 60", |ms| ms.iter().all(|m| m.next >= 60));
    same_hashes(&h, &c1, 0, "pre-perturbation host vs c1");
    same_hashes(&h, &c2, 0, "pre-perturbation host vs c2");
    assert!(h.saw.desyncs.is_empty() && c1.saw.desyncs.is_empty() && c2.saw.desyncs.is_empty());
    // Perturb ONE machine's state mid-match: a body's x moves on c2 only — its
    // next hash exchange (every 15 ticks) must disagree.
    {
        let sim = c2.scene.as_mut().unwrap().sim.as_mut().unwrap();
        sim.bond.pos[0] += 0.01;
    }
    // The desync fires on c2 (the diverged machine) and the host (it holds both
    // hashes); c1 — which only ever compares against the host — sees nothing.
    pump_until(&mut [&mut h, &mut c1, &mut c2], 60, "the desync fires", |ms| {
        !ms[0].saw.desyncs.is_empty() && !ms[2].saw.desyncs.is_empty()
    });
    assert!(!c2.saw.desyncs.is_empty(), "the diverged machine sees the mismatch");
    assert!(!h.saw.desyncs.is_empty(), "the host sees the mismatch");
    assert!(c1.saw.desyncs.is_empty(), "the third machine (in sync with the host) sees nothing");
    let (dtick, dlocal, dremote) = c2.saw.desyncs[0];
    assert_ne!(dlocal, dremote, "the two hashes differ (the log line carries both)");
    eprintln!("[resync] desync at tick {dtick}: local {dlocal:016x} remote {dremote:016x}");
    // The session policy fired at once: c2 asked, the host served, the handover
    // ran the replay (inside `frame`'s event handling), the re-announce landed.
    pump_until(&mut [&mut h, &mut c1, &mut c2], 90, "the resync completes", |ms| {
        ms[2].saw.handovers >= 1 && !ms[2].saw.slots.is_empty() && ms.iter().all(|m| m.next >= ms[0].saw.slots.last().map_or(u32::MAX, |(_, jt)| *jt).min(u32::MAX))
    });
    assert_eq!(c2.saw.handovers, 1, "exactly one handover (one resync)");
    assert!(h.saw.fatals.is_empty() && c1.saw.fatals.is_empty() && c2.saw.fatals.is_empty(), "no fatal event");
    // The re-announce: every machine armed the slot (the spawn is a NO-OP for
    // the resyncer on all three — its body never left).
    let (jslot, jjtick) = c2.saw.slots[0];
    assert_eq!(jslot as usize, c2.p.0.borrow().local_slot(), "the resyncer's own slot re-arms");
    pump_until(&mut [&mut h, &mut c1, &mut c2], 60, "the spawn tick passes", |ms| {
        ms.iter().all(|m| m.spawned.is_some() || m.next > jjtick)
    });
    assert_eq!(h.spawned, Some(false), "the host's spawn for the resyncing slot is a no-op (the body stands)");
    assert_eq!(c1.spawned, Some(false), "c1's spawn is a no-op");
    assert_eq!(c2.spawned, Some(false), "the resyncer's spawn is a no-op (its replay built the body)");
    // The match runs on: all three to tick 900, agreeing on every hash exchange.
    pump_until(&mut [&mut h, &mut c1, &mut c2], 120, "the match runs to 200 past the rejoin", |ms| {
        ms.iter().all(|m| m.next >= jjtick + 200)
    });
    same_hashes(&h, &c1, jjtick, "host vs c1 after the resync");
    same_hashes(&h, &c2, jjtick, "host vs c2 after the resync (it converged to the host)");
    same_hashes(&c1, &c2, jjtick, "c1 vs c2 after the resync");
    assert!(h.saw.fatals.is_empty() && c2.saw.fatals.is_empty(), "the match never ended");
}

/// Sister: with no perturbation nothing resyncs — no desync, no handover, no
/// re-announce, no fatal, for the whole run.
#[test]
fn without_a_perturbation_nothing_resyncs() {
    let _port = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut h, mut c1, mut c2, _s) = three_machines(&path);
    pump_until(&mut [&mut h, &mut c1, &mut c2], 120, "the clean run to 330", |ms| {
        ms.iter().all(|m| m.next >= 330)
    });
    for (name, m) in [("host", &h), ("c1", &c1), ("c2", &c2)] {
        assert!(m.saw.desyncs.is_empty(), "{name}: no desync on a clean run");
        assert!(m.saw.fatals.is_empty(), "{name}: no fatal on a clean run");
        assert_eq!(m.saw.handovers, 0, "{name}: no handover on a clean run");
        assert!(m.saw.slots.is_empty(), "{name}: no re-announce on a clean run");
    }
    same_hashes(&h, &c1, 0, "clean host vs c1");
    same_hashes(&h, &c2, 0, "clean host vs c2");
}

/// Sister 2: a SECOND perturbation within 60 s of the resync is fatal — the
/// machine falls back to ending the match (the pre-resync behaviour, the
/// lobby message's event).
#[test]
fn a_second_desync_within_the_window_is_fatal() {
    let _port = PORT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut h, mut c1, mut c2, _s) = three_machines(&path);
    pump_until(&mut [&mut h, &mut c1, &mut c2], 90, "in sync to 60", |ms| ms.iter().all(|m| m.next >= 60));
    // First desync → the resync runs to completion (as the main test).
    {
        let sim = c2.scene.as_mut().unwrap().sim.as_mut().unwrap();
        sim.bond.pos[0] += 0.01;
    }
    pump_until(&mut [&mut h, &mut c1, &mut c2], 90, "the first resync completes", |ms| {
        ms[2].saw.handovers >= 1 && !ms[2].saw.slots.is_empty()
    });
    assert!(c2.saw.fatals.is_empty(), "the first resync is not fatal");
    // The second perturbation, seconds (well inside 60 s) after the resync.
    pump_until(&mut [&mut h, &mut c1, &mut c2], 60, "the resyncer rejoins", |ms| {
        ms[2].next > ms[2].saw.slots.last().map(|(_, jt)| *jt).unwrap_or(0)
    });
    {
        let sim = c2.scene.as_mut().unwrap().sim.as_mut().unwrap();
        sim.bond.pos[0] += 0.01;
    }
    pump_until(&mut [&mut h, &mut c1, &mut c2], 60, "the second desync is fatal", |ms| {
        !ms[2].saw.fatals.is_empty()
    });
    assert_eq!(c2.saw.fatals.len(), 1, "exactly one fatal (the 60 s rule)");
    assert!(c2.saw.desyncs.len() >= 2, "both mismatches were detected");
    eprintln!("[resync] the second desync within 60 s is fatal at tick {:?}", c2.saw.fatals[0]);
}
