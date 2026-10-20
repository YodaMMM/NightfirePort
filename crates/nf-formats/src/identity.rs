//! Is this disc image the build the port targets? A quick identity check for the launcher (and anything else that
//! takes a user-picked image): 007: Nightfire, NTSC-U, SLUS-20579, version 1.00 (docs/build-identity.md).
//!
//! The check reads a few MB, never the whole 2.4 GB image:
//! - the PVD volume id is `NIGHTFIRE` and the image holds every sector the PVD declares (not truncated);
//! - `SYSTEM.CNF` boots `SLUS_205.79` with `VMODE = NTSC`;
//! - the boot ELF and `ACTION.ELF` hash to the build's SHA-1s (*measured*, build-identity.md "Executables").
//!
//! The whole-image SHA-1 (`03717059…`) is not checked here: hashing 2.4 GB takes tens of seconds and a mismatch
//! outside the executables (a re-ripped image with different padding) does not stop the port from reading it.
//!
//! [`Expected`] carries the identity as data so the tests can build a synthetic image that passes; the real values
//! are [`NIGHTFIRE_NTSC_U`].

use crate::disc::{ACTION_ELF_SHA1, BOOT_ELF_SHA1};
use crate::iso9660::IsoImage;
use crate::sha1::{hex, sha1};
use std::fmt;
use std::io::{Read, Seek};
use std::path::Path;

/// The identity a disc image must carry.
#[derive(Clone, Copy, Debug)]
pub struct Expected<'a> {
    pub volume_id: &'a str,
    /// The `BOOT2` file name in `SYSTEM.CNF` (without `cdrom0:\` and `;1`).
    pub boot_file: &'a str,
    pub vmode: &'a str,
    pub boot_sha1: &'a str,
    /// A second executable that must match (path, SHA-1).
    pub exe: (&'a str, &'a str),
    /// What to call the expected disc in messages.
    pub title: &'a str,
}

/// 007: Nightfire (USA), NTSC-U SLUS-20579 v1.00 [*measured*, docs/build-identity.md].
pub const NIGHTFIRE_NTSC_U: Expected<'static> = Expected {
    volume_id: "NIGHTFIRE",
    boot_file: "SLUS_205.79",
    vmode: "NTSC",
    boot_sha1: BOOT_ELF_SHA1,
    exe: ("/ACTION.ELF", ACTION_ELF_SHA1),
    title: "007: Nightfire (USA, SLUS-20579)",
};

/// What the check found on an image that passed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscInfo {
    pub volume_id: String,
    /// `SLUS-20579` style (the boot file's name with the underscore and dot removed).
    pub serial: String,
    pub vmode: String,
    /// `SYSTEM.CNF` `VER` (empty when absent).
    pub version: String,
    pub sectors: u32,
}

/// Why an image was rejected. `Display` is the friendly sentence the launcher shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityError {
    /// The path doesn't exist or can't be opened.
    Unreadable(String),
    /// Not an ISO 9660 image (or a damaged one).
    NotDiscImage(String),
    /// A PS2/ISO image of something else.
    WrongGame { volume_id: String },
    /// The right title, another release (serial).
    WrongRelease { boot_file: String },
    /// The right serial, another video mode.
    WrongRegion { vmode: String },
    /// Fewer sectors than the volume declares (an incomplete download/copy).
    Truncated { have: u32, want: u32 },
    /// A file the build needs is absent.
    Missing(String),
    /// A file differs from the build (a patched / modified image).
    Modified(String),
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(e) => write!(f, "The file can't be opened ({e})."),
            Self::NotDiscImage(_) => write!(f, "That file isn't a disc image. Choose the .iso of your 007: Nightfire disc."),
            Self::WrongGame { volume_id } if volume_id.is_empty() => write!(f, "That disc image isn't 007: Nightfire."),
            Self::WrongGame { volume_id } => write!(f, "That disc image is \"{volume_id}\", not 007: Nightfire."),
            Self::WrongRelease { boot_file } => write!(f, "This is a different release ({}). The port needs the USA disc, SLUS-20579.", serial_of(boot_file)),
            Self::WrongRegion { vmode } => write!(f, "This disc is {vmode}. The port needs the NTSC-U (USA) disc, SLUS-20579."),
            Self::Truncated { have, want } => write!(f, "The image is incomplete ({:.0}% of the disc). Copy or rip it again.", *have as f64 * 100.0 / (*want).max(1) as f64),
            Self::Missing(p) => write!(f, "The image is missing {p}. It may be damaged; rip it again."),
            Self::Modified(p) => write!(f, "{p} on this image differs from the retail disc (patched or damaged). The port needs an unmodified SLUS-20579 image."),
        }
    }
}

impl std::error::Error for IdentityError {}

/// `SLUS_205.79` → `SLUS-20579`.
pub fn serial_of(boot_file: &str) -> String {
    boot_file.replacen('_', "-", 1).replace('.', "")
}

/// The `KEY = value` pairs of a `SYSTEM.CNF`, keys uppercased.
fn cnf_value<'t>(cnf: &'t str, key: &str) -> Option<&'t str> {
    cnf.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        k.trim().eq_ignore_ascii_case(key).then(|| v.trim())
    })
}

/// `cdrom0:\SLUS_205.79;1` → `SLUS_205.79`.
fn boot_name(boot2: &str) -> String {
    let s = boot2.rsplit(['\\', '/', ':']).next().unwrap_or(boot2);
    s.split(';').next().unwrap_or(s).trim().to_ascii_uppercase()
}

/// Checks an open image against `exp`.
pub fn check<R: Read + Seek>(iso: &IsoImage<R>, exp: &Expected) -> Result<DiscInfo, IdentityError> {
    let volume_id = iso.volume_id().trim().to_string();
    if !volume_id.eq_ignore_ascii_case(exp.volume_id) {
        return Err(IdentityError::WrongGame { volume_id });
    }
    let cnf_e = iso.find("/SYSTEM.CNF").ok_or_else(|| IdentityError::Missing("SYSTEM.CNF".into()))?;
    let cnf = iso.read_file(cnf_e).map_err(|e| IdentityError::NotDiscImage(e.to_string()))?;
    let cnf = String::from_utf8_lossy(&cnf);
    let boot = cnf_value(&cnf, "BOOT2").map(boot_name).unwrap_or_default();
    if boot != exp.boot_file {
        return Err(IdentityError::WrongRelease { boot_file: boot });
    }
    let vmode = cnf_value(&cnf, "VMODE").unwrap_or("").to_string();
    if !vmode.eq_ignore_ascii_case(exp.vmode) {
        return Err(IdentityError::WrongRegion { vmode });
    }
    let version = cnf_value(&cnf, "VER").unwrap_or("").to_string();
    // The volume's declared size before reading files near its end: a truncated copy fails here, not as an I/O error.
    if iso.sector_count() < iso.volume_sectors() {
        return Err(IdentityError::Truncated { have: iso.sector_count(), want: iso.volume_sectors() });
    }
    for (path, want) in [(format!("/{}", exp.boot_file), exp.boot_sha1), (exp.exe.0.to_string(), exp.exe.1)] {
        let name = path.trim_start_matches('/').to_string();
        let e = iso.find(&path).ok_or_else(|| IdentityError::Missing(name.clone()))?;
        let bytes = iso.read_file(e).map_err(|_| IdentityError::Modified(name.clone()))?;
        if hex(&sha1(&bytes)) != want.to_ascii_lowercase() {
            return Err(IdentityError::Modified(name));
        }
    }
    Ok(DiscInfo { volume_id, serial: serial_of(&boot), vmode, version, sectors: iso.sector_count() })
}

/// Opens `path` and checks it. A missing file and a non-ISO file get their own errors.
pub fn check_path(path: &Path, exp: &Expected) -> Result<DiscInfo, IdentityError> {
    if !path.is_file() {
        return Err(IdentityError::Unreadable(if path.exists() { "not a file".into() } else { "file not found".into() }));
    }
    let iso = IsoImage::open(path).map_err(|e| match e {
        crate::iso9660::DiscError::Io(io) => IdentityError::Unreadable(io.to_string()),
        other => IdentityError::NotDiscImage(other.to_string()),
    })?;
    check(&iso, exp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iso9660::SECTOR_SIZE;
    use std::io::Cursor;

    fn both32(b: &mut [u8], o: usize, v: u32) {
        b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        b[o + 4..o + 8].copy_from_slice(&v.to_be_bytes());
    }
    fn rec(lba: u32, size: u32, dir: bool, name: &[u8]) -> Vec<u8> {
        let len = (33 + name.len() + 1) & !1;
        let mut r = vec![0u8; len];
        r[0] = len as u8;
        both32(&mut r, 2, lba);
        both32(&mut r, 10, size);
        r[25] = if dir { 2 } else { 0 };
        r[32] = name.len() as u8;
        r[33..33 + name.len()].copy_from_slice(name);
        r
    }

    /// A synthetic image: volume `vol`, root = SYSTEM.CNF (`cnf`), the boot file and ACTION.ELF (`action`).
    /// `declared` = the PVD volume size (sectors); the buffer holds 40 sectors.
    struct Img { vol: &'static str, cnf: String, boot_name: &'static str, boot: Vec<u8>, action: Option<Vec<u8>>, declared: u32 }

    fn good() -> Img {
        Img {
            vol: "NIGHTFIRE",
            cnf: "BOOT2 = cdrom0:\\SLUS_205.79;1\r\nVER = 1.00\r\nVMODE = NTSC\r\n".into(),
            boot_name: "SLUS_205.79",
            boot: b"\x7fELF boot loader stand-in".to_vec(),
            action: Some(b"\x7fELF action stand-in".to_vec()),
            declared: 40,
        }
    }

    fn build(i: &Img) -> Vec<u8> {
        let mut img = vec![0u8; 40 * SECTOR_SIZE];
        let pvd = &mut img[16 * SECTOR_SIZE..17 * SECTOR_SIZE];
        pvd[0] = 1;
        pvd[1..6].copy_from_slice(b"CD001");
        pvd[6] = 1;
        pvd[8..40].copy_from_slice(format!("{:<32}", "PLAYSTATION").as_bytes());
        pvd[40..72].copy_from_slice(format!("{:<32}", i.vol).as_bytes());
        both32(pvd, 80, i.declared);
        pvd[128..130].copy_from_slice(&(SECTOR_SIZE as u16).to_le_bytes());
        pvd[156..156 + 34].copy_from_slice(&rec(20, SECTOR_SIZE as u32, true, &[0]));
        let mut root = Vec::new();
        root.extend(rec(20, SECTOR_SIZE as u32, true, &[0]));
        root.extend(rec(20, SECTOR_SIZE as u32, true, &[1]));
        if let Some(a) = &i.action { root.extend(rec(25, a.len() as u32, false, b"ACTION.ELF;1")); }
        root.extend(rec(23, i.boot.len() as u32, false, format!("{};1", i.boot_name).as_bytes()));
        root.extend(rec(22, i.cnf.len() as u32, false, b"SYSTEM.CNF;1"));
        img[20 * SECTOR_SIZE..20 * SECTOR_SIZE + root.len()].copy_from_slice(&root);
        img[22 * SECTOR_SIZE..22 * SECTOR_SIZE + i.cnf.len()].copy_from_slice(i.cnf.as_bytes());
        img[23 * SECTOR_SIZE..23 * SECTOR_SIZE + i.boot.len()].copy_from_slice(&i.boot);
        if let Some(a) = &i.action { img[25 * SECTOR_SIZE..25 * SECTOR_SIZE + a.len()].copy_from_slice(a); }
        img
    }

    /// The expected identity for the synthetic image `good()` (its own executables' SHA-1s).
    fn expected_for(i: &Img) -> (String, String) {
        (hex(&sha1(&i.boot)), hex(&sha1(i.action.as_deref().unwrap_or(b""))))
    }

    fn run(i: &Img, hashes: &(String, String)) -> Result<DiscInfo, IdentityError> {
        let exp = Expected { boot_sha1: &hashes.0, exe: ("/ACTION.ELF", &hashes.1), ..NIGHTFIRE_NTSC_U };
        let iso = IsoImage::new(Cursor::new(build(i))).map_err(|e| IdentityError::NotDiscImage(e.to_string()))?;
        check(&iso, &exp)
    }

    #[test]
    fn accepts_the_expected_identity() {
        let g = good();
        let h = expected_for(&g);
        let info = run(&g, &h).expect("the synthetic SLUS-20579 image passes");
        assert_eq!(info, DiscInfo { volume_id: "NIGHTFIRE".into(), serial: "SLUS-20579".into(), vmode: "NTSC".into(), version: "1.00".into(), sectors: 40 });
    }

    #[test]
    fn rejects_each_wrong_field() {
        let h = expected_for(&good());
        let mut i = good(); i.vol = "RATCHETANDCLANK";
        assert_eq!(run(&i, &h), Err(IdentityError::WrongGame { volume_id: "RATCHETANDCLANK".into() }));
        // Same title, PAL release (SLES-51258 is the PAL serial; any other boot name is enough here).
        let mut i = good(); i.cnf = "BOOT2 = cdrom0:\\SLES_512.58;1\r\nVER = 1.00\r\nVMODE = PAL\r\n".into(); i.boot_name = "SLES_512.58";
        assert_eq!(run(&i, &h), Err(IdentityError::WrongRelease { boot_file: "SLES_512.58".into() }));
        let mut i = good(); i.cnf = i.cnf.replace("NTSC", "PAL");
        assert_eq!(run(&i, &h), Err(IdentityError::WrongRegion { vmode: "PAL".into() }));
        let mut i = good(); i.declared = 80;
        assert_eq!(run(&i, &h), Err(IdentityError::Truncated { have: 40, want: 80 }));
        let mut i = good(); i.boot[5] ^= 1;
        assert_eq!(run(&i, &h), Err(IdentityError::Modified("SLUS_205.79".into())));
        let mut i = good(); i.action.as_mut().unwrap()[0] ^= 1;
        assert_eq!(run(&i, &h), Err(IdentityError::Modified("ACTION.ELF".into())));
        let mut i = good(); i.action = None;
        assert_eq!(run(&i, &h), Err(IdentityError::Missing("ACTION.ELF".into())));
    }

    #[test]
    fn rejects_missing_and_non_iso_files() {
        let dir = std::env::temp_dir().join(format!("nf-identity-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let missing = dir.join("nope.iso");
        assert!(matches!(check_path(&missing, &NIGHTFIRE_NTSC_U), Err(IdentityError::Unreadable(_))));
        let junk = dir.join("junk.iso");
        std::fs::write(&junk, vec![0x5au8; 64 * 1024]).unwrap();
        assert!(matches!(check_path(&junk, &NIGHTFIRE_NTSC_U), Err(IdentityError::NotDiscImage(_))));
        let tiny = dir.join("tiny.iso");
        std::fs::write(&tiny, b"hello").unwrap();
        assert!(matches!(check_path(&tiny, &NIGHTFIRE_NTSC_U), Err(IdentityError::NotDiscImage(_))));
        // The synthetic image passes its own hashes but not the real build's (sister: the real SHA-1s are checked).
        let g = good();
        let ok = dir.join("synthetic.iso");
        std::fs::write(&ok, build(&g)).unwrap();
        assert_eq!(check_path(&ok, &NIGHTFIRE_NTSC_U), Err(IdentityError::Modified("SLUS_205.79".into())));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn messages_are_friendly() {
        assert_eq!(serial_of("SLUS_205.79"), "SLUS-20579");
        assert_eq!(boot_name("cdrom0:\\SLUS_205.79;1"), "SLUS_205.79");
        let m = IdentityError::WrongRegion { vmode: "PAL".into() }.to_string();
        assert!(m.contains("SLUS-20579") && m.contains("PAL"), "{m}");
        let m = IdentityError::Truncated { have: 50, want: 100 }.to_string();
        assert!(m.contains("50%"), "{m}");
    }
}
