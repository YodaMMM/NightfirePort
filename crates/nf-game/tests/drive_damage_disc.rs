//! Disc test for `nf_game::driving::damage`: every driving archive's mission set (`MSet`, the
//! health/armour words at +0x27c..+0x2a8 that `FUN_00206ec0` reads) and every pvehicle `.atr`
//! (the tyre/invulnerability words the car constructor `FUN_001862c0` and `FUN_00187350` read).
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_game::driving::damage::{HealthSet, PlayerHealth, StartInputs, VehicleDamageAttrs};

const VIVS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn entry(iso: &IsoImage, e: &nf_formats::iso9660::IsoEntry, g: &nf_formats::bigf::Bigf, i: usize) -> Option<Vec<u8>> {
    let t = &g.entries[i];
    let raw = iso.read_bytes(e.lba as u64 * 2048 + t.offset as u64, t.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None;
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).ok()? } else { raw })
}

/// (track .crp MSet payload, [(pvehicle name, .atr text)]) of one archive; `None` on a slim image.
fn archive(iso: &IsoImage, viv: &str) -> Option<(Option<Vec<u8>>, Vec<(String, String)>)> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let ti = g.entries.iter().position(|x| {
        let n = x.name.to_ascii_lowercase();
        n.starts_with("data\\track\\") && n.ends_with(".crp")
    })?;
    let crp_bytes = entry(iso, e, &g, ti)?;
    let crp = Crp::parse(&crp_bytes).ok()?;
    let mut mset = None;
    if let Some(map) = crp.pool_find(0x4d61_7020).ok()? {
        for r in crp.record_window(&map).ok()? {
            if &r.tag_bytes() == b"MSet" {
                mset = Some(crp.payload(&r).ok()?.to_vec());
            }
        }
    }
    let mut atrs = Vec::new();
    for (i, x) in g.entries.iter().enumerate() {
        let n = x.name.to_ascii_lowercase();
        if let Some(rest) = n.strip_prefix("data\\sim\\attrib\\pvehicle\\") {
            if let Some(car) = rest.strip_suffix(".atr") {
                let b = entry(iso, e, &g, i)?;
                atrs.push((car.to_string(), String::from_utf8_lossy(&b).into_owned()));
            }
        }
    }
    atrs.sort();
    Some((mset, atrs))
}

/// Per archive: (health max, health, armour max, armour, health % by difficulty 0/1/2, armour %
/// by difficulty 0/1/2) at MSet +0x27c..+0x2a8 [D, measured 2026-11-21].
const SETS: [(&str, [i32; 4], [i32; 3], [i32; 3]); 8] = [
    ("MIS01", [3500, 3500, 1000, 0], [100, 75, 50], [100, 75, 50]),
    ("MIS11", [3000, 3000, 1000, 0], [100, 88, 75], [100, 88, 75]),
    ("MIS13A", [6000, 6000, 2000, 0], [100, 90, 85], [100, 75, 60]),
    ("MIS13B", [7500, 7500, 2500, 0], [100, 95, 90], [100, 95, 90]),
    ("MIS13C", [5000, 5000, 1000, 1000], [100, 90, 80], [100, 75, 50]),
    ("MIS3", [8000, 8000, 2667, 0], [100, 95, 90], [100, 95, 90]),
    ("MIS4", [3000, 3000, 1000, 0], [100, 88, 75], [100, 88, 75]),
    ("RACE", [3000, 3000, 1000, 0], [100, 88, 75], [100, 88, 75]),
];

/// Every archive's health set, and what `FUN_00206ec0` makes of it per difficulty [D + M].
/// Sister: a wrong offset or a wrong difficulty column fails the compare.
#[test]
fn every_archive_health_set_and_start_numbers() {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("NF_ISO unset: skipped");
        return;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    for (viv, words, hp, ap) in SETS {
        let Some((mset, _)) = archive(&iso, viv) else {
            eprintln!("{viv}: slim image, skipped");
            return;
        };
        let m = mset.unwrap_or_else(|| panic!("{viv}: every driving track has an MSet [D]"));
        assert_eq!(m.len(), 0x2f8, "{viv}: the MSet payload is the 0x2f8-byte record FUN_00204de8 clears");
        let h = HealthSet::parse(&m);
        assert_eq!(([h.health_max, h.health, h.armour_max, h.armour], h.health_pct, h.armour_pct), (words, hp, ap), "{viv}");
        for d in 0..3 {
            let p = PlayerHealth::start(&h, &StartInputs { difficulty: d, ..Default::default() });
            let (hk, ak) = (hp[d as usize] as f32 / 100.0, ap[d as usize] as f32 / 100.0);
            let want = (words[1] as f32 * hk, words[0] as f32 * hk, words[3] as f32 * ak, words[2] as f32 * ak);
            assert_eq!(p.hud(), want, "{viv} difficulty {d}");
            eprintln!("{viv} difficulty {d}: health {}/{} armour {}/{}", p.health, p.health_max, p.armour, p.armour_max);
        }
    }
}

/// Every pvehicle `.atr` (66 over 8 archives): only `default.atr` carries `TYRE_DAMAGE_POINTS`
/// (30); `TYRES_INVUNERABLE` is true on the two bomb vans and `paradis_car`; no file sets
/// `IS_TANK`, only `helicopter` sets `IS_HELICOPTER`; the subs and snowmobiles are the ones the
/// tyre gate excludes; `IS_POLICE` is the alpine police car only; no file carries a health or
/// armour value [D].
#[test]
fn every_vehicle_atr_damage_words() {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("NF_ISO unset: skipped");
        return;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let mut subs = std::collections::BTreeSet::new();
    let mut snow = std::collections::BTreeSet::new();
    let mut sub_physics = std::collections::BTreeSet::new();
    let mut police = std::collections::BTreeSet::new();
    let mut tanks = std::collections::BTreeSet::new();
    let mut tyre_proof = std::collections::BTreeSet::new();
    let mut helis = std::collections::BTreeSet::new();
    let mut files = 0;
    for viv in VIVS {
        let Some((_, atrs)) = archive(&iso, viv) else {
            eprintln!("{viv}: slim image, skipped");
            return;
        };
        let def = atrs.iter().find(|(c, _)| c == "default").unwrap_or_else(|| panic!("{viv}: no default.atr"));
        let d = VehicleDamageAttrs::parse_atr(&def.1);
        assert_eq!((d.tyre_points, d.tyres_invulnerable), (Some(30), false), "{viv} default.atr");
        for (car, text) in &atrs {
            files += 1;
            let a = VehicleDamageAttrs::parse_atr(text);
            if car != "default" {
                assert_eq!(a.tyre_points, None, "{viv} {car}: TYRE_DAMAGE_POINTS");
            }
            if a.tyres_invulnerable { tyre_proof.insert(car.clone()); }
            if a.is_tank { tanks.insert(car.clone()); }
            if a.is_helicopter { helis.insert(car.clone()); }
            for key in ["HEALTH", "HITPOINTS", "ARMOUR", "ARMOR", "DAMAGE_SCALE"] {
                assert!(!text.to_ascii_uppercase().contains(key), "{viv} {car}: carries {key}");
            }
            if a.is_sub { subs.insert(car.clone()); }
            if a.is_snowmobile { snow.insert(car.clone()); }
            if a.sub_physics { sub_physics.insert(car.clone()); }
            if a.is_police { police.insert(car.clone()); }
        }
    }
    eprintln!("tanks {tanks:?} helicopters {helis:?} subs {subs:?} snow {snow:?} sub_physics {sub_physics:?} police {police:?} tyres_invulnerable {tyre_proof:?}");
    let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(subs, set(&["alpha_sub", "mini_sub", "subcopter", "vanquishsub"]));
    assert_eq!(snow, set(&["small_snowmobile", "supersnow"]));
    assert_eq!(sub_physics, set(&["alpha_sub", "mini_sub", "plane", "subcopter", "ultralight", "ultralightbig", "vanquishsub"]));
    assert_eq!(police, set(&["alpine_police"]));
    assert!(tanks.is_empty(), "no file sets IS_TANK: {tanks:?}");
    assert_eq!(helis, set(&["helicopter"]));
    assert_eq!(tyre_proof, set(&["jungle_bombvan", "paradis_car", "paris_bombvan"]));
    assert_eq!(files, 66);
    eprintln!("{files} pvehicle files over 8 archives");
}
