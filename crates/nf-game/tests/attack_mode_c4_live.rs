//! guard-mode-c4-glass: the Drone attack-type byte `+0xc4` is written for EVERY drone at
//! creation and a landed abseiler restores the mode byte from it.
//!
//! Game law [M]:
//! - `NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8:22-23 writes
//!   `+0xc4 = GetDTYPENEW(drone, +0xc7, 0)` — the type's BLOCK-0 mode. It runs from
//!   `NDrone2_DoModeSettingsNEW` 0x14c600:80, i.e. for every drone (DoModeSettingsOLD
//!   tail-calls NEW, 0x14cfa0:16); the astronaut kind-0x18 arm then overwrites
//!   `+0xc4 = 0x1d` [M DefaultInit 0x14b300:725].
//! - `GetDTYPENEW` 0x14c938: block 0 + prop 0x10 (bit 16 of the behaviour word 0,
//!   bitDescs 0x2c7250/0x2c7308) → 0x10; dtype 0xa/0xb → 9/0x12/10; dtype 0..9 outside
//!   07000007 → the block-0 tail 0.
//! - The landing reads it back: AbseilSlide's feet-probe arm `+0xc5 = +0xc4`
//!   [M 0x169730:34-36; StepOff 0x1738e0:21], and the GoToGoalPosition arrival keeps it
//!   [M 0x165640:78-79].
//!
//! Disc reach [D nf-extract drones]: the two 07000011 abseilers (p258 ch 0x13, p280 ch 0x14)
//! are dtype 3 (param 15) with normal block 0 (param 16) and behaviour word 0 param 18 =
//! 0xa8d2e0c1 (bit 16 CLEAR) — their game attack type is the block-0 tail 0.
//! Run with: `NF_ISO=… cargo test -p nf-game --test attack_mode_c4_live -- --test-threads=1`

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Rng};
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
}

/// The p258 abseiler placement's own parameters, verbatim [D nf-extract drones 07000011.bin:
/// (18.895, 46.516, 1.434) skin 0x05000008, param 1 = 0x06000892, ch 0x13].
fn abseiler_params() -> impl Fn(u32) -> u32 {
    |k: u32| match k {
        0 => 0x0500_0008, 1 => 0x0600_0892, 2 => 1, 3 => 0x13, 5 => 0x64, 8 => 0x64, 9 => 19, 10 => 1,
        15 => 3, 16 => 0, 17 => 0x5b_0003, 18 => 0xa8d2_e0c1, 19 => 0xed73_9fe7, 20 => 0x4029_1637,
        22 => 0x5b_0003, 23 => 0xa8d2_e0c1, 24 => 0xed73_9fe7, 25 => 0x4029_1637,
        _ => 0,
    }
}

/// The same placement carrying the AMBUSH bit in its behaviour word 0 (param 18 bit 16 =
/// behaviour prop 0x10 [M bitDescs 0x2c7250: word 0, mask 0x10000, shift 16]) — the game's
/// `GetDTYPENEW(type, 0)` then answers 0x10 (the AmbushInit row, start 0x37
/// [M 0x14c938:10-14; D docs/fields/dronetype-tables.md:98]) for the attack type.
fn ambush_params() -> impl Fn(u32) -> u32 {
    |k: u32| if k == 18 { 0xa8d2_e0c1 | 0x1_0000 } else { abseiler_params()(k) }
}

/// The 0700001b astronaut skin [D docs/systems/ai-dmode-inits.md; M DefaultInit 0x14b300:722-727
/// arms kind 0x18]: kind 0x18 → `+0xc4 = 0x1d` alongside `+0xc5 = 0x1d`. The placement msg is the
/// plain 0x06000000 — no disc astronaut is a roper [D], and the rope msg's own +0xc5 write
/// (0x14b300:615-627) must not shadow the arm.
fn astronaut_params() -> impl Fn(u32) -> u32 {
    |k: u32| match k { 1 => 0x0600_0000, 0 => 0x0500_0078, _ => abseiler_params()(k) }
}

fn make(fx: &Fixture, param: impl Fn(u32) -> u32) -> nf_game::drone::Drone {
    let d = fx.data();
    let mut rng = Rng::new(0x1234_5678);
    nf_game::drone::Drone::new(&d, [18.9, 46.5, 1.4], 0.0, param, &mut rng)
}

#[test]
fn the_attack_type_is_the_type_s_block_0_mode_written_at_creation() {
    let Some(fx) = Fixture::load("07000011.bin", 0x0700_0011) else { eprintln!("skip: NF_ISO not set"); return };
    // The disc abseiler: dtype 3, block 0, no ambush bit → GetDTYPENEW(3, 0) = the block-0 tail 0
    // [M 0x14c938:100-124]. This is the value the landing restores [M 0x169730:36].
    let dr = make(&fx, abseiler_params());
    assert_eq!(dr.mode, 0x1c, "the 0x06000892 msg arms the rope mode [M 0x14b300:615-627]");
    assert_eq!(dr.attack_mode, 0,
        "+0xc4 = GetDTYPENEW(3, 0) = 0 for the disc abseiler [M 0x14cbd8:22-23; D param 18 bit 16 clear]");

    // The ambush-bit drone: the game lands it INTO the AmbushInit row, not the generic guard.
    let dr = make(&fx, ambush_params());
    assert_eq!(dr.attack_mode, 0x10,
        "+0xc4 = GetDTYPENEW(3, 0) = 0x10 with behaviour prop 0x10 set [M 0x14c938:10-14]");

    // The astronaut skin: kind 0x18 rewrites the attack type to 0x1d [M 0x14b300:725].
    let dr = make(&fx, astronaut_params());
    assert_eq!(dr.mode, 0x1d, "the astronaut arm sets the mode [M 0x14b300:723]");
    assert_eq!(dr.attack_mode, 0x1d, "+0xc4 = 0x1d in the same arm [M 0x14b300:725]");
}
