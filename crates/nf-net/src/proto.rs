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
//! | 4 | INPUT | base_tick u32, agent_mask u16, count u8, ack_tick u32, ack_bits u32, count*popcount(mask) compact inputs (protocol 5: a word-mask byte + the changed 4-byte words, see [`encode_compact_inputs`]); count 0 = a pure ack; a slot absent from a host→client mask is NEUTRAL over the covered ticks |
//! | 5 | HASH | tick u32, hash u64, prev_tick u32, prev_hash u64 |
//! | 6 | KEEPALIVE | seq u32 |
//! | 7 | LEAVE | slot u8, reason u8, rsv 2B |
//! | 8 | SLOTMAP | human_slots bitmask u16, rsv 2B |
//! | 9 | GO (host→client) | first_tick u32 |
//! | 10 | DATA (app tunnel) | len u16 + bytes — opaque app-to-app blob (the lobby's choices); the session never inspects it |
//! | 11 | JOIN_BEGIN (host→late joiner) | first_tick u32, snapshot_tick u32, snapshot_hash u64 — the late-join handover head: the input log covers `first_tick..snapshot_tick` and the replay must hash to `snapshot_hash` at `snapshot_tick` |
//! | 12 | JOIN_LOG (host→late joiner) | first_tick u32, tick_count u8, chunk_index u16, chunk_count u16, tick_count × (16 × 28 B inputs) — one chunk of the recorded run log |
//! | 13 | JOIN_ACK (late joiner→host) | chunk_index u16 — all chunks ≤ chunk_index are held |
//! | 14 | JOIN_READY (late joiner→host) | — the replay verified (hash matched) and the joiner is caught up; the host picks the join tick |
//! | 15 | JOIN_SLOT (host→all) | slot u8, join_tick u32 — the slot becomes human from join_tick (its inputs start there; before it the slot rides NEUTRAL) |
//! | 16 | PING | stamp u32 (the sender's ms clock) — answered at once with PONG |
//! | 17 | PONG | stamp u32 echoed — the sender's RTT sample |
//! | 18 | DELAY (host→all) | from_tick u32, delay u8 — the input delay from that tick on (protocol 5) |
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
pub const PT_PING: u8 = 16;
pub const PT_PONG: u8 = 17;
pub const PT_DELAY: u8 = 18;
/// net-desync-detail: the desync diagnosis exchange (the components for the mismatched tick,
/// then the first diverging component's items).
pub const PT_HASHCOMPS: u8 = 19;
pub const PT_HASHDETAIL: u8 = 20;

/// The component count both the app's fold (`nf_view::scene::NET_HASH_COMPONENTS`) and
/// `PT_HASHCOMPS`'s payload agree on — the wire contract pins it.
pub const HASH_COMPONENTS: usize = 7;
/// The most items one `PT_HASHDETAIL` carries (8 B each — 160 fits [`MAX_PACKET`] with the
/// headers; a bigger world's items truncate, the desync report names the cap).
pub const HASH_DETAIL_CAP: usize = 160;
/// net-desync-resync: client → host, a state-hash mismatch happened and this machine is
/// resyncing — the host re-serves the late-join family on the live connection. 21 (19/20 are
/// net-desync-detail's, already on master; the resync branch was cut before it landed).
pub const PT_RESYNC: u8 = 21;

/// Ticks one JOIN_LOG chunk carries (16 agents × 28 B × 3 = 1344 B — fits
/// [`MAX_PACKET`] with header room (1357 B with the headers); protocol ≤ 3 carried 12 ticks of 4
/// agents, the same 1152 B [S mp-16-slots]).
pub const JOIN_LOG_CHUNK_TICKS: usize = 3;

/// codename field width on the wire (nul-padded).
pub const CODENAME_WIRE: usize = 32;
/// The most ticks one INPUT packet may carry (protocol 5). The inputs are compact
/// ([`encode_compact_inputs`]), so the real bound is the packet size: the sender adds ticks
/// while the packet stays within [`MAX_PACKET`] (the worst case — every word of every one of
/// 15 relayed agents changing — still fits 3 ticks). Protocol 4 capped the relay at 3 ticks
/// of raw 24-byte inputs, which with its oldest-unacked-first window held a 150 ms link to
/// ~15 ticks/s [S net-internet].
pub const MAX_INPUT_TICKS: usize = 64;

/// The INPUT packet's fixed size before the inputs: the 4-byte header + the 15-byte head.
pub const INPUT_HEAD_LEN: usize = 19;

/// Bytes the compact form of `cur` takes after `prev` (its reference: the same slot's
/// previous tick in the packet, NEUTRAL for the first): one word-mask byte plus each 4-byte
/// word of the 24-byte layout that differs.
pub fn compact_input_len(prev: &Input, cur: &Input) -> usize {
    let (mut a, mut b) = ([0u8; INPUT_WIRE_SIZE], [0u8; INPUT_WIRE_SIZE]);
    prev.encode(&mut a, 0);
    cur.encode(&mut b, 0);
    1 + (0..INPUT_WIRE_SIZE / 4).filter(|w| a[w * 4..w * 4 + 4] != b[w * 4..w * 4 + 4]).count() * 4
}

fn neutral_wire() -> [u8; INPUT_WIRE_SIZE] {
    let mut r = [0u8; INPUT_WIRE_SIZE];
    crate::input::NEUTRAL.encode(&mut r, 0);
    r
}

/// The INPUT body's compact inputs (protocol 5): tick-major, slot-ascending; each input is a
/// word-mask byte (bit w = 4-byte word w of the 24-byte [`Input`] layout differs from the
/// reference) followed by the differing words. The reference is the same slot's input in the
/// previous tick of the packet, NEUTRAL's layout for the first tick — sticks held and buttons
/// unchanged cost one byte, a turning mouse 9 [S net-internet].
pub fn encode_compact_inputs(agent_mask: u16, inputs: &[Input], out: &mut Vec<u8>) {
    let per_tick = agent_mask.count_ones() as usize;
    if per_tick == 0 {
        return;
    }
    let mut refs = vec![neutral_wire(); per_tick];
    for (i, inp) in inputs.iter().enumerate() {
        let j = i % per_tick;
        let mut b = [0u8; INPUT_WIRE_SIZE];
        inp.encode(&mut b, 0);
        let mut m = 0u8;
        for w in 0..INPUT_WIRE_SIZE / 4 {
            if b[w * 4..w * 4 + 4] != refs[j][w * 4..w * 4 + 4] {
                m |= 1 << w;
            }
        }
        out.push(m);
        for w in 0..INPUT_WIRE_SIZE / 4 {
            if m & (1 << w) != 0 {
                out.extend_from_slice(&b[w * 4..w * 4 + 4]);
            }
        }
        refs[j] = b;
    }
}

/// Inverse of [`encode_compact_inputs`]; `None` on a truncated or malformed body.
pub fn decode_compact_inputs(agent_mask: u16, count: usize, body: &[u8]) -> Option<Vec<Input>> {
    let per_tick = agent_mask.count_ones() as usize;
    let mut out = Vec::with_capacity(count * per_tick);
    if per_tick == 0 {
        return Some(out);
    }
    let mut refs = vec![neutral_wire(); per_tick];
    let mut p = 0usize;
    for i in 0..count * per_tick {
        let j = i % per_tick;
        let m = *body.get(p)?;
        p += 1;
        if m >> (INPUT_WIRE_SIZE / 4) != 0 {
            return None;
        }
        for w in 0..INPUT_WIRE_SIZE / 4 {
            if m & (1 << w) != 0 {
                refs[j][w * 4..w * 4 + 4].copy_from_slice(body.get(p..p + 4)?);
                p += 4;
            }
        }
        out.push(Input::decode(&refs[j], 0)?);
    }
    Some(out)
}

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
        /// net-desync-resync: the newest tick the host RAN with the (re)joining
        /// slot's real inputs — a resyncer's catch-up may use its recorded own
        /// inputs up to here and must ride NEUTRAL above it (the host's rows in
        /// between are NEUTRAL: the slot stopped contributing at the serve). A
        /// fresh late joiner's serve sets it to `snapshot_tick` (its own inputs
        /// map is empty, so the field is inert there).
        ran_through: u32,
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
    /// Either way: a round-trip probe; the receiver answers PONG with the same stamp at once
    /// (net-internet: the RTT the adaptive input delay is chosen from).
    Ping {
        stamp: u32,
    },
    Pong {
        stamp: u32,
    },
    /// Host → all: from `from_tick` on, the input sampled while running tick `t` is scheduled
    /// for `t + delay` (net-internet: the adaptive input delay, announced ahead of the
    /// boundary so every machine switches at the same tick).
    Delay {
        from_tick: u32,
        delay: u8,
    },
    /// net-desync-resync: client → host, a state-hash mismatch happened and
    /// this machine is resyncing — the host re-serves the late-join family
    /// (JOIN_BEGIN + JOIN_LOG + JOIN_SLOT) to it on the live connection. The
    /// desync's tick and both hashes ride the app's log (the Desync event
    /// carries them); the request only needs the tick it froze at.
    Resync {
        tick: u32,
    },
    /// net-desync-detail: on a desync each machine sends the components for the mismatched
    /// tick — the 7 named component hashes in the fixed fold order
    /// (`nf_view::scene::NET_HASH_COMPONENT_NAMES`; the order is this packet's contract).
    HashComps {
        tick: u32,
        comps: [u64; HASH_COMPONENTS],
    },
    /// net-desync-detail: the first diverging component's per-item hashes for the mismatched
    /// tick (the item layout per component is documented on
    /// `nf_view::scene::Sim::net_component_items`; at most [`HASH_DETAIL_CAP`] items ride one
    /// packet — a bigger world truncates the tail, the report says so).
    HashDetail {
        tick: u32,
        component: u8,
        items: Vec<u64>,
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
            encode_compact_inputs(*agent_mask, inputs, out);
        }
        Packet::HashComps { tick, comps } => {
            out.push(PT_HASHCOMPS);
            out.push(0);
            out.extend_from_slice(&tick.to_le_bytes());
            for c in comps {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        Packet::HashDetail { tick, component, items } => {
            out.push(PT_HASHDETAIL);
            out.push(0);
            out.extend_from_slice(&tick.to_le_bytes());
            out.push(*component);
            let n = items.len().min(HASH_DETAIL_CAP);
            out.extend_from_slice(&(n as u16).to_le_bytes());
            for item in items.iter().take(n) {
                out.extend_from_slice(&item.to_le_bytes());
            }
        }
        Packet::Ping { stamp } => {
            out.push(PT_PING);
            out.push(0);
            out.extend_from_slice(&stamp.to_le_bytes());
        }
        Packet::Pong { stamp } => {
            out.push(PT_PONG);
            out.push(0);
            out.extend_from_slice(&stamp.to_le_bytes());
        }
        Packet::Delay { from_tick, delay } => {
            out.push(PT_DELAY);
            out.push(0);
            out.extend_from_slice(&from_tick.to_le_bytes());
            out.push(*delay);
        }
        Packet::Resync { tick } => {
            out.push(PT_RESYNC);
            out.push(0);
            out.extend_from_slice(&tick.to_le_bytes());
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
        Packet::JoinBegin { first_tick, ran_through, snapshot_tick, snapshot_hash, joins } => {
            out.push(PT_JOIN_BEGIN);
            out.push(0);
            out.extend_from_slice(&first_tick.to_le_bytes());
            out.extend_from_slice(&snapshot_tick.to_le_bytes());
            out.extend_from_slice(&snapshot_hash.to_le_bytes());
            out.extend_from_slice(&ran_through.to_le_bytes());
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
            // count 0 = a pure ack (protocol 5); a mask of 0 = every slot NEUTRAL over the
            // covered ticks (the relay omits slots it knows are NEUTRAL there).
            if count > MAX_INPUT_TICKS {
                return None;
            }
            let inputs = decode_compact_inputs(agent_mask, count, &body[15..])?;
            Some(Packet::Input { base_tick, agent_mask, ack_tick, ack_bits, inputs })
        }
        PT_PING | PT_PONG => {
            if body.len() < 4 {
                return None;
            }
            let stamp = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
            Some(if ptype == PT_PING { Packet::Ping { stamp } } else { Packet::Pong { stamp } })
        }
        PT_DELAY => {
            if body.len() < 5 {
                return None;
            }
            Some(Packet::Delay {
                from_tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
                delay: body[4],
            })
        }
        PT_RESYNC => {
            if body.len() < 4 {
                return None;
            }
            Some(Packet::Resync { tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]) })
        }
        PT_HASHCOMPS => {
            if body.len() < 4 + HASH_COMPONENTS * 8 {
                return None;
            }
            let mut comps = [0u64; HASH_COMPONENTS];
            for (i, c) in comps.iter_mut().enumerate() {
                let o = 4 + i * 8;
                *c = u64::from_le_bytes(body[o..o + 8].try_into().unwrap());
            }
            Some(Packet::HashComps { tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]), comps })
        }
        PT_HASHDETAIL => {
            if body.len() < 7 {
                return None;
            }
            let n = u16::from_le_bytes([body[5], body[6]]) as usize;
            if body.len() < 7 + n * 8 {
                return None;
            }
            let mut items = Vec::with_capacity(n);
            for i in 0..n {
                let o = 7 + i * 8;
                items.push(u64::from_le_bytes(body[o..o + 8].try_into().unwrap()));
            }
            Some(Packet::HashDetail {
                tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
                component: body[4],
                items,
            })
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
            if body.len() < 20 {
                return None;
            }
            // The join history (mp-online-failures): a count byte + (slot u8, join_tick u32)
            // per earlier late join; absent = none.
            let mut joins = Vec::new();
            if body.len() > 20 {
                let n = body[20] as usize;
                if body.len() < 21 + n * 5 {
                    return None;
                }
                for k in 0..n {
                    let o = 21 + k * 5;
                    joins.push((body[o], u32::from_le_bytes([body[o + 1], body[o + 2], body[o + 3], body[o + 4]])));
                }
            }
            Some(Packet::JoinBegin {
                first_tick: u32::from_le_bytes([body[0], body[1], body[2], body[3]]),
                ran_through: u32::from_le_bytes([body[16], body[17], body[18], body[19]]),
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
        // The largest relay packet fits MAX_PACKET: 3 ticks × 15 agents × 28 B
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
        assert!(decode(&b).is_some());
    }

    /// net-internet (protocol 5): the compact inputs round-trip bit-exact; unchanged inputs
    /// cost one byte; the worst case (every word of 15 agents changing every tick) still fits
    /// 3 ticks in [`MAX_PACKET`]; a truncated body is refused (sister).
    #[test]
    fn compact_inputs_roundtrip_and_size() {
        let mut inputs = Vec::new();
        for t in 0..5u32 {
            for s in 0..3u32 {
                inputs.push(Input {
                    forward: if s == 1 { 1.0 } else { 0.0 },
                    turn: (t * 7 + s) as f32 * 0.013,
                    look: -0.25,
                    jump: t == 2,
                    switch_to: if t == 3 { Some(4) } else { None },
                    held: 2,
                    shot: t == 4 && s == 0,
                    ..NEUTRAL
                });
            }
        }
        let p = Packet::Input { base_tick: 77, agent_mask: 0b1011, ack_tick: 70, ack_bits: 0b11, inputs };
        let mut b = Vec::new();
        encode(&p, &mut b);
        assert_eq!(decode(&b), Some(p.clone()));
        assert!(decode(&b[..b.len() - 1]).is_none(), "a cut body is refused");
        // Unchanged inputs: one word-mask byte each.
        let idle = Packet::Input { base_tick: 0, agent_mask: 0x7fff, ack_tick: 0, ack_bits: 0, inputs: vec![NEUTRAL; 4 * 15] };
        encode(&idle, &mut b);
        assert_eq!(b.len(), INPUT_HEAD_LEN + 4 * 15, "NEUTRAL x 60 = 60 bytes");
        assert_eq!(decode(&b), Some(idle));
        assert_eq!(compact_input_len(&NEUTRAL, &NEUTRAL), 1);
        // The worst case: every word differs from the previous tick's.
        let mut worst = Vec::new();
        for t in 0..3u32 {
            for s in 0..15u32 {
                worst.push(Input {
                    forward: 0.1 + t as f32,
                    strafe: 0.2 + t as f32,
                    turn: 0.3 + t as f32 + s as f32,
                    look: 0.4 + t as f32,
                    jump: t % 2 == 0,
                    crouch: t % 2 == 1,
                    weapon_cycle: t as i8 + 1,
                    switch_to: Some(t as u8),
                    held: t as u8 + 1,
                    ..NEUTRAL
                });
            }
        }
        let w = Packet::Input { base_tick: 0, agent_mask: 0x7fff, ack_tick: 0, ack_bits: 0, inputs: worst };
        encode(&w, &mut b);
        assert_eq!(b.len(), INPUT_HEAD_LEN + 3 * 15 * 25);
        assert!(b.len() <= MAX_PACKET, "3 worst-case relay ticks fit: {}", b.len());
        assert_eq!(decode(&b), Some(w));
    }

    /// A pure ack (count 0), PING/PONG and DELAY round-trip; a bad word mask is refused.
    #[test]
    fn ack_probe_delay_roundtrip() {
        let mut b = Vec::new();
        for p in [
            Packet::Input { base_tick: 9, agent_mask: 1 << 3, ack_tick: 8, ack_bits: 5, inputs: vec![] },
            Packet::Ping { stamp: 123_456 },
            Packet::Pong { stamp: 7 },
            Packet::Delay { from_tick: 600, delay: 9 },
        ] {
            encode(&p, &mut b);
            assert_eq!(decode(&b), Some(p));
        }
        encode(&Packet::Delay { from_tick: 1, delay: 2 }, &mut b);
        assert!(decode(&b[..b.len() - 1]).is_none(), "a cut DELAY is refused");
        let mut bad = Vec::new();
        encode(&Packet::Input { base_tick: 0, agent_mask: 1, ack_tick: 0, ack_bits: 0, inputs: vec![NEUTRAL] }, &mut bad);
        let n = bad.len();
        bad[n - 1] = 0x80; // a word-mask bit past the 6 words
        assert!(decode(&bad).is_none(), "an impossible word mask is refused");
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
            let p = Packet::JoinBegin { first_tick: 0, ran_through: 300, snapshot_tick: 300, snapshot_hash: 0xDEAD_BEEF_0123, joins };
            let mut b = Vec::new();
            encode(&p, &mut b);
            assert_eq!(decode(&b), Some(p));
        }
        let mut b = Vec::new();
        encode(&Packet::JoinBegin { first_tick: 0, ran_through: 300, snapshot_tick: 1, snapshot_hash: 2, joins: vec![(1, 9)] }, &mut b);
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
