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
use nf_formats::trackstart::{map_script, track_spawn, MapScript, TAG_EL};
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
const IDS_PER_TRACK: [(&str, usize); 8] =
    [("MIS01", 88), ("MIS11", 68), ("MIS13A", 56), ("MIS13B", 50), ("MIS13C", 26), ("MIS3", 62), ("MIS4", 49), ("RACE", 28)];

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
                f.cars.push(CarFacts { kind: 2, pos: [t.pos[0], t.pos[1] + 0.5, t.pos[2]], vel: [0.0; 3], radius: 2.5 });
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
            // T65 calls stage 39; R10 wins with counter 0x2d at 3. The three counts are the
            // EIncMagicCounter of R168/R170/R171/R172 (stages 32..35); the run counts them
            // directly, the way into those stages is not exercised.
            "MIS11" => {
                for _ in 0..3 {
                    s.count(0x2d);
                }
                into(&m, 65, &mut f);
                "end zone with the three counts"
            }
            // T562 changes to stage 1; R1 there waits for programmer event 0 (set by game code;
            // the setter is not traced), then timer 5 = 3 s and stage 2; R37 wins below 0.95 s [D].
            "MIS01" => {
                into(&m, 562, &mut f);
                steps += run(&mut s, &mut f, 2);
                assert_eq!(s.stage, 1, "{viv}");
                f.player_pos = [1e4, 0.0, 1e4];
                s.set_event(0);
                "stage 1, programmer event 0"
            }
            // R16 (default list) at counter 0xb == 3 starts timer 0 (6 s); R37 changes to stage 8
            // below 4 s; R15 wins below 1 s [D]. No list of the track counts 0xb; the run counts
            // it directly (its source is not found).
            "MIS13A" => {
                for _ in 0..3 {
                    s.count(0xb);
                }
                "counter 0xb at 3"
            }
            // Its win rules need timer 6, which only R9 (stage 3) starts, and no list changes to or
            // calls stage 3; R7 (timer 0) has its active bit clear [D]. Not reachable here.
            "MIS13C" => {
                let calls3 = m.triggers.iter().flat_map(|t| &t.actions).chain(m.rules.iter().flat_map(|r| &r.actions))
                    .any(|a| (a.id == 0x28 || a.id == 0x29) && a.word(1) == 3);
                assert!(!calls3);
                got.push((*viv, s.state, s.outcome));
                continue;
            }
            _ => unreachable!(),
        };
        steps += run(&mut s, &mut f, 60 * 60);
        eprintln!("{viv}: {how}: {:?} outcome {} after {steps} steps", s.state, s.outcome);
        got.push((*viv, s.state, s.outcome));
    }
    let want: Vec<_> = TRACKS
        .iter()
        .map(|&v| if v == "MIS13C" { (v, DriveState::Playing, 3) } else { (v, DriveState::Won, 0) })
        .collect();
    assert_eq!(got, want);
}
