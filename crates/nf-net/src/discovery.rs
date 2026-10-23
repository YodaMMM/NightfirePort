//! LAN lobby discovery (docs/systems/net-lobby-list.md) [S port design — the PS2 game has no
//! online mode]: a joiner broadcasts a QUERY to the common game ports; every host answers on its
//! own game socket (a stray datagram for the session — `Session::take_stray`) with a LOBBY record;
//! the joiner keeps a list with ping, status and expiry. No extra socket on the host, so any number
//! of hosts can run on one PC. The same `LobbyInfo` record is what an internet listing server
//! (700v4 item 5) would hand out.

use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

/// `NFLQ` + u32 nonce + u32 sent-ms (echoed back for the ping).
pub const QUERY_MAGIC: [u8; 4] = *b"NFLQ";
/// `NFLB` + the echoed nonce/ms + the record.
pub const LOBBY_MAGIC: [u8; 4] = *b"NFLB";
/// The ports a query is broadcast to: the default game port and the next three [S].
pub const SCAN_PORTS: [u16; 4] = [27420, 27421, 27422, 27423];
/// Query cadence and expiry (ms).
pub const QUERY_EVERY_MS: u64 = 1500;
pub const EXPIRE_MS: u64 = 5000;

/// What a host advertises.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LobbyInfo {
    /// The lobby's name (≤ 32 bytes on the wire).
    pub name: String,
    /// The host's codename (≤ 20 bytes).
    pub host: String,
    pub level_code: u32,
    pub scenario: u32,
    pub players: u8,
    pub max_players: u8,
    /// false = in the lobby, true = the match is running (join through 700v5).
    pub in_progress: bool,
    pub protocol: u16,
    pub build: u32,
    /// The game port (the joiner dials sender-IP:port).
    pub port: u16,
}

fn put_str(out: &mut Vec<u8>, s: &str, max: usize) {
    let b: Vec<u8> = s.bytes().take(max).collect();
    out.push(b.len() as u8);
    out.extend_from_slice(&b);
}

fn get_str(d: &[u8], i: &mut usize) -> Option<String> {
    let n = *d.get(*i)? as usize;
    let s = d.get(*i + 1..*i + 1 + n)?;
    *i += 1 + n;
    Some(String::from_utf8_lossy(s).to_string())
}

impl LobbyInfo {
    pub fn encode(&self, echo: [u8; 8]) -> Vec<u8> {
        let mut o = Vec::with_capacity(96);
        o.extend_from_slice(&LOBBY_MAGIC);
        o.extend_from_slice(&echo);
        o.extend_from_slice(&self.protocol.to_le_bytes());
        o.extend_from_slice(&self.build.to_le_bytes());
        o.extend_from_slice(&self.port.to_le_bytes());
        o.extend_from_slice(&self.level_code.to_le_bytes());
        o.extend_from_slice(&self.scenario.to_le_bytes());
        o.push(self.players);
        o.push(self.max_players);
        o.push(self.in_progress as u8);
        put_str(&mut o, &self.name, 32);
        put_str(&mut o, &self.host, 20);
        o
    }

    /// (record, echoed nonce/ms).
    pub fn decode(d: &[u8]) -> Option<(LobbyInfo, [u8; 8])> {
        if d.get(0..4)? != LOBBY_MAGIC {
            return None;
        }
        let mut echo = [0u8; 8];
        echo.copy_from_slice(d.get(4..12)?);
        let u16_at = |i: usize| Some(u16::from_le_bytes([*d.get(i)?, *d.get(i + 1)?]));
        let u32_at = |i: usize| Some(u32::from_le_bytes(d.get(i..i + 4)?.try_into().ok()?));
        let mut i = 31;
        let info = LobbyInfo {
            protocol: u16_at(12)?,
            build: u32_at(14)?,
            port: u16_at(18)?,
            level_code: u32_at(20)?,
            scenario: u32_at(24)?,
            players: *d.get(28)?,
            max_players: *d.get(29)?,
            in_progress: *d.get(30)? != 0,
            name: get_str(d, &mut i)?,
            host: get_str(d, &mut i)?,
        };
        Some((info, echo))
    }
}

pub fn query_packet(nonce: u32, sent_ms: u32) -> [u8; 12] {
    let mut b = [0u8; 12];
    b[0..4].copy_from_slice(&QUERY_MAGIC);
    b[4..8].copy_from_slice(&nonce.to_le_bytes());
    b[8..12].copy_from_slice(&sent_ms.to_le_bytes());
    b
}

/// Host side: the reply to a stray datagram, if it is a QUERY.
pub fn answer(data: &[u8], info: &LobbyInfo) -> Option<Vec<u8>> {
    if data.len() != 12 || data[0..4] != QUERY_MAGIC {
        return None;
    }
    let mut echo = [0u8; 8];
    echo.copy_from_slice(&data[4..12]);
    Some(info.encode(echo))
}

/// One listed lobby.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LobbyEntry {
    pub addr: SocketAddr,
    pub info: LobbyInfo,
    pub ping_ms: u32,
    pub last_seen_ms: u64,
    /// Same protocol + build as this machine — the HELLO gate would accept it.
    pub joinable: bool,
}

/// Joiner side: the list, poll-driven over its own UDP socket.
pub struct LobbyList {
    sock: Option<UdpSocket>,
    targets: Vec<SocketAddr>,
    nonce: u32,
    next_ms: u64,
    entries: BTreeMap<SocketAddr, LobbyEntry>,
    protocol: u16,
    build: u32,
    /// The internet directory (`directory`, a self-hosted server) — None unless the setting names one.
    directory: Option<SocketAddr>,
}

impl LobbyList {
    /// Bind a discovery socket on the LAN interface (Windows sends a 0.0.0.0 socket's broadcast
    /// out whatever adapter routes first — the Hyper-V/WSL lesson of net-p2p) and aim at the
    /// subnet broadcast, the limited broadcast and this PC, on every scan port.
    pub fn open(protocol: u16, build: u32, seed: u32) -> LobbyList {
        let lan = crate::p2p::local_ipv4();
        let sock = lan
            .and_then(|ip| UdpSocket::bind(SocketAddrV4::new(ip, 0)).ok())
            .or_else(|| UdpSocket::bind("0.0.0.0:0").ok());
        if let Some(s) = &sock {
            let _ = s.set_broadcast(true);
            let _ = s.set_nonblocking(true);
        }
        let mut bcast = vec![Ipv4Addr::BROADCAST, Ipv4Addr::LOCALHOST];
        if let Some(ip) = lan {
            let o = ip.octets();
            bcast.insert(0, Ipv4Addr::new(o[0], o[1], o[2], 255));
            bcast.push(ip);
        }
        let targets = bcast.iter().flat_map(|ip| SCAN_PORTS.iter().map(move |&p| SocketAddr::V4(SocketAddrV4::new(*ip, p)))).collect();
        LobbyList::with_socket(sock, targets, protocol, build, seed)
    }

    /// Tests: an explicit socket and target list.
    pub fn with_socket(sock: Option<UdpSocket>, targets: Vec<SocketAddr>, protocol: u16, build: u32, seed: u32) -> LobbyList {
        LobbyList { sock, targets, nonce: seed | 1, next_ms: 0, entries: BTreeMap::new(), protocol, build, directory: None }
    }

    /// Also list the lobbies the internet directory knows (the joiner dials the public endpoint
    /// the directory saw).
    pub fn set_directory(&mut self, server: Option<SocketAddr>) {
        self.directory = server;
    }

    fn add(&mut self, addr: SocketAddr, info: LobbyInfo, echo: [u8; 8], now_ms: u64) {
        if u32::from_le_bytes([echo[0], echo[1], echo[2], echo[3]]) != self.nonce {
            return;
        }
        let sent = u32::from_le_bytes([echo[4], echo[5], echo[6], echo[7]]);
        let joinable = info.protocol == self.protocol && info.build == self.build;
        let ping = (now_ms as u32).wrapping_sub(sent);
        let ping = self.entries.get(&addr).filter(|e| e.last_seen_ms + QUERY_EVERY_MS > now_ms).map_or(ping, |e| e.ping_ms.min(ping));
        self.entries.insert(addr, LobbyEntry { addr, info, ping_ms: ping, last_seen_ms: now_ms, joinable });
    }

    /// Send the periodic query, drain replies, expire stale rows.
    pub fn poll(&mut self, now_ms: u64) {
        let Some(sock) = self.sock.as_ref() else { return };
        if now_ms >= self.next_ms {
            self.next_ms = now_ms + QUERY_EVERY_MS;
            let q = query_packet(self.nonce, now_ms as u32);
            for t in &self.targets {
                let _ = sock.send_to(&q, t);
            }
            if let Some(d) = self.directory {
                let _ = sock.send_to(&crate::directory::list_packet(self.nonce, now_ms as u32), d);
            }
        }
        let mut buf = [0u8; 1500];
        let mut got: Vec<(SocketAddr, LobbyInfo, [u8; 8])> = Vec::new();
        while let Ok((n, from)) = sock.recv_from(&mut buf) {
            if Some(from) == self.directory {
                if let Some((echo, rows)) = crate::directory::parse_entries(&buf[..n]) {
                    got.extend(rows.into_iter().map(|(a, i)| (a, i, echo)));
                }
                continue;
            }
            // One host answers once per target that reached it (broadcast + localhost + its LAN IP):
            // the row is keyed by its game address and keeps the best ping.
            let Some((info, echo)) = LobbyInfo::decode(&buf[..n]) else { continue };
            got.push((SocketAddr::new(from.ip(), info.port), info, echo));
        }
        for (a, i, e) in got {
            self.add(a, i, e, now_ms);
        }
        self.entries.retain(|_, e| now_ms.saturating_sub(e.last_seen_ms) <= EXPIRE_MS);
    }

    /// The rows, lobbies first then in-progress, each by name.
    pub fn rows(&self) -> Vec<LobbyEntry> {
        let mut v: Vec<LobbyEntry> = self.entries.values().cloned().collect();
        v.sort_by(|a, b| (a.info.in_progress, &a.info.name, a.addr).cmp(&(b.info.in_progress, &b.info.name, b.addr)));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_round_trip_and_rejections() {
        let info = LobbyInfo { name: "Bond's lobby".into(), host: "Bond".into(), level_code: 0x0700_0023, scenario: 1, players: 2, max_players: 4, in_progress: false, protocol: 2, build: 1, port: 27420 };
        let b = info.encode([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(LobbyInfo::decode(&b), Some((info.clone(), [1, 2, 3, 4, 5, 6, 7, 8])));
        assert_eq!(LobbyInfo::decode(&b[..20]), None, "truncated");
        let mut bad = b.clone();
        bad[0] = b'X';
        assert_eq!(LobbyInfo::decode(&bad), None);
        let long = LobbyInfo { name: "x".repeat(80), ..info };
        assert_eq!(LobbyInfo::decode(&long.encode([0; 8])).unwrap().0.name.len(), 32, "names cap at 32 bytes");
        assert!(answer(&query_packet(5, 9), &long).is_some());
        assert!(answer(b"NFLQ", &long).is_none());
        assert!(crate::proto::decode(&query_packet(5, 9)).is_none(), "a query is never a session packet");
    }
}
