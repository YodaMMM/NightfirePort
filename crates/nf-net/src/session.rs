//! The lockstep session: host/client state machine over a [`Transport`].
//!
//! API (the host app calls exactly these):
//! * [`Session::poll`] — pump the network, run timers, return [`Event`]s.
//!   Takes a logical `now_ms`: the crate never reads a clock (determinism
//!   rule — no wall clock; the app passes real ms, tests pass simulated ms).
//! * [`Session::local_input`] — record the local agent's input for a tick.
//!   The app samples controls once per 60 Hz frame and schedules the input
//!   `input_delay` ticks ahead of the tick it is executing (config default 3,
//!   task range 2..=4). While stalled, keep scheduling: the input for a
//!   future tick is sampled from present controls, never guessed.
//! * [`Session::ready_inputs`] — `Some([Input; 4])` when every agent's input
//!   for `tick` is known; run the sim tick only then. `None` = stall.
//! * [`Session::submit_hash`] — report the local state hash after running a
//!   tick (opaque u64; task 050's `state_hash(&MpWorld)` wires in here).
//! * [`Session::start_match`] (host) / [`Session::leave`].
//!
//! Lockstep invariants:
//! * **Never guess.** A missing human input stalls the sim
//!   (`ready_inputs == None`, `Event::Stalled`) until it arrives.
//! * **Dropped agent = neutral input**, the game's own disconnected-pad
//!   shape (`psiInput_PollDevices` 0x10ca78 writes buttons 0 / analog 0x7f
//!   for a dead pad and the match runs on) — not a bot takeover (bots are
//!   fixed into slots 4..8 at `MP_Start` 0x183a50, never mid-match) and not
//!   a match end (match end on the console is only the explicit pause-menu
//!   QUIT, `C_GCPAUSE_Handler` 0x204408). `DropPolicy::EndMatch` exists as
//!   an option but `Neutral` is the default.
//! * Inputs flow with redundancy: every packet carries the unacked window
//!   plus the last `Config::redundancy` inputs, so one packet surviving loss
//!   of all others still delivers every tick (see `send_window_start`).
//! * The state hash is exchanged every `hash_every` ticks (default 30) and
//!   the previous hash piggybacks on each packet; a mismatch emits
//!   [`Event::Desync`] and keeps the session alive (the app decides).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::net::SocketAddr;

use crate::input::{Input, NEUTRAL};
use crate::net::Transport;
use crate::proto::{self, Packet};
use crate::settings::MatchSettings;
use crate::{MAX_AGENTS, PROTOCOL_VERSION};

/// Why a peer was removed from the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaveReason {
    /// Sent `LEAVE` (user quit).
    Graceful,
    /// No packets for `Config::timeout_ms`.
    Timeout,
    /// The match was ended by policy (`DropPolicy::EndMatch`).
    MatchEnded,
    /// The late joiner's replay hash disagreed with the host's snapshot
    /// (mp-join-in-progress: the join refuses ITSELF with this reason).
    JoinVerifyFailed,
}

impl LeaveReason {
    pub fn to_wire(self) -> u8 {
        match self {
            LeaveReason::Graceful => 0,
            LeaveReason::Timeout => 1,
            LeaveReason::MatchEnded => 2,
            LeaveReason::JoinVerifyFailed => 3,
        }
    }
    pub fn from_wire(v: u8) -> LeaveReason {
        match v {
            0 => LeaveReason::Graceful,
            1 => LeaveReason::Timeout,
            2 => LeaveReason::MatchEnded,
            _ => LeaveReason::JoinVerifyFailed,
        }
    }
}

/// What the host refuses a joining client for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefuseReason {
    Version,
    Build,
    /// The match already started AND late join is unavailable (no verifiable
    /// snapshot — the host's recorded log or hash for the head is missing).
    /// A HELLO after GO otherwise BEGINS a late join (mp-join-in-progress).
    Started,
    Full,
}

impl RefuseReason {
    pub fn to_wire(self) -> u8 {
        match self {
            RefuseReason::Version => 1,
            RefuseReason::Build => 2,
            RefuseReason::Started => 3,
            RefuseReason::Full => 4,
        }
    }
}

/// Behaviour when a human slot's client drops. `Neutral` is the game's own
/// behaviour for a vanished player (see module docs) and is the default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropPolicy {
    /// The dropped agent receives `NEUTRAL` input every tick; the match runs
    /// to its normal end conditions. Reconnect = real input resumes.
    Neutral,
    /// End the match for everyone.
    EndMatch,
}

/// Events the session reports to the app.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Host: the transport is bound and accepting HELLOs.
    Listening,
    /// Host: a client passed the handshake and took `slot`.
    ClientJoined { slot: u8, codename: String },
    /// Client: the host refused our HELLO.
    Refused { reason: RefuseReason },
    /// Client: handshake done; the match config is authoritative from here.
    Connected { slot: u8, seed: u32, settings: MatchSettings, input_delay: u32 },
    /// Both: the host started the match (host on `start_match`, client on GO).
    MatchStarted { first_tick: u32 },
    /// Both: the human-slot bitmask changed (join/leave/drop).
    SlotMapChanged { human_slots: [bool; MAX_AGENTS] },
    /// Both: tick `tick` cannot run — inputs missing for `missing` slots.
    Stalled { tick: u32, missing: Vec<u8> },
    /// Both: inputs arrived; tick `tick` can run again.
    Resumed { tick: u32 },
    /// Both: state hashes disagree at `tick`. Session keeps running.
    Desync { tick: u32, local: u64, remote: u64 },
    /// Both: a human slot's client left (timeout or graceful).
    ClientLeft { slot: u8, reason: LeaveReason },
    /// Client: no packets from the host for `Config::timeout_ms`.
    HostTimeout,
    /// Both: `DropPolicy::EndMatch` fired.
    MatchEnded { reason: LeaveReason },
    /// Both: an app-to-app blob arrived (the lobby's choices mirror). The
    /// session never inspects the bytes.
    Data { from_slot: u8, bytes: Vec<u8> },
    /// Host: a HELLO arrived after GO and the host began serving the run log
    /// to that peer (mp-join-in-progress).
    LateJoinStarted { slot: u8, codename: String },
    /// Late joiner: the host's log transfer is complete. `log` holds the
    /// host's recorded per-tick inputs for `first_tick..snapshot_tick`
    /// inclusive; the app replays it headless and must produce
    /// `snapshot_hash` at `snapshot_tick` before calling
    /// [`Session::confirm_late_join`] (a mismatch →
    /// [`Session::leave_with`] `JoinVerifyFailed`).
    LateJoinHandover {
        slot: u8,
        seed: u32,
        settings: MatchSettings,
        first_tick: u32,
        snapshot_tick: u32,
        snapshot_hash: u64,
        log: Vec<[Input; MAX_AGENTS]>,
    },
    /// All: `slot` becomes human from `join_tick` — its inputs start there;
    /// before it the slot rides NEUTRAL. The app spawns the slot's agent at
    /// `join_tick` the game's own MP way (`MP_GetSpawnPoint` →
    /// `Player_Init` [M 0x183a50:120-127]) when the slot had no body.
    LateJoinSlot { slot: u8, join_tick: u32 },
}

/// Session configuration. Defaults are the task's recommended values.
#[derive(Clone, Debug)]
pub struct Config {
    /// Handshake protocol version; mismatched clients are refused.
    pub protocol_version: u16,
    /// Handshake build id; mismatched clients are refused (mismatched exes).
    pub build_id: u32,
    /// Ticks between sampling an input and running it (2..=4 per the task;
    /// any value works). Sent to clients in WELCOME so both sides agree.
    pub input_delay: u32,
    /// Redundancy: every INPUT packet carries the last N inputs even if the
    /// peer's ack already covers them, so isolated packet loss cannot lose a
    /// tick (the unacked window is always carried in full as well).
    pub redundancy: u32,
    /// Exchange state hashes every N ticks (wire limit 255).
    pub hash_every: u32,
    /// No packets for this long = drop (host) / host timeout (client).
    pub timeout_ms: u64,
    /// Send a keepalive after this long with nothing else to send.
    pub keepalive_ms: u64,
    /// Client HELLO retransmit interval until WELCOME/REFUSE.
    pub hello_retry_ms: u64,
    /// Minimum interval between pure re-sends of an unchanged input window.
    pub resend_ms: u64,
    /// Dropped-client behaviour (see [`DropPolicy`]).
    pub drop_policy: DropPolicy,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            protocol_version: PROTOCOL_VERSION,
            build_id: crate::DEFAULT_BUILD_ID,
            input_delay: 3,
            redundancy: 4,
            hash_every: 30,
            timeout_ms: 5000,
            keepalive_ms: 250,
            hello_retry_ms: 400,
            resend_ms: 33,
            drop_policy: DropPolicy::Neutral,
        }
    }
}

/// Errors from explicit session calls.
#[derive(Debug)]
pub enum SessionError {
    NotHost,
    NotClient,
    AlreadyStarted,
    NotConnected,
}

struct Peer {
    addr: SocketAddr,
    slot: u8,
    /// The peer's lobby display codename (the HELLO payload; not compared —
    /// the version/build check is the compatibility gate).
    codename: String,
    connected: bool,
    last_recv_ms: u64,
    last_send_ms: u64,
    /// What the peer has acked of our input sends (contiguous tick).
    their_ack: Option<u32>,
    /// Highest contiguous tick WE have acked to this peer (the sliding base
    /// for our ack scan — recv_ticks may be pruned below this, so the scan
    /// must never restart from start_tick).
    our_ack: u32,
    /// Ticks we have received from the peer (for the ack we send them).
    recv_ticks: BTreeSet<u32>,
    remote_hashes: HashMap<u32, u64>,
    go_confirmed: bool,
    last_go_ms: u64,
    /// The late-join log transfer in flight (a HELLO after GO).
    late_serve: Option<LateServe>,
    /// This peer late-joined a running match (GO retransmit is suppressed —
    /// it never receives a GO; the JOIN_SLOT announcement arms it).
    late_joined: bool,
}

/// Host: the per-peer late-join transfer state.
struct LateServe {
    snapshot_tick: u32,
    snapshot_hash: u64,
    chunk_count: u16,
    sent: u16,
    acked: u16,
    last_send_ms: u64,
}

/// Client: the late-join log being received.
struct LateRecv {
    first_tick: u32,
    snapshot_tick: u32,
    snapshot_hash: u64,
    chunk_count: u16,
    chunks: Vec<Option<Vec<Input>>>,
}

/// One lobby agent as the app sees it (slot 0 = the host).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentInfo {
    pub slot: u8,
    pub codename: String,
}

enum Role {
    Host { settings: MatchSettings, seed: u32, peers: Vec<Peer> },
    Client { host: SocketAddr, slot: Option<u8>, hello_due_ms: u64, host_peer: Option<Peer> },
}

/// How many JOIN_LOG chunks stay in flight to a late joiner
/// (16 × [`proto::JOIN_LOG_CHUNK_TICKS`] ticks).
const JOIN_LOG_WINDOW: u16 = 16;
/// Ticks between the host's live head and the announced join tick: the
/// joiner — already caught up at JOIN_READY — schedules its inputs there and
/// every peer runs the join-spawn at that boundary.
const JOIN_MARGIN_TICKS: u32 = 90;

/// A lockstep session. See the module docs for the API contract.
pub struct Session<T: Transport> {
    pub(crate) transport: T,
    cfg: Config,
    role: Role,
    now_ms: u64,
    start_tick: u32,
    started: bool,
    /// Human slots: [host, client1, client2, client3]. Non-human slots get
    /// NEUTRAL inputs on the host; clients receive their inputs from the host.
    slot_human: [bool; MAX_AGENTS],
    /// Per slot, the tick from which the slot is human (mp-join-in-progress:
    /// a late joiner's slot flips human only at the announced join tick;
    /// before it, the slot rides NEUTRAL). 0 for the match-start humans.
    join_from: [u32; MAX_AGENTS],
    /// All agents' inputs by tick. Slot `local_slot` is filled by
    /// `local_input`; remote human slots by INPUT packets.
    inputs: [HashMap<u32, Input>; MAX_AGENTS],
    /// Host: the recorded per-tick inputs the app actually ran — the exact
    /// transcript a late joiner replays (`record_run_tick`, index
    /// `tick - log_base`). At 24 B × 4 slots × 60 Hz a 20-minute match holds
    /// ≈ 6.9 MB — the price of serving late joins.
    run_log: Vec<[Input; MAX_AGENTS]>,
    /// The tick `run_log[0]` holds (the match's first executed tick).
    log_base: u32,
    /// Client: the late-join transfer being received.
    late_recv: Option<LateRecv>,
    /// Client: the WELCOME's seed + settings (the late-join handover
    /// re-delivers them with the log).
    welcomed: Option<(u32, MatchSettings)>,
    /// Client: the late-join handover completed (the log arrived); arms
    /// [`Session::confirm_late_join`].
    late_joined: bool,
    /// Client: the JOIN_READY retransmit deadline (the confirm is unacked —
    /// it resends until the host's JOIN_SLOT answers it).
    late_confirm_due: Option<u64>,
    local_slot: u8,
    local_hashes: HashMap<u32, u64>,
    compared: HashSet<u32>,
    last_hash: Option<(u32, u64)>,
    last_ready: Option<u32>,
    stalled: bool,
    /// This machine's lobby display codename (slot 0's name on the host; the
    /// local slot's on a client).
    display_name: String,
    /// Stalled/Resumed are detected by the app's sequential `ready_inputs`
    /// queries and flushed as events on the next `poll`.
    pending_stall: Option<(u32, Vec<u8>)>,
    pending_resume: Option<u32>,
    keepalive_seq: u32,
    host_timed_out: bool,
    listen_emitted: bool,
    match_ended: bool,
    /// net-p2p: datagrams that are not session packets (STUN responses, hole punches) — kept for
    /// [`Session::take_stray`] so the reachability layer can share the game socket (its public
    /// mapping IS the game port's). Capped at [`STRAY_CAP`]; the oldest are dropped.
    stray: Vec<(SocketAddr, Vec<u8>)>,
}

/// net-p2p: at most this many un-taken stray datagrams are held [S port design].
pub const STRAY_CAP: usize = 64;

/// Human-slot initial state: slot 0 (the host) is human, every other slot
/// starts empty — the count comes from [`MAX_AGENTS`] [S mp-16-slots].
fn slot0_human() -> [bool; MAX_AGENTS] {
    let mut m = [false; MAX_AGENTS];
    m[0] = true;
    m
}

/// The all-agents bitmask over [`MAX_AGENTS`] (16 bits set [S mp-16-slots]).
fn all_agents_mask() -> u16 {
    ((1u32 << MAX_AGENTS) - 1) as u16
}

impl<T: Transport> Session<T> {
    /// Create the host. `settings` and `seed` are sent to every client in
    /// the handshake; the seed is the u32 fed to the game's own
    /// `Rng::new(seed)` (`crates/nf-game/src/drone.rs:394`) so every machine
    /// seeds the bit-exact RNG identically.
    pub fn new_host(cfg: Config, transport: T, settings: MatchSettings, seed: u32) -> Session<T> {
        Session {
            transport,
            cfg,
            role: Role::Host { settings, seed, peers: Vec::new() },
            now_ms: 0,
            start_tick: 0,
            started: false,
            slot_human: slot0_human(),
            join_from: [0; MAX_AGENTS],
            inputs: Default::default(),
            run_log: Vec::new(),
            log_base: 0,
            late_recv: None,
            welcomed: None,
            late_joined: false,
            late_confirm_due: None,
            local_slot: 0,
            local_hashes: HashMap::new(),
            compared: HashSet::new(),
            last_hash: None,
            last_ready: None,
            stalled: false,
            display_name: "player".to_string(),
            pending_stall: None,
            pending_resume: None,
            keepalive_seq: 0,
            host_timed_out: false,
            listen_emitted: false,
            match_ended: false,
            stray: Vec::new(),
        }
    }

    /// Create a client that will handshake with the host at `host`.
    pub fn new_client(cfg: Config, transport: T, host: SocketAddr) -> Session<T> {
        Session {
            transport,
            cfg,
            role: Role::Client { host, slot: None, hello_due_ms: 0, host_peer: None },
            now_ms: 0,
            start_tick: 0,
            started: false,
            slot_human: slot0_human(),
            join_from: [0; MAX_AGENTS],
            inputs: Default::default(),
            run_log: Vec::new(),
            log_base: 0,
            late_recv: None,
            welcomed: None,
            late_joined: false,
            late_confirm_due: None,
            local_slot: 0,
            local_hashes: HashMap::new(),
            compared: HashSet::new(),
            last_hash: None,
            last_ready: None,
            stalled: false,
            display_name: "player".to_string(),
            pending_stall: None,
            pending_resume: None,
            keepalive_seq: 0,
            host_timed_out: false,
            listen_emitted: false,
            match_ended: false,
            stray: Vec::new(),
        }
    }

    pub fn local_slot(&self) -> u8 {
        self.local_slot
    }

    /// Set this machine's lobby display codename (rides HELLO / WELCOME;
    /// never compared — the compatibility gate is protocol + build).
    pub fn set_display_name(&mut self, name: &str) {
        self.display_name = name.chars().take(31).collect();
    }

    /// The lobby view: this machine's agent plus every peer, by slot. The
    /// match-host layer's `NetAgent` port (its socket address stays internal).
    pub fn agents(&self) -> Vec<AgentInfo> {
        let mut out = vec![AgentInfo { slot: self.local_slot, codename: self.display_name.clone() }];
        match &self.role {
            Role::Host { peers, .. } => {
                for p in peers.iter().filter(|p| p.connected) {
                    out.push(AgentInfo { slot: p.slot, codename: p.codename.clone() });
                }
            }
            Role::Client { host_peer, .. } => {
                if let Some(p) = host_peer {
                    out.push(AgentInfo { slot: 0, codename: p.codename.clone() });
                }
            }
        }
        out.sort_by_key(|a| a.slot);
        out
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Record the local agent's input for `tick`. Idempotent per tick.
    /// Ignored until the client's slot is assigned (host: always slot 0) —
    /// the app samples controls only after `Event::Connected`/`MatchStarted`,
    /// so nothing is lost.
    /// The match's input lead: the app submits the input sampled while running tick `t` for
    /// tick `t + input_delay()` (the host's config; a client takes it from the WELCOME).
    pub fn input_delay(&self) -> u32 {
        self.cfg.input_delay
    }

    pub fn local_input(&mut self, tick: u32, input: Input) {
        if let Role::Client { slot: None, .. } = &self.role {
            return;
        }
        self.inputs[self.local_slot as usize].insert(tick, input);
    }

    /// Host: record the per-tick inputs a tick actually RAN with (the app
    /// calls this right after `ready_inputs` returned `Some`, before stepping
    /// the sim). The recorded log is the exact transcript a late joiner
    /// replays — bit-exact by construction. Host-only; ticks must arrive in
    /// ascending order (a gap is filled with NEUTRAL and cannot verify — the
    /// snapshot walk stops at the log's end).
    pub fn record_run_tick(&mut self, tick: u32, inputs: [Input; MAX_AGENTS]) {
        if !matches!(self.role, Role::Host { .. }) {
            return;
        }
        if self.run_log.is_empty() {
            self.log_base = tick;
        }
        let idx = (tick.wrapping_sub(self.log_base)) as usize;
        if idx == self.run_log.len() {
            self.run_log.push(inputs);
        } else if idx < self.run_log.len() {
            self.run_log[idx] = inputs; // re-recorded tick: the latest wins
        } else {
            while self.run_log.len() < idx {
                self.run_log.push([NEUTRAL; MAX_AGENTS]);
            }
            self.run_log.push(inputs);
        }
    }

    /// Late joiner: the replay verified (`snapshot_hash` reproduced at
    /// `snapshot_tick`) — tell the host to pick the join tick. The confirm is
    /// unacked: it retransmits every `hello_retry_ms` until the host's
    /// JOIN_SLOT answers it (loss-safe).
    pub fn confirm_late_join(&mut self) {
        if self.late_joined {
            self.late_confirm_due = Some(self.now_ms);
        }
    }

    /// Leave the session with an explicit reason (the late joiner reports a
    /// failed verification — the host surfaces it verbatim).
    pub fn leave_with(&mut self, reason: LeaveReason) {
        let pkt = Packet::Leave { slot: self.local_slot, reason: reason.to_wire() };
        self.send_to_peers(&pkt);
    }

    fn host_addr(&self) -> SocketAddr {
        match &self.role {
            Role::Client { host, .. } => *host,
            Role::Host { .. } => "127.0.0.1:0".parse().unwrap(),
        }
    }

    /// Report the local state hash for `tick` (after running it). Hashes are
    /// exchanged every `hash_every` ticks; a mismatch with a peer's hash for
    /// the same tick emits [`Event::Desync`].
    pub fn submit_hash(&mut self, tick: u32, hash: u64) {
        let due = tick % self.cfg.hash_every.max(1) == 0;
        self.local_hashes.insert(tick, hash);
        if due {
            let prev = self.last_hash.unwrap_or((tick, 0));
            self.last_hash = Some((tick, hash));
            let pkt = Packet::Hash { tick, hash, prev_tick: prev.0, prev_hash: prev.1 };
            self.send_to_peers(&pkt);
        }
    }

    /// The 4 agents' inputs for `tick`, or `None` while any required human
    /// input is missing (stall — never guess). Non-human slots are NEUTRAL
    /// on the host; a slot whose `join_from` is in the future (a late joiner
    /// not yet armed) is NEUTRAL too, on every role.
    ///
    /// Call this SEQUENTIALLY (start_tick, start_tick+1, ...) as the sim
    /// advances: the sequence drives `Event::Stalled` / `Event::Resumed`
    /// (flushed on the next `poll`).
    pub fn ready_inputs(&mut self, tick: u32) -> Option<[Input; MAX_AGENTS]> {
        if !self.started {
            return None;
        }
        let mut out = [NEUTRAL; MAX_AGENTS];
        let mut complete = true;
        for s in 0..MAX_AGENTS {
            let required = self.slot_human[s] && tick >= self.join_from[s];
            let v = if s == self.local_slot as usize {
                // The local slot rides NEUTRAL before its join tick too (a
                // late joiner produces no inputs until the announced tick).
                if required {
                    self.inputs[s].get(&tick).copied()
                } else {
                    Some(NEUTRAL)
                }
            } else if !required {
                Some(NEUTRAL)
            } else {
                self.inputs[s].get(&tick).copied()
            };
            match v {
                Some(x) => out[s] = x,
                None => complete = false,
            }
        }
        if complete {
            if self.stalled {
                self.stalled = false;
                self.pending_resume = Some(tick);
            }
            self.last_ready = Some(tick);
            Some(out)
        } else {
            if !self.stalled {
                self.stalled = true;
                let missing = self.missing_slots(tick);
                self.pending_stall = Some((tick, missing));
            }
            None
        }
    }

    /// Slots missing an input for `tick` (for `Event::Stalled` diagnostics).
    fn missing_slots(&self, tick: u32) -> Vec<u8> {
        let mut miss = Vec::new();
        for s in 0..MAX_AGENTS {
            let required = self.slot_human[s] && tick >= self.join_from[s];
            let have = if s == self.local_slot as usize {
                !required || self.inputs[s].contains_key(&tick)
            } else if !required {
                true
            } else {
                self.inputs[s].contains_key(&tick)
            };
            if !have {
                miss.push(s as u8);
            }
        }
        miss
    }

    /// Host: set (or replace) the match settings + seed clients receive in
    /// WELCOME. The lobby arms the match AFTER clients joined (the confirm
    /// page runs once the panels are full), so the constructor-time payload
    /// must be replaceable — the re-send path (a re-HELLO's idempotent
    /// WELCOME) then carries the real match, not the placeholder.
    /// Host-only and only before GO; later calls are ignored.
    pub fn set_host_match(&mut self, settings: MatchSettings, seed: u32) {
        if self.started {
            return;
        }
        if let Role::Host { settings: s, seed: sd, .. } = &mut self.role {
            *s = settings;
            *sd = seed;
        }
    }

    /// Host: start the match at `first_tick` and tell every connected client.
    pub fn start_match(&mut self, first_tick: u32) -> Result<(), SessionError> {
        if self.started {
            return Err(SessionError::AlreadyStarted);
        }
        let addrs: Vec<SocketAddr> = match &mut self.role {
            Role::Host { peers, .. } => {
                self.started = true;
                self.start_tick = first_tick;
                peers.iter().filter(|p| p.connected).map(|p| p.addr).collect()
            }
            Role::Client { .. } => return Err(SessionError::NotHost),
        };
        let pkt = Packet::Go { first_tick };
        for a in addrs {
            self.send_packet(a, &pkt);
            if let Role::Host { peers, .. } = &mut self.role {
                if let Some(p) = peers.iter_mut().find(|p| p.addr == a) {
                    p.last_go_ms = self.now_ms;
                }
            }
        }
        Ok(())
    }

    /// Leave the session (send LEAVE, stop participating).
    pub fn leave(&mut self) {
        let pkt = Packet::Leave { slot: self.local_slot, reason: LeaveReason::Graceful.to_wire() };
        self.send_to_peers(&pkt);
    }

    /// Send an opaque app-to-app blob to every connected peer (the lobby's
    /// choices mirror rides this). Peers see [`Event::Data`].
    pub fn send_data(&mut self, bytes: Vec<u8>) {
        let pkt = Packet::Data { bytes };
        self.send_to_peers(&pkt);
    }

    /// net-p2p: send a raw datagram from the session's own socket (STUN binding requests, hole
    /// punches) — the NAT mapping the peer must reach is this socket's.
    pub fn send_raw(&mut self, to: SocketAddr, data: &[u8]) {
        self.transport.send(to, data);
    }

    /// net-p2p: the datagrams `poll` received that were not session packets, oldest first.
    pub fn take_stray(&mut self) -> Vec<(SocketAddr, Vec<u8>)> {
        std::mem::take(&mut self.stray)
    }

    /// The local bind address of the session's socket.
    pub fn local_addr(&self) -> SocketAddr {
        self.transport.local_addr()
    }

    // ---------- internals ----------
    fn send_to_peers(&mut self, pkt: &Packet) {
        let mut buf = Vec::new();
        proto::encode(pkt, &mut buf);
        match &mut self.role {
            Role::Host { peers, .. } => {
                for p in peers.iter() {
                    if p.connected {
                        self.transport.send(p.addr, &buf);
                    }
                }
            }
            Role::Client { host, .. } => {
                self.transport.send(*host, &buf);
            }
        }
    }

    fn send_packet(&mut self, to: SocketAddr, pkt: &Packet) {
        let mut buf = Vec::new();
        proto::encode(pkt, &mut buf);
        self.transport.send(to, &buf);
    }

    fn human_mask(&self) -> u16 {
        let mut m = 0u16;
        for (i, h) in self.slot_human.iter().enumerate() {
            if *h {
                m |= 1 << i;
            }
        }
        m
    }

    /// Pump: drain the network, run timers, return the events.
    pub fn poll(&mut self, now_ms: u64) -> Vec<Event> {
        let mut events = Vec::new();
        self.now_ms = now_ms;
        if !self.listen_emitted {
            self.listen_emitted = true;
            events.push(Event::Listening);
        }
        // 1. Drain the network.
        while let Some((from, data)) = self.transport.try_recv() {
            if let Some(pkt) = proto::decode(&data) {
                self.handle_packet(from, pkt, &mut events);
            } else {
                if self.stray.len() >= STRAY_CAP { self.stray.remove(0); }
                self.stray.push((from, data));
            }
        }
        // 2. Client HELLO retransmit until WELCOME/REFUSE; the late-join
        //    JOIN_READY retransmits until the host's JOIN_SLOT answers it.
        if let Role::Client { slot, hello_due_ms, .. } = &self.role {
            if slot.is_none() && self.cfg.hello_retry_ms > 0 && now_ms >= *hello_due_ms {
                let pkt = Packet::Hello {
                    proto: self.cfg.protocol_version,
                    build: self.cfg.build_id,
                    codename: self.display_name.clone(),
                };
                let host = match &self.role {
                    Role::Client { host, .. } => *host,
                    _ => unreachable!(),
                };
                self.send_packet(host, &pkt);
                if let Role::Client { hello_due_ms, .. } = &mut self.role {
                    *hello_due_ms = now_ms + self.cfg.hello_retry_ms;
                }
            }
        }
        // 3. Per-peer work (input windows, GO retransmit, keepalives).
        self.pump_peers(now_ms);
        // 3b. The late-join confirm retransmit (client).
        if let Some(due) = self.late_confirm_due {
            if now_ms >= due {
                self.send_packet(self.host_addr(), &Packet::JoinReady {});
                self.late_confirm_due = Some(now_ms + self.cfg.hello_retry_ms.max(1));
            }
        }
        // 4. Timeouts.
        self.check_timeouts(now_ms, &mut events);
        // 5. Stall/resume transitions.
        self.detect_stall(&mut events);
        events
    }

    fn pump_peers(&mut self, now_ms: u64) {
        struct PeerSnap {
            addr: SocketAddr,
            slot: u8,
            their_ack: Option<u32>,
            our_ack: u32,
            recv_ticks: Vec<u32>,
            idle_for_ms: u64,
            late_serving: bool,
        }
        // Phase 1: snapshot peers and build packets against &self.
        let snaps: Vec<PeerSnap> = match &self.role {
            Role::Host { peers, .. } => peers
                .iter()
                .filter(|p| p.connected)
                .map(|p| PeerSnap {
                    addr: p.addr,
                    slot: p.slot,
                    their_ack: p.their_ack,
                    our_ack: p.our_ack,
                    recv_ticks: p.recv_ticks.iter().copied().collect(),
                    idle_for_ms: now_ms.saturating_sub(p.last_send_ms),
                    late_serving: p.late_serve.is_some(),
                })
                .collect(),
            Role::Client { host_peer, .. } => host_peer
                .as_ref()
                .map(|p| {
                    vec![PeerSnap {
                        addr: p.addr,
                        slot: p.slot,
                        their_ack: p.their_ack,
                        our_ack: p.our_ack,
                        recv_ticks: p.recv_ticks.iter().copied().collect(),
                        idle_for_ms: now_ms.saturating_sub(p.last_send_ms),
                        late_serving: false,
                    }]
                })
                .unwrap_or_default(),
        };
        let mut plans: Vec<(SocketAddr, Packet)> = Vec::new();
        for s in &snaps {
            // A late-join transfer in flight suppresses the live input relay
            // to that peer: the log covers the history, and the relay's ack
            // base resumes at the snapshot on handover.
            let serving = s.late_serving;
            if self.started && !serving {
                if let Some(pkt) = self.build_input_packet(s.slot, s.their_ack, s.our_ack, &s.recv_ticks) {
                    if let Packet::Input { ack_tick, .. } = &pkt {
                        if let Role::Host { peers, .. } = &mut self.role {
                            if let Some(p) = peers.iter_mut().find(|p| p.addr == s.addr) {
                                if *ack_tick != u32::MAX {
                                    p.our_ack = *ack_tick;
                                }
                            }
                        } else if let Role::Client { host_peer, .. } = &mut self.role {
                            if let Some(p) = host_peer {
                                if *ack_tick != u32::MAX {
                                    p.our_ack = *ack_tick;
                                }
                            }
                        }
                    }
                    plans.push((s.addr, pkt));
                }
            }
            if s.idle_for_ms >= self.cfg.keepalive_ms {
                plans.push((s.addr, Packet::Keepalive { seq: self.keepalive_seq }));
            }
        }
        if !plans.is_empty() {
            self.keepalive_seq = self.keepalive_seq.wrapping_add(1);
        }
        // Late-join log chunks (host): the recorded run log rides JOIN_LOG
        // chunks, a windowed transfer re-sent on the `resend_ms` cadence
        // until the JOIN_ACKs advance (loss-safe; the ack completes it).
        if let Role::Host { peers, .. } = &mut self.role {
            for p in peers.iter_mut() {
                if !p.connected {
                    continue;
                }
                let Some(l) = p.late_serve.as_mut() else { continue };
                if l.sent >= l.chunk_count
                    && now_ms.saturating_sub(l.last_send_ms) < self.cfg.resend_ms
                {
                    continue; // everything sent; only un-acked resends remain
                }
                let window_end = (l.acked.saturating_add(JOIN_LOG_WINDOW)).min(l.chunk_count);
                let from = l.acked.min(l.sent);
                for idx in from..window_end {
                    let start = idx as usize * proto::JOIN_LOG_CHUNK_TICKS;
                    let end = (start + proto::JOIN_LOG_CHUNK_TICKS).min(self.run_log.len());
                    if start >= end {
                        break;
                    }
                    let inputs: Vec<Input> =
                        self.run_log[start..end].iter().flat_map(|r| r.iter().copied()).collect();
                    plans.push((p.addr, Packet::JoinLog {
                        first_tick: self.log_base,
                        tick_count: (end - start) as u8,
                        chunk_index: idx,
                        chunk_count: l.chunk_count,
                        inputs,
                    }));
                }
                l.sent = l.sent.max(window_end);
                l.last_send_ms = now_ms;
            }
        }
        // GO retransmit (host, until each client's inputs prove it started).
        if self.started {
            let go_addrs: Vec<SocketAddr> = match &self.role {
                Role::Host { peers, .. } => peers
                    .iter()
                    .filter(|p| p.connected && !p.go_confirmed
                        && now_ms.saturating_sub(p.last_go_ms) >= self.cfg.resend_ms)
                    .map(|p| p.addr)
                    .collect(),
                _ => Vec::new(),
            };
            for a in go_addrs {
                plans.push((a, Packet::Go { first_tick: self.start_tick }));
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(p) = peers.iter_mut().find(|p| p.addr == a) {
                        p.last_go_ms = now_ms;
                    }
                }
            }
        }
        // Phase 2: send, then mark peers as sent-to.
        for (addr, pkt) in plans {
            self.send_packet(addr, &pkt);
        }
        match &mut self.role {
            Role::Host { peers, .. } => {
                for p in peers.iter_mut() {
                    if p.connected && now_ms.saturating_sub(p.last_send_ms) >= self.cfg.keepalive_ms {
                        p.last_send_ms = now_ms;
                    }
                }
            }
            Role::Client { host_peer, .. } => {
                if let Some(p) = host_peer {
                    if now_ms.saturating_sub(p.last_send_ms) >= self.cfg.keepalive_ms {
                        p.last_send_ms = now_ms;
                    }
                }
            }
        }
    }

    /// Build the INPUT packet for a peer. Host: everything that client needs
    /// (all slots but its own), from its ack cursor up to the newest
    /// fully-ready tick. Client: the client's own inputs from the host's ack
    /// cursor up to the newest sampled input.
    fn build_input_packet(&self, peer_slot: u8, their_ack: Option<u32>, our_ack: u32, recv_ticks: &[u32]) -> Option<Packet> {
        let (mask, newest): (u16, u32) = match &self.role {
            Role::Host { .. } => {
                let newest = self.last_ready?;
                // Every agent but the receiver's own slot (the receiver knows
                // its own input) — 15 of the 16 bits [S mp-16-slots; the game
                // relayed 3 of 4, mask 0b1111 & !(1 << peer_slot)].
                let mask = all_agents_mask() & !(1u16 << peer_slot);
                (mask, newest)
            }
            Role::Client { .. } => {
                let slot = self.local_slot as usize;
                let newest = match self.inputs[slot].keys().max() {
                    Some(n) => *n,
                    None => {
                        // A late joiner before its join tick has no own
                        // inputs yet — its acks still need a carrier, so the
                        // packet rides its own slot's truthful NEUTRAL (the
                        // agent does not exist before the join tick; the
                        // host's join_from gate keeps it out of the sim).
                        let lr = self.last_ready?;
                        if !self.slot_human[slot] || lr < self.join_from[slot] {
                            lr
                        } else {
                            return None;
                        }
                    }
                };
                (1u16 << slot, newest)
            }
        };
        let per_tick = mask.count_ones() as usize;
        if per_tick == 0 {
            return None;
        }
        let cap = if let Role::Host { .. } = self.role {
            proto::MAX_INPUT_COUNT_H2C
        } else {
            proto::MAX_INPUT_COUNT_C2H
        };
        let floor = if let Role::Host { .. } = self.role {
            self.start_tick
        } else {
            self.start_tick.min(newest)
        };
        // Send window (see the module docs): the full unacked span when it
        // fits the packet; when it overflows the cap, the OLDEST unacked
        // ticks go first with the window END truncated — the peer's ack can
        // always advance, so nothing is ever silently skipped; plus
        // redundancy reach-back (the last N inputs ride along) when there
        // is room under the cap.
        let n = (self.cfg.redundancy as usize).max(1);
        let unacked_from = match their_ack {
            Some(a) if a.wrapping_add(1) > floor => a.wrapping_add(1),
            _ => floor,
        };
        if newest < unacked_from {
            return None; // nothing unacked; keepalives cover liveness
        }
        let (start, end) = {
            let span = newest.wrapping_sub(unacked_from) as usize + 1;
            if span > cap {
                let end = unacked_from.wrapping_add(cap as u32 - 1).min(newest);
                (unacked_from, end)
            } else {
                let reach = ((n - 1) as u32).min(cap as u32 - span as u32);
                let start = unacked_from.saturating_sub(reach).max(floor);
                (start, newest)
            }
        };
        let (ack_tick, ack_bits) = self.ack_for(recv_ticks, our_ack);
        let count = (end.wrapping_sub(start) + 1) as usize;
        let mut inputs = Vec::with_capacity(count * per_tick);
        for t in start..=end {
            for s in 0..MAX_AGENTS {
                if mask & (1u16 << s) == 0 {
                    continue;
                }
                match self.slot_input_for_send(s, t) {
                    Some(v) => inputs.push(v),
                    None => return None,
                }
            }
        }
        Some(Packet::Input { base_tick: start, agent_mask: mask, ack_tick, ack_bits, inputs })
    }

    /// Input to broadcast for slot `s` at tick `t` (host materialises NEUTRAL
    /// for non-human slots AND for a human slot whose join tick is still in
    /// the future — the late joiner produces no inputs before it; that
    /// substitution is authoritative and reaches every client in the
    /// broadcast). The LOCAL slot rides the same substitution: a late
    /// joiner's own slot is truthfully NEUTRAL until its join tick (the
    /// agent does not exist yet), which also gives its acks a carrier.
    fn slot_input_for_send(&self, s: usize, t: u32) -> Option<Input> {
        if s == self.local_slot as usize {
            // A late joiner whose join tick is not announced yet (join_from still 0) is
            // NEUTRAL too — without this the packet had no input to carry and no acks went out.
            let unannounced = self.late_joined && self.join_from[s] == 0;
            if !self.slot_human[s] || t < self.join_from[s] || unannounced {
                return Some(NEUTRAL);
            }
            return self.inputs[s].get(&t).copied();
        }
        if let Role::Host { .. } = self.role {
            if !self.slot_human[s] || t < self.join_from[s] {
                return Some(NEUTRAL);
            }
        }
        self.inputs[s].get(&t).copied()
    }

    /// The ack we send for the ticks a peer sent us: highest contiguous tick
    /// from our sliding base `our_ack`, plus a 32-tick bitmap past it
    /// (u32::MAX = nothing acked yet). The base slides with the ack so old
    /// pruned ticks can never break contiguity.
    fn ack_for(&self, recv_ticks: &[u32], our_ack: u32) -> (u32, u32) {
        let mut t = our_ack;
        while recv_ticks.contains(&t) {
            t = t.wrapping_add(1);
        }
        let ack_tick = t.wrapping_sub(1);
        let mut ack_bits = 0u32;
        for i in 0..32u32 {
            if recv_ticks.contains(&ack_tick.wrapping_add(1).wrapping_add(i)) {
                ack_bits |= 1 << i;
            }
        }
        (ack_tick, ack_bits)
    }

    fn handle_packet(&mut self, from: SocketAddr, pkt: Packet, events: &mut Vec<Event>) {
        match &self.role {
            Role::Host { .. } => self.handle_packet_host(from, pkt, events),
            Role::Client { .. } => self.handle_packet_client(from, pkt, events),
        }
    }

    fn handle_packet_host(&mut self, from: SocketAddr, pkt: Packet, events: &mut Vec<Event>) {
        match pkt {
            Packet::Hello { proto, build, codename } => {
                let refuse = if proto != self.cfg.protocol_version {
                    Some(RefuseReason::Version)
                } else if build != self.cfg.build_id {
                    Some(RefuseReason::Build)
                } else {
                    None
                };
                if let Some(reason) = refuse {
                    self.send_packet(from, &Packet::Refuse { reason: reason.to_wire() });
                    return;
                }
                if self.started {
                    // mp-join-in-progress: a HELLO after GO asks to join the
                    // running match — the host serves the recorded run log,
                    // the joiner replays + verifies, and the JOIN_SLOT
                    // announcement arms the new slot at an agreed tick.
                    self.begin_late_join(from, codename, events);
                    return;
                }
                // Already a peer? Retransmit WELCOME (idempotent).
                let existing: Option<u8> = match &self.role {
                    Role::Host { peers, .. } => {
                        peers.iter().find(|p| p.addr == from).map(|p| {
                            p.slot
                        })
                    }
                    _ => return,
                };
                if let Some(slot) = existing {
                    if let Role::Host { peers, .. } = &mut self.role {
                        if let Some(p) = peers.iter_mut().find(|p| p.addr == from) {
                            p.last_recv_ms = self.now_ms;
                        }
                    }
                    self.send_welcome_to(from, slot);
                    return;
                }
                // Free slot? Host is slot 0; clients take 1..=3.
                let free: Option<u8> = match &self.role {
                    Role::Host { peers, .. } => {
                        let used: Vec<u8> = peers.iter().map(|p| p.slot).collect();
                        (1..MAX_AGENTS as u8).find(|s| !used.contains(s))
                    }
                    _ => return,
                };
                let slot = match free {
                    Some(s) => s,
                    None => {
                        self.send_packet(from, &Packet::Refuse { reason: RefuseReason::Full.to_wire() });
                        return;
                    }
                };
                if let Role::Host { peers, .. } = &mut self.role {
                    peers.push(Peer {
                        addr: from,
                        slot,
                        codename: codename.chars().take(31).collect(),
                        connected: true,
                        last_recv_ms: self.now_ms,
                        last_send_ms: self.now_ms,
                        their_ack: None,
                        our_ack: 0,
                        recv_ticks: BTreeSet::new(),
                        remote_hashes: HashMap::new(),
                        go_confirmed: false,
                        last_go_ms: 0,
                        late_serve: None,
                        late_joined: false,
                    });
                    self.slot_human[slot as usize] = true;
                }
                self.send_welcome_to(from, slot);
                events.push(Event::ClientJoined { slot, codename: codename.chars().take(31).collect() });
                self.broadcast_slotmap();
            }
            Packet::Input { base_tick, agent_mask, ack_tick, ack_bits, inputs } => {
                let peer_slot = match &mut self.role {
                    Role::Host { peers, .. } => {
                        let idx = peers.iter().position(|p| p.addr == from);
                        match idx {
                            Some(i) => {
                                let p = &mut peers[i];
                                p.last_recv_ms = self.now_ms;
                                p.go_confirmed = true;
                                p.their_ack = if ack_tick == u32::MAX { None } else { Some(ack_tick) };
                                let per_tick = agent_mask.count_ones().max(1) as usize;
                                for k in 0..inputs.len() / per_tick.max(1) {
                                    p.recv_ticks.insert(base_tick.wrapping_add(k as u32));
                                }
                                peers[i].slot
                            }
                            None => return,
                        }
                    }
                    _ => return,
                };
                // Store the inputs (mask must not contain the sender's own slot).
                let per_tick = agent_mask.count_ones().max(1) as usize;
                for k in 0..inputs.len() / per_tick.max(1) {
                    let t = base_tick.wrapping_add(k as u32);
                    for (j, s) in (0..MAX_AGENTS as u8).filter(|s| agent_mask & (1u16 << s) != 0).enumerate() {
                        if s as usize == self.local_slot as usize {
                            continue;
                        }
                        let idx = k * per_tick + j;
                        if let Some(v) = inputs.get(idx) {
                            self.inputs[s as usize].insert(t, *v);
                        }
                    }
                }
                let _ = peer_slot;
            }
            Packet::Hash { tick, hash, prev_tick, prev_hash } => {
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(p) = peers.iter_mut().find(|p| p.addr == from) {
                        p.last_recv_ms = self.now_ms;
                        p.remote_hashes.insert(tick, hash);
                        // prev_tick == tick marks "no previous hash yet".
                        if prev_tick != tick {
                            p.remote_hashes.insert(prev_tick, prev_hash);
                        }
                    }
                }
            }
            Packet::Keepalive { .. } => {
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(p) = peers.iter_mut().find(|p| p.addr == from) {
                        p.last_recv_ms = self.now_ms;
                    }
                }
            }
            Packet::Data { bytes } => {
                let slot = match &mut self.role {
                    Role::Host { peers, .. } => match peers.iter_mut().find(|p| p.addr == from) {
                        Some(p) => {
                            p.last_recv_ms = self.now_ms;
                            p.slot
                        }
                        None => return,
                    },
                    Role::Client { host_peer, .. } => match host_peer {
                        Some(p) => {
                            p.last_recv_ms = self.now_ms;
                            p.slot
                        }
                        None => return,
                    },
                };
                events.push(Event::Data { from_slot: slot, bytes });
            }
            Packet::Leave { slot, reason } => {
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(i) = peers.iter().position(|p| p.addr == from) {
                        let p = peers.remove(i);
                        let reason = LeaveReason::from_wire(reason);
                        self.drop_slot(p.slot, reason, events);
                    }
                }
                let _ = slot;
            }
            Packet::JoinAck { chunk_index } => {
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(p) = peers.iter_mut().find(|p| p.addr == from) {
                        p.last_recv_ms = self.now_ms;
                        if let Some(l) = p.late_serve.as_mut() {
                            l.acked = l.acked.max(chunk_index.saturating_add(1));
                            if l.acked >= l.chunk_count {
                                p.late_serve = None; // transfer complete
                            }
                        }
                    }
                }
            }
            Packet::JoinReady {} => {
                // The joiner's replay verified and it is caught up: pick the
                // join tick (the live head + a margin), arm the slot from
                // there on EVERY machine (the JOIN_SLOT broadcast), and seed
                // the ack base — the joiner's inputs start at the join tick,
                // so everything before it is implicitly covered.
                let host_head = self.last_ready.unwrap_or(0);
                let join_tick = host_head.saturating_add(JOIN_MARGIN_TICKS);
                let mut announce: Option<(SocketAddr, u8, u32)> = None;
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(p) = peers.iter_mut().find(|p| p.addr == from) {
                        if p.late_serve.is_none() && p.late_joined && (p.slot as usize) < MAX_AGENTS {
                            p.last_recv_ms = self.now_ms;
                            p.our_ack = join_tick;
                            self.slot_human[p.slot as usize] = true;
                            self.join_from[p.slot as usize] = join_tick;
                            announce = Some((p.addr, p.slot, join_tick));
                        }
                    }
                }
                if let Some((addr, slot, join_tick)) = announce {
                    self.send_packet(addr, &Packet::JoinSlot { slot, join_tick });
                    self.send_to_peers(&Packet::JoinSlot { slot, join_tick });
                    events.push(Event::LateJoinSlot { slot, join_tick });
                }
            }
            _ => {}
        }
    }

    /// Host: a HELLO after GO — begin the late-join handover: a free slot,
    /// a WELCOME (settings + seed), then the recorded run log up to the
    /// newest verifiable snapshot tick (the log's end where the app reported
    /// a hash). The transfer itself is chunked (JOIN_LOG) and acked
    /// (JOIN_ACK); the joiner's verified confirm arrives as JOIN_READY.
    fn begin_late_join(&mut self, from: SocketAddr, codename: String, events: &mut Vec<Event>) {
        // A RETRANSMITTED HELLO from a peer already being served (the joiner
        // retries every hello_retry_ms until the WELCOME lands — under load
        // the first WELCOME can still be in flight) must not double-allocate
        // a slot: answer the WELCOME again and leave the transfer state alone
        // (mp-16-slots: the 9-human loopback hit the duplicate — the roster
        // grew a phantom slot 10).
        if let Role::Host { peers, .. } = &self.role {
            if let Some(p) = peers.iter().find(|p| p.addr == from) {
                self.send_welcome_to(from, p.slot);
                return;
            }
        }
        // Free slot? Host is slot 0; late joiners take 1..=MAX_AGENTS.
        let free: Option<u8> = match &self.role {
            Role::Host { peers, .. } => {
                let used: Vec<u8> = peers.iter().map(|p| p.slot).collect();
                (1..MAX_AGENTS as u8).find(|s| !used.contains(s))
            }
            _ => return,
        };
        let slot = match free {
            Some(s) => s,
            None => {
                // Max players: the game's own MP limit is 4 human slots
                // (`MP_Start` 0x183a50 inits humans 0..4 [M]).
                self.send_packet(from, &Packet::Refuse { reason: RefuseReason::Full.to_wire() });
                return;
            }
        };
        // The newest tick the log covers AND the app reported a hash for:
        // the replay must be verifiable, so the snapshot walks back from the
        // head to the last hash (the app submits every executed tick, so
        // this is the head in practice).
        let log_end = self.log_base + self.run_log.len() as u32;
        let mut snap = self.last_ready.unwrap_or(0).min(log_end);
        while snap > self.log_base && !self.local_hashes.contains_key(&snap) {
            snap -= 1;
        }
        let hash = self.local_hashes.get(&snap).copied();
        let Some(snapshot_hash) = hash else {
            // Nothing verifiable: keep the old behaviour (refuse).
            self.send_packet(from, &Packet::Refuse { reason: RefuseReason::Started.to_wire() });
            return;
        };
        let first_tick = self.log_base;
        let ticks = snap.saturating_sub(first_tick) as usize + 1; // inclusive
        let chunk_count = ((ticks + proto::JOIN_LOG_CHUNK_TICKS - 1) / proto::JOIN_LOG_CHUNK_TICKS) as u16;
        if let Role::Host { peers, .. } = &mut self.role {
            peers.push(Peer {
                addr: from,
                slot,
                codename: codename.chars().take(31).collect(),
                connected: true,
                last_recv_ms: self.now_ms,
                last_send_ms: self.now_ms,
                // The joiner holds the whole log through the snapshot: the
                // live relay resumes at snapshot+1 (its own acks cannot say
                // so yet — it sends no inputs before its join tick).
                their_ack: Some(snap),
                our_ack: 0,
                recv_ticks: BTreeSet::new(),
                remote_hashes: HashMap::new(),
                go_confirmed: true, // never a GO for a late joiner
                last_go_ms: 0,
                late_serve: Some(LateServe { snapshot_tick: snap, snapshot_hash, chunk_count, sent: 0, acked: 0, last_send_ms: 0 }),
                late_joined: true,
            });
            // The slot does NOT flip human here: everyone keeps riding
            // NEUTRAL for it until the JOIN_SLOT announcement.
        }
        self.send_welcome_to(from, slot);
        // The current human mask: the joiner's session starts [host] only and
        // must know which slots are live humans (its own slot stays out — it
        // is not human until the JOIN_SLOT announcement).
        self.send_packet(from, &Packet::SlotMap { human_slots: self.human_mask() });
        self.send_packet(from, &Packet::JoinBegin { first_tick, snapshot_tick: snap, snapshot_hash });
        events.push(Event::LateJoinStarted { slot, codename: codename.chars().take(31).collect() });
    }

    fn send_welcome_to(&mut self, addr: SocketAddr, slot: u8) {
        let (settings, seed) = match &self.role {
            Role::Host { settings, seed, .. } => (settings.clone(), *seed),
            _ => return,
        };
        let pkt = Packet::Welcome {
            proto: self.cfg.protocol_version,
            build: self.cfg.build_id,
            codename: self.display_name.clone(),
            slot,
            input_delay: self.cfg.input_delay.min(255) as u8,
            hash_every: self.cfg.hash_every.min(255) as u8,
            seed,
            settings,
        };
        self.send_packet(addr, &pkt);
    }

    /// Tell everyone which slots are human (SLOTMAP), clients update.
    fn broadcast_slotmap(&mut self) {
        let pkt = Packet::SlotMap { human_slots: self.human_mask() };
        self.send_to_peers(&pkt);
    }

    fn handle_packet_client(&mut self, from: SocketAddr, pkt: Packet, events: &mut Vec<Event>) {
        let host = match &self.role {
            Role::Client { host, .. } => *host,
            _ => return,
        };
        if from != host {
            return; // clients talk to the host only
        }
        match pkt {
            Packet::Welcome { proto, build, codename, slot, input_delay, hash_every, seed, settings } => {
                if let Role::Client { slot: s, host_peer, .. } = &mut self.role {
                    if s.is_some() {
                        return; // already connected
                    }
                    let ok = proto == self.cfg.protocol_version && build == self.cfg.build_id;
                    if !ok {
                        events.push(Event::Refused { reason: RefuseReason::Version });
                        return;
                    }
                    *s = Some(slot);
                    self.local_slot = slot;
                    self.welcomed = Some((seed, settings.clone()));
                    *host_peer = Some(Peer {
                        addr: from,
                        slot: 0,
                        // The WELCOME payload is the HOST's display codename.
                        codename: codename.chars().take(31).collect(),
                        connected: true,
                        last_recv_ms: self.now_ms,
                        last_send_ms: self.now_ms,
                        their_ack: None,
                        our_ack: 0,
                        recv_ticks: BTreeSet::new(),
                        remote_hashes: HashMap::new(),
                        go_confirmed: false,
                        last_go_ms: 0,
                        late_serve: None,
                        late_joined: false,
                    });
                    // The host's delay is the match's (both sides schedule the same lead).
                    self.cfg.input_delay = input_delay as u32;
                    events.push(Event::Connected {
                        slot,
                        seed,
                        settings,
                        input_delay: input_delay as u32,
                    });
                    let _ = hash_every;
                }
            }
            Packet::Refuse { reason } => {
                let reason = match reason {
                    1 => RefuseReason::Version,
                    2 => RefuseReason::Build,
                    3 => RefuseReason::Started,
                    _ => RefuseReason::Full,
                };
                events.push(Event::Refused { reason });
                if let Role::Client { hello_due_ms, .. } = &mut self.role {
                    *hello_due_ms = u64::MAX; // stop retrying
                }
            }
            Packet::Go { first_tick } => {
                if !self.started {
                    self.started = true;
                    self.start_tick = first_tick;
                    events.push(Event::MatchStarted { first_tick });
                }
            }
            Packet::Input { base_tick, agent_mask, ack_tick, ack_bits, inputs } => {
                if let Role::Client { host_peer, .. } = &mut self.role {
                    if let Some(p) = host_peer {
                        p.last_recv_ms = self.now_ms;
                        p.their_ack = if ack_tick == u32::MAX { None } else { Some(ack_tick) };
                        let per_tick = agent_mask.count_ones().max(1) as usize;
                        for k in 0..inputs.len() / per_tick.max(1) {
                            p.recv_ticks.insert(base_tick.wrapping_add(k as u32));
                        }
                    }
                }
                let per_tick = agent_mask.count_ones().max(1) as usize;
                for k in 0..inputs.len() / per_tick.max(1) {
                    let t = base_tick.wrapping_add(k as u32);
                    for (j, s) in (0..MAX_AGENTS as u8).filter(|s| agent_mask & (1u16 << s) != 0).enumerate() {
                        if s as usize == self.local_slot as usize {
                            continue;
                        }
                        let idx = k * per_tick + j;
                        if let Some(v) = inputs.get(idx) {
                            self.inputs[s as usize].insert(t, *v);
                        }
                    }
                }
            }
            Packet::Hash { tick, hash, prev_tick, prev_hash } => {
                if let Role::Client { host_peer, .. } = &mut self.role {
                    if let Some(p) = host_peer {
                        p.last_recv_ms = self.now_ms;
                        p.remote_hashes.insert(tick, hash);
                        if prev_tick != tick {
                            p.remote_hashes.insert(prev_tick, prev_hash);
                        }
                    }
                }
            }
            Packet::Keepalive { .. } => {
                if let Role::Client { host_peer, .. } = &mut self.role {
                    if let Some(p) = host_peer {
                        p.last_recv_ms = self.now_ms;
                    }
                }
            }
            Packet::Data { bytes } => {
                if let Role::Client { host_peer, .. } = &mut self.role {
                    if let Some(p) = host_peer {
                        p.last_recv_ms = self.now_ms;
                        events.push(Event::Data { from_slot: p.slot, bytes });
                    }
                }
            }
            Packet::Leave { slot, reason } => {
                let reason = LeaveReason::from_wire(reason);
                if reason == LeaveReason::MatchEnded {
                    self.match_ended = true;
                    events.push(Event::MatchEnded { reason });
                } else if (slot as usize) < MAX_AGENTS {
                    self.slot_human[slot as usize] = false;
                    events.push(Event::ClientLeft { slot, reason });
                }
            }
            Packet::SlotMap { human_slots } => {
                let old = self.slot_human;
                for i in 0..MAX_AGENTS {
                    self.slot_human[i] = human_slots & (1u16 << i) != 0;
                }
                if old != self.slot_human {
                    events.push(Event::SlotMapChanged { human_slots: self.slot_human });
                }
            }
            Packet::JoinBegin { first_tick, snapshot_tick, snapshot_hash } => {
                // mp-join-in-progress: the host began the late-join handover.
                // The chunks follow; the log assembles on the last one.
                if let Role::Client { host_peer, .. } = &mut self.role {
                    if let Some(p) = host_peer {
                        p.last_recv_ms = self.now_ms;
                    }
                }
                self.late_recv = Some(LateRecv {
                    first_tick,
                    snapshot_tick,
                    snapshot_hash,
                    chunk_count: 0,
                    chunks: Vec::new(),
                });
            }
            Packet::JoinLog { first_tick, tick_count: _, chunk_index, chunk_count, inputs } => {
                if let Role::Client { host_peer, .. } = &mut self.role {
                    if let Some(p) = host_peer {
                        p.last_recv_ms = self.now_ms;
                    }
                }
                // Store the chunk (no session call inside the borrow).
                let complete = {
                    let Some(lr) = self.late_recv.as_mut() else { return };
                    if lr.chunk_count == 0 {
                        lr.chunk_count = chunk_count;
                        lr.chunks = vec![None; chunk_count as usize];
                    }
                    if chunk_index as usize >= lr.chunks.len() || first_tick != lr.first_tick {
                        return;
                    }
                    lr.chunks[chunk_index as usize] = Some(inputs);
                    lr.chunks.iter().all(|c| c.is_some())
                };
                self.send_packet(self.host_addr(), &Packet::JoinAck { chunk_index });
                // Complete? Assemble the log and hand it to the app.
                if complete {
                    let lr = self.late_recv.take().expect("complete transfer held");
                    let per_tick = MAX_AGENTS;
                    let mut log: Vec<[Input; MAX_AGENTS]> =
                        Vec::with_capacity(lr.chunks.len() * per_tick);
                    for c in lr.chunks.into_iter().flatten() {
                        for chunk in c.chunks(per_tick) {
                            let mut row = [NEUTRAL; MAX_AGENTS];
                            for (i, v) in chunk.iter().enumerate() {
                                row[i] = *v;
                            }
                            log.push(row);
                        }
                    }
                    self.started = true;
                    self.start_tick = lr.first_tick;
                    self.late_joined = true;
                    // The joiner "has" everything through the snapshot: seed
                    // the ack base there (the relays resume at snapshot+1).
                    if let Role::Client { host_peer: Some(p), .. } = &mut self.role {
                        p.our_ack = lr.snapshot_tick.wrapping_add(1);
                        // ...and our own send window starts there too: the host seeded its ack
                        // of this slot at the snapshot (`begin_late_join`), so inputs from tick 0
                        // were never counted and the window never moved (the app's three-window
                        // check: every machine stalled at the join tick).
                        p.their_ack = Some(lr.snapshot_tick);
                    }
                    if let Some((seed, settings)) = self.welcomed.clone() {
                        events.push(Event::LateJoinHandover {
                            slot: self.local_slot,
                            seed,
                            settings,
                            first_tick: lr.first_tick,
                            snapshot_tick: lr.snapshot_tick,
                            snapshot_hash: lr.snapshot_hash,
                            log,
                        });
                    }
                }
            }
            Packet::JoinSlot { slot, join_tick } => {
                if (slot as usize) < MAX_AGENTS {
                    let mine = slot == self.local_slot && self.late_joined;
                    if let Role::Client { host_peer, .. } = &mut self.role {
                        if let Some(p) = host_peer {
                            p.last_recv_ms = self.now_ms;
                            // The joiner's own send window starts at the join tick: the host
                            // seeded its ack of this slot there (JOIN_READY: "everything before
                            // it is implicitly covered"). Without it the window kept resending
                            // the pre-join NEUTRAL ticks, never reached the join tick, and the
                            // host - stalled there - had no INPUT packet to carry the ack back:
                            // every machine froze at the join tick (the three-window check).
                            if mine { p.their_ack = Some(join_tick.wrapping_sub(1)); }
                        }
                    }
                    self.late_confirm_due = None; // the confirm was answered
                    self.slot_human[slot as usize] = true;
                    self.join_from[slot as usize] = join_tick;
                    events.push(Event::LateJoinSlot { slot, join_tick });
                }
            }
            Packet::Hello { .. } => {}
            // Host-directed handshake packets never arrive at a client.
            Packet::JoinAck { .. } | Packet::JoinReady {} => {}
        }
    }

    /// Host: remove a human from `slot` (drop policy applies).
    fn drop_slot(&mut self, slot: u8, reason: LeaveReason, events: &mut Vec<Event>) {
        if slot == 0 || slot as usize >= MAX_AGENTS {
            return;
        }
        self.slot_human[slot as usize] = false;
        match self.cfg.drop_policy {
            DropPolicy::Neutral => {
                events.push(Event::ClientLeft { slot, reason });
                // Relay the departure to the remaining clients.
                let pkt = Packet::Leave { slot, reason: reason.to_wire() };
                self.send_to_peers(&pkt);
                self.broadcast_slotmap();
            }
            DropPolicy::EndMatch => {
                self.match_ended = true;
                events.push(Event::MatchEnded { reason: LeaveReason::MatchEnded });
                let pkt = Packet::Leave { slot: 0, reason: LeaveReason::MatchEnded.to_wire() };
                self.send_to_peers(&pkt);
            }
        }
    }

    fn check_timeouts(&mut self, now_ms: u64, events: &mut Vec<Event>) {
        let timeout = self.cfg.timeout_ms;
        if timeout == 0 {
            return;
        }
        // Host: drop silent clients.
        if let Role::Host { peers, .. } = &mut self.role {
            let now = now_ms;
            let mut dropped: Vec<(u8, LeaveReason)> = Vec::new();
            peers.retain(|p| {
                if p.connected && now.saturating_sub(p.last_recv_ms) > timeout {
                    dropped.push((p.slot, LeaveReason::Timeout));
                    false
                } else {
                    true
                }
            });
            for (slot, reason) in dropped {
                self.drop_slot(slot, reason, events);
            }
            return;
        }
        // Client: host silence after connection.
        if !self.host_timed_out {
            if let Role::Client { host_peer, slot, .. } = &mut self.role {
                if slot.is_some() {
                    if let Some(p) = host_peer {
                        if now_ms.saturating_sub(p.last_recv_ms) > timeout {
                            self.host_timed_out = true;
                            events.push(Event::HostTimeout);
                        }
                    }
                }
            }
        }
    }

    /// Compare our local hashes against every peer's remote hashes.
    fn check_desync(&mut self, events: &mut Vec<Event>) {
        let peers_hashes: Vec<HashMap<u32, u64>> = match &mut self.role {
            Role::Host { peers, .. } => peers.iter().map(|p| p.remote_hashes.clone()).collect(),
            Role::Client { host_peer, .. } => host_peer
                .as_ref()
                .map(|p| vec![p.remote_hashes.clone()])
                .unwrap_or_default(),
        };
        for remote in peers_hashes {
            // cross-process-determinism: the ticks are compared in ascending order, not the
            // HashMap's per-process random order — the sim state is untouched, but WHICH desync
            // tick gets reported first (the diagnostic the user sees) must not depend on the
            // process's RandomState.
            let mut ticks: Vec<u32> = remote.keys().copied().collect();
            ticks.sort_unstable();
            for tick in ticks {
                let rhash = remote[&tick];
                if self.compared.contains(&tick) {
                    continue;
                }
                if let Some(lhash) = self.local_hashes.get(&tick) {
                    self.compared.insert(tick);
                    if *lhash != rhash {
                        events.push(Event::Desync { tick, local: *lhash, remote: rhash });
                    }
                }
            }
        }
    }

    /// Flush the stall/resume events the app's `ready_inputs` queries
    /// detected, then compare hashes and prune old state.
    fn detect_stall(&mut self, events: &mut Vec<Event>) {
        self.check_desync(events);
        if let Some((tick, missing)) = self.pending_stall.take() {
            events.push(Event::Stalled { tick, missing });
        }
        if let Some(tick) = self.pending_resume.take() {
            events.push(Event::Resumed { tick });
        }
        if !self.started {
            return;
        }
        // Prune old state (bounded memory on long matches).
        if let Some(lr) = self.last_ready {
            let cutoff = lr.saturating_sub(512);
            for m in self.inputs.iter_mut() {
                m.retain(|k, _| *k >= cutoff);
            }
            self.local_hashes.retain(|k, _| *k >= cutoff);
            self.compared.retain(|k| *k >= cutoff);
            if let Role::Host { peers, .. } = &mut self.role {
                for p in peers.iter_mut() {
                    p.remote_hashes.retain(|k, _| *k >= cutoff);
                    let ack_floor = p.our_ack.saturating_sub(512);
                    p.recv_ticks.retain(|t| *t >= ack_floor);
                }
            } else if let Role::Client { host_peer, .. } = &mut self.role {
                if let Some(p) = host_peer {
                    p.remote_hashes.retain(|k, _| *k >= cutoff);
                    let ack_floor = p.our_ack.saturating_sub(512);
                    p.recv_ticks.retain(|t| *t >= ack_floor);
                }
            }
        }
    }
}

impl Session<crate::net::UdpTransport> {
    /// Test-only fault injection on the real transport (net-e2e-loopback):
    /// deterministic send-path loss + reordering. See
    /// [`crate::net::UdpTransport::inject_fault`]. The shim transport has its
    /// own `set_loss`/`set_latency`; this is the real-socket equivalent.
    pub fn inject_link_fault(&mut self, seed: u64, drop_per_mille: u32, reorder_per_mille: u32) {
        self.transport.inject_fault(seed, drop_per_mille, reorder_per_mille);
    }
}
