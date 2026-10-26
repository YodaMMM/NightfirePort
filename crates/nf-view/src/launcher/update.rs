//! The launcher's self-update (user, 2026-10-19: "a github release and a way to auto update when we push fixes";
//! updates install automatically on launch).
//!
//! A release build carries its version (`NF_RELEASE_VERSION`, baked in by the release build — `tools/release`); a dev
//! build carries none and never updates. On start the launcher asks GitHub for the repository's release list
//! (`releases`, prereleases included — `releases/latest` skips them; it stays as the fallback) and takes the newest
//! version; when its tag is newer it downloads the release's `NightfirePort.exe` and the `.sha256` beside
//! it (a release is just the exe — no zip), checks the digest ([`nf_formats::sha256`] — a
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

/// The top-level objects of a JSON array, as slices (string-aware brace counting).
fn json_array_objects(json: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start, mut in_str, mut esc) = (Vec::new(), 0i32, None, false, false);
    for (i, c) in json.char_indices() {
        if in_str {
            if esc { esc = false } else if c == '\\' { esc = true } else if c == '"' { in_str = false }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' => { if depth == 0 { start = Some(i); } depth += 1; }
            '}' => { depth -= 1; if depth == 0 { if let Some(st) = start.take() { out.push(&json[st..=i]); } } }
            _ => {}
        }
    }
    out
}

/// Reads the `releases` list answer: the newest release (by version, prereleases included) that
/// carries the Windows exe and its `.sha256`. `releases/latest` skips prereleases, so a launcher that
/// only asked for it would never see a release published as a prerelease.
pub fn parse_newest(json: &str) -> Option<Release> {
    json_array_objects(json).into_iter()
        .filter(|o| !o.contains("\"draft\":true") && !o.contains("\"draft\": true"))
        .filter_map(parse_latest)
        .max_by(|a, b| a.version.cmp(&b.version))
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
    // The release LIST (prereleases included), newest by version; `releases/latest` as the fallback.
    // `NF_UPDATE_API` may answer either shape (a list or one release).
    let url = std::env::var("NF_UPDATE_API").unwrap_or_else(|_| format!("https://api.github.com/repos/{REPO}/releases?per_page=20"));
    let dir = work_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("temp folder: {e}"))?;
    let json_path = dir.join("latest.json");
    let listed = fetch(&url, &json_path, 15).ok()
        .and_then(|_| std::fs::read_to_string(&json_path).ok())
        .and_then(|j| if j.trim_start().starts_with('[') { parse_newest(&j) } else { parse_latest(&j) });
    let rel = match listed {
        Some(r) => r,
        None => {
            fetch(&format!("https://api.github.com/repos/{REPO}/releases/latest"), &json_path, 15)?;
            let json = std::fs::read_to_string(&json_path).map_err(|e| format!("release info: {e}"))?;
            parse_latest(&json).ok_or("the latest release has no Windows download")?
        }
    };
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
const OPENGOAL_NOTICE: &str = include_str!("../../../../THIRD_PARTY_OPENGOAL_LICENSE.txt");

/// The in-tree third-party licences the exe carries beside the baked file's text. Every
/// `THIRD_PARTY_*_LICENSE.txt` at the repo root must appear here — the notices test walks the
/// root and requires each file's text in [`notices_text`] (a new root licence file without a
/// wire-in fails there).
const IN_TREE_NOTICES: [(&str, &str); 2] = [
    ("rerac (in-tree code)", RERAC_NOTICE),
    ("OpenGOAL (in-tree port)", OPENGOAL_NOTICE),
];

/// The full notices text `write_notices` writes: what the build baked (build.rs — the
/// release-generated `THIRD_PARTY_NOTICES.txt` from tools/release/notices.py, or the dev
/// placeholder) plus every in-tree licence file.
pub fn notices_text() -> String {
    let mut text = String::from(NOTICES);
    for (name, body) in IN_TREE_NOTICES {
        text.push_str(&format!("\n\n==== {name} ====\n\n{body}"));
    }
    text
}

/// Writes the licence notices to `<config dir>/THIRD_PARTY_NOTICES.txt` (a release is just the exe; the notices
/// travel inside it). Rewritten only when they changed.
pub fn write_notices() {
    let Some(dir) = crate::settings::config_dir() else { return };
    let text = notices_text();
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

    /// Every `THIRD_PARTY_*_LICENSE.txt` at the repo root rides in the notices the exe carries: the
    /// build bakes the generated file's text (build.rs via NF_NOTICES_FILE) and [`notices_text`]
    /// appends each in-tree licence, so the full text lands in `<config dir>/THIRD_PARTY_NOTICES.txt`.
    /// A new root licence file without a wire-in fails here.
    #[test]
    fn every_root_third_party_licence_rides_in_the_notices() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        let text = notices_text();
        let mut checked = 0;
        for entry in std::fs::read_dir(&root).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            if !(name.starts_with("THIRD_PARTY_") && name.ends_with("_LICENSE.txt")) {
                continue;
            }
            let body = std::fs::read_to_string(entry.path()).unwrap();
            assert!(text.contains(&body), "{name} is missing from the exe's notices");
            checked += 1;
        }
        assert!(
            checked >= 2,
            "expected the rerac and OpenGOAL licence files at the repo root, found {checked}"
        );
    }

    // The bridge release's shape: the exe + its sha, and the old launchers' zip + sha.
    const LATEST: &str = r#"{"url":"https://api.github.com/repos/YodaMMM/NightfirePort/releases/1","tag_name":"v0.1.12","name":"NightfirePort 0.1.12",
        "assets":[{"name":"NightfirePort-v0.1.12-windows.zip.sha256","browser_download_url":"https:\/\/github.com\/YodaMMM\/NightfirePort\/releases\/download\/v0.1.12\/NightfirePort-v0.1.12-windows.zip.sha256"},
                  {"name":"NightfirePort.exe.sha256","browser_download_url":"https:\/\/github.com\/YodaMMM\/NightfirePort\/releases\/download\/v0.1.12\/NightfirePort.exe.sha256"},
                  {"name":"NightfirePort.exe","browser_download_url":"https://github.com/YodaMMM/NightfirePort/releases/download/v0.1.12/NightfirePort.exe"},
                  {"name":"NightfirePort-v0.1.12-windows.zip","browser_download_url":"https://github.com/YodaMMM/NightfirePort/releases/download/v0.1.12/NightfirePort-v0.1.12-windows.zip"}]}"#;

    /// One release object of the `releases` list (tag, prerelease/draft flags, a body with braces and quotes).
    fn listed(tag: &str, pre: bool, draft: bool) -> String {
        format!(r#"{{"tag_name":"{tag}","name":"Nightfire Rust {tag} alpha","draft":{draft},"prerelease":{pre},"body":"Single player\n- a {{brace}} and a \"quote\"",
            "assets":[{{"name":"NightfirePort.exe","browser_download_url":"https://github.com/YodaMMM/NightfirePort/releases/download/{tag}/NightfirePort.exe"}},
                      {{"name":"NightfirePort.exe.sha256","browser_download_url":"https://github.com/YodaMMM/NightfirePort/releases/download/{tag}/NightfirePort.exe.sha256"}}]}}"#)
    }

    #[test]
    fn the_list_picks_the_newest_release_prereleases_included() {
        // The list order is not trusted: the newest VERSION wins, a prerelease counts, a draft does not.
        let json = format!("[{},{},{},{}]", listed("v0.1.19", false, false), listed("v0.1.21", false, true), listed("v0.1.20", true, false), listed("v0.1.18", true, false));
        let r = parse_newest(&json).expect("parses");
        assert_eq!(r.tag, "v0.1.20");
        assert!(r.exe_url.ends_with("/v0.1.20/NightfirePort.exe") && r.sha_url.ends_with("/v0.1.20/NightfirePort.exe.sha256"), "{} {}", r.exe_url, r.sha_url);
        // Sister: the same list without the prerelease answers the full release (the split is per object).
        let json = format!("[{},{}]", listed("v0.1.18", true, false), listed("v0.1.19", false, false));
        assert_eq!(parse_newest(&json).map(|r| r.tag).as_deref(), Some("v0.1.19"));
        assert!(parse_newest("[]").is_none());
    }

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
