//! NAT-PMP (RFC 6886) — the fallback port mapping when no UPnP IGD answers
//! (docs/systems/net-p2p.md §1 step 2). Blocking with the RFC's retry schedule, shortened.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;

pub const PORT: u16 = 5351;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PmpError {
    /// No reply after the retries.
    NoReply,
    /// The gateway answered with a non-zero result code (RFC 6886 §3.5).
    Result(u16),
    Io(String),
}

/// Opcode 0 request (§3.2).
pub fn external_request() -> [u8; 2] {
    [0, 0]
}

/// Opcode 1 (UDP) map request (§3.3). `lifetime_s` 0 with `external` 0 deletes the mapping.
pub fn map_request(internal: u16, external: u16, lifetime_s: u32) -> [u8; 12] {
    let mut b = [0u8; 12];
    b[1] = 1;
    b[4..6].copy_from_slice(&internal.to_be_bytes());
    b[6..8].copy_from_slice(&external.to_be_bytes());
    b[8..12].copy_from_slice(&lifetime_s.to_be_bytes());
    b
}

/// Parse an opcode-128 reply → the external address.
pub fn parse_external(r: &[u8]) -> Result<Ipv4Addr, PmpError> {
    if r.len() < 12 || r[0] != 0 || r[1] != 128 {
        return Err(PmpError::Io("not an external-address reply".into()));
    }
    let code = u16::from_be_bytes([r[2], r[3]]);
    if code != 0 {
        return Err(PmpError::Result(code));
    }
    Ok(Ipv4Addr::new(r[8], r[9], r[10], r[11]))
}

/// Parse an opcode-129 reply → (internal, external, lifetime).
pub fn parse_map(r: &[u8]) -> Result<(u16, u16, u32), PmpError> {
    if r.len() < 16 || r[0] != 0 || r[1] != 129 {
        return Err(PmpError::Io("not a UDP map reply".into()));
    }
    let code = u16::from_be_bytes([r[2], r[3]]);
    if code != 0 {
        return Err(PmpError::Result(code));
    }
    Ok((u16::from_be_bytes([r[8], r[9]]), u16::from_be_bytes([r[10], r[11]]), u32::from_be_bytes([r[12], r[13], r[14], r[15]])))
}

/// Send `req` to the gateway and wait for a reply with opcode `want`: 250 ms, doubling, 3 tries
/// (RFC 6886 §3.1 goes to 9 tries / 64 s; a menu cannot wait that long) [S].
fn ask(gw: SocketAddr, req: &[u8], want: u8) -> Result<Vec<u8>, PmpError> {
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| PmpError::Io(e.to_string()))?;
    let mut wait = Duration::from_millis(250);
    let mut buf = [0u8; 64];
    for _ in 0..3 {
        sock.send_to(req, gw).map_err(|e| PmpError::Io(e.to_string()))?;
        sock.set_read_timeout(Some(wait)).ok();
        let t0 = std::time::Instant::now();
        while t0.elapsed() < wait {
            match sock.recv_from(&mut buf) {
                Ok((n, from)) if from.ip() == gw.ip() && n >= 2 && buf[1] == want => return Ok(buf[..n].to_vec()),
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        wait *= 2;
    }
    Err(PmpError::NoReply)
}

/// Map UDP `internal` (asking for the same external port); returns the public endpoint and the
/// granted lifetime. `gw` is the gateway's NAT-PMP address (tests pass a loopback fake).
pub fn map_port(gw: SocketAddr, internal: u16, lifetime_s: u32) -> Result<(SocketAddrV4, u32), PmpError> {
    let ext_ip = parse_external(&ask(gw, &external_request(), 128)?)?;
    let (_, ext_port, life) = parse_map(&ask(gw, &map_request(internal, internal, lifetime_s), 129)?)?;
    Ok((SocketAddrV4::new(ext_ip, ext_port), life))
}

/// Remove the mapping (lifetime 0, external 0 — §3.4).
pub fn delete(gw: SocketAddr, internal: u16) -> Result<(), PmpError> {
    parse_map(&ask(gw, &map_request(internal, 0, 0), 129)?).map(|_| ())
}

/// The gateway guess when SSDP gave none: the LAN address's x.y.z.1 [S heuristic].
pub fn guess_gateway(local: Ipv4Addr) -> SocketAddr {
    let o = local.octets();
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(o[0], o[1], o[2], 1), PORT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_and_reply_layouts() {
        // RFC 6886 §3.3: version 0, opcode 1 (UDP), reserved, internal, suggested external, lifetime.
        assert_eq!(map_request(27420, 27420, 3600), [0, 1, 0, 0, 0x6B, 0x1C, 0x6B, 0x1C, 0, 0, 0x0E, 0x10]);
        let ext = [0, 128, 0, 0, 0, 0, 0, 9, 203, 0, 113, 5];
        assert_eq!(parse_external(&ext), Ok(Ipv4Addr::new(203, 0, 113, 5)));
        let refused = [0, 128, 0, 3, 0, 0, 0, 9, 0, 0, 0, 0];
        assert_eq!(parse_external(&refused), Err(PmpError::Result(3)));
        let map = [0, 129, 0, 0, 0, 0, 0, 9, 0x6B, 0x1C, 0x6B, 0x1D, 0, 0, 0x0E, 0x10];
        assert_eq!(parse_map(&map), Ok((27420, 27421, 3600)));
        assert_eq!(guess_gateway(Ipv4Addr::new(192, 168, 4, 37)), "192.168.4.1:5351".parse().unwrap());
    }
}
