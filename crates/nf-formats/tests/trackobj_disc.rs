//! Disc test for `nf_formats::trackobj` (docs/formats/trackobj.md): the driving tracks' dynamic
//! scene objects, the trigger actions that wake/hide/show them, the parts the geometry decoder
//! leaves out, and the smackable attribute files.
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use std::collections::HashMap;

use nf_formats::crp::Crp;
use nf_formats::geoprim::{map_placements, part_models};
use nf_formats::iso9660::IsoImage;
use nf_formats::trackobj::*;
use nf_formats::trackstart::map_triggers;

const TRACKS: &[&str] = &["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

/// The VIV's files: (lower-case name, decompressed bytes).
fn viv_files(iso: &IsoImage, viv: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let mut out = Vec::new();
    for x in &g.entries {
        let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
        if raw.len() >= 64 && raw.iter().take(64).all(|&b| b == 0) {
            return None; // slim image: extent zeroed
        }
        let d = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap_or(raw) } else { raw };
        out.push((x.name.to_ascii_lowercase(), d));
    }
    Some(out)
}

fn track_crp(files: &[(String, Vec<u8>)]) -> &[u8] {
    &files.iter().find(|(n, _)| n.starts_with("data\\track\\") && n.ends_with(".crp")).expect("track .crp").1
}

fn iso() -> Option<IsoImage> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("NF_ISO unset: skipped");
        return None;
    };
    Some(IsoImage::open(&path).expect("open NF_ISO"))
}

/// (track, flag-0x10 items, scene objects, smackable behaviour, alt-wake behaviour, sim at init,
/// targetable, wake actions, hide, show, disable, enable) [D measured].
const COUNTS: &[(&str, [usize; 11])] = &[
    ("MIS01", [557, 557, 544, 0, 4, 0, 544, 0, 0, 4, 0]),
    ("MIS11", [168, 168, 0, 0, 9, 0, 0, 9, 0, 1, 0]),
    ("MIS13A", [83, 83, 16, 0, 9, 0, 16, 0, 0, 4, 1]),
    ("MIS13B", [73, 73, 32, 0, 1, 0, 32, 0, 0, 1, 0]),
    ("MIS13C", [41, 41, 16, 0, 0, 0, 16, 0, 0, 0, 0]),
    ("MIS3", [348, 348, 46, 0, 7, 0, 46, 0, 0, 5, 2]),
    ("MIS4", [270, 270, 213, 0, 16, 0, 213, 0, 0, 1, 0]),
    ("RACE", [253, 253, 212, 0, 0, 0, 212, 0, 0, 0, 0]),
];

#[test]
fn every_track_object_and_its_trigger_actions_resolve() {
    let Some(iso) = iso() else { return };
    let mut got = Vec::new();
    for &viv in TRACKS {
        let Some(files) = viv_files(&iso, viv) else {
            eprintln!("{viv}: slim image, skipped");
            return;
        };
        let crp = Crp::parse(track_crp(&files)).unwrap();
        let objs = track_objects(&crp).unwrap();
        let by_index: HashMap<usize, &TrackObject> = objs.iter().map(|o| (o.index, o)).collect();
        let acts = object_actions(&crp).unwrap();
        let mut n = [0usize; 11];
        n[0] = objs.len();
        for o in &objs {
            n[1] += o.scene_object() as usize;
            n[2] += o.smackable() as usize;
            n[3] += (o.record.behaviour & behaviour::ALT_WAKE != 0) as usize;
            n[4] += (o.record.behaviour & behaviour::SIM_AT_INIT != 0) as usize;
            n[5] += o.targetable() as usize;
        }
        let start = map_triggers(&crp).unwrap().iter().find(|t| t.actions.iter().any(|a| a.id == 0x16)).unwrap().index;
        for a in &acts {
            let o = by_index.get(&a.action.placement());
            if let ObjectAction::Wake { ps, .. } = a.action {
                let o = o.unwrap_or_else(|| panic!("{viv}: {a:?} wakes a placement without flag 0x10"));
                assert_eq!(ps, o.ps as usize, "{viv}: {a:?}: the ps reference is not the item's own record");
                assert!(o.scene_object(), "{viv}: {a:?} wakes an item without a scene object");
            }
            let k = match a.action {
                ObjectAction::Wake { .. } => 6,
                ObjectAction::Hide { .. } => 7,
                ObjectAction::Show { .. } => 8,
                ObjectAction::Disable { .. } => 9,
                ObjectAction::Enable { .. } => 10,
            };
            n[k] += 1;
            if k != 6 {
                eprintln!("  {viv} trigger {}{}: {:?} on {} ({:?})", a.trigger, if a.trigger == start { " (the start trigger)" } else { "" },
                    a.action, if o.is_some() { "a scene object" } else { "a static item" }, o.and_then(|o| o.article.clone()));
            }
        }
        eprintln!("{viv}: {n:?}");
        got.push((viv, n));
    }
    let want: Vec<_> = COUNTS.iter().map(|&(v, n)| (v, n)).collect();
    assert_eq!(got, want);
}

/// RACE: the four `POS2_Cautionguard` rails 4.5-4.7 units behind the car start are scene objects
/// with the smackable behaviour, each woken by its own trigger, and no trigger hides, shows,
/// disables or enables anything on the track — so nothing removes them at the start.
#[test]
fn race_start_barriers_are_smackables_that_nothing_hides() {
    let Some(iso) = iso() else { return };
    let Some(files) = viv_files(&iso, "RACE") else { return };
    let crp = Crp::parse(track_crp(&files)).unwrap();
    let objs = track_objects(&crp).unwrap();
    let start: Vec<&TrackObject> = objs
        .iter()
        .filter(|o| o.article.as_ref().map(|a| a.0.as_str()) == Some("POS2_Cautionguard") && (o.origin[2] - 173.4).abs() < 0.5)
        .collect();
    let idx: Vec<usize> = start.iter().map(|o| o.index).collect();
    assert_eq!(idx, [3166, 3167, 3168, 3169]);
    for o in &start {
        assert_eq!(o.flags, 0x10);
        assert_eq!(o.record, InstanceRecord { kind: 1, damage: 0, behaviour: 0x203, scene_part: 0 });
        assert!(o.scene_object() && o.smackable() && !o.targetable());
        assert_eq!(o.radius, 2.75);
    }
    let acts = object_actions(&crp).unwrap();
    let mut wakes: Vec<(usize, usize)> = acts
        .iter()
        .filter_map(|a| match a.action {
            ObjectAction::Wake { placement, .. } if idx.contains(&placement) => Some((a.trigger, placement)),
            _ => None,
        })
        .collect();
    wakes.sort();
    assert_eq!(wakes, [(9, 3169), (10, 3168), (11, 3167), (12, 3166)]);
    assert!(acts.iter().all(|a| matches!(a.action, ObjectAction::Wake { .. })), "RACE has a hide/show action");
    // The wake triggers sit on the rails (same x/z as the item).
    let trg = map_triggers(&crp).unwrap();
    for &(t, p) in &wakes {
        let o = start.iter().find(|o| o.index == p).unwrap();
        assert!((trg[t].pos[0] - o.origin[0]).abs() < 1e-3 && (trg[t].pos[2] - o.origin[2]).abs() < 1e-3);
    }
}

/// Every placement geoprim leaves without a model is either a collision-only `{as  ffff}`
/// article or an effect part (an `ef` list instead of a model).
#[test]
fn every_unresolved_part_is_collision_only_or_an_effect() {
    let Some(iso) = iso() else { return };
    for &viv in TRACKS {
        let Some(files) = viv_files(&iso, viv) else { return };
        let crp = Crp::parse(track_crp(&files)).unwrap();
        let models = part_models(&crp).unwrap();
        let kinds = placement_part_kinds(&crp).unwrap();
        let (mut coll, mut eff, mut other) = (0, 0, Vec::new());
        let mut effect_kinds: HashMap<u8, usize> = HashMap::new();
        for p in map_placements(&crp).unwrap() {
            let key = (p.article.clone(), p.part);
            if models.contains_key(&key) {
                assert_eq!(kinds.get(&key), Some(&PartKind::Model), "{viv}: {key:?}");
                continue;
            }
            match kinds.get(&key) {
                Some(PartKind::CollisionOnly) => coll += 1,
                Some(PartKind::Effect(e)) => {
                    eff += 1;
                    for x in e {
                        *effect_kinds.entry(x.kind).or_default() += 1;
                    }
                }
                k => other.push((key, k.cloned())),
            }
        }
        let mut ek: Vec<_> = effect_kinds.into_iter().collect();
        ek.sort();
        eprintln!("{viv}: collision-only {coll}, effect parts {eff} (effect kinds {ek:?}), other {other:?}");
        assert!(other.is_empty(), "{viv}: unclassified parts {other:?}");
    }
}

/// The PHYS names with no `.atr` section in their VIV [D]. The lookup then falls back to the key
/// "Bench" (`FUN_001950a0` 0x1950a0, string 0x37a6a0) [M].
const MISSING_ATR: &[&str] = &["MIS4:StreetTable", "MIS4:streettable", "RACE:StreetTable", "RACE:streettable"];

/// Every `PHYS::<name>` an article carries names a smackable `.atr` of the same VIV whose
/// `[Section]` is that name (case-insensitively), except [`MISSING_ATR`].
#[test]
fn every_phys_name_has_a_smackable_attribute_file() {
    let Some(iso) = iso() else { return };
    let mut missing = Vec::new();
    for &viv in TRACKS {
        let Some(files) = viv_files(&iso, viv) else { return };
        let crp = Crp::parse(track_crp(&files)).unwrap();
        let atr: Vec<SmackableAttrib> = files
            .iter()
            .filter(|(n, _)| n.starts_with("data\\sim\\attrib\\smackable\\") && n.ends_with(".atr"))
            .map(|(_, d)| SmackableAttrib::parse(&String::from_utf8_lossy(d)))
            .collect();
        let phys = article_phys_names(&crp).unwrap();
        // `PHYS::undefined` is the exporter's empty value; such articles fall back to "default".
        let mut names: Vec<&String> = phys.values().filter(|n| n.as_str() != "undefined").collect();
        names.sort();
        names.dedup();
        eprintln!("{viv}: {} smackable .atr, PHYS names {names:?}", atr.len());
        for n in names {
            if !atr.iter().any(|a| a.section.eq_ignore_ascii_case(n)) {
                missing.push(format!("{viv}:{n}"));
            }
        }
        eprintln!("  {viv}: default.atr present: {}", atr.iter().any(|a| a.section == "Default"));
    }
    eprintln!("missing {missing:?}");
    assert_eq!(missing, MISSING_ATR);
}
