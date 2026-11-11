//! A drone's round on another drone, on the real levels. Opt-in via NF_ISO.
//!
//! `Bullet_update` 0x12e7b0:389-421 arms the flight HITTEST with object filter word +0x98 = 0 and
//! the shooter in exclusion slot +0x74 (`DroneWeap_FireWeapon` 0x178f68:253 passes the drone as both
//! owner arguments), so `Collide_PickObj` 0x1eae80:57-59 offers every other enabled drone and
//! `Bullet_CollisionHandler` 0x12ce80 ends the round on the body. The struck drone's
//! `NDrone2_DealWithObjHit` 0x13fff0 case 5 lands the hit only across sides: a team-1 shooter on
//! teams 2/3, a team 2/3 shooter on team 1 [M :181-232], the victim alive [M :229-230]; then
//! `Drone_BulletHit` 0x13a9f8 stamps +0x2B4 with the shooter and sends msg 8.
//!
//! 07000012: the civilian #1 stands beside the two guards awake at the start. 07000008: the ally
//! #265 (team 2) and the guards behind ch 0x2a.
//!
//! Run: NF_ISO=… cargo test -p nf-view --test ally_bullets_live -- --nocapture --test-threads=1

use nf_game::player::Input;
use nf_view::scene::{Scene, Sim};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load(level: u32) -> Scene {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), &format!("{level:08x}.bin"), false, None, 3).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..6000 {
        sim.step_bond(Input::default(), false, false, false);
        if !sim.disabled && sim.drones.npc_enabled { break; }
    }
    assert!(sim.drones.npc_enabled, "{level:08x}: the AI runs");
    scene
}

fn step(sim: &mut Sim, ticks: u32) { for _ in 0..ticks { sim.step_bond(Input::default(), false, false, false); } }

fn by_placement(sim: &Sim, placement: usize) -> usize {
    sim.drones.list.iter().position(|d| d.placement == placement).expect("placed")
}

/// One round of the shooter's own weapon from 2.5 units off the victim's chest, aimed at it.
fn fire_at(sim: &mut Sim, shooter: usize, victim: usize) {
    let w = sim.drones.list[shooter].weapon as usize;
    let rec = sim.weapons.records[w].clone();
    assert_eq!(u16::from_le_bytes([rec[0], rec[1]]) as usize, w, "the record's own id is its index");
    let f = |o: usize| f32::from_le_bytes(rec[o..o + 4].try_into().unwrap());
    let v = sim.drones.list[victim].pos;
    let s = sim.drones.list[shooter].pos;
    let chest = [v[0], v[1] + 0.4, v[2]];
    // The first bearing (from the shooter's side round) with a clear line to the chest.
    let base = (s[0] - v[0]).atan2(s[2] - v[2]);
    let bearings: Vec<_> = (0..16).map(|k| {
        let a = base + k as f32 * std::f32::consts::PI / 8.0;
        [v[0] + a.sin() * 2.5, v[1] + 0.4, v[2] + a.cos() * 2.5]
    }).collect();
    // Clear to the chest, and 8 units on where the room allows, so a round that flew through
    // would live on.
    let on = |p: [f32; 3]| [chest[0] + (chest[0] - p[0]) / 2.5 * 8.0, chest[1], chest[2] + (chest[2] - p[2]) / 2.5 * 8.0];
    let from = bearings.iter().copied().find(|&p| sim.world.segment_bullet_clear(p, on(p)))
        .or_else(|| bearings.iter().copied().find(|&p| sim.world.segment_bullet_clear(p, chest)))
        .expect("a clear bearing");
    let d = [chest[0] - from[0], chest[1] - from[1], chest[2] - from[2]];
    let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    sim.drones.bullets.push(nf_game::bullet::Bullet {
        pos: from, dir: [d[0] / dl, d[1] / dl, d[2] / dl], speed: f(0x20), travelled: 0.0, range: f(0x1c),
        damage: f(0x0c), owner: shooter, state: 0, weapon: w as u16, flyby: false, bounces: 0,
        ..nf_game::bullet::Bullet::TAIL
    });
}

/// A guard's round kills the 07000012 civilian, and the guard is the killer: param 12 = 0x63 on a
/// team-3 actor fails the mission whoever kills it [M 0x13e660:21-44]. On the old walk the round
/// flew through her.
#[test]
fn a_guards_round_kills_the_12_civilian() {
    let mut scene = load(0x0700_0012);
    let sim = scene.sim.as_mut().unwrap();
    let civ = by_placement(sim, 1);
    let guard = by_placement(sim, 12);
    assert_eq!((sim.drones.list[civ].team, sim.drones.list[guard].team), (3, 1));
    assert!(sim.drones.list[guard].enabled && sim.drones.list[civ].enabled);
    let hp = sim.drones.list[civ].health;
    let hits = sim.drones.list[civ].hits_taken;
    fire_at(sim, guard, civ);
    step(sim, 3);
    let c = &sim.drones.list[civ];
    assert!(c.health < hp, "the round lands: health {} -> {}", hp, c.health);
    assert_eq!(c.hits_taken, hits + 1);
    assert!(!c.hit_by_player, "+0x2B4 is the guard [M 0x13a9f8:23]");
    for _ in 0..20 {
        if sim.drones.list[civ].health <= 0.0 { break; }
        fire_at(sim, guard, civ);
        step(sim, 3);
    }
    assert!(sim.drones.list[civ].health <= 0.0, "the civilian dies: {}", sim.drones.list[civ].health);
    assert!(!sim.drones.list[civ].hit_by_player);
    let mut failed = false;
    for _ in 0..600 { step(sim, 1); if sim.drones.channels.on(99) { failed = true; break; } }
    assert!(failed, "the 0x63 death fails the mission on a guard's kill");
}

/// Same side: a guard's round on another guard stops on the body and does nothing
/// [M 0x12ce80 state 2; 0x13fff0:181-232 team 1 on team 1 never lands]. The old walk let it fly
/// on. The shooter is a sleeping guard, so no round of his own joins the count.
#[test]
fn a_guards_round_stops_on_another_guard_without_harm() {
    let mut scene = load(0x0700_0012);
    let sim = scene.sim.as_mut().unwrap();
    let a = by_placement(sim, 10);
    let b = by_placement(sim, 13);
    assert!(!sim.drones.list[a].enabled && sim.drones.list[b].enabled);
    let hp = sim.drones.list[b].health;
    let hits = sim.drones.list[b].hits_taken;
    fire_at(sim, a, b);
    // One tick: the first step is (0.5 + FRand 0.5) · speed [M 0x12e7b0], past the 2.5 to the chest
    // for this weapon; on the old walk the round was then flying beyond him.
    step(sim, 1);
    let trail: Vec<_> = sim.drones.bullets.iter().filter(|x| x.owner == a).map(|x| x.pos).collect();
    let g = &sim.drones.list[b];
    assert_eq!(g.health, hp, "no damage across the same side");
    assert_eq!(g.hits_taken, hits);
    assert!(sim.drones.bullets.iter().all(|x| x.owner != a), "the round died on the guard's body; flight {trail:?}, speed {} range {}",
        f32::from_le_bytes(sim.weapons.records[sim.drones.list[a].weapon as usize][0x20..0x24].try_into().unwrap()),
        f32::from_le_bytes(sim.weapons.records[sim.drones.list[a].weapon as usize][0x1c..0x20].try_into().unwrap()));
}

/// 07000008: the ally's round wounds a guard (he is not Bond's: no accuracy hit), and a guard's
/// round reaches the ally (her skin takes no damage, so the hit is the count and the reaction).
#[test]
fn the_08_ally_and_the_guards_hit_each_other() {
    let mut scene = load(0x0700_0008);
    let sim = scene.sim.as_mut().unwrap();
    let ally = by_placement(sim, 265);
    let guard = by_placement(sim, 251);
    assert_eq!((sim.drones.list[ally].team, sim.drones.list[guard].team), (2, 1));
    sim.drones.channels.set(0x2a, 1, sim.drones.frame);
    for _ in 0..300 { if sim.drones.list[guard].enabled { break; } step(sim, 1); }
    assert!(sim.drones.list[guard].enabled, "ch 0x2a wakes the guard");
    let hp = sim.drones.list[guard].health;
    let log = sim.drones.list[guard].hit_log;
    fire_at(sim, ally, guard);
    step(sim, 3);
    let g = &sim.drones.list[guard];
    assert!(g.health < hp, "the ally's round lands: {} -> {}", hp, g.health);
    assert!(!g.hit_by_player);
    assert_eq!(g.hit_log, log, "+0x2B4 != +0x170: no shot-hit stat [M 0x145558:58, :121]");
    let hits = sim.drones.list[ally].hits_taken;
    fire_at(sim, guard, ally);
    step(sim, 3);
    assert_eq!(sim.drones.list[ally].hits_taken, hits + 1, "the guard's round reaches her");
    assert!(!sim.drones.list[ally].corpse);
}

