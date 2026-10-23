//! abseil-death-11 live tests: a hit/kill during the rope descent (07000011: abseiling
//! guards must fall when they die).
//! Run with: `NF_ISO=… cargo test -p nf-game --test abseil_death_live -- --test-threads=1`
//!
//! Disc reach [D]: exactly five placements carry param 1 = 0x06000892 — 07000011 ×2 (skins
//! 0x05000008, at (18.895, 46.516, 1.434) ch 0x13 and (19.745, 46.580, 0.363) ch 0x14, the second
//! gated to difficulty ≥ 2 [M Drone_Create 0x136be0:49]) and 0700004a ×3; no placement uses
//! 0x0600089C (mode 0x1b) [D nf-extract drones]. The placement msg sets mode (+0xC5) 0x1c and
//! starts AbseilInit 0x8e [M DefaultInit 0x14b300:615-627, 757-766].
//!
//! The kill routing [M]: a killing bullet impact selects the death by the mode byte +0xC5 —
//! 0x1e → BotDeathAnim 0xf2 (not ported), 0x1b/0x1c → AbseilDeath 0x92, 0x1d → AstronautDeath
//! 0xbf, else Death_Anim 0x44 [M NDrone2_BulletImpact 0x1461c8:40-52]; the killing blast the
//! same table with 0x0d → 0x44 and else → DeathByExplosion 0x45 [M 0x146480 @0x1465e4-0x146660].
//! AbseilDeath 0x92 [M 0x169b28]: ENTER sets the corpse flag obj+0xF0 & 0x20 (SP), clears the
//! rope flag +0x23, danim 0x40, DropWeapon; UPDATE waits for the feet probe (anim+0x60 & 8) →
//! danim 10 + DEAD 0x47. The corpse keeps gravity and collision — DoCollision/DoGravity gate on
//! the drone flags +0x4F8, never on the corpse flag [M 0x149ec8 / 0x141e58].

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{msg, state, Data, Drones, Hit, Player, Rng};
use std::collections::HashMap;

struct Fixture {
    tables: drone::DroneAnimData,
    props: drone::PropertyTables,
    weapons: WeaponData,
    level: Level,
    level_id: u32,
    scripts: HashMap<u32, Script>,
    world: nf_game::world::CollisionWorld,
    sequences: HashMap<u32, nf_formats::anim::Sequence>,
    skeletons: HashMap<u8, nf_formats::character::Skeleton>,
    nav: nf_game::nav::Network,
    covers: Vec<nf_game::cover::CoverNode>,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: HashMap<u32, nf_game::pickup::Model>,
    skins: HashMap<u32, nf_formats::character::Skin>,
}

impl Fixture {
    fn load(level_name: &str, level_id: u32) -> Option<Fixture> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let read = |name: &str| -> Option<Vec<u8>> {
            let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
            Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
        };
        let data = read(level_name)?;
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let skeletons: HashMap<u8, _> = lb.entries.iter().filter(|e| e.kind == 6)
            .filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok().map(|s| (s.id as u8, s))).collect();
        let anim_name = format!("{}F{}.bin", &level_name[..2], &level_name[3..8]);
        let mut sequences = HashMap::new();
        if let Some(anim) = read(&anim_name) {
            let alb = LevelBin::parse(&anim).unwrap();
            for e in alb.entries.iter().filter(|e| e.kind == 4 && e.length >= 0x40) {
                if let Ok(s) = nf_formats::anim::Sequence::parse(alb.data(&anim, e)) { sequences.insert(s.hashcode, s); }
            }
        }
        let scripts: HashMap<u32, Script> = lb.entries.iter().filter(|e| e.kind == 5)
            .filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let tables = drone::DroneAnimData::from_action_elf(&elf).unwrap();
        let props = drone::PropertyTables::from_action_elf(&elf).unwrap();
        let weapons = WeaponData::from_action_elf(&elf).unwrap();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let main = level.main_map().unwrap();
        let nav = nf_game::nav::Network::build(level.maps[main].ai_network.as_ref().unwrap_or(&Default::default()), &world, level_id);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let hitboxes = nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap();
        let volumes = nf_game::aivolume::build(&level, level_id);
        let models = nf_game::pickup::models(&level);
        let tuning = { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap();
            nf_game::tuning::read_tuning_vars(&mut t, level_id); t };
        let skins = level.skins.clone();
        Some(Fixture { tables, props, weapons, level, level_id, scripts, world, sequences, skeletons, nav, covers, tuning, hitboxes, volumes, models, skins })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world,
            level: self.level_id, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav,
            covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: None, models: &self.models,
            surfaces: None, voices: None, difficulty: 1, ai_points: &[] }
    }

    fn sim(&self) -> Drones {
        let d = self.data();
        let mut rng = Rng::new(0x1234_5678);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        let mut s = Drones::new(drones, rng);
        s.attach_level_id(&self.level, self.level_id);
        s
    }
}

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None } }

const LEVEL: &str = "07000011.bin";
const LEVEL_ID: u32 = 0x0700_0011;

/// The 07000011 rope guard: fire its switch channel 0x13 [D param 3], let the chain reach the
/// Hang, then a killing bullet. The kill must route to AbseilDeath 0x92 (mode 0x1c [M 0x1461c8:
/// 44-52]), the ENTER must release the rope (+0x23 cleared [M 0x169b28:28]), and the corpse must
/// fall — the gravity/collision predicates never read the corpse flag [M 0x141e58 / 0x149ec8] —
/// and land: the feet probe → danim 10 + DEAD 0x47 [M 0x169b28 case 3].
#[test]
fn a_guard_killed_on_the_rope_leaves_it_and_lands_below() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let mut sim = fx.sim();
    for _ in 0..3 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    let Some(ai) = sim.list.iter().position(|x| x.present && x.mode == 0x1c) else { panic!("no 0x06000892 rope guard [D]") };
    // Release the WAIT_SWITCH gate [D param 3 → ch 0x13] and let Init → Slide → Hang run.
    sim.set_channel(0x13);
    let mut hung = None;
    for t in 1..=120u32 {
        sim.tick(&d, &bond_at([0.0, -1000.0, 0.0]));
        if sim.list[ai].state == state::ABSEIL_HANG { hung = Some(t); break; }
    }
    assert!(hung.is_some(), "the rope chain reaches the Hang (720's chain, timer ≤ Rand_Rand(60))");
    assert!(sim.list[ai].air, "on the rope (+0x23) [M]");
    let rope_y = sim.list[ai].pos[1];
    // The killing bullet: a 5000-damage HITDATA through the real msg-8 path [M 0x1461c8].
    sim.list[ai].hit = Some(Hit { part: 4, ray: [0.0; 3], point: [0.0; 3], damage: 5000.0, weapon: 2 });
    sim.list[ai].send(&d, &mut sim.rng, sim.frame, msg::BULLET);
    assert_eq!(sim.list[ai].state, state::ABSEIL_DEATH, "mode 0x1c kill → AbseilDeath 0x92 [M 0x1461c8:44-52]");
    assert!(!sim.list[ai].air, "+0x23 cleared at the ENTER — released from the rope [M 0x169b28:28]");
    assert!(sim.list[ai].corpse, "obj+0xF0 & 0x20 (SP arm) [M 0x169b28:23]");
    // The corpse falls (the port must NOT short-circuit the collision handler on the corpse flag
    // — the game's DoCollision/DoGravity gate on +0x4F8 bits [M 0x149ec8 / 0x141e58]) and lands:
    // the feet probe → danim 10 + DEAD 0x47 [M 0x169b28 case 3]. WldGravity 9.8 u/s² [M
    // 0x140968's gravity arm] puts a ~10-unit drop inside ~2 s.
    let mut landed = None;
    for t in 1..=240u32 {
        sim.tick(&d, &bond_at([0.0, -1000.0, 0.0]));
        if sim.list[ai].abseil_land_latch { landed = Some(t); break; }
    }
    let t = landed.expect("the corpse lands (the feet probe → danim 10, +0xcdc = 1)");
    let dr = &sim.list[ai];
    assert!(dr.on_surface, "the feet probe found the floor [M 0x169b28 case 3]");
    assert!(dr.pos[1] < rope_y - 1.0, "fell to a floor below the rope point ({rope_y} → {})", dr.pos[1]);
    assert_eq!(dr.anim.danim, 10, "the landed-corpse danim [M 0x169b28 case 3 SetDAnimNow(10)]");
    assert_eq!(dr.state, state::ABSEIL_DEATH, "touchdown does not enter Dead yet: 0x47 is the call's completion state (+0x596) [M 0x169b28:55]");
    assert!(t <= 240, "inside the game's gravity timing ({t} ticks)");
    // DEAD lands when CallHandler sees a class 2/9 danim [M 0x13add0]: danim 10 is class 8, its follow-on
    // 0xbf class 9 [D Drone_AnimInfo 0x273200] — so Dead starts with 0xbf playing.
    let mut dead = false;
    for _ in 0..600u32 {
        sim.tick(&d, &bond_at([0.0, -1000.0, 0.0]));
        if sim.list[ai].state == state::DEAD { dead = true; break; }
    }
    assert!(dead, "Dead 0x47 follows the landing anim");
    assert_eq!(sim.list[ai].anim.danim, 0xbf, "Dead starts on the class-9 follow-on 0xbf [D 0x273200 danim 10 next = 0xbf]");
}

/// A surviving bullet on the rope: the abseil DSTATEs pass `DroneFunc_HandleImpact(0, 0)` [M
/// 0x169730:68-70] — the damage applies (resume 0, allow 0: no flinch state) and the guard stays
/// on the rope.
#[test]
fn a_surviving_bullet_hurts_the_rope_guard_without_dropping_him() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let mut sim = fx.sim();
    for _ in 0..3 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    let Some(ai) = sim.list.iter().position(|x| x.present && x.mode == 0x1c) else { panic!("no rope guard [D]") };
    sim.set_channel(0x13);
    for _ in 0..120 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    assert_eq!(sim.list[ai].state, state::ABSEIL_HANG, "hung (test 1's chain)");
    let h0 = sim.list[ai].health;
    sim.list[ai].hit = Some(Hit { part: 4, ray: [0.0; 3], point: [0.0; 3], damage: 1.0, weapon: 2 });
    sim.list[ai].send(&d, &mut sim.rng, sim.frame, msg::BULLET);
    assert!(sim.list[ai].health < h0, "the damage applies through HandleImpact(0,0) [M 0x169730:68-70]");
    assert_eq!(sim.list[ai].state, state::ABSEIL_HANG, "alive → no death state [M 0x1461c8:52]");
    assert!(sim.list[ai].air, "still on the rope");
}

/// The explosive kill uses the same mode table [M 0x146480 @0x1465e4-0x146660] — a blast killing
/// the hanging guard releases him exactly like the bullet (0x1b/0x1c → 0x92; the old port read
/// the voice set +0xD8 here, the wrong field).
#[test]
fn an_explosive_kill_releases_the_rope_guard_the_same_way() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let mut sim = fx.sim();
    for _ in 0..3 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    let Some(ai) = sim.list.iter().position(|x| x.present && x.mode == 0x1c) else { panic!("no rope guard [D]") };
    sim.set_channel(0x13);
    for _ in 0..120 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    assert_eq!(sim.list[ai].state, state::ABSEIL_HANG);
    sim.list[ai].last_damage = 5000.0;
    sim.list[ai].blast = Some(sim.list[ai].pos);
    sim.list[ai].send(&d, &mut sim.rng, sim.frame, msg::EXPLOSION);
    assert_eq!(sim.list[ai].state, state::ABSEIL_DEATH, "the blast kill routes by mode +0xC5 [M 0x146480 @0x1465e4]");
    assert!(!sim.list[ai].air, "released [M 0x169b28:28]");
    let mut landed = None;
    for t in 1..=240u32 {
        sim.tick(&d, &bond_at([0.0, -1000.0, 0.0]));
        if sim.list[ai].state == state::DEAD { landed = Some(t); break; }
    }
    assert!(landed.is_some(), "the blasted corpse lands too");
    assert!(sim.list[ai].on_surface);
    assert!(sim.list[ai].pos[1] < 46.0, "off the rope top");
}

/// Death_Anim's ENTER re-routes an abseil kill that arrived through another death path [M
/// 0x1637f8:30-38]: mode 0x1b/0x1c → AbseilDeath 0x92 (before the special-death check), 0x1d →
/// AstronautDeath 0xbf. Sister: a plain mode-0 guard stays in Death_Anim 0x44.
#[test]
fn death_anim_re_routes_abseil_modes_into_the_rope_release() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let mut sim = fx.sim();
    for _ in 0..3 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    let Some(ai) = sim.list.iter().position(|x| x.present && x.mode == 0x1c) else { panic!("no rope guard [D]") };
    // The re-route arm is Death_Anim's own ENTER [M 0x1637f8:30-38]; the forced state is the
    // tests' established stand-in for a state the level's scripting cannot reach on demand
    // (ai_live.rs:1276, blast_smoke_live.rs:290).
    sim.list[ai].state = state::DEATH_ANIM;
    sim.list[ai].send(&d, &mut sim.rng, sim.frame, msg::ENTER);
    assert_eq!(sim.list[ai].state, state::ABSEIL_DEATH, "Death_Anim re-routes mode 0x1c into 0x92 [M 0x1637f8:34-37]");
    assert!(!sim.list[ai].air, "released");
    // Sister: a plain guard (mode 0) entering Death_Anim stays there.
    let bi = sim.list.iter().position(|x| x.present && x.mode == 0 && x.team == 1 && !x.corpse).expect("a walking guard on 11");
    sim.list[bi].state = state::DEATH_ANIM;
    sim.list[bi].send(&d, &mut sim.rng, sim.frame, msg::ENTER);
    assert_eq!(sim.list[bi].state, state::DEATH_ANIM, "mode 0 keeps Death_Anim 0x44 [M 0x1637f8:29,39]");
}

/// Which 07000011 spawners abseil [D placements/params]: exactly the two 0x06000892 placements —
/// at difficulty 1 exactly ONE mode-0x1c drone exists (the second placement's param 2 = 2 gates
/// it to difficulty ≥ 2 [M Drone_Create 0x136be0:49]), no mode-0x1b drone exists anywhere [D], and
/// the stairwell walkers (skins 0x05000004/0x05000006 [D docs/levels/07000011.md]) are mode-0
/// guards — they walk in, they never rope.
#[test]
fn only_the_disc_abseilers_rope_the_stairwell_walkers_walk() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let mut sim = fx.sim();
    for _ in 0..3 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    let abseilers: Vec<usize> = sim.list.iter().enumerate().filter(|(_, x)| x.present && x.mode == 0x1c).map(|(i, _)| i).collect();
    assert_eq!(abseilers.len(), 1, "exactly the ch-0x13 placement at difficulty 1 [D + M 0x136be0:49], got {}", abseilers.len());
    let g = &sim.list[abseilers[0]];
    assert_eq!(g.skin, 0x0500_0008, "the rope guard's skin [D]");
    assert!((g.pos[1] - 46.5).abs() < 0.3, "the (18.9, 46.5, 1.4) placement [D], got {:?}", g.pos);
    assert!(sim.list.iter().all(|x| x.mode != 0x1b), "no placement uses 0x0600089C [D]");
    // The stairwell walkers: every present skin-0x05000004/06 guard is a normal dtypenew-1/2
    // guard (modes 0/2) — NEVER the rope modes: DefaultInit arms the rope only for mode
    // 0x1b/0x1c, 0x06000892 writes those [M 0x14b300:757-766 + D params] and 0x0600089c also writes mode 0x1b [M 0x14b300:620-622] — it is just absent from the disc placements [D params].
    let walkers: Vec<u8> = sim.list.iter().filter(|x| x.present && matches!(x.skin, 0x0500_0004 | 0x0500_0006)).map(|x| x.mode).collect();
    assert!(!walkers.is_empty(), "07000011 ships stairwell walkers on those skins [D docs/levels/07000011.md]");
    assert!(walkers.iter().all(|&m| !matches!(m, 0x1b | 0x1c)), "the walkers never rope [D placements + M 0x14b300:757-766], modes {walkers:?}");
}
