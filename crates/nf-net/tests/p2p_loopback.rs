//! net-p2p (docs/systems/net-p2p.md §6): the reachability layer against loopback fakes — a fake
//! UPnP IGD (SSDP responder + HTTP description + SOAP control), a fake NAT-PMP gateway, a fake STUN
//! server, and a real host/client session pair on 127.0.0.1. Port-only feature [S]; no game claims.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use nf_net::p2p::{self, natpmp, stun, upnp, HostReach, HostState, JoinReach, MapMethod, ReachConfig};
use nf_net::session::{Config, Event, Session};
use nf_net::net::Transport;
use nf_net::{MatchSettings, UdpTransport};

/// A fake IGD. Returns (ssdp addr, recorded SOAP actions). `fault_add` = the UPnPError code the
/// first AddPortMapping answers (0 = none).
fn fake_igd(fault_add: u32) -> (String, Arc<Mutex<Vec<String>>>) {
    let http = TcpListener::bind("127.0.0.1:0").unwrap();
    let hp = http.local_addr().unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let ssdp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let ssdp_addr = ssdp.local_addr().unwrap().to_string();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        while let Ok((n, from)) = ssdp.recv_from(&mut buf) {
            let m = String::from_utf8_lossy(&buf[..n]);
            if m.starts_with("M-SEARCH") && m.contains("InternetGatewayDevice:1") {
                let r = format!("HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=120\r\nST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\nLOCATION: http://{hp}/rootDesc.xml\r\n\r\n");
                let _ = ssdp.send_to(r.as_bytes(), from);
            }
        }
    });
    let l2 = log.clone();
    thread::spawn(move || {
        let mut fault = fault_add;
        for s in http.incoming() {
            let Ok(mut s) = s else { continue };
            s.set_read_timeout(Some(Duration::from_millis(500))).ok();
            let mut raw = Vec::new();
            let mut b = [0u8; 4096];
            // Read the header, then Content-Length bytes.
            loop {
                let n = s.read(&mut b).unwrap_or(0);
                if n == 0 { break; }
                raw.extend_from_slice(&b[..n]);
                let t = String::from_utf8_lossy(&raw).to_string();
                if let Some((h, body)) = t.split_once("\r\n\r\n") {
                    let cl: usize = upnp::header(h, "content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
                    if body.len() >= cl { break; }
                }
            }
            let req = String::from_utf8_lossy(&raw).to_string();
            let (status, body) = if req.starts_with("GET /rootDesc.xml") {
                // Chunked on purpose: the client must undo it.
                let desc = "<?xml version=\"1.0\"?><root><device><serviceList>\
                    <service><serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>\
                    <controlURL>/ctl/IPConn</controlURL></service></serviceList></device></root>";
                let chunked = format!("{:x}\r\n{desc}\r\n0\r\n\r\n", desc.len());
                let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/xml\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{chunked}");
                let _ = s.write_all(resp.as_bytes());
                continue;
            } else if req.starts_with("POST /ctl/IPConn") {
                let action = upnp::header(&req, "soapaction").unwrap_or_default();
                let action = action.trim_matches('"').rsplit('#').next().unwrap_or("").to_string();
                l2.lock().unwrap().push(format!("{action} {}", upnp::xml_tag(&req, "NewExternalPort").unwrap_or("")));
                match action.as_str() {
                    "AddPortMapping" if fault != 0 => {
                        let code = fault;
                        fault = 0;
                        (500, format!("<s:Envelope><s:Body><s:Fault><detail><UPnPError><errorCode>{code}</errorCode><errorDescription>x</errorDescription></UPnPError></detail></s:Fault></s:Body></s:Envelope>"))
                    }
                    "AddPortMapping" | "DeletePortMapping" => (200, "<s:Envelope><s:Body/></s:Envelope>".to_string()),
                    "GetExternalIPAddress" => (200, "<s:Envelope><s:Body><u:GetExternalIPAddressResponse><NewExternalIPAddress>203.0.113.77</NewExternalIPAddress></u:GetExternalIPAddressResponse></s:Body></s:Envelope>".to_string()),
                    _ => (401, String::new()),
                }
            } else {
                (404, String::new())
            };
            let resp = format!("HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            let _ = s.write_all(resp.as_bytes());
        }
    });
    (ssdp_addr, log)
}

#[test]
fn upnp_fake_igd_maps_reads_ip_and_deletes() {
    let (ssdp, log) = fake_igd(0);
    let (gw, public, lease) = upnp::map_port(&ssdp, 27420, 3600, Duration::from_millis(400)).expect("maps");
    assert_eq!(public, "203.0.113.77:27420".parse().unwrap());
    assert_eq!(lease, 3600);
    assert_eq!(gw.control_path, "/ctl/IPConn");
    assert_eq!(gw.local_ip, Some("127.0.0.1".parse().unwrap()));
    gw.delete_mapping(27420).unwrap();
    assert_eq!(*log.lock().unwrap(), vec!["AddPortMapping 27420", "GetExternalIPAddress ", "DeletePortMapping 27420"]);
}

#[test]
fn upnp_conflict_moves_to_the_next_port_and_permanent_only_retries_lease_0() {
    let (ssdp, log) = fake_igd(718);
    let (_, public, _) = upnp::map_port(&ssdp, 27420, 3600, Duration::from_millis(400)).expect("maps");
    assert_eq!(public.port(), 27421);
    assert_eq!(log.lock().unwrap()[..2], ["AddPortMapping 27420".to_string(), "AddPortMapping 27421".to_string()]);
    let (ssdp, _) = fake_igd(725);
    let (_, _, lease) = upnp::map_port(&ssdp, 27420, 3600, Duration::from_millis(400)).expect("maps");
    assert_eq!(lease, 0, "725 OnlyPermanentLeasesSupported → a permanent mapping (deleted on close)");
}

/// A fake NAT-PMP gateway granting `external` for any internal port.
fn fake_natpmp(external: u16) -> SocketAddr {
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    let a = s.local_addr().unwrap();
    thread::spawn(move || {
        let mut b = [0u8; 64];
        while let Ok((n, from)) = s.recv_from(&mut b) {
            let r: Vec<u8> = match (n, b[1]) {
                (2, 0) => vec![0, 128, 0, 0, 0, 0, 0, 1, 198, 51, 100, 9],
                (12, 1) => {
                    let mut r = vec![0, 129, 0, 0, 0, 0, 0, 1];
                    r.extend_from_slice(&b[4..6]);
                    r.extend_from_slice(&(if b[8..12] == [0; 4] { 0 } else { external }).to_be_bytes());
                    r.extend_from_slice(&b[8..12]);
                    r
                }
                _ => continue,
            };
            let _ = s.send_to(&r, from);
        }
    });
    a
}

#[test]
fn natpmp_fake_gateway_maps_and_deletes() {
    let gw = fake_natpmp(40500);
    let (public, life) = natpmp::map_port(gw, 27420, 3600).expect("maps");
    assert_eq!(public, "198.51.100.9:40500".parse().unwrap());
    assert_eq!(life, 3600);
    natpmp::delete(gw, 27420).expect("deletes");
    // Sister: a silent gateway is NoReply, not a mapping.
    let silent = UdpSocket::bind("127.0.0.1:0").unwrap();
    assert_eq!(natpmp::map_port(silent.local_addr().unwrap(), 27420, 3600), Err(natpmp::PmpError::NoReply));
}

/// A fake STUN server: answers every Binding request with XOR-MAPPED = `public` (or the sender's
/// own address when None).
fn fake_stun(public: Option<SocketAddr>) -> SocketAddr {
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    let a = s.local_addr().unwrap();
    thread::spawn(move || {
        let mut b = [0u8; 64];
        while let Ok((n, from)) = s.recv_from(&mut b) {
            if n < 20 || !stun::is_stun(&b[..n]) { continue; }
            let SocketAddr::V4(m) = public.unwrap_or(from) else { continue };
            let mut r = vec![0x01, 0x01, 0x00, 0x0c, 0x21, 0x12, 0xa4, 0x42];
            r.extend_from_slice(&b[8..20]);
            r.extend_from_slice(&[0x00, 0x20, 0x00, 0x08, 0x00, 0x01]);
            r.extend_from_slice(&(m.port() ^ 0x2112).to_be_bytes());
            r.extend_from_slice(&(u32::from(*m.ip()) ^ stun::MAGIC_COOKIE).to_be_bytes());
            let _ = s.send_to(&r, from);
        }
    });
    a
}

fn pump_until(mut f: impl FnMut(u64) -> bool, secs: u64) -> bool {
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(secs) {
        if f(t0.elapsed().as_millis() as u64) { return true; }
        thread::sleep(Duration::from_millis(5));
    }
    false
}

/// The host's reachability with a fake IGD, STUN through the session socket, then a client
/// connecting with the join code (the code decodes to the fake's public address, so the client is
/// pointed at 127.0.0.1 by rewriting the IP — the mapping itself is the router's job).
#[test]
fn host_reach_upnp_then_code_round_trip_and_session_connects() {
    let (ssdp, log) = fake_igd(0);
    let stun_a = fake_stun(None);
    let tr = UdpTransport::bind("127.0.0.1:0").unwrap();
    let port = tr.local_addr().port();
    let mut host = Session::new_host(Config::default(), tr, MatchSettings::default(), 7);
    let cfg = ReachConfig { stun_servers: vec![stun_a.to_string()], ssdp, ssdp_wait: Duration::from_millis(300), natpmp_gateway: None, map: true };
    let mut reach = HostReach::start(port, cfg, 1);
    let ok = pump_until(|now| {
        host.poll(now);
        for (from, d) in host.take_stray() { reach.on_packet(from, &d); }
        let mut out = Vec::new();
        reach.poll(now, &mut |to, d| out.push((to, d.to_vec())));
        for (to, d) in out { host.send_raw(to, &d); }
        let st = reach.status();
        st.state != HostState::Probing && st.code.is_some()
    }, 10);
    assert!(ok, "reach settled: {:?}", reach.status());
    let st = reach.status();
    assert_eq!(st.state, HostState::Mapped(MapMethod::Upnp));
    assert_eq!(st.public, Some(format!("203.0.113.77:{port}").parse().unwrap()));
    let jc = p2p::JoinCode::decode(st.code.as_ref().unwrap()).unwrap();
    assert_eq!(jc.addr.port(), port);
    assert_eq!(jc.flags & p2p::FLAG_NEEDS_PUNCH, 0, "a mapped host needs no punching");
    // A client dials the code's port on loopback.
    let ctr = UdpTransport::bind("127.0.0.1:0").unwrap();
    let mut client = Session::new_client(Config::default(), ctr, format!("127.0.0.1:{}", jc.addr.port()).parse().unwrap());
    let connected = pump_until(|now| {
        host.poll(now);
        client.poll(now).iter().any(|e| matches!(e, Event::Connected { .. }))
    }, 5);
    assert!(connected, "the client connected to the mapped port");
    reach.close();
    assert!(log.lock().unwrap().iter().any(|l| l.starts_with("DeletePortMapping")), "close deletes the mapping");
}

/// No gateway at all: STUN-only through the game socket; the code carries NEEDS_PUNCH, and the
/// joiner's own code (its STUN endpoint) lets the host punch to it.
#[test]
fn stun_only_host_punches_to_the_joiners_code() {
    let dead_ssdp = UdpSocket::bind("127.0.0.1:0").unwrap(); // nobody answers
    let stun_a = fake_stun(None);
    let tr = UdpTransport::bind("127.0.0.1:0").unwrap();
    let port = tr.local_addr().port();
    let mut host = Session::new_host(Config::default(), tr, MatchSettings::default(), 7);
    let silent_pmp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let cfg = ReachConfig {
        stun_servers: vec![stun_a.to_string()],
        ssdp: dead_ssdp.local_addr().unwrap().to_string(),
        ssdp_wait: Duration::from_millis(200),
        natpmp_gateway: Some(silent_pmp.local_addr().unwrap()),
        map: true,
    };
    let mut reach = HostReach::start(port, cfg.clone(), 3);
    let step_host = |host: &mut Session<UdpTransport>, reach: &mut HostReach, now: u64| -> Vec<Event> {
        let ev = host.poll(now);
        for (from, d) in host.take_stray() { reach.on_packet(from, &d); }
        let mut out = Vec::new();
        reach.poll(now, &mut |to, d| out.push((to, d.to_vec())));
        for (to, d) in out { host.send_raw(to, &d); }
        ev
    };
    assert!(pump_until(|now| { step_host(&mut host, &mut reach, now); reach.status().state != HostState::Probing }, 15));
    let st = reach.status();
    assert_eq!(st.state, HostState::StunOnly, "{st:?}");
    assert_eq!(st.public, Some(format!("127.0.0.1:{port}").parse().unwrap()));
    let code = st.code.clone().unwrap();
    assert_ne!(p2p::JoinCode::decode(&code).unwrap().flags & p2p::FLAG_NEEDS_PUNCH, 0);

    // The joiner: decodes the host's code, learns its own code by STUN on ITS game socket.
    let ctr = UdpTransport::bind("127.0.0.1:0").unwrap();
    let mut join = JoinReach::start(&code, &cfg, 9).unwrap();
    let mut client = Session::new_client(Config::default(), ctr, join.target());
    let mut my_code = None;
    let mut connected = false;
    assert!(pump_until(|now| {
        step_host(&mut host, &mut reach, now);
        if client.poll(now).iter().any(|e| matches!(e, Event::Connected { .. })) { connected = true; }
        for (from, d) in client.take_stray() { join.on_packet(from, &d); }
        let mut out = Vec::new();
        join.poll(now, &mut |to, d| out.push((to, d.to_vec())));
        for (to, d) in out { client.send_raw(to, &d); }
        my_code = join.status(now).my_code;
        my_code.is_some()
    }, 10));
    let s = join.status(10_000);
    assert!(s.host_needs_punch);
    assert!(!s.symmetric_both);
    // The host types the joiner's code → punches go out to the joiner's endpoint, and the session
    // connects (on loopback the path is open anyway; the punch state proves the datagrams flowed).
    let to = reach.expect_peer(my_code.as_ref().unwrap()).unwrap();
    assert_eq!(to, client.local_addr());
    let mut punched = false;
    assert!(pump_until(|now| {
        for e in step_host(&mut host, &mut reach, now) {
            if let Event::ClientJoined { .. } = e { reach.peer_seen(to); }
        }
        if client.poll(now).iter().any(|e| matches!(e, Event::Connected { .. })) { connected = true; }
        if client.take_stray().iter().any(|(from, d)| *from == host.local_addr() && p2p::punch::is_punch(d)) { punched = true; }
        // Either the joiner saw a punch, or the host already heard the joiner from that endpoint
        // (its same-network probes count) — both prove the path is open both ways.
        connected && (punched || reach.status().punches.iter().any(|(_, s)| *s == p2p::PunchState::Reached))
    }, 10), "punches reached the joiner and the session connected: {:?}", reach.status().punches);
}

#[test]
fn bad_codes_are_refused_with_a_reason() {
    let cfg = ReachConfig { map: false, stun_servers: vec![], ..Default::default() };
    assert!(matches!(JoinReach::start("not a code", &cfg, 1), Err(p2p::CodeError::Malformed)));
    let good = p2p::JoinCode::new("203.0.113.1:27420".parse().unwrap(), 0).encode();
    let mut bad: Vec<char> = good.chars().collect();
    bad[0] = if bad[0] == '0' { '1' } else { '0' };
    let bad: String = bad.into_iter().collect();
    assert!(matches!(JoinReach::start(&bad, &cfg, 1), Err(p2p::CodeError::Checksum)));
    assert!(JoinReach::start(&good, &cfg, 1).is_ok());
}
