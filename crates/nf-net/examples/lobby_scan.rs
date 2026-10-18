//! Scratch (not committed): print the LAN lobby list after a few seconds of scanning.
fn main() {
    let mut l = nf_net::discovery::LobbyList::open(nf_net::PROTOCOL_VERSION, nf_net::DEFAULT_BUILD_ID, 42);
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
