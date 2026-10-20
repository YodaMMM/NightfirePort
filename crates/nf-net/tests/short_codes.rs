//! net-short-codes (docs/systems/net-short-codes.md): the server-assigned 4-character join
//! codes through the player-set lobby server. Loopback only, numbers only — these pin the
//! port's own protocol [S]; game behaviour is untouched. The unchanged long-code path is the
//! standing sister assertion: `loopback_e2e.rs::net_e2e_loopback_live` stays green.

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use nf_net::discovery::LobbyInfo;
use nf_net::directory::{
    parse_entries, parse_lookup_found, register_packet, register_packet_v2, short_code_string,
    LookupFound, NackReason, ShortCodeHost, ShortLookup, CODE_FRESH_MS, DIR_PROTOCOL, EXPIRE_MS,
    LOOKUP_EVERY_MS, LOOKUP_TRIES, LOOKUP_WINDOW_MAX, SHORT_CODE_ALPHABET,
};
use nf_net::session::{Config, Event, Session};
use nf_net::{MatchSettings, UdpTransport};

fn info(name: &str) -> LobbyInfo {
    LobbyInfo {
        name: name.into(),
        host: "Bond".into(),
        level_code: 0x0700_0023,
        scenario: 1,
        players: 1,
        max_players: 4,
        in_progress: false,
        protocol: nf_net::PROTOCOL_VERSION,
        build: nf_net::DEFAULT_BUILD_ID,
        port: 27420,
    }
}

/// A live `Directory` server on 127.0.0.1:0, one thread, like nf-lobbyd's main loop.
fn server() -> SocketAddr {
    let srv = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = srv.local_addr().unwrap();
    std::thread::spawn(move || {
        let mut dir = nf_net::directory::Directory::default();
        let t0 = std::time::Instant::now();
        let mut buf = [0u8; 2048];
        loop {
            let Ok((n, from)) = srv.recv_from(&mut buf) else { continue };
            for r in dir.handle(from, &buf[..n], t0.elapsed().as_millis() as u64) {
                let _ = srv.send_to(&r, from);
            }
        }
    });
    addr
}

fn run(mut f: impl FnMut(u64), from: u64, to: u64) {
    let mut t = from;
    while t < to {
        f(t);
        std::thread::sleep(Duration::from_millis(2));
        t += 10;
    }
}

/// The server's reply to one datagram sent from `s`.
fn ask_via(s: &UdpSocket, srv: SocketAddr, pkt: &[u8]) -> Vec<u8> {
    s.set_read_timeout(Some(Duration::from_millis(2000))).unwrap();
    s.send_to(pkt, srv).unwrap();
    let mut buf = [0u8; 2048];
    let n = s.recv(&mut buf).expect("the server answers");
    buf[..n].to_vec()
}

fn lookup_packet(nonce: u32, code: [u8; 4], ver: u8) -> Vec<u8> {
    let mut p = b"NFDS".to_vec();
    p.push(ver);
    p.extend_from_slice(&nonce.to_le_bytes());
    p.extend_from_slice(&code);
    p
}

/// A v2 REGISTER with this nonce from this socket, answered; returns the ack's (nonce, code).
fn register_and_ack(s: &UdpSocket, srv: SocketAddr, nonce: u32, name: &str) -> (u32, [u8; 4]) {
    let pkt = register_packet_v2(&info(name), nonce, Some("192.168.1.50:27420".parse().unwrap()));
    let reply = ask_via(s, srv, &pkt);
    assert_eq!(reply[0..4], *b"NFDA", "the server answers the v2 REGISTER with the assigned code");
    let n = u32::from_le_bytes(reply[4..8].try_into().unwrap());
    (n, reply[8..12].try_into().unwrap())
}

#[test]
fn host_registers_and_gets_a_four_character_code() {
    let srv = server();
    let host = UdpSocket::bind("127.0.0.1:0").unwrap();
    let (n, code) = register_and_ack(&host, srv, 0xDEAD, "Bond's lobby");
    assert_eq!(n, 0xDEAD, "the ack echoes the REGISTER's nonce");
    assert!(code.iter().all(|c| SHORT_CODE_ALPHABET.contains(c)), "4 unambiguous symbols: {code:?}");
    // A re-register from the same source keeps the code (the entry is keyed by the source).
    let (_, again) = register_and_ack(&host, srv, 0xBEEF, "Bond's lobby");
    assert_eq!(again, code, "the code is stable while the host keeps registering");
    // Another host (another source) gets another code (unique among the live entries).
    let host2 = UdpSocket::bind("127.0.0.1:0").unwrap();
    let (_, other) = register_and_ack(&host2, srv, 7, "Second lobby");
    assert_ne!(other, code);
    // Sister: a code nobody registered is answered "not found" — the comparison can fail.
    let reply = ask_via(&host, srv, &lookup_packet(99, *b"ZZZZ", DIR_PROTOCOL));
    assert_eq!(reply[0..4], *b"NFDN");
    assert_eq!(reply[8], NackReason::NotFound as u8);
}

#[test]
fn joiner_looks_up_and_connects() {
    let srv = server();
    // The host: a real session on its game socket, REGISTERing through it (the server keys the
    // entry — and the LOOKUP's answer — by that socket's address).
    let tr = UdpTransport::bind("127.0.0.1:0").unwrap();
    let mut h = Session::new_host(Config::default(), tr, MatchSettings::default(), 1);
    let mut sc = ShortCodeHost::new(123);
    h.send_raw(srv, &sc.register_packet(&info("Internet game"), None));
    let mut code = None;
    run(
        |t| {
            h.poll(t);
            for (_, d) in h.take_stray() {
                if sc.on_packet(&d, t) {
                    code = sc.code(t);
                }
            }
        },
        0,
        200,
    );
    let code = code.expect("the host's REGISTER was answered with a short code");

    // The joiner looks the code up and dials the endpoint the server saw; the real handshake
    // runs on 127.0.0.1.
    let js = UdpSocket::bind("127.0.0.1:0").unwrap();
    js.set_nonblocking(true).unwrap();
    let mut lk = ShortLookup::new(srv, normalize(code.as_str()).unwrap(), 42);
    let mut found: Option<LookupFound> = None;
    let js_for_send = js.try_clone().unwrap();
    run(
        |t| {
            let mut out: Vec<(SocketAddr, Vec<u8>)> = Vec::new();
            lk.poll(t, &mut |to, d| out.push((to, d.to_vec())));
            for (to, d) in out {
                let _ = js_for_send.send_to(&d, to);
            }
            let mut buf = [0u8; 2048];
            while let Ok((n, from)) = js.recv_from(&mut buf) {
                lk.on_packet(from, &buf[..n]);
            }
            if let Some(Ok(f)) = lk.result() {
                found = Some(f.clone());
            }
        },
        0,
        400,
    );
    let f = found.expect("the lookup found the host");
    assert_eq!(SocketAddr::V4(f.public), h.local_addr(), "the endpoint the server saw (the host's game socket)");
    assert_eq!(f.info.name, "Internet game");

    let mut c = Session::new_client(Config::default(), UdpTransport::bind("127.0.0.1:0").unwrap(), SocketAddr::V4(f.public));
    let mut ok = false;
    run(
        |t| {
            h.poll(t);
            if c.poll(t).iter().any(|e| matches!(e, Event::Connected { .. })) {
                ok = true;
            }
        },
        400,
        3000,
    );
    assert!(ok, "the joiner connected through the looked-up endpoint (the real handshake)");
}

fn normalize(s: &str) -> Option<[u8; 4]> {
    nf_net::directory::normalize_short_code(s)
}

#[test]
fn unknown_code_is_a_clear_not_found() {
    let srv = server();
    // Make sure the server has at least one live entry (the answer is still "not found").
    let (n, _code) = register_and_ack(&UdpSocket::bind("127.0.0.1:0").unwrap(), srv, 5, "Some lobby");
    let _ = n;
    let js = UdpSocket::bind("127.0.0.1:0").unwrap();
    js.set_nonblocking(true).unwrap();
    let mut lk = ShortLookup::new(srv, *b"QQ7V", 9);
    let js2 = js.try_clone().unwrap();
    let mut r: Option<NackReason> = None;
    run(
        |t| {
            let mut out: Vec<(SocketAddr, Vec<u8>)> = Vec::new();
            lk.poll(t, &mut |to, d| out.push((to, d.to_vec())));
            for (to, d) in out {
                let _ = js2.send_to(&d, to);
            }
            let mut buf = [0u8; 128];
            while let Ok((n, from)) = js.recv_from(&mut buf) {
                lk.on_packet(from, &buf[..n]);
            }
            if let Some(res) = lk.result() {
                r = res.as_ref().err().copied();
            }
        },
        0,
        400,
    );
    assert_eq!(r, Some(NackReason::NotFound), "a clear not-found, not a timeout");
}

#[test]
fn old_server_silence_times_out_cleanly_and_old_host_still_lists() {
    // An "old server": a socket that drains and never answers (pre-short-code nf-lobbyd).
    let old = UdpSocket::bind("127.0.0.1:0").unwrap();
    let old_addr = old.local_addr().unwrap();
    std::thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            let _ = old.recv_from(&mut buf);
        }
    });
    let js = UdpSocket::bind("127.0.0.1:0").unwrap();
    js.set_nonblocking(true).unwrap();
    let mut lk = ShortLookup::new(old_addr, *b"AB2D", 3);
    let js2 = js.try_clone().unwrap();
    let mut r: Option<NackReason> = None;
    let mut sent = 0;
    run(
        |t| {
            let mut out: Vec<(SocketAddr, Vec<u8>)> = Vec::new();
            lk.poll(t, &mut |to, d| out.push((to, d.to_vec())));
            for (to, d) in out {
                sent += 1;
                let _ = js2.send_to(&d, to);
            }
            let mut buf = [0u8; 128];
            while let Ok((n, from)) = js.recv_from(&mut buf) {
                lk.on_packet(from, &buf[..n]);
            }
            if let Some(res) = lk.result() {
                r = res.as_ref().err().copied();
            }
        },
        0,
        LOOKUP_TRIES as u64 * LOOKUP_EVERY_MS + 300,
    );
    assert_eq!(r, Some(NackReason::NoAnswer), "silence after the tries is a clean no-answer");
    assert_eq!(sent, LOOKUP_TRIES, "it asked LOOKUP_TRIES times, then gave up");

    // The old-HOST compatibility half: a v1 REGISTER (no tail) on the new server still lists.
    let srv = server();
    let host = UdpSocket::bind("127.0.0.1:0").unwrap();
    host.send_to(&register_packet(&info("Old host")), srv).unwrap();
    std::thread::sleep(Duration::from_millis(30));
    let reply = ask_via(&UdpSocket::bind("127.0.0.1:0").unwrap(), srv, &nf_net::directory::list_packet(7, 0));
    let (_, rows) = parse_entries(&reply).expect("the ENTRIES reply");
    assert_eq!(rows.len(), 1, "an old host registers and lists exactly as before");
    assert_eq!(rows[0].1.name, "Old host");
    // It drew no answer: an old host expects none (nothing to assert beyond the silence above).
}

#[test]
fn bad_version_is_answered_unsupported() {
    let srv = server();
    let reply = ask_via(&UdpSocket::bind("127.0.0.1:0").unwrap(), srv, &lookup_packet(11, *b"AB2D", DIR_PROTOCOL + 1));
    assert_eq!(reply[0..4], *b"NFDN");
    assert_eq!(reply[8], NackReason::Unsupported as u8, "a version the server does not know is answered cleanly");
}

#[test]
fn lookups_are_rate_limited_per_source() {
    let srv = server();
    let host_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let (n, code) = register_and_ack(&host_sock, srv, 21, "Rate target");
    let _ = n;
    let code = normalize(&code.iter().map(|b| *b as char).collect::<String>()).unwrap();
    // One source hammers past the window limit; every answer past it is "rate limited".
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    s.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    let mut limited = 0;
    for i in 0..LOOKUP_WINDOW_MAX + 5 {
        s.send_to(&lookup_packet(1000 + i, code, DIR_PROTOCOL), srv).unwrap();
        let mut buf = [0u8; 64];
        let _n = s.recv(&mut buf).expect("an answer per lookup");
        if buf[0..4] == *b"NFDN" && buf[8] == NackReason::RateLimited as u8 {
            limited += 1;
        }
    }
    assert_eq!(limited, 5, "the last 5 of {} lookups were refused", LOOKUP_WINDOW_MAX + 5);
    // A second source is unaffected (the limit is per source address).
    let reply = ask_via(&UdpSocket::bind("127.0.0.1:0").unwrap(), srv, &lookup_packet(7, code, DIR_PROTOCOL));
    assert_eq!(reply[0..4], *b"NFDT", "another source still gets the found answer");
}

#[test]
fn display_falls_back_when_the_server_stops_answering() {
    let mut sc = ShortCodeHost::new(7);
    let pkt = sc.register_packet(&info("A"), None);
    // The server's ack for this nonce, built the way the server builds it (the v2 tail is the
    // packet's last 11 bytes: [ver][nonce 4][lan 6] — the nonce sits at len-10..len-6).
    let mut ack = b"NFDA".to_vec();
    ack.extend_from_slice(&pkt[pkt.len() - 10..pkt.len() - 6]); // the tail's nonce
    ack.extend_from_slice(b"AB2D");
    assert!(sc.on_packet(&ack, 1000));
    assert_eq!(sc.code(1000).as_deref(), Some("AB2D"));
    assert_eq!(sc.code(1000 + CODE_FRESH_MS), Some("AB2D".to_string()), "fresh to the last ms");
    assert_eq!(sc.code(1000 + CODE_FRESH_MS + 1), None, "stale: the long code shows again");
    // A wrong nonce is not ours: the code does not change.
    let mut bad = ack.clone();
    bad[4] ^= 1;
    assert!(!sc.on_packet(&bad, 2000));
    // EXPIRE_MS is the entry's life: the fresh window must sit inside it (the code cannot
    // outlive the entry it points at).
    assert!(CODE_FRESH_MS < EXPIRE_MS);
    let _ = short_code_string(b"AB2D"); // the display form (kept referenced for the doc's sake)
}
