//! The sensor's target radius: the game's cone/range tests read the player's WORLD SPHERE
//! (`obj+0x80` centre, `obj+0x8C` radius [M Sensor_InCone 0x1af4f0:60-77 — `Intersect_ConeSphere`
//! gets the target's sphere tag, and the range gate is `sqdist ≤ (range + obj+0x8C)²`]). The
//! player's sphere radius is **1.0576** [L — the fresh-boot EE read recorded in
//! docs/fields/obj-tag-player.md's +0x80 row, built from the AnimObject local sphere by
//! `Control_BuildWorldSph` 0x133070 [M; docs/fields/glist.md]]. The port's stand-in 0.4 failed
//! the range gate at distances the game detects.
//!
//! Pinned here through the real level + sim path (0700000b, sensor 164: cone 15°, range 5,
//! out ch 0x28 [D]):
//! * the player parked 5.5 units out on the sweep bisector — outside the stand-in gate
//!   (5 + 0.4 = 5.4 < 5.5) but inside the game's (5 + 1.0576 = 6.06 > 5.5) — must trip ch 0x28;
//!   this test FAILS on the pre-task tree (the gate returns false every tick);
//! * the sister at 6.5 units — beyond the game's own gate (6.06 < 6.5) — must never trip,
//!   pinning the gate's upper edge;
//! * the empty-volume sister: sensor 172 (out ch 0x27) stays off the whole run.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::door::interp;
use nf_game::drone::{Data, Drones, Player, Rng};
use std::collections::HashMap;

struct Level0b {
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
    models: HashMap<u32, nf_game::pickup::Model>,
    surfaces: nf_formats::sfx::SurfaceSounds,
    voices: Vec<nf_formats::sfx::VoiceLine>,
}

impl Level0b {
    fn load() -> Option<Level0b> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000b.bin")).unwrap();
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok()).map(|k| (k.id as u8, k)).collect();
        let m = level.main_map().unwrap();
        let nav = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref().unwrap(), &world, 0x0700_000b);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let volumes = nf_game::aivolume::build(&level, 0x0700_000b);
        Some(Level0b {
            tables: drone::DroneAnimData::from_action_elf(&elf).unwrap(),
            props: drone::PropertyTables::from_action_elf(&elf).unwrap(),
            weapons: WeaponData::from_action_elf(&elf).unwrap(),
            sfx: nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap(),
            tuning: { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap(); nf_game::tuning::read_tuning_vars(&mut t, 0x0700_000b); t },
            hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap(),
            sequences: disc::level_sequences(&iso, "0700000b.bin").unwrap(),
            models: nf_game::pickup::models(&level),
            surfaces: nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).unwrap(),
            voices: nf_formats::sfx::voice_lines(&elf).unwrap(),
            level, scripts, world, skeletons, nav, covers, volumes,
        })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_000b,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1, ai_points: &[] }
    }
}

/// Run the sim with the player's sphere centre parked `dist` units out on sensor 164's t=0.5
/// sweep bisector; return whether ch 0x28 (sensor 164's out) or ch 0x27 (sister 172's) turned on,
/// and the first tick of the ch-0x28 write.
fn run_at(dist: f32) -> (Option<u32>, bool) {
    let Some(l) = Level0b::load() else { eprintln!("skip: NF_ISO not set"); return (None, true) };
    let d = l.data();
    let mut rng = Rng::new(0x1234_5678);
    let mut lights = nf_game::light::LightList::default();
    let drones = nf_game::drone::level_drones(&d, &l.level, &mut rng, &mut lights);
    let mut sim = Drones::new(drones, rng);
    sim.attach_level_id(&l.level, 0x0700_000b);
    sim.difficulty = 1;
    sim.npc_enabled = false; // freeze the patrol so the sight line stays clear (sensor_live's pin)
    let s164 = sim.sensors.list.iter().find(|s| s.placement == 164).expect("sensor 164 [D]").clone();
    let bisector = interp(&s164.keys, 0, s164.limit, 0.5, true).expect("sensor 164 has a sweep");
    let dir = [bisector[2][0], bisector[2][1], bisector[2][2]];
    let at = [s164.pos[0] + dist * dir[0], s164.pos[1] + dist * dir[1], s164.pos[2] + dist * dir[2]];
    assert!(l.world.line_of_sight(s164.pos, at), "precondition: clear static LOS to the {dist} spot [M 0x1af4f0:84-87]");
    let mut bond = Player { pos: [at[0], at[1] - 0.9, at[2]], ..Default::default() };
    bond.target = bond.pos;
    bond.sph = at; // the world-sphere centre the cone test reads [M 0x1af4f0:61]
    let mut first_on = None;
    for t in 0..700u32 {
        sim.tick(&d, &bond);
        if sim.channels.on(0x28) && first_on.is_none() { first_on = Some(t); }
    }
    (first_on, sim.channels.on(0x27))
}

/// The stand-in-failing half: 5.5 units out — inside the game's range gate (5 + 1.0576), outside
/// the old stand-in's (5 + 0.4 = 5.4) — must trip.
#[test]
fn the_world_sphere_radius_reaches_the_game_range_gate() {
    let (first_on, sister) = run_at(5.5);
    let first_on = first_on.expect("5.5 ≤ range 5 + obj+0x8C 1.0576 → the game detects [M 0x1af4f0:60-62 + L 1.0576]");
    assert!(first_on >= 152, "the trip still needs 32 alert + 120 sight ticks (first at {first_on}) [M 0x1af618:211-216,154-157]");
    assert!(!sister, "sister: sensor 172's cone never holds the player [M 0x1af618:169-171]");
}

/// Sister: 6.5 units out — beyond the game's own gate (5 + 1.0576 = 6.06 < 6.5) — must never trip,
/// pinning the gate's upper edge with the real radius.
#[test]
fn sister_beyond_the_game_gate_never_trips() {
    let (first_on, sister) = run_at(6.5);
    assert!(first_on.is_none(), "6.5 > range 5 + 1.0576 → the gate rejects [M 0x1af4f0:60-62 + L 1.0576]");
    assert!(!sister, "sister: sensor 172 stays off [M 0x1af618:169-171]");
}
