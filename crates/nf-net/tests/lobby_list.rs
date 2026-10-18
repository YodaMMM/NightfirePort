//! net-lobby-list (docs/systems/net-lobby-list.md): a host session answers discovery queries on its
//! game socket; the joiner's list shows it with the right fields, flags another build as not
//! joinable, expires a vanished host, and joining from a row connects. Loopback, numbers only [S].

use std::net::{SocketAddr, UdpSocket};

use nf_net::discovery::{answer, LobbyInfo, LobbyList, EXPIRE_MS};
use nf_net::net::Transport;
use nf_net::session::{Config, Event, Session};
use nf_net::{MatchSettings, UdpTransport};

fn host(name: &str, build: u32) -> (Session<UdpTransport>, LobbyInfo) {
    let tr = UdpTransport::bind("127.0.0.1:0").unwrap();
    let port = tr.local_addr().port();
    let s = Session::new_host(Config::default(), tr, MatchSettings::default(), 1);
    let info = LobbyInfo { name: name.into(), host: "Bond".into(), level_code: 0x0700_0023, scenario: 1, players: 1, max_players: 4, in_progress: false, protocol: nf_net::PROTOCOL_VERSION, build, port };
    (s, info)
}

/// The host's per-frame duty: answer every query that reached its game socket.
fn serve(h: &mut Session<UdpTransport>, info: &LobbyInfo, now: u64) {
    h.poll(now);
    for (from, d) in h.take_stray() {
        if let Some(r) = answer(&d, info) { h.send_raw(from, &r); }
    }
}

fn list_for(targets: Vec<SocketAddr>) -> LobbyList {
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    s.set_nonblocking(true).unwrap();
    LobbyList::with_socket(Some(s), targets, nf_net::PROTOCOL_VERSION, nf_net::DEFAULT_BUILD_ID, 77)
}

fn run(mut f: impl FnMut(u64), from: u64, to: u64) {
    let mut t = from;
    while t < to {
        f(t);
        std::thread::sleep(std::time::Duration::from_millis(2));
        t += 10;
    }
}

#[test]
fn a_host_appears_with_its_fields_and_ping() {
    let (mut h, info) = host("Bond's lobby", nf_net::DEFAULT_BUILD_ID);
    let addr: SocketAddr = format!("127.0.0.1:{}", info.port).parse().unwrap();
    let mut list = list_for(vec![addr]);
    run(|t| { list.poll(t); serve(&mut h, &info, t); list.poll(t); }, 0, 400);
    let rows = list.rows();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].addr, addr);
    assert_eq!(rows[0].info, info);
    assert!(rows[0].joinable);
    assert!(rows[0].ping_ms < 1000);
}

#[test]
fn another_build_is_listed_but_not_joinable_and_a_gone_host_expires() {
    let (mut h, info) = host("Old build", nf_net::DEFAULT_BUILD_ID + 1);
    let addr: SocketAddr = format!("127.0.0.1:{}", info.port).parse().unwrap();
    let mut list = list_for(vec![addr]);
    run(|t| { list.poll(t); serve(&mut h, &info, t); list.poll(t); }, 0, 400);
    assert_eq!(list.rows().len(), 1);
    assert!(!list.rows()[0].joinable, "a different build shows, greyed out");
    // The host stops answering: the row lasts EXPIRE_MS, then goes.
    run(|t| list.poll(t), 400, 400 + EXPIRE_MS + 200);
    assert!(list.rows().is_empty(), "a vanished host expires");
}

#[test]
fn two_hosts_on_one_pc_both_list_and_joining_a_row_connects() {
    let (mut a, ia) = host("A", nf_net::DEFAULT_BUILD_ID);
    let (mut b, ib) = host("B", nf_net::DEFAULT_BUILD_ID);
    let targets = vec![format!("127.0.0.1:{}", ia.port).parse().unwrap(), format!("127.0.0.1:{}", ib.port).parse().unwrap()];
    let mut list = list_for(targets);
    run(|t| { list.poll(t); serve(&mut a, &ia, t); serve(&mut b, &ib, t); list.poll(t); }, 0, 400);
    let rows = list.rows();
    assert_eq!(rows.iter().map(|r| r.info.name.as_str()).collect::<Vec<_>>(), ["A", "B"]);
    // Join B from its row.
    let mut c = Session::new_client(Config::default(), UdpTransport::bind("127.0.0.1:0").unwrap(), rows[1].addr);
    let mut ok = false;
    run(|t| { serve(&mut b, &ib, t); if c.poll(t).iter().any(|e| matches!(e, Event::Connected { .. })) { ok = true; } }, 400, 3000);
    assert!(ok, "the client connected to the listed host");
}

/// The internet directory (700v4 item 5): a host registers from its game socket, the server keys
/// it by the source address, and a joiner's list (no LAN targets) shows it from the directory.
#[test]
fn the_directory_lists_a_registered_host() {
    let srv = UdpSocket::bind("127.0.0.1:0").unwrap();
    let srv_addr = srv.local_addr().unwrap();
    std::thread::spawn(move || {
        let mut dir = nf_net::directory::Directory::default();
        let t0 = std::time::Instant::now();
        let mut buf = [0u8; 2048];
        // Like nf-lobbyd: a recv error (Windows reports an earlier ICMP unreachable this way) is
        // not the end of the server.
        loop {
            let Ok((n, from)) = srv.recv_from(&mut buf) else { continue };
            for r in dir.handle(from, &buf[..n], t0.elapsed().as_millis() as u64) { let _ = srv.send_to(&r, from); }
        }
    });
    let (mut h, info) = host("Internet game", nf_net::DEFAULT_BUILD_ID);
    h.send_raw(srv_addr, &nf_net::directory::register_packet(&info));
    let mut list = list_for(Vec::new());
    list.set_directory(Some(srv_addr));
    run(|t| { h.poll(t); list.poll(t); }, 0, 600);
    let rows = list.rows();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].addr, h.local_addr(), "the endpoint the server saw (the host's game socket)");
    assert_eq!(rows[0].info.name, "Internet game");
}
