//! Player bug reports (user, 2026-10-19: "a way for them to report bugs … robust and useful for us so we can have
//! every necessary detail"; the inbox is GitHub Issues).
//!
//! F10 in the game (or the launcher's "Report crash" after a crash) builds a report zip in
//! `%APPDATA%\NightfirePort\reports\` and opens the repository's bug-report issue form pre-filled with the version,
//! the place and the system, with the zip selected in Explorer — the player types what happened in the form and drags
//! the zip in. The zip holds:
//!
//! | File | What |
//! |---|---|
//! | `report.txt` | `key: value` lines — version, commit, OS, GPU, display, the level / scenario / difficulty, Bond's state, the objectives, the tick, the online session, fps |
//! | `screenshot.png` | the frame at the moment of the report (in-game reports) |
//! | `session.nfr` | the input replay of the current level from its start ([`crate::replay`] format) |
//! | `entry.sav` | the carried state the level started with ([`encode_ram_save`]) — with `session.nfr`, `nf-view --bug-replay` re-runs the session exactly |
//! | `settings.toml` | the player's settings |
//! | `game-log.txt` | the end of the game's log |
//! | `crash.txt` | crash reports: the panic message and backtrace |
//!
//! Personal paths are scrubbed ([`scrub`]): the user-profile folder becomes `%USERPROFILE%` and the user name `<user>`.
//! Nothing is sent by the game itself; the player submits the form.

use std::path::{Path, PathBuf};

/// The repository the issues go to (the releases' repository).
pub const REPO: &str = crate::launcher::update::REPO;
/// The issue form (`.github/ISSUE_TEMPLATE/bug_report.yml`).
pub const TEMPLATE: &str = "bug_report.yml";

/// The GPU adapter the renderer picked (name, backend, driver) — set once at device creation.
pub static GPU_INFO: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// `%APPDATA%\NightfirePort\reports`.
pub fn reports_dir() -> Option<PathBuf> { crate::settings::config_dir().map(|d| d.join("reports")) }

/// The build's version line: `0.1.3 (commit abc1234)` / `dev build (commit …)`.
pub fn version_line() -> String {
    let v = crate::launcher::update::current_version().map(str::to_string).unwrap_or_else(|| "dev build".into());
    format!("{v} (commit {})", crate::replay::git_hash())
}

/// Windows' version string (`Microsoft Windows [Version 10.0.19045.5371]`), else the OS family.
pub fn os_line() -> String {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(o) = std::process::Command::new("cmd").args(["/c", "ver"]).creation_flags(0x0800_0000).output() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() { return s; }
        }
    }
    std::env::consts::OS.to_string()
}

/// Replaces the user-profile folder with `%USERPROFILE%` and the user name with `<user>` (paths in logs, settings).
pub fn scrub(text: &str) -> String {
    let mut out = text.to_string();
    for var in ["USERPROFILE", "HOME"] {
        if let Ok(p) = std::env::var(var) {
            if p.len() > 3 {
                out = out.replace(&p, "%USERPROFILE%");
                out = out.replace(&p.replace('\\', "/"), "%USERPROFILE%");
            }
        }
    }
    for var in ["USERNAME", "USER"] {
        if let Ok(u) = std::env::var(var) {
            if u.len() >= 3 { out = out.replace(&u, "<user>"); }
        }
    }
    out
}

/// `YYYYMMDD-HHMMSS` (UTC) from the system clock — the report's id and file name.
pub fn report_id() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, rem / 60 % 60, rem % 60)
}

// ── the entry save (the carried state a level starts with) ─────────────────────────────────────────────────────

const SAV_MAGIC: &[u8; 4] = b"NFES";
const SAV_VERSION: u16 = 1;

fn put_u32(o: &mut Vec<u8>, v: u32) { o.extend_from_slice(&v.to_le_bytes()); }

/// Serialises a [`nf_game::player::RamSave`] (all fields the port carries) for the report.
pub fn encode_ram_save(s: &nf_game::player::RamSave) -> Vec<u8> {
    let mut o = Vec::new();
    o.extend_from_slice(SAV_MAGIC);
    o.extend_from_slice(&SAV_VERSION.to_le_bytes());
    o.extend_from_slice(&s.health.to_le_bytes());
    o.extend_from_slice(&s.armour.to_le_bytes());
    o.push(s.weapon);
    put_u32(&mut o, s.carried.len() as u32);
    for v in &s.carried { o.extend_from_slice(&v.to_le_bytes()); }
    put_u32(&mut o, s.clip.len() as u32);
    for v in &s.clip { o.extend_from_slice(&v.to_le_bytes()); }
    put_u32(&mut o, s.owned.len() as u32);
    o.extend(s.owned.iter().map(|&b| b as u8));
    put_u32(&mut o, s.keys.len() as u32);
    o.extend(s.keys.iter().map(|&b| b as u8));
    put_u32(&mut o, s.channels.len() as u32);
    o.extend_from_slice(&s.channels);
    put_u32(&mut o, s.ch_time.len() as u32);
    for v in &s.ch_time { put_u32(&mut o, *v); }
    match &s.mission {
        None => o.push(0),
        Some(m) => {
            o.push(1);
            for v in [m.base_map, m.order, m.internal, m.state, m.fail_label, m.to_end] { put_u32(&mut o, v); }
            put_u32(&mut o, m.states.len() as u32);
            for v in &m.states { put_u32(&mut o, *v); }
        }
    }
    o
}

/// Reads [`encode_ram_save`]'s bytes back.
pub fn decode_ram_save(b: &[u8]) -> Result<nf_game::player::RamSave, String> {
    struct R<'a> { b: &'a [u8], p: usize }
    impl R<'_> {
        fn take(&mut self, n: usize) -> Result<&[u8], String> {
            let s = self.b.get(self.p..self.p + n).ok_or("entry.sav is truncated")?;
            self.p += n;
            Ok(s)
        }
        fn u8(&mut self) -> Result<u8, String> { Ok(self.take(1)?[0]) }
        fn u16(&mut self) -> Result<u16, String> { Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap())) }
        fn u32(&mut self) -> Result<u32, String> { Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap())) }
        fn f32(&mut self) -> Result<f32, String> { Ok(f32::from_bits(self.u32()?)) }
        fn i16(&mut self) -> Result<i16, String> { Ok(self.u16()? as i16) }
        fn len(&mut self) -> Result<usize, String> { let n = self.u32()? as usize; if n > 1 << 16 { Err("entry.sav length out of range".into()) } else { Ok(n) } }
    }
    let mut r = R { b, p: 0 };
    if r.take(4)? != SAV_MAGIC { return Err("not an entry.sav".into()); }
    let ver = r.u16()?;
    if ver != SAV_VERSION { return Err(format!("entry.sav version {ver} (this build reads {SAV_VERSION})")); }
    let mut s = nf_game::player::RamSave { health: r.f32()?, armour: r.f32()?, weapon: r.u8()?, ..Default::default() };
    let n = r.len()?; s.carried = (0..n).map(|_| r.i16()).collect::<Result<_, _>>()?;
    let n = r.len()?; s.clip = (0..n).map(|_| r.i16()).collect::<Result<_, _>>()?;
    let n = r.len()?; s.owned = r.take(n)?.iter().map(|&b| b != 0).collect();
    let n = r.len()?; s.keys = r.take(n)?.iter().map(|&b| b != 0).collect();
    let n = r.len()?; s.channels = r.take(n)?.to_vec();
    let n = r.len()?; s.ch_time = (0..n).map(|_| r.u32()).collect::<Result<_, _>>()?;
    if r.u8()? == 1 {
        let mut f = [0u32; 6];
        for v in f.iter_mut() { *v = r.u32()?; }
        let n = r.len()?;
        let states = (0..n).map(|_| r.u32()).collect::<Result<_, _>>()?;
        s.mission = Some(nf_game::mission::MissionSave { base_map: f[0], order: f[1], internal: f[2], state: f[3], fail_label: f[4], to_end: f[5], states });
    }
    Ok(s)
}

// ── a store-only zip (no compression; the system unzip and GitHub read it) ──────────────────────────────────────

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 { crc = if crc & 1 != 0 { (crc >> 1) ^ 0xedb8_8320 } else { crc >> 1 }; }
    }
    !crc
}

/// A zip of `files` (name, bytes), stored uncompressed (PKWARE APPNOTE 4.3: local headers, central directory, end
/// record; DOS time zero).
pub fn zip_store(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let crc = crc32(data);
        let off = out.len() as u32;
        let n = name.as_bytes();
        let mut h = Vec::new();
        h.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        h.extend_from_slice(&20u16.to_le_bytes()); // version needed
        h.extend_from_slice(&0x0800u16.to_le_bytes()); // flags: UTF-8 names
        h.extend_from_slice(&0u16.to_le_bytes()); // method: stored
        h.extend_from_slice(&0u32.to_le_bytes()); // time + date
        h.extend_from_slice(&crc.to_le_bytes());
        h.extend_from_slice(&(data.len() as u32).to_le_bytes());
        h.extend_from_slice(&(data.len() as u32).to_le_bytes());
        h.extend_from_slice(&(n.len() as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&h);
        out.extend_from_slice(n);
        out.extend_from_slice(data);
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central.extend_from_slice(&h[4..30]);
        central.extend_from_slice(&0u16.to_le_bytes()); // comment
        central.extend_from_slice(&0u16.to_le_bytes()); // disk
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&off.to_le_bytes());
        central.extend_from_slice(n);
    }
    let cd_off = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&cd_off.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

/// Reads [`zip_store`]'s output (stored entries only) back into (name, bytes) — the `--bug-replay` side.
pub fn unzip_store(b: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut out = Vec::new();
    let mut p = 0usize;
    let u16_at = |i: usize| b.get(i..i + 2).map(|s| u16::from_le_bytes([s[0], s[1]]));
    let u32_at = |i: usize| b.get(i..i + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]));
    while u32_at(p) == Some(0x0403_4b50) {
        let method = u16_at(p + 8).ok_or("zip truncated")?;
        let size = u32_at(p + 18).ok_or("zip truncated")? as usize;
        let nlen = u16_at(p + 26).ok_or("zip truncated")? as usize;
        let xlen = u16_at(p + 28).ok_or("zip truncated")? as usize;
        if method != 0 { return Err("the report zip was re-compressed — use the original file".into()); }
        let name = String::from_utf8_lossy(b.get(p + 30..p + 30 + nlen).ok_or("zip truncated")?).into_owned();
        let start = p + 30 + nlen + xlen;
        let data = b.get(start..start + size).ok_or("zip truncated")?.to_vec();
        out.push((name, data));
        p = start + size;
    }
    if out.is_empty() { return Err("not a NightfirePort report zip".into()); }
    Ok(out)
}

// ── the issue form and the hand-off ───────────────────────────────────────────────────────────────────────────

fn url_enc(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => o.push(b as char),
            _ => o.push_str(&format!("%{b:02X}")),
        }
    }
    o
}

/// The pre-filled issue form: the `fields` are the form's field ids (`.github/ISSUE_TEMPLATE/bug_report.yml`).
pub fn issue_url(title: &str, fields: &[(&str, String)]) -> String {
    let mut u = format!("https://github.com/{REPO}/issues/new?template={TEMPLATE}&title={}", url_enc(title));
    for (k, v) in fields {
        // Long fields would push the URL past what browsers accept; the zip carries everything anyway.
        let v: String = v.chars().take(1500).collect();
        u.push_str(&format!("&{k}={}", url_enc(&v)));
    }
    u
}

/// Opens `url` in the default browser.
pub fn open_url(url: &str) -> Result<(), String> {
    if no_open() { eprintln!("[bug-report] (not opening) {url}"); return Ok(()); }
    #[cfg(windows)]
    {
        std::process::Command::new("rundll32.exe").args(["url.dll,FileProtocolHandler", url]).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

/// Shows `file` selected in Explorer (so the player can drag it into the form).
pub fn reveal(file: &Path) {
    if no_open() { return; }
    #[cfg(windows)]
    { let _ = std::process::Command::new("explorer.exe").arg(format!("/select,{}", file.display())).spawn(); }
    #[cfg(not(windows))]
    { if let Some(d) = file.parent() { let _ = std::process::Command::new("xdg-open").arg(d).spawn(); } }
}

/// The last `max_lines` of the game log (`<config dir>/nf-view.log`, which the launcher writes), scrubbed.
pub fn log_tail(max_lines: usize) -> Option<String> {
    let p = crate::settings::config_dir()?.join("nf-view.log");
    let text = std::fs::read(&p).ok()?;
    let text = String::from_utf8_lossy(&text);
    let lines: Vec<&str> = text.lines().collect();
    Some(scrub(&lines[lines.len().saturating_sub(max_lines)..].join("\n")))
}

// ── the always-on session recording and the crash context ─────────────────────────────────────────────────────

/// The current level's recording: its replay since the level (re)started, the carried state it started with and the
/// difficulty — what `--bug-replay` needs to re-run it. Shared (not an `App` field) so the crash hook can reach it.
pub struct Session {
    pub replay: crate::replay::Replay,
    pub entry: Option<nf_game::player::RamSave>,
    pub difficulty: u32,
    level_code: u32,
    last_frame: u32,
    pub truncated: bool,
}

/// One hour of ticks (60 Hz); past it the recording stops growing (the report says so).
pub const SESSION_CAP: usize = 60 * 60 * 60;

pub static SESSION: std::sync::Mutex<Option<Session>> = std::sync::Mutex::new(None);
/// A few lines describing where the player is (refreshed about once a second) — the crash report's `report.txt`.
pub static CONTEXT: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// Called right before each local `step_bond`: starts a new recording when the level changed or restarted (the
/// tick counter went back), then appends this tick's input.
pub fn record_tick(sim: &crate::scene::Sim, entry: Option<&nf_game::player::RamSave>, difficulty: u32, t: crate::replay::TickInput) {
    let Ok(mut g) = SESSION.lock() else { return };
    let frame = sim.drones.frame;
    let fresh = match g.as_ref() { Some(s) => s.level_code != sim.level || frame < s.last_frame, None => true };
    if fresh {
        // The same header `replay::Recorder::start` writes, taken from the Sim before this tick runs.
        let header = crate::replay::Header {
            flags: 0, level: format!("{:08x}.bin", sim.level), scenario: None, git_hash: crate::replay::git_hash(),
            rng_x: sim.drones.rng.x, rng_y: sim.drones.rng.y,
            bond_pos: sim.bond.pos, bond_yaw: sim.bond.yaw, bond_pitch: sim.bond.pitch,
            start_weapon: sim.gun.weapon, start_clip: sim.gun.clip, start_spare: sim.gun.spare,
        };
        let replay = crate::replay::Replay { header, ticks: Vec::new() };
        *g = Some(Session { replay, entry: entry.cloned(), difficulty, level_code: sim.level, last_frame: frame, truncated: false });
    }
    let s = g.as_mut().unwrap();
    s.last_frame = frame;
    if s.replay.ticks.len() < SESSION_CAP { s.replay.ticks.push(t); } else { s.truncated = true; }
}

/// The session's files for a report (`session.nfr`, `entry.sav`) plus a `report.txt` note.
pub fn session_files() -> (Vec<(String, Vec<u8>)>, String) {
    let Ok(g) = SESSION.try_lock() else { return (Vec::new(), "session: (busy)".into()) };
    let Some(s) = g.as_ref() else { return (Vec::new(), "session: none (no level was being played)".into()) };
    let mut files = vec![("session.nfr".to_string(), s.replay.to_bytes())];
    if let Some(e) = &s.entry { files.push(("entry.sav".to_string(), encode_ram_save(e))); }
    let note = format!("session: {} ticks of {}{}, difficulty {}, entry save {}",
        s.replay.ticks.len(), s.replay.header.level, if s.truncated { " (TRUNCATED at the 1-hour cap)" } else { "" },
        s.difficulty, if s.entry.is_some() { "yes" } else { "no (a cold level load)" });
    (files, note)
}

/// Test aids: `NF_BUG_REPORT_AT=<tick>` takes an F10 report at that level tick (the frame loop runs it, [`DUE`]);
/// `NF_BUG_REPORT_NO_OPEN` keeps the browser and Explorer closed (the report is still saved).
pub static DUE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn report_at() -> Option<u32> { std::env::var("NF_BUG_REPORT_AT").ok()?.parse().ok() }
fn no_open() -> bool { std::env::var_os("NF_BUG_REPORT_NO_OPEN").is_some() }

pub fn set_context(text: String) { if let Ok(mut c) = CONTEXT.try_lock() { *c = text; } }

/// Installs the panic hook: a panic writes a crash report (the panic message and backtrace, the context, the session
/// and the log) to the reports folder with a `.pending` marker the launcher picks up on its next start, then the
/// previous hook runs (the console message).
pub fn install_crash_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let bt = std::backtrace::Backtrace::force_capture();
        let id = report_id();
        let ctx = CONTEXT.try_lock().map(|c| c.clone()).unwrap_or_default();
        let (mut files, session_note) = session_files();
        let report = format!("kind: crash\nreport_id: {id}\nversion: {}\nos: {}\ngpu: {}\n{session_note}\n{ctx}\n", version_line(), os_line(),
            GPU_INFO.get().map(String::as_str).unwrap_or("(not yet chosen)"));
        files.insert(0, ("report.txt".into(), scrub(&report).into_bytes()));
        files.push(("crash.txt".into(), scrub(&format!("{info}\n\n{bt}")).into_bytes()));
        if let Some(t) = log_tail(3000) { files.push(("game-log.txt".into(), t.into_bytes())); }
        if let Ok(saved) = save(&id, files, String::new()) {
            let _ = std::fs::write(saved.zip.with_extension("pending"), scrub(&format!("{info}")));
            eprintln!("[bug-report] crash report saved: {}", saved.zip.display());
        }
        prev(info);
    }));
}

/// A crash report waiting to be sent (`<zip>.pending` beside it) — the launcher's "Report crash".
pub fn pending_crash() -> Option<(PathBuf, String)> {
    let dir = reports_dir()?;
    let mut v: Vec<PathBuf> = std::fs::read_dir(&dir).ok()?.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "pending")).collect();
    v.sort();
    let marker = v.pop()?;
    let summary = std::fs::read_to_string(&marker).unwrap_or_default();
    Some((marker.with_extension("zip"), summary))
}

/// Opens the issue form for a crash report and shows its zip; clears the marker.
pub fn send_crash(zip: &Path, summary: &str) -> Result<(), String> {
    let id = zip.file_stem().and_then(|s| s.to_str()).and_then(|s| s.strip_prefix("NightfirePort-report-")).unwrap_or("").to_string();
    let first = summary.lines().next().unwrap_or("").chars().take(90).collect::<String>();
    let url = issue_url(&format!("[Crash] {first}"), &[
        ("version", version_line()),
        ("report_id", id),
        ("what_happened", format!("The game crashed.\n\n{summary}\n\n(What were you doing just before?)")),
        ("details", format!("{}\n{}", os_line(), GPU_INFO.get().map(String::as_str).unwrap_or(""))),
    ]);
    open_url(&url)?;
    reveal(zip);
    let _ = std::fs::remove_file(zip.with_extension("pending"));
    Ok(())
}

// ── our side: re-run a report ─────────────────────────────────────────────────────────────────────────────────

/// `nf-view --bug-replay <report.zip> [--shot out.png]` (NF_ISO = the disc image): loads the reported level with the
/// report's entry save and difficulty, checks the start state against the recording's header, replays the session
/// headless and prints where Bond ends against what the report recorded; `--shot` renders the last frame. Returns the
/// lines it printed.
pub fn bug_replay(iso: &str, zip: &Path, shot: Option<&Path>) -> anyhow::Result<Vec<String>> {
    use anyhow::Context;
    let files = unzip_store(&std::fs::read(zip).with_context(|| format!("read {}", zip.display()))?).map_err(anyhow::Error::msg)?;
    let get = |n: &str| files.iter().find(|(f, _)| f == n).map(|(_, b)| b.clone());
    let report = String::from_utf8_lossy(&get("report.txt").context("the zip has no report.txt")?).into_owned();
    let field = |k: &str| report.lines().find_map(|l| l.strip_prefix(&format!("{k}: ")).map(str::to_string));
    let replay = crate::replay::Replay::from_bytes(&get("session.nfr").context("the report has no session recording (taken outside a level?)")?)?;
    let entry = get("entry.sav").map(|b| decode_ram_save(&b)).transpose().map_err(anyhow::Error::msg)?;
    let difficulty: u32 = field("difficulty").and_then(|d| d.parse().ok()).unwrap_or(2);
    let mut out = vec![format!("report {} — version {}", field("report_id").unwrap_or_default(), field("version").unwrap_or_default()),
        format!("level {} difficulty {difficulty}, {} ticks, entry save {}", replay.header.level, replay.ticks.len(), entry.is_some())];
    let wids: Vec<u8> = entry.as_ref().map(|e| vec![e.weapon]).unwrap_or_default();
    let mut scene = crate::load_with_carry_weapons_difficulty(iso, &replay.header.level, false, entry.as_ref(), &wids, difficulty)?;
    {
        let sim = scene.sim.as_ref().context("the level has no Sim")?;
        let h = &replay.header;
        let same = sim.drones.rng.x == h.rng_x && sim.drones.rng.y == h.rng_y && sim.bond.pos == h.bond_pos && sim.gun.weapon == h.start_weapon;
        out.push(if same { "start state matches the recording (rng, Bond, weapon)".into() } else {
            format!("START STATE DIFFERS — rng {:x}/{:x} vs {:x}/{:x}, bond {:?} vs {:?}, weapon {:#x} vs {:#x} (the replay may diverge)",
                sim.drones.rng.x, sim.drones.rng.y, h.rng_x, h.rng_y, sim.bond.pos, h.bond_pos, sim.gun.weapon, h.start_weapon)
        });
    }
    let states = crate::replay::replay_headless(&mut scene, &replay)?;
    if let Some(last) = states.last() {
        out.push(format!("after the replay: tick {} bond {:.2?} yaw {:.3} weapon {:#04x} clip {} spare {}", last.frame, last.bond_pos, last.bond_yaw, last.weapon, last.clip, last.spare));
    }
    if let Some(b) = field("bond") { out.push(format!("the report recorded: tick {} bond {b}", field("tick").unwrap_or_default())); }
    if let Some(p) = shot {
        crate::bug_replay_shot(&scene, p)?;
        out.push(format!("wrote {}", p.display()));
    }
    Ok(out)
}

/// One finished report.
pub struct Saved {
    pub id: String,
    pub zip: PathBuf,
    pub url: String,
}

/// Writes the report zip and its `.txt` copy of `report.txt` into the reports folder; returns where.
pub fn save(id: &str, files: Vec<(String, Vec<u8>)>, url: String) -> Result<Saved, String> {
    let dir = reports_dir().ok_or("no settings folder for the report")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let zip = dir.join(format!("NightfirePort-report-{id}.zip"));
    std::fs::write(&zip, zip_store(&files)).map_err(|e| format!("{}: {e}", zip.display()))?;
    Ok(Saved { id: id.to_string(), zip, url })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ram_save_round_trips() {
        let s = nf_game::player::RamSave {
            health: 72.5, armour: 30.0, weapon: 0x16,
            carried: vec![1, -2, 300], clip: vec![7, 0], owned: vec![true, false, true], keys: vec![false, true],
            channels: (0..=255u8).collect(), ch_time: vec![0, 123_456, u32::MAX],
            mission: Some(nf_game::mission::MissionSave { base_map: 0x0700_000b, order: 2, internal: 1, state: 3, fail_label: 9, to_end: 0, states: vec![4, 3, 2] }),
        };
        let b = encode_ram_save(&s);
        let r = decode_ram_save(&b).expect("decodes");
        assert_eq!(format!("{r:?}"), format!("{s:?}"));
        // Sister: a truncated file is refused, not half-read.
        assert!(decode_ram_save(&b[..b.len() - 3]).is_err());
    }

    #[test]
    fn zip_round_trips_and_has_the_right_crc() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926, "the CRC-32 check value");
        let files = vec![("report.txt".to_string(), b"version: 0.1.0\n".to_vec()), ("a/b.bin".to_string(), vec![0u8, 1, 2, 255])];
        let z = zip_store(&files);
        assert_eq!(unzip_store(&z).unwrap(), files);
        // Sister: a non-zip is refused.
        assert!(unzip_store(b"PK\x05\x06nope").is_err());
    }

    #[test]
    fn the_issue_url_encodes_fields() {
        let u = issue_url("[Bug] 07000001: door", &[("version", "0.1.0 (commit abc)".into()), ("details", "a&b=c\nd".into())]);
        assert!(u.starts_with("https://github.com/YodaMMM/NightfirePort/issues/new?template=bug_report.yml&title=%5BBug%5D%2007000001%3A%20door"), "{u}");
        assert!(u.contains("&version=0.1.0%20%28commit%20abc%29") && u.contains("&details=a%26b%3Dc%0Ad"), "{u}");
    }

    #[test]
    fn scrub_hides_the_profile_folder() {
        if let Ok(p) = std::env::var("USERPROFILE") {
            let s = scrub(&format!(r"{p}\AppData\Roaming\NightfirePort\settings.toml"));
            assert!(!s.contains(&p) && s.starts_with("%USERPROFILE%"), "{s}");
        }
    }

    #[test]
    fn report_ids_are_dated() {
        let id = report_id();
        assert_eq!(id.len(), 15, "{id}");
        assert!(id.starts_with("20") && id.as_bytes()[8] == b'-', "{id}");
    }
}
