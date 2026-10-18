//! The optional internet lobby directory (700v4 item 5; docs/tools/listing-server.md) [S port design]:
//! a tiny UDP server (`nf-lobbyd`, the user's home server) that only LISTS lobbies — play stays P2P.
//!
//! - A host sends REGISTER from its GAME socket every `REGISTER_EVERY_MS` (`NFDR` + the
//!   `discovery::LobbyInfo` record). The server keys the lobby by the datagram's SOURCE address — the
//!   host's public game endpoint as the internet sees it (its NAT mapping) — so a joiner dials what
//!   the server saw, not what the host believes.
//! - A joiner sends LIST (`NFDL` + nonce + ms); the server answers ENTRIES datagrams (`NFDE` + the
//!   echoed 8 bytes + count + per entry: IPv4, port, record length, record), split to fit `MAX_PACKET`.
//! - Entries not refreshed for `EXPIRE_MS` drop.
//! The server's address is a game setting, default empty — never hard-coded (nothing registers or
//! lists unless the user sets it).

use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::discovery::LobbyInfo;

pub const REGISTER_MAGIC: [u8; 4] = *b"NFDR";
pub const LIST_MAGIC: [u8; 4] = *b"NFDL";
pub const ENTRIES_MAGIC: [u8; 4] = *b"NFDE";
pub const REGISTER_EVERY_MS: u64 = 5_000;
pub const EXPIRE_MS: u64 = 20_000;
/// The server's default UDP port (the setting may name another) [S].
pub const DEFAULT_PORT: u16 = 27500;
/// At most this many lobbies are kept (a home server; protects memory) [S].
pub const MAX_LOBBIES: usize = 256;

pub fn register_packet(info: &LobbyInfo) -> Vec<u8> {
    let mut rec = info.encode([0; 8]);
    rec.splice(0..4, REGISTER_MAGIC);
    rec
}

pub fn list_packet(nonce: u32, sent_ms: u32) -> [u8; 12] {
    let mut b = [0u8; 12];
    b[0..4].copy_from_slice(&LIST_MAGIC);
    b[4..8].copy_from_slice(&nonce.to_le_bytes());
    b[8..12].copy_from_slice(&sent_ms.to_le_bytes());
    b
}

/// The server's table.
#[derive(Default)]
pub struct Directory {
    lobbies: BTreeMap<SocketAddrV4, (LobbyInfo, u64)>,
}

impl Directory {
    /// One datagram in; the replies out (to `from`).
    pub fn handle(&mut self, from: SocketAddr, data: &[u8], now_ms: u64) -> Vec<Vec<u8>> {
        self.lobbies.retain(|_, (_, seen)| now_ms.saturating_sub(*seen) <= EXPIRE_MS);
        let SocketAddr::V4(from4) = from else { return Vec::new() };
        if data.len() > 4 && data[0..4] == REGISTER_MAGIC {
            let mut rec = data.to_vec();
            rec.splice(0..4, crate::discovery::LOBBY_MAGIC);
            if let Some((info, _)) = LobbyInfo::decode(&rec) {
                if self.lobbies.contains_key(&from4) || self.lobbies.len() < MAX_LOBBIES {
                    self.lobbies.insert(from4, (info, now_ms));
                }
            }
            return Vec::new();
        }
        if data.len() == 12 && data[0..4] == LIST_MAGIC {
            let echo: [u8; 8] = data[4..12].try_into().expect("12-byte list");
            return self.entries(echo);
        }
        Vec::new()
    }

    fn entries(&self, echo: [u8; 8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let start = || { let mut p = ENTRIES_MAGIC.to_vec(); p.extend_from_slice(&echo); p.push(0); p };
        let mut cur = start();
        for (addr, (info, _)) in &self.lobbies {
            let rec = info.encode([0; 8]);
            let need = 4 + 2 + 2 + rec.len();
            if cur.len() + need > crate::MAX_PACKET || cur[12] == u8::MAX {
                out.push(std::mem::replace(&mut cur, start()));
            }
            cur.extend_from_slice(&addr.ip().octets());
            cur.extend_from_slice(&addr.port().to_le_bytes());
            cur.extend_from_slice(&(rec.len() as u16).to_le_bytes());
            cur.extend_from_slice(&rec);
            cur[12] += 1;
        }
        out.push(cur);
        out
    }

    pub fn len(&self) -> usize {
        self.lobbies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lobbies.is_empty()
    }
}

/// Parse an ENTRIES datagram → (echo, [(public game endpoint, record)]).
pub fn parse_entries(d: &[u8]) -> Option<([u8; 8], Vec<(SocketAddr, LobbyInfo)>)> {
    if d.len() < 13 || d[0..4] != ENTRIES_MAGIC {
        return None;
    }
    let echo: [u8; 8] = d[4..12].try_into().ok()?;
    let n = d[12] as usize;
    let mut i = 13;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let ip = Ipv4Addr::new(*d.get(i)?, *d.get(i + 1)?, *d.get(i + 2)?, *d.get(i + 3)?);
        let port = u16::from_le_bytes([*d.get(i + 4)?, *d.get(i + 5)?]);
        let len = u16::from_le_bytes([*d.get(i + 6)?, *d.get(i + 7)?]) as usize;
        let (info, _) = LobbyInfo::decode(d.get(i + 8..i + 8 + len)?)?;
        v.push((SocketAddr::V4(SocketAddrV4::new(ip, port)), info));
        i += 8 + len;
    }
    Some((echo, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(name: &str) -> LobbyInfo {
        LobbyInfo { name: name.into(), host: "Bond".into(), level_code: 0x0700_0023, scenario: 1, players: 1, max_players: 4, in_progress: false, protocol: 2, build: 1, port: 27420 }
    }

    #[test]
    fn register_list_expire_and_split() {
        let mut d = Directory::default();
        let h1: SocketAddr = "203.0.113.5:40001".parse().unwrap();
        assert!(d.handle(h1, &register_packet(&info("A")), 0).is_empty());
        let r = d.handle("198.51.100.9:5555".parse().unwrap(), &list_packet(7, 100), 10);
        let (echo, rows) = parse_entries(&r[0]).unwrap();
        assert_eq!(echo[0..4], 7u32.to_le_bytes());
        assert_eq!(rows, vec![(h1, info("A"))], "keyed by the SOURCE address (the host's public mapping)");
        // Expiry: no refresh for EXPIRE_MS.
        let r = d.handle("198.51.100.9:5555".parse().unwrap(), &list_packet(7, 100), EXPIRE_MS + 1);
        assert!(parse_entries(&r[0]).unwrap().1.is_empty());
        // Many lobbies split across datagrams, each under MAX_PACKET, nothing lost.
        for k in 0..60u16 {
            d.handle(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, (k / 200) as u8, (k % 200) as u8 + 1), 27420)), &register_packet(&info(&"n".repeat(32))), 1000);
        }
        let r = d.handle("198.51.100.9:5555".parse().unwrap(), &list_packet(1, 0), 1001);
        assert!(r.len() > 1 && r.iter().all(|p| p.len() <= crate::MAX_PACKET));
        assert_eq!(r.iter().map(|p| parse_entries(p).unwrap().1.len()).sum::<usize>(), 60);
        assert!(d.handle(h1, b"junk", 0).is_empty());
    }
}
