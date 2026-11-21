//! The driving missions' control configs: `data\Control\*.def` inside every `DRIVING/*.VIV`
//! (docs/systems/driving-input.md §13). Plain CRLF text, parsed here the way DRIVING.ELF reads it.
//!
//! Three shapes [D, all 12 distinct files on the disc]:
//!
//! - `Master.def` — the config list: `POV 8` then eight file names, `Driving 1` then one file name,
//!   `End`. Read by `FUN_00166548` 0x166548 [M]: each line is cut at its `'\r'`, the first
//!   `strtok(" \t")` token is compared case-insensitively (`FUN_002a9198` = stricmp [M]) against
//!   `END` (stop), `Driving` (section 0, the count is `atoi` of the next token), `POV` (section 1);
//!   any other first token is copied as a file name into the current section.
//! - binding files (`FrontEnd.def`, `drivecfg.def`, `povcfg0..7.def`, `DbgCfg.def`) — a header line
//!   (`action  button  update method  optional parameter`, skipped), then one binding per line:
//!   `<action name> <value name> [<update method> [<parameter>]]`, ending at a line whose first
//!   token is `FRONTEND` or `END` (stricmp). Read by `FUN_001674d0` 0x1674d0 [M].
//! - an optional `FRONTEND` section after the bindings (the config's name/label ids for the
//!   controller page): `<field> <number>` lines up to `END`. Read by `FUN_00166788` 0x166788 [M].
//!
//! The update-method letters are the `StringToNumber` table at 0x32e5c0 [M .data, names at
//! 0x373cf0..0x373d38]: `U`=0 `+`=1 `-`=2 `>`=3 `<`=4 `=`=5 `I`=6 `N`=7 `R`=8 `M`=9. A missing or
//! unknown letter is type 0 [M 0x1674d0: `lVar6 = 0` unless the lookup returns ≠ −1]. Types 3/4
//! default their parameter to 0.5 (0x3f000000) before the optional `%f` token overrides it; every
//! other type defaults to 0.0 [M 0x1674d0 `(lVar6 < 5) && (2 < lVar6)`]. Table lookups use
//! `FUN_002a9508` = strcmp, case-sensitive [M]; the section keywords use stricmp.
//!
//! The 14 `FRONTEND` field names are the table at 0x32e540 [M .data]: `ButtonCross`,
//! `ButtonTriangle`, `ButtonCircle`, `ButtonSquare`, `DPadLeftRight`, `DPadUpDown`, `ButtonLStick`,
//! `ButtonL1`, `ButtonL2`, `ButtonRStick`, `ButtonR1`, `ButtonR2`, `DESCRIPTION`, `TITLE`, stored as
//! `atoi` values at the config record's `+0x24 + 4·index` [M 0x166788].

use crate::buf::{invalid, Result};

/// The update-method letters in table order (index = binding type) [M 0x32e5c0].
pub const METHOD_LETTERS: [&str; 10] = ["U", "+", "-", ">", "<", "=", "I", "N", "R", "M"];

/// The FRONTEND field names in table order [M 0x32e540].
pub const FRONTEND_FIELDS: [&str; 14] = [
    "ButtonCross", "ButtonTriangle", "ButtonCircle", "ButtonSquare", "DPadLeftRight", "DPadUpDown",
    "ButtonLStick", "ButtonL1", "ButtonL2", "ButtonRStick", "ButtonR1", "ButtonR2", "DESCRIPTION",
    "TITLE",
];

/// One binding line, method and parameter resolved the way `FUN_001674d0` does.
#[derive(Clone, Debug, PartialEq)]
pub struct BindingLine {
    /// The action token (resolved to an id by the game's action table; see nf-game).
    pub action: String,
    /// The value token (one of the 32 InputTable names).
    pub value: String,
    /// The binding type 0..9 (missing/unknown letter → 0).
    pub method: u8,
    /// The method letter as written, if any (kept for diagnostics).
    pub method_text: Option<String>,
    /// The threshold parameter (0.5 default for types 3/4, else 0.0; `%f` overrides).
    pub arg: f32,
}

/// A binding file: its lines plus the optional FRONTEND field block.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlDef {
    pub bindings: Vec<BindingLine>,
    /// `Some` when a FRONTEND section exists: the 14 fields (unset ones stay 0, as the game's
    /// zeroed record [M 0x166548 clears each 0x5c record]).
    pub frontend: Option<[i32; 14]>,
}

/// Master.def: the two config lists [M 0x166548].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MasterDef {
    /// Section 0 (`Driving n`): config files, with the declared count.
    pub driving: Vec<String>,
    pub driving_count: i32,
    /// Section 1 (`POV n`).
    pub pov: Vec<String>,
    pub pov_count: i32,
}

/// The game's line walk: the line is the bytes up to the first `'\r'`; the cursor moves past
/// the next `'\n'` (`FUN_00167498` 0x167498 [M]). Lines missing a `'\r'` would run to the end of
/// the buffer in the game; here they end at the `'\n'` or the end.
fn lines(text: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < text.len() {
        let rest = &text[pos..];
        let nl = rest.iter().position(|&c| c == b'\n');
        let line_end = rest.iter().position(|&c| c == b'\r').or(nl).unwrap_or(rest.len());
        out.push(&rest[..line_end]);
        match nl {
            Some(n) => pos += n + 1,
            None => break,
        }
    }
    out
}

/// `strtok(line, " \t")` tokens [M the 0x373bb8 / 0x373d68 delimiter strings "\t "].
fn tokens(line: &[u8]) -> Vec<String> {
    line.split(|&c| c == b'\t' || c == b' ')
        .filter(|t| !t.is_empty())
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .collect()
}

/// newlib `atoi`: optional sign, leading digits, 0 when none.
fn atoi(s: &str) -> i32 {
    let b = s.as_bytes();
    let (neg, start) = match b.first() {
        Some(b'-') => (true, 1),
        Some(b'+') => (false, 1),
        _ => (false, 0),
    };
    let mut v: i32 = 0;
    for &c in &b[start..] {
        if !c.is_ascii_digit() { break; }
        v = v.wrapping_mul(10).wrapping_add((c - b'0') as i32);
    }
    if neg { v.wrapping_neg() } else { v }
}

/// `sscanf(tok, "%f", &arg)`: the longest leading float literal; `None` leaves the default.
fn scan_f32(s: &str) -> Option<f32> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() || c == '.' || ((c == '-' || c == '+') && i == 0) || c == 'e' || c == 'E' {
            end = i + 1;
        } else {
            break;
        }
    }
    (1..=end).rev().find_map(|n| s[..n].parse::<f32>().ok())
}

pub fn method_type(letter: &str) -> Option<u8> {
    METHOD_LETTERS.iter().position(|&m| m == letter).map(|i| i as u8)
}

impl ControlDef {
    /// Parses a binding file [M 0x1674d0 + 0x166788].
    pub fn parse(text: &[u8]) -> Result<ControlDef> {
        let ls = lines(text);
        if ls.is_empty() { return invalid("empty control def"); }
        let mut bindings = Vec::new();
        let mut i = 1; // the header line is skipped (the loop advances before reading)
        let mut ended_on_frontend = false;
        let mut terminated = false;
        while i < ls.len() {
            let t = tokens(ls[i]);
            i += 1;
            let Some(first) = t.first() else {
                return invalid(format!("blank line {i} inside the binding list (the game's strtok would return NULL)"));
            };
            if first.eq_ignore_ascii_case("FRONTEND") { ended_on_frontend = true; terminated = true; break; }
            if first.eq_ignore_ascii_case("END") { terminated = true; break; }
            let Some(value) = t.get(1) else { return invalid(format!("binding line {i} has no value token")) };
            let method_text = t.get(2).cloned();
            let method = method_text.as_deref().and_then(method_type).unwrap_or(0);
            let mut arg = if method == 3 || method == 4 { 0.5 } else { 0.0 };
            if let Some(a) = t.get(3).and_then(|s| scan_f32(s)) { arg = a; }
            bindings.push(BindingLine { action: first.clone(), value: value.clone(), method, method_text, arg });
        }
        if !terminated { return invalid("binding list has no FRONTEND/END line"); }
        let frontend = if ended_on_frontend {
            let mut f = [0i32; 14];
            loop {
                let Some(l) = ls.get(i) else { return invalid("FRONTEND section has no END") };
                i += 1;
                let t = tokens(l);
                let Some(first) = t.first() else { return invalid("blank line inside the FRONTEND section") };
                if first.eq_ignore_ascii_case("END") { break; }
                let Some(k) = FRONTEND_FIELDS.iter().position(|&n| n == first) else {
                    return invalid(format!("unknown FRONTEND field {first}"));
                };
                f[k] = t.get(1).map(|s| atoi(s)).unwrap_or(0);
            }
            Some(f)
        } else {
            None
        };
        Ok(ControlDef { bindings, frontend })
    }
}

impl MasterDef {
    /// Parses Master.def [M 0x166548].
    pub fn parse(text: &[u8]) -> Result<MasterDef> {
        let mut m = MasterDef { driving: Vec::new(), driving_count: 0, pov: Vec::new(), pov_count: 0 };
        let mut section: Option<u8> = None;
        for l in lines(text) {
            let t = tokens(l);
            let Some(first) = t.first() else { return invalid("blank line in Master.def before End") };
            if first.eq_ignore_ascii_case("END") { return Ok(m); }
            if first.eq_ignore_ascii_case("Driving") {
                section = Some(0);
                m.driving_count = t.get(1).map(|s| atoi(s)).unwrap_or(0);
            } else if first.eq_ignore_ascii_case("POV") {
                section = Some(1);
                m.pov_count = t.get(1).map(|s| atoi(s)).unwrap_or(0);
            } else {
                match section {
                    Some(0) => m.driving.push(first.clone()),
                    Some(1) => m.pov.push(first.clone()),
                    _ => return invalid(format!("file name {first} before any section")),
                }
            }
        }
        invalid("Master.def has no End line")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "action  \t\t\t\t\t\tbutton\t\t\tupdate method\toptional parameter\t  **NOTE**\r\n";

    #[test]
    fn methods_defaults_and_args() {
        let t = format!(
            "{HEADER}GAMEACTION_STEER\t\t\tALX\r\nGAMEACTION_HANDBRAKE\t\t\tAButtonCircle\t\t\t>\t0.1\r\n\
             GAMEACTION_X\tALY\t<\r\nGAMEACTION_Y\tALY\tQ\t0.7\r\nGAMEACTION_Z\tDButtonUp\tR\r\nEND\r\n"
        );
        let d = ControlDef::parse(t.as_bytes()).unwrap();
        assert_eq!(d.bindings.len(), 5);
        assert_eq!((d.bindings[0].method, d.bindings[0].arg), (0, 0.0), "no letter = type 0 'U'");
        assert_eq!((d.bindings[1].method, d.bindings[1].arg), (3, 0.1f32), "'>' with %f 0.1");
        assert_eq!((d.bindings[2].method, d.bindings[2].arg), (4, 0.5), "'<' defaults to 0.5");
        // Unknown letter = type 0, but the %f token still overrides the 0.0 default [M 0x1674d0].
        assert_eq!((d.bindings[3].method, d.bindings[3].arg), (0, 0.7f32));
        assert_eq!(d.bindings[4].method, 8);
        assert_eq!(d.frontend, None);
    }

    #[test]
    fn method_table_order() {
        let got: Vec<u8> = METHOD_LETTERS.iter().map(|l| method_type(l).unwrap()).collect();
        assert_eq!(got, (0..10).collect::<Vec<u8>>());
        assert_eq!(method_type("u"), None, "the table lookup is strcmp — case-sensitive");
    }

    #[test]
    fn frontend_section_and_case_insensitive_keywords() {
        let t = format!("{HEADER}A\tALX\r\nfrontend\t; comment\r\nTITLE\t\t8800\r\nButtonR2\t2712\r\nend\r\n\r\n");
        let d = ControlDef::parse(t.as_bytes()).unwrap();
        assert_eq!(d.bindings.len(), 1);
        let f = d.frontend.unwrap();
        assert_eq!(f[13], 8800);
        assert_eq!(f[11], 2712);
        assert_eq!(f[0], 0);
        // The field names are strcmp-matched: a lower-case field name is not a field.
        let bad = format!("{HEADER}FRONTEND\r\ntitle\t1\r\nEND\r\n");
        assert!(ControlDef::parse(bad.as_bytes()).is_err());
    }

    #[test]
    fn master_def_sections() {
        let t = b"POV 8\r\npovcfg0.def\r\npovcfg1.def\r\nDriving 1\r\ndrivecfg.def\r\nEnd\r\n";
        let m = MasterDef::parse(t).unwrap();
        assert_eq!(m.pov, vec!["povcfg0.def", "povcfg1.def"]);
        assert_eq!((m.pov_count, m.driving_count), (8, 1));
        assert_eq!(m.driving, vec!["drivecfg.def"]);
        assert!(MasterDef::parse(b"POV 1\r\na.def\r\n").is_err(), "no End");
    }

    #[test]
    fn atoi_and_scan() {
        assert_eq!(atoi("2728"), 2728);
        assert_eq!(atoi("12x"), 12);
        assert_eq!(atoi("x"), 0);
        assert_eq!(scan_f32("0.1"), Some(0.1));
        assert_eq!(scan_f32("abc"), None);
    }
}
