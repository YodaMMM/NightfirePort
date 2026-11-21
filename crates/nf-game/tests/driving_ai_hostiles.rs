//! The driving hostile AI and AI vehicle lifecycle checked against the DRIVING.ELF decompile
//! (docs/systems/driving-ai.md §10): the AI element database FUN_0012b8b0 / FUN_0012afb8 /
//! FUN_0012b440 / FUN_0012bec8 / FUN_0012cb08 / FUN_0012c740, the traffic set FUN_0013fae0, the
//! chase speeds FUN_0012e808, the ram roll and sight rule of FUN_00131770 and the weapons
//! FUN_00130050 / 0x12f730 / 0x12fa28 / 0x12fd58 / 0x12f530. Every expected number is the
//! game's constant or arithmetic; the ISO-gated test lists each mission's AI vehicle set.

use nf_formats::crp::{tag, Crp};
use nf_formats::driveai::{flag, AiElement, AiRider, MapAi, TYPE_HENCHMAN, TYPE_VEHICLE};
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;

use nf_game::driving::ai::DriveRng;
use nf_game::driving::hostile::*;
use nf_game::driving::CarParams;

// ---------------------------------------------------------------- fixtures

fn vehicle(pos: [f32; 3], flags: u16) -> AiElement {
    AiElement {
        facing: [0.0, 0.0, 1.0],
        pos,
        flags,
        difficulty: 0,
        kind: TYPE_VEHICLE,
        state: 0,
        health: 100.0,
        name: "paris_henchbigbox".into(),
        hench_class: 0,
        hench_model: 0,
        hench_weapon: 0,
        ctor_byte: 0,
        heli: false,
        riders: Vec::new(),
        command_ref: 0,
    }
}

fn rider(seat: u32, health: f32) -> AiRider {
    AiRider { seat, model: 0x14, weapon: 1, raw_0c: 0, raw_10: 0, raw_18: 0x1e, health, skill: 3 }
}

fn db(els: Vec<AiElement>, slots: [i32; 3]) -> ElementDb {
    ElementDb::new(&MapAi { elements: els, commands: Vec::new(), ..Default::default() }, slots, 0, 1.0, 0)
}

const FWD: [f32; 3] = [0.0, 0.0, 1.0];

fn spawned(a: &[AiAction]) -> Vec<usize> {
    a.iter().filter_map(|x| if let AiAction::Spawn(s) = x { Some(s.index) } else { None }).collect()
}
fn despawned(a: &[AiAction]) -> Vec<(usize, ElemState)> {
    a.iter().filter_map(|x| if let AiAction::Despawn { index, then } = x { Some((*index, *then)) } else { None }).collect()
}

// ---------------------------------------------------------------- the element database

#[test]
fn difficulty_filter_matches_the_switch() {
    // [M 0x12cb08:113-146] — row = filter, columns = difficulty 0, 1, 2, 3.
    let want: [(u16, [bool; 4]); 8] = [
        (0, [true, true, true, true]),
        (1, [false, true, true, false]),
        (2, [false, false, true, false]),
        (3, [true, false, false, false]),
        (4, [false, true, false, false]),
        (5, [true, true, false, false]),
        (6, [true, false, true, false]),
        (7, [true, true, true, true]),
    ];
    for (f, row) in want {
        for (d, &ok) in row.iter().enumerate() {
            assert_eq!(difficulty_allows(f, d as u32), ok, "filter {f} difficulty {d}");
        }
    }
}

#[test]
fn range_spawn_hysteresis_and_the_growing_period() {
    let mut d = db(vec![vehicle([0.0, 0.0, 150.0], 0)], [32, 6, 16]);
    let player = [0.0; 3];
    let a = d.tick(0, player, FWD, &|_| None);
    assert_eq!(spawned(&a), vec![0], "in range (150 < 200), not held → spawns");
    assert_eq!(d.elements[0].state, ElemState::Live);
    assert_eq!(d.live, [0, 1, 0]);
    // Live at 210: past 200 but inside 220 → stays (hysteresis); the period grows 1 → 2.
    let at = |z: f32| move |_: usize| Some(LiveStatus::at([0.0, 0.0, z], 100.0));
    let a = d.tick(1, player, FWD, &at(210.0));
    assert!(a.is_empty(), "210 keeps the car: {a:?}");
    assert_eq!((d.elements[0].rt.period, d.elements[0].rt.next_check), (2, 3));
    // Not due at tick 2.
    assert!(d.tick(2, player, FWD, &at(300.0)).is_empty());
    // Due at 3, beyond 220 → despawn, respawnable.
    let a = d.tick(3, player, FWD, &at(230.0));
    assert_eq!(despawned(&a), vec![(0, ElemState::Idle)]);
    assert_eq!(d.live, [0, 0, 0]);
    // The period grows to 4 and stays (FUN_0012b0e0).
    let mut now = d.elements[0].rt.next_check;
    let mut periods = vec![d.elements[0].rt.period];
    for _ in 0..3 {
        d.elements[0].rec.pos = [0.0, 0.0, 500.0];
        d.tick(now, player, FWD, &|_| None);
        periods.push(d.elements[0].rt.period);
        now = d.elements[0].rt.next_check;
    }
    assert_eq!(periods, vec![3, 4, 4, 4]);
}

#[test]
fn held_and_persistent() {
    // HELD: never spawned by range, even next to the player.
    let mut d = db(vec![vehicle([0.0, 0.0, 10.0], flag::HELD)], [32, 6, 16]);
    assert!(d.tick(0, [0.0; 3], FWD, &|_| None).is_empty());
    // A live HELD element despawns at its next check (a script set the flag) [M 0x12afb8].
    d.elements[0].rec.flags = 0;
    assert_eq!(spawned(&d.tick(1, [0.0; 3], FWD, &|_| None)), vec![0]);
    d.elements[0].rec.flags = flag::HELD;
    let a = d.tick(2, [0.0; 3], FWD, &|_| Some(LiveStatus::at([0.0, 0.0, 10.0], 100.0)));
    assert_eq!(despawned(&a), vec![(0, ElemState::Idle)]);
    // HELD | PERSISTENT live: the range check keeps it (may_despawn is false).
    let mut d = db(vec![vehicle([0.0, 0.0, 10.0], 0)], [32, 6, 16]);
    d.tick(0, [0.0; 3], FWD, &|_| None);
    d.elements[0].rec.flags = flag::HELD | flag::PERSISTENT;
    assert!(despawned(&d.tick(1, [0.0; 3], FWD, &|_| Some(LiveStatus::at([0.0, 0.0, 10.0], 100.0)))).is_empty());
    // PERSISTENT spawns at any range and is never range-despawned.
    let mut d = db(vec![vehicle([0.0, 0.0, 5000.0], flag::PERSISTENT)], [32, 6, 16]);
    assert_eq!(spawned(&d.tick(0, [0.0; 3], FWD, &|_| None)), vec![0]);
    for t in 1..10 {
        assert!(d.tick(t * 4, [0.0; 3], FWD, &|_| Some(LiveStatus::at([0.0, 0.0, 5000.0], 100.0))).is_empty());
    }
}

#[test]
fn the_priority_key() {
    // key = (dot(n(pos − player), n(fwd)) + 1) · 0.5 / dist, ≥ 0 [M 0x12b440].
    let mut d = db(
        vec![vehicle([0.0, 0.0, 100.0], flag::HELD), vehicle([0.0, 0.0, -50.0], flag::HELD), vehicle([40.0, 0.0, 0.0], flag::HELD)],
        [32, 6, 16],
    );
    d.tick(0, [0.0; 3], FWD, &|_| None);
    assert!((d.elements[0].rt.key - 0.01).abs() < 1e-7, "ahead at 100: {}", d.elements[0].rt.key);
    assert_eq!(d.elements[1].rt.key, 0.0, "straight behind: (−1 + 1)/2");
    assert!((d.elements[2].rt.key - 0.5 / 40.0).abs() < 1e-7, "abeam at 40: (0 + 1)·0.5/40");
}

#[test]
fn over_budget_slots_go_by_priority_persistent_first() {
    // 8 cars in range for 6 vehicle slots: the persistent one (farthest, behind) is reserved
    // first, then the five highest keys (nearest ahead) [M 0x12bec8 passes A/B].
    let mut els: Vec<AiElement> = (0..7).map(|k| vehicle([0.0, 0.0, 20.0 + 20.0 * k as f32], 0)).collect();
    els.push(vehicle([0.0, 0.0, -190.0], flag::PERSISTENT));
    let mut d = db(els, [32, 6, 16]);
    let mut got = spawned(&d.tick(0, [0.0; 3], FWD, &|_| None));
    got.sort();
    assert_eq!(got, vec![0, 1, 2, 3, 4, 7]);
    assert_eq!(d.live[1], 6);
    // The player turns round: element 6 is now nearer ahead than live 4 … but both 5 and 6
    // are idle; the budget is full (6 live), so the pass despawns the lowest key and spawns.
    let live = |i: usize| Some(LiveStatus::at([0.0, 0.0, if i == 7 { -190.0 } else { 20.0 + 20.0 * i as f32 }], 100.0));
    let a = d.tick(1, [0.0, 0.0, 200.0], [0.0, 0.0, -1.0], &live);
    let mut sp = spawned(&a);
    sp.sort();
    let mut dp: Vec<usize> = despawned(&a).iter().map(|x| x.0).collect();
    dp.sort();
    // From z=200 facing −z: keys favour the cars at 140/120 (ahead, near); element 0 (z 20,
    // dist 180) and 1 (dist 160) rank below 5 (dist 80) and 6 (dist 60).
    assert_eq!(sp, vec![5, 6]);
    assert_eq!(dp, vec![0, 1]);
    assert_eq!(d.live[1], 6);
}

#[test]
fn riders_need_hench_slots() {
    let mut a = vehicle([0.0, 0.0, 10.0], 0);
    a.riders = vec![rider(3, 50.0), rider(4, 20.0)];
    let mut b = vehicle([0.0, 0.0, 30.0], 0);
    b.riders = vec![rider(3, 50.0)];
    // 2 hench slots, 2 vehicle slots: the wanted riders (3) do not fit → priority pass: a
    // (key higher) takes both hench slots, b's rider no longer fits (n ≤ h fails).
    let mut d = db(vec![a, b], [2, 2, 16]);
    let act = d.tick(0, [0.0; 3], FWD, &|_| None);
    assert_eq!(spawned(&act), vec![0]);
    assert_eq!(d.live, [2, 1, 0]);
    if let AiAction::Spawn(s) = &act[0] {
        assert_eq!(s.riders, vec![0, 1]);
    }
}

#[test]
fn spawn_kind_counter_and_traffic_clear() {
    let mut a = vehicle([0.0, 0.0, 10.0], flag::CHASE_BRAIN);
    a.riders = vec![rider(3, 50.0), rider(0x7, 10.0), rider(4, 10.0)];
    let b = vehicle([0.0, 0.0, 20.0], flag::UNCOUNTED);
    let mut d = db(vec![a, b], [32, 6, 16]);
    let act = d.tick(0, [0.0; 3], FWD, &|_| None);
    let s: Vec<&Spawn> = act.iter().filter_map(|x| if let AiAction::Spawn(s) = x { Some(s) } else { None }).collect();
    assert_eq!(s[0].sim_kind, 2, "flag 8 → sim kind 2 (the chase AIVehicle)");
    assert_eq!(s[1].sim_kind, 4, "no flag 8 → kind 4");
    assert_eq!(s[0].clear_traffic_pad, Some(6.0));
    // The unknown seat 7 ends the seating: only rider 0 sits [M switch default → return].
    assert_eq!(s[0].riders, vec![0]);
    // Kill counter: 1 + 3 riders for a, nothing for the UNCOUNTED b.
    assert_eq!(d.kill_count, 4);
    assert!(d.elements[0].rec.flags & flag::COUNTED != 0);
    // A respawn does not count again.
    d.despawn(0, Some(LiveStatus::at([0.0, 0.0, 10.0], 100.0)));
    d.spawn(0);
    assert_eq!(d.kill_count, 4);
}

#[test]
fn despawn_verdicts() {
    let mk = |flags| {
        let mut d = db(vec![vehicle([0.0, 0.0, 10.0], flags)], [32, 6, 16]);
        d.tick(0, [0.0; 3], FWD, &|_| None);
        d
    };
    let l = LiveStatus::at([5.0, 0.0, 15.0], 100.0);
    assert_eq!(mk(0).despawn(0, Some(l)), AiAction::Despawn { index: 0, then: ElemState::Idle });
    assert_eq!(mk(0).despawn(0, Some(LiveStatus { health: 0.0, ..l })), AiAction::Despawn { index: 0, then: ElemState::Gone });
    assert_eq!(mk(0).despawn(0, Some(LiveStatus { wheels: 3, ..l })), AiAction::Despawn { index: 0, then: ElemState::Gone });
    assert_eq!(mk(0).despawn(0, Some(LiveStatus { vehicle_3e8: true, ..l })), AiAction::Despawn { index: 0, then: ElemState::Gone });
    // KEEP_PLACE copies the live position back; without it the record keeps its spawn point.
    let mut d = mk(flag::KEEP_PLACE);
    d.despawn(0, Some(l));
    assert_eq!(d.elements[0].rec.pos, [5.0, 0.0, 15.0]);
    let mut d = mk(0);
    d.despawn(0, Some(l));
    assert_eq!(d.elements[0].rec.pos, [0.0, 0.0, 10.0]);
    // A gone element is never checked or spawned again.
    let mut d = mk(0);
    d.despawn(0, Some(LiveStatus { health: 0.0, ..l }));
    assert!(d.tick(5, [0.0; 3], FWD, &|_| None).is_empty());
}

#[test]
fn henchmen_need_a_known_class_and_the_character_system() {
    let mut h = vehicle([0.0, 0.0, 10.0], 0);
    h.kind = TYPE_HENCHMAN;
    for (class, ok) in [(0, true), (10, true), (0x12, true), (5, false)] {
        let mut e = h.clone();
        e.hench_class = class;
        let mut d = db(vec![e], [32, 6, 16]);
        assert_eq!(!spawned(&d.tick(0, [0.0; 3], FWD, &|_| None)).is_empty(), ok, "class {class}");
    }
    let mut d = db(vec![h], [32, 6, 16]);
    d.characters_on = false;
    assert!(d.tick(0, [0.0; 3], FWD, &|_| None).is_empty());
}

#[test]
fn health_scales_at_load_and_the_near_flag() {
    let mut v = vehicle([0.0, 0.0, 119.0], 0);
    v.riders = vec![rider(3, 50.0)];
    let d = ElementDb::new(&MapAi { elements: vec![v.clone()], commands: vec![], ..Default::default() }, [32, 6, 16], 1, 1.5, 0);
    assert_eq!(d.elements[0].rec.health, 150.0);
    assert_eq!(d.elements[0].rec.riders[0].health, 75.0);
    let mut d = db(vec![v], [32, 6, 16]);
    d.tick(0, [0.0; 3], FWD, &|_| None);
    assert!(d.clear_of_hostiles, "nothing live was measured at tick 0");
    d.tick(1, [0.0; 3], FWD, &|_| Some(LiveStatus::at([0.0, 0.0, 119.0], 10.0)));
    assert!(!d.clear_of_hostiles, "a live hostile at 119 (< 120)");
    d.tick(2, [0.0; 3], FWD, &|_| Some(LiveStatus::at([0.0, 0.0, 121.0], 10.0)));
    assert!(d.clear_of_hostiles);
}

// ---------------------------------------------------------------- traffic set + car params

#[test]
fn traffic_set_and_world_slots() {
    let paris = WorldAttr::parse("[paris]\nACT_ANIM_BANK=paris0.txt\nMAX_TRAFFIC=20\nNUM_TRAFFIC_TYPES_PS2=3\n");
    let jungle = WorldAttr::parse("[jungle]\nMAX_TRAFFIC=0\nVEHICLE_SLOTS_AVAIL=15\n\n[junglea_mis13a]\nCHAIN_NEXT_MISSION=4\n");
    assert_eq!(jungle.slots(), [32, 15, 16]);
    assert_eq!(paris.slots(), [32, 6, 16]);
    let colours = |_: &str| 14;
    let mut r = DriveRng::game_reset();
    let mut check = r.clone();
    let set = traffic_types(&paris, &mut r, &colours);
    assert_eq!(set.len(), 20);
    // Three slots drawn as rand·7 >> 16, then one colour draw per car: 14·rand >> 16.
    let slots: Vec<usize> = (0..3).map(|_| ((check.next() as u32 * 7) >> 16) as usize).collect();
    for (i, (name, col)) in set.iter().enumerate() {
        assert_eq!(*name, TRAFFIC_TYPE_NAMES[slots[i % 3]]);
        assert_eq!(*col, (14 * check.next() as i32) >> 16);
    }
    assert!(traffic_types(&jungle, &mut r, &colours).is_empty());
    // No keys: 20 cars, 7 types in table order, no slot draws.
    let mut r = DriveRng::game_reset();
    let set = traffic_types(&WorldAttr::default(), &mut r, &|_| 0);
    assert_eq!(set.iter().map(|x| x.0).take(8).collect::<Vec<_>>(), vec!["coupe", "sedan2", "hatchback", "sedan", "station_wagon", "suv1", "taxi", "coupe"]);
    assert!(set.iter().all(|x| x.1 == 0), "colour count 0 → 1 → (1·rand) >> 16 = 0");
}

#[test]
fn ai_cars_use_their_own_atr() {
    let mut set = AiVehicleSet::default();
    set.insert("hatchback", "[hatchback]\nMAXSTEERING=0.2 \nMAXACC=13.29      \nMINSTEER=1.0 \n");
    let p = set.params("hatchback").unwrap();
    assert_eq!((p.maxsteering, p.maxacc, p.min_steer), (0.2, 13.29, 1.0));
    assert_ne!(p.maxsteering, CarParams::vanquish().maxsteering, "not the player's 0.09");
}

// ---------------------------------------------------------------- chase speeds

fn chase() -> ChaseSpeedInputs {
    ChaseSpeedInputs {
        state: 5,
        profile: PROFILE_PURSUE,
        pattern: 3,
        alive: 1.0,
        stop_query: false,
        stop_byte: false,
        self_pos: [0.0; 3],
        self_vel: [0.0, 0.0, 20.0],
        self_forward: [0.0, 0.0, 1.0],
        self_box: 2.0,
        has_target: true,
        target_is_player: false,
        target_vel: [0.0, 0.0, 40.0],
        target_box: 2.0,
        aim: [0.0, 0.0, 100.0],
        dist: 100.0,
        dir: [0.0, 0.0, 1.0],
        dir_210: [0.0, 0.0, 1.0],
        follow: 15.0,
        catchup: 1.1,
        cap: 60.0,
        speed_mul: 1.0,
        ramming: false,
        lost: false,
        element_target: false,
        follow_target: false,
        spawn_delay: 0,
        seg_limit: 20.0,
        speed_factor: 1.0,
    }
}

#[test]
fn pursue_speeds() {
    let c = chase();
    // Far (100 > 2·15): 40 · min(1.1, 100/15·0.75) = 44, ≥ 30, × 1.1 = 48.4, cap 60.
    assert!((chase_target_speed(&c).speed - 48.4).abs() < 1e-4);
    // Cap.
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { cap: 40.0, ..c }).speed, 40.0);
    // Between follow and 2·follow, not ramming: no second catch-up: 40·min(1.1, 20/15·0.75=1.0) = 40.
    assert!((chase_target_speed(&ChaseSpeedInputs { dist: 20.0, ..c }).speed - 40.0).abs() < 1e-4);
    // Inside follow: target moving away (dot > 0) → 0.8·|v_t| = 32; toward → 0.
    assert!((chase_target_speed(&ChaseSpeedInputs { dist: 10.0, ..c }).speed - 32.0).abs() < 1e-4);
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { dist: 10.0, target_vel: [0.0, 0.0, -40.0], ..c }).speed, 0.0);
    // Ramming inside follow: max(40·1.1, 30) = 44.
    assert!((chase_target_speed(&ChaseSpeedInputs { dist: 10.0, ramming: true, ..c }).speed - 44.0).abs() < 1e-4);
    // A slow target far away: the 30 floor.
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { target_vel: [0.0, 0.0, 5.0], ..c }).speed, 30.0 * 1.1);
    // The player target: follow grows to 1.25·(2+2) = 5 … follow 15 stays; a ram ends inside 5.
    let r = chase_target_speed(&ChaseSpeedInputs { target_is_player: true, ramming: true, dist: 4.0, follow: 3.0, ..c });
    assert!(r.ram_cleared);
    // Pattern 4 not aligned with its own velocity → 0.6·|v_t|.
    let r = chase_target_speed(&ChaseSpeedInputs { pattern: 4, self_vel: [20.0, 0.0, 0.0], ..c });
    assert!((r.speed - 24.0).abs() < 1e-4);
    // Lost: max(|v_t|·catchup, 25) / 25 with no target; the element-target cap × speed factor.
    assert!((chase_target_speed(&ChaseSpeedInputs { lost: true, ..c }).speed - 44.0).abs() < 1e-4);
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { lost: true, has_target: false, ..c }).speed, 25.0);
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { element_target: true, speed_factor: 0.5, ..c }).speed, 30.0);
    // Gates.
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { alive: 0.0, ..c }).speed, 0.0);
}

#[test]
fn escort_wait_and_road_speeds() {
    let c = ChaseSpeedInputs { profile: PROFILE_ESCORT, ..chase() };
    assert!((chase_target_speed(&c).speed - 44.0).abs() < 1e-4, "facing the target → |v_t|·catchup");
    let behind = ChaseSpeedInputs { self_forward: [0.0, 0.0, -1.0], ..c };
    assert!((chase_target_speed(&behind).speed - 32.0).abs() < 1e-4, "facing away → 0.8·|v_t|");
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { lost: true, ..c }).speed, 25.0);
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { target_vel: [0.0; 3], ..c }).speed, 10.0, "10 floor");
    // Wait: 2.5 arrival, cap beyond max(follow, 10), else dist·0.5.
    let w = ChaseSpeedInputs { profile: PROFILE_WAIT, aim: [0.0, 0.0, 2.0], ..chase() };
    assert_eq!(chase_target_speed(&w).speed, 0.0);
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { aim: [0.0, 0.0, 30.0], ..w }).speed, 60.0);
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { aim: [0.0, 0.0, 12.0], dist: 12.0, ..w }).speed, 6.0);
    // Road: the segment limit × +0x134 without +0x158; with it the 1.1 / 0.85 / 0.75 rules.
    let r = ChaseSpeedInputs { profile: PROFILE_ROAD, speed_mul: 0.5, ..chase() };
    assert_eq!(chase_target_speed(&r).speed, 10.0);
    let f = ChaseSpeedInputs { follow_target: true, speed_mul: 1.0, ..r };
    assert!((chase_target_speed(&f).speed - 44.0).abs() < 1e-4, "max(20, 40·1.1)");
    let back = ChaseSpeedInputs { dir_210: [0.0, 0.0, -1.0], ..f };
    assert_eq!(chase_target_speed(&back).speed, 15.0, "dist ≥ 2·follow → 20·0.75");
    let mid = ChaseSpeedInputs { dist: 20.0, ..back };
    assert!((chase_target_speed(&mid).speed - 34.0).abs() < 1e-4, "max(20·0.85, 40·0.85)");
    assert_eq!(chase_target_speed(&ChaseSpeedInputs { spawn_delay: 3, ..f }).speed, 0.0);
}

// ---------------------------------------------------------------- ram + sight

#[test]
fn the_ram_roll() {
    let fps = 60;
    let mut st = RamState::new(fps);
    assert_eq!(st.roll, 300);
    let mut rng = DriveRng::game_reset();
    let before = rng.clone();
    for _ in 0..299 {
        assert!(!ram_tick(&mut st, false, false, true, false, 1.0, fps, &mut rng));
    }
    assert_eq!(rng.draws, before.draws, "no draw before the 5 s roll");
    assert!(ram_tick(&mut st, false, false, true, false, 1.0, fps, &mut rng), "chance 1.0 always rams");
    assert_eq!((st.ram, st.roll), (180, 300), "3 s ram, re-armed roll");
    st.roll = 1;
    assert!(ram_tick(&mut st, false, false, true, true, 1.0, fps, &mut rng));
    assert_eq!(st.ram, 60, "1 s against the player");
    // Chance 0: rolls, never rams; a lost target holds the roll at 0 without drawing.
    st.roll = 1;
    assert!(!ram_tick(&mut st, false, false, true, false, 0.0, fps, &mut rng));
    assert_eq!(st.roll, 300);
    st.roll = 1;
    let d = rng.draws;
    assert!(!ram_tick(&mut st, false, true, true, false, 1.0, fps, &mut rng));
    assert_eq!((st.roll, rng.draws), (0, d));
    // The draw is compared as rand·(1/65536): the exact threshold.
    let mut probe = rng.clone();
    let v = probe.next() as f32 * RAND_SCALE;
    st.roll = 1;
    assert!(!ram_tick(&mut st, false, false, true, false, v, fps, &mut rng), "strict <");
}

#[test]
fn the_sight_rule() {
    // +0x174 starts at 0 [M 0x12ded8]: the first test runs at tick 15 (60·0.25); until then
    // the car counts as lost (+0x1cc = 0 → +0x208 = 1).
    let mut s = SightState::default();
    assert!(sight_tick(&mut s, 14, 60, false, true, 0, 150.0, 1));
    assert!(!sight_tick(&mut s, 15, 60, false, true, 0, 150.0, 1), "150 with one hit is seen");
    // The next test is 15 ticks later; before that a lost sight is not noticed.
    assert!(!sight_tick(&mut s, 29, 60, false, true, 0, 250.0, 0));
    assert!(sight_tick(&mut s, 30, 60, false, true, 0, 250.0, 0), "beyond 200 → lost");
    assert!(sight_tick(&mut s, 45, 60, false, true, 0, 100.0, 2), "2 ray hits → lost");
    assert!(!sight_tick(&mut s, 60, 60, false, true, 0, 200.0, 0), "200 is inside");
    assert!(s.ever_seen);
    // A navigator repeat (+0x94) skips the test.
    assert!(!sight_tick(&mut s, 75, 60, true, true, 0, 500.0, 0));
    let mut s = SightState::default();
    assert!(!sight_tick(&mut s, 15, 60, false, false, 1, 0.0, 0), "drive-to-point mode is always seen");
    assert!(can_see(200.0, 1) && !can_see(200.01, 0) && !can_see(10.0, 2));
}

// ---------------------------------------------------------------- weapons

fn fire() -> FireInputs {
    FireInputs {
        alive: 1.0,
        stop_byte: false,
        weapons_on: true,
        has_target: true,
        lost: false,
        seen: true,
        profile: PROFILE_PURSUE,
        turret: false,
        turret_on_target: false,
        guns: true,
        rockets: false,
        missiles: false,
        aim_dot: 0.995,
        line_clear: true,
        rate: 2.0,
        fps: 60,
        rocket_busy: [false; 2],
        missile_busy: [false; 2],
    }
}

#[test]
fn machine_guns() {
    let mut w = WeaponState::default();
    let i = fire();
    let s = fire_step(&mut w, &i);
    assert!(s.gun && s.gun_flash == Some(true));
    assert_eq!(w.gun_cd, 120, "round(60·2.0)");
    let mut n = 1;
    while !fire_step(&mut w, &i).gun {
        n += 1;
    }
    assert_eq!(n, 121, "120 cooldown steps, the burst on the 121st call");
    // Cone 0.99 is strict; outside it the flash words drop.
    let s = fire_step(&mut w, &FireInputs { aim_dot: 0.99, ..i });
    assert_eq!((s.gun, s.gun_flash), (false, Some(false)));
    // A blocked line, a lost target, a negative rate.
    assert!(!fire_step(&mut WeaponState::default(), &FireInputs { line_clear: false, ..i }).gun);
    assert_eq!(fire_step(&mut WeaponState::default(), &FireInputs { lost: true, ..i }), Shots::default());
    let s = fire_step(&mut WeaponState::default(), &FireInputs { rate: -1.0, ..i });
    assert_eq!((s.gun, s.gun_flash), (false, Some(true)), "rate < 0 aims but never fires");
    // No floor on the gun reload: rate 0.1 → 6 ticks.
    let mut w = WeaponState::default();
    fire_step(&mut w, &FireInputs { rate: 0.1, ..i });
    assert_eq!(w.gun_cd, 6);
    // A turret car fires on its own aim, not the cone.
    let t = FireInputs { turret: true, turret_on_target: true, aim_dot: 0.0, line_clear: false, ..i };
    assert!(fire_step(&mut WeaponState::default(), &t).gun);
}

#[test]
fn rockets_and_missiles() {
    let r = FireInputs { guns: false, rockets: true, aim_dot: 0.95, rate: 0.1, ..fire() };
    let mut w = WeaponState::default();
    assert_eq!(fire_step(&mut w, &r).rocket, Some(6));
    assert_eq!(w.rocket_cd, 30, "the 0.5 s floor: max(6, 30)");
    w.rocket_cd = 0;
    assert_eq!(fire_step(&mut w, &r).rocket, Some(5), "the launchers alternate");
    w.rocket_cd = 0;
    // A launcher still animating holds the shot and the side.
    assert_eq!(fire_step(&mut w, &FireInputs { rocket_busy: [true, false], ..r }).rocket, None);
    assert_eq!((w.rocket_side, w.rocket_cd), (0, 0));
    assert_eq!(fire_step(&mut w, &FireInputs { aim_dot: 0.9, ..r }).rocket, None, "cone 0.9 strict");
    // Missiles: cone 0.707, channels 8 then 7, rate 2.0 → 120.
    let m = FireInputs { guns: false, missiles: true, aim_dot: 0.71, ..fire() };
    let mut w = WeaponState::default();
    assert_eq!(fire_step(&mut w, &m).missile, Some(8));
    assert_eq!(w.missile_cd, 120);
    w.missile_cd = 0;
    assert_eq!(fire_step(&mut w, &m).missile, Some(7));
    assert_eq!(fire_step(&mut WeaponState::default(), &FireInputs { aim_dot: 0.707, ..m }).missile, None);
    // Profile 10 fires rockets even when the per-weapon flags say guns only; 9 the reverse.
    let p10 = FireInputs { profile: PROFILE_ROCKETS_ONLY, aim_dot: 0.995, ..fire() };
    let s = fire_step(&mut WeaponState::default(), &p10);
    assert_eq!((s.gun, s.rocket), (false, Some(6)));
    let p9 = FireInputs { profile: PROFILE_GUNS_ONLY, guns: false, rockets: true, ..fire() };
    let s = fire_step(&mut WeaponState::default(), &p9);
    assert_eq!((s.gun, s.rocket), (true, None));
    // The dispatch gates.
    assert_eq!(fire_step(&mut WeaponState::default(), &FireInputs { weapons_on: false, ..fire() }), Shots::default());
    assert_eq!(fire_step(&mut WeaponState::default(), &FireInputs { alive: 0.0, ..fire() }), Shots::default());
}

#[test]
fn the_line_of_fire() {
    let car = |id, z: f32, state| LofCar { id, pos: [0.0, 0.0, z], sim_6d: 0, sim_6e: 2, chase_state: state };
    let cars = [car(1, 0.0, true), car(2, 40.0, true)];
    assert!(!line_clear(60.0, 1, [0.0, 0.0, 1.0], [0.0; 3], &cars), "a fellow chaser 40 ahead blocks a 60 shot");
    assert!(line_clear(30.0, 1, [0.0, 0.0, 1.0], [0.0; 3], &cars), "not nearer than the target");
    assert!(line_clear(60.0, 1, [0.0, 0.0, 1.0], [0.0; 3], &[car(1, 0.0, true), car(2, 40.0, false)]), "not a chaser");
    assert!(line_clear(60.0, 1, [0.0, 0.0, 1.0], [0.0; 3], &[LofCar { sim_6e: 3, ..car(2, 40.0, true) }]), "traffic never blocks");
    assert!(line_clear(60.0, 1, [0.2, 0.0, 0.98], [0.0; 3], &cars), "outside the 0.99 cone");
}

// ---------------------------------------------------------------- the real disc (NF_ISO)

/// Each mission archive's AIEl/AICo counts [D].
const COUNTS: [(&str, usize, usize); 8] = [
    ("MIS01.VIV", 25, 61),
    ("MIS11.VIV", 45, 68),
    ("MIS13A.VIV", 28, 16),
    ("MIS13B.VIV", 37, 24),
    ("MIS13C.VIV", 43, 35),
    ("MIS3.VIV", 63, 61),
    ("MIS4.VIV", 33, 37),
    ("RACE.VIV", 3, 7),
];

#[test]
fn real_mission_ai_vehicle_sets() {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return;
    };
    let iso = IsoImage::open(&path).unwrap();
    let mut seen = 0;
    for e in iso.entries().iter().filter(|e| e.path.starts_with("/DRIVING/") && e.path.ends_with(".VIV")) {
        let vname = e.path.rsplit('/').next().unwrap();
        let Some(&(_, n_el, n_co)) = COUNTS.iter().find(|c| c.0 == vname) else { continue };
        let bigf = nf_formats::disc::read_bigf(&iso, e).unwrap();
        let base = e.lba as u64 * 2048;
        let read = |want: &dyn Fn(&str) -> bool| -> Vec<(String, Vec<u8>)> {
            bigf.entries
                .iter()
                .filter(|x| want(&x.name.to_ascii_lowercase()))
                .map(|x| {
                    let raw = iso.read_bytes(base + x.offset as u64, x.size as u64).unwrap();
                    let img = if refpack::is_refpack(&raw) { refpack::decompress(&raw).unwrap() } else { raw };
                    (x.name.to_ascii_lowercase(), img)
                })
                .collect()
        };
        let track = read(&|n| n.contains("track") && n.ends_with(".crp"));
        let crp = Crp::parse(&track[0].1).unwrap();
        let ai = MapAi::from_crp(&crp).unwrap();
        let co = ai.commands.len();
        assert_eq!(ai.elements.len(), n_el, "{vname}: AIEl elements");
        assert_eq!(co, n_co, "{vname}: AICo commands");
        let atrs = read(&|n| n.contains("\\pvehicle\\") && n.ends_with(".atr"));
        let world = read(&|n| n.contains("\\world\\") && n.ends_with(".atr") && !n.ends_with("default.atr"));
        let w = WorldAttr::parse(std::str::from_utf8(&world[0].1).unwrap());
        let set = ai.vehicle_set();
        let mut line = Vec::new();
        for name in &set {
            let file = format!("\\pvehicle\\{}.atr", name.to_ascii_lowercase());
            let (_, text) = atrs.iter().find(|(n, _)| n.ends_with(&file)).unwrap_or_else(|| panic!("{vname}: no .atr for {name}"));
            let text = String::from_utf8_lossy(text);
            let p = CarParams::parse_atr(&text);
            // Every rider seat the mission uses on this car is declared in the car's .atr.
            for el in ai.elements.iter().filter(|x| &x.name == name) {
                for r in &el.riders {
                    if let Some(seat) = r.seat_attr() {
                        assert!(text.contains(seat), "{vname}: {name} lacks {seat}");
                    }
                }
            }
            let n = ai.elements.iter().filter(|x| &x.name == name).count();
            line.push(format!("{name}×{n} (steer {}, acc {})", p.maxsteering, p.maxacc));
        }
        let hench = ai.elements.iter().filter(|x| x.kind == TYPE_HENCHMAN).count();
        let held = ai.elements.iter().filter(|x| x.flags & flag::HELD != 0).count();
        let chase = ai.elements.iter().filter(|x| x.kind == TYPE_VEHICLE && x.flags & flag::CHASE_BRAIN != 0).count();
        eprintln!(
            "{vname}: {} elements ({hench} henchmen, {held} held, {chase} chase-brain), {co} commands, slots {:?}, traffic {:?}: {}",
            ai.elements.len(),
            w.slots(),
            w.max_traffic,
            line.join(", ")
        );
        // The MSet difficulty factors FUN_002084a8 / FUN_00208468 read (health +0x2c4.., speed +0x2b8..).
        if let Some(map) = crp.pool_find(tag::MAP).unwrap() {
            for r in crp.record_window(&map).unwrap() {
                if &r.tag_bytes() == b"MSet" {
                    let p = crp.payload(&r).unwrap();
                    let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                    let speed = [f(0x2b8), f(0x2bc), f(0x2c0)];
                    let health = [f(0x2c4), f(0x2c8), f(0x2cc)];
                    eprintln!("  MSet speed factors {speed:?}, health factors {health:?}");
                    assert_eq!(speed, [1.0; 3], "{vname}: no track scales the chase speed by difficulty");
                    if vname == "MIS01.VIV" {
                        assert_eq!(health, [1.0, 1.5, 2.0]);
                    }
                }
            }
        }
        if vname == "MIS01.VIV" {
            assert_eq!(set, vec!["paris_henchbigbox", "paris_bombvan", "paradis_car", "courier_truck", "road_block_truck"]);
            assert_eq!(w.slots(), [32, 6, 16]);
            assert_eq!((w.max_traffic, w.traffic_types), (Some(20), Some(3)));
            // Civilian traffic cars carry their own steering, not the player's.
            let hb = atrs.iter().find(|(n, _)| n.ends_with("\\hatchback.atr")).unwrap();
            let hb = CarParams::parse_atr(&String::from_utf8_lossy(&hb.1));
            assert_eq!(hb.maxsteering, 0.2);
            let vq = atrs.iter().find(|(n, _)| n.ends_with("\\vanquish.atr")).unwrap();
            assert_eq!(CarParams::parse_atr(&String::from_utf8_lossy(&vq.1)).maxsteering, 0.09);
            // At mission start nothing near the player is free to spawn except the
            // persistent, unheld elements: run the manager once from far away.
            let mut db = ElementDb::new(&ai, w.slots(), 0, 1.0, 0);
            let a = db.tick(0, [1.0e5, 0.0, 1.0e5], [0.0, 0.0, 1.0], &|_| None);
            let free: Vec<usize> = (0..ai.elements.len())
                .filter(|&i| ai.elements[i].flags & (flag::HELD | flag::PERSISTENT) == flag::PERSISTENT)
                .collect();
            let mut got: Vec<usize> = a.iter().filter_map(|x| if let AiAction::Spawn(s) = x { Some(s.index) } else { None }).collect();
            got.sort();
            assert_eq!(got, free, "MIS01: the unheld persistent elements spawn at once");
        }
        if vname.starts_with("MIS13") {
            assert_eq!(w.vehicle_slots, Some(15), "{vname}: the jungle world raises the vehicle slots");
        }
        seen += 1;
    }
    assert_eq!(seen, 8, "all eight mission archives");
}
