//! What the driving mission's rules and trigger walk read about the AI elements
//! (`HostileWorld::element_facts` / `script_elements` / `trigger_cars`, docs/systems/
//! driving-ai.md §14). Each test pins one decompiled rule:
//!
//! - the selector reads the element's record in any state (`FUN_0012c640`, no liveness test):
//!   health = AIEl +0x4c, which the live actor's hit points point at (`FUN_00195a08`), so a
//!   wreck keeps 0 after its despawn [M 0x209820, 0x12cb08, 0x195a58];
//! - position / speed = the runtime record's +0x30 / +0x3c: live values, else the record's
//!   position and 0 [M 0x12b440, 0x2098a0, 0x209758];
//! - the element properties `FUN_0012b138` answers (EMPTY, AWAKE);
//! - the `SRulePlayerDir` bearing (`FUN_0020ac70`);
//! - the AI ground vehicles walk the triggers as kind 2 (`FUN_001862c0`: sim types 2 and 4 → +0x6d
//!   = 2) [M].

use std::collections::HashMap;

use nf_formats::driveai::{flag, AiCommand, AiElement, AiRider, MapAi, AICO_RECORD, TYPE_HENCHMAN, TYPE_VEHICLE};
use nf_formats::trackstart::{MapScript, RuleDef, Trigger, TriggerAction};
use nf_game::driving::ai::DriveRng;
use nf_game::driving::hostile::{ElemState, WorldAttr};
use nf_game::driving::hostiles::*;
use nf_game::driving::script::{DriveScript, Facts};
use nf_game::driving::FlatWorld;

const CAR: &str = "[guncar]\nRENDER_FILENAME.s=guncar.crp\n";

fn vehicle(pos: [f32; 3], flags: u16, health: f32) -> AiElement {
    AiElement {
        facing: [0.0, 0.0, 1.0],
        pos,
        flags,
        difficulty: 0,
        kind: TYPE_VEHICLE,
        state: 0,
        health,
        name: "guncar".into(),
        hench_class: 0,
        hench_model: 0,
        hench_weapon: 0,
        ctor_byte: 0,
        heli: false,
        riders: Vec::new(),
        command_ref: 0,
    }
}

fn rider(health: f32) -> AiRider {
    AiRider { seat: 3, model: 0, weapon: 1, raw_0c: 0, raw_10: 0, raw_18: 0, health, skill: 0 }
}

fn world(els: Vec<AiElement>) -> HostileWorld {
    let cmd = AiCommand::parse(&vec![0u8; AICO_RECORD]).unwrap();
    let m = MapAi { elements: els, commands: vec![cmd], ..Default::default() };
    let w = WorldAttr { hench_slots: Some(8), vehicle_slots: Some(8), heli_slots: Some(4), ..Default::default() };
    HostileWorld::new(&m, vec![VehicleAttrs::parse("guncar", CAR)], &w, 0, 1.0, 1.0, None)
}

fn player(pos: [f32; 3]) -> PlayerState {
    PlayerState { pos, vel: [0.0; 3], fwd: [0.0, 0.0, 1.0], box_z: 2.3 }
}

fn step(w: &mut HostileWorld, p: [f32; 3]) {
    let mut rng = DriveRng::game_reset();
    let _ = w.step(None, &FlatWorld::paved(), None, &player(p), &mut rng, &mut |_, _| 0);
}

/// A script whose `sr` words 0x7372_0001.. name `{AIEl}::0x100·k` (as the disc's do) and one
/// rule entry per `(type, element, flags, payload)` on element word 0x7372_0001 + `element`.
fn script(rules: &[(u32, u32, u32, [u32; 2])]) -> DriveScript {
    let mut map = MapScript::default();
    for k in 0..4u32 {
        map.refs.insert(0x7372_0001 + k, format!("CARP::<<Map>>::{{AIEl}}::{}", k * 0x100));
    }
    map.refs.insert(0x7372_0010, "CARP::<<Map>>::{AICo}::0".into());
    for (i, (ty, el, fl, p)) in rules.iter().enumerate() {
        let mut raw = [0u8; 0x20];
        for (k, w) in [*ty, *fl, u32::MAX, 0x656c_0000, 4, 0x7372_0001 + el, p[0], p[1]].iter().enumerate() {
            raw[k * 4..k * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        let mut args = 0x83u32.to_le_bytes().to_vec();
        args.extend_from_slice(&(1u32 << i).to_le_bytes());
        map.rules.push(RuleDef { index: i, raw, actions: vec![TriggerAction { id: 0x83, args }] });
    }
    DriveScript::new(map)
}

#[test]
fn element_refs_name_the_record_by_offset() {
    let s = script(&[]);
    assert_eq!(element_refs(&s.map), vec![(0x7372_0001, 0), (0x7372_0002, 1), (0x7372_0003, 2), (0x7372_0004, 3)], "{{AICo}} words are not elements");
}

#[test]
fn health_is_the_record_float_in_every_state_and_a_wreck_keeps_0() {
    // Element 0 near the player, element 1 far (never spawns).
    let mut w = world(vec![vehicle([0.0, 0.0, 20.0], 0, 500.0), vehicle([5000.0, 0.0, 0.0], 0, 80.0)]);
    let refs = element_refs(&script(&[]).map);
    let f = w.elements_for(&refs, [0.0; 3]);
    assert_eq!((f[&0x7372_0001].health, f[&0x7372_0002].health), (500.0, 80.0), "idle: the record's +0x4c");
    assert!(!f.contains_key(&0x7372_0003), "no record 2: no facts (the selector would read past the table)");
    step(&mut w, [0.0; 3]);
    assert_eq!(w.db.elements[0].state, ElemState::Live);
    w.damage(0, 120.0);
    assert_eq!(w.element_facts(0, [0.0; 3]).unwrap().health, 380.0, "live: the actor's hit points are the record's float");
    w.damage(0, 1000.0);
    assert_eq!(w.element_facts(0, [0.0; 3]).unwrap().health, 0.0, "FUN_00195a58 floors at 0");
    // The player leaves: the wreck despawns beyond 220 and is gone; its health stays 0.
    for _ in 0..8 {
        step(&mut w, [0.0, 0.0, 3000.0]);
    }
    assert_eq!(w.db.elements[0].state, ElemState::Gone);
    assert!(w.live.is_empty());
    assert_eq!(w.element_facts(0, [0.0; 3]).unwrap().health, 0.0);
}

#[test]
fn a_death_rule_fires_once_when_the_hit_points_reach_0() {
    // SRuleDeath (type 3) on element 0, SRuleDamage (type 2, > 50 in a frame) on element 0,
    // SRuleDeath on element 1 (alive, never spawned): only the first two fire.
    let mut w = world(vec![vehicle([0.0, 0.0, 20.0], 0, 300.0), vehicle([5000.0, 0.0, 0.0], 0, 80.0)]);
    let mut s = script(&[(3, 0, 0b011, [0, 0]), (2, 0, 0b001, [50, 0]), (3, 1, 0b011, [0, 0])]);
    let mut facts = Facts { player_radius: 2.0, ..Default::default() };
    let mut tick = |w: &mut HostileWorld, s: &mut DriveScript, f: &mut Facts| {
        step(w, [0.0; 3]);
        f.elements = w.script_elements(&s.map, [0.0; 3]);
        s.step(f);
        s.take_fired_rules()
    };
    assert_eq!(tick(&mut w, &mut s, &mut facts), Vec::<usize>::new());
    w.damage(0, 40.0);
    assert_eq!(tick(&mut w, &mut s, &mut facts), Vec::<usize>::new(), "40 in a frame is not above 50");
    w.damage(0, 60.0);
    assert_eq!(tick(&mut w, &mut s, &mut facts), vec![1]);
    w.damage(0, 500.0);
    assert_eq!(tick(&mut w, &mut s, &mut facts), vec![0, 1], "dies: Death, and the 200 lost");
    assert_eq!(tick(&mut w, &mut s, &mut facts), Vec::<usize>::new(), "once");
    assert_eq!(s.score[1], 1 + 2 + 2);
}

#[test]
fn position_and_speed_are_live_values_else_the_record_and_0() {
    let mut w = world(vec![vehicle([0.0, 0.0, 20.0], flag::CHASE_BRAIN, 300.0)]);
    let f = w.element_facts(0, [0.0; 3]).unwrap();
    assert_eq!((f.pos, f.speed), ([0.0, 0.0, 20.0], 0.0));
    step(&mut w, [0.0; 3]);
    if let LiveKind::Car { car, .. } = &mut w.live[0].kind {
        car.pos = [3.0, 0.0, 40.0];
        car.vel = [3.0, 0.0, 4.0];
    }
    let f = w.element_facts(0, [0.0; 3]).unwrap();
    assert_eq!((f.pos, f.speed), ([3.0, 0.0, 40.0], 5.0));
}

#[test]
fn properties_follow_fun_0012b138() {
    let mut riders = vehicle([0.0, 0.0, 20.0], 0, 300.0);
    riders.riders = vec![rider(30.0), rider(0.0)];
    let mut empty = vehicle([0.0, 0.0, 25.0], 0, 300.0);
    empty.riders = vec![rider(0.0), rider(0.0)];
    let bare = vehicle([0.0, 0.0, 30.0], 0, 300.0);
    let mut hench = vehicle([0.0, 0.0, 35.0], 0, 50.0);
    hench.kind = TYPE_HENCHMAN;
    let mut w = world(vec![riders, empty, bare, hench]);
    let props = |w: &HostileWorld, i: usize| w.element_facts(i, [0.0; 3]).unwrap().props;
    // EMPTY: the riders' +0x14 healths sum to 0 (no riders counts), any state, vehicles only.
    assert_eq!((props(&w, 0), props(&w, 1), props(&w, 2), props(&w, 3)), (vec![], vec!["EMPTY".to_string()], vec!["EMPTY".to_string()], vec![]));
    // AWAKE: live, and a vehicle only while its health is above 0.
    step(&mut w, [0.0; 3]);
    assert_eq!(w.db.elements[2].state, ElemState::Live);
    assert_eq!(props(&w, 2), vec!["EMPTY".to_string(), "AWAKE".to_string()]);
    w.damage(2, 1000.0);
    assert_eq!(props(&w, 2), vec!["EMPTY".to_string()]);
}

#[test]
fn bearing_is_the_signed_angle_from_the_car_heading_to_the_player() {
    let mut w = world(vec![vehicle([0.0, 0.0, 0.0], 0, 300.0)]);
    assert_eq!(w.element_facts(0, [10.0, 0.0, 0.0]).unwrap().bearing_deg, None, "not live: FUN_00209958 has no matrix");
    step(&mut w, [0.0, 0.0, 10.0]);
    if let LiveKind::Car { car, .. } = &mut w.live[0].kind {
        car.pos = [0.0; 3];
        car.rot = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    let b = |p: [f32; 3]| w.element_facts(0, p).unwrap().bearing_deg.unwrap();
    // cross(row 2, d).y = fz·dx − fx·dz: the player on +x reads +90, on −x −90, ahead 0.
    assert!((b([10.0, 0.0, 0.0]) - 90.0).abs() < 1e-3);
    assert!((b([-10.0, 0.0, 0.0]) + 90.0).abs() < 1e-3);
    assert!(b([0.0, 0.0, 10.0]).abs() < 1e-2);
    assert!((b([0.0, 0.0, -10.0]).abs() - 180.0).abs() < 1e-2);
}

#[test]
fn ai_ground_vehicles_walk_the_triggers_as_kind_2() {
    // Sim type 2 (chase brain) and 4 (none): both kind byte 2 [M 0x1862c0].
    let mut w = world(vec![vehicle([0.0, 0.0, 20.0], flag::CHASE_BRAIN, 300.0), vehicle([0.0, 0.0, 30.0], 0, 300.0)]);
    step(&mut w, [0.0; 3]);
    assert_eq!(w.live.iter().map(|v| v.sim_kind).collect::<Vec<_>>(), vec![2, 4]);
    let cars = w.trigger_cars();
    assert_eq!(cars.iter().map(|c| c.kind).collect::<Vec<_>>(), vec![2, 2]);
    // A bit-3 trigger (kind 2, enabled, once) at the second car runs for it; a bit-6 one does not.
    let mut map = MapScript::default();
    let t = |index: usize, flags: u16, n: u32| {
        let mut args = 0x83u32.to_le_bytes().to_vec();
        args.extend_from_slice(&n.to_le_bytes());
        Trigger { index, pos: cars[1].pos, radius: 3.0, kind: 0, shape: 2, flags, height: 1.0, axis: [1.0, 0.0, 0.0], width: 1.0, dir: [0.0, 0.0, 1.0], length: 1.0, actions: vec![TriggerAction { id: 0x83, args }] }
    };
    map.triggers.push(t(0, 0x0b, 1));
    map.triggers.push(t(1, 0x43, 10));
    let mut s = DriveScript::new(map);
    s.step(&Facts { player_pos: [0.0, 0.0, -500.0], player_radius: 2.0, cars: cars.clone(), elements: HashMap::new(), ..Default::default() });
    assert_eq!(s.take_fired(), vec![(0, 2)], "walk slot 2 = cars[1]");
    assert_eq!(s.score[1], 1);
}
