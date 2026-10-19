//! UDP lockstep multiplayer session for 007: Nightfire (PS2 NTSC-U) port.
//!
//! This crate owns the *session* layer only: handshake, per-tick input
//! exchange, state-hash exchange, stall and timeout handling. It has no
//! dependency on the game sim (`nf-game`) and no rendering; the host app
//! wires it in (see `docs/systems/net-lockstep.md`):
//!
//! * the match seed (`u32`) received on the client is fed to
//!   `nf_game::drone::Rng::new(seed)` (the game's own RNG, bit-exact,
//!   [M] `Rand_Rand` 0x1e3738);
//! * `MatchSettings` carries the host's `MpSettings` values (field mapping
//!   documented in [`settings`]);
//! * the per-tick state hash is an opaque `u64` the caller reports via
//!   [`Session::submit_hash`] — task 050's `state_hash(&MpWorld)` wires in
//!   there once it lands (it does not exist yet, checked 2026-10-15);
//! * [`Input`] maps to `nf_game::player::Input` + the separate
//!   `trigger/reload/use_key` args of `Sim::step_bond` (mapping documented).
//!
//! Determinism rules: no wall clock anywhere — every [`Session::poll`] takes
//! a logical `now_ms` supplied by the caller (real time in the app, simulated
//! time in tests), and the bit-exact RNG is never touched by this crate.
//!
//! Wire protocol summary (little-endian, see [`proto`]):
//! 4-byte header `4E 46 <ptype> <reserved>`, then per-type payloads.
//! Star topology: clients talk only to the host; the host relays.

pub mod hash;
pub mod input;
pub mod lobby;
pub mod net;
pub mod p2p;
pub mod pace;
pub mod discovery;
pub mod directory;
pub mod proto;
pub mod session;
pub mod settings;
pub mod shim;
pub mod teams;

pub use hash::Hasher;
pub use input::{Input, NEUTRAL};
pub use lobby::{LobbyBot, LobbyEdit, LobbyPlayer, LobbyState};
pub use net::UdpTransport;
pub use session::{AgentInfo, Config, DropPolicy, Event, LeaveReason, RefuseReason, Session, SessionError};
pub use settings::MatchSettings;
pub use teams::{scenario_has_teams, start_check, team_label, StartError, TeamEdit, TeamSelection};

/// Maximum agents in an MP match. The GAME's own limit is 8 slots — humans
/// occupy slots 0..4 (`MpSettings.human_count`, bots 4..8), `MP_Start`
/// 0x183a50 inits humans into slots 0..player_count-1 and bots into
/// 4..4+bot_count-1 [M] (`work/decomp` MP_Start;
/// `crates/nf-game/src/mp/mod.rs:59,664`; `docs/atlas/multiplayer.md`
/// "MP 1-4"). The PORT extends the cap to 16 participants, any mix of humans
/// and bots (humans 1..16, bots 0..15, humans + bots ≤ 16) [S — the user's
/// request; every per-slot rule stays the game's, only the counts grow]. The
/// slot ORDER stays the game's shape: humans 0..humans-1, bots follow
/// (humans..humans+bots-1) — the game's order, extended [S].
pub const MAX_AGENTS: usize = 16;

/// Wire protocol version. Bumped on any incompatible packet-layout change.
/// 2: settings 208 B (per-player radar_on + registered_pickups), the HELLO
/// codename is a display name (not compared), REFUSE 3 = started, DATA added
/// (net-consolidate 2026-10-16).
/// 3: the input carries `held` (byte 21) + `shot` (flag-1 bit 2) — the online combat (700v6).
/// 4: mp-16-slots — 16 agents: the INPUT packet's `agent_mask` and the
/// SLOTMAP's `human_slots` are u16, the WELCOME settings carry 16 per-player
/// records (334 B), the INPUT header grows to 15 bytes, JOIN_LOG chunks
/// carry 3 ticks (16 agents × 24 B × 3 = 1152 B) and MAX_INPUT_COUNT_H2C
/// drops to 3 so the largest relay packet still fits [`MAX_PACKET`] [S].
/// 5: mp-combat-weapons — the input grows to 28 bytes: the shot description
/// (byte 22 the fired record, byte 23 the shot flags, bytes 24..28 the aim
/// yaw/pitch as i16 angles). JOIN_LOG chunks stay 3 ticks (1344 B of inputs).
/// 6: net-internet (compact delta inputs, adaptive delay) + mp-combat-weapons (28-byte input)
/// merged — the compact encoding runs over the 28-byte layout (7 words).
/// 7: mp-lobby-settings merged — the settings grow a tail (the four enviro-mod
/// words + the prepared bot records) so every lobby setting reaches the clients'
/// matches [S]; Location Damage stays the 20th scalar.
/// 8: net-desync-detail — PT_HASHCOMPS/PT_HASHDETAIL (the desync diagnosis exchange). The
/// combined state hash also became the fold of the named component hashes
/// (`nf_view::scene::fold_component_hashes`), so hashes from a protocol-7 build never match an
/// 8 build's; the handshake refuses that mix.
pub const PROTOCOL_VERSION: u16 = 8;

/// Build id carried in the handshake so mismatched executables refuse cleanly.
/// The host app sets this (e.g. a build counter); both sides must match exactly.
pub const DEFAULT_BUILD_ID: u32 = 1;

/// Largest packet we ever build or accept (fits a 1500-byte MTU with headroom).
pub const MAX_PACKET: usize = 1400;

/// Magic u16 at the start of every packet: 'N','F'.
pub const MAGIC: u16 = 0x4E46;
