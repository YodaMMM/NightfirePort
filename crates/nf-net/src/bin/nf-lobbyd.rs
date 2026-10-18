//! `nf-lobbyd` — the NightfirePort lobby directory for the user's home server
//! (docs/tools/listing-server.md). It only lists lobbies; play stays peer-to-peer.
//! Usage: nf-lobbyd [bind address, default 0.0.0.0:27500]

use std::net::UdpSocket;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let bind = std::env::args().nth(1).unwrap_or_else(|| format!("0.0.0.0:{}", nf_net::directory::DEFAULT_PORT));
    let sock = UdpSocket::bind(&bind)?;
    eprintln!("nf-lobbyd: listening on udp {bind}");
    let t0 = Instant::now();
    let mut dir = nf_net::directory::Directory::default();
    let mut buf = [0u8; 2048];
    let mut last_count = usize::MAX;
    loop {
        let (n, from) = match sock.recv_from(&mut buf) {
            Ok(v) => v,
            // Windows reports an ICMP port-unreachable from an earlier reply as a recv error.
            Err(_) => continue,
        };
        for reply in dir.handle(from, &buf[..n], t0.elapsed().as_millis() as u64) {
            let _ = sock.send_to(&reply, from);
        }
        if dir.len() != last_count {
            last_count = dir.len();
            eprintln!("nf-lobbyd: {last_count} lobby(ies) listed");
        }
    }
}
