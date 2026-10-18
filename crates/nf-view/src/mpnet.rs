//! The ONE session layer's app surface (net-consolidate 2026-10-16).
//!
//! Two workers built two UDP sessions (051 `nf-net` and 053 `nf_game::net`);
//! this consolidation keeps 051's crate — the stronger protocol: a contiguous
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
use nf_net::{MatchSettings, UdpTransport};

use crate::menu::{MpNetAgent, MpNetChoices, MpNetError, MpNetLink};

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
        players: s.players.each_ref().map(|p| (p.team, p.skin, p.armor_bonus, p.radar_on)),
        registered_pickups: s.registered_pickups,
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
        // 700v6: the caller fills `held` / `shot` from the local weapon pipeline (lib.rs).
        held: 0,
        shot: false,
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

fn encode_match_start(m: &MatchSettings, seed: u32) -> Vec<u8> {
    let mut o = vec![1u8];
    let mut buf = [0u8; nf_net::settings::SETTINGS_WIRE_SIZE];
    m.encode(&mut buf);
    o.extend_from_slice(&buf);
    o.extend_from_slice(&seed.to_le_bytes());
    o
}

fn decode_match_start(b: &[u8]) -> Option<(MatchSettings, u32)> {
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
    /// The state hashes diverged at `tick` (the app shows the box, back to lobby).
    Desync { tick: u32 },
    /// Client: the host went silent past the timeout.
    HostTimeout,
    /// Client: the host refused the join.
    Refused,
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
    lobby: Option<nf_net::discovery::LobbyInfo>,
    /// net-lobby-list: the joiner's LAN browser while the Join Online page is up.
    browse: Option<nf_net::discovery::LobbyList>,
    /// The next REGISTER to the lobby directory (ms on the core's clock).
    dir_next_ms: u64,
}

/// The host repeats the match-start payload every this many polls …
const START_RESEND_EVERY: u32 = 4;
/// … for this many polls after the arm (≈ 2 s of 60 Hz frames).
const START_RESEND_POLLS: u32 = 120;

fn session_cfg() -> Config {
    Config { hash_every: 15, ..Default::default() } // NET_HASH_EVERY's cadence
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
            closed: false,
            events: Vec::new(),
            start_resend: None,
            reach_cfg: None,
            reach_host: None,
            reach_join: None,
            reach_line: None,
            lobby: None,
            browse: None,
            dir_next_ms: 0,
        })))
    }

    /// The lobby's re-bind, host side: a fresh listening Session on the SAME
    /// core, so every handle to this core (the app's `net`, the front end's
    /// `MpSession`, a test's clone) keeps working (net-e2e-loopback — the old
    /// replace-the-Rc shape left the other holders on the stale placeholder
    /// socket). The lobby calls it only before anything flows.
    pub fn rebind_host(&mut self, port: u16) -> Result<(), String> {
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
        self.closed = false;
        self.events.clear();
        Ok(())
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
        self.closed = false;
        self.events.clear();
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
            closed: false,
            events: Vec::new(),
            start_resend: None,
            reach_cfg: None,
            reach_host: None,
            reach_join: None,
            reach_line: None,
            lobby: None,
            browse: None,
            dir_next_ms: 0,
        })))
    }

    /// net-p2p: probe internet reachability on every later listen / dial (the app's real
    /// sessions; never the tests' direct cores).
    pub fn enable_reach(&mut self, cfg: nf_net::p2p::ReachConfig) {
        self.reach_cfg = Some(cfg);
    }

    /// Client: dial a typed target — a join code or `a.b.c.d:port` — on the SAME core, starting
    /// "my code" discovery when reach is enabled.
    pub fn dial(&mut self, typed: &str) -> Result<(), String> {
        let jc = nf_net::p2p::parse_target(typed).map_err(|e| format!("{e:?}"))?;
        self.rebind_client(std::net::SocketAddr::V4(jc.addr))?;
        if let Some(cfg) = self.reach_cfg.clone() {
            self.reach_join = nf_net::p2p::JoinReach::start(typed, &cfg, reach_seed()).ok();
        }
        Ok(())
    }

    /// net-p2p: stop probing; a host's mapping is deleted on a background thread (the UPnP/NAT-PMP
    /// exchange blocks for up to a few seconds).
    fn drop_reach(&mut self) {
        if let Some(mut r) = self.reach_host.take() {
            std::thread::spawn(move || r.close());
        }
        self.reach_join = None;
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
            return Some(crate::menu::MpReach { hosting: true, state, code: s.code, my_code: None, note });
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
        if self.is_host && !self.closed {
            // The internet directory: REGISTER from the game socket every few seconds (the server
            // lists the source address — this socket's public mapping).
            if let (Some(dir), Some(info)) = (self.reach_cfg.as_ref().and_then(|c| c.directory), self.lobby.as_ref()) {
                if now >= self.dir_next_ms {
                    self.dir_next_ms = now + nf_net::directory::REGISTER_EVERY_MS;
                    let pkt = nf_net::directory::register_packet(info);
                    self.sess.send_raw(dir, &pkt);
                }
            }
            if let Some(info) = self.lobby.as_mut() {
                info.in_progress = self.match_start.is_some();
                info.port = self.sess.local_addr().port();
                for (from, d) in &strays {
                    if let Some(r) = nf_net::discovery::answer(d, info) { self.sess.send_raw(*from, &r); }
                }
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
                Event::Refused { .. } => {
                    self.refused = true;
                    self.events.push(CoreEvent::Refused);
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
                Event::Desync { tick, .. } => {
                    self.events.push(CoreEvent::Desync { tick });
                }
                Event::Data { bytes, .. } => {
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
                        _ => {}
                    }
                }
                Event::Listening | Event::Connected { .. } | Event::SlotMapChanged { .. }
                | Event::Stalled { .. } | Event::Resumed { .. } | Event::MatchEnded { .. } => {}
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

    pub fn publish_choices(&mut self, c: &MpNetChoices) {
        self.choices = Some(c.clone());
        self.sess.send_data(c.encode());
    }

    pub fn set_codename(&mut self, name: &str) {
        self.sess.set_display_name(name);
    }

    pub fn close(&mut self) {
        if !self.closed {
            self.sess.leave();
            self.closed = true;
            self.drop_reach();
        }
    }

    /// The link state the join flow reads.
    pub fn link(&self) -> MpNetLink {
        if self.closed { return MpNetLink::Idle; }
        if self.is_host { return MpNetLink::Listening; }
        if self.host_timed_out { return MpNetLink::Lost; }
        if self.refused { return MpNetLink::Refused; }
        if self.sess.local_slot() != 0 || self.match_start.is_some() {
            return MpNetLink::Connected;
        }
        MpNetLink::Dialing
    }

    // ── the app's match surface (the lockstep tick gate) ────────────────

    pub fn ready_inputs(&mut self, tick: u32) -> Option<[nf_net::Input; 4]> {
        self.sess.ready_inputs(tick)
    }

    pub fn local_input(&mut self, tick: u32, input: nf_net::Input) {
        self.sess.local_input(tick, input);
    }

    /// The match's input lead (see `nf_net::Session::input_delay`).
    pub fn input_delay(&self) -> u32 {
        self.sess.input_delay()
    }

    pub fn submit_hash(&mut self, tick: u32, hash: u64) {
        self.sess.submit_hash(tick, hash);
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
    /// True once the match is armed (host) or the match payload arrived
    /// (client) — the app's adoption cue.
    pub fn has_match(&self) -> bool {
        self.0.borrow().match_started().is_some()
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
        // net-p2p: a join code or `a.b.c.d:port` (nf_net::p2p::parse_target).
        if nf_net::p2p::parse_target(addr).is_err() {
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
    fn close(&mut self) { self.0.borrow_mut().close(); }
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
        c.lobby = Some(nf_net::discovery::LobbyInfo {
            name: name.to_string(), host, level_code: level, scenario, players, max_players: nf_net::MAX_AGENTS as u8,
            in_progress, protocol: nf_net::PROTOCOL_VERSION, build: nf_net::DEFAULT_BUILD_ID, port,
        });
    }
    fn expect_peer(&mut self, code: &str) -> bool { self.0.borrow_mut().expect_peer(code) }
}

/// A per-run seed for STUN transaction ids / punch nonces (not security relevant).
fn reach_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64)
}
