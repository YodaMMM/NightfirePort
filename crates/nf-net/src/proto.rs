//! Wire protocol: packet types, layouts, encode/decode.
//!
//! Every packet starts with a 4-byte header: `MAGIC` u16 (`0x4E46`), packet
//! type u8, reserved u8 (0). All integers little-endian. No packet exceeds
//! [`MAX_PACKET`] bytes; anything longer, with a bad magic or an unknown type
//! is rejected by [`decode`]. Layouts are tagged in `docs/systems/net-lockstep.md`;
//! the protocol itself is a port feature [S] (the PS2 game has no network mode).
//!
//! | type | name | payload |
//! |------|------|---------|
//! | 1 | HELLO (client→host) | proto u16, build u32, display codename 32B |
//! | 2 | WELCOME (host→client) | proto u16, build u32, host codename 32B, slot u8, input_delay u8, hash_every u8, rsv u8, seed u32, settings (334 B, protocol 4) |
//! | 3 | REFUSE (host→client) | reason u8 (1 version, 2 build, 3 started, 4 full), rsv 3B |
//! | 4 | INPUT | base_tick u32, agent_mask u16, count u8, ack_tick u32, ack_bits u32, count*popcount(mask)*24B inputs |
//! | 5 | HASH | tick u32, hash u64, prev_tick u32, prev_hash u64 |
//! | 6 | KEEPALIVE | seq u32 |
//! | 7 | LEAVE | slot u8, reason u8, rsv 2B |
//! | 8 | SLOTMAP | human_slots bitmask u16, rsv 2B |
//! | 9 | GO (host→client) | first_tick u32 |
//! | 10 | DATA (app tunnel) | len u16 + bytes — opaque app-to-app blob (the lobby's choices); the session never inspects it |
//! | 11 | JOIN_BEGIN (host→late joiner) | first_tick u32, snapshot_tick u32, snapshot_hash u64 — the late-join handover head: the input log covers `first_tick..snapshot_tick` and the replay must hash to `snapshot_hash` at `snapshot_tick` |
//! | 12 | JOIN_LOG (host→late joiner) | first_tick u32, tick_count u8, chunk_index u16, chunk_count u16, tick_count × (16 × 24 B inputs) — one chunk of the recorded run log |
//! | 13 | JOIN_ACK (late joiner→host) | chunk_index u16 — all chunks ≤ chunk_index are held |
//! | 14 | JOIN_READY (late joiner→host) | — the replay verified (hash matched) and the joiner is caught up; the host picks the join tick |
//! | 15 | JOIN_SLOT (host→all) | slot u8, join_tick u32 — the slot becomes human from join_tick (its inputs start there; before it the slot rides NEUTRAL) |
//!
//! INPUT `agent_mask` bit i = slot i has an input in each tick run; inputs
//! are stored per tick in ascending slot order. The ack fields acknowledge
//! ticks received from that peer (see session module): `ack_tick` = highest
//! tick T with all ticks in `[start..=T]` received; bit i of `ack_bits` =
//! tick `T+1+i` received.

use crate::input::{Input, INPUT_WIRE_SIZE};
use crate::settings::{MatchSettings, SETTINGS_WIRE_SIZE};
use crate::{MAGIC, MAX_AGENTS, MAX_PACKET, PROTOCOL_VERSION};

pub const PT_HELLO: u8 = 1;
pub const PT_WELCOME: u8 = 2;
pub const PT_REFUSE: u8 = 3;
pub const PT_INPUT: u8 = 4;
pub const PT_HASH: u8 = 5;
pub const PT_KEEPALIVE: u8 = 6;
pub const PT_LEAVE: u8 = 7;
pub const PT_SLOTMAP: u8 = 8;
pub const PT_GO: u8 = 9;
pub const PT_DATA: u8 = 10;
pub const PT_JOIN_BEGIN: u8 = 11;
pub const PT_JOIN_LOG: u8 = 12;
pub const PT_JOIN_ACK: u8 = 13;
pub const PT_JOIN_READY: u8 = 14;
pub const PT_JOIN_SLOT: u8 = 15;

/// Ticks one JOIN_LOG chunk carries (16 agents × 24 B × 3 = 1152 B — fits
/// [`MAX_PACKET`] with header room; protocol ≤ 3 carried 12 ticks of 4
/// agents, the same 1152 B [S mp-16-slots]).
pub const JOIN_LOG_CHUNK_TICKS: usize = 3;

/// codename field width on the wire (nul-padded).
pub const CODENAME_WIRE: usize = 32;
/// How many ticks one INPUT packet may carry client→host (24B each; a
/// client's mask holds its own slot only, so 32 ticks × 24 B + the 15-byte
/// header stays well under [`MAX_PACKET`]).
pub const MAX_INPUT_COUNT_C2H: usize = 32;
/// How many ticks one INPUT packet may carry host→client. The relay mask
/// holds every agent but the receiver's (15 of 16 [S mp-16-slots]):
/// 3 ticks × 15 × 24 B + the 15-byte header = 1095 ≤ [`MAX_PACKET`].
/// (Protocol ≤ 3: 8 ticks of 3 agents.)
pub const MAX_INPUT_COUNT_H2C: usize = 3;

/// A decoded packet. Sizes are exact; `inputs` holds
/// `count * popcount(agent_mask)` entries in tick-major, slot-ascending order.
#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    Hello {
        proto: u16,
        build: u32,
        codename: String,
    },
    Welcome {
        proto: u16,
        build: u32,
        codename: String,
        slot: u8,
        input_delay: u8,
        hash_every: u8,
        seed: u32,
        settings: MatchSettings,
    },
    Refuse {
        reason: u8,
    },
    Input {
        base_tick: u32,
        /// Bit i = slot i carries an input each tick (u16 since protocol 4 —
        /// 16 agents [S mp-16-slots]; was u8).
        agent_mask: u16,
        ack_tick: u32,
        ack_bits: u32,
        inputs: Vec<Input>,
    },
    Hash {
        tick: u32,
        hash: u64,
        prev_tick: u32,
        prev_hash: u64,
    },
    Keepalive {
        seq: u32,
    },
    Leave {
        slot: u8,
        reason: u8,
    },
    SlotMap {
        /// Bit i = slot i is a live human (u16 since protocol 4 [S
        /// mp-16-slots]; was u8).
        human_slots: u16,
    },
    Go {
        first_tick: u32,
    },
    /// Opaque app-to-app payload (the lobby's choices mirror). The session
    /// relays it between connected peers and hands it up through
    /// `Event::Data` untouched.
    Data {
        bytes: Vec<u8>,
    },
    /// Host → late joiner: the handover head (see the module table).
    JoinBegin {
        first_tick: u32,
        snapshot_tick: u32,
        snapshot_hash: u64,
        /// The earlier late joins (slot, join tick) up to the snapshot — the replay must spawn
        /// those bodies where the live machines did (mp-online-failures: a second late join, or
        /// a reconnect of a late joiner, could never verify without them).
        joins: Vec<(u8, u32)>,
    },
    /// Host → late joiner: one chunk of the recorded run log
    /// (`tick_count × MAX_AGENTS` inputs, tick-major, slot-ascending).
    JoinLog {
        first_tick: u32,
        tick_count: u8,
        chunk_index: u16,
        chunk_count: u16,
        inputs: Vec<Input>,
    },
    /// Late joiner → host: chunks ≤ `chunk_index` are held.
    JoinAck {
        chunk_index: u16,
    },
    /// Late joiner → host: the replay verified and the joiner is caught up.
    JoinReady {},
    /// Host → all: `slot` becomes human from `join_tick` (inputs start there).
    JoinSlot {
        slot: u8,
        join_tick: u32,
    },
}

fn put_codename(out: &mut Vec<u8>, name: &str) {
    let mut buf = [0u8; CODENAME_WIRE];
    let bytes = name.as_bytes();
    let n = bytes.len().min(CODENAME_WIRE - 1);
    buf[..n].copy_from_slice(&bytes[..n]);
    out.extend_from_slice(&buf);
}

fn get_codename(buf: &[u8], off: usize) -> Option<String> {
    let raw = buf.get(off..off + CODENAME_WIRE)?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(CODENAME_WIRE);
    Some(String::from_utf8_lossy(&raw[..end]).into_owned())
}

/// Encode a packet to bytes (little-endian, header first).
pub fn encode(p: &Packet, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(64);
    out.extend_from_slice(&MAGIC.to_le_bytes());
    match p {
        Packet::Hello { proto, build, codename } => {
            out.push(PT_HELLO);
            out.push(0);
            out.extend_from_slice(&proto.to_le_bytes());
            out.extend_from_slice(&build.to_le_bytes());
            put_codename(out, codename);
        }
        Packet::Welcome { proto, build, codename, slot, input_delay, hash_every, seed, settings } => {
            out.push(PT_WELCOME);
            out.push(0);
            out.extend_from_slice(&proto.to_le_bytes());
            out.extend_from_slice(&build.to_le_bytes());
            put_codename(out, codename);
            out.push(*slot);
            out.push(*input_delay);
            out.push(*hash_every);
            out.push(0);
            out.extend_from_slice(&seed.to_le_bytes());
            let mut sb = [0u8; SETTINGS_WIRE_SIZE];
            settings.encode(&mut sb);
            out.extend_from_slice(&sb);
        }
        Packet::Refuse { reason } => {
            out.push(PT_REFUSE);
            out.push(0);
            out.push(*reason);
            out.extend_from_slice(&[0, 0, 0]);
        }
        Packet::Input { base_tick, agent_mask, ack_tick, ack_bits, inputs } => {
            out.push(PT_INPUT);
            out.push(0);
            out.extend_from_slice(&base_tick.to_le_bytes());
            out.extend_from_slice(&agent_mask.to_le_bytes());
            let per_tick = agent_mask.count_ones().max(1) as usize;
            out.push((inputs.len() / per_tick) as u8);
            out.extend_from_slice(&ack_tick.to_le_bytes());
            out.extend_from_slice(&ack_bits.to_le_bytes());
            for inp in inputs {
                let mut b = [0u8; INPUT_WIRE_SIZE];
                inp.encode(&mut b, 0);
                out.extend_from_slice(&b);
            }
        }
        Packet::Hash { tick, hash, prev_tick, prev_hash } => {
            out.push(PT_HASH);
            out.push(0);
            out.extend_from_slice(&tick.to_le_bytes());
            out.extend_from_slice(&hash.to_le_bytes());
            out.extend_from_slice(&prev_tick.to_le_bytes());
            out.extend_from_slice(&prev_hash.to_le_bytes());
        }
        Packet::Keepalive { seq } => {
            out.push(PT_KEEPALIVE);
            out.push(0);
            out.extend_from_slice(&seq.to_le_bytes());
        }
        Packet::Leave { slot, reason } => {
            out.push(PT_LEAVE);
            out.push(0);
            out.push(*slot);
            out.push(*reason);
            out.extend_from_slice(&[0, 0]);
        }
        Packet::SlotMap { human_slots } => {
            out.push(PT_SLOTMAP);
            out.push(0);
            out.extend_from_slice(&human_slots.to_le_bytes());
            out.extend_from_slice(&[0, 0]);
        }
        Packet::Go { first_tick } => {
            out.push(PT_GO);
            out.push(0);
            out.extend_from_slice(&first_tick.to_le_bytes());
        }
        Packet::Data { bytes } => {
            out.push(PT_DATA);
            out.push(0);
            out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
            out.extend_from_slice(bytes);
        }
        Packet::JoinBegin { first_tick, snapshot_tick, snapshot_hash, joins } => {
            out.push(PT_JOIN_BEGIN);
            out.push(0);
            out.extend_from_slice(&first_tick.to_le_bytes());
            out.extend_from_slice(&snapshot_tick.to_le_bytes());
            out.extend_from_slice(&snapshot_hash.to_le_bytes());
            let n = joins.len().min(255);
            out.push(n as u8);
            for (slot, tick) in joins.iter().take(n) {
                out.push(*slot);
                out.extend_from_slice(&tick.to_le_bytes());
            }
        }
        Packet::JoinLog { first_tick, tick_count, chunk_index, chunk_count, inputs } => {
            out.push(PT_JOIN_LOG);
            out.push(0);
            out.extend_from_slice(&first_tick.to_le_bytes());
            out.push(*tick_count);
            out.extend_from_slice(&chunk_index.to_le_bytes());
            out.extend_from_slice(&chunk_count.to_le_bytes());
            for inp in inputs {
                let mut b = [0u8; INPUT_WIRE_SIZE];
                inp.encode(&mut b, 0);
                out.extend_from_slice(&b);
            }
        }
        Packet::JoinAck { chunk_index } => {
            out.push(PT_JOIN_ACK);
            out.push(0);
            out.extend_from_slice(&chunk_index.to_le_bytes());
        }
        Packet::JoinReady {} => {
            out.push(PT_JOIN_READY);
            out.push(0);
        }
        Packet::JoinSlot { slot, join_tick } => {
            out.push(PT_JOIN_SLOT);
            out.push(0);
            out.push(*slot);
            out.extend_from_slice(&join_tick.to_le_bytes());
        }
    }
}

/// Decode bytes into a [`Packet`]; `None` if malformed (bad magic, unknown
/// type, truncated payload, over-long, inconsistent INPUT count/mask).
pub fn decode(buf: &[u8]) -> Option<Packet> {
    if buf.len() < 4 || buf.len() > MAX_PACKET {
        return None;
    }
    if u16::from_le_bytes([buf[0], buf[1]]) != MAGIC {
        return None;
    }
    let ptype = buf[2];
    let body = &buf[4..];
    match ptype {
        PT_HELLO => {
            if body.len() < 6 + CODENAME_WIRE {
                return None;
            }
            Some(Packet::Hello {
                proto: u16::from_le_bytes([body[0], body[1]]),
                build: u32::from_le_bytes([body[2], body[3], body[4], body[5]]),
                codename: get_codename(body, 6)?,
            })
        }
        PT_WELCOME => {
            if body.len() < 6 + CODENAME_WIRE + 4 + 4 + SETTINGS_WIRE_SIZE {
                return None;
            }
            Some(Packet::Welcome {
                proto: u16::from_le_bytes([body[0], body[1]]),
                build: u32::from_le_bytes([body[2], body[3], body[4], body[5]]),
                codename: get_codename(body, 6)?,
                slot: body[6 + CODENAME_WIRE],
                input_delay: body[6 + CODENAME_WIRE + 1],
                hash_every: body[6 + CODENAME_WIRE + 2],
                seed: u32::from_le_bytes([
                    body[6 + CODENAME_WIRE + 4],
                    body[6 + CODENAME_WIRE + 5],
                    body[6 + CODENAME_WIRE + 6],
                    body[6 + CODENAME_WIRE + 7],
                ]),
                settings: MatchSettings::decode(&body[6 + CODENAME_WIRE + 8..])?,
            })
        }
        PT_REFUSE => {
            if body.is_empty() {
                return None;
            }
            Some(Packet::Refuse { reason: body[0] })
        }
        PT_INPUT => {
            // The 15-byte fixed head (protocol ≤ 3: 14 — the mask was u8).
            if body.len() < 15 {
                return None;
            }
            let base_tick = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
            let agent_mask = u16::from_le_bytes([body[4], body[5]]);
            let count = body[6] as usize;
            let ack_tick = u32::from_le_bytes([body[7], body[8], body[9], body[10]]);
            let ack_bits = u32::from_le_bytes([body[11], body[12], body[13], body[14]]);
            if agent_mask == 0 {
                return None;
            }
            let per_tick = agent_mask.count_ones() as usize;
            if count == 0 || count > MAX_INPUT_COUNT_C2H {
                return None;
            }
            let need = 15 + count * per_tick * INPUT_WIRE_SIZE;
            if body.len() < need {
                return None;
            }
            let mut inputs = Vec::with_capacity(count * per_tick);
            for i in 0..count * per_tick {
                inputs.push(Input::decode(body, 15 + i * INPUT_WIRE_SIZE)?);
            }
            Some(Packet::Input { base_tick, agent_mask, ack_tick, ack_bits, inputs })
        }
        PT_HASH => {
            if body.len() < 24 {
                return None;
            }
            Some(Packet::Hash {
                tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
                hash: u64::from_le_bytes([
                    body[4], body[5], body[6], body[7], body[8], body[9], body[10], body[11],
                ]),
                prev_tick: u32::from_le_bytes([body[12], body[13], body[14], body[15]]),
                prev_hash: u64::from_le_bytes([
                    body[16], body[17], body[18], body[19], body[20], body[21], body[22], body[23],
                ]),
            })
        }
        PT_KEEPALIVE => {
            if body.len() < 4 {
                return None;
            }
            Some(Packet::Keepalive {
                seq: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
            })
        }
        PT_LEAVE => {
            if body.len() < 2 {
                return None;
            }
            Some(Packet::Leave { slot: body[0], reason: body[1] })
        }
        PT_SLOTMAP => {
            if body.len() < 2 {
                return None;
            }
            Some(Packet::SlotMap { human_slots: u16::from_le_bytes([body[0], body[1]]) })
        }
        PT_GO => {
            if body.len() < 4 {
                return None;
            }
            Some(Packet::Go {
                first_tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
            })
        }
        PT_DATA => {
            if body.len() < 2 {
                return None;
            }
            let n = u16::from_le_bytes([body[0], body[1]]) as usize;
            if body.len() < 2 + n {
                return None;
            }
            Some(Packet::Data { bytes: body[2..2 + n].to_vec() })
        }
        PT_JOIN_BEGIN => {
            if body.len() < 16 {
                return None;
            }
            // The join history (mp-online-failures): a count byte + (slot u8, join_tick u32)
            // per earlier late join; absent = none.
            let mut joins = Vec::new();
            if body.len() > 16 {
                let n = body[16] as usize;
                if body.len() < 17 + n * 5 {
                    return None;
                }
                for k in 0..n {
                    let o = 17 + k * 5;
                    joins.push((body[o], u32::from_le_bytes([body[o + 1], body[o + 2], body[o + 3], body[o + 4]])));
                }
            }
            Some(Packet::JoinBegin {
                first_tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
                snapshot_tick: u32::from_le_bytes([body[4], body[5], body[6], body[7]]),
                snapshot_hash: u64::from_le_bytes([
                    body[8], body[9], body[10], body[11], body[12], body[13], body[14], body[15],
                ]),
                joins,
            })
        }
        PT_JOIN_LOG => {
            if body.len() < 11 {
                return None;
            }
            let first_tick = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
            let tick_count = body[4] as usize;
            let chunk_index = u16::from_le_bytes([body[5], body[6]]);
            let chunk_count = u16::from_le_bytes([body[7], body[8]]);
            if tick_count == 0 || tick_count > JOIN_LOG_CHUNK_TICKS {
                return None;
            }
            let need = 9 + tick_count * MAX_AGENTS * INPUT_WIRE_SIZE;
            if body.len() < need {
                return None;
            }
            let mut inputs = Vec::with_capacity(tick_count * MAX_AGENTS);
            for i in 0..tick_count * MAX_AGENTS {
                inputs.push(Input::decode(body, 9 + i * INPUT_WIRE_SIZE)?);
            }
            Some(Packet::JoinLog { first_tick, tick_count: tick_count as u8, chunk_index, chunk_count, inputs })
        }
        PT_JOIN_ACK => {
            if body.len() < 2 {
                return None;
            }
            Some(Packet::JoinAck { chunk_index: u16::from_le_bytes([body[0], body[1]]) })
        }
        PT_JOIN_READY => Some(Packet::JoinReady {}),
        PT_JOIN_SLOT => {
            if body.len() < 5 {
                return None;
            }
            Some(Packet::JoinSlot {
                slot: body[0],
                join_tick: u32::from_le_bytes([body[1], body[2], body[3], body[4]]),
            })
        }
        _ => None,
    }
}

/// Convenience: default protocol version check used by both roles.
pub fn version_matches(proto: u16) -> bool {
    proto == PROTOCOL_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::NEUTRAL;

    #[test]
    fn hello_roundtrip() {
        let p = Packet::Hello {
            proto: PROTOCOL_VERSION,
            build: 42,
            codename: "nightfire-port".into(),
        };
        let mut b = Vec::new();
        encode(&p, &mut b);
        assert_eq!(decode(&b).unwrap(), p);
    }

    #[test]
    fn input_roundtrip() {
        let mut inputs = Vec::new();
        for i in 0..6u32 {
            inputs.push(Input {
                forward: i as f32,
                ..NEUTRAL
            });
        }
        let p = Packet::Input {
            base_tick: 100,
            agent_mask: 0b0111u16,
            ack_tick: 97,
            ack_bits: 0b101,
            inputs,
        };
        let mut b = Vec::new();
        encode(&p, &mut b);
        assert_eq!(decode(&b).unwrap(), p);
    }

    #[test]
    fn input_u16_mask_and_size_limits() {
        // mp-16-slots [S]: a full 16-agent mask roundtrips — bit 15 rides the
        // high byte the u8 mask could not carry.
        let inputs = vec![NEUTRAL; 2 * 16];
        let p = Packet::Input {
            base_tick: 500,
            agent_mask: 0xffffu16,
            ack_tick: 497,
            ack_bits: 0,
            inputs,
        };
        let mut b = Vec::new();
        encode(&p, &mut b);
        assert_eq!(decode(&b).unwrap(), p);
        assert_eq!(b[8], 0xff, "mask low byte (header 4 + tick 4)");
        assert_eq!(b[9], 0xff, "mask high byte");
        // The largest relay packet fits MAX_PACKET: 3 ticks × 15 agents × 24 B
        // + the 15-byte head + the 4-byte header.
        let h2c = Packet::Input {
            base_tick: 0,
            agent_mask: 0xffffu16 & !(1u16 << 7), // every agent but slot 7
            ack_tick: u32::MAX,
            ack_bits: 0,
            inputs: vec![NEUTRAL; 3 * 15],
        };
        encode(&h2c, &mut b);
        assert!(b.len() <= MAX_PACKET, "the 16-agent H2C relay packet must fit MAX_PACKET: {}", b.len());
        // Sister: one more tick would overflow the wire cap — the sender must
        // cap at MAX_INPUT_COUNT_H2C.
        assert!(3 * 16 * 24 + 15 + 4 > 3 * 15 * 24 + 15 + 4);
        assert!(decode(&b).is_some());
    }

    #[test]
    fn slotmap_u16_roundtrip() {
        // mp-16-slots [S]: slot 15's bit survives (u8 could not carry it).
        let p = Packet::SlotMap { human_slots: 0b1000_0000_0000_0001u16 };
        let mut b = Vec::new();
        encode(&p, &mut b);
        assert_eq!(decode(&b).unwrap(), p);
        // Sister: a body shorter than the u16 mask must not decode.
        assert!(decode(&b[..b.len() - 3]).is_none());
    }

    #[test]
    fn welcome_roundtrip() {
        let p = Packet::Welcome {
            proto: PROTOCOL_VERSION,
            build: 7,
            codename: "nightfire-port".into(),
            slot: 2,
            input_delay: 3,
            hash_every: 30,
            seed: 0xdead_beef,
            settings: MatchSettings::default(),
        };
        let mut b = Vec::new();
        encode(&p, &mut b);
        assert_eq!(decode(&b).unwrap(), p);
    }

    #[test]
    fn garbage_is_none() {
        assert!(decode(&[]).is_none());
        assert!(decode(&[0x00, 0x00, 0, 0, 1, 2, 3]).is_none());
        let mut b = Vec::new();
        encode(&Packet::Keepalive { seq: 1 }, &mut b);
        b[2] = 200; // unknown type
        assert!(decode(&b).is_none());
        // Sister assertion: truncated payload must not decode.
        assert!(decode(&b[..3]).is_none());
    }

    /// mp-online-failures: JOIN_BEGIN carries the join history; an empty one round-trips too,
    /// and a truncated history is rejected (sister).
    #[test]
    fn join_begin_history_roundtrip_and_truncation() {
        for joins in [vec![], vec![(2u8, 391u32), (5, 77_001)]] {
            let p = Packet::JoinBegin { first_tick: 0, snapshot_tick: 300, snapshot_hash: 0xDEAD_BEEF_0123, joins };
            let mut b = Vec::new();
            encode(&p, &mut b);
            assert_eq!(decode(&b), Some(p));
        }
        let mut b = Vec::new();
        encode(&Packet::JoinBegin { first_tick: 0, snapshot_tick: 1, snapshot_hash: 2, joins: vec![(1, 9)] }, &mut b);
        b.pop();
        assert_eq!(decode(&b), None, "a cut history entry is refused");
    }

    #[test]
    fn data_roundtrip_and_truncation() {
        let p = Packet::Data { bytes: vec![1, 2, 3, 0xAA, 0xBB] };
        let mut b = Vec::new();
        encode(&p, &mut b);
        assert_eq!(decode(&b).unwrap(), p);
        // Sister: a short body must not decode (the len prefix outruns it).
        assert!(decode(&b[..b.len() - 1]).is_none());
        assert!(decode(&b[..4]).is_none());
    }
}
