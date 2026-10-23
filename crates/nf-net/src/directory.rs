//! The optional internet lobby directory (docs/tools/listing-server.md) [S port design]:
//! a tiny UDP server (`nf-lobbyd`, self-hosted) that only LISTS lobbies — play stays P2P.
//!
//! - A host sends REGISTER from its GAME socket every `REGISTER_EVERY_MS` (`NFDR` + the
//!   `discovery::LobbyInfo` record). The server keys the lobby by the datagram's SOURCE address — the
//!   host's public game endpoint as the internet sees it (its NAT mapping) — so a joiner dials what
//!   the server saw, not what the host believes.
//! - A joiner sends LIST (`NFDL` + nonce + ms); the server answers ENTRIES datagrams (`NFDE` + the
//!   echoed 8 bytes + count + per entry: IPv4, port, record length, record), split to fit `MAX_PACKET`.
//! - Entries not refreshed for `EXPIRE_MS` drop.
//! - net-short-codes (docs/systems/net-short-codes.md) [S]: a v2 REGISTER (the old packet + a
//!   versioned tail) is answered with `NFDA` carrying a 4-character code from an unambiguous
//!   alphabet (no 0/O, no 1/I/L/U); `NFDS` looks a code up to the host's public endpoint as the
//!   server saw it (plus the LAN endpoint the host reported, so the same-network path works).
//!   Versioned: a request with an unknown version is answered `NFDN` "unsupported"; a
//!   pre-versioning server answers nothing and clients fall back cleanly. LOOKUP is rate limited
//!   per source address (codes are brute-forceable). The long self-contained join code
//!   (p2p/joincode.rs) stays the fallback whenever no server answered.
//! The server's address is a game setting, default empty — never hard-coded (nothing registers or
//! lists unless the player sets it).

use std::collections::BTreeMap;
use std::hash::{BuildHasher, Hasher};
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

// ── net-short-codes (docs/systems/net-short-codes.md) [S] ────────────────────────────
/// The version of the short-code messages; a request with another value is answered
/// "unsupported" (docs/tools/listing-server.md redeploys pick this up) [S].
pub const DIR_PROTOCOL: u8 = 1;
/// `NFDA`: REGISTER ack — the assigned code (server → host).
pub const REGISTER_ACK_MAGIC: [u8; 4] = *b"NFDA";
/// `NFDS`: LOOKUP a short code (joiner → server).
pub const LOOKUP_MAGIC: [u8; 4] = *b"NFDS";
/// `NFDT`: LOOKUP found (server → joiner).
pub const FOUND_MAGIC: [u8; 4] = *b"NFDT";
/// `NFDN`: a negative answer (server → host/joiner).
pub const NACK_MAGIC: [u8; 4] = *b"NFDN";
/// The short code's unambiguous alphabet: digits 2-9 + the Crockford letters minus I/L/O/U
/// — 30 symbols, 30^4 = 810 000 codes; no 0/O and no 1/I/L to confuse (U stays out for V,
/// as in the long code's Crockford base32) [S].
pub const SHORT_CODE_ALPHABET: &[u8; 30] = b"23456789ABCDEFGHJKMNPQRSTVWXYZ";
/// The short code's length in symbols.
pub const SHORT_CODE_LEN: usize = 4;
/// The host shows the server's code while it keeps answering; three missed register periods
/// (the entry has 20 s to live) drop it and the long code shows again [S].
pub const CODE_FRESH_MS: u64 = 15_000;
/// LOOKUP cadence and tries: ~4 s of silence means "no answer" (an old server, or down) [S].
pub const LOOKUP_EVERY_MS: u64 = 400;
pub const LOOKUP_TRIES: u32 = 10;
/// The LOOKUP rate limit per source address: 30 per 10 s sliding window [S] — 810 k codes are
/// brute-forceable, so lookups stay tight; a joiner types one code.
pub const LOOKUP_WINDOW_MS: u64 = 10_000;
pub const LOOKUP_WINDOW_MAX: u32 = 30;

/// Why a short-code request was refused (the `NFDN` reason byte; `NoAnswer` is client-side
/// only — the server never answered in time) [S].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NackReason {
    NotFound = 0,
    RateLimited = 1,
    Unsupported = 2,
    NoAnswer = 255,
}

impl NackReason {
    fn from_byte(b: u8) -> NackReason {
        match b {
            0 => NackReason::NotFound,
            1 => NackReason::RateLimited,
            _ => NackReason::Unsupported,
        }
    }
}

/// The short code typed: uppercase, dashes and whitespace ignored; exactly [`SHORT_CODE_LEN`]
/// symbols of [`SHORT_CODE_ALPHABET`] (so I/L/O/U/0/1 cannot be a short code — they would be
/// ambiguous). `None` = not a short code (the caller falls back to the long-code parse) [S].
pub fn normalize_short_code(s: &str) -> Option<[u8; SHORT_CODE_LEN]> {
    let stripped: Vec<char> = s.chars().filter(|c| *c != '-' && !c.is_whitespace()).map(|c| c.to_ascii_uppercase()).collect();
    if stripped.len() != SHORT_CODE_LEN {
        return None;
    }
    let mut out = [0u8; SHORT_CODE_LEN];
    for (i, c) in stripped.iter().enumerate() {
        let at = SHORT_CODE_ALPHABET.iter().position(|&a| a as char == *c)?;
        out[i] = SHORT_CODE_ALPHABET[at];
    }
    Some(out)
}

/// The code as it displays: four bare symbols (a port choice) [S].
pub fn short_code_string(code: &[u8; SHORT_CODE_LEN]) -> String {
    code.iter().map(|&b| b as char).collect()
}

/// Where a REGISTER's `LobbyInfo` record ends on the wire, walking the layout (magic 4 + echo 8
/// + the fixed 19 + two length-prefixed strings). The v2 tail detection needs this instead of
/// the decoded lengths: a truncated multi-byte name lossy-decodes shorter [S].
fn lobby_record_wire_len(d: &[u8]) -> Option<usize> {
    if d.len() < 31 {
        return None;
    }
    let name_len = d[31] as usize;
    let host_at = 32 + name_len;
    let host_len = *d.get(host_at)? as usize;
    let end = host_at + 1 + host_len;
    (d.len() >= end).then_some(end)
}

pub fn register_packet(info: &LobbyInfo) -> Vec<u8> {
    let mut rec = info.encode([0; 8]);
    rec.splice(0..4, REGISTER_MAGIC);
    rec
}

/// The v2 REGISTER: the old packet (an old server reads the record unchanged and ignores the
/// tail) + `u8 DIR_PROTOCOL` + `u32 nonce` + the host's LAN endpoint (ip 0 = unknown) [S].
pub fn register_packet_v2(info: &LobbyInfo, nonce: u32, lan: Option<SocketAddrV4>) -> Vec<u8> {
    let mut p = register_packet(info);
    p.push(DIR_PROTOCOL);
    p.extend_from_slice(&nonce.to_le_bytes());
    match lan {
        Some(a) => {
            p.extend_from_slice(&a.ip().octets());
            p.extend_from_slice(&a.port().to_le_bytes());
        }
        None => p.extend_from_slice(&[0; 6]),
    }
    p
}

/// The host's short-code bookkeeping: the REGISTER carries this period's nonce; the `NFDA`
/// for it stores the code, shown while fresh (three register periods) [S].
pub struct ShortCodeHost {
    nonce: u32,
    code: Option<[u8; SHORT_CODE_LEN]>,
    fresh_until: u64,
}

impl ShortCodeHost {
    pub fn new(seed: u32) -> ShortCodeHost {
        ShortCodeHost { nonce: seed | 1, code: None, fresh_until: 0 }
    }

    /// The packet for one register period (the LAN endpoint the joiner on our network needs).
    pub fn register_packet(&self, info: &LobbyInfo, lan: Option<SocketAddrV4>) -> Vec<u8> {
        register_packet_v2(info, self.nonce, lan)
    }

    /// A stray datagram; true when it was our answer and the code was stored.
    pub fn on_packet(&mut self, d: &[u8], now_ms: u64) -> bool {
        // `NFDA` + `u32 nonce` + the 4 code symbols.
        if d.len() != 12 || d[0..4] != REGISTER_ACK_MAGIC {
            return false;
        }
        let nonce = u32::from_le_bytes([d[4], d[5], d[6], d[7]]);
        if nonce != self.nonce {
            return false;
        }
        // A fresh period: the next REGISTER asks again (a server restart hands a new code).
        self.nonce = self.nonce.wrapping_add(0x9E37_79B9);
        let mut code = [0u8; SHORT_CODE_LEN];
        code.copy_from_slice(&d[8..12]);
        self.code = Some(code);
        self.fresh_until = now_ms.saturating_add(CODE_FRESH_MS);
        true
    }

    /// The code while fresh (`None` → the long code shows).
    pub fn code(&self, now_ms: u64) -> Option<String> {
        (now_ms <= self.fresh_until).then(|| self.code.as_ref().map(|c| short_code_string(c))).flatten()
    }
}

/// The joiner's short-code lookup: `NFDS` to the player's lobby server every [`LOOKUP_EVERY_MS`],
/// [`LOOKUP_TRIES`] times; the answer is the host's public endpoint as the server saw it (plus the
/// host's LAN endpoint and its `LobbyInfo` record) — the joiner then dials it exactly as a long
/// code does. Silence after the tries is `NackReason::NoAnswer` (an old server answers nothing) [S].
pub struct ShortLookup {
    server: SocketAddr,
    code: [u8; SHORT_CODE_LEN],
    nonce: u32,
    next_ms: u64,
    tries: u32,
    answer: Option<Result<LookupFound, NackReason>>,
}

/// What a successful LOOKUP found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LookupFound {
    /// The host's public game endpoint (the datagram source the server registered).
    pub public: SocketAddrV4,
    /// The host's LAN endpoint as the host reported it (the same-network path) — None when the
    /// host did not report one.
    pub lan: Option<SocketAddrV4>,
    /// The host's lobby record (name, level, players…).
    pub info: LobbyInfo,
}

impl ShortLookup {
    pub fn new(server: SocketAddr, code: [u8; SHORT_CODE_LEN], seed: u32) -> ShortLookup {
        ShortLookup { server, code, nonce: seed | 1, next_ms: 0, tries: LOOKUP_TRIES, answer: None }
    }

    /// The code this lookup asks for (as sent, canonical form).
    pub fn code(&self) -> String {
        short_code_string(&self.code)
    }

    pub fn finished(&self) -> bool {
        self.answer.is_some()
    }

    /// The final answer, once [`Self::finished`] — `Err(NoAnswer)` = the server never answered.
    pub fn result(&self) -> Option<&Result<LookupFound, NackReason>> {
        self.answer.as_ref()
    }

    /// Advance; `send` transmits from the game socket (`Session::send_raw`).
    pub fn poll(&mut self, now_ms: u64, send: &mut dyn FnMut(SocketAddr, &[u8])) {
        if self.answer.is_some() {
            return;
        }
        if now_ms >= self.next_ms {
            self.next_ms = now_ms + LOOKUP_EVERY_MS;
            self.tries = self.tries.saturating_sub(1);
            let mut p = LOOKUP_MAGIC.to_vec();
            p.push(DIR_PROTOCOL);
            p.extend_from_slice(&self.nonce.to_le_bytes());
            p.extend_from_slice(&self.code);
            send(self.server, &p);
            if self.tries == 0 {
                self.answer = Some(Err(NackReason::NoAnswer));
            }
        }
    }

    /// A stray datagram; true when it was our answer (found or a clean negative).
    pub fn on_packet(&mut self, from: SocketAddr, d: &[u8]) -> bool {
        if self.answer.is_some() || from != self.server {
            return false;
        }
        if let Some((n, f)) = parse_lookup_found(d) {
            if n == self.nonce {
                self.answer = Some(Ok(f));
                return true;
            }
        }
        if d.len() == 9 && d[0..4] == NACK_MAGIC && u32::from_le_bytes(d[4..8].try_into().unwrap_or([0; 4])) == self.nonce {
            self.answer = Some(Err(NackReason::from_byte(d[8])));
            return true;
        }
        false
    }
}

/// Parse an `NFDT` FOUND datagram → (nonce, what was found).
pub fn parse_lookup_found(d: &[u8]) -> Option<(u32, LookupFound)> {
    if d.len() < 25 || d[0..4] != FOUND_MAGIC {
        return None;
    }
    let nonce = u32::from_le_bytes(d[4..8].try_into().ok()?);
    let ip = Ipv4Addr::new(d[8], d[9], d[10], d[11]);
    let port = u16::from_le_bytes([d[12], d[13]]);
    let lan_ip = Ipv4Addr::new(d[14], d[15], d[16], d[17]);
    let lan_port = u16::from_le_bytes([d[18], d[19]]);
    let (info, _) = LobbyInfo::decode(d.get(20..)?)?;
    Some((nonce, LookupFound {
        public: SocketAddrV4::new(ip, port),
        lan: (lan_ip != Ipv4Addr::UNSPECIFIED && lan_port != 0).then(|| SocketAddrV4::new(lan_ip, lan_port)),
        info,
    }))
}

pub fn list_packet(nonce: u32, sent_ms: u32) -> [u8; 12] {
    let mut b = [0u8; 12];
    b[0..4].copy_from_slice(&LIST_MAGIC);
    b[4..8].copy_from_slice(&nonce.to_le_bytes());
    b[8..12].copy_from_slice(&sent_ms.to_le_bytes());
    b
}

/// One live lobby: the record, the last refresh, the assigned short code and the host's LAN
/// endpoint (net-short-codes) [S].
struct DirectoryEntry {
    info: LobbyInfo,
    seen_ms: u64,
    code: [u8; SHORT_CODE_LEN],
    lan: Option<SocketAddrV4>,
}

/// The server's table.
pub struct Directory {
    lobbies: BTreeMap<SocketAddrV4, DirectoryEntry>,
    /// LOOKUP timestamps per source address, for the per-source rate limit [S].
    lookups: BTreeMap<SocketAddrV4, Vec<u64>>,
    /// xorshift64* state for the code draws (seeded from the clock + `RandomState`; the server
    /// is not the Sim, wall-clock entropy is fine here) [S].
    rng: u64,
}

impl Default for Directory {
    fn default() -> Directory {
        let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0x2545_F491_4F6C_DD1D, |d| d.as_nanos() as u64);
        seed ^= std::collections::hash_map::RandomState::new().build_hasher().finish();
        if seed == 0 {
            seed = 0x2545_F491_4F6C_DD1D;
        }
        Directory { lobbies: BTreeMap::new(), lookups: BTreeMap::new(), rng: seed }
    }
}

impl Directory {
    fn next_rand(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    /// A code unique among the live entries [S].
    fn assign_code(&mut self) -> [u8; SHORT_CODE_LEN] {
        loop {
            let mut code = [0u8; SHORT_CODE_LEN];
            for c in code.iter_mut() {
                *c = SHORT_CODE_ALPHABET[(self.next_rand() % SHORT_CODE_ALPHABET.len() as u64) as usize];
            }
            if !self.lobbies.values().any(|e| e.code == code) {
                return code;
            }
        }
    }

    fn nack(reason: NackReason, nonce: u32) -> Vec<u8> {
        let mut p = NACK_MAGIC.to_vec();
        p.extend_from_slice(&nonce.to_le_bytes());
        p.push(reason as u8);
        p
    }
}

impl Directory {
    /// One datagram in; the replies out (to `from`).
    pub fn handle(&mut self, from: SocketAddr, data: &[u8], now_ms: u64) -> Vec<Vec<u8>> {
        self.lobbies.retain(|_, e| now_ms.saturating_sub(e.seen_ms) <= EXPIRE_MS);
        self.lookups.retain(|_, v| !v.is_empty());
        let SocketAddr::V4(from4) = from else { return Vec::new() };
        if data.len() > 4 && data[0..4] == REGISTER_MAGIC {
            let mut rec = data.to_vec();
            rec.splice(0..4, crate::discovery::LOBBY_MAGIC);
            if let Some((info, _)) = LobbyInfo::decode(&rec) {
                let listed = self.lobbies.contains_key(&from4) || self.lobbies.len() < MAX_LOBBIES;
                if !listed {
                    return Vec::new();
                }
                // net-short-codes: the v2 tail (version + nonce + LAN endpoint) rides AFTER the
                // record; an old server never looks there and an old host sends no tail [S].
                let tail = lobby_record_wire_len(data)
                    .filter(|end| data.len() >= end + 11 && data[*end] == DIR_PROTOCOL)
                    .map(|end| {
                        let n = u32::from_le_bytes(data[end + 1..end + 5].try_into().expect("tail nonce"));
                        let lan = {
                            let ip = Ipv4Addr::new(data[end + 5], data[end + 6], data[end + 7], data[end + 8]);
                            let port = u16::from_le_bytes([data[end + 9], data[end + 10]]);
                            (ip != Ipv4Addr::UNSPECIFIED && port != 0).then(|| SocketAddrV4::new(ip, port))
                        };
                        (n, lan)
                    });
                let code = match self.lobbies.get_mut(&from4) {
                    // Refreshing: the entry — and its code — stays stable [S].
                    Some(e) => {
                        e.info = info;
                        e.seen_ms = now_ms;
                        if let Some((_, lan)) = &tail {
                            e.lan = *lan;
                        }
                        e.code
                    }
                    None => {
                        let code = self.assign_code();
                        self.lobbies.insert(from4, DirectoryEntry { info, seen_ms: now_ms, code, lan: tail.as_ref().map_or(None, |(_, l)| *l) });
                        code
                    }
                };
                if let Some((nonce, _)) = tail {
                    let mut p = REGISTER_ACK_MAGIC.to_vec();
                    p.extend_from_slice(&nonce.to_le_bytes());
                    p.extend_from_slice(&code);
                    return vec![p];
                }
            }
            return Vec::new();
        }
        if data.len() == 12 && data[0..4] == LIST_MAGIC {
            let echo: [u8; 8] = data[4..12].try_into().expect("12-byte list");
            return self.entries(echo);
        }
        if data.len() == 13 && data[0..4] == LOOKUP_MAGIC {
            let nonce = u32::from_le_bytes(data[5..9].try_into().expect("13-byte lookup"));
            if data[4] != DIR_PROTOCOL {
                return vec![Self::nack(NackReason::Unsupported, nonce)];
            }
            // The per-source rate limit (codes are brute-forceable) [S].
            let window = self.lookups.entry(from4).or_default();
            window.retain(|&t| now_ms.saturating_sub(t) <= LOOKUP_WINDOW_MS);
            if window.len() as u32 >= LOOKUP_WINDOW_MAX {
                return vec![Self::nack(NackReason::RateLimited, nonce)];
            }
            window.push(now_ms);
            // `NFDS` + ver(1) + nonce(4) + code(4): the code sits at data[9..13].
            let mut code = [0u8; SHORT_CODE_LEN];
            code.copy_from_slice(&data[9..13]);
            let code = normalize_short_code(&String::from_utf8_lossy(&code).to_string()).map_or(code, |c| c);
            if let Some((host_addr, e)) = self.lobbies.iter().find(|(_, e)| e.code == code) {
                let mut p = FOUND_MAGIC.to_vec();
                p.extend_from_slice(&nonce.to_le_bytes());
                // The host's endpoint is the entry's KEY — the REGISTER source the server saw —
                // never the lookup sender's [S].
                p.extend_from_slice(&host_addr.ip().octets());
                p.extend_from_slice(&host_addr.port().to_le_bytes());
                match e.lan {
                    Some(l) => {
                        p.extend_from_slice(&l.ip().octets());
                        p.extend_from_slice(&l.port().to_le_bytes());
                    }
                    None => p.extend_from_slice(&[0; 6]),
                }
                p.extend_from_slice(&e.info.encode([0; 8]));
                return vec![p];
            }
            return vec![Self::nack(NackReason::NotFound, nonce)];
        }
        Vec::new()
    }

    fn entries(&self, echo: [u8; 8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let start = || { let mut p = ENTRIES_MAGIC.to_vec(); p.extend_from_slice(&echo); p.push(0); p };
        let mut cur = start();
        for (addr, e) in &self.lobbies {
            let info = &e.info;
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

