//! Disc test for `nf_game::driving::props`: which track scene objects are hidden at the first
//! frame (the `EStartMission` trigger's actions, fired at Init World [M 0x22cd30]) on every
//! driving track, and that RACE's start barriers stay drawn (docs/formats/trackobj.md).
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_formats::trackobj::{object_actions, track_objects, ObjectAction};
use nf_formats::trackstart::{map_triggers, ACTION_START_MISSION};
use nf_game::driving::props::TrackProps;

fn track_crp(iso: &IsoImage, viv: &str) -> Option<Vec<u8>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let t = g.entries.iter().find(|x| {
        let n = x.name.to_ascii_lowercase();
        n.starts_with("data\\track\\") && n.ends_with(".crp")
    })?;
    let raw = iso.read_bytes(e.lba as u64 * 2048 + t.offset as u64, t.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None;
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

/// (track, scene objects, placements hidden at the first frame) [D measured over M rules].
const HIDDEN_AT_START: &[(&str, usize, &[usize])] = &[
    ("MIS01", 557, &[]),
    ("MIS11", 168, &[]),
    ("MIS13A", 83, &[571, 572, 573, 643]),
    ("MIS13B", 73, &[]),
    ("MIS13C", 41, &[]),
    ("MIS3", 348, &[]),
    ("MIS4", 270, &[3196]),
    ("RACE", 253, &[]),
];

#[test]
fn hidden_scene_objects_at_the_first_frame() {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("NF_ISO unset: skipped");
        return;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let mut got = Vec::new();
    for &(viv, ..) in HIDDEN_AT_START {
        let Some(d) = track_crp(&iso, viv) else {
            eprintln!("{viv}: slim image, skipped");
            return;
        };
        let crp = Crp::parse(&d).unwrap();
        let objs = track_objects(&crp).unwrap();
        let start = map_triggers(&crp).unwrap().into_iter().find(|t| t.actions.iter().any(|a| a.id == ACTION_START_MISSION)).unwrap().index;
        let acts: Vec<ObjectAction> = object_actions(&crp).unwrap().into_iter().filter(|a| a.trigger == start).map(|a| a.action).collect();
        let t = TrackProps::at_start(&objs, &acts);
        let mut hidden: Vec<usize> = t.props.iter().filter(|p| !p.visible).map(|p| p.placement).collect();
        hidden.sort();
        eprintln!("{viv}: {} scene objects, hidden at start {hidden:?}", t.props.len());
        if viv == "RACE" {
            for i in 3166..=3169 {
                let p = t.get(i).expect("RACE start barrier is a scene object");
                assert!(p.visible && !p.in_sim && p.smackable_hp.is_none(), "RACE barrier {i}: {p:?}");
            }
        }
        got.push((viv, t.props.len(), hidden));
    }
    let want: Vec<_> = HIDDEN_AT_START.iter().map(|&(v, n, h)| (v, n, h.to_vec())).collect();
    assert_eq!(got, want);
}
