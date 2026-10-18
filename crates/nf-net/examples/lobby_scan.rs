//! Diagnostic: print the lobby list (LAN, plus the directory when NF_LOBBY_SERVER is set) after 4 s of scanning.
fn main() {
    let mut l = nf_net::discovery::LobbyList::open(nf_net::PROTOCOL_VERSION, nf_net::DEFAULT_BUILD_ID, 42);
    if let Ok(s) = std::env::var("NF_LOBBY_SERVER") {
        use std::net::ToSocketAddrs;
        let s = if s.contains(':') { s } else { format!("{s}:{}", nf_net::directory::DEFAULT_PORT) };
        l.set_directory(s.to_socket_addrs().ok().and_then(|mut a| a.find(|a| a.is_ipv4())));
    }
    let t0 = std::time::Instant::now();
    while t0.elapsed().as_secs() < 4 {
        l.poll(t0.elapsed().as_millis() as u64);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    for r in l.rows() {
        println!("{} {:?} ping {} joinable {}", r.addr, r.info, r.ping_ms, r.joinable);
    }
    println!("{} rows", l.rows().len());
}
