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
    /// net-desync-resync: the desync is NOT recoverable on this machine — a
    /// second mismatch inside the resync window, a resync that never got
    /// served inside `timeout_ms`, or a second slot desyncing while one
    /// resync is in flight. The app ends the match for everyone (the
    /// pre-resync behaviour).
    DesyncFatal { tick: u32 },
    /// Both: a human slot's client left (timeout or graceful).
    ClientLeft { slot: u8, reason: LeaveReason },
    /// Client: no packets from the host for `Config::timeout_ms`.
    HostTimeout,
    /// Client: the host sent LEAVE (it quit the match or closed the lobby) —
    /// reported at once instead of after the timeout [S mp-online-failures].
    HostLeft,
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
    /// net-internet: the host measures every peer's RTT (PING/PONG) and picks the input delay
    /// so a remote input normally arrives before its tick runs; the choice is announced
    /// (DELAY) ahead of a tick boundary and every machine switches there. Off = `input_delay`
    /// for the whole match. The app must schedule through [`Session::submit_local`] (it
    /// follows the schedule); `local_input` callers keep their own fixed lead.
    pub adaptive_delay: bool,
    /// The adaptive delay's range, in ticks.
    pub min_delay: u32,
    pub max_delay: u32,
    /// PING interval per peer (0 = never; the RTT then stays unknown). Pings also keep the
    /// link alive, so with pings on the keepalive rarely fires.
    pub ping_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            protocol_version: PROTOCOL_VERSION,
            build_id: crate::DEFAULT_BUILD_ID,
            input_delay: 3,
            redundancy: 2,
            hash_every: 30,
            // net-internet: 8 s — a Wi-Fi roam or a home router's hiccup lasts seconds; 5 s
            // dropped players a real link brings back [S].
            timeout_ms: 8000,
            keepalive_ms: 250,
            hello_retry_ms: 400,
            resend_ms: 33,
            drop_policy: DropPolicy::Neutral,
            adaptive_delay: false,
            min_delay: 2,
            max_delay: 15,
            ping_ms: 250,
        }
    }
}

/// One tick in ms (the game's fixed 60 Hz).
const TICK_MS: f64 = 1000.0 / 60.0;
/// The adaptive delay is re-evaluated every this many ticks of the host's head …
const DELAY_EVAL_TICKS: u32 = 30;
/// … a change takes effect this many ticks past the host's head (every machine — clients
/// run behind the host — hears of it before it runs that tick) …
const DELAY_LEAD_TICKS: u32 = 30;
/// … and a lower delay needs this many evaluations in a row asking for it (≈ 3 s; a rise
/// is immediate — a stall costs more than a tick of extra latency).
const DELAY_LOWER_EVALS: u32 = 6;
/// Extra ticks over the measured need (srtt + 2·rttvar).
const DELAY_SLACK: u32 = 1;
/// PINGs per loss-estimate window (20 × 250 ms = 5 s).
const LOSS_WINDOW: u32 = 20;

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
    /// The join tick already announced for this late joiner: a retransmitted
    /// JOIN_READY (the joiner resends until JOIN_SLOT lands — a round trip
    /// longer than `hello_retry_ms` sends two) gets the SAME tick again. A
    /// second pick moved the join tick on some machines and not others.
    announced: Option<u32>,
    /// net-desync-resync: this CONNECTED peer is resyncing — its slot rides
    /// NEUTRAL from the serve until the re-announce (`join_from` holds the
    /// pending marker), and the JOIN_READY arms it back at the new join tick
    /// without a `join_history` entry (its body never left the match).
    resyncing: bool,
    /// net-internet send state: the newest tick sent to this peer …
    sent_hi: Option<u32>,
    /// … a data INPUT arrived since our last INPUT to this peer (our ack must reach it even
    /// when we have nothing new to send — see `plan_inputs`) …
    ack_dirty: bool,
    /// … when we last sent this peer an INPUT …
    last_input_ms: u64,
    /// … and when its ack of our sends last advanced (the retransmit timer's base).
    progress_ms: u64,
    /// Smoothed RTT and its mean deviation (ms; TCP's estimator, RFC 6298), from PONGs.
    srtt_ms: f64,
    rttvar_ms: f64,
    rtt_samples: u32,
    last_ping_ms: u64,
    /// Host: when the current DELAY announcement last went to this peer.
    delay_sent_ms: u64,
    /// The peer's ack bitmap for the ticks past its contiguous ack (fast retransmit).
    their_bits: u32,
    /// The first fresh tick of each of our last `redundancy - 1` fresh packets (the
    /// reach-back of the next one).
    fresh_starts: std::collections::VecDeque<u32>,
    /// PINGs sent / PONGs back in the current window, and the round-trip loss of the last
    /// full window (per mille) — the link's measured loss raises this peer's redundancy.
    pings_sent: u32,
    pongs_back: u32,
    rt_loss_pm: u32,
}

impl Peer {
    fn new(addr: SocketAddr, slot: u8, codename: String, now_ms: u64) -> Peer {
        Peer {
            addr,
            slot,
            codename,
            connected: true,
            last_recv_ms: now_ms,
            last_send_ms: now_ms,
            their_ack: None,
            our_ack: 0,
            recv_ticks: BTreeSet::new(),
            remote_hashes: HashMap::new(),
            go_confirmed: false,
            last_go_ms: 0,
            late_serve: None,
            late_joined: false,
            announced: None,
            resyncing: false,
            sent_hi: None,
            ack_dirty: false,
            last_input_ms: 0,
            progress_ms: now_ms,
            srtt_ms: 0.0,
            rttvar_ms: 0.0,
            rtt_samples: 0,
            last_ping_ms: 0,
            delay_sent_ms: 0,
            their_bits: 0,
            fresh_starts: std::collections::VecDeque::new(),
            pings_sent: 0,
            pongs_back: 0,
            rt_loss_pm: 0,
        }
    }

    /// Packets each fresh tick rides for this peer: the configured redundancy, +1 when the
    /// smoothed round-trip loss is 4 % or more (≈ 2 % each way), +2 at 12 % or more.
    fn redundancy(&self, base: u32) -> u32 {
        base.max(1) + match self.rt_loss_pm {
            0..=39 => 0,
            40..=119 => 1,
            _ => 2,
        }
    }

    /// One PING went out; every `LOSS_WINDOW` pings the window's PONG count becomes the
    /// round-trip loss estimate.
    fn ping_sent(&mut self) {
        if self.pings_sent >= LOSS_WINDOW {
            // The last PING's PONG may still be in flight: count the window one short.
            let n = self.pings_sent - 1;
            let back = self.pongs_back.min(n);
            let window = 1000 - back * 1000 / n.max(1);
            // Smoothed over windows (one unlucky window does not raise the redundancy for
            // long; a lossy link reaches its level within a couple of windows).
            self.rt_loss_pm = (self.rt_loss_pm + window) / 2;
            self.pings_sent = 0;
            self.pongs_back = self.pongs_back.saturating_sub(back);
        }
        self.pings_sent += 1;
    }

    /// The ack the peer sent (monotonic: a reordered older packet never moves it back —
    /// protocol 4 let a late datagram rewind it, one half of the loopback wedge).
    fn take_ack(&mut self, ack_tick: u32, ack_bits: u32, now_ms: u64) {
        if ack_tick == u32::MAX {
            return;
        }
        match self.their_ack {
            Some(a) if ack_tick < a => {}
            Some(a) if ack_tick == a => self.their_bits |= ack_bits,
            _ => {
                self.their_ack = Some(ack_tick);
                self.their_bits = ack_bits;
                self.progress_ms = now_ms;
            }
        }
    }

    fn rtt_sample(&mut self, rtt: f64) {
        if self.rtt_samples == 0 {
            self.srtt_ms = rtt;
            self.rttvar_ms = rtt / 2.0;
        } else {
            self.rttvar_ms = 0.75 * self.rttvar_ms + 0.25 * (self.srtt_ms - rtt).abs();
            self.srtt_ms = 0.875 * self.srtt_ms + 0.125 * rtt;
        }
        self.rtt_samples += 1;
    }

    /// The retransmit timeout for an unacked input (RFC 6298's srtt + 4·rttvar, plus a tick
    /// for the peer's frame pacing), clamped to [2·resend, 1 s]; 250 ms before any sample.
    fn rto_ms(&self, resend_ms: u64) -> u64 {
        let floor = (2 * resend_ms).max(30);
        if self.rtt_samples == 0 {
            return 250u64.max(floor);
        }
        ((self.srtt_ms + 4.0 * self.rttvar_ms + TICK_MS) as u64).clamp(floor, 1000)
    }
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

/// net-desync-resync: the resync window — a machine that desyncs again
/// within 60 s of its last resync request falls back to ending the match
/// (the task's fallback rule; the session's logical clock, not the Sim's).
const RESYNC_RETRY_WINDOW_MS: u64 = 60_000;
/// Ticks between the host's live head and the announced join tick: the
/// joiner — already caught up at JOIN_READY — schedules its inputs there and
/// every peer runs the join-spawn at that boundary.
pub const JOIN_MARGIN_TICKS: u32 = 90;

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
    /// Host: per slot, the first tick a departed human no longer drives (its
    /// drop tick — `last_ready + 1` when it left). The ticks before it RAN
    /// with the leaver's real inputs, so the host keeps relaying those (a
    /// client a few ticks behind still needs them) and only the ticks from
    /// here on are NEUTRAL [S mp-online-failures: relaying NEUTRAL for the
    /// already-run ticks desynced every client that was behind the host].
    left_at: [u32; MAX_AGENTS],
    /// Host: the codename of each slot's last departed human — a reconnect
    /// with the same (non-empty) codename takes its old slot back (its body
    /// still stands on `DropPolicy::Neutral`).
    left_names: [String; MAX_AGENTS],
    /// Host: the match is over (the debriefing is up) — HELLOs from unknown
    /// peers are ignored, so a client re-dialing for the rematch keeps
    /// retrying until the NEXT lobby listens on the port instead of
    /// late-joining the finished match.
    joins_closed: bool,
    /// Client: the host sent LEAVE ([`Event::HostLeft`] fired).
    host_left: bool,
    /// The match's late joins as (slot, join tick): the host appends each announcement; a late
    /// joiner receives the ones up to its snapshot in JOIN_BEGIN — its replay spawns those
    /// bodies at those ticks, exactly as the live machines did (mp-online-failures).
    join_history: Vec<(u8, u32)>,
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
    /// net-desync-resync: this client detected a hash mismatch and asked the
    /// host to re-serve the match state — it stops contributing inputs (its
    /// own `ready_inputs` is None: never guess) until the re-serve's
    /// JOIN_SLOT arms its slot again. The request retransmits every
    /// `resend_ms` until a JOIN_BEGIN lands, and `timeout_ms` without a
    /// serve is fatal.
    resync_catchup: bool,
    resync_pending: bool,
    /// net-desync-resync: the tick this client froze at (rides the RESYNC
    /// request; the host logs the mismatch from its own Desync event).
    resync_tick: u32,
    /// net-desync-resync: the last RESYNC request send (the retransmit base).
    resync_req_ms: u64,
    /// net-desync-resync: when the request dies (`timeout_ms` without a
    /// JOIN_BEGIN — the host never served).
    resync_deadline_ms: u64,
    /// net-desync-resync: the last resync REQUEST (the 60 s window — a
    /// second mismatch inside it is fatal, per the task's fallback rule).
    last_resync_ms: Option<u64>,
    /// net-desync-resync: host — the desync we are watching: (slot, tick,
    /// deadline). The offending client is expected to request a resync; the
    /// watch expires at `timeout_ms` (the session's own silent-peer rule)
    /// and is fatal — the pre-resync behaviour for a peer that never asks.
    resync_watch: Option<(u8, u32, u64)>,
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
    /// net-internet: the input-delay schedule, (from_tick, delay) ascending — the host's
    /// announcements (DELAY); empty = `cfg.input_delay` throughout.
    delay_sched: Vec<(u32, u32)>,
    /// The next own-slot tick with no scheduled input (`submit_local` / `local_input`).
    sched_next: Option<u32>,
    /// A sample `submit_local` could not place (the delay dropped: its tick was already
    /// scheduled) — its one-shot lanes merge into the next sample.
    carry: Option<Input>,
    /// Host: the newest tick relayed to the clients — every tick up to it has every input
    /// (`relay_horizon`), which can run ahead of `last_ready`: the relay leaves as soon as a
    /// tick is complete, not when the host's own clock reaches it.
    committed_hi: Option<u32>,
    /// Host: the next head tick the adaptive delay is re-evaluated at, and how many
    /// evaluations in a row asked for a lower delay.
    delay_eval_tick: u32,
    lower_streak: u32,
    /// Bytes / datagrams this session sent (diagnostics: the app's bytes/s readout).
    sent_bytes: u64,
    sent_packets: u64,
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

/// A fill tick's input (the delay rose): the held lanes of `inp` only — no mouse delta, no
/// one-shot press, no fired round, so nothing happens twice.
fn held_lanes(inp: &Input) -> Input {
    Input {
        turn: 0.0,
        look: 0.0,
        jump: false,
        crouch: false,
        mode_switch: false,
        reload: false,
        use_key: false,
        weapon_cycle: 0,
        gadget_cycle: 0,
        switch_to: None,
        shot: false,
        ..*inp
    }
}

/// A dropped sample (the delay fell) merged into the next: the mouse deltas add, presses
/// and a fired round survive; held lanes are the newer sample's.
fn merge_carry(c: &Input, n: &Input) -> Input {
    Input {
        turn: c.turn + n.turn,
        look: c.look + n.look,
        jump: c.jump || n.jump,
        crouch: c.crouch || n.crouch,
        mode_switch: c.mode_switch || n.mode_switch,
        reload: c.reload || n.reload,
        use_key: c.use_key || n.use_key,
        weapon_cycle: c.weapon_cycle.saturating_add(n.weapon_cycle),
        gadget_cycle: c.gadget_cycle.saturating_add(n.gadget_cycle),
        switch_to: n.switch_to.or(c.switch_to),
        shot: c.shot || n.shot,
        ..*n
    }
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
            left_at: [0; MAX_AGENTS],
            left_names: Default::default(),
            joins_closed: false,
            host_left: false,
            join_history: Vec::new(),
            inputs: Default::default(),
            run_log: Vec::new(),
            log_base: 0,
            late_recv: None,
            welcomed: None,
            late_joined: false,
            late_confirm_due: None,
            // net-desync-resync: no resync in flight at build (both roles).
            resync_pending: false,
            resync_catchup: false,
            resync_tick: 0,
            resync_req_ms: 0,
            resync_deadline_ms: 0,
            last_resync_ms: None,
            resync_watch: None,
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
            delay_sched: Vec::new(),
            sched_next: None,
            carry: None,
            committed_hi: None,
            delay_eval_tick: 0,
            lower_streak: 0,
            sent_bytes: 0,
            sent_packets: 0,
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
            left_at: [0; MAX_AGENTS],
            left_names: Default::default(),
            joins_closed: false,
            host_left: false,
            join_history: Vec::new(),
            inputs: Default::default(),
            run_log: Vec::new(),
            log_base: 0,
            late_recv: None,
            welcomed: None,
            late_joined: false,
            late_confirm_due: None,
            // net-desync-resync: no resync in flight at build (both roles).
            resync_pending: false,
            resync_catchup: false,
            resync_tick: 0,
            resync_req_ms: 0,
            resync_deadline_ms: 0,
            last_resync_ms: None,
            resync_watch: None,
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
            delay_sched: Vec::new(),
            sched_next: None,
            carry: None,
            committed_hi: None,
            delay_eval_tick: 0,
            lower_streak: 0,
            sent_bytes: 0,
            sent_packets: 0,
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

    /// Client: this session is joining a match already in progress — the
    /// host began the late-join handover (JOIN_BEGIN arrived) or completed it.
    /// The front end shows "joining" instead of the pre-match lobby.
    pub fn late_join_pending(&self) -> bool {
        matches!(self.role, Role::Client { .. }) && (self.late_recv.is_some() || self.late_joined)
    }

    /// Host: the match is over — ignore HELLOs from unknown peers from now on
    /// (a re-dial for the rematch must reach the NEXT lobby, never late-join
    /// this finished match). Known peers are unaffected.
    pub fn close_joins(&mut self) {
        self.joins_closed = true;
    }

    /// Client: the host sent LEAVE.
    pub fn host_left(&self) -> bool {
        self.host_left
    }

    /// Late joiner: the earlier late joins (slot, join tick) up to the snapshot, from the
    /// host's JOIN_BEGIN — the replay must spawn those bodies at those ticks (`Sim::mp_join_spawn`
    /// before the tick, as every live machine did). Host: every join announced so far.
    pub fn join_history(&self) -> &[(u8, u32)] {
        &self.join_history
    }

    /// Host: whether `s`'s input for `tick` is a real input — a live human
    /// from its join tick, or a departed human for the ticks before its drop
    /// (`left_at`): those ran with its real inputs on the host and every
    /// client must run them the same way.
    fn host_slot_required(&self, s: usize, tick: u32) -> bool {
        if self.slot_human[s] {
            tick >= self.join_from[s]
        } else {
            tick >= self.join_from[s] && tick < self.left_at[s]
        }
    }

    /// The input `ready_inputs` uses for slot `s` at `tick`, or None while it
    /// is missing. The local slot rides NEUTRAL before its join tick (a late
    /// joiner produces no inputs until then). The HOST decides every other
    /// slot (`host_slot_required`); a CLIENT takes every other slot from the
    /// host's relay — the host's materialised NEUTRAL included — so a drop or
    /// a join can never make a client guess differently from the host for a
    /// tick the host already ran [S mp-online-failures].
    fn tick_input(&self, s: usize, tick: u32) -> Option<Input> {
        if s == self.local_slot as usize {
            // net-desync-resync: a resyncing client contributes nothing while it
            // waits for the serve (its own `ready_inputs` is None: never guess,
            // the app freezes on the last ticked frame). During the CATCH-UP
            // (the replay done, the re-announce not yet landed) its own slot
            // runs the recorded inputs where they exist — the host ran those
            // real rows before the freeze — and NEUTRAL where they do not (the
            // host's post-serve rows are NEUTRAL for the slot too).
            if self.resync_pending {
                return None;
            }
            if self.resync_catchup {
                // Tick-aware: below the re-announced join tick the slot runs the
                // recorded inputs where they exist (the host ran those real rows
                // before the freeze) and NEUTRAL where it does not (the host's
                // post-serve rows are NEUTRAL for the slot too); from the join
                // tick on, the normal arm (a missing own input stalls).
                if tick < self.join_from[s] {
                    return self.inputs[s].get(&tick).copied().or(Some(NEUTRAL));
                }
                return self.inputs[s].get(&tick).copied();
            }
            if self.slot_human[s] && tick >= self.join_from[s] {
                return self.inputs[s].get(&tick).copied();
            }
            return Some(NEUTRAL);
        }
        if matches!(self.role, Role::Host { .. }) && !self.host_slot_required(s, tick) {
            return Some(NEUTRAL);
        }
        self.inputs[s].get(&tick).copied()
    }

    /// Record the local agent's input for `tick`. Idempotent per tick.
    /// Ignored until the client's slot is assigned (host: always slot 0) —
    /// the app samples controls only after `Event::Connected`/`MatchStarted`,
    /// so nothing is lost.
    /// The match's input lead: the app submits the input sampled while running tick `t` for
    /// tick `t + input_delay()` (the host's config; a client takes it from the WELCOME).
    /// With the adaptive delay this is the delay for the next tick to run (the schedule's
    /// entry in force there).
    pub fn input_delay(&self) -> u32 {
        self.delay_at(self.next_run_tick())
    }

    /// The input delay in force at `tick` (the schedule's last entry at or before it).
    pub fn delay_at(&self, tick: u32) -> u32 {
        self.delay_sched.iter().rev().find(|(f, _)| *f <= tick).map_or(self.cfg.input_delay, |(_, d)| *d)
    }

    /// The input-delay schedule (from_tick, delay) — identical on every machine that heard
    /// every announcement in time (net-internet; the agreement test compares them).
    pub fn delay_schedule(&self) -> &[(u32, u32)] {
        &self.delay_sched
    }

    fn next_run_tick(&self) -> u32 {
        self.last_ready.map_or(self.start_tick, |t| t.wrapping_add(1))
    }

    /// The smoothed RTT to each peer in ms, by slot (`None` before the first PONG).
    pub fn peer_rtts(&self) -> Vec<(u8, Option<f64>)> {
        let peers: Vec<&Peer> = match &self.role {
            Role::Host { peers, .. } => peers.iter().filter(|p| p.connected).collect(),
            Role::Client { host_peer, .. } => host_peer.iter().collect(),
        };
        peers.iter().map(|p| (p.slot, (p.rtt_samples > 0).then_some(p.srtt_ms))).collect()
    }

    /// Diagnostics: each peer's ack of our input sends, by slot (`None` = nothing acked).
    pub fn peer_acks(&self) -> Vec<(u8, Option<u32>)> {
        let peers: Vec<&Peer> = match &self.role {
            Role::Host { peers, .. } => peers.iter().filter(|p| p.connected).collect(),
            Role::Client { host_peer, .. } => host_peer.iter().collect(),
        };
        peers.iter().map(|p| (p.slot, p.their_ack)).collect()
    }

    /// Bytes and datagrams this session has sent (payload; add the 28-byte UDP/IPv4 header
    /// per datagram for the link's view).
    pub fn sent_totals(&self) -> (u64, u64) {
        (self.sent_bytes, self.sent_packets)
    }

    pub fn local_input(&mut self, tick: u32, input: Input) {
        if let Role::Client { slot: None, .. } = &self.role {
            return;
        }
        self.inputs[self.local_slot as usize].insert(tick, input);
        let next = tick.wrapping_add(1);
        if self.sched_next.map_or(true, |n| next > n) {
            self.sched_next = Some(next);
        }
    }

    /// net-internet: schedule the local input sampled while running `run_tick` — it lands on
    /// `run_tick + delay_at(run_tick)`, following the (adaptive) delay schedule. When the
    /// delay rose, the ticks the old lead never reached are filled with this sample's held
    /// lanes (sticks, aim, trigger, the held weapon — no mouse delta and no one-shot press,
    /// so nothing doubles); when it fell, the sample's tick was already scheduled, so its
    /// one-shot lanes (mouse delta, presses, a fired round) carry into the next sample.
    /// Never overwrites a scheduled tick (it may already be on the wire).
    pub fn submit_local(&mut self, run_tick: u32, input: Input) {
        if let Role::Client { slot: None, .. } = &self.role {
            return;
        }
        let target = run_tick.wrapping_add(self.delay_at(run_tick));
        let mut inp = input;
        if let Some(c) = self.carry.take() {
            inp = merge_carry(&c, &inp);
        }
        let slot = self.local_slot as usize;
        if let Some(next) = self.sched_next {
            if target < next {
                self.carry = Some(inp);
                return;
            }
            let fill = held_lanes(&inp);
            for t in next..target {
                self.inputs[slot].insert(t, fill);
            }
        }
        self.inputs[slot].insert(target, inp);
        self.sched_next = Some(target.wrapping_add(1));
    }

    /// Send what is due now without draining the socket — the app calls it right after its
    /// ticks, so the inputs just scheduled leave this frame instead of at the next poll.
    pub fn flush(&mut self, now_ms: u64) {
        self.now_ms = now_ms;
        self.transport.set_now(now_ms);
        self.pump_peers(now_ms);
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
        // net-desync-resync: a machine that is resyncing (waiting for the serve
        // or replaying/catching up) holds hashes of a state the match has not
        // agreed on yet — store them (the replay's re-submissions keep the
        // local book honest) but do NOT send: the host must not compare them.
        if self.resync_pending || self.resync_catchup {
            return;
        }
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
            match self.tick_input(s, tick) {
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
            if self.tick_input(s, tick).is_none() {
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
        // net-internet: the first delay comes from the lobby's RTT samples (the pings ran
        // while the players picked); the DELAY announcement rides the next pumps.
        if self.cfg.adaptive_delay {
            let d0 = self.needed_delay().unwrap_or(self.cfg.input_delay);
            self.delay_sched = vec![(first_tick, d0)];
            self.delay_eval_tick = first_tick + DELAY_EVAL_TICKS;
        }
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
        self.sent_bytes += data.len() as u64;
        self.sent_packets += 1;
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

    /// Diagnostics: a one-line dump of the lockstep cursors (last ready tick, per peer the
    /// acks both ways and the newest received tick) — what a wedged run prints.
    pub fn debug_state(&self) -> String {
        let mut s = format!(
            "slot={} started={} last_ready={:?} stalled={} newest_local={:?}",
            self.local_slot,
            self.started,
            self.last_ready,
            self.stalled,
            self.inputs[self.local_slot as usize].keys().max()
        );
        let peers: Vec<&Peer> = match &self.role {
            Role::Host { peers, .. } => peers.iter().collect(),
            Role::Client { host_peer, .. } => host_peer.iter().collect(),
        };
        for p in peers {
            s.push_str(&format!(
                " | peer slot={} their_ack={:?} our_ack={} recv_max={:?} recv_n={} last_recv={}",
                p.slot,
                p.their_ack,
                p.our_ack,
                p.recv_ticks.iter().next_back(),
                p.recv_ticks.len(),
                p.last_recv_ms
            ));
        }
        for sl in 0..MAX_AGENTS {
            if let Some(m) = self.inputs[sl].keys().max() {
                s.push_str(&format!(" in[{sl}]max={m}"));
            }
        }
        s
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
                        self.sent_bytes += buf.len() as u64;
                        self.sent_packets += 1;
                    }
                }
            }
            Role::Client { host, .. } => {
                self.transport.send(*host, &buf);
                self.sent_bytes += buf.len() as u64;
                self.sent_packets += 1;
            }
        }
    }

    fn send_packet(&mut self, to: SocketAddr, pkt: &Packet) {
        let mut buf = Vec::new();
        proto::encode(pkt, &mut buf);
        self.sent_bytes += buf.len() as u64;
        self.sent_packets += 1;
        self.transport.send(to, &buf);
    }

    /// Host: the adaptive delay for the peers' measured RTTs — a remote input sampled `d`
    /// ticks ahead must cover a round trip (the client's sample reaches the host, the
    /// host's relay reaches every client: the worst client's RTT bounds both) plus its
    /// jitter (2·rttvar) plus a tick of frame pacing. `None` before any RTT sample.
    fn needed_delay(&self) -> Option<u32> {
        let Role::Host { peers, .. } = &self.role else { return None };
        let worst = peers
            .iter()
            .filter(|p| p.connected && p.rtt_samples > 0)
            .map(|p| p.srtt_ms + 2.0 * p.rttvar_ms)
            .fold(None, |a: Option<f64>, v| Some(a.map_or(v, |a| a.max(v))))?;
        let d = (worst / TICK_MS).ceil() as u32 + DELAY_SLACK;
        Some(d.clamp(self.cfg.min_delay.max(1), self.cfg.max_delay.max(self.cfg.min_delay.max(1))))
    }

    /// Host: re-evaluate the adaptive delay every `DELAY_EVAL_TICKS` of the head; a change is
    /// scheduled `DELAY_LEAD_TICKS` past the head and announced (re-sent by `pump_peers`).
    fn adapt_delay(&mut self) {
        if !self.cfg.adaptive_delay || !self.started || !matches!(self.role, Role::Host { .. }) {
            return;
        }
        let head = self.head_tick();
        if head < self.delay_eval_tick {
            return;
        }
        self.delay_eval_tick = head + DELAY_EVAL_TICKS;
        let Some(need) = self.needed_delay() else { return };
        let cur = self.delay_sched.last().map_or(self.cfg.input_delay, |e| e.1);
        let pick = if need > cur {
            self.lower_streak = 0;
            Some(need)
        } else if need + 1 < cur {
            self.lower_streak += 1;
            if self.lower_streak >= DELAY_LOWER_EVALS {
                self.lower_streak = 0;
                Some(need + 1)
            } else {
                None
            }
        } else {
            self.lower_streak = 0;
            None
        };
        if let Some(d) = pick {
            let from = head + DELAY_LEAD_TICKS;
            self.delay_sched.retain(|e| e.0 < from);
            self.delay_sched.push((from, d));
            // Bounded history: only the entries a future tick can still need (and the one in
            // force now) are kept.
            let now_tick = self.next_run_tick();
            while self.delay_sched.len() > 2 && self.delay_sched[1].0 <= now_tick {
                self.delay_sched.remove(0);
            }
            if let Role::Host { peers, .. } = &mut self.role {
                for p in peers.iter_mut() {
                    p.delay_sent_ms = 0; // announce at once
                }
            }
        }
    }

    /// Host: the newest tick the match has reached — run or relayed.
    fn head_tick(&self) -> u32 {
        match (self.last_ready, self.committed_hi) {
            (Some(a), Some(b)) => a.max(b),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => self.start_tick,
        }
    }

    /// Host: extend `committed_hi` over every tick whose inputs are all known (bounded),
    /// returning the newest tick the clients can be sent.
    fn relay_horizon(&mut self) -> Option<u32> {
        let mut c = match (self.last_ready, self.committed_hi) {
            (Some(a), Some(b)) => a.max(b).wrapping_add(1),
            (Some(a), None) | (None, Some(a)) => a.wrapping_add(1),
            (None, None) => self.start_tick,
        };
        let limit = c.wrapping_add(256);
        while c != limit && (0..MAX_AGENTS).all(|s| self.tick_input(s, c).is_some()) {
            self.committed_hi = Some(c);
            c = c.wrapping_add(1);
        }
        match (self.last_ready, self.committed_hi) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
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
        self.transport.set_now(now_ms);
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
        // 3. Per-peer work (input windows, GO retransmit, pings, keepalives), after the
        //    host's adaptive-delay evaluation (net-internet).
        self.adapt_delay();
        self.pump_peers(now_ms);
        // 3b. The late-join confirm retransmit (client).
        if let Some(due) = self.late_confirm_due {
            if now_ms >= due {
                self.send_packet(self.host_addr(), &Packet::JoinReady {});
                self.late_confirm_due = Some(now_ms + self.cfg.hello_retry_ms.max(1));
            }
        }
        // 3c. net-desync-resync (client): the RESYNC request retransmits until
        // a JOIN_BEGIN lands (the serve), and `timeout_ms` without a serve is
        // fatal — the host never picked the request up.
        if self.resync_pending {
            if now_ms >= self.resync_deadline_ms {
                self.resync_pending = false;
                events.push(Event::DesyncFatal { tick: self.resync_tick });
            } else if now_ms.saturating_sub(self.resync_req_ms) >= self.cfg.resend_ms {
                self.send_packet(self.host_addr(), &Packet::Resync { tick: self.resync_tick });
                self.resync_req_ms = now_ms;
            }
        }
        // 4. Timeouts.
        self.check_timeouts(now_ms, &mut events);
        // 4b. net-desync-resync (host): the desynced peer never requested a
        // resync inside `timeout_ms` — the pre-resync behaviour (end the match).
        if let Some((_slot, tick, deadline)) = self.resync_watch {
            if now_ms >= deadline {
                self.resync_watch = None;
                events.push(Event::DesyncFatal { tick });
            }
        }
        // 5. Stall/resume transitions.
        self.detect_stall(&mut events);
        events
    }

    fn pump_peers(&mut self, now_ms: u64) {
        // Take the peers out for the pass: planning reads the session (`&self`) while it
        // updates each peer's send state. Nothing in here dispatches packets in, so the
        // peers cannot be needed elsewhere meanwhile; they go back at the end.
        let mut peers: Vec<Peer> = match &mut self.role {
            Role::Host { peers, .. } => std::mem::take(peers),
            Role::Client { host_peer, .. } => host_peer.take().into_iter().collect(),
        };
        let is_host = matches!(self.role, Role::Host { .. });
        // The newest tick each peer can be sent: host — the relay horizon (every input
        // known); client — its own newest scheduled input.
        let newest: Option<u32> = if !self.started {
            None
        } else if is_host {
            self.relay_horizon()
        } else {
            self.client_newest()
        };
        let mut plans: Vec<(SocketAddr, Packet)> = Vec::new();
        for p in peers.iter_mut().filter(|p| p.connected) {
            let before = plans.len();
            // A late-join transfer in flight suppresses the live input relay
            // to that peer: the log covers the history, and the relay's ack
            // base resumes at the snapshot on handover.
            if self.started && p.late_serve.is_none() {
                self.plan_inputs(p, newest, now_ms, &mut plans);
            }
            // RTT probes (they double as keepalives).
            if self.cfg.ping_ms > 0 && now_ms.saturating_sub(p.last_ping_ms) >= self.cfg.ping_ms {
                plans.push((p.addr, Packet::Ping { stamp: now_ms as u32 }));
                p.last_ping_ms = now_ms;
                p.ping_sent();
            }
            // The adaptive delay's announcement (host): re-sent every 100 ms until the host is
            // well past the boundary, then once a second (a peer that missed every copy still
            // converges; correctness never depends on it — see `submit_local`).
            if is_host && self.cfg.adaptive_delay {
                if let Some(&(from, d)) = self.delay_sched.last() {
                    let every = if self.head_tick() < from.saturating_add(60) { 100 } else { 1000 };
                    if p.delay_sent_ms == 0 || now_ms.saturating_sub(p.delay_sent_ms) >= every {
                        plans.push((p.addr, Packet::Delay { from_tick: from, delay: d.min(255) as u8 }));
                        p.delay_sent_ms = now_ms.max(1);
                    }
                }
            }
            if plans.len() == before && now_ms.saturating_sub(p.last_send_ms) >= self.cfg.keepalive_ms {
                plans.push((p.addr, Packet::Keepalive { seq: self.keepalive_seq }));
                self.keepalive_seq = self.keepalive_seq.wrapping_add(1);
            }
        }
        // Late-join log chunks (host): the recorded run log rides JOIN_LOG
        // chunks, a windowed transfer re-sent on the `resend_ms` cadence
        // until the JOIN_ACKs advance (loss-safe; the ack completes it).
        if is_host {
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
        if self.started && is_host {
            for p in peers.iter_mut() {
                if p.connected && !p.go_confirmed && now_ms.saturating_sub(p.last_go_ms) >= self.cfg.resend_ms {
                    plans.push((p.addr, Packet::Go { first_tick: self.start_tick }));
                    p.last_go_ms = now_ms;
                }
            }
        }
        // Send, then mark the peers sent-to (the keepalive's idle clock).
        for (addr, pkt) in plans {
            self.send_packet(addr, &pkt);
            if let Some(p) = peers.iter_mut().find(|p| p.addr == addr) {
                p.last_send_ms = now_ms;
            }
        }
        match &mut self.role {
            Role::Host { peers: slot, .. } => *slot = peers,
            Role::Client { host_peer, .. } => *host_peer = peers.pop(),
        }
    }

    /// Client: the newest tick of its own slot it can send (its newest scheduled input; a
    /// late joiner before its join tick rides its truthful NEUTRAL up to the newest ready
    /// tick so its acks have a carrier).
    fn client_newest(&self) -> Option<u32> {
        let slot = self.local_slot as usize;
        if let Some(n) = self.sched_next.filter(|_| !self.inputs[slot].is_empty()) {
            return Some(n.wrapping_sub(1));
        }
        if let Some(n) = self.inputs[slot].keys().max() {
            return Some(*n);
        }
        let lr = self.last_ready?;
        if !self.slot_human[slot] || lr < self.join_from[slot] {
            Some(lr)
        } else {
            None
        }
    }

    /// net-internet: what INPUT packets go to `p` now. Three reasons to send, each with its
    /// own window (protocol 4 sent the whole unacked window every poll, oldest first and at
    /// most 3 relay ticks — a 150 ms link ran at ~15 ticks/s — and acks rode only on data, so
    /// a peer with nothing new to send never repaired a lost ack: the loopback wedge):
    /// * **fresh ticks** (newer than any sent): sent at once, each packet reaching back over
    ///   up to `redundancy - 1` already-sent unacked ticks, so an isolated loss costs nothing;
    /// * **retransmit**: the peer's ack has not moved for an RTO while ticks are unacked —
    ///   the window is re-sent from the oldest unacked tick;
    /// * **ack**: a data packet arrived since our last INPUT and nothing else is going out —
    ///   a pure ack (count 0), so the peer's window always learns what we hold.
    fn plan_inputs(&self, p: &mut Peer, newest: Option<u32>, now_ms: u64, plans: &mut Vec<(SocketAddr, Packet)>) {
        let is_host = matches!(self.role, Role::Host { .. });
        let mut sent = false;
        if let Some(newest) = newest {
            let floor = if is_host { self.start_tick } else { self.start_tick.min(newest) };
            let unacked_from = match p.their_ack {
                Some(a) if a.wrapping_add(1) > floor => a.wrapping_add(1),
                _ => floor,
            };
            if newest >= unacked_from {
                let rto = p.rto_ms(self.cfg.resend_ms);
                let in_flight = p.sent_hi.is_some_and(|s| s >= unacked_from);
                let stuck_ms = now_ms.saturating_sub(p.progress_ms);
                // Fast retransmit: the peer's ack bitmap shows ticks past a hole — the hole
                // was lost (or is a jitter swap: wait out rttvar + a tick first) — re-send the
                // hole without waiting a whole RTO.
                let fast = in_flight
                    && p.their_bits != 0
                    && stuck_ms as f64 >= (p.rttvar_ms + TICK_MS).max(self.cfg.resend_ms as f64);
                // Retransmit (the oldest unacked first).
                if in_flight && (fast || stuck_ms >= rto) {
                    let hi = p.sent_hi.unwrap_or(newest).min(newest);
                    let hi = if fast {
                        // Up to the first tick the peer holds past the hole.
                        hi.min(unacked_from.wrapping_add(p.their_bits.trailing_zeros()))
                    } else {
                        hi
                    };
                    if let Some((pkt, _end)) = self.build_input_range(p, unacked_from, hi) {
                        plans.push((p.addr, pkt));
                        sent = true;
                    }
                    p.progress_ms = now_ms;
                }
                // Fresh ticks, each packet reaching back to where the packet `redundancy - 1`
                // sends ago began (still unacked): every tick rides `redundancy` packets even
                // when several turn fresh at once (a reach-back counted in TICKS left a batch's
                // first ticks in one packet only — the 150 ms run's 300 ms hole).
                let fresh = p.sent_hi.map_or(unacked_from, |s| s.wrapping_add(1).max(unacked_from));
                if fresh <= newest {
                    if !in_flight {
                        p.progress_ms = now_ms; // the retransmit clock starts with the window
                        p.fresh_starts.clear();
                    }
                    let keep = p.redundancy(self.cfg.redundancy) as usize - 1;
                    let mut start = p.fresh_starts.iter().copied().min().unwrap_or(fresh).min(fresh).max(unacked_from);
                    for _ in 0..4 {
                        let Some((pkt, end)) = self.build_input_range(p, start, newest) else { break };
                        plans.push((p.addr, pkt));
                        sent = true;
                        let first_new = p.sent_hi.map_or(start, |s| s.wrapping_add(1).max(start));
                        p.sent_hi = Some(p.sent_hi.map_or(end, |s| s.max(end)));
                        if keep > 0 {
                            p.fresh_starts.push_back(first_new);
                            while p.fresh_starts.len() > keep {
                                p.fresh_starts.pop_front();
                            }
                        }
                        if end >= newest {
                            break;
                        }
                        start = end.wrapping_add(1);
                    }
                }
            }
        }
        // The pure ack (at most once per resend/2 — a peer sending every frame is answered
        // every frame while we have nothing of our own).
        if !sent && p.ack_dirty && now_ms.saturating_sub(p.last_input_ms) >= (self.cfg.resend_ms / 2).max(1) {
            let (ack_tick, ack_bits) = Self::ack_for(&p.recv_ticks, p.our_ack);
            let mask = if is_host { 1u16 } else { 1u16 << self.local_slot };
            let base = newest.map_or(self.start_tick, |n| n.wrapping_add(1));
            plans.push((p.addr, Packet::Input { base_tick: base, agent_mask: mask, ack_tick, ack_bits, inputs: Vec::new() }));
            sent = true;
        }
        if sent {
            let (ack_tick, _) = Self::ack_for(&p.recv_ticks, p.our_ack);
            if ack_tick != u32::MAX {
                p.our_ack = ack_tick;
            }
            p.ack_dirty = false;
            p.last_input_ms = now_ms;
        }
    }

    /// One INPUT packet for `p`: ticks `start..` up to `newest`, as many as fit
    /// [`crate::MAX_PACKET`] (compact inputs), with our current ack. Host → client: every
    /// slot but the receiver's that is not NEUTRAL somewhere in the window (absent = NEUTRAL;
    /// slot 0 — the host, always human — keeps the mask non-empty). Client → host: its own
    /// slot. Returns the packet and its last tick; `None` if `start` has no input yet.
    fn build_input_range(&self, p: &Peer, start: u32, newest: u32) -> Option<(Packet, u32)> {
        let is_host = matches!(self.role, Role::Host { .. });
        let span_end = newest.min(start.wrapping_add(proto::MAX_INPUT_TICKS as u32 - 1));
        if span_end < start {
            return None;
        }
        let mask: u16 = if is_host {
            let mut m = 1u16;
            for s in 1..MAX_AGENTS {
                if s == p.slot as usize {
                    continue;
                }
                let live = (start..=span_end).any(|t| self.slot_input_for_send(s, t) != Some(NEUTRAL));
                if live {
                    m |= 1 << s;
                }
            }
            m & !(1u16 << p.slot)
        } else {
            1u16 << self.local_slot
        };
        let slots: Vec<usize> = (0..MAX_AGENTS).filter(|s| mask & (1u16 << s) != 0).collect();
        let mut prev = vec![NEUTRAL; slots.len()];
        let mut size = proto::INPUT_HEAD_LEN;
        let mut inputs: Vec<Input> = Vec::new();
        let mut end = None;
        let mut t = start;
        loop {
            let mut row = Vec::with_capacity(slots.len());
            let mut cost = 0;
            for (j, &s) in slots.iter().enumerate() {
                match self.slot_input_for_send(s, t) {
                    Some(v) => {
                        cost += proto::compact_input_len(&prev[j], &v);
                        row.push(v);
                    }
                    None => break,
                }
            }
            if row.len() < slots.len() {
                break; // an input not known yet: the packet ends before it
            }
            if end.is_some() && size + cost > crate::MAX_PACKET {
                break;
            }
            size += cost;
            prev.copy_from_slice(&row);
            inputs.extend(row);
            end = Some(t);
            if t >= span_end {
                break;
            }
            t = t.wrapping_add(1);
        }
        let end = end?;
        let (ack_tick, ack_bits) = Self::ack_for(&p.recv_ticks, p.our_ack);
        Some((Packet::Input { base_tick: start, agent_mask: mask, ack_tick, ack_bits, inputs }, end))
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
            // A departed human's already-run ticks relay its REAL inputs (the
            // ticks before `left_at`); NEUTRAL only from its drop tick on.
            if !self.host_slot_required(s, t) {
                return Some(NEUTRAL);
            }
        }
        self.inputs[s].get(&t).copied()
    }

    /// The ack we send for the ticks a peer sent us: highest contiguous tick
    /// from our sliding base `our_ack`, plus a 32-tick bitmap past it
    /// (u32::MAX = nothing acked yet). The base slides with the ack so old
    /// pruned ticks can never break contiguity.
    fn ack_for(recv_ticks: &BTreeSet<u32>, our_ack: u32) -> (u32, u32) {
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
            Packet::Resync { tick: _ } => {
                // net-desync-resync: a connected client's hash mismatched and it
                // asked to resync. Re-serve the late-join family on the LIVE
                // connection: pick the newest verifiable snapshot (prefer a tick
                // the peer's hash AGREES with — its state was right up to there),
                // mark the slot non-contributing until the re-announce, and send
                // JOIN_BEGIN + the run log; JOIN_READY then re-arms the slot.
                if !self.started {
                    return;
                }
                let log_end = self.log_base + self.run_log.len() as u32;
                let head = self.last_ready.unwrap_or(0).min(log_end);
                let local_hashes = self.local_hashes.clone();
                let serve = {
                    let Role::Host { peers, .. } = &mut self.role else { return };
                    let Some(p) = peers.iter_mut().find(|p| p.addr == from && p.connected) else { return };
                    p.last_recv_ms = self.now_ms;
                    if p.resyncing {
                        return; // a retransmitted request: the transfer state machine is ack-driven
                    }
                    // Walk back from the head: the newest tick BOTH sides hashed
                    // and AGREED on; failing that the newest tick both hashed
                    // (the replay's verification decides — the host's state is
                    // the one the log produces, so the resyncer converges to it).
                    let mut agree = None;
                    let mut both = None;
                    let mut t = head;
                    while t > self.log_base {
                        let l = local_hashes.get(&t).copied();
                        let r = p.remote_hashes.get(&t).copied();
                        if let (Some(l), Some(r)) = (l, r) {
                            if both.is_none() {
                                both = Some(t);
                            }
                            if l == r {
                                agree = Some(t);
                                break;
                            }
                        }
                        t -= 1;
                    }
                    let Some(snap) = agree.or(both) else {
                        // Nothing comparable in the hash window: do not serve —
                        // the client's request deadline falls back to the lobby.
                        return;
                    };
                    let snapshot_hash = local_hashes[&snap];
                    let first_tick = self.log_base;
                    let ticks = snap.saturating_sub(first_tick) as usize + 1; // inclusive
                    let chunk_count = ((ticks + proto::JOIN_LOG_CHUNK_TICKS - 1) / proto::JOIN_LOG_CHUNK_TICKS) as u16;
                    // The slot stops contributing from NOW (`join_from` past every
                    // tick → `host_slot_required` false → NEUTRAL, the same shape
                    // a dropped peer rides under `DropPolicy::Neutral`) until the
                    // JOIN_READY re-announce moves it to the new join tick.
                    self.join_from[p.slot as usize] = u32::MAX;
                    p.their_ack = Some(snap);
                    p.sent_hi = Some(snap);
                    p.resyncing = true;
                    p.announced = None;
                    p.late_serve = Some(LateServe { snapshot_tick: snap, snapshot_hash, chunk_count, sent: 0, acked: 0, last_send_ms: 0 });
                    Some((p.addr, p.slot, p.codename.clone(), first_tick, snap, snapshot_hash))
                };
                if let Some((addr, slot, codename, first_tick, snap, snapshot_hash)) = serve {
                    let joins: Vec<(u8, u32)> =
                        self.join_history.iter().copied().filter(|(_, t)| *t <= snap).collect();
                    self.send_packet(addr, &Packet::JoinBegin { first_tick, snapshot_tick: snap, snapshot_hash, joins });
                    events.push(Event::LateJoinStarted { slot, codename: codename.chars().take(31).collect() });
                }
            }
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
                // The finished match (the debriefing) answers no new peer: the
                // HELLO goes unanswered and the dialer keeps retrying until the
                // next lobby listens on this port [S mp-online-failures].
                if self.joins_closed {
                    let known = matches!(&self.role, Role::Host { peers, .. } if peers.iter().any(|p| p.addr == from));
                    if !known {
                        return;
                    }
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
                    peers.push(Peer::new(from, slot, codename.chars().take(31).collect(), self.now_ms));
                    self.slot_human[slot as usize] = true;
                }
                self.send_welcome_to(from, slot);
                events.push(Event::ClientJoined { slot, codename: codename.chars().take(31).collect() });
                self.broadcast_slotmap();
            }
            Packet::Input { base_tick, agent_mask, ack_tick, ack_bits, inputs } => {
                let _ = ack_bits;
                let now = self.now_ms;
                let peer_slot = match &mut self.role {
                    Role::Host { peers, .. } => {
                        let idx = peers.iter().position(|p| p.addr == from);
                        match idx {
                            Some(i) => {
                                let p = &mut peers[i];
                                p.last_recv_ms = now;
                                p.go_confirmed = true;
                                p.take_ack(ack_tick, ack_bits, now);
                                let per_tick = agent_mask.count_ones().max(1) as usize;
                                let count = inputs.len() / per_tick;
                                for k in 0..count {
                                    p.recv_ticks.insert(base_tick.wrapping_add(k as u32));
                                }
                                if count > 0 {
                                    p.ack_dirty = true;
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
                        // net-desync-resync: a resyncing peer's hashes describe its
                        // replay-in-progress — not match state; the host neither
                        // stores nor compares them until the re-announce lands.
                        if !p.resyncing {
                            p.remote_hashes.insert(tick, hash);
                            // prev_tick == tick marks "no previous hash yet".
                            if prev_tick != tick {
                                p.remote_hashes.insert(prev_tick, prev_hash);
                            }
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
            Packet::Ping { stamp } => {
                let now = self.now_ms;
                let known = match &mut self.role {
                    Role::Host { peers, .. } => peers.iter_mut().find(|p| p.addr == from).map(|p| p.last_recv_ms = now).is_some(),
                    _ => false,
                };
                if known {
                    self.send_packet(from, &Packet::Pong { stamp });
                }
            }
            Packet::Pong { stamp } => {
                let now = self.now_ms;
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(p) = peers.iter_mut().find(|p| p.addr == from) {
                        p.last_recv_ms = now;
                        p.pongs_back += 1;
                        p.rtt_sample((now as u32).wrapping_sub(stamp) as f64);
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
                        self.drop_slot(p.slot, &p.codename, reason, events);
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
                // The head counts relayed ticks too (they run ahead of `last_ready`).
                let host_head = self.head_tick();
                let join_tick = host_head.saturating_add(JOIN_MARGIN_TICKS);
                let mut announce: Option<(SocketAddr, u8, u32)> = None;
                let mut repeat: Option<(SocketAddr, u8, u32)> = None;
                if let Role::Host { peers, .. } = &mut self.role {
                    if let Some(p) = peers.iter_mut().find(|p| p.addr == from) {
                        if let Some(jt) = p.announced {
                            // A retransmitted JOIN_READY (the joiner resends until a
                            // JOIN_SLOT lands): the SAME tick again, to the joiner only.
                            // A second pick moved the join tick on some machines and
                            // not others (mp-online-failures: the reconnect test caught
                            // the slot arming at a later tick than first announced).
                            p.last_recv_ms = self.now_ms;
                            repeat = Some((p.addr, p.slot, jt));
                        } else if p.late_serve.is_none() && (p.late_joined || p.resyncing) && (p.slot as usize) < MAX_AGENTS {
                            let resyncing = p.resyncing;
                            p.announced = Some(join_tick);
                            p.last_recv_ms = self.now_ms;
                            p.our_ack = join_tick;
                            self.slot_human[p.slot as usize] = true;
                            self.join_from[p.slot as usize] = join_tick;
                            p.resyncing = false;
                            announce = Some((p.addr, p.slot, join_tick));
                            // net-desync-resync: the resyncing body never left the
                            // match — no join_history entry (a FUTURE late joiner's
                            // replay spawns it once, at its original join tick; the
                            // resync gap ran NEUTRAL, which the log's rows carry).
                            if !resyncing {
                                self.join_history.push((p.slot, join_tick));
                            } else if self.resync_watch.map(|(s, _, _)| s) == Some(p.slot) {
                                self.resync_watch = None; // the resync completed
                            }
                        }
                    }
                }
                if let Some((addr, slot, join_tick)) = repeat {
                    self.send_packet(addr, &Packet::JoinSlot { slot, join_tick });
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
        // Free slot? Host is slot 0; late joiners take 1..=MAX_AGENTS. A
        // RECONNECT — the codename of a slot's departed human, non-empty —
        // takes that slot back (its body still stands on NEUTRAL); anyone
        // else the first free slot [S mp-online-failures].
        let free: Option<u8> = match &self.role {
            Role::Host { peers, .. } => {
                let used: Vec<u8> = peers.iter().map(|p| p.slot).collect();
                let name: String = codename.chars().take(31).collect();
                let mine = (1..MAX_AGENTS as u8)
                    .find(|s| !used.contains(s) && !name.is_empty() && self.left_names[*s as usize] == name);
                mine.or_else(|| (1..MAX_AGENTS as u8).find(|s| !used.contains(s)))
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
            let mut p = Peer::new(from, slot, codename.chars().take(31).collect(), self.now_ms);
            // The joiner holds the whole log through the snapshot: the
            // live relay resumes at snapshot+1 (its own acks cannot say
            // so yet — it sends no inputs before its join tick).
            p.their_ack = Some(snap);
            p.sent_hi = Some(snap);
            p.go_confirmed = true; // never a GO for a late joiner
            p.late_serve = Some(LateServe { snapshot_tick: snap, snapshot_hash, chunk_count, sent: 0, acked: 0, last_send_ms: 0 });
            p.late_joined = true;
            peers.push(p);
            // The slot does NOT flip human here: everyone keeps riding
            // NEUTRAL for it until the JOIN_SLOT announcement.
        }
        self.send_welcome_to(from, slot);
        // The current human mask: the joiner's session starts [host] only and
        // must know which slots are live humans (its own slot stays out — it
        // is not human until the JOIN_SLOT announcement).
        self.send_packet(from, &Packet::SlotMap { human_slots: self.human_mask() });
        let joins: Vec<(u8, u32)> = self.join_history.iter().copied().filter(|(_, t)| *t <= snap).collect();
        self.send_packet(from, &Packet::JoinBegin { first_tick, snapshot_tick: snap, snapshot_hash, joins });
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
            // net-desync-resync: a RESYNC arriving at a client is a stray (only
            // the host serves one); ignore it.
            Packet::Resync { .. } => {}
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
                    // The WELCOME payload is the HOST's display codename.
                    *host_peer = Some(Peer::new(from, 0, codename.chars().take(31).collect(), self.now_ms));
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
                let _ = ack_bits;
                let now = self.now_ms;
                let per_tick = agent_mask.count_ones().max(1) as usize;
                let count = inputs.len() / per_tick;
                if let Role::Client { host_peer, .. } = &mut self.role {
                    if let Some(p) = host_peer {
                        p.last_recv_ms = now;
                        p.take_ack(ack_tick, ack_bits, now);
                        for k in 0..count {
                            p.recv_ticks.insert(base_tick.wrapping_add(k as u32));
                        }
                        if count > 0 {
                            p.ack_dirty = true;
                        }
                    }
                }
                for k in 0..count {
                    let t = base_tick.wrapping_add(k as u32);
                    let mut j = 0;
                    for s in 0..MAX_AGENTS {
                        let present = agent_mask & (1u16 << s) != 0;
                        let v = if present {
                            let v = inputs.get(k * per_tick + j).copied();
                            j += 1;
                            v
                        } else {
                            // protocol 5: a slot the relay left out is NEUTRAL over the
                            // covered ticks (the host omits slots it knows are NEUTRAL).
                            Some(NEUTRAL)
                        };
                        if s == self.local_slot as usize {
                            continue;
                        }
                        if let Some(v) = v {
                            self.inputs[s].insert(t, v);
                        }
                    }
                }
            }
            Packet::Ping { stamp } => {
                if let Role::Client { host_peer: Some(p), .. } = &mut self.role {
                    p.last_recv_ms = self.now_ms;
                }
                self.send_packet(from, &Packet::Pong { stamp });
            }
            Packet::Pong { stamp } => {
                let now = self.now_ms;
                if let Role::Client { host_peer: Some(p), .. } = &mut self.role {
                    p.last_recv_ms = now;
                    p.pongs_back += 1;
                    p.rtt_sample((now as u32).wrapping_sub(stamp) as f64);
                }
            }
            Packet::Delay { from_tick, delay } => {
                // net-internet: the host's delay announcement (re-sent until well past the
                // boundary — keep the first copy; an entry for the same tick is replaced).
                if let Role::Client { host_peer: Some(p), .. } = &mut self.role {
                    p.last_recv_ms = self.now_ms;
                }
                let d = (delay as u32).max(1);
                match self.delay_sched.iter().position(|e| e.0 >= from_tick) {
                    Some(i) if self.delay_sched[i].0 == from_tick => self.delay_sched[i].1 = d,
                    Some(i) => {
                        // An announcement older than one we hold arrived late: entries after it
                        // superseded it — keep them; insert it in order.
                        self.delay_sched.insert(i, (from_tick, d));
                    }
                    None => self.delay_sched.push((from_tick, d)),
                }
                let now_tick = self.next_run_tick();
                while self.delay_sched.len() > 4 && self.delay_sched[1].0 <= now_tick {
                    self.delay_sched.remove(0);
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
                } else if slot == 0 {
                    // The HOST left (its own LEAVE — the host never relays a
                    // slot-0 departure): the match and the lobby are gone. Report
                    // it now; waiting out the timeout froze the client for 5 s,
                    // and a host still polling its socket (a quit to the menus)
                    // kept the keepalives coming, so the timeout never fired.
                    if !self.host_left {
                        self.host_left = true;
                        events.push(Event::HostLeft);
                    }
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
            Packet::JoinBegin { first_tick, snapshot_tick, snapshot_hash, joins } => {
                // mp-join-in-progress: the host began the late-join handover.
                // The chunks follow; the log assembles on the last one.
                // net-desync-resync: also the resync serve — the request is
                // answered (the deadline stops; the catch-up runs until the
                // re-announce arms the slot again).
                self.resync_pending = false;
                self.resync_catchup = true;
                self.join_history = joins;
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
                // The joiner hears the announcement twice (the direct send + the
                // broadcast, and again per retransmitted JOIN_READY): arm once.
                let dup = (slot as usize) < MAX_AGENTS
                    && self.slot_human[slot as usize]
                    && self.join_from[slot as usize] == join_tick;
                if dup {
                    self.late_confirm_due = None;
                    if let Role::Client { host_peer: Some(p), .. } = &mut self.role {
                        p.last_recv_ms = self.now_ms;
                    }
                } else if (slot as usize) < MAX_AGENTS {
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
                    if slot == self.local_slot {
                        self.resync_catchup = false; // net-desync-resync: playing again
                    }
                    events.push(Event::LateJoinSlot { slot, join_tick });
                }
            }
            Packet::Hello { .. } => {}
            // Host-directed handshake packets never arrive at a client.
            Packet::JoinAck { .. } | Packet::JoinReady {} => {}
        }
    }

    /// Host: remove a human from `slot` (drop policy applies).
    fn drop_slot(&mut self, slot: u8, codename: &str, reason: LeaveReason, events: &mut Vec<Event>) {
        if slot == 0 || slot as usize >= MAX_AGENTS {
            return;
        }
        let s = slot as usize;
        // net-desync-resync: a resync watch on a slot that LEFT dies with it —
        // the match runs on over the slot's NEUTRAL (the Neutral policy), the
        // watch must not end the match for a peer that is already gone.
        if self.resync_watch.map(|(ws, _, _)| ws) == Some(slot) {
            self.resync_watch = None;
        }
        // The drop tick: every tick up to the last one the host ran used the
        // leaver's real inputs (they keep relaying — a client behind the host
        // still needs them); from the next tick on the slot is NEUTRAL. Only a
        // slot that WAS human moves the boundary (a late joiner that never
        // reached its join tick drove nothing).
        // The relay may already carry ticks past `last_ready` (`relay_horizon`): those went
        // out with the leaver's real inputs, so the boundary is past them too.
        let next = if self.last_ready.is_some() || self.committed_hi.is_some() {
            self.head_tick().wrapping_add(1)
        } else {
            self.start_tick
        };
        if self.slot_human[s] {
            self.left_at[s] = next.max(self.join_from[s]);
        }
        let keep_below = self.left_at[s];
        self.inputs[s].retain(|t, _| *t < keep_below);
        self.left_names[s] = codename.chars().take(31).collect();
        self.slot_human[s] = false;
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
            let mut dropped: Vec<(u8, String, LeaveReason)> = Vec::new();
            peers.retain(|p| {
                if p.connected && now.saturating_sub(p.last_recv_ms) > timeout {
                    dropped.push((p.slot, p.codename.clone(), LeaveReason::Timeout));
                    false
                } else {
                    true
                }
            });
            for (slot, name, reason) in dropped {
                self.drop_slot(slot, &name, reason, events);
            }
            return;
        }
        // Client: host silence after connection.
        if !self.host_timed_out && !self.host_left {
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
        // (slot, hashes) per peer: the host arms its resync watch per slot; a
        // client has only the host peer.
        let peers_hashes: Vec<(u8, HashMap<u32, u64>)> = match &mut self.role {
            Role::Host { peers, .. } => peers.iter().map(|p| (p.slot, p.remote_hashes.clone())).collect(),
            Role::Client { host_peer, .. } => host_peer
                .as_ref()
                .map(|p| vec![(p.slot, p.remote_hashes.clone())])
                .unwrap_or_default(),
        };
        let now = self.now_ms;
        let timeout = self.cfg.timeout_ms;
        for (slot, remote) in peers_hashes {
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
                        // net-desync-resync: the host is the authority (it owns
                        // the input log a resync replays), so a mismatching
                        // CLIENT resyncs — it asks and replays; the host arms
                        // its watch and keeps playing. The fallbacks: a second
                        // mismatch inside the resync window (client), a request
                        // never served inside `timeout_ms` (client), a watch
                        // that never got its request (host), or a second slot
                        // desyncing while one resync is in flight (host) — all
                        // fatal, the pre-resync behaviour.
                        match &self.role {
                            Role::Host { .. } => match self.resync_watch {
                                None => self.resync_watch = Some((slot, tick, now + timeout)),
                                Some((s, _, _)) if s == slot => {}
                                Some(_) => events.push(Event::DesyncFatal { tick }),
                            },
                            Role::Client { .. } => {
                                // A mismatch while this machine is resyncing (pending, or
                                // replaying/catching up) is the SAME unresolved divergence:
                                // the stale hashes of the diverged run are compared as the
                                // peer's hashes arrive, and the replay's re-submissions have
                                // not covered them yet. Ignore until the slot is live again;
                                // the fatal cases: a NEW mismatch after the re-announce but
                                // inside the 60 s window, or the request timing out.
                                if self.resync_pending || self.resync_catchup {
                                } else if self
                                    .last_resync_ms
                                    .map(|t| now.saturating_sub(t) < RESYNC_RETRY_WINDOW_MS)
                                    .unwrap_or(false)
                                {
                                    events.push(Event::DesyncFatal { tick });
                                } else {
                                    self.last_resync_ms = Some(now);
                                    self.resync_pending = true;
                                    self.resync_tick = tick;
                                    self.resync_req_ms = 0; // send on this poll
                                    self.resync_deadline_ms = now + timeout;
                                }
                            }
                        }
                        if std::env::var_os("NF_RESYNC_DBG").is_some() {
                            eprintln!("[dbg-mismatch] host={} tick {tick} local {lhash:016x} remote {rhash:016x}", matches!(self.role, Role::Host { .. }));
                        }
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

    /// net-internet: the fault line with a one-way delay and jitter (ms on the session's
    /// clock) — this machine's outgoing half of a simulated internet path.
    pub fn inject_link_delay(&mut self, seed: u64, drop_per_mille: u32, reorder_per_mille: u32, delay_ms: u64, jitter_ms: u64) {
        self.transport.inject_link(seed, drop_per_mille, reorder_per_mille, delay_ms, jitter_ms);
    }
}
