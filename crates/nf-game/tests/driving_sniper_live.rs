//! Paris Prelude's opening sniper section through the real disc: `/DRIVING/MIS01.VIV` →
//! `data\track\paris_mis01.crp` → the `Map ` script (`el` lists, `sr` strings, `Trgr`, `Rule`,
//! `MSet`) → `nf_game::driving::pov`. Needs `NF_ISO` (the retail image); prints "skip" without.
//! Every pinned number is [D] (measured from these bytes); the code meaning of each field is
//! cited in `nf_formats::drive_script` / `nf_game::driving::pov` [M].

use nf_formats::bigf::Bigf;
use nf_formats::crp::Crp;
use nf_formats::drive_script::{parse_ai_elements, MapScript, Ref};
use nf_formats::iso9660::IsoImage;
use nf_game::driving::mission::MissionSet;
use nf_game::driving::pov::{GunnerOut, PovSession, KIND_SNIPER, MISSILE, SNIPER_RIFLE};

struct Mis01 {
    script: MapScript,
    entries: Vec<(String, Vec<u8>)>,
}

fn load() -> Option<Mis01> {
    let Some(iso) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return None };
    let iso = IsoImage::open(&iso).expect("open the ISO");
    let e = iso.find("/DRIVING/MIS01.VIV").expect("/DRIVING/MIS01.VIV");
    let raw = iso.read_bytes(e.lba as u64 * 2048, e.size as u64).expect("read MIS01.VIV");
    let g = Bigf::parse(&raw, raw.len() as u64).expect("BIGF");
    let mut entries = Vec::new();
    for x in &g.entries {
        let b = &raw[x.offset as usize..(x.offset + x.size) as usize];
        let b = if b.len() >= 2 && (b[0] & 0x3e) == 0x10 && b[1] == 0xfb { nf_formats::refpack::decompress(b).expect("refpack") } else { b.to_vec() };
        entries.push((x.name.clone(), b));
    }
    let crp_bytes = &entries.iter().find(|(n, _)| n == "data\\track\\paris_mis01.crp").expect("track crp").1;
    let crp = Crp::parse(crp_bytes).expect("CARP");
    let script = MapScript::load(&crp).expect("Map script");
    Some(Mis01 { script, entries })
}

fn kinds(s: &MapScript, list: u16) -> Vec<u32> {
    s.lists[&list].iter().map(|e| e.kind).collect()
}

#[test]
fn the_map_script_counts_and_every_reference_resolves() {
    let Some(m) = load() else { return };
    let s = &m.script;
    assert_eq!(s.lists.len(), 868, "el lists");
    assert_eq!(s.strings.len(), 2787, "sr strings");
    assert_eq!(s.triggers.len(), 754, "Trgr records");
    assert_eq!(s.rules.len(), 114, "Rule entries");
    for (i, t) in s.triggers.iter().enumerate() {
        let l = t.events.unwrap_or_else(|| panic!("trigger {i} has no el reference at +0x18"));
        assert!(s.lists.contains_key(&l), "trigger {i}: el{l} missing");
    }
    for (i, r) in s.rules.iter().enumerate() {
        let l = r.events.unwrap_or_else(|| panic!("rule {i} has no el reference at +0x0c"));
        assert!(s.lists.contains_key(&l), "rule {i}: el{l} missing");
    }
    // Every string reference inside every event resolves, except four the archive does not
    // carry — all four are EPlaySystemAnim (36) parameters in el573/el655.
    let mut missing = Vec::new();
    for (l, evs) in &s.lists {
        for e in evs {
            for w in 1..e.params.len() / 4 {
                if let Some(Ref::Str(i)) = e.reference(w) {
                    if !s.strings.contains_key(&i) { missing.push((*l, e.kind, i)); }
                }
            }
        }
    }
    assert_eq!(missing, vec![(573, 36, 2559), (573, 36, 2560), (655, 36, 2600), (655, 36, 2601)]);
}

#[test]
fn the_start_trigger_and_the_sniper_intro() {
    let Some(m) = load() else { return };
    let s = &m.script;
    // FUN_0022cd30 fires the triggers holding EStartMission (0x16) at world init.
    assert_eq!(s.triggers_with(22), vec![684]);
    assert_eq!(s.triggers[684].events, Some(685));
    assert_eq!(kinds(s, 685), vec![22]);
    // The only ESetAutoDriveCamOn in the mission sits on the next trigger, beside the start one.
    assert_eq!(s.triggers_with(77), vec![685]);
    let (a, b) = (&s.triggers[684], &s.triggers[685]);
    assert_eq!(b.events, Some(686));
    assert_eq!((b.pos[0], b.pos[2], b.radius), (778.332_76, 207.423_84, 2.5));
    let d = ((a.pos[0] - b.pos[0]).powi(2) + (a.pos[2] - b.pos[2]).powi(2)).sqrt();
    assert!(d < 1.0, "start and intro triggers {d} apart");
    assert_eq!(kinds(s, 686), vec![202, 77, 116, 41, 20, 225, 104, 229, 149, 149, 149, 149, 279]);
    let ev = &s.lists[&686];
    assert_eq!(s.string(ev[0].reference(2).unwrap()), Some("CARP::<<Map>>::{AISp}::0"), "EControlToSpline: the player on spline 0");
    assert_eq!((ev[1].f32(1), ev[1].f32(2)), (Some(-60.0), Some(-20.0)), "ESetAutoDriveCamOn angle/pitch");
    assert_eq!(ev[2].f32(3), Some(0.71), "EPathSetDesiredThrottleStatic");
    assert_eq!(ev[3].word(1), Some(19), "EChangeStage 19");
    assert_eq!(s.string(ev[4].reference(2).unwrap()), Some("CHAR::snipeintro"));
    assert_eq!((ev[5].word(1), ev[5].word(2)), (Some(2800), Some(2803)), "EObjectiveAdd");
    // ESetReflectiveObject names AIEl offset 5888 = record 23, Dominique's paradis_car.
    assert_eq!(s.string(ev[12].reference(1).unwrap()), Some("CARP::<<Map>>::{AIEl}::5888"));
    let ai = parse_ai_elements(&s.chunks["AIEl"]);
    assert_eq!(ai.len(), 25);
    assert_eq!(ai[23].vehicle, "paradis_car");
    // Stage 19's rules are the sniper section's (the hand-over rule sits in group 13).
    let g19: Vec<usize> = (0..s.rules.len()).filter(|&i| s.rules[i].group == 19).collect();
    assert_eq!(g19, vec![3, 45, 46, 79, 80, 85, 105, 107, 113]);
}

#[test]
fn the_hand_over_to_the_vanquish() {
    let Some(m) = load() else { return };
    let s = &m.script;
    let rules: Vec<usize> = (0..s.rules.len()).filter(|&i| {
        s.lists[&s.rules[i].events.unwrap()].iter().any(|e| e.kind == 240)
    }).collect();
    assert_eq!(rules, vec![34]);
    assert_eq!((s.rules[34].kind, s.rules[34].group, s.rules[34].events), (9, 13, Some(789)));
    assert_eq!(s.triggers_with(240), vec![741]);
    assert_eq!(s.triggers[741].events, Some(742));
    for l in [789u16, 742] {
        let ev = &s.lists[&l];
        let k: Vec<u32> = ev.iter().map(|e| e.kind).collect();
        for need in [240, 76, 78, 239, 140] { assert!(k.contains(&need), "el{l} lacks kind {need}"); }
        let t = ev.iter().find(|e| e.kind == 240).unwrap();
        assert_eq!(s.string(t.reference(1).unwrap()), Some("CHAR::paris"));
        let r = ev.iter().find(|e| e.kind == 140).unwrap();
        assert_eq!(s.trigger_of(r.reference(1).unwrap()), Some(683), "EResetPlayerCarPos → Trgr 43712");
        let v = ev.iter().find(|e| e.kind == 78).unwrap();
        assert_eq!(s.string(v.reference(1).unwrap()), Some("CHAR::paris2.mad"));
    }
    // The player vehicle: the MSet name, whose .atr names the Vanquish as its secondary.
    let atr = &m.entries.iter().find(|(n, _)| n == "data\\sim\\attrib\\pvehicle\\helicopter.atr").unwrap().1;
    let atr = String::from_utf8_lossy(atr);
    assert!(atr.contains("RENDER_FILENAME.s=helicopter.crp"));
    assert!(atr.contains("SECONDARY_TYPE.s=vanquish"));
}

#[test]
fn the_phase_runs_on_the_real_script() {
    let Some(m) = load() else { return };
    let s = &m.script;
    let ms = MissionSet::parse(&s.chunks["MSet"]);
    assert_eq!(ms.name, "helicopter");
    let (mut p, spawn) = PovSession::load(ms, "paris_mis01", 60, false);
    assert_eq!(p.loadout.current, MISSILE, "the load selects the Missile");
    let g = p.gunner.as_ref().expect("a POV weapon is owned → the gunner exists");
    assert_eq!((g.kind, g.state), (KIND_SNIPER, 8));
    assert!(spawn.contains(&GunnerOut::Play { kind: 3, anim: 2, bank: 0xe }));

    p.apply_list(685, s);
    assert!(!p.pov);
    let out = p.apply_list(686, s);
    assert!(p.pov);
    assert_eq!(out, vec![GunnerOut::Model { kind: 3, shown: true }]);
    assert_eq!(p.cam_start_turns, Some((-60.0 / 360.0, -20.0 / 360.0)));
    assert_eq!(p.on_spline.as_deref(), Some("CARP::<<Map>>::{AISp}::0"));
    assert_eq!(p.next_stage, Some(19));
    p.update(1);
    assert_eq!(p.loadout.current, SNIPER_RIFLE, "the POV swap selects the Sniper Rifle");
    assert_eq!(p.loadout.other, MISSILE);
    assert_eq!(p.loadout.slots[SNIPER_RIFLE].total, 24, "24 rounds carried");

    let out = p.apply_list(789, s);
    assert!(!p.pov);
    assert!(out.contains(&GunnerOut::Model { kind: 3, shown: false }));
    assert!(!p.loadout.slots[SNIPER_RIFLE].owned(), "the transition drops the rifle");
    assert_eq!(p.loadout.slots[0].total, 500);
    assert_eq!(p.loadout.slots[MISSILE].total, 30);
    assert_eq!(p.loadout.current, MISSILE);
    p.update(2);
    assert_eq!(p.loadout.current, MISSILE);
    assert_eq!(p.video, Some(("paris2.mad".to_string(), Some("paris2.sub".to_string()))));
    assert_eq!(p.reset_to_trigger, Some(683));
    assert_eq!(p.on_spline, None);
}
