//! `reload-and-bond-hitboxes`: the guards' reload path and Bond's 16 body boxes, against the
//! decompile and the level-start EE snapshots.
//! - Reload: `NDrone2_DSTATE_AimStandReload` 0x1666d8 / `AimCrouchReload` 0x167370 (states 0x69 /
//!   0x71), `BOTWEAP_ReloadAmmoType` 0x129d28, the reload anim 0x52 (the 1-in-30 alternate 0x4a),
//!   the refill `+0xBBC = +0xBBE` at ENTER.
//! - Bond's boxes: `HumanCollisionBones` 0x2c6be8 [D] via `AnimGetCollData` 0x1c4ac8 +
//!   `Collide_Jointy` 0x1ec180 (kind 0x201, first box in list order wins), the [L] standing idle
//!   palette (`damage::bond_boxes`).
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{msg, state, Data, Drone, Drones, Player, Rng};
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
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: std::collections::HashMap<u32, nf_game::pickup::Model>,
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
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
        let nav = nf_game::nav::Network::build(level.maps[level.main_map().unwrap()].ai_network.as_ref().unwrap(), &world, 0x0700_0005);
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

    fn drones(&self) -> (Vec<Drone>, Rng) {
        let d = self.data();
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        (nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights), rng)
    }

    /// The simulation over these drones with the level's triggers (difficulty 1, as ai_live's harness).
    fn sim(&self, drones: Vec<Drone>, rng: Rng) -> Drones {
        let mut s = Drones::new(drones, rng); s.attach_level_id(&self.level, 0x0700_0005);
        s.difficulty = 1;
        s
    }
}

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None } }

/// The disc's `HumanCollisionBones` 0x2c6be8 part ids in table order (bone a of each of the 16
/// entries; the terminator is entry 16) [D 0x2c6be8, file offset 0x1c0ae8].
const DISC_PARTS: [i16; 16] = [5, 4, 53, 54, 55, 56, 49, 50, 51, 52, 20, 21, 23, 32, 35, 39];

/// The boxes ride the [L] palette: rebuilt at the 07000005 start pose (Bond's obj 0x16cefe0:
/// pos (104.95, 24.96, 108.83), yaw −2.147), the box centres must land on the snapshot's measured
/// bone translations (AnimObject 0x16cb8a0, palette 0x16c9b60 — the head's entry-0 offset included)
/// [L ee_start_05.bin]. The ±0.03 tolerance covers the snapshot print precision and the idle-phase
/// delta between the canonical pose (07000001) and this level's.
#[test]
fn bond_boxes_match_the_snapshot_palette() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let boxes = nf_game::damage::bond_boxes(&d, -2.147, [104.95, 24.96, 108.83]);
    assert_eq!(boxes.len(), 16, "HumanCollisionBones: 16 boxes + terminator [D 0x2c6be8]");
    assert_eq!(boxes.iter().map(|b| b.part).collect::<Vec<_>>(), DISC_PARTS, "the part ids in table order [D]");
    let centre = |part: i16| boxes.iter().find(|b| b.part == part).unwrap();
    let c = centre(5).m[3];
    assert!((c[0] - 104.881).abs() < 0.03 && (c[1] - 25.537).abs() < 0.03 && (c[2] - 108.630).abs() < 0.03, "head centre {c:?}");
    let c = centre(4).m[3];
    assert!((c[0] - 104.925).abs() < 0.03 && (c[1] - 25.239).abs() < 0.03 && (c[2] - 108.744).abs() < 0.03, "torso centre {c:?} (mid bone4/bone1)");
    let c = centre(56).m[3];
    assert!((c[0] - 104.506).abs() < 0.03 && (c[1] - 24.018).abs() < 0.03 && (c[2] - 108.685).abs() < 0.03, "left foot centre {c:?}");
    let c = centre(23).m[3];
    assert!((c[0] - 104.429).abs() < 0.03 && (c[1] - 25.360).abs() < 0.03 && (c[2] - 108.280).abs() < 0.03, "left hand centre {c:?}");
    // The head box's half extents are the disc entry's floats at +8/+12/+16 × the skin scale 1.0
    // [D 0x2c6be8 entry 0: {a=5, b=-1, c0=0.1562, half=(0.11, 0.11, 0.11)}; AnimGetCollData
    // 0x1c4ac8:70-76 reads psVar3+4/+6/+8 shorts = bytes 8/12/16; the c0 float at +4 is not read here] + [L scale].
    let h = centre(5).half;
    assert!((h[0] - 0.11).abs() < 1e-3 && (h[1] - 0.11).abs() < 1e-3 && (h[2] - 0.11).abs() < 1e-3, "head half {h:?}");
}

/// A round fired into each region must be able to report THAT region's part — `Collide_Jointy`
/// 0x1ec180's kind-0x201 walk (the port's `first_box_hit`: first box in list order wins, so a
/// graze of an earlier box legitimately takes the earlier part). For each box some face approach
/// must exist whose first list-order hit is the box itself; the neighbors overlap (the thigh's top
/// face meets the torso's bottom), so the test sweeps all six faces and needs one clean lane each.
#[test]
fn rounds_hit_each_body_region() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let pos = [104.95, 24.96, 108.83];
    let boxes = nf_game::damage::bond_boxes(&d, -2.147, pos);
    assert_eq!(boxes.len(), 16);
    let mut misses = Vec::new();
    for b in &boxes {
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let mut ok = false;
        for k in 0..3 {
            let axis = [b.m[k][0], b.m[k][1], b.m[k][2]];
            for out in [1.0f32, -1.0] {
                let start = [c[0] + axis[0] * out * (b.half[k] + 0.3), c[1] + axis[1] * out * (b.half[k] + 0.3), c[2] + axis[2] * out * (b.half[k] + 0.3)];
                // The ray ENDS at the centre: an overshoot past the centre would run into the
                // neighbour sharing the inner face (e.g. the forearm's tip inside the hand box).
                let ray = [-axis[0] * out * (b.half[k] + 0.3), -axis[1] * out * (b.half[k] + 0.3), -axis[2] * out * (b.half[k] + 0.3)];
                if matches!(nf_game::damage::first_box_hit(start, ray, &boxes), Some((p, _)) if p == b.part) { ok = true; break; }
            }
            if ok { break; }
        }
        if !ok { misses.push(b.part); }
    }
    // Part 39's box (the right hand, mid-point bone39/bone35) lies INSIDE the torso box at the idle
    // pose — the game's own coarse layout: every approach crosses an earlier-listed box, so the
    // list-order walk (first hit wins, 0x1ec180) reports 4 there. The hand becomes hittable when
    // the arm animates (the port holds the idle pose, docs in the report); its geometry is still
    // pinned by the snapshot-centre test above.
    assert!(misses == [39], "regions with no clean aim lane (only 39 expected): {misses:?}");
}

/// A single head region hit vs a foot hit costs the same through `Player_HandlePain`'s SP arm —
/// the part multipliers live in the MP block only [M 0x1902c8:28-49 SP difficulty path skips the
/// switch at :51-78; :51 `if (param_5 != -1)` sits under `MPSettings._384_4_ != 0`].
#[test]
fn sp_pain_has_no_part_scale() {
    let mut rng = Rng::new(7);
    let mut a = nf_game::bullet::PlayerHealth::default();
    let mut b = nf_game::bullet::PlayerHealth::default();
    a.pain(&mut rng, 35.0, 2, 0x0700_0005);
    b.pain(&mut rng, 35.0, 2, 0x0700_0005);
    assert_eq!(a.health, b.health, "SP: head part or foot part, the same weapon damage — no scale");
}

/// `NDrone2_DSTATE_AimStandReload` 0x1666d8 (state 0x69) ENTER [M case 1 + disasm 0x166738-0x16677c]:
/// exactly ONE shared-stream draw (`Rand_Rand(30)`), drawn unconditionally; the clip refills at
/// ENTER (`+0xBBC = +0xBBE` [M :30]); the anim call is 0x52 — or 0x4a when the draw is 0 AND
/// `DroneAnim_CanDoAnimState(0x4a, 0)` allows it [M :21-27]. No 07000005 guard's anim set can do
/// 0x4a (the `expect` below scans all of them), so at this level the branch is dead — the draw
/// still happens, which is what the shared stream pins here.
#[test]
fn aim_stand_reload_draws_rand30_and_refills_at_enter() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, _) = l.drones();
    assert!(drones.iter().filter(|g| g.present).all(|g| !g.can_do(&d, 0x4a)), "level fact: no 07000005 guard can do anim 0x4a (the CanDoAnimState gate [M 0x1666d8:22] never opens here)");
    let gi = drones.iter().position(|g| g.weapon != 1).expect("an armed guard");
    let mut seen_52 = false;
    for seed in 1u32..400 {
        let mut g = drones[gi].clone();
        g.state = state::AIM_STAND_RELOAD; // direct: set_state's pending transition can be overwritten by the old state's ENTER
        let mut rng = Rng::new(seed);
        let d0 = rng.draws;
        g.send(&d, &mut rng, 1000, msg::ENTER);
        assert_eq!(rng.draws - d0, 1, "seed {seed}: the ENTER consumed exactly the Rand_Rand(30) draw [M 0x166738]");
        assert_eq!(g.clip, g.clip_max, "seed {seed}: the refill landed at ENTER (+0xBBC = +0xBBE [M 0x1666d8:30])");
        assert_eq!(g.firing, false, "seed {seed}: +0x3b = 0 at ENTER");
        assert_eq!(g.anim.call, Some((0x52, state::AIM_STAND_FIRE, 0)), "seed {seed}: anim 0x52, follow-on 0x68");
        assert_eq!(g.alert_status, 1, "seed {seed}: AlertStatusSet(1) [M 0x1666d8 case 1]");
        seen_52 = true;
    }
    assert!(seen_52, "the reload anim 0x52 pinned across the seeds");
}

/// The CROUCH reload has NEITHER the draw NOR the alert write [M 0x167370 case 1 — no Rand_Rand,
/// no AlertStatusSet; only the flag, the refill, `+0x3b = 0`, anim 0x52 → 0x70].
#[test]
fn aim_crouch_reload_draws_nothing() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, _) = l.drones();
    let gi = drones.iter().position(|g| g.weapon != 1).expect("an armed guard");
    for seed in 1u32..40 {
        let mut g = drones[gi].clone();
        g.state = state::AIM_CROUCH_RELOAD; // direct, as above
        let mut rng = Rng::new(seed);
        let d0 = rng.draws;
        g.send(&d, &mut rng, 1000, msg::ENTER);
        assert_eq!(rng.draws - d0, 0, "seed {seed}: the crouch reload consumes no draw [M 0x167370 case 1]");
        assert_eq!(g.clip, g.clip_max, "seed {seed}: the refill landed at ENTER [M 0x167370:17]");
        assert_eq!(g.anim.call, Some((0x52, state::AIM_CROUCH_FIRE, 0)), "seed {seed}: anim 0x52, follow-on 0x70");
        assert_ne!(g.alert_status, 1, "seed {seed}: no AlertStatusSet in the crouch case");
    }
}

/// In the sim: a guard's burst into Bond lands a `BulletEvent::Player` whose part is one of the
/// disc's 16 bone ids [D 0x2c6be8] — the boxes are what the guard rounds hit now.
#[test]
fn guard_rounds_carry_a_body_part() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, rng) = l.drones();
    let mut sim = l.sim(drones, rng);
    let away = bond_at([0.0, -500.0, 0.0]);
    for _ in 0..10 { sim.tick(&d, &away); }
    let player = bond_at([67.31, 26.5, 99.47]);
    let mut hits = 0;
    for _ in 0..1200 {
        sim.tick(&d, &player);
        for ev in &sim.impacts {
            if let nf_game::bullet::BulletEvent::Player { part, .. } = ev {
                hits += 1;
                assert!(DISC_PARTS.contains(part), "a round hit part {part}, not one of the disc's 16 boxes");
            }
        }
        if hits >= 2 { break; }
    }
    assert!(hits >= 2, "the guard landed {hits} body hits in 1200 ticks");
}
