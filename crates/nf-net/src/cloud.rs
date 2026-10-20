//! The Cloudflare lobby service's game-side client (net-cloud-lobby, docs/systems/net-cloud-lobby.md) [S port
//! design]: the built-in meeting point. Lists live public lobbies and resolves 4-character codes to the
//! host's endpoints — play stays P2P, the service never sees a packet of the match.
//!
//! Transport: the system `curl` (`curl.exe` on Windows) exactly as the launcher's updater runs it
//! (nf-view/src/launcher/update.rs) — one short-lived process per request, captured output, **on a
//! background thread** (the game never blocks; the answer comes back through a channel the next poll
//! drains). No new crates, no TLS stack of our own.
//!
//! - [`CloudHost`]: the host's register/refresh (`POST /v1/host` every [`CLOUD_REFRESH_MS`]) and leave
//!   (`DELETE /v1/host`). The answer's 4-character code shows while fresh ([`CLOUD_FRESH_MS`] — the
//!   entry expires server-side at 90 s, so a service that stopped answering drops the code too) and
//!   `join_code_display` prefers it over the `nf-lobbyd` short code and the long code.
//! - [`CloudBrowse`]: the joiner's public-lobby list (`GET /v1/lobbies?version=…`) refreshed every
//!   [`CLOUD_LIST_EVERY_MS`] while the page is open. Rows never carry an IP.
//! - [`CloudLookup`]: one code lookup (`GET /v1/code/{CODE}`) → the host's public endpoint (+ its LAN
//!   endpoint for the same-network path) — the joiner then dials exactly as a long code does.
//!
//! The service URL: [`LOBBY_SERVICE_URL`] (empty until the user deploys = off), overridable by
//! `NF_LOBBY_SERVICE` or the `lobby_service =` line of net.txt — resolved in nf-view, which owns the
//! config dir; every client takes its URL explicitly so tests can point at a local mock server.
//! When the player-set `nf-lobbyd` (`lobby_server =`) is configured it overrides this service entirely.

use std::io::Read;
use std::net::SocketAddrV4;
use std::process::Command;
use std::sync::mpsc;

use crate::directory::{SHORT_CODE_ALPHABET, SHORT_CODE_LEN};

/// The deployed service's URL — **empty until the user deploys the worker (docs/tools/cloud-lobby.md),
/// which means the feature is off**; overridable per player by `NF_LOBBY_SERVICE` or the
/// `lobby_service =` line of `%APPDATA%\NightfirePort\net.txt` [S].
pub const LOBBY_SERVICE_URL: &str = "";
/// The host refreshes every 30 s (the entry expires after 90 s) [S].
pub const CLOUD_REFRESH_MS: u64 = 30_000;
/// The host shows the service's code while its last refresh succeeded within this window: the 90 s
/// server-side expiry + one refresh period, so a dead service drops the code within ~2 min [S].
pub const CLOUD_FRESH_MS: u64 = CLOUD_REFRESH_MS * 4;
/// The join page refreshes the public-lobby list every 5 s while it is open [S].
pub const CLOUD_LIST_EVERY_MS: u64 = 5_000;
/// One HTTPS attempt gives up after this long (the poll keeps going; only the request times out) [S].
pub const CLOUD_TIMEOUT_SECS: u32 = 8;

// ── HTTP through the system curl (the updater's shape, on a background thread) ──────────────

/// One request for [`http_send`].
pub struct HttpRequest {
    pub method: &'static str,
    pub url: String,
    /// The JSON body for POST / DELETE (the service caps it at 2 KB).
    pub body: Option<String>,
    pub max_secs: u32,
}

/// One answer: the HTTP status and the (whole, short) body.
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

fn system_tool() -> std::path::PathBuf {
    if cfg!(windows) {
        let root = std::env::var_os("SystemRoot").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"));
        let p = root.join("System32").join("curl.exe");
        if p.exists() { return p; }
        return std::path::PathBuf::from("curl.exe");
    }
    std::path::PathBuf::from("curl")
}

/// One blocking HTTPS exchange through the system curl — background threads only. The status rides the
/// output's last line (`-w`), so 404/429 answers come back as statuses instead of transport errors.
pub fn http_send(req: HttpRequest) -> Result<HttpResponse, String> {
    let mut cmd = Command::new(system_tool());
    cmd.args(["-sS", "--max-time", &req.max_secs.to_string(), "-H", "User-Agent: NightfirePort-lobby",
        "-H", "Accept: application/json", "-w", "\n%{http_code}", "-X", req.method, "-o", "-"]);
    if let Some(b) = &req.body {
        cmd.args(["-H", "Content-Type: application/json", "--data-binary"]).arg(b);
    }
    cmd.arg(&req.url);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW, as the updater does
    }
    let o = cmd.output().map_err(|e| format!("couldn't run curl: {e}"))?;
    if !o.status.success() {
        return Err(format!("curl failed ({}): {}", o.status, String::from_utf8_lossy(&o.stderr).trim()));
    }
    let out = String::from_utf8_lossy(&o.stdout);
    let (body, code) = out.rsplit_once('\n').ok_or("curl printed nothing")?;
    let status: u16 = code.trim().parse().map_err(|_| format!("curl printed no status: {code:?}"))?;
    Ok(HttpResponse { status, body: body.to_string() })
}

/// A request running on its background thread; [`Pending::take`] drains the answer once it landed.
pub struct Pending {
    rx: mpsc::Receiver<Result<HttpResponse, String>>,
}

/// Starts `req` on a fresh background thread (one short-lived thread per request; the cadences here are
/// tens of seconds, never per frame) [S].
pub fn spawn_http(req: HttpRequest) -> Pending {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("nf-cloud".into())
        .spawn(move || { let _ = tx.send(http_send(req)); })
        .expect("spawn the cloud request thread");
    Pending { rx }
}

impl Pending {
    /// The answer when it has arrived (None = still in flight). Never blocks.
    pub fn take(&mut self) -> Option<Result<HttpResponse, String>> {
        self.rx.try_recv().ok()
    }
}

// ── the JSON we speak (std only; the shapes are §1 of the system doc) ────────────────────────

/// A minimal JSON value — the service's answers are small and flat.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(v) => v.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Json::Num(n) if *n >= 0.0 && *n <= u32::MAX as f64 && n.fract() == 0.0 => Some(*n as u32),
            _ => None,
        }
    }
    pub fn as_u8(&self) -> Option<u8> {
        self.as_u32().filter(|v| *v <= u8::MAX as u32).map(|v| v as u8)
    }
}

/// Parses one JSON text (objects, arrays, strings with the two-byte escapes, numbers, bools, null).
pub fn json_parse(s: &str) -> Option<Json> {
    let b = s.as_bytes();
    let mut i = 0usize;
    let v = parse_value(b, &mut i)?;
    skip_ws(b, &mut i);
    (i == b.len()).then_some(v)
}

fn skip_ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && matches!(b[*i], b' ' | b'\t' | b'\n' | b'\r') {
        *i += 1;
    }
}

fn parse_value(b: &[u8], i: &mut usize) -> Option<Json> {
    skip_ws(b, i);
    match *b.get(*i)? {
        b'{' => {
            *i += 1;
            let mut out = Vec::new();
            skip_ws(b, i);
            if b.get(*i) == Some(&b'}') {
                *i += 1;
                return Some(Json::Obj(out));
            }
            loop {
                skip_ws(b, i);
                let k = match parse_value(b, i)? {
                    Json::Str(s) => s,
                    _ => return None,
                };
                skip_ws(b, i);
                if b.get(*i) != Some(&b':') { return None; }
                *i += 1;
                let v = parse_value(b, i)?;
                out.push((k, v));
                skip_ws(b, i);
                match b.get(*i)? {
                    b',' => *i += 1,
                    b'}' => {
                        *i += 1;
                        return Some(Json::Obj(out));
                    }
                    _ => return None,
                }
            }
        }
        b'[' => {
            *i += 1;
            let mut out = Vec::new();
            skip_ws(b, i);
            if b.get(*i) == Some(&b']') {
                *i += 1;
                return Some(Json::Arr(out));
            }
            loop {
                out.push(parse_value(b, i)?);
                skip_ws(b, i);
                match b.get(*i)? {
                    b',' => *i += 1,
                    b']' => {
                        *i += 1;
                        return Some(Json::Arr(out));
                    }
                    _ => return None,
                }
            }
        }
        b'"' => {
            *i += 1;
            let mut s = String::new();
            loop {
                let c = *b.get(*i)?;
                *i += 1;
                match c {
                    b'"' => return Some(Json::Str(s)),
                    b'\\' => match b.get(*i)? {
                        b'"' => { s.push('"'); *i += 1; }
                        b'\\' => { s.push('\\'); *i += 1; }
                        b'/' => { s.push('/'); *i += 1; }
                        b'n' => { s.push('\n'); *i += 1; }
                        b't' => { s.push('\t'); *i += 1; }
                        b'r' => { s.push('\r'); *i += 1; }
                        b'b' => { s.push('\u{8}'); *i += 1; }
                        b'f' => { s.push('\u{c}'); *i += 1; }
                        b'u' => {
                            *i += 1; // past the 'u'
                            if *i + 4 > b.len() { return None; }
                            let hex = std::str::from_utf8(&b[*i..*i + 4]).ok()?;
                            let cp = u32::from_str_radix(hex, 16).ok()?;
                            *i += 4;
                            // A surrogate pair (the service never sends one, but a name may carry one).
                            if (0xD800..0xDC00).contains(&cp) {
                                if b.get(*i) != Some(&b'\\') || b.get(*i + 1) != Some(&b'u') { return None; }
                                *i += 2;
                                let hex2 = std::str::from_utf8(b.get(*i..*i + 4)?).ok()?;
                                let lo = u32::from_str_radix(hex2, 16).ok()?;
                                if !(0xDC00..0xE000).contains(&lo) { return None; }
                                *i += 4;
                                s.push(char::from_u32(0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00))?);
                            } else {
                                s.push(char::from_u32(cp)?);
                            }
                        }
                        _ => return None,
                    },
                    c if c < 0x20 => return None,
                    c => {
                        // Re-encode the UTF-8 byte sequence starting here.
                        let len = (c < 0x80) as usize + ((0xC0..0xE0).contains(&c)) as usize * 1
                            + ((0xE0..0xF0).contains(&c)) as usize * 2 + ((0xF0..0xF8).contains(&c)) as usize * 3;
                        let end = (*i - 1) + len;
                        if len == 0 || end > b.len() { return None; }
                        s.push_str(std::str::from_utf8(&b[(*i - 1)..end]).ok()?);
                        *i = end;
                    }
                }
            }
        }
        b't' => { (b.get(*i..*i + 4) == Some(b"true")).then(|| { *i += 4; Json::Bool(true) }) }
        b'f' => { (b.get(*i..*i + 5) == Some(b"false")).then(|| { *i += 5; Json::Bool(false) }) }
        b'n' => { (b.get(*i..*i + 4) == Some(b"null")).then(|| { *i += 4; Json::Null }) }
        _ => {
            let start = *i;
            if b.get(*i) == Some(&b'-') { *i += 1; }
            while matches!(b.get(*i), Some(c) if c.is_ascii_digit() || *c == b'.' || *c == b'e' || *c == b'E' || *c == b'+' || *c == b'-') {
                *i += 1;
            }
            if *i == start { return None; }
            let t = std::str::from_utf8(&b[start..*i]).ok()?;
            t.parse::<f64>().ok().map(Json::Num)
        }
    }
}

/// A JSON string literal (the request bodies; the answers are parsed, not written).
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A code is 4 symbols of the short-code alphabet (the service's alphabet is the same one,
/// directory.rs) — a lookup whose answer is not of that shape is refused [S].
fn code_ok(code: &str) -> bool {
    code.len() == SHORT_CODE_LEN && code.bytes().all(|b| SHORT_CODE_ALPHABET.contains(&b))
}

/// The version string the host registers and the joiner lists carry: `"<protocol>.<build>"` —
/// the service string-matches it, so an older joiner never sees a lobby it cannot join [S].
pub fn game_version() -> String {
    format!("{}.{}", crate::PROTOCOL_VERSION, crate::DEFAULT_BUILD_ID)
}

// ── the host: register / refresh / leave ─────────────────────────────────────────────────────

/// What the host tells the service about its lobby (the POST /v1/host body; §1 of the system doc) [S].
pub struct CloudAdvert {
    /// `<protocol>.<build>` — the service string-matches it for the list (older joiners never see a
    /// lobby they cannot join) [S].
    pub version: String,
    pub name: String,
    pub map: u32,
    pub scenario: u32,
    pub players: u8,
    pub max_players: u8,
    /// false = code only (the entry resolves but never lists) [S].
    pub public: bool,
    /// The host's LAN endpoint ("a.b.c.d:port") for the same-network path; empty when unknown.
    pub lan: String,
    /// The UDP port a joiner dials on the public IP — the STUN-observed mapping's port when
    /// reachability knows one, else the game port [S].
    pub port: u16,
}

/// The host's cloud bookkeeping: one POST every [`CLOUD_REFRESH_MS`]; the answer's code is the lobby's
/// short code while fresh. Requests run on background threads; [`Self::poll`] drains them [S].
pub struct CloudHost {
    url: String,
    advert: Option<CloudAdvert>,
    inflight: Option<Pending>,
    leaving: bool,
    code: Option<String>,
    /// When the code stops showing (the last successful refresh + [`CLOUD_FRESH_MS`]).
    fresh_until: u64,
}

impl CloudHost {
    pub fn new(url: String) -> CloudHost {
        CloudHost { url, advert: None, inflight: None, leaving: false, code: None, fresh_until: 0 }
    }

    pub fn service_url(&self) -> &str {
        &self.url
    }

    /// Sets this refresh period's body (the next poll POSTs it). The same while refreshed = the same
    /// entry = the same code (the service keys the entry by the requester's IP + our port) [S].
    pub fn set_advert(&mut self, a: CloudAdvert) {
        self.advert = Some(a);
    }

    /// `DELETE /v1/host`, fire and forget on its own thread — the lobby is closing and must not wait [S].
    pub fn leave(&mut self) {
        if self.leaving { return; }
        self.leaving = true;
        self.inflight = None;
        let url = format!("{}/v1/host", self.url.trim_end_matches('/'));
        std::thread::Builder::new()
            .name("nf-cloud-leave".into())
            .spawn(move || {
                let _ = http_send(HttpRequest { method: "DELETE", url, body: None, max_secs: CLOUD_TIMEOUT_SECS });
            })
            .ok();
    }

    /// Starts the period's POST when due; drains the previous one's answer.
    pub fn poll(&mut self, now_ms: u64) {
        if self.leaving { return; }
        if let Some(ans) = self.inflight.as_mut().and_then(Pending::take) {
            self.inflight = None;
            if let Ok(r) = ans {
                if r.status == 200 {
                    if let Some(code) = json_parse(&r.body).and_then(|v| v.get("code").and_then(Json::as_str).map(str::to_string)) {
                        if code_ok(&code) {
                            self.code = Some(code);
                            self.fresh_until = now_ms.saturating_add(CLOUD_FRESH_MS);
                        }
                    }
                }
            }
        }
        if self.inflight.is_none() && self.advert.is_some() {
            let a = self.advert.as_ref().expect("advert checked");
            let body = format!(
                "{{\"version\":{},\"name\":{},\"map\":{},\"scenario\":{},\"players\":{},\"max\":{},\"public\":{},\"lan\":{},\"port\":{}}}",
                json_str(&a.version), json_str(&a.name), a.map, a.scenario, a.players, a.max_players, a.public, json_str(&a.lan), a.port);
            let url = format!("{}/v1/host", self.url.trim_end_matches('/'));
            self.inflight = Some(spawn_http(HttpRequest { method: "POST", url, body: Some(body), max_secs: CLOUD_TIMEOUT_SECS }));
            self.advert = None; // one POST per period; the caller sets a fresh advert next period
        }
    }

    /// The service's code while fresh (`None` → the caller falls back to the `nf-lobbyd` short code /
    /// the long code) [S].
    pub fn code(&self, now_ms: u64) -> Option<&str> {
        (now_ms <= self.fresh_until).then_some(()).and(self.code.as_deref())
    }

    /// A request is out (the tests wait for it).
    pub fn busy(&self) -> bool {
        self.inflight.is_some()
    }
}

// ── the joiner: the public list, and one code lookup ─────────────────────────────────────────

/// One row of the public-lobby list (`GET /v1/lobbies`): display fields only — never an address [S].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudLobby {
    pub code: String,
    pub name: String,
    /// The arena's level code (the joiner's draw maps it to the arena name).
    pub map: u32,
    pub scenario: u32,
    pub players: u8,
    pub max_players: u8,
}

/// The join page's list poller: `GET /v1/lobbies?version=…` every [`CLOUD_LIST_EVERY_MS`] [S].
pub struct CloudBrowse {
    url: String,
    version: String,
    rows: Vec<CloudLobby>,
    inflight: Option<Pending>,
    next_ms: u64,
}

impl CloudBrowse {
    pub fn open(url: String, version: String) -> CloudBrowse {
        CloudBrowse { url, version, rows: Vec::new(), inflight: None, next_ms: 0 }
    }

    /// The rows now (the last answer's; empty until one lands) [S].
    pub fn rows(&self) -> &[CloudLobby] {
        &self.rows
    }

    /// A list request is out (the tests wait for it).
    pub fn busy(&self) -> bool {
        self.inflight.is_some()
    }

    pub fn poll(&mut self, now_ms: u64) {
        if let Some(ans) = self.inflight.as_mut().and_then(Pending::take) {
            self.inflight = None;
            if let Ok(r) = ans {
                if r.status == 200 {
                    if let Some(list) = json_parse(&r.body).and_then(|v| v.get("lobbies").cloned()) {
                        if let Json::Arr(items) = list {
                            self.rows = items.into_iter().filter_map(|it| {
                                Some(CloudLobby {
                                    code: it.get("code")?.as_str()?.to_string(),
                                    name: it.get("name")?.as_str()?.to_string(),
                                    map: it.get("map")?.as_u32()?,
                                    scenario: it.get("scenario")?.as_u32()?,
                                    players: it.get("players")?.as_u8()?,
                                    max_players: it.get("max")?.as_u8()?,
                                })
                            }).filter(|r| code_ok(&r.code)).collect();
                        }
                    }
                }
            }
        }
        if self.inflight.is_none() && now_ms >= self.next_ms {
            self.next_ms = now_ms + CLOUD_LIST_EVERY_MS;
            let url = format!("{}/v1/lobbies?version={}", self.url.trim_end_matches('/'), urlencode(&self.version));
            self.inflight = Some(spawn_http(HttpRequest { method: "GET", url, body: None, max_secs: CLOUD_TIMEOUT_SECS }));
        }
    }
}

/// One joiner's code lookup (`GET /v1/code/{CODE}`) [S].
pub struct CloudLookup {
    url: String,
    code: String,
    inflight: Option<Pending>,
    answer: Option<Result<CloudFound, CloudLookupError>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudLookupError {
    /// 404 — nobody holds that code right now ("Code ABCD was not found").
    NotFound,
    /// Anything else (down, rate limited, offline) — the join page's note says the service is unreachable.
    Unreachable,
}

/// What a successful lookup found: the host's public endpoint (the dial) and its LAN endpoint (the
/// same-network probe hint), plus the display fields the join page shows while dialing [S].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudFound {
    pub public: SocketAddrV4,
    pub lan: Option<SocketAddrV4>,
    pub name: String,
    pub map: u32,
    pub scenario: u32,
    pub players: u8,
    pub max_players: u8,
}

fn parse_endpoint(s: &str) -> Option<SocketAddrV4> {
    let (ip, port) = s.split_once(':')?;
    let ip: std::net::Ipv4Addr = ip.parse().ok()?;
    let port: u16 = port.parse().ok()?;
    (port != 0).then_some(SocketAddrV4::new(ip, port))
}

impl CloudLookup {
    pub fn new(url: String, code: &str) -> CloudLookup {
        CloudLookup { url, code: code.to_string(), inflight: None, answer: None }
    }

    /// The code this lookup asks for, canonical form [S].
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn finished(&self) -> bool {
        self.answer.is_some()
    }

    /// The final answer, once [`Self::finished`].
    pub fn result(&self) -> Option<&Result<CloudFound, CloudLookupError>> {
        self.answer.as_ref()
    }

    pub fn poll(&mut self) {
        if self.answer.is_some() { return; }
        if self.inflight.is_none() {
            let url = format!("{}/v1/code/{}", self.url.trim_end_matches('/'), self.code);
            self.inflight = Some(spawn_http(HttpRequest { method: "GET", url, body: None, max_secs: CLOUD_TIMEOUT_SECS }));
            return;
        }
        if let Some(ans) = self.inflight.as_mut().and_then(Pending::take) {
            self.inflight = None;
            self.answer = Some(match ans {
                Err(_) => Err(CloudLookupError::Unreachable),
                Ok(r) if r.status == 404 => Err(CloudLookupError::NotFound),
                Ok(r) if r.status != 200 => Err(CloudLookupError::Unreachable),
                Ok(r) => match json_parse(&r.body) {
                    Some(v) => {
                        let public = v.get("public_addr").and_then(Json::as_str).and_then(parse_endpoint);
                        let lan = v.get("lan_addr").and_then(Json::as_str).filter(|s| !s.is_empty()).and_then(parse_endpoint);
                        match public {
                            Some(public) => Ok(CloudFound {
                                public,
                                lan,
                                name: v.get("name").and_then(Json::as_str).unwrap_or_default().to_string(),
                                map: v.get("map").and_then(Json::as_u32).unwrap_or(0),
                                scenario: v.get("scenario").and_then(Json::as_u32).unwrap_or(0),
                                players: v.get("players").and_then(Json::as_u8).unwrap_or(1),
                                max_players: v.get("max").and_then(Json::as_u8).unwrap_or(4),
                            }),
                            None => Err(CloudLookupError::Unreachable),
                        }
                    }
                    None => Err(CloudLookupError::Unreachable),
                },
            });
        }
    }
}

/// The version string in a URL query (a bare `8.1` is already safe; this is belt and braces).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip_shapes() {
        let v = json_parse(r#"{"code":"AB2C","public_addr":"203.0.113.5:27420","expires_in":90,"public":true,"lan_addr":""}"#).expect("parses");
        assert_eq!(v.get("code").and_then(Json::as_str), Some("AB2C"));
        assert_eq!(v.get("expires_in").and_then(Json::as_u32), Some(90));
        assert_eq!(v.get("public"), Some(&Json::Bool(true)));
        assert_eq!(v.get("lan_addr").and_then(Json::as_str), Some(""));
        assert_eq!(v.get("missing"), None);
        let arr = json_parse(r#"{"lobbies":[{"code":"H0K7","name":"a \"b\"","map":5,"players":2,"max":4}]}"#).expect("parses");
        let first = match arr.get("lobbies") {
            Some(Json::Arr(a)) => &a[0],
            _ => panic!("array"),
        };
        assert_eq!(first.get("name").and_then(Json::as_str), Some("a \"b\""));
        assert_eq!(first.get("map").and_then(Json::as_u32), Some(5));
        assert_eq!(json_parse("{broken"), None);
        assert_eq!(json_parse("{}trailing"), None);
        assert_eq!(json_parse("\"\\u00e9\\ud83d\\ude00\""), Some(Json::Str("é😀".into())));
    }

    #[test]
    fn json_writes_a_tight_host_body() {
        let a = CloudAdvert { version: "8.1".into(), name: "Bond's \"game\"".into(), map: 0x0700_0023, scenario: 1,
            players: 2, max_players: 4, public: false, lan: "192.168.1.10:27420".into(), port: 27420 };
        let body = format!(
            "{{\"version\":{},\"name\":{},\"map\":{},\"scenario\":{},\"players\":{},\"max\":{},\"public\":{},\"lan\":{},\"port\":{}}}",
            json_str(&a.version), json_str(&a.name), a.map, a.scenario, a.players, a.max_players, a.public, json_str(&a.lan), a.port);
        let v = json_parse(&body).expect("the body parses");
        assert_eq!(v.get("name").and_then(Json::as_str), Some("Bond's \"game\""));
        assert_eq!(v.get("map").and_then(Json::as_u32), Some(0x0700_0023));
        assert_eq!(v.get("public"), Some(&Json::Bool(false)));
        assert_eq!(v.get("port").and_then(Json::as_u32), Some(27420));
    }

    #[test]
    fn endpoints_parse_and_codes_shape_check() {
        assert_eq!(parse_endpoint("203.0.113.5:27420"), Some(SocketAddrV4::new([203, 0, 113, 5].into(), 27420)));
        assert_eq!(parse_endpoint("203.0.113.5:0"), None, "port 0 is no endpoint");
        assert_eq!(parse_endpoint("nope"), None);
        assert!(code_ok("AB2C"));
        assert!(!code_ok("0OIL"), "the unambiguous alphabet excludes 0/O/1/I/L");
        assert!(!code_ok("ABCDX"));
    }

    /// The whole host flow against a fake transport is covered by tests/cloud_lobby.rs's mock server
    /// (real HTTP through real curl on loopback); here only the fresh-window arithmetic.
    #[test]
    fn host_code_freshness_window() {
        let mut h = CloudHost::new("http://127.0.0.1:1".into());
        assert!(h.code(0).is_none(), "no refresh yet — no code");
        h.code = Some("AB2C".into());
        h.fresh_until = 1_000;
        assert_eq!(h.code(1_000), Some("AB2C"));
        assert_eq!(h.code(1_001), None, "stale — the long code shows again");
    }
}
