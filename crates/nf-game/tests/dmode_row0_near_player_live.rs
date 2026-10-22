//! NF_ISO tests for unported-guard-ai-other-2 (docs/systems/ai-dmode-inits.md):
//! the OLD-mode row 0 (`NDrone2_init_DMODE_Normal` 0x14d958 → `NDrone2_init_DMODE_Defaults`
//! 0x14d528, reached through the 07000001 coder event and MP bots) and
//! `DroneFunc_NearPlayerCollision` 0x14b190 (the < 2.0 per-player physics gate).
//! Opt-in: `NF_ISO=<iso>`.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drones, Player, Rng};
use std::collections::HashMap;

struct Fixture {
    tables: drone::DroneAnimData,
    props: drone::PropertyTables,
    weapons: WeaponData,
    level: Level,
    level_id: u32,
    scripts: HashMap<u32, nf_formats::anim::Script>,
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
        let scripts: HashMap<u32, nf_formats::anim::Script> = lb.entries.iter().filter(|e| e.kind == 5)
            .filter_map(|e| nf_formats::anim::Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
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

const LEVEL: &str = "0700001b.bin";
const LEVEL_ID: u32 = 0x0700_001b;

/// The 07000001 coder event's exact DIVars shape [D census: NIS 0x0600003B kind 8, params
/// 0x0500005B / 0x06000106 / 16 / 1 → words 11/12/20; everything else memset 0, so parameter 5
/// (word 18) = 0 → `DoModeSettingsOLD` row 0]. Parameter k = word 13+k [M Drone_Create
/// 0x136be0:59,81 copies template+0x2C to Drone+0xC34].
fn coder_event_params() -> impl Fn(u32) -> u32 {
    |k: u32| match k {
        0 => 0x0500_005b,
        1 => 0x0600_0106,
        7 => 1,
        _ => 0,
    }
}

/// Row 0 through the real spawn path: `Drone::new` with the coder event's parameters sets
/// Defaults' 40 prop bits [M 0x14d528:4-44] plus `DoModeSettingsOLD`'s row-0 tail 0x38
/// [M 0x14cfa0], with 0x17 re-cleared by DefaultInit's own tail [M 0x14b300:778] and the
/// resting alertness at row 0's word+4 = 0.0 [D table 0x29ad40 row 0]. Sisters: prop 0x2b
/// (in no list here) stays clear, and the same shape at parameter 5 = 100 (the NEW path every
/// placement takes [D 555/555]) must NOT carry the Defaults bits.
#[test]
fn coder_event_row0_sets_the_defaults_props() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { return };
    let d = fx.data();
    let mut rng = Rng::new(0x1234_5678);
    let dr = nf_game::drone::Drone::new(&d, [4.0, 1.0, 4.0], 0.0, coder_event_params(), &mut rng);
    for id in [1usize, 6, 7, 10, 0xc, 0x48, 0xf, 0x11, 0x13, 0x18, 0x1d, 0x4c, 0x21, 0x29, 0x2e, 0x37, 0x41, 0x42, 0x4f, 0x51, 0x5a, 0x24, 0x3c, 0x1f, 0x44, 0x33, 0x32, 0x31, 0x28, 0x27, 0x26, 0x30, 0x2f, 0x2a, 0x43, 0x3b, 0x46, 0x38] {
        assert_eq!(d.props.get(id, &dr.props), 1, "Defaults prop {id:#x} must be set on the OLD row-0 path");
    }
    assert_eq!(d.props.get(0x17, &dr.props), 0, "DefaultInit 0x14b300:778 re-clears 0x17 after Defaults set it");
    assert_eq!(d.props.get(0x2b, &dr.props), 0, "sister: 0x2b is in no row-0 list");
    assert_eq!((dr.alert, dr.alert_floor), (0.0, 0.0), "row 0's word+4 is 0.0");
    assert_eq!(dr.mode, 0, "row 0's mode byte is 0");
    assert_eq!(dr.team, 1, "row 0's team byte is 1");
    // Sister: the same shape one word over (parameter 5 = 100, every placement [D 555/555])
    // dispatches NEW and must not carry the Defaults-only bits.
    let mut rng2 = Rng::new(0x1234_5678);
    let new_path = nf_game::drone::Drone::new(&d, [4.0, 1.0, 4.0], 0.0, |k| match k { 0 => 0x0500_005b, 1 => 0x0600_0106, 5 => 100, 7 => 1, _ => 0 }, &mut rng2);
    assert_eq!(d.props.get(0x48, &new_path.props), 0, "NEW path must not run Defaults");
    assert_eq!(d.props.get(0x51, &new_path.props), 0, "sister: 0x51 is Defaults/0x17-mode only");
}

/// `DroneFunc_NearPlayerCollision` 0x14b190 through the real tick: `Drones::tick` fills the
/// +0x154 per-player distances (`NDrone2_PreDroneControl` 0x148db0:35-48) and the probe reads
/// them. A guard beside Bond is near; the same guard at exactly 2.0 is NOT (the decompile
/// continues past `2.0 <= dist` [M 0x14b190:10-16]); a guard across the level is not.
#[test]
fn near_player_collision_gate_fills_and_reads() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { return };
    let d = fx.data();
    let mut sim = fx.sim();
    for c in 1..=255u8 { sim.set_channel(c); }
    for _ in 0..3 { sim.tick(&d, &bond_at([0.0, -1000.0, 0.0])); }
    let idx = sim.list.iter().position(|x| x.present)
        .expect("0700001b ships 555-disc drones; at least one placement enables");
    // The fill: slot 0 is the live 3-D distance, slots 1..3 the game's empty-slot 100000.0.
    let far_bond = bond_at([0.0, -1000.0, 0.0]);
    sim.tick(&d, &far_bond);
    let dr = &sim.list[idx];
    let want = ((dr.pos[0] - far_bond.pos[0]).powi(2) + (dr.pos[1] - far_bond.pos[1]).powi(2) + (dr.pos[2] - far_bond.pos[2]).powi(2)).sqrt();
    assert!((dr.dist_to_player[0] - want).abs() < 1e-3, "slot 0 must be Vec_Dist3D: {} vs {want}", dr.dist_to_player[0]);
    assert_eq!(dr.dist_to_player[1], 100000.0, "the port's empty slots keep the game's 100000.0");
    assert!(!dr.near_player_collision(), "sister: a guard ~1000 away is not near");

    // Beside Bond: 1.0 away → near. Exactly 2.0 → the decompile's boundary is exclusive.
    // (Exact-representable positions so the f32 subtraction recovers the exact distance —
    // a placement-derived x plus 2.0 does not round-trip.)
    sim.list[idx].pos = [0.0, 1.0, 0.0];
    sim.tick(&d, &bond_at([1.0, 1.0, 0.0]));
    assert!((sim.list[idx].dist_to_player[0] - 1.0).abs() < 1e-6);
    assert!(sim.list[idx].near_player_collision(), "a guard 1.0 from the player is near [M 0x14b190]");
    sim.list[idx].pos = [0.0, 1.0, 0.0];
    sim.tick(&d, &bond_at([2.0, 1.0, 0.0]));
    assert_eq!(sim.list[idx].dist_to_player[0], 2.0);
    assert!(!sim.list[idx].near_player_collision(), "2.0 itself is the decompile's not-near boundary");
}
