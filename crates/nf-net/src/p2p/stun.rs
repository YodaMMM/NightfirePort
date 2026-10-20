//! STUN Binding (RFC 5389) — just enough to learn this socket's public endpoint
//! (docs/systems/net-p2p.md §1 step 3). Pure encode/decode plus a poll-driven client that sends
//! through any callback (the game socket via `Session::send_raw`) and is fed the replies.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// RFC 5389 §6: the magic cookie.
pub const MAGIC_COOKIE: u32 = 0x2112_A442;
const BINDING_REQUEST: u16 = 0x0001;
const BINDING_SUCCESS: u16 = 0x0101;
const ATTR_MAPPED_ADDRESS: u16 = 0x0001;
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;
const ATTR_FINGERPRINT: u16 = 0x8028;

/// The public STUN servers tried when the setting is left at its default [S].
pub const DEFAULT_SERVERS: &[&str] = &["stun.l.google.com:19302", "stun1.l.google.com:19302", "stun.cloudflare.com:3478"];

/// A Binding request with this transaction id (20 bytes, no attributes).
pub fn binding_request(txid: [u8; 12]) -> [u8; 20] {
    let mut b = [0u8; 20];
    b[0..2].copy_from_slice(&BINDING_REQUEST.to_be_bytes());
    // length 0
    b[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
    b[8..20].copy_from_slice(&txid);
    b
}

/// True when `data` looks like a STUN message (RFC 5389 §6: top two bits 0, the cookie at 4).
pub fn is_stun(data: &[u8]) -> bool {
    data.len() >= 20 && data[0] & 0xC0 == 0 && data[4..8] == MAGIC_COOKIE.to_be_bytes()
}

/// CRC-32 (IEEE, reflected 0xEDB88320) — the FINGERPRINT attribute's (RFC 5389 §15.5).
pub fn crc32(data: &[u8]) -> u32 {
    let mut c: u32 = 0xFFFF_FFFF;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    !c
}

/// A parsed Binding success response.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingResponse {
    pub txid: [u8; 12],
    pub mapped: SocketAddrV4,
    /// Some(ok) when a FINGERPRINT attribute was present.
    pub fingerprint_ok: Option<bool>,
}

/// Parse a Binding success response carrying an IPv4 (XOR-)MAPPED-ADDRESS. None for anything
/// else (an error response, IPv6 only, truncation).
pub fn parse_binding_response(data: &[u8]) -> Option<BindingResponse> {
    if !is_stun(data) || u16::from_be_bytes([data[0], data[1]]) != BINDING_SUCCESS {
        return None;
    }
    let len = u16::from_be_bytes([data[2], data[3]]) as usize;
    if data.len() < 20 + len {
        return None;
    }
    let mut txid = [0u8; 12];
    txid.copy_from_slice(&data[8..20]);
    let mut xor = None;
    let mut plain = None;
    let mut fingerprint_ok = None;
    let mut i = 20;
    while i + 4 <= 20 + len {
        let ty = u16::from_be_bytes([data[i], data[i + 1]]);
        let alen = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        let v = data.get(i + 4..i + 4 + alen)?;
        match ty {
            ATTR_XOR_MAPPED_ADDRESS | ATTR_MAPPED_ADDRESS if alen >= 8 && v[1] == 0x01 => {
                let mut port = u16::from_be_bytes([v[2], v[3]]);
                let mut ip = u32::from_be_bytes([v[4], v[5], v[6], v[7]]);
                if ty == ATTR_XOR_MAPPED_ADDRESS {
                    port ^= (MAGIC_COOKIE >> 16) as u16;
                    ip ^= MAGIC_COOKIE;
                    xor = Some(SocketAddrV4::new(Ipv4Addr::from(ip), port));
                } else {
                    plain = Some(SocketAddrV4::new(Ipv4Addr::from(ip), port));
                }
            }
            ATTR_FINGERPRINT if alen == 4 => {
                let want = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                // §15.5: CRC-32 of the message up to (excluding) this attribute, XOR 0x5354554e,
                // with the header length already covering the fingerprint.
                fingerprint_ok = Some(crc32(&data[..i]) ^ 0x5354_554E == want);
            }
            _ => {}
        }
        i += 4 + (alen + 3) / 4 * 4;
    }
    Some(BindingResponse { txid, mapped: xor.or(plain)?, fingerprint_ok })
}

/// One server's query state.
#[derive(Clone, Debug)]
struct Query {
    server: SocketAddr,
    txid: [u8; 12],
    sent: u32,
    next_ms: u64,
    result: Option<SocketAddrV4>,
}

/// The poll-driven STUN client: `poll(now)` returns the datagrams to send; feed every received
/// datagram to `on_packet`. Each server gets `TRIES` requests `RETRY_MS` apart.
#[derive(Clone, Debug)]
pub struct StunClient {
    queries: Vec<Query>,
}

pub const TRIES: u32 = 3;
pub const RETRY_MS: u64 = 500;

impl StunClient {
    /// `seed` makes the transaction ids (any per-run value; not security relevant here).
    pub fn new(servers: &[SocketAddr], seed: u64) -> StunClient {
        let mut x = seed ^ 0x9E37_79B9_7F4A_7C15;
        let queries = servers
            .iter()
            .filter(|s| s.is_ipv4())
            .map(|&server| {
                let mut txid = [0u8; 12];
                for b in txid.iter_mut() {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    *b = x as u8;
                }
                Query { server, txid, sent: 0, next_ms: 0, result: None }
            })
            .collect();
        StunClient { queries }
    }

    pub fn poll(&mut self, now_ms: u64) -> Vec<(SocketAddr, [u8; 20])> {
        let mut out = Vec::new();
        for q in &mut self.queries {
            if q.result.is_none() && q.sent < TRIES && now_ms >= q.next_ms {
                q.sent += 1;
                q.next_ms = now_ms + RETRY_MS;
                out.push((q.server, binding_request(q.txid)));
            }
        }
        out
    }

    /// Returns true when the datagram was a reply to one of our requests.
    pub fn on_packet(&mut self, from: SocketAddr, data: &[u8]) -> bool {
        let Some(r) = parse_binding_response(data) else { return false };
        if r.fingerprint_ok == Some(false) {
            return false;
        }
        for q in &mut self.queries {
            if q.txid == r.txid && q.server == from {
                q.result = Some(r.mapped);
                return true;
            }
        }
        false
    }

    /// Every query answered or out of tries (and the last try's wait elapsed).
    pub fn done(&self, now_ms: u64) -> bool {
        self.queries.iter().all(|q| q.result.is_some() || (q.sent >= TRIES && now_ms >= q.next_ms))
    }

    /// The mapped endpoints, one per answering server.
    pub fn results(&self) -> Vec<SocketAddrV4> {
        self.queries.iter().filter_map(|q| q.result).collect()
    }

    /// The public endpoint (the first answer).
    pub fn public(&self) -> Option<SocketAddrV4> {
        self.results().first().copied()
    }

    /// Two servers saw different mapped endpoints for the same socket: endpoint-dependent
    /// mapping (RFC 4787 §4.1) — a punch to a third party's endpoint will not be predictable.
    pub fn symmetric(&self) -> bool {
        let r = self.results();
        r.windows(2).any(|w| w[0] != w[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 5769 §2.2 "Sample IPv4 Response" (mapped 192.0.2.1:32853, SOFTWARE "test vector ",
    /// MESSAGE-INTEGRITY, FINGERPRINT). The FINGERPRINT check proves the vector was copied right.
    const RFC5769_IPV4_RESPONSE: [u8; 80] = [
        0x01, 0x01, 0x00, 0x3c, 0x21, 0x12, 0xa4, 0x42, 0xb7, 0xe7, 0xa7, 0x01, 0xbc, 0x34, 0xd6, 0x86, 0xfa, 0x87, 0xdf,
        0xae, 0x80, 0x22, 0x00, 0x0b, 0x74, 0x65, 0x73, 0x74, 0x20, 0x76, 0x65, 0x63, 0x74, 0x6f, 0x72, 0x20, 0x00, 0x20,
        0x00, 0x08, 0x00, 0x01, 0xa1, 0x47, 0xe1, 0x12, 0xa6, 0x43, 0x00, 0x08, 0x00, 0x14, 0x2b, 0x91, 0xf5, 0x99, 0xfd,
        0x9e, 0x90, 0xc3, 0x8c, 0x74, 0x89, 0xf9, 0x2a, 0xf9, 0xba, 0x53, 0xf0, 0x6b, 0xe7, 0xd7, 0x80, 0x28, 0x00, 0x04,
        0xc0, 0x7d, 0x4c, 0x96,
    ];

    #[test]
    fn rfc5769_ipv4_sample_response() {
        let r = parse_binding_response(&RFC5769_IPV4_RESPONSE).expect("parses");
        assert_eq!(r.mapped, "192.0.2.1:32853".parse().unwrap());
        assert_eq!(r.txid, [0xb7, 0xe7, 0xa7, 0x01, 0xbc, 0x34, 0xd6, 0x86, 0xfa, 0x87, 0xdf, 0xae]);
        assert_eq!(r.fingerprint_ok, Some(true));
        // Sister: one flipped byte breaks the fingerprint.
        let mut bad = RFC5769_IPV4_RESPONSE;
        bad[30] ^= 1;
        assert_eq!(parse_binding_response(&bad).unwrap().fingerprint_ok, Some(false));
    }

    #[test]
    fn request_layout_and_client_flow() {
        let req = binding_request([7; 12]);
        assert_eq!(&req[0..8], &[0x00, 0x01, 0x00, 0x00, 0x21, 0x12, 0xA4, 0x42]);
        assert!(is_stun(&req));
        assert!(!is_stun(b"NF\x01\x00........................"));
        let a: SocketAddr = "198.51.100.1:3478".parse().unwrap();
        let b: SocketAddr = "198.51.100.2:3478".parse().unwrap();
        let mut c = StunClient::new(&[a, b], 1);
        let sends = c.poll(0);
        assert_eq!(sends.len(), 2);
        assert!(c.poll(100).is_empty());
        assert_eq!(c.poll(500).len(), 2);
        // A canned reply for server a: XOR-MAPPED 203.0.113.9:40000, txid of a's request.
        let reply = |txid: &[u8], port: u16| {
            let mut m = vec![0x01, 0x01, 0x00, 0x0c, 0x21, 0x12, 0xa4, 0x42];
            m.extend_from_slice(txid);
            m.extend_from_slice(&[0x00, 0x20, 0x00, 0x08, 0x00, 0x01]);
            m.extend_from_slice(&(port ^ 0x2112).to_be_bytes());
            m.extend_from_slice(&(u32::from(Ipv4Addr::new(203, 0, 113, 9)) ^ MAGIC_COOKIE).to_be_bytes());
            m
        };
        let (sa, ra) = sends[0];
        let (_, rb) = sends[1];
        assert!(!c.on_packet(b, &reply(&ra[8..20], 40000)), "a reply from the wrong server is ignored");
        assert!(c.on_packet(sa, &reply(&ra[8..20], 40000)));
        assert_eq!(c.public(), Some("203.0.113.9:40000".parse().unwrap()));
        assert!(!c.symmetric());
        assert!(c.on_packet(b, &reply(&rb[8..20], 40001)));
        assert!(c.symmetric(), "different mapped ports per server = endpoint-dependent mapping");
        assert!(c.done(600));
    }
}
