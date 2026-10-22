//! The `DroneVision_GunThroughWall` 0x178520 hold through the real sim path: a guard whose muzzle
//! is buried in a wall AIMS but does not send the fire request (`+0x3b = !+0x1e` [M
//! 0x166328:132,141-147 AimStandFire case 3; 0x167060:72,92-99 AimCrouchFire case 3]) — the wall
//! burial is built from the guard's own measured aim-pose muzzle (`DroneWeap_GetWeaponInfo`
//! 0x17a5a0 → `AnimGetBoneWorldTrans` 0x1c3120), not injected state: the flag only turns via the
//! sight tick (`SeekOpponentLOS` 0x175cb8:68-82 → `gun_through_wall = los && aims_torso && probe`),
//! and the game's unconditional arms stay game-true (entry fire 0x166328:59-62 / 0x167060:27-29,
//! timer 0xC 0x166328:180-183). The probe ray itself (`DroneFunc_ClampPosToCollisionCylinder`
//! 0x1499d0 0.4-cylinder start → raw muzzle, `Collide_RayIntersect` mask 0x927 [M 0x178520:38-52])
//! runs against the harness wall; everything else (tables, props, skins, poses, the state machine)
//! is the disc's, through `Drones::tick` with Bond's position as the only input. Opt-in: `NF_ISO`.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{state, Data, Drone, Drones, Player, Rng};
use nf_game::world::CollisionWorld;
use std::collections::HashMap;

/// Everything `Data` borrows, owned (the ai_live/guard_reaction_live harness).
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

    /// `Data` over a caller-chosen collision world (the disc world for the probe run, the harness
    /// floor(+wall) world for the hold/sister runs — the sight, the probe ray and the collision all
    /// read the same world the game reads).
    fn data_with_world<'a>(&'a self, world: &'a CollisionWorld) -> Data<'a> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world, level: 0x0700_0005,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1,
                ai_points: &[],
            }
    }

    /// The disc's data (the real collision world).
    fn data(&self) -> Data<'_> { self.data_with_world(&self.world) }

    /// The lone courtyard guard at GUARD_X (the ai_live notice spot), alertness 1.0 (the
    /// guard-reaction path-2 setup: no reaction floor), the rest of the level's guards dropped so
    /// the harness world's open floor keeps stray sight lines out of the scenario. No drone state
    /// is injected beyond the harness's alertness seed (the guard-reaction path-2 precedent).
    fn lone_guard(&self, alert: f32, seed: u32) -> (Vec<Drone>, Rng) {
        let d = self.data();
        let mut rng = Rng::new(seed);
        let mut lights = nf_game::light::LightList::default();
        let mut drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        drones.retain(|dr| (dr.pos[0] - GUARD_X).abs() < 0.01);
        assert_eq!(drones.len(), 1, "the courtyard guard at {GUARD_X}");
        for dr in &mut drones { dr.alert = alert; dr.alert_floor = alert; }
        (drones, rng)
    }

    fn sim<'a>(&'a self, drones: Vec<Drone>, rng: Rng, world: &'a CollisionWorld) -> (Drones, Data<'a>) {
        let d = self.data_with_world(world);
        let mut s = Drones::new(drones, rng);
        s.attach_level_id(&self.level, 0x0700_0005);
        s.difficulty = 1;
        (s, d)
    }
}

/// Where the guard stands (the ai_live notice spot) and where Bond appears: 6 units along +x (the
/// stand-fire band, ≥ 6 [M 0x173280]), same height — the sight ray runs level over the wall.
const GUARD_X: f32 = 61.342;
const FAR: [f32; 3] = [0.0, -500.0, 0.0];
const WARM: u32 = 10;
const RUN: u32 = 300;

/// The scenario search: the case-3 fire request (the hold's target, 0x166328:141-147) only rides
/// on an AIM_STAND_FIRE update with `combat_state == 0` OUTSIDE a burst — the observed cadence
/// sidesteps (0x73/0x74) after the first burst, so the hold needs a (seed, distance) whose guard
/// STAYS in the aim-fire state across a burst boundary. Search seeds × distances and keep the
/// first whose first aim phase runs 45+ ticks continuously (the burst lands ~21-29 ticks in).
fn find_scenario(l: &Level05) -> (u32, f32) {
    for seed in 1..=12u32 {
        for dx in [7.0f32, 9.0, 5.0, 6.0] {
            let (drones, rng) = l.lone_guard(1.0, seed);
            let world = CollisionWorld::from_triangles(floor_tris(drones[0].pos[1] - 1.0));
            let (mut sim, d) = l.sim(drones, rng, &world);
            let bond = bond_at([GUARD_X + dx, sim.list[0].pos[1], 99.47]);
            for _ in 0..WARM { sim.tick(&d, &bond_at(FAR)); }
            let mut run = 0;
            let mut fired = false;
            for _ in 0..120 {
                sim.tick(&d, &bond);
                let g = &sim.list[0];
                if matches!(g.state, state::AIM_STAND_FIRE | state::AIM_CROUCH_FIRE) {
                    run += 1;
                    if !g.shots.is_empty() { fired = true; }
                    if run >= 45 && fired {
                        eprintln!("[scenario] seed {seed} dx {dx}: aim run {run}, shots {}", g.shots.len());
                        return (seed, dx);
                    }
                } else if !g.shots.is_empty() || run > 0 { break; }
            }
        }
    }
    panic!("no (seed, distance) keeps the guard in the aim-fire state across a burst boundary");
}

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None } }

/// A flat floor at `y` (feet height, both windings so every ray family hits) spanning the yard.
fn floor_tris(y: f32) -> Vec<[[f32; 3]; 3]> {
    let (x0, x1, z0, z1) = (GUARD_X - 8.0, GUARD_X + 14.0, 91.0, 108.0);
    vec![
        [[x0, y, z0], [x1, y, z1], [x1, y, z0]],
        [[x0, y, z0], [x0, y, z1], [x1, y, z1]],
        [[x0, y, z0], [x1, y, z0], [x1, y, z1]],
        [[x0, y, z0], [x1, y, z1], [x0, y, z1]],
    ]
}

/// A vertical wall at `x` spanning `z0..z1`, from the floor up to `top` (both windings).
fn wall_tris(x: f32, z0: f32, z1: f32, y0: f32, top: f32) -> Vec<[[f32; 3]; 3]> {
    vec![
        [[x, y0, z0], [x, top, z1], [x, top, z0]],
        [[x, y0, z0], [x, y0, z1], [x, top, z1]],
        [[x, y0, z0], [x, top, z0], [x, top, z1]],
        [[x, y0, z0], [x, top, z1], [x, y0, z1]],
    ]
}

/// One sampled aim-pose muzzle of the lone guard (the disc's pose pipeline, standing in the aim
/// state facing Bond) plus its head/eye line — the numbers the harness wall is derived from.
struct AimMuzzle { muzzle: [f32; 3], head_y: f32, pos: [f32; 3] }

/// Trace the guard's FIRST standing-aim phase (entry → the sidestep at ~30 ticks): every tick's
/// muzzle/pos while `AIM_STAND_FIRE`/`AIM_CROUCH_FIRE`. The wall derives from the phase's extremes
/// so it buries the muzzle across the whole pose family (the pose whose case-3 fire requests ride).
fn trace_aim_phase(l: &Level05, drones: Vec<Drone>, rng: Rng, floor_y: f32, dx: f32) -> (Vec<AimMuzzle>, f32) {
    let probe = CollisionWorld::from_triangles(floor_tris(floor_y));
    let (mut sim, d) = l.sim(drones, rng, &probe);
    let bond = bond_at([GUARD_X + dx, sim.list[0].pos[1], 99.47]);
    for _ in 0..WARM { sim.tick(&d, &bond_at(FAR)); }
    let mut out = Vec::new();
    let mut head_y = 0.0;
    for _ in 0..60 {
        sim.tick(&d, &bond);
        let dr = &sim.list[0];
        if matches!(dr.state, state::AIM_STAND_FIRE | state::AIM_CROUCH_FIRE) {
            let wi = dr.weapon_info(&d).expect("the armed guard's GetWeaponInfo resolves in the aim pose");
            head_y = dr.head()[1];
            out.push(AimMuzzle { muzzle: wi.muzzle, head_y, pos: dr.pos });
            if out.len() >= 45 { break; }
        } else if !out.is_empty() {
            break; // the phase ended — the scenario search guarantees 45 continuous ticks
        }
    }
    assert!(out.len() >= 40, "the aim phase was too short to trace: {} samples, log {:?}", out.len(), sim.list[0].log);
    (out, head_y)
}

/// The harness wall from the traced pose family: the face at the phase's MINIMUM muzzle bearing
/// distance − 0.10 (past the 0.4 clamp start [M 0x1499d0:36-43], so the probe ray
/// `clamp → muzzle` [M 0x178520:38-52] crosses at every sampled pose), the top at the phase's
/// MAXIMUM muzzle y + 0.05, under the level sight line (head → Bond target, level over the wall).
fn harness_wall(l: &Level05) -> (CollisionWorld, f32, f32, [f32; 3], u32, f32) {
    let (seed, dx) = find_scenario(l);
    let (drones, rng) = l.lone_guard(1.0, seed);
    let floor_y = drones[0].pos[1] - 1.0;
    let (trace, head_y) = trace_aim_phase(l, drones, rng, floor_y, dx);
    let m = &trace[trace.len() / 2];
    let mut min_d = f32::MAX;
    let mut max_y = f32::MIN;
    let mut sum = [0.0f32; 2];
    for s in &trace {
        let dx = s.muzzle[0] - s.pos[0];
        let dz = s.muzzle[2] - s.pos[2];
        min_d = min_d.min((dx * dx + dz * dz).sqrt());
        max_y = max_y.max(s.muzzle[1]);
        sum[0] += dx; sum[1] += dz;
    }
    // The bearing: the trace's mean muzzle offset (the torso tracks Bond; the sweep is small).
    let bl = (sum[0] * sum[0] + sum[1] * sum[1]).sqrt();
    let (bx, bz) = (sum[0] / bl, sum[1] / bl);
    let face_d = (min_d - 0.10).max(0.41);
    let cx = m.pos[0] + bx * face_d;
    let cz = m.pos[2] + bz * face_d;
    let bond_y = m.pos[1];
    let ray_y = head_y + (bond_y + 0.5 - head_y) * (face_d / 6.0);
    let top = max_y + 0.05;
    assert!(min_d > 0.52, "the posed muzzle clears the clamp start: {min_d}");
    assert!(top + 0.02 < ray_y - 0.02, "the wall top {top} reaches the sight line {ray_y} (max muzzle y {max_y}) — the wall cannot separate them; re-derive the scenario");
    eprintln!("[wall] bearing ({bx:.3},{bz:.3}) face_d {face_d} top {top} ray_y {ray_y} max_muzzle_y {max_y} samples {}", trace.len());
    // Wall corners: face centre (cx, cz) ± the tangent (−bz, bx) × 3, floor → top, both windings.
    let (tx, tz) = (-bz * 3.0, bx * 3.0);
    let (y0, y1) = (floor_y, top);
    let c = [[cx - tx, y0, cz - tz], [cx + tx, y0, cz + tz], [cx + tx, y1, cz + tz], [cx - tx, y1, cz - tz]];
    let wall = vec![c[0], c[1], c[2], c[0], c[2], c[3], c[0], c[2], c[1], c[0], c[3], c[2]];
    let w = CollisionWorld::from_triangles(floor_tris(floor_y).into_iter().chain(wall.chunks(3).map(|t| [t[0], t[1], t[2]])).collect());
    (w, floor_y, top, m.muzzle, seed, dx)
}

/// THE LATCH through the real sight tick: with the harness wall burying the measured aim muzzle,
/// `SeekOpponentLOS` 0x175cb8:68-82 -> `GunThroughWall` 0x178520 latches `+0x1e` while the guard is
/// in an aim state (`aims_torso`), through `Drones::tick` with Bond's position as the only input —
/// no state injection. Fails-first against a tree without the LOS hook (the latch can never turn).
/// The full case-3 hold window (no second burst while latched) is NOT pinned here: in the searched
/// scenarios (seeds 1-12 x dist 5-9) the stand guard sidesteps (0x73/0x74) and the crouch guard
/// drifts to the cover family after the first burst, whose fire arms carry the later bursts — a
/// real-path window where the case-3 request itself rides was not found this session (report).
#[test]
fn buried_muzzle_latches_through_the_sight_tick() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let (world, _floor_y, top, muzzle, seed, dx) = harness_wall(&l);
    eprintln!("[hold] harness wall top {top}, measured aim muzzle {muzzle:?} (seed {seed}, dx {dx})");
    let (drones, rng) = l.lone_guard(1.0, seed);
    let (mut sim, d) = l.sim(drones, rng, &world);
    let bond = bond_at([GUARD_X + dx, sim.list[0].pos[1], 99.47]);
    for _ in 0..WARM { sim.tick(&d, &bond_at(FAR)); }
    let mut latched_aiming = false;
    let mut first_shot = None;
    for _ in 0..RUN {
        sim.tick(&d, &bond);
        let g = &sim.list[0];
        if g.gun_through_wall && matches!(g.state, state::AIM_STAND_FIRE | state::AIM_CROUCH_FIRE) {
            latched_aiming = true;
        }
        if first_shot.is_none() && !g.shots.is_empty() { first_shot = Some((sim.frame - WARM, g.state)); }
    }
    let g = &sim.list[0];
    eprintln!("[hold] latched_aiming {latched_aiming}, shots {} (frames {:?}), gun_through_wall {}, state {:#x}",
        g.shots.len(), g.shots, g.gun_through_wall, g.state);
    assert!(latched_aiming, "the sight tick never latched +0x1e while the guard aimed (wall top {top}, muzzle {muzzle:?})");
    // Sister within the test: the game-true entry fire still rode (0x166328:59-62 / 0x167060:27-29
    // are unconditional) — the latch does not freeze the guard's first request.
    let (shot_tick, shot_state) = first_shot.expect("the entry fire request still rode");
    assert!(shot_tick < 120, "the first shot {shot_tick} left the entry window");
    assert!(matches!(shot_state, state::AIM_STAND_FIRE | state::AIM_CROUCH_FIRE), "the entry shot rode from {:#x}", shot_state);
}

/// SISTER: the same scenario without the wall — the probe clears, the latch stays open and the
/// guard keeps firing (the case-3 request rides `!gun_through_wall`): the sister fails if the
/// hold over-gates (e.g. the entry/timer arms 858 wanted to gate) or the probe never latches.
#[test]
fn open_muzzle_keeps_firing() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let (_seed, dx) = find_scenario(&l);
    let (drones, rng) = l.lone_guard(1.0, _seed);
    let world = CollisionWorld::from_triangles(floor_tris(drones[0].pos[1] - 1.0));
    let (mut sim, d) = l.sim(drones, rng, &world);
    let bond = bond_at([GUARD_X + dx, sim.list[0].pos[1], 99.47]);
    for _ in 0..WARM { sim.tick(&d, &bond_at(FAR)); }
    let mut ever_fired_after = false;
    for _ in 0..RUN {
        sim.tick(&d, &bond);
        if sim.list[0].shots.len() >= 3 { ever_fired_after = true; }
    }
    let g = &sim.list[0];
    eprintln!("[sister] shots {} (frames {:?}), clip {}/{} (max {}), gun_through_wall {}, state {:#x}",
        g.shots.len(), g.shots, g.clip, g.clip_max, g.clip_max, g.gun_through_wall, g.state);
    assert!(!g.gun_through_wall, "the open-air muzzle never latches");
    assert!(ever_fired_after, "the open guard keeps firing past the entry burst ({} shots)", g.shots.len());
}
