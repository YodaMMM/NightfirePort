//! UDP hole punching (docs/systems/net-p2p.md §3) [S, RFC 5128 §3.3]: the host sends PUNCH
//! datagrams to a joiner's public endpoint so its own NAT lets the joiner's HELLOs in.

use std::net::SocketAddr;

/// The punch packet type: `4E 46 50 00` + 8-byte nonce. The session decoder has no type 0x50,
/// so a punch is a stray datagram on both sides (never a session packet).
pub const PT_PUNCH: u8 = 0x50;
pub const PUNCH_EVERY_MS: u64 = 100;
pub const PUNCH_FOR_MS: u64 = 15_000;

pub fn punch_packet(nonce: [u8; 8]) -> [u8; 12] {
    let mut b = [0u8; 12];
    b[0] = 0x4E;
    b[1] = 0x46;
    b[2] = PT_PUNCH;
    b[4..12].copy_from_slice(&nonce);
    b
}

pub fn is_punch(data: &[u8]) -> bool {
    data.len() == 12 && data[0] == 0x4E && data[1] == 0x46 && data[2] == PT_PUNCH
}

/// Same-network shortcut (docs/systems/net-p2p.md §3a): a joiner whose public IP equals the
/// code's (it sits behind the host's own NAT, where many routers do not hairpin) broadcasts a
/// PROBE on the code's port; the host answers PROBE_ACK from its game socket, and the joiner dials
/// that LAN address instead. Same 12-byte layout as a punch.
pub const PT_PROBE: u8 = 0x51;
pub const PT_PROBE_ACK: u8 = 0x52;

pub fn probe_packet(ty: u8, nonce: [u8; 8]) -> [u8; 12] {
    let mut b = punch_packet(nonce);
    b[2] = ty;
    b
}

/// (type, nonce) of a probe / probe-ack.
pub fn parse_probe(data: &[u8]) -> Option<(u8, [u8; 8])> {
    if data.len() == 12 && data[0] == 0x4E && data[1] == 0x46 && matches!(data[2], PT_PROBE | PT_PROBE_ACK) {
        let mut n = [0u8; 8];
        n.copy_from_slice(&data[4..12]);
        return Some((data[2], n));
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunchState {
    Punching,
    /// A datagram from the target arrived (its NAT and ours are open both ways).
    Reached,
    /// `PUNCH_FOR_MS` passed with nothing from the target.
    Expired,
}

/// One target's punch schedule.
#[derive(Clone, Debug)]
pub struct Puncher {
    pub target: SocketAddr,
    nonce: [u8; 8],
    start_ms: Option<u64>,
    next_ms: u64,
    pub state: PunchState,
}

impl Puncher {
    pub fn new(target: SocketAddr, nonce: [u8; 8]) -> Puncher {
        Puncher { target, nonce, start_ms: None, next_ms: 0, state: PunchState::Punching }
    }

    /// The datagram to send now, if any.
    pub fn poll(&mut self, now_ms: u64) -> Option<(SocketAddr, [u8; 12])> {
        if self.state != PunchState::Punching {
            return None;
        }
        let start = *self.start_ms.get_or_insert(now_ms);
        if now_ms.saturating_sub(start) >= PUNCH_FOR_MS {
            self.state = PunchState::Expired;
            return None;
        }
        if now_ms < self.next_ms {
            return None;
        }
        self.next_ms = now_ms + PUNCH_EVERY_MS;
        Some((self.target, punch_packet(self.nonce)))
    }

    /// Any datagram from the target (a punch or a session packet) proves the path.
    pub fn on_datagram(&mut self, from: SocketAddr) -> bool {
        if from == self.target && self.state != PunchState::Reached {
            self.state = PunchState::Reached;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_and_states() {
        let t: SocketAddr = "198.51.100.4:40000".parse().unwrap();
        let mut p = Puncher::new(t, [1; 8]);
        assert!(p.poll(1000).is_some());
        assert!(p.poll(1050).is_none());
        assert!(p.poll(1100).is_some());
        assert!(!p.on_datagram("198.51.100.4:40001".parse().unwrap()), "another port is not the target");
        assert!(p.on_datagram(t));
        assert_eq!(p.state, PunchState::Reached);
        assert!(p.poll(1200).is_none(), "a reached target is not punched again");
        let mut q = Puncher::new(t, [2; 8]);
        q.poll(0);
        assert!(q.poll(PUNCH_FOR_MS).is_none());
        assert_eq!(q.state, PunchState::Expired);
        assert!(is_punch(&punch_packet([3; 8])));
        assert!(crate::proto::decode(&punch_packet([3; 8])).is_none(), "the session ignores a punch");
    }
}
