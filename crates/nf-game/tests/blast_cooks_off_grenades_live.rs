//! control-funcs-collide — `Explode_Propagate` 0x17c518 case 5: a blast reaching a grenade-family
//! bullet cooks it off (`Bullet_handle_object_destruction` 0x12cd98) at the FULL record power —
//! the falloff amount is only the reach gate. Red on the old code: the old blast sweep had no
//! case-5 arm, so a remote mine sitting in a frag's blast radius stayed live forever (the stuck
//! 0x37 has no fuse death — it only waits for the remote detonate), and no second blast ever
//! fired from its position.
//!
//! Real path: level 07000005, the level-start spot [L ai_live.rs — the snapshot's start position,
//! open ground]. Run 1 measures where the primer frag settles+explodes (the sim is deterministic:
//! bit-exact RNG, fixed 60 Hz); run 2 is a fresh sim from the same load with the remote mine
//! dropped at exactly that measured spot. No teleports after the load — the grenades fall and
//! fly through the real world from the start spot. Opt-in via `NF_ISO`.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drone, Drones, Player, Rng};
use std::collections::HashMap;

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
    models: HashMap<u32, nf_game::pickup::Model>,
    hud: nf_formats::hud::HudTables,
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
            hud: nf_formats::hud::HudTables::from_action_elf(&elf).unwrap(),
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

    fn sim(&self, drones: Vec<Drone>, rng: Rng) -> Drones {
        let mut s = Drones::new(drones, rng); s.attach_level_id(&self.level, 0x0700_0005); s.difficulty = 1; s
    }
}

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, body: Default::default(), cel: None, move_local: [0.0; 2], riding: false, crouched: false } }

/// The level-start spot [L ai_live.rs: the snapshot's start position] — open ground.
const SPOT: [f32; 3] = [104.954, 25.663, 108.825];

/// The frag 0x34 [L ee_snapshot.bin weapon_data 0x2c27c0: radius 8, damage 100, speed 0.1667,
/// +0x6C 0x8c08 (gravity), +0x70 0x280 (the fuse 0x200 + the bounce 0x80)], the 210-tick fuse
/// [M Bullet_init 0x12c5a0] — the grenade_rattle_live helper's values.
fn frag_drop(x: f32, y: f32, z: f32) -> nf_game::grenade::Grenade {
    nf_game::grenade::Grenade {
        pos: [x, y, z], velocity: [0.0, 0.0, 0.0], speed: 0.1667,
        travelled: 0.0, range: 210.0, damage: 100.0, radius: 8.0,
        weapon_type: 0x34, mesh: 0, owner: 0, state: 0, bounces: 0,
        smoke_popped: false, resting: false, stuck: false, seek_target: None, phase: None, burn: 0, defused: false,
        loop_handle: 0,
    }
}

/// The remote mine 0x37 [L ee_start_05.bin @0x2bf150 + 0x37·0x10C: radius 8, damage 100,
/// +0x68 0x14090 (the 0x4000 destruct gate SET), +0x70 0x18300 (the stick 0x100; 0x80204 has
/// 0x200)] — passes both cook-off gates, and once stuck it has NO fuse death (the stick re-arms
/// the fuse slot past the 7777 armed marker [M Bullet_CollisionHandler 0x12ce80:315;
/// grenade.rs's armed_mine]), so any blast at its position is the chain, never its own timer.
fn mine_drop(x: f32, y: f32, z: f32) -> nf_game::grenade::Grenade {
    nf_game::grenade::Grenade {
        pos: [x, y, z], velocity: [0.0, 0.0, 0.0], speed: 0.1667,
        travelled: 0.0, range: 420.0, damage: 100.0, radius: 8.0,
        weapon_type: 0x37, mesh: 0, owner: 0, state: 0, bounces: 0,
        smoke_popped: false, resting: false, stuck: false, seek_target: None, phase: None, burn: 0, defused: false,
        loop_handle: 0,
    }
}

/// Run the sim until the first blast fires; returns (tick, the blast record).
fn first_blast(sim: &mut Drones, d: &Data<'_>, bond: &Player, max_ticks: usize) -> (usize, (V3, f32, f32, usize, u16)) {
    for t in 0..max_ticks {
        sim.tick(d, bond);
        if let Some(b) = sim.explosions.first() {
            return (t, (b.0, b.1, b.2, b.3, b.4));
        }
    }
    panic!("no blast within {max_ticks} ticks");
}

type V3 = [f32; 3];

/// The chain: the primer frag's fuse blast (tick T) cooks the mine in reach — the mine's own
/// 8/100 blast fires from the mine's position at tick T+1 (the game's new explosion object
/// propagates on its first collide pass, the frame AFTER the cooking blast — the dispatch walk
/// never revisits the list head [M Collide_Update 0x1ea398:55-69 + the pending-blast drain
/// order]); the stuck mine itself is delete-marked (state 2, removed by the retain).
#[test]
fn blast_cooks_off_a_remote_mine_in_reach() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let away = bond_at([SPOT[0] + 40.0, SPOT[1], SPOT[2]]); // Bond away: no reactions
    let (drones, rng) = {
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        (nf_game::drone::level_drones(&d, &l.level, &mut rng, &mut lights), rng)
    };

    // Run 1: the primer alone — measure where the frag's fuse blast fires.
    let (rest, t_explode) = {
        let mut sim = l.sim(drones.clone(), Rng::new(1));
        for dr in &mut sim.list { dr.present = false; } // a pure physics drop (the rattle convention)
        sim.grenades.push(frag_drop(SPOT[0], SPOT[1], SPOT[2]));
        let (t, b) = first_blast(&mut sim, &d, &away, 400);
        let dist = ((b.0[0] - SPOT[0]).powi(2) + (b.0[2] - SPOT[2]).powi(2)).sqrt();
        assert!(dist < 4.0, "the primer explodes at the drop spot (dist {dist:.2})");
        assert_eq!(b.1, 8.0, "the frag blast radius is the record's 8.0 [L]");
        assert_eq!(b.2, 100.0, "the frag blast damage is the record's 100.0 [L]");
        eprintln!("[chain] primer blast at t={t} pos=({:.3},{:.3},{:.3})", b.0[0], b.0[1], b.0[2]);
        (b.0, t)
    };

    // Run 2: fresh sim, same deterministic load — the primer + the remote mine at the measured
    // spot (the mine falls a body's length onto the same ground and sticks there).
    let mut sim = l.sim(drones, Rng::new(1));
    for dr in &mut sim.list { dr.present = false; }
    sim.grenades.push(frag_drop(SPOT[0], SPOT[1], SPOT[2]));
    sim.grenades.push(mine_drop(rest[0], rest[1] + 0.5, rest[2]));
    let mut blasts: Vec<(usize, V3, f32, f32, u16)> = Vec::new();
    for t in 0..(t_explode + 4) {
        sim.tick(&d, &away);
        for b in &sim.explosions { blasts.push((t, b.0, b.1, b.2, b.4)); }
    }
    eprintln!("[chain] blasts: {:?}", blasts.iter().map(|(t, p, r, dm, w)| (*t, w, r, dm, (p[0] * 10.0).round() / 10.0, (p[2] * 10.0).round() / 10.0)).collect::<Vec<_>>());
    // The primer's blast at t_explode, then the MINE's cook-off at t_explode + 1 from the mine's
    // position, with the mine record's own power [L 8.0/100.0] — full power, no falloff on the
    // amount [M 0x17c518 case 5 — the case never reads the amount].
    assert!(blasts.iter().any(|(t, p, r, dm, w)| *t == t_explode && *w == 0x34 && *r == 8.0 && *dm == 100.0
        && (p[0] - rest[0]).abs() < 1.0 && (p[2] - rest[2]).abs() < 1.0), "the primer blast at t={t_explode}");
    let cooked = blasts.iter().find(|(t, p, r, dm, w)| *t == t_explode + 1 && *w == 0x37)
        .unwrap_or_else(|| panic!("no cook-off blast at t={} — the case-5 sweep is missing (red on the old code); blasts={blasts:?}", t_explode + 1));
    assert_eq!(cooked.2, 8.0, "the cook-off blast carries the mine record's radius [M 0x12cd98 rec+0x08]");
    assert_eq!(cooked.3, 100.0, "the cook-off blast carries the mine record's damage [M 0x12cd98 rec+0x0C]");
    assert!((cooked.1[0] - rest[0]).abs() < 1.5 && (cooked.1[2] - rest[2]).abs() < 1.5,
        "the cook-off fires at the mine's position");
    // The stuck mine itself is gone (delete-marked [M 0x12cd98:17 obj+0xFE |= 1]).
    assert!(!sim.grenades.iter().any(|g| g.weapon_type == 0x37 && g.state < 2), "the cooked mine is removed");
}

/// The reach gate: the same mine just OUTSIDE the primer's 8-unit blast radius stays live —
/// the falloff gate drops it [M 0x17c518:82-88] and no second blast ever fires from its spot.
#[test]
fn blast_out_of_reach_leaves_the_mine_live() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let away = bond_at([SPOT[0] + 40.0, SPOT[1], SPOT[2]]);
    let (drones, rng) = {
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        (nf_game::drone::level_drones(&d, &l.level, &mut rng, &mut lights), rng)
    };
    // Measure the primer again (deterministic — the same rest spot).
    let (rest, t_explode) = {
        let mut sim = l.sim(drones.clone(), Rng::new(1));
        for dr in &mut sim.list { dr.present = false; }
        sim.grenades.push(frag_drop(SPOT[0], SPOT[1], SPOT[2]));
        let (t, b) = first_blast(&mut sim, &d, &away, 400);
        (b.0, t)
    };
    // Run 2: the mine 10 units sideways — outside the 8-unit blast sphere.
    let mut sim = l.sim(drones, Rng::new(1));
    for dr in &mut sim.list { dr.present = false; }
    sim.grenades.push(frag_drop(SPOT[0], SPOT[1], SPOT[2]));
    sim.grenades.push(mine_drop(rest[0] + 10.0, rest[1] + 0.5, rest[2]));
    for _ in 0..(t_explode + 4) { sim.tick(&d, &away); }
    assert!(sim.explosions.iter().all(|b| b.4 != 0x37), "no blast from the out-of-reach mine");
    assert!(sim.grenades.iter().any(|g| g.weapon_type == 0x37 && g.state < 2), "the out-of-reach mine is still live (no fuse death)");
}

/// The tripbomb 0x3a [L ee_start_05.bin @0x2bf150 + 0x3a·0x10C: radius 8, damage 100, +0x70
/// carries 0x80000 — the attached-beam tail]. No fuse bit: once stuck it lives until something
/// sets it off.
fn trip_drop(x: f32, y: f32, z: f32) -> nf_game::grenade::Grenade {
    nf_game::grenade::Grenade { weapon_type: 0x3a, ..mine_drop(x, y, z) }
}

/// The cook-off's `+0x70 & 0x80000` tail [M Bullet_handle_object_destruction 0x12cd98:28-34]:
/// the cooked tripbomb's attached beam object (BU+0x38) is delete-marked in the same call, so
/// the beam dies with the round on the cooking frame. Red on the old code: the tick rebuilds
/// `tripbomb_beams` BEFORE the blast drain, so the cooked bomb's beam stayed in the list for
/// that tick after the round itself was gone.
#[test]
fn blast_cooking_a_tripbomb_drops_its_beam_the_same_tick() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let away = bond_at([SPOT[0] + 40.0, SPOT[1], SPOT[2]]);
    let (drones, rng) = {
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        (nf_game::drone::level_drones(&d, &l.level, &mut rng, &mut lights), rng)
    };
    let (rest, t_explode) = {
        let mut sim = l.sim(drones.clone(), Rng::new(1));
        for dr in &mut sim.list { dr.present = false; }
        sim.grenades.push(frag_drop(SPOT[0], SPOT[1], SPOT[2]));
        let (t, b) = first_blast(&mut sim, &d, &away, 400);
        (b.0, t)
    };
    let mut sim = l.sim(drones, Rng::new(1));
    for dr in &mut sim.list { dr.present = false; }
    sim.grenades.push(frag_drop(SPOT[0], SPOT[1], SPOT[2]));
    sim.grenades.push(trip_drop(rest[0] + 1.0, rest[1] + 0.5, rest[2]));
    for _ in 0..t_explode { sim.tick(&d, &away); }
    let bomb = sim.grenades.iter().find(|g| g.weapon_type == 0x3a && g.state < 2)
        .expect("the dropped tripbomb is live before the primer blast");
    assert!(bomb.stuck && bomb.phase.as_ref().map_or(false, |p| p.armed),
        "the tripbomb stuck and armed before the frag's fuse ran out (stuck {} phase {:?})", bomb.stuck, bomb.phase.as_ref().map(|p| p.armed));
    let bomb_pos = bomb.pos;
    assert!(sim.tripbomb_beams.iter().any(|b| b.0 == bomb_pos), "the armed tripbomb's beam is up");
    // The primer's tick: its blast reaches the bomb 1 u away and cooks it.
    sim.tick(&d, &away);
    assert!(sim.explosions.iter().any(|b| b.4 == 0x34), "the primer blast fires on the measured tick");
    assert!(!sim.grenades.iter().any(|g| g.weapon_type == 0x3a && g.state < 2), "the cooked tripbomb is removed");
    assert!(!sim.tripbomb_beams.iter().any(|b| b.0 == bomb_pos),
        "the cooked tripbomb's beam dies with it on the cooking tick [M 0x12cd98 +0x70 & 0x80000 → BU+0x38]");
    sim.tick(&d, &away);
    assert!(sim.explosions.iter().any(|b| b.4 == 0x3a), "the tripbomb's own 8/100 blast follows on the next tick");
}
