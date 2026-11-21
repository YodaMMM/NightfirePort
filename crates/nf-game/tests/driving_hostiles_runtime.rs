//! The hostile runtime `driving::hostiles::HostileWorld` (docs/systems/driving-ai.md §13): the
//! element database's spawns and despawns becoming live vehicles, the spawn command, the chase
//! brain driving a `Car`, the shots resolved into damage calls, the draw list and the model list.
//! The rules are the ported functions' (hostile.rs, patterns.rs, weapons.rs, riders.rs); these
//! tests check the wiring. The last test runs MIS01 and MIS4 from the disc (NF_ISO).

use nf_formats::crp::Crp;
use nf_formats::driveai::{flag, AiCommand, AiElement, AiRider, MapAi, AICO_RECORD, TYPE_HENCHMAN, TYPE_VEHICLE};
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;

use nf_game::driving::ai::{DriveRng, RoadNav, NAV_KIND_TRAFFIC};
use nf_game::driving::hostile::{AiAction, ElemState, WorldAttr, ACTIVATE_RADIUS, DEACTIVATE_RADIUS};
use nf_game::driving::hostiles::*;
use nf_game::driving::road::{LaneSet, RoadNet};
use nf_game::driving::weapons::{GUN, MISSILE, ROCKET, WEAPONS};
use nf_game::driving::{CollisionGround, FlatGround, FlatWorld, DT};

// ---------------------------------------------------------------- fixtures

const SUPERSNOW: &str = "[supersnow]\nRENDER_FILENAME.s=supersnow.crp\nRENDER_NUMCOLORS.u=8\nAIC_EVGUN_POS.v=+0.0000,+0.3700,-1.3100,1.0\nHAS_ROCKETS=1\nHAS_MACHINEGUNS=1\nHAS_TURRET=1\n";
const BIGBOX: &str = "[paris_henchbigbox]\nRENDER_FILENAME.s=paris_henchbigbox.crp\nAIC_EWIND_POS.v=+0.3830,-0.5540,-0.6530,1.0\nAIC_ESUNR_POS.v=+0.0000,-0.5600,-0.1000,1.0\n";
const GUNCAR: &str = "[guncar]\nRENDER_FILENAME.s=guncar.crp\nHAS_MACHINEGUNS=1\nHAS_ROCKETS=1\n";

fn vehicle(name: &str, pos: [f32; 3], flags: u16, command_ref: u32) -> AiElement {
    AiElement {
        facing: [0.0, 0.0, 1.0],
        pos,
        flags,
        difficulty: 0,
        kind: TYPE_VEHICLE,
        state: 0,
        health: 500.0,
        name: name.into(),
        hench_class: 0,
        hench_model: 0,
        hench_weapon: 0,
        ctor_byte: 0,
        heli: false,
        riders: Vec::new(),
        command_ref,
    }
}

/// An AICo record: chase the player (mode 0), pattern 2, profile 0xd, follow 15, catch-up 1.2,
/// cap 30, `rate` s between shots, guns / rockets bytes.
fn command(rate: f32, guns: bool, rockets: bool) -> AiCommand {
    let mut b = vec![0u8; AICO_RECORD];
    b[0x4c..0x50].copy_from_slice(&15.0f32.to_le_bytes());
    b[0x5c..0x60].copy_from_slice(&1.2f32.to_le_bytes());
    b[0x60..0x64].copy_from_slice(&30.0f32.to_le_bytes());
    b[0x64..0x68].copy_from_slice(&rate.to_le_bytes());
    b[0x78] = 0;
    b[0x79] = 2;
    b[0x7a] = 0xd;
    b[0x87] = guns as u8;
    b[0x90] = rockets as u8;
    b[0x93] = 100; // aim skill 100: every shell takes the exact aim
    AiCommand::parse(&b).unwrap()
}

/// A map with `els` and one command record named by `sr#1` → `{AICo}::0`.
fn map(els: Vec<AiElement>, cmd: AiCommand) -> MapAi {
    let mut m = MapAi { elements: els, commands: vec![cmd], ..Default::default() };
    m.refs.insert(1, "CARP::<<Map>>::{AICo}::0".into());
    m
}

const CMD: u32 = 0x7372_0001;

fn world(m: &MapAi, attrs: &[(&str, &str)], slots: [i32; 3]) -> HostileWorld {
    let w = WorldAttr { hench_slots: Some(slots[0]), vehicle_slots: Some(slots[1]), heli_slots: Some(slots[2]), ..Default::default() };
    HostileWorld::new(m, attrs.iter().map(|(n, t)| VehicleAttrs::parse(n, t)).collect(), &w, 0, 1.0, 1.0, None)
}

fn player(pos: [f32; 3]) -> PlayerState {
    PlayerState { pos, vel: [0.0; 3], fwd: [0.0, 0.0, 1.0], box_z: 2.3 }
}

fn no_ray(_: [f32; 3], _: [f32; 3]) -> u32 {
    0
}

fn spawned(a: &[AiAction]) -> Vec<usize> {
    a.iter().filter_map(|x| if let AiAction::Spawn(s) = x { Some(s.index) } else { None }).collect()
}

// ---------------------------------------------------------------- attributes and models

#[test]
fn vehicle_attributes_from_the_atr() {
    let a = VehicleAttrs::parse("supersnow", SUPERSNOW);
    assert_eq!(a.model(), "supersnow");
    assert!(a.machine_guns && a.rockets && a.turret && !a.missiles && !a.cannons && !a.is_heli);
    assert_eq!(a.seat("AIC_EVGUN_POS"), Some([0.0, 0.37, -1.31]));
    let b = VehicleAttrs::parse("paris_henchbigbox", BIGBOX);
    assert!(!b.machine_guns && !b.rockets, "the big box carries no vehicle weapon; its riders shoot");
    assert_eq!(b.seat("AIC_ESUNR_POS"), Some([0.0, -0.56, -0.1]));
    // No RENDER_FILENAME: the vehicle name stands in.
    assert_eq!(VehicleAttrs::parse("x", "[x]\nMASS=3\n").model(), "x");
}

#[test]
fn models_needed_lists_every_vehicle_once() {
    let m = map(
        vec![vehicle("supersnow", [0.0; 3], 0, 0), vehicle("paris_henchbigbox", [5.0, 0.0, 0.0], 0, 0), vehicle("supersnow", [9.0, 0.0, 0.0], 0, 0), vehicle("plane", [0.0; 3], 0, 0)],
        command(1.0, false, false),
    );
    let set = [VehicleAttrs::parse("supersnow", SUPERSNOW), VehicleAttrs::parse("paris_henchbigbox", BIGBOX)];
    let n = models_needed(&m, &set);
    assert_eq!(n.iter().map(|x| (x.vehicle.as_str(), x.model.as_str(), x.missing_atr)).collect::<Vec<_>>(), vec![
        ("supersnow", "supersnow", false),
        ("paris_henchbigbox", "paris_henchbigbox", false),
        ("plane", "plane", true),
    ]);
}

#[test]
fn channels_map_to_the_weapon_barrels() {
    // 0x33a0b0 [ELF data]: gun 9, rockets 5/6, missiles 7/8 — the channels the AI fire
    // functions play [M 0x12f730 / 0x12fa28 / 0x12fd58].
    assert_eq!(weapon_of_channel(9), Some(GUN));
    assert_eq!((weapon_of_channel(5), weapon_of_channel(6)), (Some(ROCKET), Some(ROCKET)));
    assert_eq!((weapon_of_channel(7), weapon_of_channel(8)), (Some(MISSILE), Some(MISSILE)));
    assert_eq!(weapon_of_channel(0x48), Some(16), "the cannon barrel");
    assert_eq!(weapon_of_channel(1), None);
}

// ---------------------------------------------------------------- lifecycle

#[test]
fn spawns_inside_200_despawns_beyond_220() {
    let m = map(vec![vehicle("guncar", [0.0, 0.0, 0.0], 0, 0)], command(1.0, false, false));
    let mut w = world(&m, &[("guncar", GUNCAR)], [32, 6, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    // At 210: out of range, nothing.
    let o = w.step(None, &g, None, &player([0.0, 0.0, 210.0]), &mut rng, &mut no_ray);
    assert!(spawned(&o.actions).is_empty() && w.live.is_empty());
    // At 150: the next range check spawns it.
    let mut n = 0;
    while w.live.is_empty() && n < 10 {
        w.step(None, &g, None, &player([0.0, 0.0, 150.0]), &mut rng, &mut no_ray);
        n += 1;
    }
    assert_eq!(w.live.len(), 1, "spawned within {n} ticks of entering 200");
    assert_eq!(w.db.live[1], 1);
    // The spawn asked for the traffic within box + 6 to be parked [M 0x12cb08].
    // Between 200 and 220 it stays (hysteresis); beyond 220 it goes, and may come back.
    for _ in 0..10 {
        w.step(None, &g, None, &player([0.0, 0.0, 210.0]), &mut rng, &mut no_ray);
    }
    assert_eq!(w.live.len(), 1, "kept between 200 and 220");
    let mut gone = None;
    for _ in 0..10 {
        let o = w.step(None, &g, None, &player([0.0, 0.0, 240.0]), &mut rng, &mut no_ray);
        for a in o.actions {
            if let AiAction::Despawn { index, then } = a {
                gone = Some((index, then));
            }
        }
    }
    assert_eq!(gone, Some((0, ElemState::Idle)));
    assert!(w.live.is_empty() && w.db.live[1] == 0);
}

#[test]
fn the_vehicle_slot_cap_holds_and_held_elements_wait_for_activation() {
    let els = vec![
        vehicle("guncar", [0.0, 0.0, 10.0], 0, 0),
        vehicle("guncar", [0.0, 0.0, 20.0], 0, 0),
        vehicle("guncar", [0.0, 0.0, 30.0], 0, 0),
        vehicle("guncar", [0.0, 0.0, 40.0], flag::HELD, 0),
    ];
    let m = map(els, command(1.0, false, false));
    let mut w = world(&m, &[("guncar", GUNCAR)], [32, 2, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    for _ in 0..20 {
        w.step(None, &g, None, &player([0.0, 0.0, 0.0]), &mut rng, &mut no_ray);
        assert!(w.live_vehicles() <= 2, "the world's VEHICLE_SLOTS_AVAIL");
    }
    assert_eq!(w.live_vehicles(), 2);
    let mut live: Vec<usize> = w.live.iter().map(|v| v.element).collect();
    live.sort();
    // The two nearest by the priority key (dot + 1)·0.5/dist [M 0x12b440].
    assert_eq!(live, vec![0, 1]);
    assert!(w.db.elements[3].state == ElemState::Idle);
    // EActivateAIElement clears HELD [M 0x1476b8]; EDeactivateAIElement sets it and despawns
    // the vehicle [M 0x14b510].
    w.activate(3);
    assert_eq!(w.db.elements[3].rec.flags & flag::HELD, 0);
    let a = w.deactivate(0).expect("a live vehicle despawns");
    assert!(matches!(a, AiAction::Despawn { index: 0, .. }));
    assert!(w.db.elements[0].rec.flags & flag::HELD != 0 && w.live.iter().all(|v| v.element != 0));
}

#[test]
fn the_spawn_command_drives_the_car_at_the_player() {
    // A chase-brain car (flag 8 → sim kind 2) whose +0xf8 names a pattern-2 profile-0xd command.
    let m = map(vec![vehicle("guncar", [0.0, 0.0, 0.0], flag::CHASE_BRAIN, CMD)], command(-1.0, false, false));
    let mut w = world(&m, &[("guncar", GUNCAR)], [32, 6, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    let p = player([0.0, 0.0, 120.0]);
    w.step(None, &g, None, &p, &mut rng, &mut no_ray);
    let v = &w.live[0];
    assert_eq!(v.sim_kind, 2);
    let LiveKind::Car { brain: Some(b), .. } = &v.kind else { panic!("chase brain") };
    assert_eq!((b.pattern, b.profile, b.cap, b.command), (2, 0xd, 30.0, Some(0)));
    assert!(b.has_target, "target mode 0 chases the player");
    for _ in 0..600 {
        w.step(None, &g, None, &p, &mut rng, &mut no_ray);
    }
    let pos = w.live[0].pos();
    let gap = ((pos[0] - p.pos[0]).powi(2) + (pos[2] - p.pos[2]).powi(2)).sqrt();
    assert!(gap < 60.0, "the pursuer closed on the player: gap {gap}");
    // A kind-4 car (no flag 8) has no AIVehicle controller and does not drive.
    let m = map(vec![vehicle("guncar", [0.0, 0.0, 0.0], 0, CMD)], command(-1.0, false, false));
    let mut w = world(&m, &[("guncar", GUNCAR)], [32, 6, 16]);
    for _ in 0..300 {
        w.step(None, &g, None, &p, &mut rng, &mut no_ray);
    }
    let q = w.live[0].pos();
    assert!(q[0].abs() < 0.5 && q[2].abs() < 0.5, "kind 4 stays put: {q:?}");
    assert!(matches!(&w.live[0].kind, LiveKind::Car { brain: None, .. }));
}

// ---------------------------------------------------------------- shots

#[test]
fn guns_and_rockets_land_damage_calls() {
    // The car sits facing +z with the player 40 ahead: in the 0.99 / 0.9 cones, seen, in range.
    let m = map(vec![vehicle("guncar", [0.0, 0.0, 0.0], flag::CHASE_BRAIN, CMD)], command(0.5, true, true));
    let mut w = world(&m, &[("guncar", GUNCAR)], [32, 6, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    let p = player([0.0, 0.0, 40.0]);
    let (mut gun, mut rocket, mut launches) = (Vec::new(), Vec::new(), 0);
    for _ in 0..240 {
        let o = w.step(None, &g, None, &p, &mut rng, &mut no_ray);
        launches += o.events.iter().filter(|e| matches!(e, HostileEvent::Launch { .. })).count();
        for h in o.hits {
            if h.weapon == GUN {
                gun.push(h.hit);
            } else {
                rocket.push(h.hit);
            }
        }
    }
    assert!(!gun.is_empty() && launches > 0 && !rocket.is_empty(), "gun {} rockets launched {launches} hit {}", gun.len(), rocket.len());
    // The gun: the player's table damage (10) × ai_damage_scale(0) = 10, kind 2, dent 0.25
    // [M 0x14ca60, 0x15de48].
    assert!(gun.iter().all(|h| h.amount == WEAPONS[GUN].damage as f32 && h.kind == 2 && h.dent == 0.25 && !h.from_player));
    // The rocket: 250, kind 2 [M 0x182978].
    assert!(rocket.iter().all(|h| h.amount == 250.0 && h.kind == 2));
    // The cadence: fps·0.5 = 30 ticks between bursts [M 0x12f730] → at most 240/30 + 1.
    assert!(gun.len() <= 9, "{} bursts", gun.len());
}

#[test]
fn no_weapon_bytes_without_the_atr_declaration() {
    // FUN_00145ff8 honours the command's gun byte only when the car's .atr declares
    // HAS_MACHINEGUNS / HAS_CANNONS [M].
    let m = map(vec![vehicle("paris_henchbigbox", [0.0, 0.0, 0.0], flag::CHASE_BRAIN, CMD)], command(0.5, true, true));
    let mut w = world(&m, &[("paris_henchbigbox", BIGBOX)], [32, 6, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    let mut hits = 0;
    for _ in 0..240 {
        hits += w.step(None, &g, None, &player([0.0, 0.0, 40.0]), &mut rng, &mut no_ray).hits.len();
    }
    assert_eq!(hits, 0);
}

#[test]
fn riders_are_seated_and_fire_on_their_cadence() {
    let mut el = vehicle("paris_henchbigbox", [0.0, 0.0, 0.0], 0, 0);
    el.riders = vec![
        AiRider { seat: 4, model: 0, weapon: 1, raw_0c: 0, raw_10: 0, raw_18: 0, health: 50.0, skill: 0 },
        AiRider { seat: 9, model: 0, weapon: 1, raw_0c: 0, raw_10: 0, raw_18: 0, health: 50.0, skill: 0 },
    ];
    let m = map(vec![el], command(1.0, false, false));
    let mut w = world(&m, &[("paris_henchbigbox", BIGBOX)], [32, 6, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    let mut shots = Vec::new();
    // The player straight ahead inside the sunroof's 75-unit range, on the aim line.
    for _ in 0..200 {
        let o = w.step(None, &g, None, &player([0.0, 0.0, 30.0]), &mut rng, &mut no_ray);
        shots.extend(o.events.into_iter().filter(|e| matches!(e, HostileEvent::RiderShot { .. })));
    }
    // Seat 9 is not a seat: the seating stops there [M 0x12cb08]; the hench count = 1.
    assert_eq!(w.live[0].riders.len(), 1);
    assert_eq!(w.db.live[0], 1);
    // The sunroof (weapon 1): 25 shots, one every 60 ticks [M 0x118720 / 0x120b70 / 0x120fe0].
    assert!((3..=4).contains(&shots.len()), "{} rider shots", shots.len());
    assert!(shots.iter().all(|e| matches!(e, HostileEvent::RiderShot { anim, .. } if (6..12).contains(anim))), "anim = (rand·6 >> 16) + 6·weapon");
}

#[test]
fn a_helicopter_takes_the_heli_pool_and_guns_on_approach() {
    // +0x77 set: FUN_002003c8 → the AIHelicopter; command state 3 (approach), mode 2 (hover),
    // +0x68 = 2 rockets (the approach guns while +0xdc ≠ 0), 0.5 s between shots [M 0x138fa8].
    let mut el = vehicle("heli", [0.0, 30.0, 0.0], 0, CMD);
    el.heli = true;
    let mut c = command(0.5, false, false);
    c.profile = 3;
    c.pattern = 2;
    c.ram_chance = 2.0;
    c.follow = 20.0;
    c.catchup = 1.0;
    c.speed_cap = 30.0;
    let m = map(vec![el], c);
    let mut w = world(&m, &[("heli", "[heli]
RENDER_FILENAME.s=heli.crp
")], [32, 6, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    let mut guns = 0;
    let mut hits = 0;
    let before = rng.draws;
    for _ in 0..600 {
        let o = w.step(None, &g, None, &player([0.0, 0.0, 40.0]), &mut rng, &mut no_ray);
        guns += o.events.iter().filter(|e| matches!(e, HostileEvent::Shot { weapon: GUN, .. })).count();
        hits += o.hits.len();
    }
    assert_eq!((w.live_helis(), w.live_vehicles(), w.db.live[2], w.db.live[1]), (1, 0, 1, 0));
    let LiveKind::Heli(h) = &w.live[0].kind else { panic!() };
    assert_eq!((h.heli.state, h.heli.mode), (3, 2));
    assert!(guns > 0 && hits > 0, "guns {guns} hits {hits}");
    // One flight-control draw per tick, three per gun shot, one aim roll per shell [M 0x13b4f8 /
    // 0x1387a0 / 0x14ca60].
    assert_eq!(rng.draws - before, 600 + 3 * guns as u32 + guns as u32);
    assert!(w.draw_list()[0].heli);
}

#[test]
fn the_draw_list() {
    let mut h = vehicle("guncar", [3.0, 0.0, 4.0], 0, 0);
    h.facing = [1.0, 0.0, 0.0];
    let mut hench = vehicle("", [10.0, 0.0, 0.0], 0, 0);
    hench.kind = TYPE_HENCHMAN;
    let m = map(vec![h, hench], command(1.0, false, false));
    let mut w = world(&m, &[("guncar", GUNCAR)], [32, 6, 16]);
    let g = FlatGround::paved(0.0);
    let mut rng = DriveRng::game_reset();
    w.step(None, &g, None, &player([0.0, 0.0, 0.0]), &mut rng, &mut no_ray);
    let d = w.draw_list();
    assert_eq!(d.len(), 2);
    let car = d.iter().find(|x| x.element == 0).unwrap();
    assert_eq!(car.model, "guncar");
    assert!((car.matrix[2][0] - 1.0).abs() < 1e-3, "the forward row is the record's facing +0x20");
    assert!((car.matrix[3][0] - 3.0).abs() < 0.1 && (car.matrix[3][2] - 4.0).abs() < 0.1);
    assert!(car.wheels.iter().all(|p| (p[0] - 3.0).abs() < 3.0));
    assert!(d.iter().any(|x| x.element == 1 && x.henchman));
}

// ---------------------------------------------------------------- the disc (NF_ISO)

struct Track {
    crp_bytes: Vec<u8>,
    atrs: Vec<(String, String)>,
    world: String,
}

fn load(iso: &IsoImage, viv: &str) -> Track {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV")).unwrap();
    let bigf = nf_formats::disc::read_bigf(iso, e).unwrap();
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
    let crp_bytes = read(&|n| n.contains("data\\track\\") && n.ends_with(".crp")).remove(0).1;
    let atrs = read(&|n| n.contains("\\pvehicle\\") && n.ends_with(".atr"))
        .into_iter()
        .map(|(n, b)| (n.rsplit('\\').next().unwrap().trim_end_matches(".atr").to_string(), String::from_utf8_lossy(&b).into_owned()))
        .collect();
    let world = read(&|n| n.contains("\\world\\") && n.ends_with(".atr") && !n.ends_with("default.atr")).remove(0).1;
    Track { crp_bytes, atrs, world: String::from_utf8_lossy(&world).into_owned() }
}

struct Report {
    spawns: usize,
    despawns: usize,
    max_live: usize,
    hits: usize,
    damage: f32,
    shots: usize,
    launches: usize,
    rider_shots: usize,
    worst_road: f32,
    moved: usize,
    max_persistent: usize,
}

/// `viv` for `ticks` at 1/60 with the player driven along the road network at 20 u/s from the
/// track's start [port script]. Each element is released (`EActivateAIElement`) the first time
/// the player comes within 200 of its record [S: the mission's triggers that release them are not
/// run; on MIS01 and MIS4 every element but four starts HELD].
#[derive(Clone, Copy, PartialEq, Debug)]
enum Physics {
    /// `Car::step` on the track's collision triangles.
    Land,
    /// The rail stand-in on the track's heights (how the view runs its traffic today).
    Rail,
}

fn run(iso: &IsoImage, viv: &str, ticks: i32, physics: Physics) -> Report {
    let t = load(iso, viv);
    let crp = Crp::parse(&t.crp_bytes).unwrap();
    let ai = MapAi::from_crp(&crp).unwrap();
    let net = RoadNet::parse_crp(&crp).unwrap();
    let tris = crp.extract_collision_tris().unwrap();
    let start = nf_formats::trackstart::track_spawn(&crp, &tris).unwrap();
    let ground = CollisionGround::new(tris);
    let wattr = WorldAttr::parse(&t.world);
    let set: Vec<VehicleAttrs> = ai.vehicle_set().iter().filter_map(|n| t.atrs.iter().find(|(a, _)| a.eq_ignore_ascii_case(n)).map(|(a, x)| VehicleAttrs::parse(a, x))).collect();
    let need = models_needed(&ai, &set);
    eprintln!("{viv}: models {:?}", need.iter().map(|m| (m.vehicle.as_str(), m.model.as_str(), m.missing_atr)).collect::<Vec<_>>());
    assert!(need.iter().all(|m| !m.missing_atr), "{viv}: every vehicle has its .atr");
    let mut w = HostileWorld::new(&ai, set, &wattr, 0, 1.0, 1.0, Some(&net));
    let mut released = vec![false; ai.elements.len()];
    let slots = wattr.slots();

    // The player's path: a traffic navigator from the road piece nearest the start.
    let mut best = (f32::MAX, 0usize);
    for k in 0..net.segs.len() {
        if let Ok(p) = net.seg_point(k, 0.5) {
            let d = (p[0] - start.pos[0]).powi(2) + (p[2] - start.pos[2]).powi(2);
            if d < best.0 {
                best = (d, k);
            }
        }
    }
    let mut pnav = RoadNav::join(&net, best.1, 0.5, 0, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    let mut prng = DriveRng::game_reset();
    let mut road: Vec<[f32; 3]> = Vec::new();
    for s in 0..net.segs.len() {
        for k in 0..=8 {
            if let Ok(p) = net.seg_point(s, k as f32 / 8.0) {
                road.push(p);
            }
        }
    }
    let road_dist = |p: [f32; 3]| road.iter().map(|q| ((q[0] - p[0]).powi(2) + (q[2] - p[2]).powi(2)).sqrt()).fold(f32::MAX, f32::min);
    let spawn_road: Vec<f32> = ai.elements.iter().map(|e| road_dist(e.pos)).collect();

    let mut rng = DriveRng::game_reset();
    let mut r = Report { spawns: 0, despawns: 0, max_live: 0, hits: 0, damage: 0.0, shots: 0, launches: 0, rider_shots: 0, worst_road: 0.0, moved: 0, max_persistent: 0 };
    let mut moved = std::collections::BTreeSet::new();
    for tick in 0..ticks {
        let _ = pnav.advance(&net, 20.0 * DT, &mut || prng.next());
        let dir = pnav.dir;
        let ppos = [pnav.pos[0], pnav.pos[1] + 0.8, pnav.pos[2]];
        let p = PlayerState { pos: ppos, vel: [dir[0] * 20.0, dir[1] * 20.0, dir[2] * 20.0], fwd: dir, box_z: 2.3 };
        for (i, e) in ai.elements.iter().enumerate() {
            if !released[i] && ((e.pos[0] - p.pos[0]).powi(2) + (e.pos[2] - p.pos[2]).powi(2)).sqrt() < ACTIVATE_RADIUS {
                released[i] = true;
                w.activate(i);
            }
        }
        let o = match physics {
            Physics::Land => w.step(Some(&net), &ground, None, &p, &mut rng, &mut |_, _| 0),
            Physics::Rail => w.step(Some(&net), &FlatWorld::paved(), Some(&ground), &p, &mut rng, &mut |_, _| 0),
        };
        for a in &o.actions {
            match a {
                AiAction::Spawn(s) => {
                    r.spawns += 1;
                    let e = &ai.elements[s.index];
                    let d = ((e.pos[0] - p.pos[0]).powi(2) + (e.pos[1] - p.pos[1]).powi(2) + (e.pos[2] - p.pos[2]).powi(2)).sqrt();
                    // Range checks run every 1..4 ticks; 4 ticks at 20 u/s move the player 1.4.
                    assert!(d < ACTIVATE_RADIUS + 1.5 || e.flags & flag::PERSISTENT != 0, "{viv} tick {tick}: element {} spawned at {d}", s.index);
                }
                AiAction::Despawn { index, .. } => {
                    r.despawns += 1;
                    let el = &w.db.elements[*index];
                    let pos = el.rec.pos;
                    let d = ((pos[0] - p.pos[0]).powi(2) + (pos[1] - p.pos[1]).powi(2) + (pos[2] - p.pos[2]).powi(2)).sqrt();
                    // By range (beyond 220, measured on the live car's last position, which
                    // KEEP_PLACE copies back), or a slot taken by a higher priority [M 0x12bec8].
                    let by_range = el.rt.dist > DEACTIVATE_RADIUS;
                    assert!(by_range || !el.rt.keep || el.rec.flags & flag::HELD != 0, "{viv} tick {tick}: element {index} despawned at {} (record {d})", el.rt.dist);
                }
            }
        }
        // The slot rule of FUN_0012bec8 [M]: everything fits the budget, or PERSISTENT elements
        // keep their slots first (even past the budget) and the rest fill what is left.
        let pers = w.live.iter().filter(|v| matches!(v.kind, LiveKind::Car { .. }) && ai.elements[v.element].flags & flag::PERSISTENT != 0).count() as i32;
        assert!(w.live_vehicles() as i32 <= slots[1].max(pers), "{viv} tick {tick}: {} live vehicles, budget {}, persistent {pers}", w.live_vehicles(), slots[1]);
        r.max_persistent = r.max_persistent.max(pers as usize);
        assert!(w.live_helis() as i32 <= slots[2], "{viv} tick {tick}: over the heli cap");
        assert_eq!(w.live_vehicles() as i32, w.db.live[1], "{viv}: the live list and the database agree");
        assert!(w.db.live[0] <= slots[0], "{viv}: henchmen + riders over HENCH_SLOTS_AVAIL");
        r.max_live = r.max_live.max(w.live.len());
        for h in &o.hits {
            r.hits += 1;
            r.damage += h.hit.amount;
            assert!(h.hit.kind == 2 && h.hit.amount > 0.0 && !h.hit.from_player);
        }
        for e in &o.events {
            match e {
                HostileEvent::Shot { .. } => r.shots += 1,
                HostileEvent::Launch { .. } => r.launches += 1,
                HostileEvent::RiderShot { .. } => r.rider_shots += 1,
                _ => {}
            }
        }
        for v in &w.live {
            if let LiveKind::Car { car, .. } = &v.kind {
                assert!(car.pos.iter().all(|x| x.is_finite()), "{viv}: element {} position", v.element);
                let rec = ai.elements[v.element].pos;
                let travel = ((car.pos[0] - rec[0]).powi(2) + (car.pos[2] - rec[2]).powi(2)).sqrt();
                if travel > 5.0 {
                    moved.insert(v.element);
                    // How far past its own spawn's distance from the road it strays.
                    let off = road_dist(car.pos) - spawn_road[v.element];
                    r.worst_road = r.worst_road.max(off);
                }
            }
        }
    }
    r.moved = moved.len();
    eprintln!(
        "{viv} {physics:?}: {} elements, slots {slots:?}; {ticks} ticks: {} spawns, {} despawns, max live {} (persistent {}), {} cars drove; worst road offset past the spawn's {:.1}; shells {} ({} hits, {:.0} damage), launches {}, rider shots {}; drive draws {}",
        ai.elements.len(),
        r.spawns,
        r.despawns,
        r.max_live,
        r.max_persistent,
        r.moved,
        r.worst_road,
        r.shots,
        r.hits,
        r.damage,
        r.launches,
        r.rider_shots,
        rng.draws
    );
    r
}

#[test]
fn mis01_and_mis4_for_3600_ticks() {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return;
    };
    let iso = IsoImage::open(&path).unwrap();
    for viv in ["MIS01", "MIS4"] {
        let land = run(&iso, viv, 3600, Physics::Land);
        eprintln!("  {viv} on the land physics: worst road offset {:.1}", land.worst_road);
    }
    let a = run(&iso, "MIS01", 3600, Physics::Rail);
    let b = run(&iso, "MIS4", 3600, Physics::Rail);
    for (viv, r) in [("MIS01", &a), ("MIS4", &b)] {
        assert!(r.spawns > 0 && r.moved > 0, "{viv}: hostiles spawned and drove");
        assert!(r.worst_road < 40.0, "{viv}: a car strayed {} from the road", r.worst_road);
    }
    // MIS4's snowmobiles carry guns and rockets (supersnow.atr [D]).
    assert!(b.hits > 0, "MIS4: shots landed damage calls");
    // MIS01's hostiles carry no vehicle weapon in their .atr [D]; their riders fire on their
    // cadence, but the rider's round is not traced, so no damage call comes from them.
    assert!(a.rider_shots > 0, "MIS01: the riders fired");
}
