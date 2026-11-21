//! Paris Prelude's opening sniper section through the real disc: `/DRIVING/MIS01.VIV` →
//! `data\track\paris_mis01.crp` → the `Map ` script (`el` lists, `sr` strings, `Trgr`, `Rule`,
//! `MSet`) → `nf_game::driving::pov`. Needs `NF_ISO` (the retail image); prints "skip" without.
//! Every pinned number is [D] (measured from these bytes); the code meaning of each field is
//! cited in `nf_formats::drive_script` / `nf_game::driving::pov` [M].

use nf_formats::bigf::Bigf;
use nf_formats::crp::Crp;
use nf_formats::drive_script::{parse_ai_elements, MapScript, Ref, ScriptEvent};
use nf_game::driving::flight::{self, AimArm, Body, MissionTimer, PathFollower, TimerRule, Triggers};
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

// ---------------------------------------------------------------------------------------------
// The flight (nf_game::driving::flight): AISp → pt tracks, the follower, the trigger contacts.

#[test]
fn the_flight_path_data() {
    let Some(m) = load() else { return };
    let s = &m.script;
    let sp = s.splines();
    assert_eq!(sp.len(), 19, "AISp = 19 x 0x70");
    assert_eq!(s.tracks.len(), 19, "pt records");
    assert_eq!((sp[0].name.as_str(), sp[0].identity_offset), ("path_Helicopter02", true));
    assert_eq!(s.track_of_spline(&sp[0]), Some(1));
    assert_eq!((sp[1].name.as_str(), s.track_of_spline(&sp[1])), ("path_Heli_Hover", Some(3)));
    let t = &s.tracks[&1];
    assert_eq!((t.duration, t.rot, t.pos, t.name.as_str()), (1264.0, Some(0), Some(1), "path_Helicopter02"));
    assert_eq!((t.channels[0].flags, t.channels[0].times.len()), (2, 131), "xform: quaternion keys");
    assert_eq!((t.channels[1].flags, t.channels[1].times.len()), (0, 150), "xlate: position keys");
    let k0 = t.channels[1].keys[0];
    assert!((k0[0] - 809.648).abs() < 1e-3 && (k0[2] - 215.038).abs() < 1e-3 && k0[3] == 1.0, "first position {k0:?}");
    // Every MIS01 track uses only the ported channel modes (lerp positions, nlerp rotations),
    // and every AISp record points at one.
    for (i, tr) in &s.tracks {
        for c in &tr.channels { assert!(c.flags == 0 || c.flags == 2, "pt {i}: channel flags {:#x}", c.flags); }
        assert!(tr.channels.iter().all(|c| c.times.windows(2).all(|w| w[0] <= w[1])), "pt {i}: times sorted");
    }
    for r in &sp { assert!(s.track_of_spline(r).is_some_and(|i| s.tracks.contains_key(&i)), "{}", r.name); }
    // The sniper damage is the MSet third array (+0x14c) for weapon 26: 50 (FUN_0014ca60
    // case 0x1a reads the slot +0x34); the Missile has 100.
    let ms = MissionSet::parse(&s.chunks["MSet"]);
    let (p, _) = PovSession::load(ms, "paris_mis01", 60, false);
    assert_eq!((p.loadout.slots[SNIPER_RIFLE].p34, p.loadout.slots[MISSILE].p34), (50, 100));
}

/// Every driving mission's `Map ` script still loads with the `pt  ` decoder.
#[test]
fn every_mission_track_decodes() {
    let Some(iso) = std::env::var_os("NF_ISO") else { return };
    let iso = IsoImage::open(&iso).expect("open the ISO");
    let mut counts = Vec::new();
    for (viv, track) in [
        ("MIS01.VIV", "paris_mis01.crp"), ("MIS11.VIV", "uw_mis11.crp"), ("MIS13A.VIV", "junglea_mis13a.crp"),
        ("MIS13B.VIV", "jungleb_mis13b.crp"), ("MIS13C.VIV", "junglec_mis13c.crp"), ("MIS3.VIV", "snow1a_mis3.crp"),
        ("MIS4.VIV", "snow2a_mis4.crp"), ("RACE.VIV", "snow2a_race.crp"),
    ] {
        let e = iso.find(&format!("/DRIVING/{viv}")).expect(viv);
        let raw = iso.read_bytes(e.lba as u64 * 2048, e.size as u64).expect("read");
        let g = Bigf::parse(&raw, raw.len() as u64).expect("BIGF");
        let x = g.entries.iter().find(|x| x.name == format!("data\\track\\{track}")).expect(track);
        let b = &raw[x.offset as usize..(x.offset + x.size) as usize];
        let b = if b.len() >= 2 && (b[0] & 0x3e) == 0x10 && b[1] == 0xfb { nf_formats::refpack::decompress(b).expect("refpack") } else { b.to_vec() };
        let s = MapScript::load(&Crp::parse(&b).expect("CARP")).unwrap_or_else(|e| panic!("{track}: {e}"));
        let other = s.tracks.values().flat_map(|t| &t.channels).filter(|c| c.flags != 0 && c.flags != 2).count();
        counts.push((track, s.tracks.len(), s.splines().len(), other));
    }
    eprintln!("{counts:?}");
    // MIS01 uses only the ported modes; other missions carry channels in the unported modes
    // (bit 0 / bit 3), which `channel_value` refuses.
    assert_eq!(counts[0], ("paris_mis01.crp", 19, 19, 0));
}

/// What the run saw: (tick, trigger) contacts in order, and where the follower ended.
struct Flight {
    fired: Vec<(i32, usize)>,
    stop_tick: i32,
    end: [f32; 3],
    track: Option<u16>,
    top_speed: f32,
}

fn run_events(s: &MapScript, evs: &[ScriptEvent], fol: &mut Option<PathFollower>, trig: &mut Triggers, now: f32) {
    let splines = s.splines();
    for e in evs {
        let player = e.word(1) == Some(0);
        match e.kind {
            // EControlToSpline / ESetNextAISpline: word 1 the vehicle (0 = the player), word 2
            // the AISp record.
            202 | 209 if player => {
                let tr = e.reference(2).and_then(|r| s.spline_of(r)).and_then(|i| s.track_of_spline(&splines[i])).expect("AISp track");
                if e.kind == 202 {
                    *fol = Some(PathFollower::start(tr, &s.tracks, now));
                } else if let Some(f) = fol {
                    f.next = Some(tr);
                }
            }
            239 if player => *fol = None,
            // The path events: word 1 the element (0 = the vehicle), word 2 the vehicle.
            116 | 153 | 215 if player && e.word(2) == Some(0) => {
                if let Some(f) = fol {
                    match e.kind {
                        116 => f.set_desired(e.f32(3).unwrap()),
                        153 => f.set_time_fraction(e.f32(3).unwrap(), &s.tracks),
                        _ => f.change_throttle(e.word(3).unwrap() as i32, e.f32(4).unwrap(), e.f32(5).unwrap()),
                    }
                }
            }
            53 | 54 => {
                if let Some(i) = e.reference(1).and_then(|r| s.trigger_of(r)) { trig.enable(i, e.kind == 54) }
            }
            _ => {}
        }
    }
}

fn fly(s: &MapScript, radius: f32) -> Flight {
    let fps = 60;
    let mut trig = Triggers::load(&s.triggers);
    let mut fol: Option<PathFollower> = None;
    let mut fired = Vec::new();
    let mut top_speed = 0.0f32;
    run_events(s, &s.lists[&686], &mut fol, &mut trig, flight::clock60(0, fps));
    let mut tick = 0;
    while tick < 20_000 {
        tick += 1;
        let Some(f) = fol.as_mut() else { break };
        f.update(flight::clock60(tick, fps), fps, &s.tracks);
        top_speed = top_speed.max(f.speed);
        if !f.playing { break; }
        let body = Body { pos: [f.pose[3][0], f.pose[3][1], f.pose[3][2]], vel: f.velocity, radius, class: 1 };
        let mut lists = Vec::new();
        for (i, rec) in s.triggers.iter().enumerate() {
            if trig.touch(i, rec, &body) {
                fired.push((tick, i));
                lists.push(rec.events.unwrap());
            }
        }
        for l in lists { run_events(s, &s.lists[&l], &mut fol, &mut trig, flight::clock60(tick, fps)); }
    }
    let f = fol.expect("still on a spline");
    Flight { fired, stop_tick: tick, end: [f.pose[3][0], f.pose[3][1], f.pose[3][2]], track: f.current, top_speed }
}

#[test]
fn the_helicopter_flies_the_phase() {
    let Some(m) = load() else { return };
    let s = &m.script;
    // The helicopter body radius is not traced (the half-extent length FUN_001f6e98 stores);
    // the contact order must not depend on it within a plausible range.
    let a = fly(s, 0.5);
    let b = fly(s, 2.0);
    let order = |f: &Flight| f.fired.iter().map(|x| x.1).collect::<Vec<_>>();
    eprintln!("r 0.5: {:?} stop {} end {:?} track {:?} top {}", a.fired, a.stop_tick, a.end, a.track, a.top_speed);
    eprintln!("r 2.0: {:?} stop {} end {:?} track {:?}", b.fired, b.stop_tick, b.end, b.track);
    assert_eq!(order(&a), order(&b));
    // Every player-class trigger the path crosses, in order, with the tick (60 Hz, radius
    // 0.5): music / speech / stage calls along the river (596, 631, 593, 736, 644, 591, 592,
    // 748, 731), the throttle changes (723: 0.6, 698: 0.85, 700: 0.68, 735: 0.65, 703: 1.0,
    // 701: 0 at 0.01 per frame), trigger 712 (stage 5 and its camera anim), 569 (a camera
    // spline) and 742 (ESetNextAISpline: path_Heli_Hover).
    assert_eq!(a.fired, vec![
        (22, 596), (46, 631), (105, 593), (190, 736), (420, 723), (922, 644), (973, 712), (1424, 698), (1850, 591),
        (1898, 592), (2069, 700), (2137, 735), (2372, 748), (2746, 703), (2905, 731), (3057, 569), (3435, 742), (3452, 701),
    ]);
    // 701 brakes from 1.0 at 0.01 a frame: stopped 102 updates later, on the hover track,
    // at the end of path_Helicopter02 in x/z.
    assert_eq!((a.stop_tick, a.track), (3554, Some(3)));
    assert!((a.end[0] - -143.019).abs() < 1e-3 && (a.end[2] - 542.398).abs() < 1e-3, "{:?}", a.end);
    assert_eq!(a.top_speed, 1.0);
}

#[test]
fn the_copter_view_limits_on_disc() {
    let Some(m) = load() else { return };
    let ini = String::from_utf8_lossy(&m.entries.iter().find(|(n, _)| n == "data\\render\\camera.ini").unwrap().1).into_owned();
    let arm = AimArm::from_ini(&ini, "CopterAutoDriveCam:Arm0");
    assert_eq!(arm.limits.max_yaw, [-0.315, -0.09, -0.07, 0.0, 0.0]);
    assert_eq!(arm.limits.min_yaw, [-0.315, -0.09, -0.07, 0.0, 0.0]);
    assert_eq!(arm.limits.max_pitch, [0.57, 0.28, 0.28, 0.0, 0.0]);
    assert_eq!(arm.limits.min_pitch, [-0.75, -0.53, 0.28, 0.0, 0.0]);
    assert_eq!((arm.aim_fov, arm.min_fov, arm.lock_to_car, arm.allow_deadzone), (20.0, 2.0, true, true));
    // The script start view (list 686: -60, -20 degrees) sits inside the limits.
    let ev = &m.script.lists[&686][1];
    let (mut pitch, mut yaw) = (-ev.f32(2).unwrap() / 360.0, ev.f32(1).unwrap() / 360.0);
    assert!(!arm.limits.clamp_view(&mut pitch, &mut yaw));
}

#[test]
fn the_hand_over_timer() {
    let Some(m) = load() else { return };
    let s = &m.script;
    // Trigger 727 (list 728): timer 5 = 4 s, counting down, stage 13.
    let k: Vec<(u32, Option<u32>, Option<u32>)> = s.lists[&728].iter().map(|e| (e.kind, e.word(1), e.word(2))).collect();
    assert!(k.contains(&(47, Some(5), Some(4.0f32.to_bits()))));
    assert!(k.contains(&(48, Some(5), Some(u32::MAX))));
    assert!(k.contains(&(41, Some(13), None)));
    // Stage 13 timer rules (type 9) and the tick each fires after the trigger.
    let mut timers = [MissionTimer::default(); 10];
    let t0 = 5000;
    timers[5].set_seconds(4.0, 60, t0);
    timers[5].start_counting(-1, t0);
    let mut rules: Vec<(usize, TimerRule)> = (0..s.rules.len())
        .filter(|&i| s.rules[i].group == 13 && s.rules[i].kind == 9)
        .map(|i| (i, TimerRule::from_payload(&s.rules[i].payload))).collect();
    assert_eq!(rules.iter().map(|r| (r.0, r.1.timer, r.1.rising, r.1.threshold)).collect::<Vec<_>>(),
        vec![(33, 5, false, 4.9), (34, 5, false, 1.0), (35, 5, false, 3.5), (36, 5, false, 3.9)]);
    let mut fired = Vec::new();
    for n in t0 + 1..t0 + 600 {
        for (i, r) in rules.iter_mut() { if r.eval(&timers, n, 60) { fired.push((*i, n - t0)); } }
    }
    // 36 (AI elements on) at +8, 35 (camera anim) at +32, 34 (the hand-over, list 789) at
    // +182 ticks; 33 never fires on this route (the timer starts below 4.9).
    assert_eq!(fired, vec![(36, 8), (35, 32), (34, 182)]);
    assert_eq!(s.rules[34].events, Some(789));
}
