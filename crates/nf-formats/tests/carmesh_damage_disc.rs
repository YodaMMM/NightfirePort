//! Disc test for the car damage arrays (`nf_formats::carmesh::{CarDamageVertex, deformed}`,
//! docs/formats/car-model.md §10.5): every car file of the 8 driving archives.
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use std::collections::BTreeMap;

use nf_formats::carmesh::{car_mesh, deformed, CarMaterial, DamageZone, DAMAGE_ZONES};
use nf_formats::iso9660::IsoImage;

const ARCHIVES: &[&str] = &["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn read(iso: &IsoImage, lba: u32, off: u32, size: u32) -> Option<Vec<u8>> {
    let raw = iso.read_bytes(lba as u64 * 2048 + off as u64, size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None;
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

/// (archive, stem, crp bytes) of every `data\car\model\*.crp`; `None` on a slim image.
fn cars() -> Option<Vec<(String, String, Vec<u8>)>> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let mut out = Vec::new();
    for viv in ARCHIVES {
        let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
        let g = nf_formats::disc::read_bigf(&iso, e).ok()?;
        for x in &g.entries {
            let n = x.name.to_ascii_lowercase();
            let Some(stem) = n.strip_prefix(r"data\car\model\").and_then(|s| s.strip_suffix(".crp")) else { continue };
            let Some(crp) = read(&iso, e.lba, x.offset, x.size) else {
                eprintln!("skip: no /DRIVING/ extents in this image");
                return None;
            };
            out.push((viv.to_string(), stem.to_owned(), crp));
        }
    }
    Some(out)
}

/// Every strip of every car file: the two damage arrays go to exactly the VU1 addresses the
/// strip's microcode reads them from ([`CarMaterial::damage_slots`]), as `V4-16` exactly for the
/// programs with the packed set-up loop; every zone index is one of the 16 `DamageZones`; with
/// zero zones the damage loop changes nothing; with every zone at a = 1 the deformation reaches
/// the measured extremes. Counts are [D measured].
#[test]
fn every_car_strip_carries_damage_arrays_where_its_microcode_reads_them() {
    let Some(cars) = cars() else { return };
    assert_eq!(cars.len(), 58);
    let mut strips: BTreeMap<CarMaterial, usize> = BTreeMap::new();
    let mut zones = [0usize; 16];
    let (mut verts, mut moved, mut w_neg, mut w_zero) = (0usize, 0usize, 0usize, 0usize);
    let mut max_off = 0.0f32;
    let none = [DamageZone::default(); DAMAGE_ZONES];
    let full = [DamageZone { x: 1.0, y: 0.0, z: 0.0, w: 0x7f }; DAMAGE_ZONES];
    for (viv, stem, crp) in &cars {
        let m = car_mesh(crp).unwrap_or_else(|e| panic!("{viv} {stem}: {e}"));
        for p in &m.parts {
            for s in &p.strips {
                let at = format!("{viv} {stem} {:?}", s.material);
                *strips.entry(s.material).or_default() += 1;
                let (a, b) = s.material.damage_slots().unwrap_or_else(|| panic!("{at}: no damage slots"));
                assert_eq!(s.damage_vu, Some([a, b]), "{at}");
                assert_eq!(s.damage_packed, s.material.packed_setup(), "{at}");
                assert_eq!(s.damage.len(), s.positions.len(), "{at}");
                let u = deformed(s, &none).unwrap_or_else(|| panic!("{at}: zone index past 15"));
                assert_eq!(u.positions, s.positions, "{at}");
                assert_eq!(u.normals, s.normals, "{at}");
                assert!(u.rgb.iter().all(|&c| c == 127) && u.c_w.iter().all(|&c| c == 127.0), "{at}");
                let want_alpha = if s.material.glass_alpha() { 0 } else { 127 };
                assert!(u.alpha.iter().all(|&a| a == want_alpha), "{at}");
                let f = deformed(s, &full).unwrap();
                for (k, d) in s.damage.iter().enumerate() {
                    verts += 1;
                    zones[d.zone as usize] += 1;
                    let off = [f.positions[k][0] - s.positions[k][0], f.positions[k][1] - s.positions[k][1], f.positions[k][2] - s.positions[k][2]];
                    let l = (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt();
                    moved += (l > 0.0) as usize;
                    max_off = max_off.max(l);
                    w_neg += (d.vec[3] < 0.0) as usize;
                    w_zero += (d.vec[3] == 0.0) as usize;
                }
                if s.material.glass_alpha() {
                    assert!(f.alpha.iter().all(|&a| a == 0x7f), "{at}");
                }
            }
        }
    }
    eprintln!("strips {strips:?}");
    eprintln!("{verts} vertices, {moved} move at a = 1 (max {max_off}), DmgVec.w < 0 on {w_neg}, = 0 on {w_zero}; zones {zones:?}");
    let want: BTreeMap<CarMaterial, usize> = [
        (CarMaterial::TexturePacked, 10917),
        (CarMaterial::TextureDullPacked, 18626),
        (CarMaterial::TextureSwitchPacked, 987),
        (CarMaterial::GouraudDullPacked, 136),
        (CarMaterial::GlassPacked, 198),
        (CarMaterial::Chrome, 2728),
        (CarMaterial::Dash, 6598),
    ]
    .into_iter()
    .collect();
    assert_eq!(strips, want);
    assert_eq!(verts, 249_596);
    assert_eq!(moved, 99_631);
    assert!((max_off - 2.053_075_3).abs() < 1e-5, "{max_off}");
    assert_eq!(zones, [24929, 21438, 8595, 11612, 8624, 8777, 27234, 15082, 13384, 27485, 11924, 8963, 18827, 19038, 11731, 11953]);
}
