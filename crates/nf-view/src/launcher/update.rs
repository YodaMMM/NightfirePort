//! The launcher's self-update (user, 2026-10-19: "a github release and a way to auto update when we push fixes";
//! updates install automatically on launch).
//!
//! A release build carries its version (`NF_RELEASE_VERSION`, baked in by the release build — `tools/release`); a dev
//! build carries none and never updates. On start the launcher asks GitHub for the repository's latest release
//! (`releases/latest`); when its tag is newer it downloads the release's Windows zip and the `.sha256` beside it,
//! checks the digest ([`nf_formats::sha256`] — a corruption/truncation check; both files come over HTTPS from the
//! same release), unpacks the zip into a staging folder and swaps the files in. Windows lets a running program be
//! renamed but not overwritten, so each replaced file is first renamed to `<name>.old` (deleted on the next start),
//! then the new launcher is started and this one exits. Any failure (offline, GitHub down, no write access to the
//! folder) leaves the installed version untouched and the game playable.
//!
//! No new crates: the transfers use the system `curl.exe` and the zip the system `tar.exe` (bsdtar), both part of
//! Windows 10 1803+ and Windows 11. `NF_UPDATE_API` overrides the release query URL (tests point it at a local
//! `file://` JSON).

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
    pub zip_url: String,
    pub sha_url: String,
}

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

/// Reads the `releases/latest` answer: the tag, and the Windows zip + its `.sha256` among the assets.
pub fn parse_latest(json: &str) -> Option<Release> {
    let tag = json_strings(json, "tag_name").into_iter().next()?;
    let version = parse_version(&tag)?;
    let urls = json_strings(json, "browser_download_url");
    let zip_url = urls.iter().find(|u| u.ends_with("-windows.zip"))?.clone();
    let sha_url = urls.iter().find(|u| u.ends_with("-windows.zip.sha256"))?.clone();
    Some(Release { tag, version, zip_url, sha_url })
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

/// Downloads, verifies and unpacks `rel` into a fresh staging folder; returns it.
pub fn download(rel: &Release) -> Result<PathBuf, String> {
    let dir = work_dir();
    let stage = dir.join(format!("stage-{}", rel.tag));
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage).map_err(|e| format!("temp folder: {e}"))?;
    let zip = dir.join(format!("{}.zip", rel.tag));
    let sha = dir.join(format!("{}.zip.sha256", rel.tag));
    fetch(&rel.sha_url, &sha, 60)?;
    fetch(&rel.zip_url, &zip, 900)?;
    let want = std::fs::read_to_string(&sha).ok().as_deref().and_then(parse_sha_file).ok_or("the release's checksum file is unreadable")?;
    let bytes = std::fs::read(&zip).map_err(|e| format!("downloaded zip: {e}"))?;
    let got = nf_formats::sha256::hex(&nf_formats::sha256::digest(&bytes));
    if got != want { return Err(format!("the download is damaged (checksum {got}, expected {want})")); }
    let o = quiet(Command::new(system_tool("tar.exe")).arg("-xf").arg(&zip).arg("-C").arg(&stage)).output()
        .map_err(|e| format!("couldn't run tar: {e}"))?;
    if !o.status.success() { return Err(format!("unpacking failed: {}", String::from_utf8_lossy(&o.stderr).trim())); }
    if !stage.join(if cfg!(windows) { "NightfirePort.exe" } else { "NightfirePort" }).exists() {
        return Err("the release zip has no launcher in it".into());
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

/// Deletes the `.old` files a previous update left beside the launcher (they were still running then).
pub fn cleanup_old(dir: &Path) {
    for rel in files_under(dir) {
        if rel.extension().is_some_and(|e| e == "old") { let _ = std::fs::remove_file(dir.join(rel)); }
    }
}

/// The launcher's own folder.
pub fn install_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LATEST: &str = r#"{"url":"https://api.github.com/repos/YodaMMM/NightfirePort/releases/1","tag_name":"v0.1.12","name":"NightfirePort 0.1.12",
        "assets":[{"name":"NightfirePort-v0.1.12-windows.zip.sha256","browser_download_url":"https:\/\/github.com\/YodaMMM\/NightfirePort\/releases\/download\/v0.1.12\/NightfirePort-v0.1.12-windows.zip.sha256"},
                  {"name":"NightfirePort-v0.1.12-windows.zip","browser_download_url":"https://github.com/YodaMMM/NightfirePort/releases/download/v0.1.12/NightfirePort-v0.1.12-windows.zip"}]}"#;

    #[test]
    fn reads_the_latest_release() {
        let r = parse_latest(LATEST).expect("parses");
        assert_eq!(r.tag, "v0.1.12");
        assert_eq!(r.version, (0, 1, 12));
        assert!(r.zip_url.ends_with("/v0.1.12/NightfirePort-v0.1.12-windows.zip"), "{}", r.zip_url);
        assert!(r.sha_url.ends_with("-windows.zip.sha256") && r.sha_url.starts_with("https://github.com/YodaMMM/"), "{}", r.sha_url);
        // Sister: a release without the Windows zip is not an update.
        assert!(parse_latest(r#"{"tag_name":"v0.2.0","assets":[]}"#).is_none());
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
        std::fs::write(stage.join("nf-view.exe"), b"new game").unwrap();
        std::fs::write(dest.join("NightfirePort.exe"), b"old launcher").unwrap();
        install(&stage, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("NightfirePort.exe")).unwrap(), b"new launcher");
        assert_eq!(std::fs::read(dest.join("nf-view.exe")).unwrap(), b"new game");
        assert_eq!(std::fs::read(dest.join("NightfirePort.exe.old")).unwrap(), b"old launcher");
        cleanup_old(&dest);
        assert!(!dest.join("NightfirePort.exe.old").exists());
        let _ = std::fs::remove_dir_all(&base);
    }
}
