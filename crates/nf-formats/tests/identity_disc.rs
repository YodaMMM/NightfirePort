//! The launcher's disc identity check (`nf_formats::identity`) against the user's own image. Opt-in: `NF_ISO`.
//! Sister: the same image fails against an expected identity with one SHA-1 character changed.

use nf_formats::identity::{check_path, Expected, IdentityError, NIGHTFIRE_NTSC_U};

#[test]
fn the_real_disc_passes_the_identity_check() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let path = std::path::PathBuf::from(path);
    let t = std::time::Instant::now();
    let info = check_path(&path, &NIGHTFIRE_NTSC_U).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    eprintln!("identity check: {info:?} in {:?}", t.elapsed());
    assert_eq!(info.volume_id, "NIGHTFIRE");
    assert_eq!(info.serial, "SLUS-20579");
    assert_eq!(info.vmode, "NTSC");
    assert_eq!(info.version, "1.00");
    assert_eq!(info.sectors, 1_191_072, "docs/build-identity.md: 1,191,072 sectors");

    // Sister: flip one hex digit of ACTION.ELF's expected SHA-1 — the same image must now be rejected.
    let mut bad = NIGHTFIRE_NTSC_U.exe.1.to_string();
    let last = bad.pop().unwrap();
    bad.push(if last == '0' { '1' } else { '0' });
    let exp = Expected { exe: ("/ACTION.ELF", &bad), ..NIGHTFIRE_NTSC_U };
    assert_eq!(check_path(&path, &exp), Err(IdentityError::Modified("ACTION.ELF".into())));
}
