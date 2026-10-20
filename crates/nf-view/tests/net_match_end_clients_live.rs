//! net-match-end-clients (700w0): the debriefing handover over the real loopback session —
//! every machine simulates through its own final tick and opens the same debriefing, and the
//! host's session stays alive until every peer has acked (docs/systems/net-match-end.md). The
//! soak's failure shape (the host tearing down at its own match end, the joiner stranded at
//! frame 107653 of 108000 with "[net] The host left the match") is the regression these tests
//! pin against: a 3-machine match ends by the TIME limit and by the FRAG limit with all three
//! boards identical and no leave event anywhere, and the real departures mid-match still fire
//! the existing events (HostLeft for the host, ClientLeft for a client). The harness models the
//! app's match-end policy the way net_e2e_loopback_live models the handover (a machine whose sim
//! has requested the debriefing stops ticking and submitting, but its session keeps polling).
//! NF_ISO-gated (the arena loads from the disc like net_match_live).

use std::time::{Duration, Instant};

use nf_game::mp::debrief::board_hash;
use nf_view::menu::{Action, FrontEnd, Page};
use nf_view::mpnet::{CoreEvent, SessionProxy};

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// Distinct per test binary: the binaries' tests run in parallel threads.
const PORT: u16 = 28177;
const PORT_FRAG: u16 = 28187;
const ARENA: u32 = 0x0700_004B; // MP_MAP_ROWS[7] = Ravine — the lightest arena that fits three scenes in the sandbox (Skyrail OOMs at 3 × ~1 GB)

/// One action, then the frames a player would wait (net_e2e_loopback_live's act): the
/// page-change frames and a slice of real time for the lobby's exchange; every peer FE breathes
/// too, so the sessions answer each other's keepalives.
fn act(fe: &mut FrontEnd, others: &mut [&mut FrontEnd], a: Action) -> bool {
    let r = fe.action(a);
    for _ in 0..60 {
        fe.tick();
        for o in others.iter_mut() {
            o.tick();
        }
    }
    for _ in 0..5000 {
        if !fe.intro_playing && !fe.attract_playing {
            break;
        }
        fe.tick();
        for o in others.iter_mut() {
            o.tick();
        }
    }
    r
}

/// Title → START → P_PARISENUM → P_ESTHERO → Main (net_e2e_loopback_live's to_main; the skip is
/// refused for the 0xb4-tick grace [M P_ESTHERO 0x20a060:16-45, Menu_StopMovie 0x215a50:6]).
fn to_main(fe: &mut FrontEnd) {
    assert_eq!(fe.page, Page::Start, "the boot opens the title");
    assert!(fe.action(Action::Confirm), "the title's Enter");
    assert_eq!(fe.page, Page::ParisEnum, "START opens P_PARISENUM [S unverified: the send is menu-def script data]");
    for _ in 0..8 {
        if fe.intro_movie {
            break;
        }
        fe.tick();
    }
    assert_eq!(fe.page, Page::EstHero, "P_ESTHERO plays the intro [M 0x20a648:43]");
    assert!(fe.intro_movie);
    for _ in 0..nf_view::menu::ESTHERO_GRACE {
        fe.tick();
    }
    assert!(fe.action(Action::Confirm), "the skip past the grace");
    fe.tick();
    assert_eq!(fe.page, Page::Main, "the stopped movie's next 0x50 opens Main [M 0x20a060:28-45]");
}

/// Main → Multiplayer → Host Online → the port page → Start Listening (the real
/// `mp_host_begin`) [S net-online-lobby §2 — the game's Multiplayer row opens 0x40000019,
/// C_GOMULTIPLAYER 0x216a78].
fn to_host_page(fe: &mut FrontEnd) {
    to_main(fe);
    assert!(act(fe, &mut [], Action::Down), "the Multiplayer row");
    assert!(act(fe, &mut [], Action::Confirm), "Multiplayer");
    assert_eq!(fe.page, Page::MpMode, "the mode choice [S port page]");
    assert!(act(fe, &mut [], Action::Down), "the Host Online row");
    assert!(act(fe, &mut [], Action::Confirm), "Host Online");
    assert_eq!(fe.page, Page::MpHost, "the port page [S port page]");
}

/// Main → Multiplayer → Join Online → the address page [S port pages, net-online-lobby §2].
fn to_join_page(fe: &mut FrontEnd) {
    to_main(fe);
    assert!(act(fe, &mut [], Action::Down), "the Multiplayer row");
    assert!(act(fe, &mut [], Action::Confirm), "Multiplayer");
    assert_eq!(fe.page, Page::MpMode, "the mode choice [S port page]");
    assert!(act(fe, &mut [], Action::Down), "past Local");
    assert!(act(fe, &mut [], Action::Down), "the Join Online row");
    assert!(act(fe, &mut [], Action::Confirm), "Join Online");
    assert_eq!(fe.page, Page::MpNetJoin, "the address page [S port page]");
}

fn host_listen(fe: &mut FrontEnd, port: u16) {
    for _ in 0..6 {
        assert!(fe.mp_net_backspace(), "the backspace clears the default port");
    }
    for c in port.to_string().chars() {
        assert!(fe.mp_net_type(c), "the port digit {c} lands");
    }
    assert!(act(fe, &mut [], Action::Down), "the Start Listening row");
    assert!(act(fe, &mut [], Action::Confirm), "Start Listening");
    assert!(fe.mp_hosting, "the host is hosting");
    assert_eq!(fe.page, Page::MpOnlineLobby, "the online path lands on the ONE screen [S net-online-lobby §6]");
}

fn client_connect(fe: &mut FrontEnd, host: &mut FrontEnd, addr: &str) {
    for c in addr.chars() {
        assert!(fe.mp_net_type(c), "the address char {c} lands");
    }
    assert!(act(fe, &mut [host], Action::Confirm), "Connect");
    // The host FE breathes inside act, so on loopback the handshake may already have completed
    // here (the state past Dialing); either way the lobby must come up.
    assert!(fe.page == Page::MpOnlineLobby || fe.mp_net_status() == Some("Connecting..."),
        "the dial started (page {:?}, status {:?})", fe.page, fe.mp_net_status());
    let deadline = Instant::now() + Duration::from_secs(10);
    while fe.page != Page::MpOnlineLobby {
        fe.tick();
        host.tick();
        std::thread::sleep(Duration::from_millis(2));
        assert!(Instant::now() < deadline, "the client reaches the lobby");
    }
}

/// The scripted per-slot controls (net_e2e_loopback_live's script — every human moves).
fn scripted_input(t: u32, slot: u8) -> nf_net::Input {
    let mut i = nf_net::NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 97) as f32 / 97.0;
    i.strafe = if t % 11 == 0 { -0.5 } else { 0.0 };
    i.turn = ((t % 13) as f32 - 6.0) * 0.002;
    i.look = if t % 29 == 0 { 0.25 } else { 0.0 };
    i.jump = t % 61 == 0;
    i.crouch = t % 43 == 0;
    i.fire = t % 23 == 0;
    i
}

/// One machine of the running match: its scene, its session, its own sim tick, and every core
/// event it has seen (the no-leave pins read these).
struct Mach {
    scene: nf_view::scene::Scene,
    sess: SessionProxy,
    slot: usize,
    tick: u32,
    done: bool,
    events: Vec<String>,
    /// The debriefing front end the handover built (the boards' comparison reads it).
    fe_board: Option<FrontEnd>,
}

impl Mach {
    fn pump(&mut self) {
        let evs = {
            let mut c = self.sess.0.borrow_mut();
            c.poll();
            c.take_app_events()
        };
        for e in evs {
            self.events.push(format!("{e:?}"));
        }
    }
    fn submit(&mut self) {
        if !self.done {
            self.sess.0.borrow_mut().local_input(self.tick, scripted_input(self.tick, self.slot as u8));
        }
    }
    fn advance(&mut self, kills: &[u32]) {
        if self.done {
            return;
        }
        let rows = {
            self.sess.0.borrow_mut().ready_inputs(self.tick)
        };
        let Some(rows) = rows else { return };
        {
            let sim = self.scene.sim.as_mut().expect("the match sim");
            nf_view::net_tick(sim, &rows, self.slot);
        }
        self.tick += 1;
        let sim = self.scene.sim.as_mut().expect("the match sim");
        if kills.contains(&self.tick) {
            let (mp, rng) = (&mut sim.mp, &mut sim.drones.rng);
            mp.as_mut().expect("the mp match").player_killed(1, Some(0), false, false, rng);
        }
        if sim.mp.as_ref().expect("the mp match").game.request_debriefing {
            self.done = true;
            // The app's handover (net-match-end §4): the finished machine
            // announces its own end over the real session — the final tick
            // plus its board's fold (Data tag 4; the debriefing open in the
            // runner sends exactly this on every netted machine).
            let bh = nf_game::mp::debrief::board_hash(&sim.mp.as_ref().unwrap().debriefing());
            self.sess.0.borrow_mut().send_debrief_ack(self.tick, bh);
        }
    }
}

/// The main agent, 2026-10-19: after the fixed-step loop, pump until all three have run `to` (bounded). UDP delivery is
/// asynchronous, so a step can leave a machine behind — and a machine behind never submitted the ticks it missed
/// (`submit` sends its CURRENT tick), so it submits its current tick here too. Without this the "all three ran the
/// same ticks" asserts failed 1-2 runs in 3 under load.
fn catch_up(mh: &mut Mach, m1: &mut Mach, m2: &mut Mach, to: u32) {
    let t0 = Instant::now();
    while (mh.tick, m1.tick, m2.tick) != (to, to, to) && t0.elapsed() < Duration::from_secs(5) {
        for m in [&mut *mh, &mut *m1, &mut *m2] { if m.tick < to { m.submit(); } }
        mh.pump();
        m1.pump();
        m2.pump();
        mh.advance(&[]);
        m1.advance(&[]);
        m2.advance(&[]);
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// The three-machine lobby walk + match start: the host listens on `port`, both clients join,
/// `limit_left` steps the Limit row and `duration_left` the Duration row (the real rows path
/// [M P_MPRULES 0x20f790:21-76's lists]), then Start hands every machine its settings.
fn start_three_machine_match(path: &str, port: u16, limit_left: usize, duration_left: usize) -> (Mach, Mach, Mach, nf_game::mp::MpSettings) {
    let (hp, c1p, c2p) = (
        SessionProxy::host(0).expect("the host proxy"),
        SessionProxy::host(0).expect("the client 1 placeholder"),
        SessionProxy::host(0).expect("the client 2 placeholder"),
    );
    let mut feh = FrontEnd::new();
    feh.mp_session = Some(Box::new(hp.clone()));
    let mut fe1 = FrontEnd::new();
    fe1.mp_session = Some(Box::new(c1p.clone()));
    let mut fe2 = FrontEnd::new();
    fe2.mp_session = Some(Box::new(c2p.clone()));

    to_host_page(&mut feh);
    host_listen(&mut feh, port);
    to_join_page(&mut fe1);
    client_connect(&mut fe1, &mut feh, &format!("127.0.0.1:{port}"));
    to_join_page(&mut fe2);
    client_connect(&mut fe2, &mut feh, &format!("127.0.0.1:{port}"));

    // The lobby mirrors all three agents.
    let deadline = Instant::now() + Duration::from_secs(15);
    while !(feh.mp_join[1].joined && feh.mp_join[2].joined && fe1.mp_join[0].joined && fe2.mp_join[0].joined) {
        feh.tick();
        fe1.tick();
        fe2.tick();
        std::thread::sleep(Duration::from_millis(2));
        assert!(Instant::now() < deadline, "all three lobbies see the three agents");
    }
    assert_eq!(feh.mp_human_count(), 3, "the host counts all three humans");

    // The rules rows (the real steps).
    for _ in 0..limit_left {
        let lim = feh.mp_lobby_rows().unwrap().iter()
            .position(|r| r.id == nf_view::mp_lobby::MpLobbyRowId::Limit).unwrap();
        feh.mp_one.focus = lim;
        assert!(act(&mut feh, &mut [&mut fe1, &mut fe2], Action::Left), "the limit steps down");
    }
    for _ in 0..duration_left {
        let dur = feh.mp_lobby_rows().unwrap().iter()
            .position(|r| r.id == nf_view::mp_lobby::MpLobbyRowId::Duration).unwrap();
        feh.mp_one.focus = dur;
        assert!(act(&mut feh, &mut [&mut fe1, &mut fe2], Action::Left), "the duration steps down");
    }

    let start = feh.mp_lobby_rows().unwrap().iter()
        .position(|r| r.id == nf_view::mp_lobby::MpLobbyRowId::Start).unwrap();
    feh.mp_one.focus = start;
    assert!(act(&mut feh, &mut [&mut fe1, &mut fe2], Action::Confirm), "Start hands over [M P_MPCONFIRM 0x20ec00:94-157]");
    let hs = feh.mp_match_settings.clone().expect("the host's settings ride the request");

    // The clients' handover: the tunnelled match payload reaches both (the paired FE ticks pump
    // the sessions).
    let deadline = Instant::now() + Duration::from_secs(15);
    while fe1.requested_level.is_none() || fe2.requested_level.is_none() {
        feh.tick();
        fe1.tick();
        fe2.tick();
        std::thread::sleep(Duration::from_millis(2));
        assert!(Instant::now() < deadline, "both clients receive the match start");
    }
    let cs1 = fe1.mp_match_settings.clone().expect("the client 1 handover settings");
    let cs2 = fe2.mp_match_settings.clone().expect("the client 2 handover settings");
    assert_eq!(cs1.seed, hs.seed, "the tunnelled seed is the host's");
    assert_eq!(cs2.seed, hs.seed);
    assert_eq!(cs1.time_limit, hs.time_limit, "the time limit rode the tunnel (seconds [M mp_confirm_start's ×0x3c])");
    assert_eq!(cs2.time_limit, hs.time_limit);
    assert_eq!(cs1.human_count, 3);
    assert_eq!(cs2.human_count, 3);

    // The arena loads, one machine at a time: three concurrent scene loads
    // spike past a 4 GB sandbox (OOM-kill, dmesg `anon-rss` 2.9 GB). Every
    // session is still polled throughout each blocking load, so a load longer
    // than the 5 s session timeout must not drop a peer — the property the
    // old three-worker load exercised, with one load's transient at a time.
    let arena = format!("{ARENA:08x}.bin");
    let hs2 = hs.clone();
    let cs1b = cs1.clone();
    let cs2b = cs2.clone();
    let (hscene, s1, s2) = std::thread::scope(|sc| {
        let poll_all = || {
            let _ = hp.0.borrow_mut().poll();
            let _ = c1p.0.borrow_mut().poll();
            let _ = c2p.0.borrow_mut().poll();
        };
        let hj = sc.spawn(|| nf_view::load_mp_carry(path, &arena, false, None, hs2).expect("the host sim loads"));
        while !hj.is_finished() {
            poll_all();
            std::thread::sleep(Duration::from_millis(16));
        }
        let hscene = hj.join().unwrap();
        let j1 = sc.spawn(|| nf_view::load_mp_carry(path, &arena, false, None, cs1b).expect("the client 1 sim loads"));
        while !j1.is_finished() {
            poll_all();
            std::thread::sleep(Duration::from_millis(16));
        }
        let s1 = j1.join().unwrap();
        let j2 = sc.spawn(|| nf_view::load_mp_carry(path, &arena, false, None, cs2b).expect("the client 2 sim loads"));
        while !j2.is_finished() {
            poll_all();
            std::thread::sleep(Duration::from_millis(16));
        }
        let s2 = j2.join().unwrap();
        (hscene, s1, s2)
    });
    let mut mh = Mach { scene: hscene, sess: hp, slot: 0, tick: 0, done: false, events: Vec::new(), fe_board: None };
    let mut m1 = Mach { scene: s1, sess: c1p, slot: 1, tick: 0, done: false, events: Vec::new(), fe_board: None };
    let mut m2 = Mach { scene: s2, sess: c2p, slot: 2, tick: 0, done: false, events: Vec::new(), fe_board: None };
    mh.scene.sim.as_mut().expect("host sim").swap_local_slot(0);
    m1.scene.sim.as_mut().expect("client 1 sim").swap_local_slot(1);
    m2.scene.sim.as_mut().expect("client 2 sim").swap_local_slot(2);
    // The match start's per-machine generator (the online start seeds it by slot).
    for (m, slot) in [(&mut mh, 0u32), (&mut m1, 1u32), (&mut m2, 2u32)] {
        m.scene.sim.as_mut().unwrap().net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ slot.wrapping_mul(0x9e37_79b9)));
        assert!(m.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().settings.started, "slot {slot}'s match started [M MP_Start 0x183a50:207]");
    }
    (mh, m1, m2, hs)
}

/// The lockstep run to every machine's own final tick, with the app's match-end policy modelled:
/// a machine whose sim has requested the debriefing stops ticking and submitting (the front end
/// gates the sim ticks) but its session keeps polling. Returns (end ticks per machine, the
/// boards' folds).
fn run_to_debriefing(machines: (&mut Mach, &mut Mach, &mut Mach), kills: &[u32], limit_ticks: u32) -> ([u32; 3], [u64; 3]) {
    let (mh, m1, m2) = machines;
    let deadline = Instant::now() + Duration::from_secs(240);
    while !(mh.done && m1.done && m2.done) {
        mh.submit();
        m1.submit();
        m2.submit();
        // Two pump passes: the host's catch of a client's fresh input and the
        // clients' catch of the host's relay both land inside the step, so a
        // step runs one tick whatever the wall clock does (one pass left the
        // step's progress to the relay rhythm — two iterations per tick).
        mh.pump();
        m1.pump();
        m2.pump();
        mh.pump();
        m1.pump();
        m2.pump();
        mh.advance(kills);
        m1.advance(kills);
        m2.advance(kills);
        // The per-tick state hash, exchanged every 15 ticks (net_e2e_loopback_live's shape) —
        // the machines must stay bit-identical for the boards to be.
        if mh.tick == m1.tick && m1.tick == m2.tick && mh.tick > 0 && mh.tick % 15 == 0 && !mh.done {
            let hashes = [
                mh.scene.sim.as_ref().unwrap().net_state_hash(0),
                m1.scene.sim.as_ref().unwrap().net_state_hash(1),
                m2.scene.sim.as_ref().unwrap().net_state_hash(2),
            ];
            assert_eq!(hashes[0], hashes[1], "tick {}: the host and client 1 agree", mh.tick);
            assert_eq!(hashes[0], hashes[2], "tick {}: the host and client 2 agree", mh.tick);
            mh.sess.0.borrow_mut().submit_hash(mh.tick, hashes[0]);
            m1.sess.0.borrow_mut().submit_hash(m1.tick, hashes[1]);
            m2.sess.0.borrow_mut().submit_hash(m2.tick, hashes[2]);
        }
        assert!(mh.tick <= limit_ticks + 60 && m1.tick <= limit_ticks + 60 && m2.tick <= limit_ticks + 60,
            "no machine ran past its final tick+60 (host {}, c1 {}, c2 {})", mh.tick, m1.tick, m2.tick);
        assert!(Instant::now() < deadline, "the match reached every debriefing within 4 minutes");
        std::thread::sleep(Duration::from_millis(1));
    }
    // The app's end handover: the finished machines announced their ends (the
    // tag-4 ack at the done transition) and the host keeps polling its live
    // session until every human peer's ack landed — the end wait's done
    // condition (docs/systems/net-match-end.md §4). The loop above exits the
    // moment the last sim finishes; the acks ride the next polls.
    let deadline = Instant::now() + Duration::from_secs(10);
    while !(mh.events.iter().any(|e| e.contains("DebriefAck { slot: 1"))
        && mh.events.iter().any(|e| e.contains("DebriefAck { slot: 2")))
    {
        mh.pump();
        m1.pump();
        m2.pump();
        assert!(Instant::now() < deadline, "the host's end wait collects both clients' acks");
        std::thread::sleep(Duration::from_millis(1));
    }
    let ends = [mh.tick, m1.tick, m2.tick];
    let folds = [
        board_hash(&mh.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().debriefing()),
        board_hash(&m1.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().debriefing()),
        board_hash(&m2.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().debriefing()),
    ];
    (ends, folds)
}

/// No machine saw a departure or a desync — the soak's "[net] The host left the match" is the
/// regression.
fn assert_no_leaves(ms: [&Mach; 3]) {
    for (i, m) in ms.iter().enumerate() {
        for e in &m.events {
            assert!(!e.contains("ClientLeft") && !e.contains("HostLeft") && !e.contains("HostTimeout"),
                "machine {i} saw a departure event: {e}");
            assert!(!e.contains("Desync"), "machine {i} desynced: {e}");
        }
    }
}

/// The time-limit end: three machines, the Duration row at 1 minute, the host stops ticking at
/// its own final tick while both clients are still draining — the live session keeps feeding
/// them, both clients reach their own final tick, both acks arrive (the real Data-tag-4 acks
/// [S]), and all three boards are identical. No "left" event anywhere (the soak's regression).
#[test]
fn three_machines_reach_the_debriefing_with_identical_boards() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut mh, mut m1, mut m2, hs) = start_three_machine_match(&path, PORT, 0, 9);
    assert_eq!(hs.time_limit, 60, "the duration row's 1 minute is 60 s [M the ×0x3c]");
    let limit_ticks = (hs.time_limit as u32) * 60;
    let (ends, folds) = run_to_debriefing((&mut mh, &mut m1, &mut m2), &[], limit_ticks);
    // The end is the same tick on every machine (the same deterministic sim — the elapsed clock
    // is ticks/60 [M 0x184268:44-46]).
    assert_eq!(ends[0], ends[1], "the host and client 1 ended on the same tick");
    assert_eq!(ends[0], ends[2], "the host and client 2 ended on the same tick");
    assert!(ends[0] >= limit_ticks, "the end is past the time limit's tick");
    // The acks rode the real session: the host saw both clients' tag-4 with the same fold and
    // the same final tick.
    let acks: Vec<(u8, u32, u64)> = mh.events.iter().filter_map(|e| {
        let r = e.split("DebriefAck { slot: ").nth(1)?;
        let slot: u8 = r.get(..1)?.parse().ok()?;
        let tick: u32 = r.split("final_tick: ").nth(1)?.split(",").next()?.trim().parse().ok()?;
        // The event's Debug prints the fold in decimal (derive on u64).
        let hash: u64 = r.split("board_hash: ").nth(1)?.split(" }").next()?.trim().parse().ok()?;
        Some((slot, tick, hash))
    }).collect();
    assert!(acks.iter().any(|&(s, _, _)| s == 1), "client 1's ack arrived (Debrief events: {:?})",
        mh.events.iter().filter(|e| e.contains("Debrief")).collect::<Vec<_>>());
    assert!(acks.iter().any(|&(s, _, _)| s == 2), "client 2's ack arrived");
    assert!(acks.iter().all(|&(_, t, _)| t == ends[0]), "every ack carries the shared final tick");
    assert!(acks.iter().all(|&(_, _, h)| h == folds[0]), "every ack's board fold matches the host's");
    assert_no_leaves([&mh, &m1, &m2]);
    // All three land on the debriefing with identical boards.
    for m in [&mut mh, &mut m1, &mut m2] {
        let mut fe = FrontEnd::new();
        fe.open_mp_debriefing(m.scene.sim.as_ref().unwrap().mp.as_ref().unwrap());
        assert!(fe.reopen_queued_page(), "slot {}'s load tail opens the queued debriefing [M 0x1d6360:160-163]", m.slot);
        assert_eq!(fe.page, Page::MpDebriefing, "slot {} lands on the debriefing", m.slot);
        m.fe_board = Some(fe);
    }
    let rows_of = |m: &Mach| -> Vec<(u32, u32, i32, i32, i32, i32)> {
        m.fe_board.as_ref().unwrap().mp_debriefing_rows().iter()
            .map(|r| (r.0, r.2, r.3, r.4, r.5, r.6))
            .collect()
    };
    let (rh, r1, r2) = (rows_of(&mh), rows_of(&m1), rows_of(&m2));
    assert_eq!(r1, rh, "client 1's board rows equal the host's (rank label, portrait, points, victories, deaths, total)");
    assert_eq!(r2, rh, "client 2's board rows equal the host's");
    assert_eq!(folds[0], folds[1], "the host's and client 1's board folds agree");
    assert_eq!(folds[0], folds[2], "the host's and client 2's board folds agree");
}

/// Sister: the host really departs mid-match — the client's session fires `CoreEvent::HostLeft`
/// (the app's "The host left the match" path [S mp-online-failures]) and the input flow stops.
/// All three machines submit (three humans — a missing slot stalls the lockstep).
#[test]
fn a_host_departure_mid_match_is_still_host_left() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut mh, mut m1, mut m2, _hs) = start_three_machine_match(&path, PORT + 2, 0, 0);
    // 200 ticks of honest lockstep, then the host's session closes (a real departure).
    for t in 0..200u32 {
        mh.submit();
        m1.submit();
        m2.submit();
        // Two pump passes: the relay converges within the step (the host's
        // catch of a client's fresh input and the clients' catch of the relay
        // both land here), so a step runs one tick at any sandbox speed.
        mh.pump();
        m1.pump();
        m2.pump();
        mh.pump();
        m1.pump();
        m2.pump();
        mh.advance(&[]);
        m1.advance(&[]);
        m2.advance(&[]);
        if t % 50 == 0 {
            eprintln!("[sister] t{t}: host tick {} c1 {} c2 {}", mh.tick, m1.tick, m2.tick);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    catch_up(&mut mh, &mut m1, &mut m2, 200);
    assert_eq!((mh.tick, m1.tick, m2.tick), (200, 200, 200), "all three ran the same ticks (host, c1, c2)");
    eprintln!("[sister] closing the host's session");
    mh.sess.0.borrow_mut().close();
    eprintln!("[sister] closed; waiting for the client's HostLeft");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !m1.events.iter().any(|e| e.contains("HostLeft")) {
        m1.pump();
        m2.pump();
        std::thread::sleep(Duration::from_millis(5));
        assert!(Instant::now() < deadline, "the client's session reports HostLeft within 10 s");
    }
}

/// Sister: a client that really leaves mid-match fires `ClientLeft` on the host (graceful) and
/// the match runs on — the departed slot rides NEUTRAL from its boundary [M Session::drop_slot's
/// `left_at`], so the host's later ticks stay ready.
#[test]
fn a_client_departure_mid_match_still_runs_the_match_on() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut mh, mut m1, mut m2, _hs) = start_three_machine_match(&path, PORT + 4, 0, 0);
    for t in 0..200u32 {
        mh.submit();
        m1.submit();
        m2.submit();
        // Two pump passes, as in the host-departure sister: the relay
        // converges within the step, so a step runs one tick at any sandbox
        // speed (one pass left every step at the relay's mercy — two
        // iterations per tick on a fast loop).
        mh.pump();
        m1.pump();
        m2.pump();
        mh.pump();
        m1.pump();
        m2.pump();
        mh.advance(&[]);
        m1.advance(&[]);
        m2.advance(&[]);
        std::thread::sleep(Duration::from_millis(1));
    }
    catch_up(&mut mh, &mut m1, &mut m2, 200);
    assert_eq!((mh.tick, m1.tick, m2.tick), (200, 200, 200), "all three ran the same ticks (host, c1, c2)");
    m1.sess.0.borrow_mut().close();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !mh.events.iter().any(|e| e.contains("ClientLeft")) {
        mh.pump();
        m2.pump();
        std::thread::sleep(Duration::from_millis(5));
        assert!(Instant::now() < deadline, "the host's session reports the client's departure within 10 s");
    }
    assert!(mh.events.iter().any(|e| e.contains("ClientLeft { slot: 1") && e.contains("graceful: true")),
        "the departure is slot 1's graceful quit");
    // The match runs on: the ticks past the departure are ready (the departed slot NEUTRAL, the
    // other two still submitting).
    // The match runs on: the departed slot rides NEUTRAL from its boundary
    // [M Session::drop_slot's left_at], the other two keep sampling every tick
    // (the app submits each tick's controls), and the live relay carries both
    // to the host — the relay sends contiguous runs from its last sent tick,
    // so the ticks must be sampled without gaps for the later ones to turn
    // ready.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut t = 200u32;
    while t < 260 {
        mh.sess.0.borrow_mut().local_input(t, scripted_input(t, 0));
        m2.sess.0.borrow_mut().local_input(t, scripted_input(t, 2));
        mh.pump();
        m2.pump();
        mh.pump();
        m2.pump();
        if mh.sess.0.borrow_mut().ready_inputs(t).is_some() {
            t += 1;
        }
        assert!(Instant::now() < deadline, "the match runs on to tick 260 after the departure");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Sister 2: the frag-limit end behaves the same as the time-limit end — the same 3-machine
/// match ended by two scripted kills (the game's scoring path [M 0x188f70]) reaches three
/// identical debriefings with no leave event, and the acks agree.
#[test]
fn the_frag_limit_end_reaches_the_same_debriefing() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut mh, mut m1, mut m2, hs) = start_three_machine_match(&path, PORT_FRAG, 8, 0);
    assert_eq!(hs.frag_limit, 2, "the limit row's 2 — the scripted kills end the match");
    assert_eq!(hs.time_limit, 600, "the duration stays at its default 10 minutes");
    let (ends, folds) = run_to_debriefing((&mut mh, &mut m1, &mut m2), &[400, 900], (hs.time_limit as u32) * 60);
    assert_eq!(ends[0], ends[1]);
    assert_eq!(ends[0], ends[2]);
    let acks = mh.events.iter().filter(|e| e.contains("DebriefAck")).count();
    assert!(acks >= 2, "both clients' acks arrived (saw {acks})");
    assert_no_leaves([&mh, &m1, &m2]);
    for m in [&mut mh, &mut m1, &mut m2] {
        let mut fe = FrontEnd::new();
        fe.open_mp_debriefing(m.scene.sim.as_ref().unwrap().mp.as_ref().unwrap());
        assert!(fe.reopen_queued_page(), "slot {}'s load tail opens the queued debriefing", m.slot);
        assert_eq!(fe.page, Page::MpDebriefing, "slot {} lands on the debriefing", m.slot);
        m.fe_board = Some(fe);
    }
    let rows_of = |m: &Mach| -> Vec<(u32, u32, i32, i32, i32, i32)> {
        m.fe_board.as_ref().unwrap().mp_debriefing_rows().iter()
            .map(|r| (r.0, r.2, r.3, r.4, r.5, r.6))
            .collect()
    };
    let (rh, r1, r2) = (rows_of(&mh), rows_of(&m1), rows_of(&m2));
    assert_eq!(r1, rh, "client 1's board rows equal the host's");
    assert_eq!(r2, rh, "client 2's board rows equal the host's");
    assert_eq!(folds[0], folds[1]);
    assert_eq!(folds[0], folds[2]);
}
