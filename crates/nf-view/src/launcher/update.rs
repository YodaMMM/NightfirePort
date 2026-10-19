//! The launcher's self-update (user, 2026-10-19: "a github release and a way to auto update when we push fixes";
//! updates install automatically on launch).
//!
//! A release build carries its version (`NF_RELEASE_VERSION`, baked in by the release build — `tools/release`); a dev
//! build carries none and never updates. On start the launcher asks GitHub for the repository's latest release
//! (`releases/latest`); when its tag is newer it downloads the release's `NightfirePort.exe` and the `.sha256` beside
//! it (the user, 2026-10-19: a release is just the exe — no zip), checks the digest ([`nf_formats::sha256`] — a
//! corruption/truncation check; both files come over HTTPS from the same release) and swaps the exe in. Windows lets
//! a running program be renamed but not overwritten, so the running exe is first renamed to `<name>.old` (deleted on
//! the next start), then the new one is started and this one exits. Any failure (offline, GitHub down, no write
//! access to the folder) leaves the installed version untouched and the game playable.
//!
//! v0.1.0–v0.1.2 launchers only knew a `-windows.zip` asset, so the bridge release v0.1.3 also carries one (holding
//! just the exe) that moves them onto this updater.
//!
//! No new crates: the transfers use the system `curl.exe` (Windows 10 1803+ and Windows 11). `NF_UPDATE_API`
//! overrides the release query URL (tests point it at a local `file://` JSON).

use std::path::{Path, PathBuf};
use std::process::Command;

/// The GitHub repository the releases come from.
pub const REPO: &str = "YodaMMM/NightfirePort";

/// This build's release version (`0.1.7`), or None for a dev build (no updates).
pub fn current_version() -> Option<&'static str> {
    option_env!("NF_RELEASE_VERSION").filter(|v| !v.is_empty())
}

/// The version shown in the launcher's footer.
pub fn display_version() -> String {
    match current_version() {
        Some(v) => format!("v{v}"),
        None => format!("v{} (dev build)", env!("CARGO_PKG_VERSION")),
    }
}

/// One published release as the updater needs it.
#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub tag: String,
    pub version: (u32, u32, u32),
    pub exe_url: String,
    pub sha_url: String,
}

/// The release asset's file name.
pub const ASSET: &str = "NightfirePort.exe";

/// `v1.2.3` / `1.2.3` → (1, 2, 3).
pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    let mut it = s.split('.').map(|p| p.trim().parse::<u32>());
    let v = (it.next()?.ok()?, it.next()?.ok()?, it.next().map_or(Some(0), |p| p.ok())?);
    if it.next().is_some() { return None; }
    Some(v)
}

/// The string values of every `"key": "…"` pair in a JSON text, in order (the release query's few fields; JSON
/// escapes `\"`, `\\`, `\/` are undone — enough for tags and URLs).
fn json_strings(json: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\"");
    let mut out = Vec::new();
    let mut rest = json;
    while let Some(i) = rest.find(&needle) {
        rest = &rest[i + needle.len()..];
        let Some(colon) = rest.find(':') else { break };
        let after = rest[colon + 1..].trim_start();
        if !after.starts_with('"') { continue; }
        let body = &after[1..];
        let mut val = String::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => match chars.next() { Some('/') => val.push('/'), Some('"') => val.push('"'), Some('\\') => val.push('\\'), Some(o) => { val.push('\\'); val.push(o); } None => break },
                c => val.push(c),
            }
        }
        out.push(val);
    }
    out
}

/// Reads the `releases/latest` answer: the tag, and `NightfirePort.exe` + its `.sha256` among the assets.
pub fn parse_latest(json: &str) -> Option<Release> {
    let tag = json_strings(json, "tag_name").into_iter().next()?;
    let version = parse_version(&tag)?;
    let urls = json_strings(json, "browser_download_url");
    let exe_url = urls.iter().find(|u| u.ends_with(&format!("/{ASSET}")))?.clone();
    let sha_url = urls.iter().find(|u| u.ends_with(&format!("/{ASSET}.sha256")))?.clone();
    Some(Release { tag, version, exe_url, sha_url })
}

/// The first 64-hex-digit word of a `.sha256` file (`<hex>  <name>` or just the hex).
pub fn parse_sha_file(text: &str) -> Option<String> {
    let w = text.split_whitespace().next()?.to_ascii_lowercase();
    (w.len() == 64 && w.bytes().all(|b| b.is_ascii_hexdigit())).then_some(w)
}

fn system_tool(name: &str) -> PathBuf {
    if cfg!(windows) {
        let root = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        let p = root.join("System32").join(name);
        if p.exists() { return p; }
    }
    PathBuf::from(name)
}

fn quiet(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

/// `curl.exe` fetch of `url` into `out` (follows redirects; fails on HTTP errors).
fn fetch(url: &str, out: &Path, max_secs: u32) -> Result<(), String> {
    let o = quiet(Command::new(system_tool("curl.exe"))
        .args(["-fsSL", "--retry", "2", "--max-time", &max_secs.to_string(), "-H", "User-Agent: NightfirePort-launcher",
               "-H", "Accept: application/vnd.github+json", "-o"])
        .arg(out).arg(url))
        .output().map_err(|e| format!("couldn't run curl: {e}"))?;
    if o.status.success() { Ok(()) } else { Err(format!("download failed ({}): {}", o.status, String::from_utf8_lossy(&o.stderr).trim())) }
}

fn work_dir() -> PathBuf { std::env::temp_dir().join("NightfirePort-update") }

/// Asks for the latest release; Some when it is newer than this build. Dev builds never look.
pub fn check() -> Result<Option<Release>, String> {
    let Some(cur) = current_version().and_then(parse_version) else { return Ok(None) };
    let url = std::env::var("NF_UPDATE_API").unwrap_or_else(|_| format!("https://api.github.com/repos/{REPO}/releases/latest"));
    let dir = work_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("temp folder: {e}"))?;
    let json_path = dir.join("latest.json");
    fetch(&url, &json_path, 15)?;
    let json = std::fs::read_to_string(&json_path).map_err(|e| format!("release info: {e}"))?;
    let rel = parse_latest(&json).ok_or("the latest release has no Windows download")?;
    Ok((rel.version > cur).then_some(rel))
}

/// The running exe's file name (the player may have renamed it; the update keeps their name).
fn own_name() -> std::ffi::OsString {
    std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_owned())).unwrap_or_else(|| ASSET.into())
}

/// Downloads and verifies `rel`'s exe into a fresh staging folder (under the running exe's own name); returns it.
pub fn download(rel: &Release) -> Result<PathBuf, String> {
    let dir = work_dir();
    let stage = dir.join(format!("stage-{}", rel.tag));
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage).map_err(|e| format!("temp folder: {e}"))?;
    let exe = stage.join(own_name());
    let sha = dir.join(format!("{}.sha256", rel.tag));
    fetch(&rel.sha_url, &sha, 60)?;
    fetch(&rel.exe_url, &exe, 900)?;
    let want = std::fs::read_to_string(&sha).ok().as_deref().and_then(parse_sha_file).ok_or("the release's checksum file is unreadable")?;
    let bytes = std::fs::read(&exe).map_err(|e| format!("downloaded exe: {e}"))?;
    let got = nf_formats::sha256::hex(&nf_formats::sha256::digest(&bytes));
    if got != want {
        let _ = std::fs::remove_dir_all(&stage);
        return Err(format!("the download is damaged (checksum {got}, expected {want})"));
    }
    Ok(stage)
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { stack.push(p); } else if let Ok(rel) = p.strip_prefix(root) { out.push(rel.to_path_buf()); }
        }
    }
    out.sort();
    out
}

fn old_name(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(".old");
    PathBuf::from(s)
}

/// Swaps the staged files into `dest` (the launcher's folder): each existing file is renamed to `.old` first, so a
/// running launcher can be replaced. On any failure the renamed files are put back.
pub fn install(stage: &Path, dest: &Path) -> Result<(), String> {
    let files = files_under(stage);
    let mut moved: Vec<PathBuf> = Vec::new();
    let mut placed: Vec<PathBuf> = Vec::new();
    let result = (|| {
        for rel in &files {
            let target = dest.join(rel);
            if let Some(parent) = target.parent() { std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?; }
            if target.exists() {
                let old = old_name(&target);
                let _ = std::fs::remove_file(&old);
                std::fs::rename(&target, &old).map_err(|e| format!("can't replace {} ({e}) — is the folder read-only?", target.display()))?;
                moved.push(target.clone());
            }
            std::fs::copy(stage.join(rel), &target).map_err(|e| format!("can't write {}: {e}", target.display()))?;
            placed.push(target);
        }
        Ok(())
    })();
    if result.is_err() {
        for p in &placed { if !moved.contains(p) { let _ = std::fs::remove_file(p); } }
        for p in &moved { let _ = std::fs::remove_file(p); let _ = std::fs::rename(old_name(p), p); }
    }
    result
}

/// The files an update can have renamed to `.old`: this exe (under its own name) and what the zip releases
/// (v0.1.0–v0.1.3) held.
const ZIP_ERA_FILES: [&str; 5] = ["NightfirePort.exe", "NightfirePort-game.exe", "README.txt", "THIRD_PARTY_NOTICES.txt", "THIRD_PARTY_RERAC_LICENSE.txt"];

/// Deletes the `.old` files a previous update left beside the launcher (they were still running then). Only those
/// exact names: a lone exe may sit in Downloads or on the desktop, beside the player's own files.
pub fn cleanup_old(dir: &Path) {
    let own = own_name();
    for name in ZIP_ERA_FILES.iter().map(std::ffi::OsString::from).chain(std::iter::once(own)) {
        let _ = std::fs::remove_file(old_name(&dir.join(name)));
    }
}

/// The licence notices baked into this exe (build.rs), plus the in-tree rerac code's notice.
pub const NOTICES: &str = include_str!(concat!(env!("OUT_DIR"), "/notices.txt"));
const RERAC_NOTICE: &str = include_str!("../../../../THIRD_PARTY_RERAC_LICENSE.txt");

/// Writes the licence notices to `<config dir>/THIRD_PARTY_NOTICES.txt` (a release is just the exe; the notices
/// travel inside it). Rewritten only when they changed.
pub fn write_notices() {
    let Some(dir) = crate::settings::config_dir() else { return };
    let text = format!("{NOTICES}\n\n==== rerac (in-tree code) ====\n\n{RERAC_NOTICE}");
    let path = dir.join("THIRD_PARTY_NOTICES.txt");
    if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(&path, text);
    }
}

/// The launcher's own folder.
pub fn install_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The bridge release's shape: the exe + its sha, and the old launchers' zip + sha.
    const LATEST: &str = r#"{"url":"https://api.github.com/repos/YodaMMM/NightfirePort/releases/1","tag_name":"v0.1.12","name":"NightfirePort 0.1.12",
        "assets":[{"name":"NightfirePort-v0.1.12-windows.zip.sha256","browser_download_url":"https:\/\/github.com\/YodaMMM\/NightfirePort\/releases\/download\/v0.1.12\/NightfirePort-v0.1.12-windows.zip.sha256"},
                  {"name":"NightfirePort.exe.sha256","browser_download_url":"https:\/\/github.com\/YodaMMM\/NightfirePort\/releases\/download\/v0.1.12\/NightfirePort.exe.sha256"},
                  {"name":"NightfirePort.exe","browser_download_url":"https://github.com/YodaMMM/NightfirePort/releases/download/v0.1.12/NightfirePort.exe"},
                  {"name":"NightfirePort-v0.1.12-windows.zip","browser_download_url":"https://github.com/YodaMMM/NightfirePort/releases/download/v0.1.12/NightfirePort-v0.1.12-windows.zip"}]}"#;

    #[test]
    fn reads_the_latest_release() {
        let r = parse_latest(LATEST).expect("parses");
        assert_eq!(r.tag, "v0.1.12");
        assert_eq!(r.version, (0, 1, 12));
        assert!(r.exe_url.ends_with("/v0.1.12/NightfirePort.exe"), "{}", r.exe_url);
        assert!(r.sha_url.ends_with("/v0.1.12/NightfirePort.exe.sha256") && r.sha_url.starts_with("https://github.com/YodaMMM/"), "{}", r.sha_url);
        // Sister: a release without the exe is not an update (an old zip-only release included).
        assert!(parse_latest(r#"{"tag_name":"v0.2.0","assets":[]}"#).is_none());
        assert!(parse_latest(r#"{"tag_name":"v0.2.0","assets":[{"browser_download_url":"https://x/NightfirePort-v0.2.0-windows.zip"},{"browser_download_url":"https://x/NightfirePort-v0.2.0-windows.zip.sha256"}]}"#).is_none());
    }

    #[test]
    fn versions_compare_numerically() {
        assert_eq!(parse_version("v0.1.12"), Some((0, 1, 12)));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert!(parse_version("v0.1.12").unwrap() > parse_version("v0.1.9").unwrap(), "12 > 9, not text order");
        assert_eq!(parse_version("nightly"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
    }

    #[test]
    fn reads_checksum_files() {
        let h = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(parse_sha_file(&format!("{h}  NightfirePort-v0.1.12-windows.zip\n")).as_deref(), Some(h));
        assert_eq!(parse_sha_file(&h.to_uppercase()).as_deref(), Some(h));
        assert_eq!(parse_sha_file("not a hash"), None);
    }

    /// The swap: files land, the replaced ones become `.old`, and the cleanup removes those.
    #[test]
    fn install_swaps_files_and_cleans_up() {
        let base = std::env::temp_dir().join(format!("nf-update-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (stage, dest) = (base.join("stage"), base.join("dest"));
        std::fs::create_dir_all(&stage).unwrap();
        std::fs::create_dir_all(&dest).unwrap();
        std::fs::write(stage.join("NightfirePort.exe"), b"new launcher").unwrap();
        std::fs::write(dest.join("NightfirePort.exe"), b"old launcher").unwrap();
        std::fs::write(dest.join("README.txt"), b"zip-era readme").unwrap();
        std::fs::write(dest.join("README.txt.old"), b"zip-era leftover").unwrap();
        // The player's own files beside a lone exe (e.g. Downloads) are never touched.
        std::fs::write(dest.join("backup.old"), b"player's file").unwrap();
        std::fs::create_dir_all(dest.join("sub")).unwrap();
        std::fs::write(dest.join("sub").join("NightfirePort.exe.old"), b"player's file").unwrap();
        install(&stage, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("NightfirePort.exe")).unwrap(), b"new launcher");
        assert_eq!(std::fs::read(dest.join("NightfirePort.exe.old")).unwrap(), b"old launcher");
        cleanup_old(&dest);
        assert!(!dest.join("NightfirePort.exe.old").exists());
        assert!(!dest.join("README.txt.old").exists());
        assert!(dest.join("README.txt").exists(), "only .old files go");
        assert!(dest.join("backup.old").exists() && dest.join("sub").join("NightfirePort.exe.old").exists(),
            "sister: other .old files and subfolders are left alone");
        let _ = std::fs::remove_dir_all(&base);
    }
}
