//! The launcher's own file: `launcher.toml` beside `settings.toml` in the config dir (`%APPDATA%\NightfirePort`,
//! `NF_CONFIG_DIR` in tests). It holds only the remembered disc image path.
//!
//! A separate file, not a `[launcher]` section of `settings.toml`: the game's settings parser rejects any
//! non-integer value as corrupt (settings.rs `parse`) and `save_opts` rewrites the whole file, so a path stored
//! there would reset the game's options and then be dropped on the next save.
//!
//! Shape (a TOML basic string, so a path with quotes, backslashes or non-ASCII survives):
//! ```toml
//! iso = "C:\\Games\\PS2\\007 - Nightfire (USA).iso"
//! ```

use std::path::{Path, PathBuf};

const FILE_NAME: &str = "launcher.toml";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LauncherConfig {
    /// The disc image PLAY starts the game with.
    pub iso: Option<PathBuf>,
}

/// `<config dir>/launcher.toml`.
pub fn default_path() -> Option<PathBuf> { crate::settings::config_dir().map(|d| d.join(FILE_NAME)) }

/// A TOML basic string: `"` + escapes + `"`.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Reads a TOML basic or literal string value; the rest of the line after it may only be a `#` comment.
fn unquote(v: &str) -> Result<String, String> {
    let v = v.trim();
    if let Some(body) = v.strip_prefix('\'') {
        let end = body.find('\'').ok_or("unterminated '...' string")?;
        let rest = body[end + 1..].trim();
        if !(rest.is_empty() || rest.starts_with('#')) { return Err(format!("text after the string: {rest}")); }
        return Ok(body[..end].to_string());
    }
    let mut it = v.strip_prefix('"').ok_or("expected a quoted string")?.chars();
    let mut out = String::new();
    loop {
        match it.next().ok_or("unterminated \"...\" string")? {
            '"' => break,
            '\\' => match it.next().ok_or("dangling escape")? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{c}'),
                e @ ('u' | 'U') => {
                    let n = if e == 'u' { 4 } else { 8 };
                    let hex: String = (&mut it).take(n).collect();
                    let cp = u32::from_str_radix(&hex, 16).map_err(|_| format!("bad \\{e} escape"))?;
                    out.push(char::from_u32(cp).ok_or("escape is not a character")?);
                }
                e => return Err(format!("unknown escape \\{e}")),
            },
            c => out.push(c),
        }
    }
    let rest: String = it.collect();
    let rest = rest.trim();
    if !(rest.is_empty() || rest.starts_with('#')) { return Err(format!("text after the string: {rest}")); }
    Ok(out)
}

impl LauncherConfig {
    pub fn to_text(&self) -> String {
        let mut s = String::from("# NightfirePort launcher: the disc image PLAY starts the game with.\n");
        if let Some(p) = &self.iso { s.push_str(&format!("iso = {}\n", quote(&p.to_string_lossy()))); }
        s
    }

    /// Unknown keys are ignored (a newer launcher's); a malformed line is an error.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut cfg = LauncherConfig::default();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') { continue; }
            let (k, v) = line.split_once('=').ok_or_else(|| format!("line {}: expected `key = value`", n + 1))?;
            if k.trim() == "iso" {
                let s = unquote(v).map_err(|e| format!("line {}: {e}", n + 1))?;
                cfg.iso = (!s.is_empty()).then(|| PathBuf::from(s));
            }
        }
        Ok(cfg)
    }

    /// Missing or unreadable → the empty config (first run).
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|t| Self::parse(&t).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() { std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?; }
        std::fs::write(path, self.to_text()).map_err(|e| format!("{}: {e}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_awkward_paths() {
        for p in [
            r"C:\Users\Player\Downloads\PS2\007 - Nightfire (USA).iso",
            r#"D:\Bond's "best" games\nightfire.iso"#,
            "/home/q/Spiel – Nachtfeuer ✓/nf.iso",
            "C:\\tab\there\\x.iso",
        ] {
            let cfg = LauncherConfig { iso: Some(PathBuf::from(p)) };
            let text = cfg.to_text();
            assert_eq!(LauncherConfig::parse(&text), Ok(cfg.clone()), "{text}");
        }
        assert_eq!(LauncherConfig::parse(&LauncherConfig::default().to_text()), Ok(LauncherConfig::default()));
    }

    #[test]
    fn parses_hand_edits_and_rejects_damage() {
        let c = LauncherConfig::parse("# hi\n[launcher]\nfuture = 3\niso = 'C:\\a b\\x.iso'   # literal\n").unwrap();
        assert_eq!(c.iso, Some(PathBuf::from(r"C:\a b\x.iso")));
        let c = LauncherConfig::parse("iso = \"C:\\\\\\u00e9.iso\"").unwrap();
        assert_eq!(c.iso, Some(PathBuf::from("C:\\\u{e9}.iso")));
        assert!(LauncherConfig::parse("iso = \"C:\\x.iso").is_err(), "unterminated");
        assert!(LauncherConfig::parse("iso = C:\\x.iso").is_err(), "unquoted");
        assert!(LauncherConfig::parse("iso = \"a\" junk").is_err(), "trailing text");
        assert!(LauncherConfig::parse("just words").is_err());
        // Sister: a different path must not compare equal after a round trip.
        let a = LauncherConfig { iso: Some(PathBuf::from(r"C:\a.iso")) };
        assert_ne!(LauncherConfig::parse(&a.to_text()).unwrap().iso, Some(PathBuf::from(r"C:\b.iso")));
    }

    #[test]
    fn save_and_load_through_a_file() {
        let dir = std::env::temp_dir().join(format!("nf-launcher-cfg-{}", std::process::id()));
        let path = dir.join("sub").join(FILE_NAME);
        assert_eq!(LauncherConfig::load(&path), LauncherConfig::default(), "missing file = first run");
        let cfg = LauncherConfig { iso: Some(PathBuf::from(r"E:\PS2\007 - Nightfire (USA).iso")) };
        cfg.save(&path).unwrap();
        assert_eq!(LauncherConfig::load(&path), cfg);
        std::fs::write(&path, "iso = \"broken").unwrap();
        assert_eq!(LauncherConfig::load(&path), LauncherConfig::default(), "corrupt file = first run");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
