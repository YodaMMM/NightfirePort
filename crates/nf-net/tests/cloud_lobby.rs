//! net-cloud-lobby (docs/systems/net-cloud-lobby.md): the game side against a local mock of the
//! Cloudflare lobby service — a plain `std::net::TcpListener` HTTP server speaking the same four
//! routes, so the real client path (the system curl, background threads, the JSON answers) runs
//! end to end on loopback. These pin the port's own client behaviour [S]; game behaviour is
//! untouched. The unchanged long-code path is the standing sister assertion:
//! `loopback_e2e.rs::net_e2e_loopback_live` stays green.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nf_net::cloud::{
    json_parse, CloudAdvert, CloudBrowse, CloudHost, CloudLookup, CloudLookupError, Json,
    CLOUD_REFRESH_MS,
};
use nf_net::directory::SHORT_CODE_ALPHABET;
use nf_net::session::{Config, Event, Session};
use nf_net::{MatchSettings, UdpTransport};

// ── the mock service (the worker's routes; expiry + stability, no rate limits — the worker's
// own unit tests cover those; here the client's behaviour is the subject) ────────────────────

struct Entry {
    code: String,
    ip: String,
    port: u16,
    version: String,
    name: String,
    map: u32,
    players: u8,
    public: bool,
    lan: String,
    seen: u64,
}

#[derive(Default)]
struct Svc {
    entries: BTreeMap<String, Entry>, // "ip:port" → entry
    codes: BTreeMap<String, String>,  // code → key
    now: u64,                         // the virtual clock the tests advance
    rng: u64,
}

const TTL_MS: u64 = 90_000;

impl Svc {
    fn next_code(&mut self) -> String {
        loop {
            let mut code = String::new();
            self.rng = self.rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let mut x = self.rng;
            for _ in 0..4 {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                code.push(SHORT_CODE_ALPHABET[(x >> 33) as usize % SHORT_CODE_ALPHABET.len()] as char);
            }
            if !self.codes.contains_key(&code) {
                return code;
            }
        }
    }

    fn drop_expired(&mut self) {
        let dead: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, e)| self.now.saturating_sub(e.seen) > TTL_MS)
            .map(|(k, _)| k.clone())
            .collect();
        for k in dead {
            if let Some(e) = self.entries.remove(&k) {
                self.codes.remove(&e.code);
            }
        }
    }

    fn handle(&mut self, method: &str, path: &str, query: &str, ip: &str, body: Option<&str>) -> (u16, String) {
        self.drop_expired();
        match (method, path) {
            ("POST", "/v1/host") => {
                let b = match body.and_then(json_parse) {
                    Some(Json::Obj(_)) => body.unwrap().to_string(),
                    _ => return (400, "{\"error\":\"bad JSON\"}".into()),
                };
                let get = |k: &str| json_parse(&b).and_then(|v| v.get(k).cloned());
                let (Some(Json::Str(version)), Some(Json::Str(name)), Some(Json::Num(port))) =
                    (get("version"), get("name"), get("port"))
                else {
                    return (400, "{\"error\":\"bad body\"}".into());
                };
                let map = get("map").and_then(|v| v.as_u32()).unwrap_or(0);
                let players = get("players").and_then(|v| v.as_u8()).unwrap_or(1);
                let public = get("public") != Some(Json::Bool(false));
                let lan = match get("lan") {
                    Some(Json::Str(s)) => s,
                    _ => String::new(),
                };
                let port = port as u64 as u16;
                let key = format!("{ip}:{port}");
                // net-cloud-code-stable: the mock mirrors the worker — the body may carry the code
                // the host already holds; a new key moves the same host's entry instead of minting
                // a new code. (4 alphabet symbols, case-insensitive; anything else is no ask.)
                let asked = match get("code") {
                    Some(Json::Str(s)) if s.len() == 4 => {
                        let up = s.to_uppercase();
                        if up.bytes().all(|b| SHORT_CODE_ALPHABET.contains(&b)) { up } else { String::new() }
                    }
                    _ => String::new(),
                };
                let code = match self.entries.get(&key) {
                    Some(e) => e.code.clone(),
                    None => {
                        if !asked.is_empty() {
                            if let Some(owner_key) = self.codes.get(&asked).cloned() {
                                if owner_key != key {
                                    if let Some(owner) = self.entries.get(&owner_key) {
                                        if owner.ip == ip {
                                            self.entries.remove(&owner_key);
                                            self.codes.remove(&asked);
                                        }
                                    }
                                }
                            }
                        }
                        if !asked.is_empty() && !self.codes.contains_key(&asked) {
                            self.codes.insert(asked.clone(), key.clone());
                            asked
                        } else {
                            let c = self.next_code();
                            self.codes.insert(c.clone(), key.clone());
                            c
                        }
                    }
                };
                self.entries.insert(
                    key,
                    Entry { code: code.clone(), ip: ip.to_string(), port, version, name, map, players, public, lan, seen: self.now },
                );
                (200, format!("{{\"code\":\"{code}\",\"public_addr\":\"{ip}:{port}\"}}"))
            }
            ("DELETE", "/v1/host") => {
                let mine: Vec<String> = self.entries.keys().filter(|k| k.starts_with(&format!("{ip}:"))).cloned().collect();
                for k in mine {
                    if let Some(e) = self.entries.remove(&k) {
                        self.codes.remove(&e.code);
                    }
                }
                (200, "{\"ok\":true}".into())
            }
            ("GET", "/v1/lobbies") => {
                let want = query.strip_prefix("version=").unwrap_or("");
                let rows: Vec<String> = self
                    .entries
                    .values()
                    .filter(|e| e.public && e.version == want)
                    .map(|e| format!("{{\"code\":\"{}\",\"name\":{:?},\"map\":{},\"scenario\":1,\"players\":{},\"max\":4}}", e.code, e.name, e.map, e.players))
                    .collect();
                (200, format!("{{\"lobbies\":[{}]}}", rows.join(",")))
            }
            ("GET", p) if p.starts_with("/v1/code/") => {
                let code = p["/v1/code/".len()..].to_uppercase();
                match self.codes.get(&code).and_then(|k| self.entries.get(k)) {
                    Some(e) => (
                        200,
                        format!(
                            "{{\"code\":\"{}\",\"public_addr\":\"{}:{}\",\"lan_addr\":\"{}\",\"name\":{:?}}}",
                            e.code, e.ip, e.port, e.lan, e.name
                        ),
                    ),
                    None => (404, "{\"error\":\"not found\"}".into()),
                }
            }
            _ => (404, "{\"error\":\"no such route\"}".into()),
        }
    }
}

/// The mock service on 127.0.0.1:0 — `url()` is the base the clients take, `advance()` moves its
/// virtual clock, `entry_name()` inspects what a register stored.
struct Service {
    url: String,
    svc: Arc<Mutex<Svc>>,
}

impl Service {
    fn start() -> Service {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock service");
        let port = listener.local_addr().unwrap().port();
        let svc = Arc::new(Mutex::new(Svc { rng: 0x1234_5678, ..Default::default() }));
        let svc2 = svc.clone();
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                let Ok(mut s) = conn else { continue };
                let svc = svc2.clone();
                std::thread::spawn(move || {
                    let _ = handle_conn(&mut s, &svc);
                });
            }
        });
        Service { url: format!("http://127.0.0.1:{port}"), svc }
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn advance(&self, ms: u64) {
        self.svc.lock().unwrap().now += ms;
    }

    fn entry_name(&self, code: &str) -> Option<String> {
        let svc = self.svc.lock().unwrap();
        svc.codes.get(code).and_then(|k| svc.entries.get(k)).map(|e| e.name.clone())
    }
}

fn handle_conn(s: &mut TcpStream, svc: &Arc<Mutex<Svc>>) -> std::io::Result<()> {
    s.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    // One request per connection (the client never pipelines; Connection: close).
    loop {
        let n = s.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let head_end = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4).unwrap_or(buf.len());
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let mut lines = head.lines();
    let first = lines.next().unwrap_or_default();
    let mut it = first.split_whitespace();
    let method = it.next().unwrap_or("").to_string();
    let target = it.next().unwrap_or("").to_string();
    let mut content_length = 0usize;
    for l in lines {
        if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = buf[head_end..].to_vec();
    while body.len() < content_length {
        let n = s.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target, String::new()),
    };
    let ip = s.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|_| "0.0.0.0".into());
    let body_str = String::from_utf8_lossy(&body).trim().to_string();
    let body_opt = if body_str.is_empty() { None } else { Some(body_str.as_str()) };
    let (status, json) = svc.lock().unwrap().handle(&method, &path, &query, &ip, body_opt);
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Error",
    };
    let resp = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", json.len(), json);
    let _ = s.write_all(resp.as_bytes());
    let _ = s.flush();
    Ok(())
}

// ── helpers ─────────────────────────────────────────────────────────────────────────────────

fn advert(name: &str, public: bool, port: u16, version: &str) -> CloudAdvert {
    CloudAdvert {
        version: version.into(),
        name: name.into(),
        map: 0x0700_0023,
        scenario: 1,
        players: 1,
        max_players: 4,
        public,
        lan: "192.168.1.50:27420".into(),
        port,
    }
}

/// Drives `step` until `done`, or fails after 10 s (the requests run on background threads).
fn wait_until(mut done: impl FnMut() -> bool) {
    let t0 = Instant::now();
    while !done() {
        assert!(t0.elapsed() < Duration::from_secs(10), "timed out waiting for the service");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn info(name: &str) -> nf_net::discovery::LobbyInfo {
    nf_net::discovery::LobbyInfo {
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

// ── the tests ────────────────────────────────────────────────────────────────────────────────

/// net-cloud-code-stable: the advert's dial port can change mid-session (the STUN-observed mapping
/// lands around the lobby → match hand-over) and the service keys entries by ip:port — so the same
/// CloudHost re-registers under the new port carrying its code, and the service must answer the
/// SAME code (it moves the entry), with the lookup resolving the new endpoint.
#[test]
fn the_code_survives_the_port_change_re_register() {
    let svc = Service::start();
    let mut h = CloudHost::new(svc.url());
    h.set_advert(advert("Bond's game", true, 27420, &nf_net::cloud::game_version()));
    wait_until(|| {
        h.poll(0);
        !h.busy()
    });
    h.poll(1);
    let code = h.code(1).expect("registered").to_string();

    // The STUN mapping is known now: the next period's POST names the new dial port. The client
    // sends the same advert through the same CloudHost — the body now also carries `code`.
    h.set_advert(advert("Bond's game", true, 51234, &nf_net::cloud::game_version()));
    wait_until(|| {
        h.poll(CLOUD_REFRESH_MS);
        !h.busy()
    });
    h.poll(CLOUD_REFRESH_MS + 1);
    assert_eq!(h.code(CLOUD_REFRESH_MS + 1), Some(code.as_str()), "the code survives the port change");

    // The joiner's lookup now answers the NEW endpoint (the entry moved, not doubled).
    let mut lk = CloudLookup::new(svc.url(), &code);
    wait_until(|| {
        lk.poll();
        lk.finished()
    });
    let f = lk.result().expect("finished").as_ref().expect("found").clone();
    assert_eq!(f.public.port(), 51234, "the service recorded the new dial port");
    assert_eq!(f.name, "Bond's game");

    // Sister: a second host (its own session, no code held) still gets its own, different code.
    let mut h2 = CloudHost::new(svc.url());
    h2.set_advert(advert("Other host", true, 27421, &nf_net::cloud::game_version()));
    wait_until(|| {
        h2.poll(CLOUD_REFRESH_MS);
        !h2.busy()
    });
    let _ = h2.poll(CLOUD_REFRESH_MS + 1);
    let other = h2.code(CLOUD_REFRESH_MS + 1).expect("the second host's code");
    assert_ne!(other, code, "two different hosts never share a code");
}

#[test]
fn host_registers_gets_a_code_and_the_refresh_keeps_it() {
    let svc = Service::start();
    let mut h = CloudHost::new(svc.url());
    assert!(h.code(0).is_none(), "nothing registered — no code (the long code shows)");
    h.set_advert(advert("Bond's game", true, 27420, &nf_net::cloud::game_version()));
    wait_until(|| {
        h.poll(0);
        !h.busy()
    });
    h.poll(1);
    let code = h.code(1).expect("the service answered with a code").to_string();
    assert_eq!(code.len(), 4, "4 characters: {code}");
    assert!(code.bytes().all(|b| SHORT_CODE_ALPHABET.contains(&b)), "unambiguous alphabet: {code}");
    assert_eq!(svc.entry_name(&code).as_deref(), Some("Bond's game"), "the body reached the service");
    // The refresh (the next period's POST) keeps the entry — and the code — stable.
    h.set_advert(advert("Bond's game", true, 27420, &nf_net::cloud::game_version()));
    wait_until(|| {
        h.poll(CLOUD_REFRESH_MS);
        !h.busy()
    });
    h.poll(CLOUD_REFRESH_MS + 1);
    assert_eq!(h.code(CLOUD_REFRESH_MS + 1), Some(code.as_str()), "the same while refreshed");
    // Sister: a different game version registers a different entry (the list filter's other side).
    let mut h2 = CloudHost::new(svc.url());
    h2.set_advert(advert("Old build", true, 27421, "7.1"));
    wait_until(|| {
        h2.poll(0);
        !h2.busy()
    });
    let _ = h2.poll(1);
    let other = h2.code(1).expect("the second host's code");
    assert_ne!(other, code, "codes are unique among the live entries");
}

#[test]
fn joiner_looks_a_code_up_and_a_real_loopback_session_connects() {
    let svc = Service::start();
    // The host: a real session on its game socket; the register names that socket's port, so the
    // service's recorded endpoint is exactly what a joiner must dial.
    let tr = UdpTransport::bind("127.0.0.1:0").unwrap();
    let mut h = Session::new_host(Config::default(), tr, MatchSettings::default(), 1);
    let port = h.local_addr().port();
    let mut ch = CloudHost::new(svc.url());
    ch.set_advert(advert("Internet game", true, port, &nf_net::cloud::game_version()));
    wait_until(|| {
        ch.poll(0);
        !ch.busy()
    });
    ch.poll(1);
    let code = ch.code(1).expect("registered").to_string();

    // The joiner: GET /v1/code/{CODE} through the service, then the real handshake on 127.0.0.1.
    let mut lk = CloudLookup::new(svc.url(), &code);
    wait_until(|| {
        lk.poll();
        lk.finished()
    });
    let f = lk.result().expect("finished").as_ref().expect("found").clone();
    assert_eq!(SocketAddr::from(f.public), h.local_addr(), "the endpoint the service recorded");
    assert_eq!(f.name, "Internet game");
    assert_eq!(f.lan.map(SocketAddr::from), Some(SocketAddr::from(([192, 168, 1, 50], 27420))), "the LAN endpoint rides along");

    let mut c = Session::new_client(Config::default(), UdpTransport::bind("127.0.0.1:0").unwrap(), SocketAddr::from(f.public));
    let mut ok = false;
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(5) {
        h.poll(h.local_addr().port() as u64 + t0.elapsed().as_millis() as u64);
        if c.poll(t0.elapsed().as_millis() as u64).iter().any(|e| matches!(e, Event::Connected { .. })) {
            ok = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(ok, "the joiner connected through the looked-up endpoint (the real handshake)");
}

#[test]
fn the_list_filters_by_version_and_the_public_flag() {
    let svc = Service::start();
    let mk = |name: &str, public: bool, port: u16, version: &str| {
        let mut h = CloudHost::new(svc.url());
        h.set_advert(advert(name, public, port, version));
        wait_until(|| {
            h.poll(0);
            !h.busy()
        });
        let _ = h.poll(1);
        h
    };
    let _public = mk("Public game", true, 27420, &nf_net::cloud::game_version());
    let _private = mk("Private game", false, 27421, &nf_net::cloud::game_version());
    let _old = mk("Old build", true, 27422, "7.1");

    let mut b = CloudBrowse::open(svc.url(), nf_net::cloud::game_version());
    wait_until(|| {
        b.poll(0);
        !b.rows().is_empty()
    });
    let rows = b.rows();
    assert_eq!(rows.len(), 1, "public + same version only: {rows:?}");
    assert_eq!(rows[0].name, "Public game");
    assert_eq!(rows[0].code.len(), 4);
    assert_eq!(rows[0].map, 0x0700_0023, "the arena's level code rides the row");
    assert_eq!(rows[0].max_players, 4);
    // The rows carry no address at all (a joiner resolves one only through the code).
    assert!(!format!("{rows:?}").contains("127.0.0.1"), "never an IP in the list: {rows:?}");
}

#[test]
fn expiry_drops_the_entry_and_the_lookup_turns_not_found() {
    let svc = Service::start();
    let mut h = CloudHost::new(svc.url());
    h.set_advert(advert("Doomed lobby", true, 27420, &nf_net::cloud::game_version()));
    wait_until(|| {
        h.poll(0);
        !h.busy()
    });
    let _ = h.poll(1);
    let code = h.code(1).expect("registered").to_string();

    // Nobody refreshes; 90 s later the entry is gone server-side.
    svc.advance(90_001);
    let mut b = CloudBrowse::open(svc.url(), nf_net::cloud::game_version());
    wait_until(|| {
        b.poll(0);
        !b.busy()
    });
    assert!(b.rows().is_empty(), "the expired lobby does not list");
    let mut lk = CloudLookup::new(svc.url(), &code);
    wait_until(|| {
        lk.poll();
        lk.finished()
    });
    assert_eq!(lk.result(), Some(&Err(CloudLookupError::NotFound)), "a dead entry is a clean not-found");
}

#[test]
fn an_unknown_code_is_a_clean_not_found() {
    let svc = Service::start();
    let mut lk = CloudLookup::new(svc.url(), "ZZ9V");
    wait_until(|| {
        lk.poll();
        lk.finished()
    });
    assert_eq!(lk.result(), Some(&Err(CloudLookupError::NotFound)));
    assert_eq!(lk.code(), "ZZ9V", "the note getter reports the code as typed (canonical)");
}

#[test]
fn an_unreachable_service_falls_back_and_fails_cleanly() {
    // A port with no listener: curl fails at once, the client treats it as unreachable.
    let dead = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = dead.local_addr().unwrap().port();
    drop(dead);
    let url = format!("http://127.0.0.1:{port}");

    // The host: no code ever arrives — the long code shows (today's fallback).
    let mut h = CloudHost::new(url.clone());
    h.set_advert(advert("Offline lobby", true, 27420, &nf_net::cloud::game_version()));
    wait_until(|| {
        h.poll(0);
        !h.busy()
    });
    let _ = h.poll(1);
    assert!(h.code(1).is_none(), "the service is unreachable — no short code (the long code shows)");
    assert!(h.service_url().ends_with(&port.to_string()));

    // The joiner's lookup says unreachable (not a hang, not "not found").
    let mut lk = CloudLookup::new(url, "AB2C");
    wait_until(|| {
        lk.poll();
        lk.finished()
    });
    assert_eq!(lk.result(), Some(&Err(CloudLookupError::Unreachable)));
}
