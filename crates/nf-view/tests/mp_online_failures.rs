//! mp-online-failures: the online flow's failure paths, headless over real 127.0.0.1 UDP —
//! FrontEnds driven through their actions + the app's session glue (`SessionProxy` /
//! `NetCore`), the way `net_e2e_loopback_live.rs` and `mp_net_lobby.rs` drive them:
//!
//! 1. a client leaves mid-match: the others run on in sync and the leaver reconnects into its
//!    own slot (the body stands — the reconnect path of `Sim::mp_join_spawn`) [ISO];
//! 2. the host leaves mid-match: every client hears it at once (`CoreEvent::HostLeft`) and lands
//!    on its Join Online page with the address kept and the reason on screen;
//! 3. joining a match in progress FROM THE MENUS: the dial turns into the late-join handover
//!    (`MpNetState::Joining`, `LobbyAdopt::LateJoin`) and the joiner plays in sync [ISO];
//! 4. the rematch with three machines: all back to the lobby, a second match starts — a client
//!    pressing Continue before the host waits for the next lobby instead of late-joining the
//!    finished match;
//! 5. Back on the lobby releases the socket: re-hosting the same port works at once.
//!
//! Numbers and ids only. Ports 28110..28199 (distinct from the other live tests).

use std::time::{Duration, Instant};

use nf_view::menu::{Action, FrontEnd, MpNetState, Page};
use nf_view::mpnet::{CoreEvent, LobbyAdopt, SessionProxy};

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn sleep() {
    std::thread::sleep(Duration::from_millis(2));
}

/// A front end wired the way the app wires it on the online pages (`ensure_lobby_session`:
/// a placeholder proxy the listen/dial rows re-bind).
fn online_fe() -> (FrontEnd, SessionProxy) {
    let p = SessionProxy::host(0).expect("the placeholder proxy");
    let mut fe = FrontEnd::new();
    fe.mp_session = Some(Box::new(p.clone()));
    (fe, p)
}

/// Host: the port page → Start Listening (the real `mp_host_begin`).
fn host_listen(fe: &mut FrontEnd, port: u16) {
    fe.page = Page::MpHost;
    fe.cursor = 1;
    fe.mp_net_port = port as u32;
    assert!(fe.action(Action::Confirm), "Start Listening");
    assert!(fe.mp_hosting, "hosting on {port}");
    assert_eq!(fe.page, Page::MpOnlineLobby);
}

/// Client: the address page → Connect (the real `mp_net_dial`).
fn client_dial(fe: &mut FrontEnd, port: u16) {
    fe.page = Page::MpNetJoin;
    fe.cursor = 0;
    fe.mp_net_addr.clear();
    for c in format!("127.0.0.1:{port}").chars() { assert!(fe.mp_net_type(c)); }
    assert!(fe.action(Action::Confirm), "Connect");
    assert_eq!(fe.mp_net_state, MpNetState::Dialing);
}

fn tick_all(fes: &mut [&mut FrontEnd]) {
    for f in fes.iter_mut() { f.tick(); }
    sleep();
}

/// Tick every front end until `done`, bounded.
fn until(fes: &mut [&mut FrontEnd], secs: u64, what: &str, mut done: impl FnMut(&[&mut FrontEnd]) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while !done(fes) {
        tick_all(fes);
        assert!(Instant::now() < deadline, "{what}");
    }
}

// ── 5. Back releases the socket ──────────────────────────────────────────────────────────────

/// Host Back on the lobby frees the port at once (a raw bind succeeds) and Host Online on the
/// SAME port listens again immediately; a client's Back frees its socket too. Sister: an open
/// session holds its port (the raw bind fails) — the check can fail.
#[test]
fn back_releases_the_socket_and_the_same_port_rehosts_at_once() {
    const PORT: u16 = 28110;
    let (mut h, hp) = online_fe();
    host_listen(&mut h, PORT);
    // Sister: while listening, the port is taken.
    assert!(std::net::UdpSocket::bind(("0.0.0.0", PORT)).is_err(), "an open lobby holds its port");
    let (mut c, cp) = online_fe();
    client_dial(&mut c, PORT);
    until(&mut [&mut h, &mut c], 10, "the client reaches the lobby", |f| f[1].page == Page::MpOnlineLobby);
    let client_port = cp.0.borrow().local_addr().port();
    assert!(std::net::UdpSocket::bind(("0.0.0.0", client_port)).is_err(), "the client's dial socket is bound");

    // The client backs out: its socket is released at once.
    assert!(c.action(Action::Back));
    assert_eq!(c.page, Page::MpMode);
    assert!(cp.core_is_closed());
    std::net::UdpSocket::bind(("0.0.0.0", client_port)).expect("the client's port is free after Back");

    // The host backs out: the port is free at once …
    assert!(h.action(Action::Back));
    assert_eq!(h.page, Page::MpMode);
    std::net::UdpSocket::bind(("0.0.0.0", PORT)).expect("the host's port is free after Back");
    // … and Host Online → Start Listening on the SAME port works on the first try (the same
    // proxy re-binds — the app keeps it in `net_lobby`).
    assert!(h.action(Action::Down), "the Host Online row");
    assert!(h.action(Action::Confirm), "Host Online");
    assert_eq!(h.page, Page::MpHost);
    assert!(h.action(Action::Down), "the Start Listening row");
    assert!(h.action(Action::Confirm), "Start Listening again");
    assert!(h.mp_hosting && h.mp_net_status() != Some("Port already in use"), "re-hosted on {PORT} at once");
    assert_eq!(hp.0.borrow().local_addr().port(), PORT);
}

// ── 2. the host leaves ───────────────────────────────────────────────────────────────────────

/// The host backs out of the lobby: the client is told at once and lands on its address page
/// with a message — not after the 5 s timeout.
#[test]
fn the_host_leaving_the_lobby_returns_the_client_at_once() {
    const PORT: u16 = 28120;
    let (mut h, _hp) = online_fe();
    host_listen(&mut h, PORT);
    let (mut c, _cp) = online_fe();
    client_dial(&mut c, PORT);
    until(&mut [&mut h, &mut c], 10, "the client reaches the lobby", |f| f[1].page == Page::MpOnlineLobby);
    assert!(h.action(Action::Back));
    let t0 = Instant::now();
    until(&mut [&mut h, &mut c], 10, "the client leaves the dead lobby", |f| f[1].page != Page::MpOnlineLobby);
    assert!(t0.elapsed() < Duration::from_secs(2), "at once, not the timeout: {:?}", t0.elapsed());
    assert_eq!(c.page, Page::MpNetJoin, "back on the address page");
    assert_eq!(c.mp_net_addr, format!("127.0.0.1:{PORT}"), "the address is kept");
    assert_eq!(c.net_notice.as_deref(), Some("The host left the lobby"));
}

/// Start a match with `n` clients through the lobby; returns the front ends and proxies.
fn start_lobby_match(port: u16, n: usize) -> (FrontEnd, SessionProxy, Vec<(FrontEnd, SessionProxy)>) {
    let (mut h, hp) = online_fe();
    host_listen(&mut h, port);
    let mut cs: Vec<(FrontEnd, SessionProxy)> = Vec::new();
    for i in 0..n {
        let (mut c, cp) = online_fe();
        client_dial(&mut c, port);
        let deadline = Instant::now() + Duration::from_secs(10);
        while c.page != Page::MpOnlineLobby {
            c.tick();
            h.tick();
            for (o, _) in cs.iter_mut() { o.tick(); }
            sleep();
            assert!(Instant::now() < deadline, "client {i} reaches the lobby");
        }
        cs.push((c, cp));
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while h.mp_join.iter().filter(|a| a.joined).count() < n + 1 {
        h.tick();
        for (o, _) in cs.iter_mut() { o.tick(); }
        sleep();
        assert!(Instant::now() < deadline, "the host's lobby sees everyone");
    }
    host_start(&mut h, &mut cs);
    (h, hp, cs)
}

/// The host's Start on the one screen; every client receives the handover.
fn host_start(h: &mut FrontEnd, cs: &mut [(FrontEnd, SessionProxy)]) {
    // The publish settles before Start (the clients mirror the snapshot).
    for _ in 0..30 { h.tick(); for (o, _) in cs.iter_mut() { o.tick(); } sleep(); }
    let start = h.mp_lobby_rows().unwrap().iter().position(|r| r.id == nf_view::mp_lobby::MpLobbyRowId::Start).unwrap();
    h.mp_one.focus = start;
    assert!(h.mp_lobby_start_enabled(), "every joined human is ready");
    assert!(h.action(Action::Confirm), "Start");
    assert!(h.requested_level.is_some(), "the host requests the arena");
    let deadline = Instant::now() + Duration::from_secs(10);
    while cs.iter().any(|(c, _)| c.requested_level.is_none()) {
        h.tick();
        for (o, _) in cs.iter_mut() { o.tick(); }
        sleep();
        assert!(Instant::now() < deadline, "every client receives the match start");
    }
}

/// The host quits a running match (the app's `net_leave_match` closes its session): both
/// clients' sessions report `HostLeft` at once, and the app's landing (`mp_net_return`, what
/// `net_fail` runs) puts each on its Join Online page, address kept, the reason on screen.
#[test]
fn the_host_leaving_mid_match_reaches_every_client_at_once() {
    const PORT: u16 = 28130;
    let (_h, hp, cs) = start_lobby_match(PORT, 2);
    // The match runs (the app adopted the sessions): a few honest polls, then the host quits.
    for _ in 0..20 { hp.0.borrow_mut().poll(); for (_, cp) in &cs { cp.0.borrow_mut().poll(); } sleep(); }
    for (_, cp) in &cs { cp.0.borrow_mut().take_app_events(); }
    hp.0.borrow_mut().close();
    let t0 = Instant::now();
    let mut heard = [false; 2];
    while !heard.iter().all(|h| *h) {
        for (i, (_, cp)) in cs.iter().enumerate() {
            cp.0.borrow_mut().poll();
            for e in cp.0.borrow_mut().take_app_events() {
                match e {
                    CoreEvent::HostLeft => heard[i] = true,
                    CoreEvent::HostTimeout => panic!("client {i}: a timeout, not the host's LEAVE"),
                    _ => {}
                }
            }
        }
        sleep();
        assert!(t0.elapsed() < Duration::from_secs(4), "both clients hear the host leave (heard {heard:?})");
    }
    assert!(t0.elapsed() < Duration::from_secs(1), "at once: {:?}", t0.elapsed());
    for (_, cp) in &cs {
        assert_eq!(nf_view::menu::MpSession::link(cp), nf_view::menu::MpNetLink::Lost, "the link reads Lost");
    }
    // The app's landing (net_fail → FrontEnd::mp_net_return) on a fresh front end.
    let mut fe = FrontEnd::new();
    fe.mp_net_return(Some((false, format!("127.0.0.1:{PORT}"))), "The host left the match");
    assert_eq!(fe.page, Page::MpNetJoin, "the client lands on Join Online");
    assert_eq!(fe.mp_net_addr, format!("127.0.0.1:{PORT}"));
    assert_eq!(fe.net_notice.as_deref(), Some("The host left the match"));
    assert_eq!(fe.mp_net_state, MpNetState::Idle);
    let mut feh = FrontEnd::new();
    feh.mp_net_return(Some((true, PORT.to_string())), "State desync");
    assert_eq!((feh.page, feh.cursor, feh.mp_net_port), (Page::MpHost, 1, PORT as u32), "a host lands on its port page, Start focused");
    let mut fen = FrontEnd::new();
    fen.mp_net_return(None, "x");
    assert_eq!(fen.page, Page::MpMode, "no origin: the mode page");
}

// ── 4. the rematch with three machines ───────────────────────────────────────────────────────

/// The app's debriefing hand-off (`mp_results`): a fresh front end on the debriefing with the
/// rejoin target; the finished session stays open (polled) and takes no new peer.
fn to_debriefing(rejoin: (bool, String)) -> FrontEnd {
    let mut fe = FrontEnd::new();
    fe.page = Page::MpDebriefing;
    fe.cursor = 0;
    fe.mp_online_rejoin = Some(rejoin);
    fe
}

/// Continue on the debriefing, then the app glue `menu_after` runs: the finished session closes
/// (it left the debriefing) and a fresh proxy is wired for the online page.
fn continue_online(fe: &mut FrontEnd, finished: &SessionProxy) -> SessionProxy {
    assert!(fe.action(Action::Confirm), "Continue");
    assert!(matches!(fe.page, Page::MpHost | Page::MpNetJoin), "Continue returns to the online page");
    finished.0.borrow_mut().close();
    let p = SessionProxy::host(0).unwrap();
    fe.mp_session = Some(Box::new(p.clone()));
    p
}

#[test]
fn three_machines_rematch_through_the_lobby() {
    const PORT: u16 = 28140;
    let (_h, hp, cs) = start_lobby_match(PORT, 2);
    // The match "ends": every machine's app opens the debriefing. The host's finished session
    // takes no new peer (`finish_match`, mp_results).
    hp.0.borrow_mut().finish_match();
    let mut h = to_debriefing((true, PORT.to_string()));
    let mut c1 = to_debriefing((false, format!("127.0.0.1:{PORT}")));
    let mut c2 = to_debriefing((false, format!("127.0.0.1:{PORT}")));
    let (c1_old, c2_old) = (cs[0].1.clone(), cs[1].1.clone());
    drop(cs);
    // The CLIENTS press Continue first: they re-dial the port, where the host's finished match
    // still listens on its debriefing.
    let _c1p = continue_online(&mut c1, &c1_old);
    let _c2p = continue_online(&mut c2, &c2_old);
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_millis(1500) {
        c1.tick();
        c2.tick();
        hp.0.borrow_mut().poll(); // the host's finished session, polled under its debriefing
        sleep();
        for (i, c) in [&c1, &c2].iter().enumerate() {
            assert_ne!(c.mp_net_state, MpNetState::Joining, "client {i} must not late-join the finished match");
            assert_ne!(c.page, Page::MpOnlineLobby, "client {i} has no lobby before the host re-listens");
        }
    }
    // The host presses Continue: its finished session closes, the re-listen on the same port.
    let _hp2 = continue_online(&mut h, &hp);
    until(&mut [&mut h, &mut c1, &mut c2], 30, "all three back in the lobby",
        |f| f.iter().all(|x| x.page == Page::MpOnlineLobby) && f[0].mp_join.iter().filter(|a| a.joined).count() == 3);
    assert!(h.mp_hosting, "the host re-listened on {PORT}");
    // The second match starts and reaches both clients.
    let mut cs2 = vec![(c1, _c1p), (c2, _c2p)];
    host_start(&mut h, &mut cs2);
    assert_eq!(cs2[0].0.requested_level, h.requested_level, "the clients load the host's arena");
    assert!(cs2.iter().all(|(c, _)| c.mp_match_settings.as_ref().is_some_and(|s| s.human_count == 3)), "three humans in match two");
}

// ── 1 + 3: the live sims (NF_ISO) ────────────────────────────────────────────────────────────

fn scripted_input(t: u32, slot: usize) -> nf_net::Input {
    let mut i = nf_net::NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 97) as f32 / 97.0;
    i.strafe = if (t + slot as u32) % 11 == 0 { -0.5 } else { 0.0 };
    i.turn = ((t % 13) as f32 - 6.0) * 0.002;
    i.jump = (t + slot as u32 * 7) % 61 == 0;
    i
}

/// One machine of the live sims: its core, its scene, its own next tick, the last hash and the
/// announced join (the app's `net_join_pending`).
struct Live {
    p: SessionProxy,
    scene: Option<nf_view::scene::Scene>,
    next: u32,
    hash: u64,
    hashes: Vec<(u32, u64)>,
    join: Option<(u8, u32)>,
    /// The own slot's input starts here (a late joiner: its join tick).
    sample_from: u32,
    spawned: Option<bool>,
}

impl Live {
    fn new(p: SessionProxy) -> Live {
        Live { p, scene: None, next: 0, hash: 0, hashes: Vec::new(), join: None, sample_from: 0, spawned: None }
    }
    /// One frame: own input, pump, run the ready tick (the app's net tick as slot 0's view).
    fn frame(&mut self, host: bool) -> Vec<CoreEvent> {
        let t = self.next;
        let slot = self.p.0.borrow().local_slot();
        if t >= self.sample_from { self.p.0.borrow_mut().local_input(t, scripted_input(t, slot)); }
        self.p.0.borrow_mut().poll();
        let ready = self.p.0.borrow_mut().ready_inputs(t);
        if let (Some(row), Some(scene)) = (ready, self.scene.as_mut()) {
            let s = scene.sim.as_mut().unwrap();
            if let Some((js, jt)) = self.join {
                if t == jt { self.spawned = Some(s.mp_join_spawn(js as usize)); self.join = None; }
            }
            self.hash = nf_view::net_slot0_tick(s, &row);
            if host { self.p.0.borrow_mut().record_run_tick(t, row); }
            self.p.0.borrow_mut().submit_hash(t, self.hash);
            self.hashes.push((t, self.hash));
            self.next = t + 1;
        }
        self.p.0.borrow_mut().take_app_events()
    }
}

fn hash_at(m: &Live, t: u32) -> Option<u64> {
    m.hashes.iter().rev().find(|(x, _)| *x == t).map(|(_, h)| *h)
}

/// Every tick from `from` that both ran: the same hash.
fn same_hashes(a: &Live, b: &Live, from: u32, what: &str) {
    let mut n = 0;
    for (t, h) in a.hashes.iter().filter(|(t, _)| *t >= from) {
        if let Some(o) = hash_at(b, *t) { assert_eq!(*h, o, "{what}: tick {t}"); n += 1; }
    }
    assert!(n > 50, "{what}: {n} common ticks compared");
}

/// A joiner dialed through the menus: Joining → adopt (LateJoin) → the handover's replay →
/// confirm. Returns the joiner's Live machine (scene = the replay, next = snapshot + 1).
fn menu_late_join(path: &str, port: u16, h: &mut Live, c1: &mut Live, want_slot: u8, want_joins: usize) -> (FrontEnd, Live) {
    let (mut fe, p) = online_fe();
    client_dial(&mut fe, port);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut handover = None;
    while handover.is_none() {
        fe.tick(); // the join page pumps the session (mp_net_poll)
        h.frame(true);
        c1.frame(false);
        // The app adopts the lobby session the frame the dial turns into a late join, and its
        // event drain owns the handover from then on.
        if p.adopt_kind() == LobbyAdopt::LateJoin {
            assert_eq!(fe.mp_net_state, MpNetState::Joining, "the join page says joining");
            assert_eq!(fe.page, Page::MpNetJoin);
            assert_eq!(fe.mp_net_status(), Some("Joining the match in progress..."));
            for e in p.0.borrow_mut().take_app_events() {
                if let CoreEvent::LateJoinHandover { slot, settings, snapshot_tick, snapshot_hash, log, joins } = e {
                    handover = Some((slot, settings, snapshot_tick, snapshot_hash, log, joins));
                }
            }
        }
        assert!(p.adopt_kind() != LobbyAdopt::Match, "a join in progress is never a fresh match load");
        sleep();
        assert!(Instant::now() < deadline, "the handover arrives (joiner state {:?})", fe.mp_net_state);
    }
    let (slot, settings, snapshot_tick, snapshot_hash, log, joins) = handover.unwrap();
    assert_eq!(slot, want_slot, "the joiner's slot");
    // The replay runs while the others play on (the app blocks its frame; here the host and
    // client keep pumping between replay chunks would be the threaded version — the 5 s
    // timeout is ample for a short match).
    assert_eq!(joins.len(), want_joins, "the earlier late joins ride the handover");
    let (scene, hash) = nf_view::mp_late_join_catchup_joins(path, *settings, &log, snapshot_tick, &joins).expect("the replay");
    assert_eq!(hash, snapshot_hash, "the replay verifies");
    p.0.borrow_mut().confirm_late_join();
    let mut j = Live::new(p);
    j.scene = Some(scene);
    j.next = snapshot_tick + 1;
    j.sample_from = u32::MAX;
    (fe, j)
}

/// Pump the three until the join tick is announced everywhere; arm it.
fn arm_join(h: &mut Live, c1: &mut Live, j: &mut Live, slot: u8) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut got = [None; 3];
    while got.iter().any(|g| g.is_none()) {
        for (i, (m, host)) in [(&mut *h, true), (&mut *c1, false), (&mut *j, false)].into_iter().enumerate() {
            for e in m.frame(host) {
                if let CoreEvent::LateJoinSlot { slot: s, join_tick } = e {
                    if s == slot { got[i] = Some(join_tick); m.join = Some((s, join_tick)); }
                }
            }
        }
        sleep();
        assert!(Instant::now() < deadline, "the join tick reaches all three ({got:?})");
    }
    assert!(got[0] == got[1] && got[1] == got[2], "one join tick everywhere: {got:?}");
    let jt = got[0].unwrap();
    j.sample_from = jt;
    jt
}

fn run_three(h: &mut Live, c1: &mut Live, j: &mut Live, until_tick: u32) {
    let deadline = Instant::now() + Duration::from_secs(120);
    while h.next < until_tick || c1.next < until_tick || j.next < until_tick {
        for e in h.frame(true).into_iter().chain(c1.frame(false)).chain(j.frame(false)) {
            assert!(!matches!(e, CoreEvent::Desync { .. }), "no desync: {e:?}");
        }
        sleep();
        assert!(Instant::now() < deadline, "the three run to {until_tick} (h {} c {} j {})", h.next, c1.next, j.next);
    }
}

/// Scenarios 3 and 1 on the live sims: a third machine joins the RUNNING match from the Join
/// Online page (the late-join path, its own slot spawned at the announced tick), plays in sync;
/// then it leaves (the app's quit closes its session) — the host and the other client run on in
/// sync on its NEUTRAL body — and it reconnects through the menus into the SAME slot (the body
/// stands: `mp_join_spawn` creates nothing) and plays on in sync.
#[test]
fn join_in_progress_from_the_menus_then_leave_and_reconnect() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    const PORT: u16 = 28150;
    // Host + client through the flag-path cores (the lobby flow is pinned elsewhere).
    let hp = SessionProxy::host(0).unwrap();
    hp.0.borrow_mut().rebind_host(PORT).unwrap();
    hp.0.borrow_mut().set_codename("Host");
    let cp = SessionProxy::host(0).unwrap();
    cp.0.borrow_mut().dial(&format!("127.0.0.1:{PORT}")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while cp.0.borrow().local_slot() == 0 {
        hp.0.borrow_mut().poll();
        cp.0.borrow_mut().poll();
        sleep();
        assert!(Instant::now() < deadline, "handshake");
    }
    let mut s = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.seed = 0x0bad_5eed;
    hp.0.borrow_mut().start_match(&s);
    let deadline = Instant::now() + Duration::from_secs(10);
    while cp.0.borrow().match_started().is_none() {
        hp.0.borrow_mut().poll();
        cp.0.borrow_mut().poll();
        sleep();
        assert!(Instant::now() < deadline, "the client gets the match");
    }
    let arena = format!("{:08x}.bin", s.level_code);
    let mut h = Live::new(hp);
    let mut c1 = Live::new(cp);
    for m in [&mut h, &mut c1] {
        let mut sc = nf_view::load_mp_carry(&path, &arena, false, None, s.clone()).expect("the arena loads");
        nf_view::net_slot0_setup(sc.sim.as_mut().unwrap());
        m.scene = Some(sc);
        m.p.0.borrow_mut().take_app_events();
    }
    let deadline = Instant::now() + Duration::from_secs(60);
    while h.next < 200 || c1.next < 200 {
        h.frame(true);
        c1.frame(false);
        sleep();
        assert!(Instant::now() < deadline, "the opening 200 ticks");
    }

    // ── 3: join from the menus ──
    let (_fe, mut j) = menu_late_join(&path, PORT, &mut h, &mut c1, 2, 0);
    let jt = arm_join(&mut h, &mut c1, &mut j, 2);
    run_three(&mut h, &mut c1, &mut j, jt + 120);
    assert_eq!(h.spawned, Some(true), "the host spawned the joiner's body at the join tick");
    assert_eq!(j.spawned, Some(true), "the joiner spawned its own body there too");
    same_hashes(&h, &c1, 0, "host vs client");
    same_hashes(&h, &j, jt, "host vs the menu joiner");

    // ── 1: the joiner leaves (its app's quit: close = LEAVE) ──
    j.p.0.borrow_mut().close();
    let left_at = h.next;
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut host_heard = false;
    while !host_heard || h.next < left_at + 150 || c1.next < left_at + 150 {
        for e in h.frame(true) { if matches!(e, CoreEvent::ClientLeft { slot: 2, graceful: true }) { host_heard = true; } }
        for e in c1.frame(false) { assert!(!matches!(e, CoreEvent::Desync { .. }), "no desync after the leave"); }
        sleep();
        assert!(Instant::now() < deadline, "the match runs on after the leave");
    }
    same_hashes(&h, &c1, left_at.saturating_sub(30), "host vs client through the leave");
    // The body stands (DropPolicy::Neutral): slot 2 still has its agent on the host.
    assert!(h.scene.as_ref().unwrap().sim.as_ref().unwrap().mp_bonds[2].is_some(), "the leaver's body stands on NEUTRAL");

    // ── 1: the leaver reconnects through the menus: slot 2 again, no new body ──
    let (_fe2, mut j2) = menu_late_join(&path, PORT, &mut h, &mut c1, 2, 1);
    let jt2 = arm_join(&mut h, &mut c1, &mut j2, 2);
    run_three(&mut h, &mut c1, &mut j2, jt2 + 120);
    assert_eq!(h.spawned, Some(false), "a reconnect spawns no second body (the reconnect path)");
    same_hashes(&h, &c1, jt2, "host vs client after the reconnect");
    same_hashes(&h, &j2, jt2, "host vs the reconnected machine");
}
