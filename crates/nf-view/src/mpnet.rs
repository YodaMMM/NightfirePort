//! The ONE session layer's app surface (net-consolidate 2026-10-16).
//!
//! There were two UDP sessions (`nf-net` and `nf_game::net`);
//! this consolidation keeps the `nf-net` crate — the stronger protocol: a contiguous
//! ack + 32-tick bitmap so the full unacked window rides every packet (an
//! isolated loss can never lose a tick), a configurable input delay 2..=4
//! carried in WELCOME, hash exchanges with the previous hash piggybacking on
//! every HASH packet, keepalives, GO retransmit, and the deterministic shim
//! that makes the lossy tests bit-reproducible. 053's unique features moved
//! in: the full `MpSettings` fidelity on the wire (per-player `radar_on`,
//! `registered_pickups`), the lobby display codename + `agents()` view, the
//! refuse-when-started gate, the FNV `Hasher` (now `nf_net::hash::Hasher`)
//! and the `MpSettings` match handover this module drives (053's START: the
//! host's final settings ride a tagged DATA tunnel — the WELCOME at join time
//! cannot know them, the confirm page arms later).
//!
//! `NetCore` owns the `nf_net::Session<UdpTransport>`; `SessionProxy` is the
//! cheap handle both the front end (`FrontEnd::mp_session`, the lobby's
//! `MpSession` trait object) and the app (`App.net`, the lockstep tick gate)
//! share — one core, two drivers, exactly the wiring 052's report promised.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use nf_game::mp::{MpPlayer, MpSettings};
use nf_game::player;
use nf_net::session::{Config, Event, Session};
use nf_net::{MatchSettings, UdpTransport, MAX_AGENTS};

use crate::menu::{MpNetAgent, MpNetChoices, MpNetError, MpNetLink};

/// net-desync-detail: the component count `PT_HASHCOMPS` carries (the app's fold pins it —
/// `crate::scene::NET_HASH_COMPONENTS`).
pub const HASH_COMPONENTS: usize = nf_net::proto::HASH_COMPONENTS;

// ── conversions ──────────────────────────────────────────────────────────────

/// `MpSettings` → the wire's `MatchSettings` (field mapping documented in
/// nf-net's `settings` module; the seed rides the handshake separately, so it
/// is not in `MatchSettings`).
pub fn mp_to_match(s: &MpSettings) -> MatchSettings {
    MatchSettings {
        started: s.started as u32,
        gametype_mask: s.gametype_mask,
        player_count: s.player_count as u32,
        master: s.master,
        master_copy: s.master_copy,
        team_game: s.team_game,
        bit30: s.bit30,
        total_players: s.total_players,
        frag_limit: s.frag_limit,
        time_limit: s.time_limit,
        scenario: s.scenario,
        level_code: s.level_code,
        human_count: s.human_count,
        bot_count: s.bot_count,
        weapon_set: s.weapon_set,
        capture_reward: s.capture_reward,
        spawn_policy: s.spawn_policy,
        radar: s.radar,
        friendly_fire: s.friendly_fire,
        location_damage: s.location_damage,
        players: s.players.each_ref().map(|p| (p.team, p.skin, p.armor_bonus, p.radar_on)),
        registered_pickups: s.registered_pickups,
        mods: [s.gun_emplacements, s.vehicles, s.grapple, s.explosive_scenery],
        bot_records: s.bot_records.iter().map(|r| r.0).collect(),
    }
}

/// `MatchSettings` + the handshake's seed → the app's `MpSettings`
/// (`load_mp_carry` builds the level from it; the seed feeds the bit-exact
/// `Rng::new` on every machine).
pub fn match_to_mp(m: &MatchSettings, seed: u32) -> MpSettings {
    MpSettings {
        players: m.players.map(|(team, skin, armor_bonus, radar_on)| MpPlayer {
            team, skin, armor_bonus, radar_on, codename: String::new(),
        }),
        started: m.started != 0,
        gametype_mask: m.gametype_mask,
        player_count: m.player_count as u16,
        master: m.master,
        master_copy: m.master_copy,
        team_game: m.team_game,
        bit30: m.bit30,
        total_players: m.total_players,
        frag_limit: m.frag_limit,
        time_limit: m.time_limit,
        scenario: m.scenario,
        level_code: m.level_code,
        human_count: m.human_count,
        bot_count: m.bot_count,
        weapon_set: m.weapon_set,
        capture_reward: m.capture_reward,
        spawn_policy: m.spawn_policy,
        radar: m.radar,
        registered_pickups: m.registered_pickups,
        friendly_fire: m.friendly_fire,
        location_damage: m.location_damage,
        gun_emplacements: m.mods[0],
        vehicles: m.mods[1],
        grapple: m.mods[2],
        explosive_scenery: m.mods[3],
        bot_records: m.bot_records.iter().map(|r| nf_game::mp::bots::BotRecord(*r)).collect(),
        seed,
    }
}

/// The app's per-tick input + the three separate `step_bond` args → the
/// 24-byte wire input (nf-net's `Input` is the superset: it carries
/// `trigger`/`reload`/`use_key`, which 053's 20-byte packing dropped).
pub fn to_net_input(i: &player::Input, trigger: bool, reload: bool, use_key: bool) -> nf_net::Input {
    nf_net::Input {
        forward: i.forward,
        strafe: i.strafe,
        turn: i.turn,
        look: i.look,
        jump: i.jump,
        crouch: i.crouch,
        aiming: i.aiming,
        zoom: i.zoom,
        scope: i.scope,
        fire: i.fire,
        mode_switch: i.mode_switch,
        trigger,
        reload,
        use_key,
        weapon_cycle: i.weapon_cycle,
        gadget_cycle: i.gadget_cycle,
        switch_to: i.switch_to,
        // 700v6: the caller fills `held` / `shot` from the local weapon pipeline (lib.rs);
        // mp-combat-weapons: and the shot description.
        held: 0,
        shot: false,
        shot_weapon: 0,
        shot_flags: 0,
        shot_aim: [0, 0],
    }
}

/// The wire input → the sim's `player::Input` (the trigger/reload/use_key
/// lanes are the local machine's own — a remote body ticks `tick_ctx`, which
/// consumes the movement lanes only, exactly like the match-host cut did).
pub fn to_player_input(i: &nf_net::Input) -> player::Input {
    player::Input {
        forward: i.forward,
        strafe: i.strafe,
        turn: i.turn,
        look: i.look,
        jump: i.jump,
        crouch: i.crouch,
        aiming: i.aiming,
        zoom: i.zoom,
        scope: i.scope,
        fire: i.fire,
        // Zero-g's lift/stabilise lanes are local view state (the `zoom_cycle` precedent): the
        // net wire format is unchanged and remotes never tick the zero-g substates today.
        lift: 0.0,
        stabilise: false,
        goggles_action: false,
        // The pack-flight toggle is a local-view action edge (the `stabilise` precedent): the
        // wire format is unchanged and remotes never tick the fly substate today.
        fly_toggle: false,
        // options-census: not net-encoded (the wire has no mode lane); the drain sets the
        // real value for the local player each tick, the game default (Toggle) for the remote arm.
        crouch_hold: false,
        mode_switch: i.mode_switch,
        weapon_cycle: i.weapon_cycle,
        gadget_cycle: i.gadget_cycle,
        switch_to: i.switch_to,
        // microcam-wheel-zoom: the wheel's zoom steps are local view input, never on the wire.
        zoom_cycle: 0,
    }
}

// ── the DATA tunnel codec ────────────────────────────────────────────────────
// tag 0: the host's lobby choices (13 words). tag 1: the match-start payload
// (208 B MatchSettings + the seed) — 053's START packet, riding the tunnel.

impl MpNetChoices {
    fn encode(&self) -> Vec<u8> {
        let mut o = vec![0u8];
        for v in [
            self.scenario, self.level, self.weapon_set, self.radar, self.friendly_fire,
            self.frag_limit as u32, self.time_limit as u32,
            self.player_mods[0], self.player_mods[1],
            self.envmods[0], self.envmods[1], self.envmods[2], self.envmods[3], self.envmods[4],
        ] {
            o.extend_from_slice(&v.to_le_bytes());
        }
        o
    }
    fn decode(b: &[u8]) -> Option<MpNetChoices> {
        if b.len() < 1 + 13 * 4 { return None; }
        let w = |i: usize| u32::from_le_bytes([b[1 + i * 4], b[2 + i * 4], b[3 + i * 4], b[4 + i * 4]]);
        Some(MpNetChoices {
            scenario: w(0),
            level: w(1),
            weapon_set: w(2),
            radar: w(3),
            friendly_fire: w(4),
            frag_limit: w(5) as i32,
            time_limit: w(6) as i32,
            player_mods: [w(7), w(8)],
            envmods: [w(9), w(10), w(11), w(12), w(13)],
        })
    }
}

/// net-desync-detail (public: the desync report's `match-settings.bin` is this payload): the
/// tag-1 match-start payload — the 208-byte MatchSettings + the seed.
pub fn encode_match_start(m: &MatchSettings, seed: u32) -> Vec<u8> {
    let mut o = vec![1u8];
    let mut buf = [0u8; nf_net::settings::SETTINGS_WIRE_SIZE];
    m.encode(&mut buf);
    o.extend_from_slice(&buf);
    o.extend_from_slice(&seed.to_le_bytes());
    o
}

/// net-desync-detail (public: `--bug-replay` of a desync report rebuilds the match from this
/// payload): decodes the tag-1 match-start payload — the MatchSettings + the seed.
pub fn decode_match_start(b: &[u8]) -> Option<(MatchSettings, u32)> {
    if b.len() < 1 + nf_net::settings::SETTINGS_WIRE_SIZE + 4 || b[0] != 1 {
        return None;
    }
    let m = MatchSettings::decode(&b[1..1 + nf_net::settings::SETTINGS_WIRE_SIZE])?;
    let off = 1 + nf_net::settings::SETTINGS_WIRE_SIZE;
    let seed = u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]]);
    Some((m, seed))
}

// ── the core + proxy ─────────────────────────────────────────────────────────

/// What the app must react to (the old `NetEvent` arms).
#[derive(Clone, Debug)]
pub enum CoreEvent {
    /// Host: a client took a slot (the flag path's auto-start cue).
    AgentJoined { slot: u8 },
    /// The match is starting: the host on arm, the client on the tunnel's
    /// match payload. The app loads the level from it (053's MatchStarted).
    MatchStarted(Box<MpSettings>),
    /// A human slot's client left (Neutral policy: the match runs on, the
    /// game's own disconnected-pad shape).
    ClientLeft { slot: u8, graceful: bool },
    /// The state hashes diverged at `tick` (both hashes ride for the log; the
    /// session's resync policy decides recover-vs-fatal — net-desync-resync).
    Desync { tick: u32, local: u64, remote: u64 },
    /// net-desync-resync: the desync is not recoverable (a second mismatch
    /// inside the 60 s window, an unserved request, an unanswered watch) — the
    /// app shows the pre-resync box and returns to the lobby.
    DesyncFatal { tick: u32 },
    /// net-desync-detail: the peer's component hashes for the desync tick arrived
    /// (`PT_HASHCOMPS`) — the desync watcher pairs them with our own.
    HashComps { tick: u32, comps: [u64; HASH_COMPONENTS] },
    /// net-desync-detail: the peer's per-item breakdown of `component` for the desync tick
    /// arrived (`PT_HASHDETAIL`) — completes the report's slot diagnosis.
    HashDetail { tick: u32, component: u8, items: Vec<u64> },
    /// Client: the host went silent past the timeout.
    HostTimeout,
    /// Client: the host LEFT (quit the match / closed the lobby) — at once,
    /// not after the timeout. The app returns to the online pages with a
    /// message [S mp-online-failures].
    HostLeft,
    /// Client: the host refused the join.
    Refused,
    /// mp-join-in-progress. Host: a HELLO arrived after GO; the run-log
    /// transfer to that peer began (the match runs on untouched).
    LateJoinStarted { slot: u8 },
    /// mp-join-in-progress. The late joiner's log transfer completed: the app
    /// loads the level from `settings` (the WELCOME's handover), replays
    /// `log` headless (ticks `0..=snapshot_tick` —
    /// `crate::mp_late_join_catchup`), and MUST produce `snapshot_hash` at
    /// `snapshot_tick` — then `NetCore::confirm_late_join` (or
    /// `late_join_verify_failed` on a mismatch).
    LateJoinHandover {
        slot: u8,
        settings: Box<MpSettings>,
        snapshot_tick: u32,
        snapshot_hash: u64,
        log: Vec<[nf_net::Input; MAX_AGENTS]>,
        /// The earlier late joins (slot, join tick) the replay spawns
        /// (`crate::mp_late_join_catchup_joins`) — mp-online-failures.
        joins: Vec<(u8, u32)>,
    },
    /// mp-join-in-progress. All machines: `slot` becomes human from
    /// `join_tick` — the app spawns its agent there (`Sim::mp_join_spawn`)
    /// and the joiner adopts the slot as its own view.
    LateJoinSlot { slot: u8, join_tick: u32 },
    /// net-match-end: a machine ran its own handover — it simulated through its final tick and
    /// opened the debriefing (`slot`'s own board folds to `board_hash`; `final_tick` is the sim
    /// frame it stopped at). The host collects these (every human peer, or a departure) before it
    /// lets the finished session go [S port protocol — the disc has no online].
    DebriefAck { slot: u8, final_tick: u32, board_hash: u64 },
}

/// The shared session state: one per match/lobby. See the module docs.
pub struct NetCore {
    sess: Session<UdpTransport>,
    is_host: bool,
    /// The session's logical clock (wall ms since the core's birth — the
    /// session layer sits OUTSIDE the Sim, so the determinism rule holds).
    epoch: Instant,
    /// The `MpSettings` handover the lobby's `match_started()` reads (the
    /// client mirrors it into `mp_client_start`; the host's own handover is
    /// the confirm page's, this is set on arm for symmetry).
    match_start: Option<MpSettings>,
    /// The host's last published lobby choices (the client's mirror source).
    choices: Option<MpNetChoices>,
    refused: bool,
    host_timed_out: bool,
    /// Client: the host sent LEAVE (the link reads Lost at once).
    host_left: bool,
    closed: bool,
    /// Events for the app (`take_app_events`).
    events: Vec<CoreEvent>,
    /// Host: the match-start payload (tag 1) and the polls left to repeat it. The DATA tunnel is
    /// unacknowledged, so a single send is lost with the link's loss rate (the 5 % e2e lost it);
    /// the host repeats it every `START_RESEND_EVERY` polls for `START_RESEND_POLLS` polls and
    /// the client keeps the first copy [S port protocol].
    start_resend: Option<(Vec<u8>, u32)>,
    /// net-p2p: internet reachability — None until the app enables it (`enable_reach`); tests that
    /// build cores directly never probe the LAN gateway or a public STUN server.
    reach_cfg: Option<nf_net::p2p::ReachConfig>,
    reach_host: Option<nf_net::p2p::HostReach>,
    reach_join: Option<nf_net::p2p::JoinReach>,
    /// The last reach status line printed to the console.
    reach_line: Option<String>,
    /// net-lobby-list: what this host answers discovery queries with (None = not advertising).
    advert: Option<nf_net::discovery::LobbyInfo>,
    /// net-lobby-list: the joiner's LAN browser while the Join Online page is up.
    browse: Option<nf_net::discovery::LobbyList>,
    /// The next REGISTER to the lobby directory (ms on the core's clock).
    dir_next_ms: u64,
    /// net-short-codes: the host's short-code bookkeeping (None until the first REGISTER goes
    /// out; the code shows while the server keeps answering) [S docs/systems/net-short-codes.md].
    dir_code: Option<nf_net::directory::ShortCodeHost>,
    /// net-short-codes: the joiner's LOOKUP for a 4-character code, before any dial [S].
    pending_join: Option<nf_net::directory::ShortLookup>,
    /// net-short-codes: the joiner's short-code status line for the join page (the note getter
    /// hands it to the draw code) [S].
    short_note: Option<String>,
    /// net-cloud-lobby: the lobby service's URL when the cloud meeting point is on (None = off —
    /// unset, or a player-set nf-lobbyd overrides it) [S docs/systems/net-cloud-lobby.md].
    cloud_cfg: Option<String>,
    /// net-cloud-lobby: the host's register/refresh/leave client (None until the first refresh
    /// period; the code shows while the service keeps answering) [S].
    cloud_host: Option<nf_net::cloud::CloudHost>,
    /// net-cloud-lobby: the next POST /v1/host (ms on the core's clock).
    cloud_next_ms: u64,
    /// net-cloud-lobby: the joiner's public-lobby list while the Join Online page is open [S].
    cloud_browse: Option<nf_net::cloud::CloudBrowse>,
    /// net-cloud-lobby: the joiner's code lookup through the service, before any dial [S].
    cloud_pending: Option<nf_net::cloud::CloudLookup>,
    /// net-cloud-lobby: the host's public/private toggle — public lists, private is code only
    /// (lobbies are public by default) [S].
    pub_lobby: bool,
    /// mp-lobby-one-screen: the host's last published snapshot / the client's last received
    /// one, and the arrived client edits the host has not drained [S §6.4].
    lobby: Option<nf_net::LobbyState>,
    lobby_edits: Vec<nf_net::LobbyEdit>,
}

/// The host repeats the match-start payload every this many polls …
const START_RESEND_EVERY: u32 = 4;
/// … for this many polls after the arm (≈ 2 s of 60 Hz frames).
const START_RESEND_POLLS: u32 = 120;

fn session_cfg() -> Config {
    // NET_HASH_EVERY's cadence; net-internet: the host measures every peer's RTT and adapts
    // the input delay (the app schedules through `submit_local`, which follows it).
    Config { hash_every: 15, adaptive_delay: true, ..Default::default() }
}

impl NetCore {
    /// Host: bind `0.0.0.0:port` and listen.
    pub fn host(port: u16) -> Result<Rc<RefCell<NetCore>>, String> {
        let tr = UdpTransport::bind(&format!("0.0.0.0:{port}")).map_err(|e| format!("bind {port}: {e}"))?;
        Ok(Rc::new(RefCell::new(NetCore {
            sess: Session::new_host(session_cfg(), tr, MatchSettings::default(), 0),
            is_host: true,
            epoch: Instant::now(),
            match_start: None,
            choices: None,
            refused: false,
            host_timed_out: false,
            host_left: false,
            closed: false,
            events: Vec::new(),
            start_resend: None,
            reach_cfg: None,
            reach_host: None,
            reach_join: None,
            reach_line: None,
            advert: None,
            browse: None,
            dir_next_ms: 0,
            dir_code: None,
            pending_join: None,
            short_note: None,
            cloud_cfg: None,
            cloud_host: None,
            cloud_next_ms: 0,
            cloud_browse: None,
            cloud_pending: None,
            pub_lobby: true,
            lobby: None,
            lobby_edits: Vec::new(),
        })))
    }

    /// The lobby's re-bind, host side: a fresh listening Session on the SAME
    /// core, so every handle to this core (the app's `net`, the front end's
    /// `MpSession`, a test's clone) keeps working (net-e2e-loopback — the old
    /// replace-the-Rc shape left the other holders on the stale placeholder
    /// socket). The lobby calls it only before anything flows.
    pub fn rebind_host(&mut self, port: u16) -> Result<(), String> {
        // mp-online-failures: the old session's socket goes FIRST — a core re-listening on the
        // port it already holds (Back then Host again, the rematch's re-listen) could never bind
        // while its own old socket stood.
        self.release_socket();
        let tr = UdpTransport::bind(&format!("0.0.0.0:{port}")).map_err(|e| format!("bind {port}: {e}"))?;
        self.sess = Session::new_host(session_cfg(), tr, MatchSettings::default(), 0);
        self.is_host = true;
        self.epoch = Instant::now();
        self.drop_reach();
        if let Some(cfg) = self.reach_cfg.clone() {
            self.reach_host = Some(nf_net::p2p::HostReach::start(port, cfg, reach_seed()));
        }
        self.match_start = None;
        self.choices = None;
        self.refused = false;
        self.host_timed_out = false;
        self.host_left = false;
        self.closed = false;
        self.events.clear();
        self.start_resend = None;
        self.lobby = None;
        self.lobby_edits.clear();
        // net-short-codes: a re-listen binds a NEW socket, so the directory's entry (keyed by the
        // source address) is stale — the code goes and the next REGISTER asks for a fresh one.
        self.dir_code = None;
        self.pending_join = None;
        self.short_note = None;
        // net-cloud-lobby: the same for the cloud entry (keyed by our IP + port) — a fresh code
        // arrives with the next refresh period; a leave for the old entry would delete the new
        // one, so the client object is dropped without a DELETE (it expires server-side anyway).
        self.cloud_host = None;
        self.cloud_next_ms = 0;
        self.cloud_pending = None;
        self.cloud_browse = None;
        Ok(())
    }

    /// mp-online-failures: swap the session for an unbound-port placeholder (an ephemeral
    /// loopback socket), dropping the old one — its UDP port is free at once. Nothing is sent.
    fn release_socket(&mut self) {
        if let Ok(tr) = UdpTransport::bind("127.0.0.1:0") {
            self.sess = Session::new_host(session_cfg(), tr, MatchSettings::default(), 0);
        }
    }

    /// The lobby's re-bind, client side: dial `addr` on the SAME core (see
    /// `rebind_host`).
    pub fn rebind_client(&mut self, addr: std::net::SocketAddr) -> Result<(), String> {
        let tr = UdpTransport::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
        self.sess = Session::new_client(session_cfg(), tr, addr);
        self.is_host = false;
        self.epoch = Instant::now();
        self.drop_reach();
        self.match_start = None;
        self.choices = None;
        self.refused = false;
        self.host_timed_out = false;
        self.host_left = false;
        self.closed = false;
        self.events.clear();
        self.start_resend = None;
        self.lobby = None;
        self.lobby_edits.clear();
        self.pending_join = None;
        self.short_note = None;
        self.cloud_pending = None;
        self.cloud_browse = None;
        Ok(())
    }

    /// Client: dial `addr` ("host:port").
    pub fn connect(addr: &str) -> Result<Rc<RefCell<NetCore>>, String> {
        let a: std::net::SocketAddr = addr.parse().map_err(|e| format!("resolve {addr}: {e}"))?;
        let tr = UdpTransport::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
        Ok(Rc::new(RefCell::new(NetCore {
            sess: Session::new_client(session_cfg(), tr, a),
            is_host: false,
            epoch: Instant::now(),
            match_start: None,
            choices: None,
            refused: false,
            host_timed_out: false,
            host_left: false,
            closed: false,
            events: Vec::new(),
            start_resend: None,
            reach_cfg: None,
            reach_host: None,
            reach_join: None,
            reach_line: None,
            advert: None,
            browse: None,
            dir_next_ms: 0,
            dir_code: None,
            pending_join: None,
            short_note: None,
            cloud_cfg: None,
            cloud_host: None,
            cloud_next_ms: 0,
            cloud_browse: None,
            cloud_pending: None,
            pub_lobby: true,
            lobby: None,
            lobby_edits: Vec::new(),
        })))
    }

    /// net-p2p: probe internet reachability on every later listen / dial (the app's real
    /// sessions; never the tests' direct cores).
    pub fn enable_reach(&mut self, cfg: nf_net::p2p::ReachConfig) {
        self.reach_cfg = Some(cfg);
    }

    /// net-cloud-lobby: the cloud meeting point's URL (None = off — unset, or a player-set
    /// nf-lobbyd overrides it). Tests point it at their local mock server [S].
    pub fn enable_cloud(&mut self, url: Option<String>) {
        self.cloud_cfg = url;
        self.cloud_host = None;
        self.cloud_next_ms = 0;
    }

    /// net-cloud-lobby: the host's public/private toggle (public lists, private is code only).
    pub fn set_public_lobby(&mut self, public: bool) {
        self.pub_lobby = public;
    }

    pub fn public_lobby(&self) -> bool {
        self.pub_lobby
    }

    /// Client: dial a typed target — a 4-character code, a join code or `a.b.c.d:port` — on the
    /// SAME core, starting "my code" discovery when reach is enabled. A 4-character code is not
    /// dialled here: the lookup (the cloud service first, else the player's nf-lobbyd) runs in
    /// `poll` and dials what it found (net-short-codes, net-cloud-lobby) [S].
    pub fn dial(&mut self, typed: &str) -> Result<(), String> {
        match nf_net::p2p::parse_join_target(typed).map_err(|e| format!("{e:?}"))? {
            nf_net::p2p::JoinTarget::Short(code) => {
                let code_s = nf_net::directory::short_code_string(&code);
                // net-cloud-lobby: the cloud service is the default meeting point — a 4-character
                // code is looked up over HTTPS and dialled when the answer lands (in poll) [S].
                if let Some(url) = self.cloud_cfg.clone() {
                    self.short_note = Some(format!("Looking up {code_s} on the lobby service..."));
                    self.cloud_pending = Some(nf_net::cloud::CloudLookup::new(url, &code_s));
                    return Ok(());
                }
                let server = self
                    .reach_cfg
                    .as_ref()
                    .and_then(|c| c.directory)
                    .ok_or_else(|| {
                        self.short_note = Some("No lobby server is set - 4-character codes need one (net.txt)".to_string());
                        "no lobby server set".to_string()
                    })?;
                self.short_note = Some(format!("Looking up {} on the lobby server...", nf_net::directory::short_code_string(&code)));
                self.pending_join = Some(nf_net::directory::ShortLookup::new(server, code, reach_seed() as u32));
                Ok(())
            }
            nf_net::p2p::JoinTarget::Address(a) => {
                self.rebind_client(std::net::SocketAddr::V4(a))?;
                if let Some(cfg) = self.reach_cfg.clone() {
                    self.reach_join = nf_net::p2p::JoinReach::start(typed, &cfg, reach_seed()).ok();
                }
                Ok(())
            }
            nf_net::p2p::JoinTarget::Code(jc) => {
                self.rebind_client(std::net::SocketAddr::V4(jc.addr))?;
                if let Some(cfg) = self.reach_cfg.clone() {
                    self.reach_join = nf_net::p2p::JoinReach::start(typed, &cfg, reach_seed()).ok();
                }
                Ok(())
            }
        }
    }

    /// net-p2p: stop probing; a host's mapping is deleted on a background thread (the UPnP/NAT-PMP
    /// exchange blocks for up to a few seconds).
    fn drop_reach(&mut self) {
        if let Some(mut r) = self.reach_host.take() {
            std::thread::spawn(move || r.close());
        }
        self.reach_join = None;
    }

    /// net-cloud-lobby: the code the lobby shows — the cloud service's while it keeps answering,
    /// then the nf-lobbyd short code, then the long self-contained code [S].
    pub fn join_code_display(&self) -> Option<String> {
        let now = self.epoch.elapsed().as_millis() as u64;
        if let Some(c) = self.cloud_host.as_ref().and_then(|h| h.code(now)) {
            return Some(c.to_string());
        }
        if let Some(c) = self.dir_code.as_ref().and_then(|sc| sc.code(now)) {
            return Some(c);
        }
        self.reach_host.as_ref().map(|r| r.status()).and_then(|s| s.code)
    }

    /// net-short-codes: the joiner's short-code status line for the join page (None when there
    /// is nothing to say) [S].
    pub fn join_code_note(&self) -> Option<String> {
        self.short_note.clone()
    }

    /// net-cloud-lobby: the public-lobby list rows now (empty when the page is closed, the
    /// service is off, or no answer has landed yet) [S].
    pub fn cloud_lobbies(&self) -> Vec<nf_net::cloud::CloudLobby> {
        self.cloud_browse.as_ref().map_or(Vec::new(), |b| b.rows().to_vec())
    }

    /// net-p2p: the online pages' reachability lines.
    pub fn reach(&self) -> Option<crate::menu::MpReach> {
        use nf_net::p2p::{HostState, MapMethod};
        if let Some(r) = self.reach_host.as_ref() {
            let s = r.status();
            let state = match s.state {
                HostState::Probing => "Internet: opening the port...".to_string(),
                HostState::Mapped(MapMethod::Upnp) => "Internet: open (UPnP)".to_string(),
                HostState::Mapped(MapMethod::NatPmp) => "Internet: open (NAT-PMP)".to_string(),
                HostState::StunOnly => "Internet: no port forward - Ctrl+V a friend's code".to_string(),
                HostState::Failed => "Internet: unavailable (LAN only)".to_string(),
            };
            let note = match s.state {
                HostState::StunOnly if s.symmetric => "Your router changes ports per peer: direct play may fail".to_string(),
                HostState::StunOnly => {
                    let n = s.punches.len();
                    if n == 0 { "Friends send you their code from Join Online".to_string() } else { format!("Opening a path to {n} friend(s)") }
                }
                HostState::Failed => format!("Forward UDP {} on your router to play online", s.local_port),
                _ => String::new(),
            };
            return Some(crate::menu::MpReach { hosting: true, state, code: self.join_code_display(), my_code: None, note });
        }
        if let Some(j) = self.reach_join.as_ref() {
            let s = j.status(self.epoch.elapsed().as_millis() as u64);
            let (state, note) = if s.symmetric_both {
                ("Both routers block direct play".to_string(), "A relay would be needed (not supported)".to_string())
            } else if s.host_needs_punch {
                ("The host has no port forward".to_string(), "Send the host your code (Ctrl+C copies it)".to_string())
            } else {
                (String::new(), String::new())
            };
            return Some(crate::menu::MpReach { hosting: false, state, code: None, my_code: s.my_code, note });
        }
        None
    }

    /// net-p2p: the host pasted a joiner's code.
    pub fn expect_peer(&mut self, code: &str) -> bool {
        self.reach_host.as_mut().is_some_and(|r| r.expect_peer(code).is_ok())
    }

    /// Pump the transport once per frame: receive, timers, events → state.
    pub fn poll(&mut self) {
        let now = self.epoch.elapsed().as_millis() as u64;
        // net-p2p: STUN replies and punches share the game socket (the session hands back
        // what is not a session packet); the reachability layer sends through it too.
        // net-lobby-list: a host answers discovery queries on its game socket (in the lobby and
        // in progress — the record says which).
        if let Some(b) = self.browse.as_mut() { b.poll(now); }
        let strays = self.sess.take_stray();
        // net-cloud-lobby: the joiner's public-lobby list refreshes while the page drives it [S].
        if let Some(cb) = self.cloud_browse.as_mut() { cb.poll(now); }
        if self.is_host && !self.closed {
            // The internet directory: REGISTER from the game socket every few seconds (the server
            // lists the source address — this socket's public mapping). net-short-codes: the
            // REGISTER is a v2 packet (this period's nonce + our LAN endpoint); the server's ack
            // carries the 4-character code [S docs/systems/net-short-codes.md].
            if let (Some(dir), Some(info)) = (self.reach_cfg.as_ref().and_then(|c| c.directory), self.advert.as_ref()) {
                if now >= self.dir_next_ms {
                    self.dir_next_ms = now + nf_net::directory::REGISTER_EVERY_MS;
                    let lan = nf_net::p2p::local_ipv4().map(|ip| std::net::SocketAddrV4::new(ip, self.sess.local_addr().port()));
                    let sc = self.dir_code.get_or_insert_with(|| nf_net::directory::ShortCodeHost::new(reach_seed() as u32));
                    let pkt = sc.register_packet(info, lan);
                    self.sess.send_raw(dir, &pkt);
                }
            }
            if let Some(info) = self.advert.as_mut() {
                info.in_progress = self.match_start.is_some();
                info.port = self.sess.local_addr().port();
                for (from, d) in &strays {
                    if let Some(r) = nf_net::discovery::answer(d, info) { self.sess.send_raw(*from, &r); }
                    if let Some(sc) = self.dir_code.as_mut() { sc.on_packet(d, now); }
                }
            }
            // net-cloud-lobby: POST /v1/host every CLOUD_REFRESH_MS while hosting + advertising —
            // the body's port is the STUN-observed public UDP port when reachability knows one,
            // else the game port; the service records (our public IP as it saw it, that port) [S].
            if let Some(url) = self.cloud_cfg.clone() {
                if self.advert.is_some() && now >= self.cloud_next_ms {
                    self.cloud_next_ms = now + nf_net::cloud::CLOUD_REFRESH_MS;
                    let info = self.advert.clone().expect("advert checked");
                    let public_port = self
                        .reach_host
                        .as_ref()
                        .map(|r| r.status())
                        .and_then(|s| s.public)
                        .map(|p| p.port())
                        .unwrap_or_else(|| self.sess.local_addr().port());
                    let lan = nf_net::p2p::local_ipv4()
                        .map(|ip| format!("{ip}:{}", self.sess.local_addr().port()))
                        .unwrap_or_default();
                    let ch = self.cloud_host.get_or_insert_with(|| nf_net::cloud::CloudHost::new(url));
                    ch.set_advert(nf_net::cloud::CloudAdvert {
                        version: nf_net::cloud::game_version(),
                        name: info.name,
                        map: info.level_code,
                        scenario: info.scenario,
                        players: info.players,
                        max_players: info.max_players,
                        public: self.pub_lobby,
                        lan,
                        port: public_port,
                    });
                }
                if let Some(ch) = self.cloud_host.as_mut() { ch.poll(now); }
            }
        }
        // net-short-codes: the joiner's 4-character code is being looked up; the found endpoint
        // is dialled exactly as a long code's (rebind + JoinReach, the LAN endpoint as a probe
        // hint). Any failure leaves a clear note for the join page [S].
        if self.pending_join.is_some() {
            let mut lk = self.pending_join.take().expect("pending_join checked");
            for (from, d) in &strays {
                lk.on_packet(*from, d);
            }
            let mut out: Vec<(std::net::SocketAddr, Vec<u8>)> = Vec::new();
            lk.poll(now, &mut |to, d| out.push((to, d.to_vec())));
            for (to, d) in out { self.sess.send_raw(to, &d); }
            if let Some(r) = lk.result() {
                match r {
                    Ok(f) => {
                        let public = std::net::SocketAddr::V4(f.public);
                        eprintln!("[net] code {}: the server knows {}", lk.code(), public);
                        let name = self.display_name();
                        if let Err(e) = self.rebind_client(public) {
                            // The ephemeral re-bind could not open a socket: the join page's own
                            // timeout ends the attempt; the note says why.
                            eprintln!("[net] code {}: dial failed: {e}", lk.code());
                            self.short_note = Some("Could not open a socket to dial the host".to_string());
                            return;
                        }
                        if !name.is_empty() { self.sess.set_display_name(&name); }
                        if let Some(cfg) = self.reach_cfg.clone() {
                            let mut jr = nf_net::p2p::JoinReach::start(&public.to_string(), &cfg, reach_seed()).ok();
                            if let Some(j) = jr.as_mut() {
                                j.set_lan_hint(f.lan.map(|a| std::net::SocketAddr::V4(a)));
                            }
                            self.reach_join = jr;
                        }
                        self.short_note = None;
                    }
                    Err(reason) => {
                        eprintln!("[net] code {}: lookup failed: {reason:?}", lk.code());
                        self.short_note = Some(match reason {
                            nf_net::directory::NackReason::NotFound => format!("Code {} was not found on the lobby server", lk.code()),
                            nf_net::directory::NackReason::RateLimited => "The lobby server rate-limited lookups - wait a moment".to_string(),
                            _ => "The lobby server did not answer - an older nf-lobbyd without short codes?".to_string(),
                        });
                    }
                }
            } else {
                self.pending_join = Some(lk);
            }
        }
        // net-cloud-lobby: the joiner's 4-character code is being looked up over HTTPS; the found
        // endpoint is dialled exactly as a long code's (rebind + JoinReach, the LAN endpoint as a
        // probe hint). Any failure leaves a clear note for the join page [S].
        if self.cloud_pending.is_some() {
            let mut lk = self.cloud_pending.take().expect("cloud_pending checked");
            lk.poll();
            if let Some(r) = lk.result() {
                match r {
                    Ok(f) => {
                        let public = std::net::SocketAddr::V4(f.public);
                        eprintln!("[net] code {}: the service knows {}", lk.code(), public);
                        let name = self.display_name();
                        if let Err(e) = self.rebind_client(public) {
                            eprintln!("[net] code {}: dial failed: {e}", lk.code());
                            self.short_note = Some("Could not open a socket to dial the host".to_string());
                            return;
                        }
                        if !name.is_empty() { self.sess.set_display_name(&name); }
                        if let Some(cfg) = self.reach_cfg.clone() {
                            let mut jr = nf_net::p2p::JoinReach::start(&public.to_string(), &cfg, reach_seed()).ok();
                            if let Some(j) = jr.as_mut() {
                                j.set_lan_hint(f.lan.map(|a| std::net::SocketAddr::V4(a)));
                            }
                            self.reach_join = jr;
                        }
                        self.short_note = None;
                    }
                    Err(nf_net::cloud::CloudLookupError::NotFound) => {
                        eprintln!("[net] code {}: not found on the service", lk.code());
                        self.short_note = Some(format!("Code {} was not found", lk.code()));
                    }
                    Err(nf_net::cloud::CloudLookupError::Unreachable) => {
                        eprintln!("[net] code {}: the service is unreachable", lk.code());
                        self.short_note = Some("The lobby service is unreachable - check lobby_service in net.txt".to_string());
                    }
                }
            } else {
                self.cloud_pending = Some(lk);
            }
        }
        if self.reach_host.is_some() || self.reach_join.is_some() {
            for (from, d) in strays {
                if let Some(r) = self.reach_host.as_mut() { r.on_packet(from, &d); }
                if let Some(r) = self.reach_join.as_mut() { r.on_packet(from, &d); }
            }
            let mut out: Vec<(std::net::SocketAddr, Vec<u8>)> = Vec::new();
            if let Some(r) = self.reach_host.as_mut() { r.poll(now, &mut |to, d| out.push((to, d.to_vec()))); }
            if let Some(r) = self.reach_join.as_mut() { r.poll(now, &mut |to, d| out.push((to, d.to_vec()))); }
            for (to, d) in out { self.sess.send_raw(to, &d); }
            // net-p2p §3a: the host answered our LAN probe — dial its LAN address instead of the
            // public one (no hairpin needed), keeping the join reach and the codename.
            let redirect = if self.sess.local_slot() == 0 && self.match_start.is_none() {
                self.reach_join.as_mut().and_then(|r| r.take_redirect())
            } else {
                None
            };
            if let Some(a) = redirect {
                eprintln!("[net] same network: dialing the host's LAN address {a}");
                let name = self.display_name();
                if let Ok(tr) = UdpTransport::bind("0.0.0.0:0") {
                    self.sess = Session::new_client(session_cfg(), tr, a);
                    if !name.is_empty() { self.sess.set_display_name(&name); }
                }
            }
            // The console line the CLI path (--net-host / --net-join) reads: the state and the
            // code once, and again when they change.
            let line = self.reach().map(|r| format!("{} | code {} | my code {} | {}", r.state, r.code.unwrap_or_default(), r.my_code.unwrap_or_default(), r.note));
            if line.is_some() && line != self.reach_line {
                if let Some(l) = &line { eprintln!("[net] {l}"); }
                self.reach_line = line;
            }
        }
        if self.is_host && !self.closed {
            if let Some((bytes, left)) = self.start_resend.as_mut() {
                *left = left.saturating_sub(1);
                let resend = (*left % START_RESEND_EVERY == 0).then(|| bytes.clone());
                if *left == 0 { self.start_resend = None; }
                if let Some(b) = resend { self.sess.send_data(b); }
            }
        }
        for ev in self.sess.poll(now) {
            match ev {
                Event::Refused { reason } => {
                    eprintln!("[net] refused by the host: {reason:?}");
                    self.refused = true;
                    self.events.push(CoreEvent::Refused);
                }
                Event::LateJoinStarted { slot, .. } => {
                    self.events.push(CoreEvent::LateJoinStarted { slot });
                }
                Event::LateJoinHandover { slot, seed, settings, first_tick: _, snapshot_tick, snapshot_hash, log } => {
                    let s = match_to_mp(&settings, seed);
                    self.events.push(CoreEvent::LateJoinHandover {
                        slot,
                        settings: Box::new(s),
                        snapshot_tick,
                        snapshot_hash,
                        log,
                        joins: self.sess.join_history().to_vec(),
                    });
                }
                Event::LateJoinSlot { slot, join_tick } => {
                    self.events.push(CoreEvent::LateJoinSlot { slot, join_tick });
                }
                Event::ClientJoined { slot, .. } => {
                    self.events.push(CoreEvent::AgentJoined { slot });
                }
                Event::MatchStarted { .. } => {
                    // Client GO: the final settings must already have arrived
                    // through the tunnel (the host sends them on arm, before
                    // GO). If none came, the handover stays the WELCOME's.
                }
                Event::ClientLeft { slot, reason } => {
                    self.events.push(CoreEvent::ClientLeft {
                        slot,
                        graceful: reason == nf_net::LeaveReason::Graceful,
                    });
                }
                Event::HostTimeout => {
                    self.host_timed_out = true;
                    self.events.push(CoreEvent::HostTimeout);
                }
                Event::HostLeft => {
                    self.host_left = true;
                    self.events.push(CoreEvent::HostLeft);
                }
                Event::Desync { tick, local, remote } => {
                    self.events.push(CoreEvent::Desync { tick, local, remote });
                }
                Event::DesyncFatal { tick } => {
                    self.events.push(CoreEvent::DesyncFatal { tick });
                }
                Event::HashComps { tick, comps } => {
                    self.events.push(CoreEvent::HashComps { tick, comps });
                }
                Event::HashDetail { tick, component, items } => {
                    self.events.push(CoreEvent::HashDetail { tick, component, items });
                }
                Event::Data { from_slot, bytes } => {
                    match bytes.first() {
                        Some(0) => {
                            self.choices = MpNetChoices::decode(&bytes);
                        }
                        // The host repeats the start (start_resend); the first copy wins.
                        Some(1) if self.match_start.is_none() => {
                            if let Some((m, seed)) = decode_match_start(&bytes) {
                                let s = match_to_mp(&m, seed);
                                self.match_start = Some(s.clone());
                                self.events.push(CoreEvent::MatchStarted(Box::new(s)));
                            }
                        }
                        // mp-lobby-one-screen: tag 2 = the one-screen snapshot, tag 3 = a
                        // client's own-row edit [S docs/systems/net-online-lobby.md §6.4].
                        Some(2) => {
                            if let Some(st) = nf_net::LobbyState::decode(&bytes[1..]) {
                                self.lobby = Some(st);
                            }
                        }
                        Some(3) => {
                            if let Some(e) = nf_net::LobbyEdit::decode(&bytes[1..]) {
                                self.lobby_edits.push(e);
                            }
                        }
                        // net-match-end: tag 4 = the debriefing ack (final tick + the board's
                        // fold) — the host's end wait reads these; every other machine ignores it.
                        Some(4) if bytes.len() >= 13 => {
                            let final_tick = u32::from_le_bytes(bytes[1..5].try_into().expect("4 bytes"));
                            let board_hash = u64::from_le_bytes(bytes[5..13].try_into().expect("8 bytes"));
                            self.events.push(CoreEvent::DebriefAck { slot: from_slot, final_tick, board_hash });
                        }
                        _ => {}
                    }
                }
                Event::Listening | Event::Connected { .. } | Event::SlotMapChanged { .. }
                | Event::Stalled { .. } | Event::Resumed { .. } | Event::MatchEnded { .. }
                | Event::ClientJoined { .. } | Event::MatchStarted { .. } => {}
            }
        }
    }

    /// The app's events since the last call.
    pub fn take_app_events(&mut self) -> Vec<CoreEvent> {
        std::mem::take(&mut self.events)
    }

    /// Host: arm the match — the settings + seed replace the handshake
    /// placeholder, GO starts every connected client, and the final payload
    /// rides the tunnel (053's START). This machine takes the same handover.
    pub fn start_match(&mut self, s: &MpSettings) {
        if !self.is_host || self.closed {
            return;
        }
        self.sess.set_host_match(mp_to_match(s), s.seed);
        self.sess.start_match(0).ok();
        self.match_start = Some(s.clone());
        self.events.push(CoreEvent::MatchStarted(Box::new(s.clone())));
        let payload = encode_match_start(&mp_to_match(s), s.seed);
        self.sess.send_data(payload.clone());
        self.start_resend = Some((payload, START_RESEND_POLLS));
    }

    /// The lobby's agents (the trait's `agents`).
    pub fn agents(&self) -> Vec<MpNetAgent> {
        self.sess.agents().into_iter().map(|a| MpNetAgent {
            slot: a.slot as usize,
            codename: a.codename,
            ready: true, // the lobby auto-readies joined panels [S §2.1]
        }).collect()
    }

    pub fn host_choices(&self) -> Option<MpNetChoices> {
        self.choices.clone()
    }

    /// mp-lobby-one-screen: the client reads the host's last snapshot [S §6.4].
    pub fn lobby_state(&self) -> Option<nf_net::LobbyState> {
        self.lobby.clone()
    }

    /// mp-lobby-one-screen: the host stores + pushes the whole snapshot [S §6.4].
    pub fn publish_lobby(&mut self, st: &nf_net::LobbyState) {
        self.lobby = Some(st.clone());
        let mut payload = vec![2u8];
        payload.extend_from_slice(&st.encode());
        self.sess.send_data(payload);
    }

    /// mp-lobby-one-screen: a client pushes one own-row edit [S §6.4].
    pub fn push_row_edit(&mut self, e: &nf_net::LobbyEdit) {
        let mut payload = vec![3u8];
        payload.extend_from_slice(&e.encode());
        self.sess.send_data(payload);
    }

    /// net-match-end: this machine's handover ran — announce the final tick and the board's
    /// fold (Data tag 4; see docs/systems/net-match-end.md §4) [S port protocol].
    pub fn send_debrief_ack(&mut self, final_tick: u32, board_hash: u64) {
        let mut payload = vec![4u8];
        payload.extend_from_slice(&final_tick.to_le_bytes());
        payload.extend_from_slice(&board_hash.to_le_bytes());
        self.sess.send_data(payload);
    }

    /// net-match-end: the human slots other than this machine's own (the end wait's peer set —
    /// a departed slot has already left `slot_human` [M Session::drop_slot]).
    pub fn human_peers(&self) -> Vec<usize> {
        self.sess.human_slots().into_iter().filter(|&s| s != self.sess.local_slot() as usize).collect()
    }

    /// mp-lobby-one-screen: the host drains one arrived client edit [S §6.4].
    pub fn take_row_edit(&mut self) -> Option<nf_net::LobbyEdit> {
        if self.lobby_edits.is_empty() { None } else { Some(self.lobby_edits.remove(0)) }
    }

    pub fn publish_choices(&mut self, c: &MpNetChoices) {
        self.choices = Some(c.clone());
        self.sess.send_data(c.encode());
    }

    pub fn set_codename(&mut self, name: &str) {
        self.sess.set_display_name(name);
    }

    /// Leave (LEAVE to every peer) and RELEASE the socket: the port is free the moment this
    /// returns, so a re-host on the same port works at once (mp-online-failures — the closed
    /// session used to keep its socket bound until the whole core dropped: Back then Host again
    /// on the same port read "Port already in use").
    pub fn close(&mut self) {
        if !self.closed {
            self.sess.leave();
            self.closed = true;
            // net-cloud-lobby: the service's entry goes at once (fire and forget; a lost DELETE
            // expires out server-side within 90 s) [S].
            if let Some(h) = self.cloud_host.as_mut() { h.leave(); }
            self.drop_reach();
            self.release_socket();
            self.advert = None;
            self.browse = None;
            self.cloud_browse = None;
            self.cloud_pending = None;
        }
    }

    /// mp-online-failures: the match is over (the host's debriefing) — the finished session
    /// keeps its peers (polled, not ticked) but answers no new HELLO, so a client re-dialing for
    /// the rematch reaches the next lobby on this port instead of late-joining this match.
    pub fn finish_match(&mut self) {
        if self.is_host { self.sess.close_joins(); }
        self.advert = None;
    }

    /// mp-online-failures: a client dialed a match already in progress — the host began (or
    /// completed) the late-join handover. The front end shows "joining" and the app adopts the
    /// session for the replay instead of waiting for a lobby that will never come.
    pub fn joining_match(&self) -> bool {
        !self.is_host && !self.closed && self.sess.late_join_pending()
    }

    /// The session socket's local address (tests: the port is released after `close`).
    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.sess.local_addr()
    }

    /// The link state the join flow reads.
    pub fn link(&self) -> MpNetLink {
        if self.closed { return MpNetLink::Idle; }
        if self.is_host { return MpNetLink::Listening; }
        if self.host_timed_out || self.host_left { return MpNetLink::Lost; }
        if self.refused { return MpNetLink::Refused; }
        if self.sess.local_slot() != 0 || self.match_start.is_some() {
            return MpNetLink::Connected;
        }
        MpNetLink::Dialing
    }

    // ── the app's match surface (the lockstep tick gate) ────────────────

    pub fn ready_inputs(&mut self, tick: u32) -> Option<[nf_net::Input; MAX_AGENTS]> {
        self.sess.ready_inputs(tick)
    }

    pub fn local_input(&mut self, tick: u32, input: nf_net::Input) {
        self.sess.local_input(tick, input);
    }

    /// The match's input lead (see `nf_net::Session::input_delay`).
    pub fn input_delay(&self) -> u32 {
        self.sess.input_delay()
    }

    /// mp-local-look: see `nf_net::Session::pending_local_look`.
    pub fn pending_local_look(&self, after: u32) -> (f32, f32) {
        self.sess.pending_local_look(after)
    }

    /// net-internet: schedule the local input sampled while running `run_tick` (it lands
    /// `delay_at(run_tick)` ticks ahead — the host's adaptive delay; see
    /// `nf_net::Session::submit_local`).
    pub fn submit_local(&mut self, run_tick: u32, input: nf_net::Input) {
        self.sess.submit_local(run_tick, input);
    }

    /// net-internet: send what the frame's ticks just scheduled now (no receive, no events) —
    /// the app calls it after its tick loop, so its inputs leave this frame, not the next.
    pub fn flush(&mut self) {
        if self.closed {
            return;
        }
        let now = self.epoch.elapsed().as_millis() as u64;
        self.sess.flush(now);
    }

    /// net-internet diagnostics: (the worst peer RTT in ms, the delay in force, bytes and
    /// datagrams sent so far).
    pub fn net_stats(&self) -> (Option<f64>, u32, u64, u64) {
        let rtt = self.sess.peer_rtts().into_iter().filter_map(|(_, r)| r).fold(None, |a: Option<f64>, r| Some(a.map_or(r, |a| a.max(r))));
        let (b, n) = self.sess.sent_totals();
        (rtt, self.sess.input_delay(), b, n)
    }

    /// net-internet: the session's input-delay schedule (from_tick, delay).
    pub fn delay_schedule(&self) -> Vec<(u32, u32)> {
        self.sess.delay_schedule().to_vec()
    }

    /// net-desync-detail: the recorded run transcript — the base tick and the per-tick rows
    /// since the match start (the desync report's `match-log.bin` holds a copy).
    pub fn run_log(&self) -> (u32, Vec<[nf_net::Input; MAX_AGENTS]>) {
        let (base, log) = self.sess.run_log();
        (base, log.to_vec())
    }

    /// The input delay in force at `tick`.
    pub fn delay_at(&self, tick: u32) -> u32 {
        self.sess.delay_at(tick)
    }

    /// net-desync-detail: the hash-exchange cadence (the watcher's detail rows ride it).
    pub fn hash_every(&self) -> u32 {
        self.sess.hash_every()
    }

    pub fn submit_hash(&mut self, tick: u32, hash: u64) {
        self.sess.submit_hash(tick, hash);
    }

    /// net-desync-detail: send our component hashes for the desync tick (the watcher's first
    /// exchange packet).
    pub fn send_hash_comps(&mut self, tick: u32, comps: [u64; HASH_COMPONENTS]) {
        if !self.closed {
            self.sess.send_hash_comps(tick, comps);
        }
    }

    /// net-desync-detail: send the first diverging component's per-item hashes.
    pub fn send_hash_detail(&mut self, tick: u32, component: u8, items: Vec<u64>) {
        if !self.closed {
            self.sess.send_hash_detail(tick, component, items);
        }
    }

    /// Host: record the per-tick inputs a tick ran with (the late-join
    /// transcript; the app calls it right after `ready_inputs` returned
    /// `Some`). Since net-desync-detail every machine records (the desync
    /// report's match-log.bin is this log at rest).
    pub fn record_run_tick(&mut self, tick: u32, inputs: [nf_net::Input; MAX_AGENTS]) {
        self.sess.record_run_tick(tick, inputs);
    }

    /// Late joiner: the replay verified — the host picks the join tick and
    /// every machine arms the slot at it (JOIN_SLOT →
    /// [`CoreEvent::LateJoinSlot`]).
    pub fn confirm_late_join(&mut self) {
        self.sess.confirm_late_join();
    }

    /// Late joiner: the replay hash disagreed with the host's snapshot —
    /// leave with the explicit reason (the host reports
    /// `ClientLeft { JoinVerifyFailed }`; the match never sees this joiner).
    pub fn late_join_verify_failed(&mut self) {
        self.sess.leave_with(nf_net::LeaveReason::JoinVerifyFailed);
    }

    pub fn local_slot(&self) -> usize {
        self.sess.local_slot() as usize
    }

    /// This machine's display codename (the re-bind path carries it over).
    pub fn display_name(&self) -> String {
        self.sess.agents().into_iter()
            .find(|a| a.slot == self.sess.local_slot())
            .map(|a| a.codename)
            .unwrap_or_default()
    }

    /// The match handover peek (the lobby's `match_started`).
    pub fn match_started(&self) -> Option<MpSettings> {
        self.match_start.clone()
    }

    /// Test-only (net-e2e-loopback): deterministic loss + reordering on this
    /// core's UDP sends. See `nf_net::Session::inject_link_fault`.
    pub fn inject_link_fault(&mut self, seed: u64, drop_per_mille: u32, reorder_per_mille: u32) {
        self.sess.inject_link_fault(seed, drop_per_mille, reorder_per_mille);
    }

    /// Test-only (net-internet): the fault line plus a one-way delay and jitter in ms on
    /// this core's sends — half of a simulated internet path.
    pub fn inject_link_delay(&mut self, seed: u64, drop_per_mille: u32, delay_ms: u64, jitter_ms: u64) {
        self.sess.inject_link_delay(seed, drop_per_mille, 0, delay_ms, jitter_ms);
    }
}

/// mp-online-failures: a core dropped while still open (the window closed, the app exiting
/// mid-match) says goodbye — the peers learn at once (`ClientLeft` on the host, `HostLeft` on a
/// client) instead of stalling out the 5 s timeout.
impl Drop for NetCore {
    fn drop(&mut self) {
        if !self.closed {
            self.sess.leave();
        }
    }
}

/// A cheap cloneable handle: the front end holds one as its `MpSession`
/// trait object, the app drives the match through another. The second field
/// is the test-only link-fault recipe (`None` in production): the lobby's
/// `listen`/`connect` replace the core, and the recipe rides along so a
/// lossy proxy STAYS lossy across the re-bind (net-e2e-loopback).
#[derive(Clone)]
pub struct SessionProxy(pub Rc<RefCell<NetCore>>, pub Option<(u64, u32, u32)>);

impl SessionProxy {
    pub fn host(port: u16) -> Result<SessionProxy, String> {
        NetCore::host(port).map(|c| SessionProxy(c, None))
    }
    pub fn connect(addr: &str) -> Result<SessionProxy, String> {
        NetCore::connect(addr).map(|c| SessionProxy(c, None))
    }
    /// The lossy variant: every core this proxy builds (now and on the
    /// lobby's re-bind) drops `drop_per_mille`‰ of its sends and transposes
    /// `reorder_per_mille`‰ of them, from the seeded SplitMix64.
    pub fn host_faulty(port: u16, fault: (u64, u32, u32)) -> Result<SessionProxy, String> {
        let c = NetCore::host(port)?;
        c.borrow_mut().inject_link_fault(fault.0, fault.1, fault.2);
        Ok(SessionProxy(c, Some(fault)))
    }
    pub fn connect_faulty(addr: &str, fault: (u64, u32, u32)) -> Result<SessionProxy, String> {
        let c = NetCore::connect(addr)?;
        c.borrow_mut().inject_link_fault(fault.0, fault.1, fault.2);
        Ok(SessionProxy(c, Some(fault)))
    }
    pub fn core_is_closed(&self) -> bool {
        self.0.borrow().closed
    }
    /// net-match-end: is this machine the session's host (the end wait runs on it).
    pub fn is_host(&self) -> bool {
        self.0.borrow().is_host
    }
    /// True once the match is armed (host) or the match payload arrived
    /// (client) — the app's adoption cue.
    pub fn has_match(&self) -> bool {
        self.0.borrow().match_started().is_some()
    }
    /// mp-online-failures: the lobby's dial reached a match in progress — the app's late-join
    /// adoption cue (the handover replays instead of a normal match load).
    pub fn late_joining(&self) -> bool {
        self.0.borrow().joining_match()
    }
}

/// mp-online-failures: what the app does with the lobby's session this frame — the pure
/// decision `App::adopt_lobby_session` runs (pinned headless by the failure-path tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LobbyAdopt {
    /// Nothing to adopt yet (still in the lobby, or cancelled).
    No,
    /// The host armed / the client received the match start: adopt and load the arena.
    Match,
    /// The dial reached a match in progress: adopt for the late-join handover (no arena load —
    /// the handover's replay builds the scene).
    LateJoin,
}

impl SessionProxy {
    /// The adoption decision for this lobby proxy.
    pub fn adopt_kind(&self) -> LobbyAdopt {
        if self.core_is_closed() {
            LobbyAdopt::No
        } else if self.has_match() {
            LobbyAdopt::Match
        } else if self.late_joining() {
            LobbyAdopt::LateJoin
        } else {
            LobbyAdopt::No
        }
    }
}

impl crate::menu::MpSession for SessionProxy {
    fn poll(&mut self) { self.0.borrow_mut().poll(); }
    fn link(&self) -> MpNetLink { self.0.borrow().link() }
    fn listen(&mut self, port: u16) -> Result<(), MpNetError> {
        // The core binds at construction; the lobby's listen row RE-BINDS the
        // same core in place (net-e2e-loopback: the old replace-the-Rc shape
        // left every other handle — the app's `net`, test clones — on the
        // stale placeholder socket). Called only before anything flows, so a
        // fresh session is behaviour-identical. The fault recipe (if any)
        // re-applies — a lossy proxy stays lossy across the re-bind.
        let name = self.0.borrow().display_name();
        let r = self.0.borrow_mut().rebind_host(port);
        match r {
            Ok(()) => {
                if let Some(f) = self.1 {
                    self.0.borrow_mut().inject_link_fault(f.0, f.1, f.2);
                }
                if !name.is_empty() { self.0.borrow_mut().set_codename(&name); }
                Ok(())
            }
            Err(_) => Err(MpNetError::Busy),
        }
    }
    fn connect(&mut self, addr: &str) -> Result<(), MpNetError> {
        let name = self.0.borrow().display_name();
        // A join code, `a.b.c.d:port`, or a 4-character lobby code — `dial` below takes all three
        // (parse_join_target); the old parse_target gate refused the short codes.
        if nf_net::p2p::parse_join_target(addr).is_err() {
            return Err(MpNetError::BadAddress);
        }
        let r = self.0.borrow_mut().dial(addr);
        match r {
            Ok(()) => {
                if let Some(f) = self.1 {
                    self.0.borrow_mut().inject_link_fault(f.0, f.1, f.2);
                }
                if !name.is_empty() { self.0.borrow_mut().set_codename(&name); }
                Ok(())
            }
            Err(_) => Err(MpNetError::BadAddress),
        }
    }
    fn agents(&self) -> Vec<MpNetAgent> { self.0.borrow().agents() }
    fn host_choices(&self) -> Option<MpNetChoices> { self.0.borrow().host_choices() }
    fn publish_choices(&mut self, c: &MpNetChoices) { self.0.borrow_mut().publish_choices(c); }
    fn start_match(&mut self, s: &nf_game::mp::MpSettings) { self.0.borrow_mut().start_match(s); }
    fn match_started(&self) -> Option<nf_game::mp::MpSettings> { self.0.borrow().match_started() }
    fn set_codename(&mut self, name: &str) { self.0.borrow_mut().set_codename(name); }
    fn local_slot(&self) -> usize { self.0.borrow().local_slot() }
    fn lobby_state(&self) -> Option<nf_net::LobbyState> { self.0.borrow().lobby_state() }
    fn publish_lobby(&mut self, s: &nf_net::LobbyState) { self.0.borrow_mut().publish_lobby(s); }
    fn push_row_edit(&mut self, e: &nf_net::LobbyEdit) { self.0.borrow_mut().push_row_edit(e); }
    fn take_row_edit(&mut self) -> Option<nf_net::LobbyEdit> { self.0.borrow_mut().take_row_edit() }
    fn close(&mut self) { self.0.borrow_mut().close(); }
    fn joining_match(&self) -> bool { self.0.borrow().joining_match() }
    fn reach(&self) -> Option<crate::menu::MpReach> { self.0.borrow().reach() }
    fn browse(&mut self, on: bool) {
        let mut c = self.0.borrow_mut();
        if on && c.browse.is_none() {
            let mut l = nf_net::discovery::LobbyList::open(nf_net::PROTOCOL_VERSION, nf_net::DEFAULT_BUILD_ID, reach_seed() as u32);
            l.set_directory(c.reach_cfg.as_ref().and_then(|r| r.directory));
            c.browse = Some(l);
        } else if !on {
            c.browse = None;
        }
        // net-cloud-lobby: the Join Online page drives the public list too, when the service is
        // on (the same on/off as the LAN browse; the rows refresh in the core's poll) [S].
        if on {
            if let (Some(url), None) = (&c.cloud_cfg, &c.cloud_browse) {
                c.cloud_browse = Some(nf_net::cloud::CloudBrowse::open(url.clone(), nf_net::cloud::game_version()));
            }
        } else {
            c.cloud_browse = None;
        }
    }
    fn browse_rows(&self) -> Vec<crate::menu::MpLobbyRow> {
        self.0.borrow().browse.as_ref().map_or(Vec::new(), |b| b.rows().into_iter().map(|e| crate::menu::MpLobbyRow {
            name: e.info.name, host: e.info.host, level: e.info.level_code, scenario: e.info.scenario,
            players: e.info.players, max_players: e.info.max_players, in_progress: e.info.in_progress,
            ping_ms: e.ping_ms, joinable: e.joinable, addr: e.addr.to_string(),
        }).collect())
    }
    fn set_lobby(&mut self, name: &str, level: u32, scenario: u32, players: u8) {
        let mut c = self.0.borrow_mut();
        let host = c.display_name();
        let port = c.sess.local_addr().port();
        let in_progress = c.match_start.is_some();
        c.advert = Some(nf_net::discovery::LobbyInfo {
            name: name.to_string(), host, level_code: level, scenario, players, max_players: nf_net::MAX_AGENTS as u8,
            in_progress, protocol: nf_net::PROTOCOL_VERSION, build: nf_net::DEFAULT_BUILD_ID, port,
        });
    }
    fn expect_peer(&mut self, code: &str) -> bool { self.0.borrow_mut().expect_peer(code) }
    fn join_code_display(&self) -> Option<String> { self.0.borrow().join_code_display() }
    fn join_code_note(&self) -> Option<String> { self.0.borrow().join_code_note() }
    fn cloud_lobbies(&self) -> Vec<crate::menu::MpCloudLobby> {
        self.0.borrow().cloud_lobbies().into_iter().map(|l| crate::menu::MpCloudLobby {
            code: l.code, name: l.name, level: l.map, scenario: l.scenario, players: l.players, max_players: l.max_players,
        }).collect()
    }
    fn public_lobby(&self) -> bool { self.0.borrow().public_lobby() }
    fn set_public_lobby(&mut self, public: bool) { self.0.borrow_mut().set_public_lobby(public); }
}

/// A per-run seed for STUN transaction ids / punch nonces (not security relevant).
fn reach_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64)
}

#[cfg(test)]
mod cloud_tests {
    //! net-cloud-lobby's NetCore-level golden path: the host's register/refresh through the core's
    //! poll shows the service's 4-character code, and a joiner dials a typed 4-character code
    //! through the service into a real loopback session (the same mock service shape as
    //! nf-net's tests/cloud_lobby.rs, std-only) [S docs/systems/net-cloud-lobby.md].

    use super::*;
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;

    struct Entry {
        code: String,
        ip: String,
        port: u16,
        version: String,
        public: bool,
        seen: u64,
    }

    #[derive(Default)]
    struct Svc {
        entries: BTreeMap<String, Entry>,
        codes: BTreeMap<String, String>,
        rng: u64,
    }

    fn next_code(s: &mut Svc) -> String {
        loop {
            let mut code = String::new();
            s.rng = s.rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let mut x = s.rng;
            for _ in 0..4 {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                code.push(nf_net::directory::SHORT_CODE_ALPHABET[(x >> 33) as usize % 30] as char);
            }
            if !s.codes.contains_key(&code) {
                return code;
            }
        }
    }

    fn handle(s: &mut Svc, method: &str, path: &str, ip: &str, body: Option<&str>) -> (u16, String) {
        match (method, path) {
            ("POST", "/v1/host") => {
                let v = body.and_then(nf_net::cloud::json_parse).expect("a body");
                let g = |k: &str| v.get(k).cloned();
                let (Some(nf_net::cloud::Json::Str(version)), Some(nf_net::cloud::Json::Num(port))) = (g("version"), g("port")) else {
                    return (400, "{\"error\":\"bad\"}".into());
                };
                let public = g("public") != Some(nf_net::cloud::Json::Bool(false));
                let port = port as u64 as u16;
                let key = format!("{ip}:{port}");
                let code = match s.entries.get(&key) {
                    Some(e) => e.code.clone(),
                    None => {
                        let c = next_code(s);
                        s.codes.insert(c.clone(), key.clone());
                        c
                    }
                };
                s.entries.insert(key, Entry { code: code.clone(), ip: ip.to_string(), port, version, public, seen: 0 });
                (200, format!("{{\"code\":\"{code}\",\"public_addr\":\"{ip}:{port}\"}}"))
            }
            ("GET", p) if p.starts_with("/v1/code/") => {
                let code = p["/v1/code/".len()..].to_uppercase();
                match s.codes.get(&code).and_then(|k| s.entries.get(k)) {
                    Some(e) => (200, format!("{{\"code\":\"{}\",\"public_addr\":\"{}:{}\",\"lan_addr\":\"\",\"name\":\"L\"}}", e.code, e.ip, e.port)),
                    None => (404, "{\"error\":\"not found\"}".into()),
                }
            }
            _ => (404, "{\"error\":\"no route\"}".into()),
        }
    }

    /// The mock service on 127.0.0.1:0 (its clock is irrelevant here — the entries never expire
    /// inside a test run).
    fn service() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let svc = Arc::new(std::sync::Mutex::new(Svc::default()));
        let svc2 = svc.clone();
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                let Ok(mut st) = conn else { continue };
                let svc = svc2.clone();
                std::thread::spawn(move || {
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    loop {
                        let Ok(n) = st.read(&mut chunk) else { return };
                        if n == 0 { return; }
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.windows(4).any(|w| w == b"\r\n\r\n") { break; }
                    }
                    let head_end = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4).unwrap_or(buf.len());
                    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                    let mut lines = head.lines();
                    let mut it = lines.next().unwrap_or_default().split_whitespace();
                    let (method, target) = (it.next().unwrap_or("").to_string(), it.next().unwrap_or("").to_string());
                    let mut cl = 0;
                    for l in lines {
                        if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                            cl = v.trim().parse().unwrap_or(0);
                        }
                    }
                    let mut body = buf[head_end..].to_vec();
                    while body.len() < cl {
                        let Ok(n) = st.read(&mut chunk) else { break };
                        if n == 0 { break; }
                        body.extend_from_slice(&chunk[..n]);
                    }
                    let body = String::from_utf8_lossy(&body).trim().to_string();
                    let ip = st.peer_addr().map(|a| a.ip().to_string()).unwrap_or_default();
                    let (status, json) = handle(&mut svc.lock().unwrap(), &method, &target, &ip, (!body.is_empty()).then_some(body.as_str()));
                    let resp = format!("HTTP/1.1 {status} S\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", json.len(), json);
                    let _ = st.write_all(resp.as_bytes());
                    let _ = st.flush();
                });
            }
        });
        format!("http://127.0.0.1:{port}")
    }

    fn wait_until(mut done: impl FnMut() -> bool) {
        let t0 = std::time::Instant::now();
        while !done() {
            assert!(t0.elapsed() < std::time::Duration::from_secs(10), "timed out");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn the_host_shows_the_service_code_and_the_joiner_dials_a_short_code_through_it() {
        let url = service();

        // The host: a real core, advertising, cloud service on, public toggle on.
        let host = NetCore::host(0).unwrap();
        host.borrow_mut().enable_cloud(Some(url.clone()));
        host.borrow_mut().set_public_lobby(true);
        crate::menu::MpSession::set_lobby(&mut SessionProxy(host.clone(), None), "Cloud game", 0x0700_0023, 1, 1);
        let mut code = String::new();
        wait_until(|| {
            host.borrow_mut().poll();
            code = host.borrow().join_code_display().unwrap_or_default();
            code.len() == 4 && code.bytes().all(|b| nf_net::directory::SHORT_CODE_ALPHABET.contains(&b))
        });
        assert!(host.borrow().public_lobby(), "the getter exposes the toggle");
        assert!(code.bytes().all(|b| nf_net::directory::SHORT_CODE_ALPHABET.contains(&b)), "the service's code shows: {code}");

        // The joiner: a second core dials the 4-character code; the cloud lookup lands, the core
        // re-binds and the real handshake connects (the long-code path's own shape).
        let joiner = NetCore::host(0).unwrap();
        joiner.borrow_mut().enable_cloud(Some(url.clone()));
        joiner.borrow_mut().set_codename("Guest");
        joiner.borrow_mut().dial(&code).expect("the lookup starts");
        let want_note = format!("Looking up {code} on the lobby service...");
        assert_eq!(joiner.borrow().join_code_note().as_deref(), Some(want_note.as_str()));
        wait_until(|| {
            host.borrow_mut().poll();
            joiner.borrow_mut().poll();
            joiner.borrow().link() == MpNetLink::Connected
        });
        assert_eq!(joiner.borrow().join_code_note(), None, "the note clears once connected");
        host.borrow_mut().close();
        joiner.borrow_mut().close();
    }

    #[test]
    fn an_unknown_cloud_code_leaves_a_clear_note() {
        let url = service();
        let joiner = NetCore::host(0).unwrap();
        joiner.borrow_mut().enable_cloud(Some(url));
        joiner.borrow_mut().dial("ZZ9V").unwrap();
        wait_until(|| {
            joiner.borrow_mut().poll();
            joiner.borrow().join_code_note().is_some_and(|n| n.contains("was not found"))
        });
        assert_eq!(joiner.borrow().join_code_note().as_deref(), Some("Code ZZ9V was not found"));
        joiner.borrow_mut().close();
    }
}
