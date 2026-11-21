//! Every `data\Control\*.def` in every `DRIVING/*.VIV` of the disc, decoded with
//! `nf_formats::controldef` and resolved into binding sets with `nf_game::input::driving_cfg`
//! (docs/systems/driving-input.md §13). Opt-in via NF_ISO (the full image; a slim image whose
//! `/DRIVING/` extents are zero prints "skip" and passes).
//!
//! Pinned [D]: which archives carry the configs, the 12 distinct files, each file's binding count
//! and FRONTEND block, that every action and value name resolves (no 0x75 unknown id, no value
//! outside the InputTable), that Master.def names exactly the files present, and the shipped
//! driving mapping (drivecfg.def + FrontEnd.def) pad button → action.

use std::collections::BTreeMap;

use nf_formats::controldef::{ControlDef, MasterDef};
use nf_formats::iso9660::IsoImage;
use nf_game::input::driving::{action, value, BindingKind};
use nf_game::input::driving_cfg::{action_id, binding_set, frontend, value_index, UNKNOWN_ACTION};

/// (archive, `.def` entries) [D measured on the disc].
const ARCHIVES: &[(&str, usize)] = &[
    ("MIS01", 11), ("MIS11", 11), ("MIS13A", 11), ("MIS13B", 11), ("MIS13C", 11), ("MIS3", 11),
    ("MIS4", 11), ("MISC", 12), ("RACE", 11),
];

/// (file name lower-cased, binding lines, has FRONTEND block) [D].
const FILES: &[(&str, usize, bool)] = &[
    ("dbgcfg.def", 5, false),
    ("drivecfg.def", 24, true),
    ("frontend.def", 34, false),
    ("povcfg0.def", 21, true),
    ("povcfg1.def", 20, true),
    ("povcfg2.def", 20, true),
    ("povcfg3.def", 19, true),
    ("povcfg4.def", 19, true),
    ("povcfg5.def", 19, true),
    ("povcfg6.def", 19, true),
    ("povcfg7.def", 20, true),
];

fn iso() -> Option<IsoImage> {
    let p = std::env::var_os("NF_ISO")?;
    IsoImage::open(p).ok()
}

/// Every control `.def` of one archive: (lower-cased file name, decompressed bytes). `None` on a
/// slim image.
fn defs(iso: &IsoImage, viv: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let mut out = Vec::new();
    for x in &g.entries {
        let n = x.name.to_ascii_lowercase();
        let Some(file) = n.strip_prefix(r"data\control\") else { continue };
        if !file.ends_with(".def") { continue; }
        let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
        if raw.iter().take(32).all(|&b| b == 0) { return None; }
        let data = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
        out.push((file.to_string(), data));
    }
    Some(out)
}

fn all() -> Option<BTreeMap<String, Vec<(String, Vec<u8>)>>> {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return None };
    let mut m = BTreeMap::new();
    for e in iso.entries() {
        let p = e.path.to_ascii_uppercase();
        let Some(stem) = p.strip_prefix("/DRIVING/").and_then(|s| s.strip_suffix(".VIV")) else { continue };
        let Some(d) = defs(&iso, stem) else { eprintln!("skip: {stem}.VIV extents are zero (slim image)"); return None };
        if !d.is_empty() { m.insert(stem.to_string(), d); }
    }
    Some(m)
}

#[test]
fn every_def_in_every_archive_decodes_and_resolves() {
    let Some(m) = all() else { return };
    let got: Vec<(&str, usize)> = m.iter().map(|(k, v)| (k.as_str(), v.len())).collect();
    assert_eq!(got, ARCHIVES.to_vec(), "archives carrying control configs");

    // Copies are byte-identical across archives (by name) [D].
    let mut distinct: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (viv, files) in &m {
        for (name, data) in files {
            if let Some(prev) = distinct.get(name) {
                assert_eq!(prev, data, "{viv}: {name} differs from another archive's copy");
            } else {
                distinct.insert(name.clone(), data.clone());
            }
        }
    }
    assert_eq!(distinct.len(), 12, "11 binding files + Master.def");

    let master = MasterDef::parse(&distinct["master.def"]).unwrap();
    assert_eq!((master.pov_count, master.driving_count), (8, 1));
    assert_eq!(master.pov, (0..8).map(|i| format!("povcfg{i}.def")).collect::<Vec<_>>());
    assert_eq!(master.driving, vec!["drivecfg.def".to_string()]);
    for (viv, files) in &m {
        for f in master.pov.iter().chain(&master.driving).map(|s| s.to_ascii_lowercase()).chain(["frontend.def".into()]) {
            assert!(files.iter().any(|(n, _)| *n == f), "{viv}: Master.def / the loader names {f}, not in the archive");
        }
    }

    let mut rows = Vec::new();
    for (name, data) in &distinct {
        if name == "master.def" { continue; }
        let d = ControlDef::parse(data).unwrap_or_else(|e| panic!("{name}: {e}"));
        for b in &d.bindings {
            assert_ne!(action_id(&b.action), UNKNOWN_ACTION, "{name}: unknown action {}", b.action);
            assert!(value_index(&b.value).is_some(), "{name}: unknown value {}", b.value);
            if let Some(t) = &b.method_text {
                assert!(nf_formats::controldef::method_type(t).is_some(), "{name}: unknown method letter {t}");
            }
        }
        binding_set(&[&d]).unwrap();
        rows.push((name.as_str(), d.bindings.len(), d.frontend.is_some()));
    }
    assert_eq!(rows, FILES.to_vec());
}

#[test]
fn shipped_driving_mapping() {
    let Some(m) = all() else { return };
    let files = &m["MIS01"];
    let get = |n: &str| ControlDef::parse(&files.iter().find(|(f, _)| f == n).unwrap().1).unwrap();
    let (fe, dc) = (get("frontend.def"), get("drivecfg.def"));
    let set = binding_set(&[&fe, &dc]).unwrap();
    let on = |v: &str| -> Vec<(BindingKind, u8)> {
        set[value_index(v).unwrap()].bindings.iter().map(|b| (b.kind, b.action)).collect()
    };
    use BindingKind::*;
    // Steering = the left stick X, raw (type 0), plus the camera and the steer sounds [D].
    assert!(on("ALX").contains(&(Always, action::STEER)));
    assert_eq!(set[value::ALX].bindings.iter().filter(|b| b.action == action::STEER).count(), 1);
    // Pedals are the analog pressures of ✕ (gas) and □ (brake) [D].
    assert_eq!(on("AButtonCross"), vec![(Always, action::GAS)]);
    assert_eq!(on("AButtonSquare"), vec![(Always, action::BRAKE)]);
    // Handbrake: ○ pressure crossing 0.1 up / down [D].
    let hb = &set[value_index("AButtonCircle").unwrap()].bindings;
    assert_eq!(hb.len(), 2);
    assert_eq!((hb[0].kind, hb[0].arg, hb[0].action), (CrossPos, 0.1, action::HANDBRAKE));
    assert_eq!((hb[1].kind, hb[1].arg, hb[1].action), (CrossNeg, 0.1, action::HANDBRAKERELEASE));
    // Fire = R1 → FIRESECONDARY; nothing binds FIREPRIMARY [D].
    // (FrontEnd.def's cheat-code entries ride the same buttons first: R1 = CHEATACTION_TRIGGER_RIGHT.)
    assert_eq!(on("DButtonR1"), vec![(Pressed, 0x63), (Released, 0x64), (Pressed, action::FIRESECONDARY), (Released, action::FIRESECONDARYRELEASE)]);
    assert!(set.iter().all(|v| v.bindings.iter().all(|b| b.action != action::FIREPRIMARY)));
    assert_eq!(on("DButtonL1"), vec![(Pressed, 0x61), (Released, 0x62), (Pressed, action::FIREGADGET), (Released, action::FIREGADGETRELEASE)]);
    // Weapon cycling: d-pad Left = next (UP), d-pad Right / R2 = previous (DOWN) [D].
    assert!(on("DButtonLeft").contains(&(Pressed, action::TOGGLESECONDARYUP)));
    assert!(on("DButtonRight").contains(&(Pressed, action::TOGGLESECONDARYDOWN)));
    assert_eq!(on("DButtonR2"), vec![(Pressed, action::TOGGLESECONDARYDOWN)]);
    // Camera view △, look back L2 hold pair [D].
    assert!(on("DButtonTriangle").contains(&(Pressed, action::CHANGECAMERA)));
    assert_eq!(on("DButtonL2"), vec![(Pressed, action::CAMLOOKBACK), (Released, action::CAMLOOKBACKRELEASE)]);
    // Pause = Start through FrontEnd.def [D].
    assert_eq!(on("DButtonStart"), vec![(Pressed, frontend::START)]);
    // No reload, horn or select binding while driving; POVRELOAD lives in the POV configs [D].
    assert!(set.iter().all(|v| v.bindings.iter().all(|b| b.action != action::POVRELOAD)));
    let pov0 = get("povcfg0.def");
    assert!(pov0.bindings.iter().any(|b| b.action == "GAMEACTION_POVRELOAD"));
    // The POV configs carry the invert pair: N (7) on CAMROTATEY, I (6) on CAMROTATEYINV [D].
    let y: Vec<(u8, &str)> = pov0.bindings.iter().filter(|b| b.action.starts_with("GAMEACTION_CAMROTATEY"))
        .map(|b| (b.method, b.action.as_str())).collect();
    assert!(y.contains(&(7, "GAMEACTION_CAMROTATEY")) && y.contains(&(6, "GAMEACTION_CAMROTATEYINV")));
}
