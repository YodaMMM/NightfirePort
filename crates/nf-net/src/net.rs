//! The real transport: a non-blocking [`std::net::UdpSocket`] wrapper.
//!
//! This is the only place this crate touches the OS network stack; tests use
//! the in-memory shim in [`crate::shim`] instead. No new crates — std only.

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use crate::MAX_PACKET;

/// Transport abstraction: send to an address, drain pending datagrams,
/// report the local bind address. Implemented by [`UdpTransport`] (real
/// UDP) and the test shim.
pub trait Transport {
    fn send(&mut self, to: SocketAddr, data: &[u8]);
    /// Non-blocking: returns one pending datagram and its origin, if any.
    fn try_recv(&mut self) -> Option<(SocketAddr, Vec<u8>)>;
    fn local_addr(&self) -> SocketAddr;
    /// The session's logical clock, passed on every poll/flush before any send or receive
    /// (net-internet: the fault line's delay queue runs on it). Default: ignored.
    fn set_now(&mut self, _now_ms: u64) {}
}

/// Real UDP transport. The host binds `0.0.0.0:port`; clients bind an
/// ephemeral port (`bind "127.0.0.1:0"` or the interface they need).
pub struct UdpTransport {
    sock: UdpSocket,
    scratch: Vec<u8>,
    /// Optional test fault injection (`inject_fault`): drops, transpositions and delay on
    /// the SEND path only, from a seeded SplitMix64 — the receive path and the protocol stay
    /// untouched. Production never sets it; the e2e tests use it to prove the lockstep
    /// survives a lossy, slow link.
    fault: Option<Fault>,
}

/// The seeded fault state (net-e2e-loopback, net-internet): `drop_per_mille`/1000 of the
/// sends are lost; `reorder_per_mille`/1000 are HELD in a one-slot delay line and flushed
/// after the next sent datagram — a transposition; with `delay_ms`/`jitter_ms` every
/// surviving datagram waits `delay + uniform(0..=jitter)` ms on the session's clock
/// ([`Transport::set_now`]) before it leaves — a one-way latency, and jitter reorders like
/// a real path does. The RNG is the same SplitMix64 the shim and the nf-net loss tests use,
/// so a given packet sequence sees a reproducible fault pattern.
struct Fault {
    rng: u64,
    drop_per_mille: u32,
    reorder_per_mille: u32,
    held: Option<(SocketAddr, Vec<u8>)>,
    delay_ms: u64,
    jitter_ms: u64,
    now_ms: u64,
    /// (release ms, to, bytes) — unsorted (jitter); released by `pump`.
    queue: Vec<(u64, SocketAddr, Vec<u8>)>,
}

impl UdpTransport {
    /// Bind a UDP socket. Hosts: `"0.0.0.0:7777"`; clients: `"127.0.0.1:0"`.
    pub fn bind(addr: &str) -> std::io::Result<UdpTransport> {
        let sock = UdpSocket::bind(addr)?;
        sock.set_nonblocking(true)?;
        // net-p2p §3a: the same-network PROBE is a broadcast.
        let _ = sock.set_broadcast(true);
        // Read timeout is irrelevant in non-blocking mode but set a sane
        // bound anyway; try_recv never blocks.
        sock.set_read_timeout(Some(Duration::from_millis(1)))?;
        Ok(UdpTransport { sock, scratch: vec![0u8; MAX_PACKET], fault: None })
    }

    /// Inject deterministic loss + reordering on this transport's sends
    /// (test-only; the e2e loopback variant's 5%-loss link). `seed` drives
    /// the SplitMix64; the same packet sequence + seed = the same faults.
    pub fn inject_fault(&mut self, seed: u64, drop_per_mille: u32, reorder_per_mille: u32) {
        self.inject_link(seed, drop_per_mille, reorder_per_mille, 0, 0);
    }

    /// net-internet: [`Self::inject_fault`] plus a one-way delay and uniform jitter (ms on
    /// the session's clock) — a simulated internet path in front of a real socket.
    pub fn inject_link(&mut self, seed: u64, drop_per_mille: u32, reorder_per_mille: u32, delay_ms: u64, jitter_ms: u64) {
        let now_ms = self.fault.as_ref().map_or(0, |f| f.now_ms);
        self.fault = Some(Fault {
            rng: seed.wrapping_add(0x1234_5678_9ABC_DEF0),
            drop_per_mille,
            reorder_per_mille,
            held: None,
            delay_ms,
            jitter_ms,
            now_ms,
            queue: Vec::new(),
        });
    }

    fn fault_rand(rng: &mut u64) -> u64 {
        *rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Release every delayed datagram whose time has come.
    fn pump(&mut self) {
        let Some(f) = self.fault.as_mut() else { return };
        let now = f.now_ms;
        let mut i = 0;
        while i < f.queue.len() {
            if f.queue[i].0 <= now {
                let (_, to, d) = f.queue.swap_remove(i);
                let _ = self.sock.send_to(&d, to);
            } else {
                i += 1;
            }
        }
    }

    /// The wire send after the loss/reorder dice: straight out, or into the delay queue.
    fn emit(sock: &UdpSocket, f: &mut Fault, to: SocketAddr, data: &[u8]) {
        if f.delay_ms == 0 && f.jitter_ms == 0 {
            let _ = sock.send_to(data, to);
            return;
        }
        let j = if f.jitter_ms > 0 { Self::fault_rand(&mut f.rng) % (f.jitter_ms + 1) } else { 0 };
        f.queue.push((f.now_ms + f.delay_ms + j, to, data.to_vec()));
    }
}

impl Transport for UdpTransport {
    fn send(&mut self, to: SocketAddr, data: &[u8]) {
        if let Some(f) = self.fault.as_mut() {
            if f.drop_per_mille > 0 && (Self::fault_rand(&mut f.rng) % 1000) < f.drop_per_mille as u64 {
                return; // lost
            }
            if f.reorder_per_mille > 0
                && (Self::fault_rand(&mut f.rng) % 1000) < f.reorder_per_mille as u64
            {
                // Hold this one; the previously held one (if any) goes out
                // NOW, after the send that displaced it — a transposition.
                let late = f.held.take();
                f.held = Some((to, data.to_vec()));
                if let Some((to, d)) = late {
                    Self::emit(&self.sock, f, to, &d);
                }
                return;
            }
            // The held datagram (if any) flushes with this send — it goes
            // first, so an ordinary send after a held one is also a swap.
            if let Some((hto, d)) = f.held.take() {
                Self::emit(&self.sock, f, hto, &d);
            }
            Self::emit(&self.sock, f, to, data);
            self.pump();
            return;
        }
        let _ = self.sock.send_to(data, to);
    }

    fn try_recv(&mut self) -> Option<(SocketAddr, Vec<u8>)> {
        self.pump();
        match self.sock.recv_from(&mut self.scratch) {
            Ok((n, from)) => Some((from, self.scratch[..n].to_vec())),
            Err(_) => None,
        }
    }

    fn local_addr(&self) -> SocketAddr {
        self.sock.local_addr().unwrap_or_else(|_| {
            "0.0.0.0:0".parse().expect("static fallback SocketAddr")
        })
    }

    fn set_now(&mut self, now_ms: u64) {
        if let Some(f) = self.fault.as_mut() {
            f.now_ms = now_ms;
        }
        self.pump();
    }
}
