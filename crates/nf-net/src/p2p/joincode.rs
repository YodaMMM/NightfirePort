//! Join codes (docs/systems/net-p2p.md §2) [S port design]: a public IPv4 endpoint + protocol
//! version + flags + CRC-16, as 16 Crockford base32 symbols `XXXX-XXXX-XXXX-XXXX`.

use std::net::{Ipv4Addr, SocketAddrV4};

/// Flag bit 0: the code's owner has no port mapping — the other side must punch.
pub const FLAG_NEEDS_PUNCH: u8 = 1;
/// Flag bit 1: the code's owner is behind an endpoint-dependent ("symmetric") NAT.
pub const FLAG_SYMMETRIC: u8 = 2;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// A decoded join code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinCode {
    pub addr: SocketAddrV4,
    pub version: u8,
    pub flags: u8,
}

/// Why a typed code or address was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeError {
    /// Not 16 base32 symbols and not `a.b.c.d:port`.
    Malformed,
    /// The checksum does not match (a typo).
    Checksum,
    /// Made by a build speaking another protocol version.
    Version { theirs: u8, ours: u8 },
}

/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF, no reflection, no final xor).
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}

impl JoinCode {
    pub fn new(addr: SocketAddrV4, flags: u8) -> JoinCode {
        JoinCode { addr, version: crate::PROTOCOL_VERSION as u8, flags }
    }

    fn bytes(&self) -> [u8; 10] {
        let mut b = [0u8; 10];
        b[0..4].copy_from_slice(&self.addr.ip().octets());
        b[4..6].copy_from_slice(&self.addr.port().to_be_bytes());
        b[6] = self.version;
        b[7] = self.flags;
        let c = crc16(&b[0..8]);
        b[8..10].copy_from_slice(&c.to_be_bytes());
        b
    }

    /// The display form, `XXXX-XXXX-XXXX-XXXX`.
    pub fn encode(&self) -> String {
        let b = self.bytes();
        let mut bits: u128 = 0;
        for &x in &b {
            bits = (bits << 8) | x as u128;
        }
        let mut out = String::with_capacity(19);
        for i in 0..16 {
            if i > 0 && i % 4 == 0 {
                out.push('-');
            }
            let sym = ((bits >> (75 - 5 * i)) & 31) as usize;
            out.push(ALPHABET[sym] as char);
        }
        out
    }

    /// Decode a code (any case; dashes/spaces ignored; I/L → 1, O → 0). Checks the CRC and
    /// the protocol version.
    pub fn decode(s: &str) -> Result<JoinCode, CodeError> {
        let mut bits: u128 = 0;
        let mut n = 0;
        for c in s.chars() {
            if c == '-' || c.is_whitespace() {
                continue;
            }
            let c = match c.to_ascii_uppercase() {
                'I' | 'L' => '1',
                'O' => '0',
                c => c,
            };
            let v = ALPHABET.iter().position(|&a| a as char == c).ok_or(CodeError::Malformed)?;
            bits = (bits << 5) | v as u128;
            n += 1;
            if n > 16 {
                return Err(CodeError::Malformed);
            }
        }
        if n != 16 {
            return Err(CodeError::Malformed);
        }
        let mut b = [0u8; 10];
        for (i, x) in b.iter_mut().enumerate() {
            *x = (bits >> (72 - 8 * i)) as u8;
        }
        if crc16(&b[0..8]).to_be_bytes() != [b[8], b[9]] {
            return Err(CodeError::Checksum);
        }
        let ours = crate::PROTOCOL_VERSION as u8;
        if b[6] != ours {
            return Err(CodeError::Version { theirs: b[6], ours });
        }
        Ok(JoinCode {
            addr: SocketAddrV4::new(Ipv4Addr::new(b[0], b[1], b[2], b[3]), u16::from_be_bytes([b[4], b[5]])),
            version: b[6],
            flags: b[7],
        })
    }
}

/// What the join page typed: a join code or a raw `a.b.c.d:port`. A raw address carries no
/// flags (it is assumed reachable).
pub fn parse_target(s: &str) -> Result<JoinCode, CodeError> {
    let t = s.trim();
    if let Ok(a) = t.parse::<SocketAddrV4>() {
        if a.port() == 0 {
            return Err(CodeError::Malformed);
        }
        return Ok(JoinCode::new(a, 0));
    }
    JoinCode::decode(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc16_ccitt_false_check_value() {
        // The catalogue check value of CRC-16/CCITT-FALSE over "123456789" is 0x29B1.
        assert_eq!(crc16(b"123456789"), 0x29B1);
    }

    #[test]
    fn round_trip_and_rejections() {
        let jc = JoinCode::new("203.0.113.7:27420".parse().unwrap(), FLAG_NEEDS_PUNCH);
        let s = jc.encode();
        assert_eq!(s.len(), 19);
        assert_eq!(JoinCode::decode(&s), Ok(jc));
        assert_eq!(JoinCode::decode(&s.to_lowercase().replace('-', " ")), Ok(jc));
        // One symbol changed → checksum (the sister: the comparison can fail).
        let mut bad: Vec<char> = s.chars().collect();
        bad[2] = if bad[2] == 'A' { 'B' } else { 'A' };
        let bad: String = bad.into_iter().collect();
        assert_eq!(JoinCode::decode(&bad), Err(CodeError::Checksum));
        // Another protocol version.
        let other = JoinCode { version: jc.version.wrapping_add(1), ..jc };
        assert_eq!(JoinCode::decode(&other.encode()), Err(CodeError::Version { theirs: jc.version.wrapping_add(1), ours: jc.version }));
        assert_eq!(JoinCode::decode("ABC"), Err(CodeError::Malformed));
        assert_eq!(JoinCode::decode("UUUU-UUUU-UUUU-UUUU"), Err(CodeError::Malformed));
    }

    #[test]
    fn raw_address_or_code() {
        assert_eq!(parse_target(" 192.168.1.20:27420 ").unwrap().addr, "192.168.1.20:27420".parse().unwrap());
        assert_eq!(parse_target("192.168.1.20:0"), Err(CodeError::Malformed));
        let jc = JoinCode::new("198.51.100.2:5000".parse().unwrap(), 0);
        assert_eq!(parse_target(&jc.encode()), Ok(jc));
    }
}
