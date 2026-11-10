//! fx-broken-object-bark — `Drone_BrokenObject` 0x138588 through the real kill path: Bond's
//! bullet breaks a 07000001 breakable (`Break_Update` 0x12bfd8's contact walk → `Break_Kill`
//! 0x12b970's else arm → `Drone_BrokenObject`) and every live type-0x0c guard barks
//! `0x40 + Rand(4)` [M 0x138688-0x1386a4 + `DroneFunc_ChooseSFX` 0x1497e8] — the voice rows
//! 0x40..0x43 are sfx 721/719/720/718, each with a facial id [L the parsed Drone_SfxInfo]. The
//! same break by an EXPLOSION barks nothing (its hitdata +0x58 is the breakable itself, kind
//! 0x20 — the kind-5 gate fails [M 0x17c518:163-164, 0x1385d8-0x1385e0]), the same break on
//! another level barks nothing (the `GameState+0xc == 0x7000001` gate [M 0x1385b0-0x1385bc]),
//! and a dead type-0x0c guard ABORTS the walk for every later drone [M 0x138628-0x138634
//! `c.ole.s` + `bc1tl` → return]. The bark lines play DIRECT on this level: the rows' facial
//! scripts (0x60006c4/0x60006c7/0x60006ca/0x60006cc) are absent from 07000001's anim-script set
//! [D the parsed kind-5 entries] — the game's hashtable-miss arm [M 0x141b00].
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::anim::Script;
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
    scripts: HashMap<u32, Script>,
    world: nf_game::world::CollisionWorld,
    sequences: HashMap<u32, nf_formats::anim::Sequence>,
    skeletons: HashMap<u8, nf_formats::character::Skeleton>,
    nav: nf_game::nav::Network,
    covers: Vec<nf_game::cover::CoverNode>,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: HashMap<u32, nf_game::pickup::Model>,
    skins: HashMap<u32, nf_formats::character::Skin>,
    voices: Vec<nf_formats::sfx::VoiceLine>,
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
}

impl Fixture {
    fn load(level_name: &str, level_id: u32) -> Option<Fixture> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let voices = nf_formats::sfx::voice_lines(&elf).unwrap();
        let sfx = nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap();
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
        Some(Fixture { tables, weapons, props, level, level_id, scripts, world, sequences, skeletons, nav, covers, volumes, models, skins, voices, sfx, tuning, hitboxes })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world,
            level: self.level_id, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav,
            covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: Some(&self.sfx), models: &self.models,
            surfaces: None, voices: Some(&self.voices), difficulty: 1, ai_points: &[] }
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

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, body: Default::default(), cel: None, obj_cel: None, move_local: [0.0; 2], riding: false, crouched: false } }

/// The bark's voice rows, read from the real `Drone_SfxInfo` [L the parsed table]:
/// 0x40..0x43 = sfx 721/719/720/718, all with a facial id and repeat 180.
const BARK_SFX: [u32; 4] = [721, 719, 720, 718];

fn bark_guard_voices(sim: &Drones) -> Vec<(usize, u32)> {
    // The game's walk filter [M 0x138618-0x138634]: the type byte +0xc5 == 0x0c and health
    // +0xac > 0 — nothing else (no present/enabled/corpse gate on the bark itself).
    sim.list.iter().enumerate()
        .filter(|(_, x)| x.dtype == 0x0c && x.health > 0.0)
        .filter_map(|(i, x)| x.voice.map(|v| (i, v.1)))
        .filter(|(_, sfx)| BARK_SFX.contains(sfx))
        .collect()
}

fn shoot(sim: &mut Drones, d: &Data, from: [f32; 3], at: [f32; 3]) {
    let n = ((at[0] - from[0]).powi(2) + (at[1] - from[1]).powi(2) + (at[2] - from[2]).powi(2)).sqrt();
    let dir = [(at[0] - from[0]) / n, (at[1] - from[1]) / n, (at[2] - from[2]) / n];
    let sound = nf_formats::sfx::SfxHearing { id: 0x403, inner: 0.0, outer: 0.0, weight: 0.0, looped: false, cull: false };
    sim.fire_player(d, from, dir, 2, sound); // weapon 2, the pistol round ai_live's pins drive
}

/// Bond shoots 07000001's placement-22 breakable (kind 0, health 0 — the first bullet kills it
/// [D the parsed placement; kind 2 = glass takes the shatter arm that never reaches the bark
/// [M 0x12b970:78-133]]): the kill goes through the bullet path, the guards bark — every live
/// type-0x0c drone of the list walk draws its own 0x40+Rand(4) line [M 0x138618-0x1386b0], and
/// the played sfx is one of the four bark rows [L 721/719/720/718].
#[test]
fn bond_breaks_a_castle_object_and_the_guards_bark() {
    let Some(fx) = Fixture::load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let sim = fx.sim();
    // The disc values this test pins [D]: the breakable row and the voice table.
    let br = &sim.breaks.list[0];
    assert_eq!(br.placement, 22, "07000001's first breakable is placement 22");
    assert_eq!(br.kind, 0, "and it is NOT glass (kind 2 never reaches the bark [M 0x12b970:78])");
    assert!(!br.src.records.is_empty(), "and it carries collision records");
    for (i, row) in [0x40usize, 0x41, 0x42, 0x43].iter().enumerate() {
        assert_eq!(fx.voices[*row].sfx, BARK_SFX[i], "voice row {row:#x} is bark sfx {}", BARK_SFX[i]);
        assert_ne!(fx.voices[*row].facial, 0, "row {row:#x} carries a facial id [L]");
    }
    let at = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
    let from = [at[0], at[1] - 0.4, at[2] + 1.5];
    let mut sim = sim;
    // Past the speech path's 30-frame post-load gate [M 0x141b00's lastFrame shape; drone.rs speech].
    for _ in 0..35 { sim.tick(&d, &bond_at([-100.0, 6.0, -100.0])); }
    let guards = sim.list.iter().filter(|x| x.dtype == 0x0c && x.health > 0.0).count();
    assert!(guards > 0, "07000001 fields live type-0x0c guards [D the level's drones: 20 drones, one dtype 0x0c]");
    assert!(bark_guard_voices(&sim).is_empty(), "no guard is barking before the break");
    let kills_before = sim.break_kills.len();
    shoot(&mut sim, &d, from, at);
    sim.tick(&d, &bond_at([-100.0, 6.0, -100.0]));
    assert!(sim.break_kills.len() > kills_before, "the bullet killed the breakable (the same Break_Kill path)");
    let barks = bark_guard_voices(&sim);
    assert!(!barks.is_empty(), "the broken-object bark played: {barks:?}");
}

/// The same break ON ANOTHER LEVEL barks nothing [M 0x1385b0-0x1385bc `GameState+0xc !=
/// 0x7000001` → return]: 07000002's placement-3 breakable (kind 0, health 0 [D]) dies through
/// the identical bullet path and no guard gains a bark voice.
#[test]
fn the_same_break_on_another_level_is_silent() {
    let Some(fx) = Fixture::load("07000002.bin", 0x0700_0002) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let sim = fx.sim();
    let br = &sim.breaks.list[0];
    assert_eq!(br.placement, 3, "07000002's first breakable is placement 3");
    assert_eq!(br.kind, 0, "and it is not glass");
    let at = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
    let from = [at[0], at[1] - 0.4, at[2] + 1.5];
    let mut sim = sim;
    for _ in 0..35 { sim.tick(&d, &bond_at([100.0, 6.0, 100.0])); }
    assert!(bark_guard_voices(&sim).is_empty());
    let kills_before = sim.break_kills.len();
    shoot(&mut sim, &d, from, at);
    sim.tick(&d, &bond_at([100.0, 6.0, 100.0]));
    assert!(sim.break_kills.len() > kills_before, "the bullet killed the breakable on 07000002 too");
    assert!(bark_guard_voices(&sim).is_empty(), "no bark off 07000001 [M 0x1385b0]");
}

/// The same break BY AN EXPLOSION barks nothing [M 0x17c518:163-164 — `Break_ApplyDamage`'s
/// hitdata +0x58 is the breakable itself (kind 0x20), which fails the kind-5 gate
/// 0x1385d8-0x1385e0 before any draw; even a player-thrown grenade]: the blast kills the same
/// placement-22 breakable through the explode path and no guard barks.
#[test]
fn an_explosion_break_barks_nothing() {
    let Some(fx) = Fixture::load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let mut sim = fx.sim();
    for _ in 0..35 { sim.tick(&d, &bond_at([-100.0, 6.0, -100.0])); }
    assert!(bark_guard_voices(&sim).is_empty());
    let br = &sim.breaks.list[0];
    assert_eq!(br.kind, 0, "the same non-glass breakable");
    let at = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
    let kills_before = sim.break_kills.len();
    // `Explode_Propagate` case 0x20 → `Break_ApplyDamage` [M 0x17c518:163-164] — the real blast
    // drain (`Drones::tick`'s pending_blasts slot), a lethal radius at the object.
    sim.pending_blasts.push((at, 3.0, 100.0, usize::MAX, 0x34, 0x0600_0052, 1.0, 0));
    sim.tick(&d, &bond_at([-100.0, 6.0, -100.0]));
    assert!(sim.break_kills.len() > kills_before, "the blast killed the breakable through the explode path");
    assert!(bark_guard_voices(&sim).is_empty(), "the explosion's break barks nothing [M 0x1385d8 kind gate]");
}

/// The walk's order semantics [M 0x138618-0x1386b0]: a DEAD type-0x0c drone ABORTS the whole
/// walk (`c.ole.s` + `bc1tl` → return — later drones get nothing), while a NON-0x0c drone is
/// skipped and the walk continues to the next 0x0c one.
#[test]
fn the_walk_aborts_at_a_dead_type0c_guard_and_skips_others() {
    let Some(fx) = Fixture::load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let br = &fx.sim().breaks.list[0];
    let at = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
    let from = [at[0], at[1] - 0.4, at[2] + 1.5];
    // A dead first 0x0c guard: the walk returns before ANY bark, even for later live guards.
    let mut sim = fx.sim();
    for _ in 0..35 { sim.tick(&d, &bond_at([-100.0, 6.0, -100.0])); }
    let first_0c = sim.list.iter().position(|x| x.dtype == 0x0c && x.health > 0.0).expect("a 0x0c guard");
    sim.list[first_0c].health = 0.0;
    shoot(&mut sim, &d, from, at);
    sim.tick(&d, &bond_at([-100.0, 6.0, -100.0]));
    assert!(sim.break_kills.len() > 0, "the breakable still died");
    assert!(bark_guard_voices(&sim).is_empty(), "the dead 0x0c guard aborts the walk [M 0x138634]");
    // The type gate: the same break with the level's only 0x0c guard re-typed (0xc → 2) barks
    // nothing [M 0x138620 `bnel v0, s3` → skip] — while the positive test above already shows
    // the walk reaching him PAST the dtype-0x02 drones in front of him in the list.
    let mut sim = fx.sim();
    for _ in 0..35 { sim.tick(&d, &bond_at([-100.0, 6.0, -100.0])); }
    let first_0c = sim.list.iter().position(|x| x.dtype == 0x0c && x.health > 0.0).expect("a 0x0c guard");
    sim.list[first_0c].dtype = 2;
    shoot(&mut sim, &d, from, at);
    sim.tick(&d, &bond_at([-100.0, 6.0, -100.0]));
    assert!(sim.break_kills.len() > 0, "the breakable still died");
    assert!(bark_guard_voices(&sim).is_empty(), "no 0x0c drone left, no bark [M 0x138620]");
}
