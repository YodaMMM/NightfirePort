//! The per-side obstacle-bit gates on the guard's dodge/step/roll pairs (drone-probe-side-gates):
//! the game's six Can* probes each gate on their drone+0x31c side bit — 8 = an obstacle to the
//! LEFT blocks every Left probe, 0x10 = RIGHT blocks every Right probe [M 0x14fdf8/0x14ffc0/
//! 0x14f640/0x14f808/0x150338/0x150500 heads] — and the right side of the dodge/strafe/step pairs
//! additionally demands the right-side anim 0x2a / 0xd / 0x5b [M 0x14ffc0/0x14fbe0/0x14f808].
//! The side meaning itself is proven at the `no_bunching` field doc (the NoBunching disassembly +
//! the obj matrix columns). These tests drive the real combat path (level 07000005, the
//! guard_reaction_live combat spot: Bond walks up, the guard sights, aims, and picks combat moves)
//! and inject only the +0x31c mirror (`no_bunching`, pub) — the tick scan that would write it is
//! held inert by a prior session (NB_SCAN_LIVE, the walkreal_07000001 re-pin), so injection is the
//! only way to reach the gates; the pre-fix pair() ignored the bits and the right-side anims
//! entirely. Opt-in: `NF_ISO`.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drone, Drones, Player, Rng, state};
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
    models: std::collections::HashMap<u32, nf_game::pickup::Model>,
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

    fn sim(&self, drones: Vec<Drone>, rng: Rng) -> Drones {
        let mut s = Drones::new(drones, rng); s.attach_level_id(&self.level, 0x0700_0005);
        s.difficulty = 1;
        s
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_0005,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1,
                ai_points: &[],
            }
    }

    fn drones_seeded(&self, alert: Option<f32>, x: f32, seed: u32) -> (Vec<Drone>, usize, Rng) {
        let seed = std::env::var("NF_TEST_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(seed);
        let d = self.data();
        let mut rng = Rng::new(seed);
        let mut lights = nf_game::light::LightList::default();
        let mut drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        if let Some(a) = alert { for dr in &mut drones { dr.alert = a; dr.alert_floor = a; } }
        let found = drones.iter().position(|dr| (dr.pos[0] - x).abs() < 0.01).unwrap();
        (drones, found, rng)
    }
}

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None } }

const GUARD_X: f32 = 61.342;
const BOND: [f32; 3] = [67.31, 26.5, 99.47];
const FAR: [f32; 3] = [0.0, -500.0, 0.0];
const WARM: u32 = 10;
const RUN: u32 = 400;

/// The dodge/step/roll move states the pairs feed [M ChooseCombatMove 0x150c18 case bodies].
const LEFT_MOVES: [u8; 3] = [0x77, 0x73, 0x79]; // STRAFE_DODGE_LEFT / STEP_AIM_LEFT / ROLL_LEFT_CROUCH
const RIGHT_MOVES: [u8; 3] = [0x78, 0x74, 0x7a];

fn has_move(log: &[(u32, u8)], moves: &[u8]) -> Option<u8> {
    log.iter().find(|(_, s)| moves.contains(s)).map(|(_, s)| *s)
}

/// Warm with Bond far, then bring Bond to the combat spot and tick until the guard is fighting
/// from AIM_STAND_FIRE. Returns after `aim_frame` is found (or the window runs out).
fn bring_to_combat(l: &Level05, seed: u32) -> Option<(Drones, usize, u32)> {
    let d = l.data();
    let (drones, gi, rng) = l.drones_seeded(None, GUARD_X, seed);
    let mut sim = l.sim(drones, rng);
    for _ in 0..WARM { sim.tick(&d, &bond_at(FAR)); }
    let mut aim = None;
    for f in 0..300 {
        sim.tick(&d, &bond_at(BOND));
        if aim.is_none() && sim.list[gi].state == state::AIM_STAND_FIRE { aim = Some(sim.frame); }
        if aim.is_some() && sim.frame >= aim.unwrap() + 20 { break; }
    }
    aim.map(|a| (sim, gi, a))
}

/// Which anims the guard's set can do — the census the tests pick from (printed for the report).
fn census(l: &Level05, gi: usize, drones: &[Drone]) {
    let d = l.data();
    let set = drones[gi].anim_set;
    let has = |a: u16| d.tables.script_for(a, set).is_some();
    eprintln!("[census] guard {gi} anim_set {set:#x}: 0x29 {} 0x2a {} | 0xc {} 0xd {} | 0x53 {} | 0x5a {} 0x5b {}",
        has(0x29), has(0x2a), has(0xc), has(0xd), has(0x53), has(0x5a), has(0x5b));
}

/// A guard with the LEFT obstacle bit set never enters a left dodge/step/roll: the Left Can*
/// probes' first gate is drone+0x31c bit 8 clear [M 0x14fdf8/0x14f640/0x150338 heads], so the
/// pair must fall through to the right side (or nothing). Injection: no_bunching = 8, stamped at
/// `injected_at` — only the log after that frame counts (the pre-injection warm may have moved).
#[test]
fn left_bit_blocks_every_left_move() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let Some((mut sim, gi, _aim_f)) = bring_to_combat(&l, 5) else { panic!("the guard never reached AIM_STAND_FIRE") };
    census(&l, gi, &sim.list);
    let injected_at;
    {
        let g = &mut sim.list[gi];
        g.no_bunching = 8; // an obstacle to the guard's LEFT (the proven side meaning)
        injected_at = sim.frame;
    }
    let d = l.data();
    for _ in 0..RUN { sim.tick(&d, &bond_at(BOND)); }
    let g = &sim.list[gi];
    let after: Vec<(u32, u8)> = g.log.iter().filter(|(f, _)| *f > injected_at).copied().collect();
    let hit = after.iter().find(|(_, s)| LEFT_MOVES.contains(s)).map(|(_, s)| *s);
    assert!(hit.is_none(), "guard entered left move {hit:?} with the left bit set (post-injection log {after:?})");
}

/// Sister: with the bit clear the left side IS picked in some seeds — the guard at this spot
/// makes left moves at all (the block test is not vacuously green).
#[test]
fn bit_clear_left_is_picked_in_some_seeds() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let mut left_seeds = Vec::new();
    for seed in 1..=8u32 {
        let Some((mut sim, gi, _)) = bring_to_combat(&l, seed) else { continue };
        let d = l.data();
        for _ in 0..RUN { sim.tick(&d, &bond_at(BOND)); }
        let g = &sim.list[gi];
        eprintln!("[seed {seed}] moves {:?}", g.log.iter().map(|(_, s)| *s).filter(|s| [0x73u8, 0x74, 0x77, 0x78, 0x79, 0x7a].contains(s)).collect::<Vec<_>>());
        if let Some(m) = has_move(&g.log, &LEFT_MOVES) { left_seeds.push((seed, m)); }
    }
    assert!(!left_seeds.is_empty(), "no seed picked a left dodge/step/roll with the bit clear — the block test is vacuous; moves seen: check the [seed] lines");
}

/// Sister of the left-bit test on the right side: with the RIGHT obstacle bit (0x10) set, the
/// Right Can* probes' head gate fails [M 0x14ffc0/0x14f808/0x150500] and the guard never enters a
/// right dodge/step/roll; the strafe/step states that do appear must be left-side ones.
#[test]
fn right_bit_blocks_every_right_move() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let Some((mut sim, gi, _)) = bring_to_combat(&l, 5) else { panic!("the guard never reached AIM_STAND_FIRE") };
    let injected_at;
    {
        let g = &mut sim.list[gi];
        g.no_bunching = 0x10; // an obstacle to the guard's RIGHT
        injected_at = sim.frame;
    }
    let d = l.data();
    for _ in 0..RUN { sim.tick(&d, &bond_at(BOND)); }
    let g = &sim.list[gi];
    let after: Vec<(u32, u8)> = g.log.iter().filter(|(f, _)| *f > injected_at).copied().collect();
    let hit = after.iter().find(|(_, s)| RIGHT_MOVES.contains(s)).map(|(_, s)| *s);
    assert!(hit.is_none(), "guard entered right move {hit:?} with the right bit set (post-injection log {after:?})");
}

/// The right side of each pair demands the right anim as well as the left one — dodge 0x29+0x2a
/// [M 0x14ffc0], strafe 0xc+0xd [M 0x14fbe0], step 0x5a+0x5b [M 0x14f808]. The port's per-side
/// can_do gate is wired to that shape, but the census proves the asymmetry NEVER OCCURS in the
/// shipped data (all 27 anim sets, through can_do's own-danim remap): every set that can do a
/// pair's left anim can do its right one too. So there is no guard the gate ever rejects — the
/// test pins that fact (a future table change that introduces a lopsided set keeps the gate
/// live, and this census is where the disposition is re-checked).
#[test]
fn right_anim_missing_no_right_move() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    // The census across every set row, through the port's own can_do mapping (the anim state's
    // `own` danim first, then the set's script row with the set-0 fallback — script_for alone
    // skips the remap and misreports the lopsided sets).
    let can = |anim: u16, set: u16| -> bool {
        d.tables.states.get(anim as usize).is_some_and(|(own, _)| d.tables.script_for(*own, set).is_some())
    };
    let mut step_lopsided = Vec::new();
    let mut dodge_lopsided = Vec::new();
    let mut strafe_lopsided = Vec::new();
    let max_set = d.tables.tables.first().map(|r| r.len()).unwrap_or(0);
    for set in 0..max_set as u16 {
        if can(0x5a, set) && !can(0x5b, set) { step_lopsided.push(set); }
        if can(0x29, set) && !can(0x2a, set) { dodge_lopsided.push(set); }
        if can(0xc, set) && !can(0xd, set) { strafe_lopsided.push(set); }
    }
    eprintln!("[census] sets {max_set}: step-lopsided {step_lopsided:?}, dodge-lopsided {dodge_lopsided:?}, strafe-lopsided {strafe_lopsided:?}");
    let Some((mut sim, gi, _)) = bring_to_combat(&l, 5) else { panic!("the guard never reached AIM_STAND_FIRE") };
    let real_set = sim.list[gi].anim_set;
    eprintln!("[census] the combat guard's set {real_set:#x}");
    let Some(&lop) = step_lopsided.first() else {
        eprintln!("[census] no set has 0x5a without 0x5b — the asymmetry never occurs in the game data; nothing to pin");
        return;
    };
    sim.list[gi].anim_set = lop;
    let swapped_at = sim.frame; // the log is cumulative — only post-swap entries count
    let d2 = l.data();
    for _ in 0..RUN { sim.tick(&d2, &bond_at(BOND)); }
    let g = &sim.list[gi];
    let post: Vec<u8> = g.log.iter().filter(|(f, _)| *f > swapped_at).map(|(_, s)| *s).collect();
    let right_step = post.iter().find(|s| **s == 0x74).copied();
    let left_step = post.iter().find(|s| **s == 0x73).copied();
    eprintln!("[lopsided set {lop:#x}] post-swap moves {post:?} (left 0x73: {left_step:?}, right 0x74: {right_step:?})");
    assert!(right_step.is_none(), "guard entered right step 0x74 with no 0x5b in set {lop:#x} (pre-fix: the pair never checked it)");
}
