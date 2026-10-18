//! Internet reachability for the online session (docs/systems/net-p2p.md) [S port design]:
//! UPnP IGD / NAT-PMP port mapping, STUN, join codes and hole punching — std only.
//!
//! The blocking parts (SSDP/HTTP/SOAP, NAT-PMP, DNS for the STUN servers) run on one background
//! thread ([`spawn_mapper`]); STUN and punching are poll-driven and ride the GAME socket through
//! `Session::send_raw` / `Session::take_stray`, because the public mapping a peer must reach is
//! that socket's. [`HostReach`] and [`JoinReach`] are what the front end reads.

pub mod joincode;
pub mod natpmp;
pub mod punch;
pub mod stun;
pub mod upnp;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use joincode::{parse_target, CodeError, JoinCode, FLAG_NEEDS_PUNCH, FLAG_SYMMETRIC};
pub use punch::{PunchState, Puncher};
pub use stun::StunClient;

/// Mapping lease asked for (seconds); renewed at half [S].
pub const LEASE_S: u32 = 3600;

/// Where the reachability layer looks (tests point everything at loopback fakes).
#[derive(Clone, Debug)]
pub struct ReachConfig {
    /// STUN servers ("host:port"); the setting's default is [`stun::DEFAULT_SERVERS`].
    pub stun_servers: Vec<String>,
    /// The SSDP multicast address (tests: a loopback fake).
    pub ssdp: String,
    /// How long to collect SSDP answers.
    pub ssdp_wait: Duration,
    /// The NAT-PMP gateway; None = the UPnP device's host, else the LAN x.y.z.1 guess.
    pub natpmp_gateway: Option<SocketAddr>,
    /// Try UPnP / NAT-PMP at all (off: STUN only — the joiner side and tests).
    pub map: bool,
}

impl Default for ReachConfig {
    fn default() -> Self {
        ReachConfig {
            stun_servers: stun::DEFAULT_SERVERS.iter().map(|s| s.to_string()).collect(),
            ssdp: upnp::SSDP_ADDR.to_string(),
            ssdp_wait: Duration::from_millis(1500),
            natpmp_gateway: None,
            map: true,
        }
    }
}

/// How the host's port got (or did not get) reachable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapMethod {
    Upnp,
    NatPmp,
}

/// What the background thread found.
#[derive(Clone, Debug)]
pub enum MapOutcome {
    Upnp { gw: upnp::Gateway, public: SocketAddrV4, lease_s: u32 },
    NatPmp { gw: SocketAddr, public: SocketAddrV4, lease_s: u32 },
    /// No mapping; the reasons tried, for the UI's note.
    None { why: String },
}

/// The background thread's results: the mapping outcome and the resolved STUN servers.
#[derive(Clone, Debug, Default)]
pub struct MapperResult {
    pub outcome: Option<MapOutcome>,
    pub stun: Option<Vec<SocketAddr>>,
}

/// Run the blocking discovery for `port` on a thread; poll the returned cell.
pub fn spawn_mapper(port: u16, cfg: &ReachConfig) -> Arc<Mutex<MapperResult>> {
    let cell = Arc::new(Mutex::new(MapperResult::default()));
    let out = cell.clone();
    let cfg = cfg.clone();
    std::thread::spawn(move || {
        // STUN names first: a joiner (map off) needs only these.
        let stun: Vec<SocketAddr> = cfg
            .stun_servers
            .iter()
            .filter_map(|s| s.to_socket_addrs().ok())
            .flat_map(|it| it.filter(|a| a.is_ipv4()).take(1))
            .collect();
        if let Ok(mut c) = out.lock() {
            c.stun = Some(stun);
        }
        let outcome = if cfg.map { map_any(port, &cfg) } else { MapOutcome::None { why: "mapping off".into() } };
        if let Ok(mut c) = out.lock() {
            c.outcome = Some(outcome);
        }
    });
    cell
}

/// UPnP, then NAT-PMP (blocking).
pub fn map_any(port: u16, cfg: &ReachConfig) -> MapOutcome {
    let mut why = Vec::new();
    let mut gw_ip: Option<Ipv4Addr> = None;
    match upnp::map_port(&cfg.ssdp, port, LEASE_S, cfg.ssdp_wait) {
        Ok((gw, public, lease_s)) => return MapOutcome::Upnp { gw, public, lease_s },
        Err(e) => why.push(format!("UPnP: {e:?}")),
    }
    // A gateway that answered SSDP but refused the mapping is still the NAT-PMP candidate.
    if let Ok(locs) = upnp::discover_locations(&cfg.ssdp, Duration::from_millis(300)) {
        gw_ip = locs.iter().filter_map(|l| upnp::split_url(l)).filter_map(|(hp, _)| hp.parse::<SocketAddrV4>().ok()).map(|a| *a.ip()).next();
    }
    let gw = cfg.natpmp_gateway.or_else(|| gw_ip.map(|ip| SocketAddr::V4(SocketAddrV4::new(ip, natpmp::PORT)))).or_else(|| {
        let l = local_ipv4()?;
        Some(natpmp::guess_gateway(l))
    });
    match gw {
        Some(gw) => match natpmp::map_port(gw, port, LEASE_S) {
            Ok((public, lease_s)) => return MapOutcome::NatPmp { gw, public, lease_s },
            Err(e) => why.push(format!("NAT-PMP {gw}: {e:?}")),
        },
        None => why.push("NAT-PMP: no gateway".into()),
    }
    MapOutcome::None { why: why.join("; ") }
}

/// This PC's LAN IPv4 (the route to a documentation address; nothing is sent).
pub fn local_ipv4() -> Option<Ipv4Addr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?;
    match s.local_addr().ok()? {
        SocketAddr::V4(a) if !a.ip().is_unspecified() => Some(*a.ip()),
        _ => None,
    }
}

// ───────────────────────────── host ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostState {
    /// Mapping / STUN in progress.
    Probing,
    /// The router forwards the port: anyone with the code can join.
    Mapped(MapMethod),
    /// No mapping, but the public endpoint is known: joiners need their code typed in here
    /// (hole punching).
    StunOnly,
    /// Nothing worked: LAN only (or a manual port forward + the LAN address).
    Failed,
}

/// The host page's getter (docs §4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostStatus {
    pub state: HostState,
    pub public: Option<SocketAddrV4>,
    pub code: Option<String>,
    pub symmetric: bool,
    pub local_port: u16,
    /// Why mapping failed (for the UI's small print).
    pub note: String,
    /// Joiners being punched toward: (endpoint, state).
    pub punches: Vec<(SocketAddr, PunchState)>,
}

pub struct HostReach {
    port: u16,
    cfg: ReachConfig,
    mapper: Arc<Mutex<MapperResult>>,
    outcome: Option<MapOutcome>,
    stun: Option<StunClient>,
    state: HostState,
    public: Option<SocketAddrV4>,
    symmetric: bool,
    note: String,
    punches: Vec<Puncher>,
    renew_due_ms: Option<u64>,
    seed: u64,
    /// Replies waiting for the next `poll` (PROBE_ACKs to same-network joiners).
    out: Vec<(SocketAddr, [u8; 12])>,
}

impl HostReach {
    /// Start probing for the session socket bound to `port`.
    pub fn start(port: u16, cfg: ReachConfig, seed: u64) -> HostReach {
        let mapper = spawn_mapper(port, &cfg);
        HostReach {
            port,
            cfg,
            mapper,
            outcome: None,
            stun: None,
            state: HostState::Probing,
            public: None,
            symmetric: false,
            note: String::new(),
            punches: Vec::new(),
            renew_due_ms: None,
            seed,
            out: Vec::new(),
        }
    }

    /// Advance; `send` transmits from the game socket (`Session::send_raw`).
    pub fn poll(&mut self, now_ms: u64, send: &mut dyn FnMut(SocketAddr, &[u8])) {
        if self.outcome.is_none() {
            let r = self.mapper.lock().ok().and_then(|m| m.outcome.clone());
            if let Some(o) = r {
                match &o {
                    MapOutcome::Upnp { public, lease_s, .. } | MapOutcome::NatPmp { public, lease_s, .. } => {
                        self.public = Some(*public);
                        self.state = HostState::Mapped(if matches!(o, MapOutcome::Upnp { .. }) { MapMethod::Upnp } else { MapMethod::NatPmp });
                        if *lease_s > 0 {
                            self.renew_due_ms = Some(now_ms + *lease_s as u64 * 500);
                        }
                    }
                    MapOutcome::None { why } => self.note = why.clone(),
                }
                self.outcome = Some(o);
            }
        }
        // STUN once the names resolved: always (it detects symmetric NAT and confirms the IP), and
        // it is the only source of the public endpoint when nothing mapped.
        if self.stun.is_none() {
            let servers = self.mapper.lock().ok().and_then(|m| m.stun.clone());
            if let Some(s) = servers {
                self.stun = Some(StunClient::new(&s, self.seed));
            }
        }
        if let Some(st) = self.stun.as_mut() {
            for (to, req) in st.poll(now_ms) {
                send(to, &req);
            }
            if st.done(now_ms) {
                self.symmetric = st.symmetric();
                if self.public.is_none() {
                    self.public = st.public();
                }
                if self.state == HostState::Probing && self.outcome.is_some() {
                    self.state = if self.public.is_some() { HostState::StunOnly } else { HostState::Failed };
                }
            }
        }
        if let Some(due) = self.renew_due_ms {
            if now_ms >= due {
                self.renew_due_ms = Some(now_ms + LEASE_S as u64 * 500);
                self.renew();
            }
        }
        for p in &mut self.punches {
            if let Some((to, pkt)) = p.poll(now_ms) {
                send(to, &pkt);
            }
        }
        for (to, pkt) in self.out.drain(..) {
            send(to, &pkt);
        }
    }

    /// Offer every stray datagram (STUN replies, punches); true when consumed.
    pub fn on_packet(&mut self, from: SocketAddr, data: &[u8]) -> bool {
        for p in &mut self.punches {
            p.on_datagram(from);
        }
        if let Some((punch::PT_PROBE, nonce)) = punch::parse_probe(data) {
            if self.out.len() < 16 {
                self.out.push((from, punch::probe_packet(punch::PT_PROBE_ACK, nonce)));
            }
            return true;
        }
        if let Some(st) = self.stun.as_mut() {
            if st.on_packet(from, data) {
                return true;
            }
        }
        punch::is_punch(data)
    }

    /// A session peer connected from `from` (stops punching it).
    pub fn peer_seen(&mut self, from: SocketAddr) {
        for p in &mut self.punches {
            p.on_datagram(from);
        }
    }

    /// The host typed a joiner's code: punch toward it.
    pub fn expect_peer(&mut self, code: &str) -> Result<SocketAddr, CodeError> {
        let jc = parse_target(code)?;
        let to = SocketAddr::V4(jc.addr);
        self.punches.retain(|p| p.target != to);
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.punches.push(Puncher::new(to, self.seed.to_le_bytes()));
        Ok(to)
    }

    pub fn status(&self) -> HostStatus {
        let mut flags = 0;
        if matches!(self.state, HostState::StunOnly) {
            flags |= FLAG_NEEDS_PUNCH;
        }
        if self.symmetric {
            flags |= FLAG_SYMMETRIC;
        }
        HostStatus {
            state: self.state,
            public: self.public,
            code: (!matches!(self.state, HostState::Probing | HostState::Failed)).then(|| self.public.map(|p| JoinCode::new(p, flags).encode())).flatten(),
            symmetric: self.symmetric,
            local_port: self.port,
            note: self.note.clone(),
            punches: self.punches.iter().map(|p| (p.target, p.state)).collect(),
        }
    }

    fn renew(&self) {
        let port = self.port;
        match self.outcome.clone() {
            Some(MapOutcome::Upnp { gw, public, .. }) => {
                std::thread::spawn(move || {
                    let _ = gw.add_mapping(public.port(), port, LEASE_S);
                });
            }
            Some(MapOutcome::NatPmp { gw, .. }) => {
                std::thread::spawn(move || {
                    let _ = natpmp::map_port(gw, port, LEASE_S);
                });
            }
            _ => {}
        }
    }

    /// Remove the mapping (blocking, short timeouts — call on close/exit).
    pub fn close(&mut self) {
        match self.outcome.take() {
            Some(MapOutcome::Upnp { gw, public, .. }) => {
                let _ = gw.delete_mapping(public.port());
            }
            Some(MapOutcome::NatPmp { gw, .. }) => {
                let _ = natpmp::delete(gw, self.port);
            }
            _ => {}
        }
        self.renew_due_ms = None;
    }

    pub fn config(&self) -> &ReachConfig {
        &self.cfg
    }
}

// ───────────────────────────── joiner ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinState {
    /// Learning our own public endpoint (for "my code").
    Resolving,
    /// Our code is known (or STUN gave up); the session's HELLOs are on their way.
    Ready,
}

/// The join page's reachability getter (docs §4). The connect itself is the session's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JoinStatus {
    pub state: JoinState,
    /// The target the typed text decoded to.
    pub target: SocketAddrV4,
    /// The host has no mapping: give it `my_code` so it can punch.
    pub host_needs_punch: bool,
    /// This side's code (its STUN endpoint), to give to a host that needs to punch.
    pub my_code: Option<String>,
    /// Both sides behind endpoint-dependent NAT: punching cannot work, a relay would be needed.
    pub symmetric_both: bool,
}

pub struct JoinReach {
    target: JoinCode,
    mapper: Arc<Mutex<MapperResult>>,
    stun: Option<StunClient>,
    seed: u64,
    /// §3a: the next PROBE time, and the host's LAN address once a PROBE_ACK came back.
    probe_next_ms: u64,
    probes_sent: u32,
    lan_host: Option<SocketAddr>,
    redirected: bool,
}

/// §3a: PROBEs go out every this many ms, this many times.
pub const PROBE_EVERY_MS: u64 = 400;
pub const PROBE_TRIES: u32 = 10;

impl JoinReach {
    /// Decode the typed text (a code or `a.b.c.d:port`) and start STUN for "my code".
    pub fn start(typed: &str, cfg: &ReachConfig, seed: u64) -> Result<JoinReach, CodeError> {
        let target = parse_target(typed)?;
        let cfg = ReachConfig { map: false, ..cfg.clone() };
        Ok(JoinReach { target, mapper: spawn_mapper(0, &cfg), stun: None, seed, probe_next_ms: 0, probes_sent: 0, lan_host: None, redirected: false })
    }

    pub fn target(&self) -> SocketAddr {
        SocketAddr::V4(self.target.addr)
    }

    pub fn poll(&mut self, now_ms: u64, send: &mut dyn FnMut(SocketAddr, &[u8])) {
        if self.stun.is_none() {
            if let Some(s) = self.mapper.lock().ok().and_then(|m| m.stun.clone()) {
                self.stun = Some(StunClient::new(&s, self.seed));
            }
        }
        if let Some(st) = self.stun.as_mut() {
            for (to, req) in st.poll(now_ms) {
                send(to, &req);
            }
        }
        // §3a: behind the host's own NAT (our public IP = the code's) → find it on the LAN.
        let same_nat = self.stun.as_ref().and_then(|s| s.public()).is_some_and(|me| me.ip() == self.target.addr.ip());
        if same_nat && self.lan_host.is_none() && self.probes_sent < PROBE_TRIES && now_ms >= self.probe_next_ms {
            self.probes_sent += 1;
            self.probe_next_ms = now_ms + PROBE_EVERY_MS;
            let pkt = punch::probe_packet(punch::PT_PROBE, self.seed.to_le_bytes());
            let port = self.target.addr.port();
            send(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::BROADCAST, port)), &pkt);
            // The same PC (a second window): broadcasts do not always loop back.
            send(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)), &pkt);
        }
    }

    pub fn on_packet(&mut self, from: SocketAddr, data: &[u8]) -> bool {
        if let Some((punch::PT_PROBE_ACK, nonce)) = punch::parse_probe(data) {
            if nonce == self.seed.to_le_bytes() && self.lan_host.is_none() {
                self.lan_host = Some(from);
            }
            return true;
        }
        self.stun.as_mut().is_some_and(|s| s.on_packet(from, data)) || punch::is_punch(data)
    }

    /// §3a: the host's LAN address, once (the caller re-dials it).
    pub fn take_redirect(&mut self) -> Option<SocketAddr> {
        if self.redirected {
            return None;
        }
        let a = self.lan_host?;
        self.redirected = true;
        Some(a)
    }

    pub fn status(&self, now_ms: u64) -> JoinStatus {
        let done = self.stun.as_ref().is_some_and(|s| s.done(now_ms) || s.public().is_some());
        let me = self.stun.as_ref().and_then(|s| s.public());
        let my_sym = self.stun.as_ref().is_some_and(|s| s.symmetric());
        let mut flags = FLAG_NEEDS_PUNCH;
        if my_sym {
            flags |= FLAG_SYMMETRIC;
        }
        JoinStatus {
            state: if done { JoinState::Ready } else { JoinState::Resolving },
            target: self.target.addr,
            host_needs_punch: self.target.flags & FLAG_NEEDS_PUNCH != 0,
            my_code: me.map(|m| JoinCode::new(m, flags).encode()),
            symmetric_both: my_sym && self.target.flags & FLAG_SYMMETRIC != 0,
        }
    }
}
