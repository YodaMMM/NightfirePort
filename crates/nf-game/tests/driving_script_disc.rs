//! Disc proof for the driving missions' rules (`nf_game::driving::script`, docs/systems/
//! driving-missions.md): every action id the eight tracks use, with its event class and handler,
//! and, per track, the start, a fail and (where the track's data reaches it) a win run through
//! the game's own triggers, rules and events.
//!
//! Reads the full retail image named by `NF_ISO` (the `DRIVING/*.VIV` archives); skips cleanly
//! when it is unset or the image has no `DRIVING/` extents.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_formats::trackstart::{map_script, prop_scripts, track_spawn, MapScript, PropScript, TAG_EL};
use nf_game::driving::script_effects::{decode, Effect, Ref};
use nf_game::driving::script::{action_info, CarFacts, DriveScript, DriveState, ElementFacts, Facts, Output, ACTIONS, PORTED};

const TRACKS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn track_crp(iso: &IsoImage, viv: &str) -> Option<Vec<u8>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let t = g.entries.iter().find(|x| {
        let n = x.name.to_ascii_lowercase();
        n.starts_with("data\\track\\") && n.ends_with(".crp")
    })?;
    let raw = iso.read_bytes(e.lba as u64 * 2048 + t.offset as u64, t.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None; // slim image: extent zeroed
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

fn tracks() -> Option<Vec<(&'static str, Vec<u8>)>> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if track_crp(&iso, "MIS01").is_none() {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return None;
    }
    Some(TRACKS.iter().map(|&v| (v, track_crp(&iso, v).unwrap())).collect())
}

/// Every `el` action list of the `Map` window: (el index, action ids), for the lists no trigger or
/// rule points at too (the AI elements' lists).
fn all_lists(crp: &Crp) -> Vec<(u16, Vec<u32>)> {
    let map = crp.pool().unwrap().into_iter().find(|r| r.tag == nf_formats::crp::tag::MAP).unwrap();
    let mut out = Vec::new();
    for r in crp.record_window(&map).unwrap() {
        if r.tag_code() != TAG_EL {
            continue;
        }
        let p = crp.payload(&r).unwrap();
        let w = |o: usize| u32::from_le_bytes(p[o..o + 4].try_into().unwrap());
        out.push((r.instance().unwrap(), (0..w(0) as usize).map(|j| w(0x10 + j * 0x10)).collect()));
    }
    out
}

/// Distinct action ids per track [D], pinned.
/// Distinct action ids per track (the `Map` lists and the prop keyframe lists) [D], pinned.
const IDS_PER_TRACK: [(&str, usize); 8] =
    [("MIS01", 95), ("MIS11", 77), ("MIS13A", 70), ("MIS13B", 63), ("MIS13C", 34), ("MIS3", 75), ("MIS4", 58), ("RACE", 34)];

#[test]
fn every_action_id_on_the_eight_tracks_has_its_class_and_handler() {
    let Some(tracks) = tracks() else { return };
    let mut union = BTreeSet::new();
    let mut per_track = Vec::new();
    for (viv, dec) in &tracks {
        let crp = Crp::parse(dec).unwrap();
        let m = map_script(&crp).unwrap();
        // Uses by owner: trigger lists, rule lists, other lists.
        let mut uses: BTreeMap<u32, [usize; 3]> = BTreeMap::new();
        let owned: HashMap<u32, usize> = m
            .triggers
            .iter()
            .flat_map(|t| t.actions.iter().map(|a| (a.id, 0)))
            .chain(m.rules.iter().flat_map(|r| r.actions.iter().map(|a| (a.id, 1))))
            .fold(HashMap::new(), |mut h, (id, k)| {
                *h.entry(id << 1 | k as u32).or_insert(0) += 1;
                h
            });
        for (_, ids) in all_lists(&crp) {
            for id in ids {
                uses.entry(id).or_default()[2] += 1;
            }
        }
        // The prop articles' keyframe lists count as "other" too.
        for p in prop_scripts(&crp).unwrap() {
            for t in &p.tracks {
                for (_, l) in &t.events {
                    for a in l {
                        uses.entry(a.id).or_default()[2] += 1;
                    }
                }
            }
        }
        for (key, n) in owned {
            let e = uses.entry(key >> 1).or_default();
            e[(key & 1) as usize] += n;
            e[2] -= n;
        }
        eprintln!("{viv}: {} triggers, {} rules, {} distinct action ids", m.triggers.len(), m.rules.len(), uses.len());
        for (id, [t, r, o]) in &uses {
            let a = action_info(*id).unwrap_or_else(|| panic!("{viv}: action {id:#x} has no table entry"));
            eprintln!(
                "  {id:#05x} {:<30} creator {:#08x} handler {:#08x} | triggers {t:4} rules {r:4} other {o:4}{}",
                a.name,
                a.creator,
                a.handler,
                if PORTED.contains(id) { " | ported" } else { "" }
            );
            union.insert(*id);
        }
        per_track.push((*viv, uses.len()));
    }
    eprintln!("{per_track:?}");
    // The table holds exactly the ids the disc uses [D].
    assert_eq!(union.len(), ACTIONS.len());
    assert!(ACTIONS.iter().all(|a| union.contains(&a.id)));
    assert_eq!(per_track, IDS_PER_TRACK.to_vec());
}

/// A car standing in trigger `t` (moving along its direction, for the one-way ones).
fn into(m: &MapScript, t: usize, f: &mut Facts) {
    let tr = &m.triggers[t];
    f.player_pos = [tr.pos[0], tr.pos[1] + 0.5, tr.pos[2]];
    f.player_vel = [tr.dir[0] * 20.0, tr.dir[1] * 20.0, tr.dir[2] * 20.0];
}

/// A prop article's keyframe script by name.
fn prop(dec: &[u8], name: &str) -> PropScript {
    prop_scripts(&Crp::parse(dec).unwrap()).unwrap().into_iter().find(|p| p.name == name).unwrap()
}

/// The vehicle name of the `AIEl` element at byte offset `off` (0x100-byte elements, the name at
/// `+0x54`) [D].
fn aiel_vehicle(dec: &[u8], off: usize) -> String {
    let crp = Crp::parse(dec).unwrap();
    let map = crp.pool().unwrap().into_iter().find(|r| r.tag == nf_formats::crp::tag::MAP).unwrap();
    let r = crp.record_window(&map).unwrap().into_iter().find(|r| r.tag == 0x4149_456c).unwrap();
    let p = &crp.payload(&r).unwrap()[off + 0x54..off + 0x74];
    String::from_utf8_lossy(&p[..p.iter().position(|&b| b == 0).unwrap()]).into_owned()
}

fn element(m: &MapScript, path: &str) -> u32 {
    *m.refs.iter().find(|(_, p)| p.as_str() == path).unwrap_or_else(|| panic!("no sr {path}")).0
}

fn new_run(m: &MapScript) -> (DriveScript, Facts) {
    let mut s = DriveScript::new(m.clone());
    // The player parked far from every volume.
    let mut f = Facts { player_pos: [1e5, 0.0, 1e5], player_radius: 2.5, difficulty: 1, ..Default::default() };
    // Every AI element a rule names, alive, keeping up with the player ([`run`]).
    for r in &m.rules {
        if r.word(0x10) == 4 {
            f.elements.insert(r.word(0x14), ElementFacts { health: 1000.0, speed: 20.0, pos: [1e5, 0.0, 1e5], ..Default::default() });
        }
    }
    s.drain(&f);
    (s, f)
}

fn run(s: &mut DriveScript, f: &mut Facts, steps: usize) -> usize {
    for n in 1..=steps {
        let p = f.player_pos;
        for e in f.elements.values_mut() {
            e.pos = p;
        }
        s.step(f);
        if s.state != DriveState::Playing {
            return n;
        }
    }
    steps
}

#[test]
fn every_track_starts_at_its_start_mission_trigger() {
    let Some(tracks) = tracks() else { return };
    for (viv, dec) in &tracks {
        let crp = Crp::parse(dec).unwrap();
        let m = map_script(&crp).unwrap();
        let spawn = track_spawn(&crp, &crp.extract_collision_tris().unwrap()).unwrap();
        let (mut s, _) = new_run(&m);
        let starts: Vec<_> = s.take_output().into_iter().filter(|o| matches!(o, Output::StartMission { .. })).collect();
        eprintln!("{viv}: {starts:?}");
        assert!(s.started, "{viv}");
        assert_eq!(starts, vec![Output::StartMission { trigger: Some(spawn.trigger), speed_factor: spawn.speed_factor }], "{viv}");
        assert_eq!((s.state, s.outcome), (DriveState::Playing, 3), "{viv}");
    }
}

#[test]
fn every_track_fails_by_its_own_rules() {
    let Some(tracks) = tracks() else { return };
    let mut got = Vec::new();
    for (viv, dec) in &tracks {
        let m = map_script(&Crp::parse(dec).unwrap()).unwrap();
        let (mut s, mut f) = new_run(&m);
        let how = match *viv {
            // The rival (a kind-2 car) reaches the finish volume T229 first: it disables the
            // player's finish T214 and loses [D RACE T229 flags 0x40b].
            "RACE" => {
                let t = &m.triggers[229];
                f.cars.push(CarFacts { kind: 2, pos: [t.pos[0], t.pos[1] + 0.5, t.pos[2]], vel: [0.0; 3], radius: 2.5, bbox: None });
                "rival at the finish"
            }
            // The end zone T65 calls stage 39, where R11 loses while counter 0x2d < 3 [D].
            "MIS11" => {
                into(&m, 65, &mut f);
                "end zone without the three counts"
            }
            // The others carry a player Death (type 3) or Health (type 4, below 1) rule in the
            // default list [D]; the car's health runs out. The Health rule fires on the crossing
            // only (its previous value starts at 0 [M 0x20a128]), so the run drives with health
            // first.
            _ => {
                assert_eq!(run(&mut s, &mut f, 5), 5, "{viv}");
                s.health = 0.0;
                "no health left"
            }
        };
        let n = run(&mut s, &mut f, 600);
        eprintln!("{viv}: {how}: {:?} outcome {} after {n} steps", s.state, s.outcome);
        got.push((*viv, s.state, s.outcome));
    }
    let want: Vec<_> = TRACKS
        .iter()
        .map(|&v| (v, DriveState::Lost, if v == "RACE" || v == "MIS11" { 1 } else { 4 }))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn tracks_win_by_their_own_rules() {
    let Some(tracks) = tracks() else { return };
    let mut got = Vec::new();
    for (viv, dec) in &tracks {
        let m = map_script(&Crp::parse(dec).unwrap()).unwrap();
        let (mut s, mut f) = new_run(&m);
        let mut steps = 0;
        let how = match *viv {
            // The player reaches the finish T214: timer 9 = 5 s, stage 2; R9 wins below 1 s [D].
            "RACE" => {
                into(&m, 214, &mut f);
                "finish line"
            }
            // T244: EPlayerWin [D].
            "MIS3" => {
                into(&m, 244, &mut f);
                "end trigger"
            }
            // T33: EEndMission [D].
            "MIS13B" => {
                into(&m, 33, &mut f);
                "end trigger"
            }
            // T260 changes to stage 11; R38..R43 (Death of six AI elements) each count 0x2f;
            // R52 at 6 starts timer 2 (18 s); R37 wins below 1 s [D].
            "MIS4" => {
                into(&m, 260, &mut f);
                steps += run(&mut s, &mut f, 2);
                assert_eq!(s.stage, 11, "{viv}");
                for p in [2304, 2816, 5120, 3328, 3584, 4864] {
                    f.elements.get_mut(&element(&m, &format!("CARP::<<Map>>::{{AIEl}}::{p}"))).unwrap().health = 0.0;
                }
                "end zone, six targets down"
            }
            // The three upload points T51 / T52 / T54 (spheres, fired every frame while inside)
            // give special weapon 0x14 and call stages 32 / 34 / 35; firing it there sets counter
            // 0x22 (FUN_0020de20 case 0x14), so that stage's rule (R168 / R171 / R172: counter
            // 0x22 > 0) shows the instance, counts 0x2d and disables its trigger. Then the end
            // zone T65 calls stage 39, where R10 wins with counter 0x2d at 3 [D, M 0x20de20].
            "MIS11" => {
                for t in [51, 52, 54] {
                    into(&m, t, &mut f);
                    steps += run(&mut s, &mut f, 1);
                    assert_eq!(s.special_weapon, Some(0x14), "{viv} T{t}");
                    let before = s.counters[0x2d];
                    s.player_fired(0x14);
                    steps += run(&mut s, &mut f, 1);
                    assert_eq!(s.counters[0x2d], before + 1, "{viv} T{t}");
                }
                into(&m, 65, &mut f);
                "three uploads, end zone"
            }
            // T562 changes to stage 1; R1 there waits for programmer event 0, which the player's
            // EMP sets on hitting a car named "paris_bombvan" (FUN_0018ac28); then timer 5 = 3 s
            // and stage 2; R37 wins below 0.95 s [D, M 0x18ac28].
            "MIS01" => {
                into(&m, 562, &mut f);
                steps += run(&mut s, &mut f, 2);
                assert_eq!(s.stage, 1, "{viv}");
                f.player_pos = [1e4, 0.0, 1e4];
                let van = aiel_vehicle(dec, 0x100);
                assert_eq!(van, "paris_bombvan");
                s.emp_hit(&van, true);
                "stage 1, the bomb van EMP'd"
            }
            // R16 (default list) at counter 0xb == 3 starts timer 0 (6 s); R37 changes to stage 8
            // below 4 s; R15 wins below 1 s. Counter 0xb is counted by the JA_SAL prop's
            // animation an6 (track aq9 keyframe 0: EIncMagicCounter 0xb), run by the animation
            // player FUN_001a35f0; three of its four instances playing it reach 3 [D, M 0x1a35f0].
            "MIS13A" => {
                let p = prop(dec, "JA_SAL");
                for _ in 0..3 {
                    for l in p.anim_events(6) {
                        s.run_actions(l);
                    }
                    steps += run(&mut s, &mut f, 1);
                }
                assert_eq!(s.counters[0xb], 3, "{viv}");
                "three JA_SAL props played an6"
            }
            // The subMissiles prop's animation an3 (track aq4 keyframe 3: EHideInstance +
            // ECallStage 3) runs stage 3: R9 starts timer 6 (4 s), and R24 (default list) wins
            // below 1 s [D, M 0x1a35f0].
            "MIS13C" => {
                let p = prop(dec, "subMissiles");
                assert!(p.anim_events(3).iter().any(|l| l.iter().any(|a| a.id == 0x28 && a.word(1) == 3)));
                for l in p.anim_events(3) {
                    s.run_actions(l);
                }
                "subMissiles played an3"
            }
            _ => unreachable!(),
        };
        steps += run(&mut s, &mut f, 60 * 60);
        eprintln!("{viv}: {how}: {:?} outcome {} after {steps} steps", s.state, s.outcome);
        got.push((*viv, s.state, s.outcome));
    }
    let want: Vec<_> = TRACKS.iter().map(|&v| (v, DriveState::Won, 0)).collect();
    assert_eq!(got, want);
}

#[test]
fn prop_keyframe_lists_hold_the_counts_and_the_stage_call_the_wins_wait_for() {
    // MIS13A: JA_SAL's animation an6 plays tracks aq9 / aq10 / aq11; aq9 (7 keyframes) runs
    // EIncMagicCounter(0xb) on keyframe 0 and EHideInstance on keyframe 6. MIS13C: subMissiles'
    // an3 plays aq4 / aq5 / aq6; aq4 keyframe 3 runs EHideInstance + ECallStage(3) [D].
    let Some(tracks) = tracks() else { return };
    let get = |v: &str| &tracks.iter().find(|(t, _)| *t == v).unwrap().1;
    let ids = |l: &[nf_formats::trackstart::TriggerAction]| l.iter().map(|a| (a.id, a.word(1))).collect::<Vec<_>>();
    let sal = prop(get("MIS13A"), "JA_SAL");
    assert_eq!(sal.anims.iter().find(|a| a.index == 6).unwrap().tracks, vec![Some(9), Some(10), Some(11), None]);
    let aq9 = sal.tracks.iter().find(|t| t.index == 9).unwrap();
    assert_eq!(aq9.keys, 7);
    assert_eq!(aq9.events.iter().map(|(k, l)| (*k, ids(l))).collect::<Vec<_>>(), vec![(0, vec![(0xb9, 0xb)]), (6, vec![(0xb, 0)])]);
    let sub = prop(get("MIS13C"), "subMissiles");
    assert_eq!(sub.anims.iter().find(|a| a.index == 3).unwrap().tracks, vec![Some(4), Some(5), Some(6), None]);
    let aq4 = sub.tracks.iter().find(|t| t.index == 4).unwrap();
    assert_eq!(aq4.events.iter().map(|(k, l)| (*k, ids(l))).collect::<Vec<_>>(), vec![(3, vec![(0xb, 3), (0x28, 3)])]);
    // No Map list of either track counts 0xb or calls stage 3 (the reason the props are needed).
    for (v, hit) in [("MIS13A", (0xb9u32, 0xbu32)), ("MIS13C", (0x28, 3))] {
        let m = map_script(&Crp::parse(get(v)).unwrap()).unwrap();
        let all = m.triggers.iter().flat_map(|t| &t.actions).chain(m.rules.iter().flat_map(|r| &r.actions));
        assert!(!all.into_iter().any(|a| (a.id, a.word(1)) == hit || (hit.0 == 0x28 && a.id == 0x29 && a.word(1) == 3)), "{v}");
    }
}

#[test]
fn every_effect_decodes_with_references_of_the_right_kind() {
    // Every unported action of the eight tracks' Map lists decodes; the reference fields name
    // the record kinds the handlers expect [D]: elements {AIEl}, commands {AICo}, splines {AISp},
    // instances {in  0000}, camera animations {Cams}, sounds and channels CHAR:: strings.
    let Some(tracks) = tracks() else { return };
    // An empty path = a word naming an `sr` record the file lacks (e.g. MIS01 0x9ff / 0xa00,
    // MIS11 0x26e, absent from their Map windows [D]).
    let missing = std::cell::RefCell::new(BTreeSet::new());
    let kind = |r: &Option<Ref>, want: &[&str]| {
        r.as_ref().map_or(true, |r| {
            if r.path.is_empty() {
                missing.borrow_mut().insert(r.word);
                return true;
            }
            want.iter().any(|w| r.path.contains(w))
        })
    };
    let mut n = 0usize;
    let mut other: BTreeSet<u32> = BTreeSet::new();
    for (viv, dec) in &tracks {
        let m = map_script(&Crp::parse(dec).unwrap()).unwrap();
        let lists = m.triggers.iter().map(|t| (Some(t.index), &t.actions)).chain(m.rules.iter().map(|r| (None, &r.actions)));
        for (trig, l) in lists {
            for a in l {
                if PORTED.contains(&a.id) {
                    continue;
                }
                n += 1;
                let e = decode(a, trig, &m);
                let ok = match &e {
                    Effect::AiCommand { command, target, element, .. } => {
                        kind(command, &["{AICo}"]) && kind(target, &["{AIEl}"]) && kind(element, &["{AIEl}"])
                    }
                    Effect::AiActivate { element, other } => kind(element, &["{AIEl}"]) && kind(other, &["{AIEl}"]),
                    Effect::AiDeactivate { element }
                    | Effect::AiFire { element, .. }
                    | Effect::AiAccuracy { element, .. }
                    | Effect::AiKill { element, .. }
                    | Effect::AiVehicleParams { element, .. }
                    | Effect::AiWakeRange { element, .. }
                    | Effect::TargetBeacon { element, .. }
                    | Effect::AiAvoidZone { element }
                    | Effect::CarCameraAnim { element, .. } => kind(element, &["{AIEl}"]),
                    Effect::CameraLockOn { element, .. } => kind(element, &["{AIEl}"]),
                    Effect::CameraAnim { cams, .. } => kind(cams, &["{Cams}"]),
                    Effect::CameraSpline { spline, .. } | Effect::AiNextSpline { spline, .. } | Effect::ControlToSpline { spline, .. } => {
                        kind(spline, &["{AISp}"])
                    }
                    Effect::Stream { channel, stream, .. } => kind(channel, &["CHAR::"]) && kind(stream, &["CHAR::"]),
                    Effect::Sound { bank, name } => kind(bank, &["CHAR::"]) && kind(name, &["CHAR::"]),
                    Effect::SoundAt { at, bank, name, .. } => {
                        (a.word(1) == 0 || at.is_some()) && kind(bank, &["CHAR::"]) && kind(name, &["CHAR::"])
                    }
                    Effect::StopSoundAt { at } => at.is_some(),
                    Effect::AudioMix { channel, .. } | Effect::RevertAudioMix { channel, .. } | Effect::StingerChannel { channel } => {
                        kind(channel, &["CHAR::"])
                    }
                    Effect::Movie { file, subtitles } => kind(file, &["CHAR::"]) && kind(subtitles, &["CHAR::"]),
                    Effect::InstanceVisible { instance, .. }
                    | Effect::CollisionGeometry { instance, .. }
                    | Effect::PathSound { instance, .. }
                    | Effect::SpawnSentry { instance, .. } => kind(instance, &["{in  0000}"]),
                    Effect::SystemAnim { instance, placement, .. } | Effect::WakeSmackable { instance, placement, .. } => {
                        kind(instance, &["{in  0000}"]) && kind(placement, &["{ps  0000}"])
                    }
                    Effect::PlayEffect { effect, instance, .. } | Effect::SuppressEffect { effect, instance, .. } => {
                        kind(effect, &["{ef  "]) && kind(instance, &["{in  0000}"])
                    }
                    Effect::Explosion { at, trigger, instance, .. } => {
                        kind(instance, &["{in  0000}"])
                            && kind(at, &["{Trgr}", "TRIGGER0=INSTANCE_ELEMENT::"])
                            && at.as_ref().map_or(true, |r| r.path.contains("{Trgr}") == trigger.is_some())
                    }
                    Effect::SpawnTraffic { at, .. } => at.is_some(),
                    Effect::Other { id, .. } => {
                        other.insert(*id);
                        true
                    }
                    _ => true,
                };
                assert!(ok, "{viv}: {:#x} {e:?}", a.id);
            }
        }
    }
    eprintln!("{n} effect actions; kept as Other: {:x?}; missing sr: {:x?}", other, missing.borrow());
    // Ten distinct reference words over the eight tracks name `sr` records their files lack [D].
    assert_eq!(missing.borrow().len(), 10);
    // The ids left as `Other` are HUD, power-up, path-throttle and player-car tuning actions.
    assert!(other.iter().all(|id| ![0x3f, 0x14, 0x41, 0x42, 0x1c, 0x78, 0x17, 0x18, 0x23, 0x4e].contains(id)));
}
