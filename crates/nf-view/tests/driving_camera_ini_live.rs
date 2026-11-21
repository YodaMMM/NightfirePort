//! The chase camera's tuning against the disc: `data\render\camera.ini` in every
//! `/DRIVING/*.VIV` (refpack) carries the `[AnyCarHeliCam0]` / `[VanquishHeliCam0]` sections the
//! port's `nf_game::driving::camera` presets copy [D]. Opt-in: NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, refpack};
use nf_game::driving::camera::{parse_heli, ANYCAR_HELI_CAM0, VANQUISH_HELI_CAM0};

const VIVS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn camera_ini(iso: &IsoImage, viv: &str) -> String {
    let entry = iso.find(&format!("/DRIVING/{viv}.VIV")).expect("viv");
    let bigf = disc::read_bigf(iso, entry).expect("bigf");
    let e = bigf
        .entries
        .iter()
        .find(|x| x.name.eq_ignore_ascii_case("data\\render\\camera.ini"))
        .expect("camera.ini");
    let raw = iso.read_bytes(entry.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let bytes = if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb {
        refpack::decompress(&raw).expect("refpack")
    } else {
        raw
    };
    String::from_utf8_lossy(&bytes).into_owned()
}

#[test]
fn every_driving_viv_ships_the_camera_presets() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return };
    let iso = IsoImage::open(&path).expect("iso");
    let first = camera_ini(&iso, VIVS[0]);
    assert!(first.contains("[AnyCarHeliCam0]"), "no AnyCar heli cam");
    for viv in VIVS {
        let t = camera_ini(&iso, viv);
        assert_eq!(t, first, "{viv}: camera.ini differs from MIS01's");
        assert_eq!(parse_heli(&t, "AnyCarHeliCam0"), ANYCAR_HELI_CAM0, "{viv} AnyCarHeliCam0");
        assert_eq!(parse_heli(&t, "VanquishHeliCam0"), VANQUISH_HELI_CAM0, "{viv} VanquishHeliCam0");
    }
}
