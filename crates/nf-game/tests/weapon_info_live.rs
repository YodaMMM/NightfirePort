//! `DroneWeap_GetWeaponInfo` 0x17a5a0 on the real path: the disc's weapon_data (runtime-initialised,
//! the port's `WeaponData::from_action_elf` emulation) and 07000005's posed guards. Pins the
//! datum/sound resolution (`AnimDatumGetWeaponInfo` 0x1c7900), the muzzle bone world transform
//! (`AnimGetBoneWorldTrans` 0x1c3120) and the `DroneVision_GunThroughWall` 0x178520 probe in the
//! open. Opt-in: set `NF_ISO` to the disc image.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drone, Drones, Player, Rng};
use std::collections::HashMap;

/// Everything `Data` borrows, owned.
struct Level05 {
    tables: drone::DroneAnimData,
    props: drone::PropertyTables,
    weapons: WeaponData,
    level: Level,
    scripts: HashMap<u32, Script>,
    world: nf_game::world::CollisionWorld,
    sequences: HashMap<u32, nf_formats::anim::Sequence>,
    skeletons: HashMap<u8, nf_formats::character::Skeleton>,
    nav: nf_game::nav::Network,
    covers: Vec<nf_game::cover::CoverNode>,
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: std::collections::HashMap<u32, nf_game::pickup::Model>,
    surfaces: nf_formats::sfx::SurfaceSounds,
    voices: Vec<nf_formats::sfx::VoiceLine>,
}

impl Level05 {
    fn load() -> Option<Level05> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).unwrap();
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok()).map(|k| (k.id as u8, k)).collect();
        let m = level.main_map().unwrap();
        let nav = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref().unwrap(), &world, 0x0700_0005);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let volumes = nf_game::aivolume::build(&level, 0x0700_0005);
        Some(Level05 {
            tables: drone::DroneAnimData::from_action_elf(&elf).unwrap(),
            props: drone::PropertyTables::from_action_elf(&elf).unwrap(),
            weapons: WeaponData::from_action_elf(&elf).unwrap(),
            sfx: nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap(),
            tuning: { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap(); nf_game::tuning::read_tuning_vars(&mut t, 0x0700_0005); t },
            hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap(),
            sequences: disc::level_sequences(&iso, "07000005.bin").unwrap(),
            models: nf_game::pickup::models(&level),
            surfaces: nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).unwrap(),
            voices: nf_formats::sfx::voice_lines(&elf).unwrap(),
            level, scripts, world, skeletons, nav, covers, volumes,
        })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_0005,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1,
                ai_points: &[],
            }
    }

    /// Every drone present at level start, and the index of the drone placed at `x`.
    fn drones(&self, x: f32) -> (Vec<Drone>, usize, Rng) {
        let d = self.data();
        let mut rng = Rng::new(5);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        let found = drones.iter().position(|dr| (dr.pos[0] - x).abs() < 0.01).unwrap();
        (drones, found, rng)
    }

    fn sim(&self, drones: Vec<Drone>, rng: Rng) -> Drones {
        let mut s = Drones::new(drones, rng); s.attach_level_id(&self.level, 0x0700_0005);
        s.difficulty = 1;
        s
    }
}

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None, crouched: false } }

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

/// The real-path resolution: the disc's datum-less weapons fail (`AnimDatumGetWeaponInfo`'s 0xFF
/// → −1 arm [M 0x1c7900:23-25]), an armed level-start guard resolves the held weapon's datum,
/// sound (with the 0 → 0xFFFF fixup [M 0x17a5a0 0x17a618-0x17a630]) and the muzzle bone world
/// transform (`AnimGetBoneWorldTrans` 0x1c3120: the posed path, not the fallback).
#[test]
fn weapon_info_resolves_the_held_weapon() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    // [D] census: the disc's weapon table carries datum-less records (the game's −1 arm is
    // exercisable on real data).
    let naked: Vec<usize> = (0..d.weapons.records.len()).filter(|&w| d.weapons.records[w][0x50] == 0xff).collect();
    assert!(!naked.is_empty(), "the disc carries a weapon with no model datum (of {})", d.weapons.records.len());
    // A datum-less weapon fails GetWeaponInfo even with a live pose [M 0x1c7900:23-25 → 0x17a5a0].
    let (drones, gi, rng) = l.drones(61.342);
    let mut sim = l.sim(drones, rng);
    let away = bond_at([0.0, -500.0, 0.0]);
    for _ in 0..10 { sim.tick(&d, &away); }
    let mut dr = sim.list[gi].clone();
    dr.weapon = naked[0] as u32;
    assert!(dr.weapon_info(&d).is_none(), "weapon {:#x} (+0x50 = 0xFF) → the game's −1 → return 0", naked[0]);
    // And ≥ 0x73 fails [M 0x1c7900:11].
    dr.weapon = 0x73;
    assert!(dr.weapon_info(&d).is_none(), "weapon byte 0x73 → fail");

    // The armed guard: GetWeaponInfo resolves. The level's patrol guard carries a gun; pick the
    // first drone whose weapon byte is armed and whose record has a model datum.
    let armed = l.drones(61.342).0.into_iter().find(|g| matches!(g.weapon, 2..=0x46 | 0x6a..=0x71) && d.weapons.records.get(g.weapon as usize).is_some_and(|r| r[0x50] != 0xff)).expect("the level starts an armed guard");
    let wi = armed.weapon_info(&d).expect("the armed guard's GetWeaponInfo resolves");
    let r = &d.weapons.records[armed.weapon as usize];
    // The datum byte rides through [M 0x1c7900:22].
    assert_eq!(wi.datum, r[0x50]);
    // The sound id with the 0 → 0xFFFF fixup [M 0x17a5a0].
    let raw = u16::from_le_bytes([r[0x60], r[0x61]]);
    assert_eq!(wi.sound, if raw == 0 { 0xffff } else { raw }, "sound fixup for weapon {:#x}", armed.weapon);
    // The muzzle is the posed bone (weapon datum + 1 hung on a skin bone), not the fallback
    // chest stand-in — a distinct point within 2.0 of the guard.
    let (s, c) = (nf_game::trig::sinf(armed.yaw), nf_game::trig::cosf(armed.yaw));
    let fallback = [armed.pos[0] + s * 0.5, armed.pos[1] + 0.4, armed.pos[2] + c * 0.5];
    assert!(dist(wi.muzzle, fallback) > 0.01, "posed muzzle {:?} vs fallback {fallback:?}", wi.muzzle);
    assert!(dist(wi.muzzle, armed.pos) < 2.0, "the gun bone sits on the guard: {:?}", wi.muzzle);
    // The direction = −(row 0) of a rigid world matrix → unit length [M 0x1c3120 chain].
    let len = (wi.dir[0] * wi.dir[0] + wi.dir[1] * wi.dir[1] + wi.dir[2] * wi.dir[2]).sqrt();
    assert!((len - 1.0).abs() < 1e-3, "dir is a unit axis: {len}");
    // [M 0x178520] the guard stands in the open at (61.34, 99.50) — the clamped-to-muzzle ray is
    // clear, so the through-wall probe is false (sister: `muzzle_through_wall`'s wall test pins
    // the true arm synthetically).
    assert!(!armed.gun_through_wall(&d), "an open-air guard's gun is not through a wall");
}

/// The 0 → 0xFFFF sound fixup [M 0x17a5a0 0x17a618-0x17a630] on a silent weapon with a model datum:
/// the patrol guard's weapon swapped to it, through the same live pose.
#[test]
fn silent_weapons_sound_0xffff() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let silent: Vec<usize> = (0..d.weapons.records.len()).filter(|&w| {
        let r = &d.weapons.records[w];
        r[0x50] != 0xff && u16::from_le_bytes([r[0x60], r[0x61]]) == 0
    }).collect();
    if silent.is_empty() { eprintln!("the disc has no silent weapon with a model datum"); return; }
    let (drones, gi, rng) = l.drones(61.342);
    let mut sim = l.sim(drones, rng);
    let away = bond_at([0.0, -500.0, 0.0]);
    for _ in 0..10 { sim.tick(&d, &away); }
    let mut g = sim.list[gi].clone();
    g.weapon = silent[0] as u32;
    let wi = g.weapon_info(&d).expect("the swapped weapon keeps a model datum and a live pose");
    assert_eq!(wi.sound, 0xffff, "weapon {:#x}: +0x60 = 0 → 0xFFFF [M 0x17a5a0]", silent[0]);
}
