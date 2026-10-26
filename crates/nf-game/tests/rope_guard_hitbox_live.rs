//! level-11-rope-guard-hitbox live tests: rounds aimed at a roping guard on 07000011 must hit
//! his boxes (not the wall behind) in every abseil state, and ejected casings must fall.
//! Run with: `NF_ISO=… cargo test -p nf-game --test rope_guard_hitbox_live -- --test-threads=1`
//!
//! The game's pick [M]: `Collide_PickObj` 0x1eae80's 0x201 arm — the closest point on the flight
//! segment to the object's world-sphere centre (obj+0x80, radius obj+0x8c,
//! `Control_BuildWorldSph` 0x133070) contacts when `sqdist < r²`; then `Collide_Jointy`
//! 0x1ec180 walks `AnimGetCollData` 0x1c4ac8's boxes (`HumanCollisionBones` 0x2c6be8 over the
//! POSED palette — the same palette the render draws) with `Intersect_RayBox`, first box in
//! list order wins, HITDATA+0x52 = the part, +0xc = t·raylen.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{state, Data, Drones, Player, Rng};
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
    sfx: nf_formats::sfx::SfxTable,
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
        let sfx = nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap();
        Some(Fixture { tables, props, weapons, level, level_id, scripts, world, sequences, skeletons, nav, covers, tuning, hitboxes, volumes, models, skins, sfx })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world,
            level: self.level_id, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav,
            covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: Some(&self.sfx), models: &self.models,
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
/// The reported spot: the stairwell landing the shots come from.
const LANDING: [f32; 3] = [14.11, 34.03, 0.26];

/// The rope guard p258 (ch 0x13 [D param 3]): release the switch, run the chain to the wanted
/// state, return (index, ticks).
fn rope_guard_to(sim: &mut Drones, d: &Data, p: &Player, want: u8, max: u32) -> Option<(usize, u32)> {
    for _ in 0..3 { sim.tick(d, p); }
    let ai = sim.list.iter().position(|x| x.present && x.mode == 0x1c)?;
    sim.set_channel(0x13);
    for t in 1..=max {
        sim.tick(d, p);
        if sim.list[ai].state == want { return Some((ai, t)); }
    }
    None
}

#[test]
fn probe_rope_guard_boxes_and_shot() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let p = bond_at(LANDING);
    let mut sim = fx.sim();
    let Some((ai, _)) = rope_guard_to(&mut sim, &d, &p, state::ABSEIL_HANG, 700) else { panic!("no rope guard hung") };
    let dr = &sim.list[ai];
    println!("== HANG state: pos {:?} rope {:?} yaw {} anim.script {:?} danim {}",
        dr.pos, dr.rope, dr.yaw, dr.anim.script, dr.anim.danim);
    let pal = dr.palette(&d);
    println!("== palette: {} ({} bones)", if pal.is_some() { "Some" } else { "None" }, pal.as_ref().map_or(0, |p| p.len()));
    let boxes = dr.hit_boxes(&d);
    println!("== hit_boxes: {}", boxes.len());
    for b in &boxes {
        println!("   part {:3} centre ({:.3}, {:.3}, {:.3}) half {:?}", b.part, b.m[3][0], b.m[3][1], b.m[3][2], b.half);
    }
    // The world pick along the aim ray to each box centre from Bond's eye.
    let e = p.eye;
    let mut any_drone = false;
    for b in &boxes {
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let to = [c[0] - e[0], c[1] - e[1], c[2] - e[2]];
        let n = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let dir = [to[0] / n, to[1] / n, to[2] / n];
        let h0 = sim.list[ai].health;
        sim.fire_player(&d, e, dir, 2, fx.sfx.get(0x403));
        for _ in 0..3 { sim.tick(&d, &p); }
        let impacts: Vec<String> = sim.player_impacts.iter().map(|i| format!("{i:?}")).collect();
        let hit = sim.player_impacts.iter().any(|i| matches!(i, nf_game::drone::PlayerImpact::Drone { drone, .. } if *drone == ai));
        any_drone |= hit;
        println!("   shot at part {:3}: impacts [{}] health {:.2} → {:.2}", b.part, impacts.join(", "), h0, sim.list[ai].health);
    }
    println!("== any drone hit: {any_drone}");
}

/// The hit rate of dead-on chest aims through the real fire path, per abseil state. The P2K's
/// spread rides `fire_player` (the game's own scatter); the chest aim is the drawn torso box's
/// centre recomputed EVERY shot (the crosshair tracking a descending guard).
#[test]
fn probe_hit_rate_per_abseil_state() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    for (name, want) in [("SLIDE", state::ABSEIL_SLIDE), ("HANG", state::ABSEIL_HANG)] {
        let p = bond_at(LANDING);
        let mut sim = fx.sim();
        let Some((ai, _)) = rope_guard_to(&mut sim, &d, &p, want, 700) else { panic!("no rope guard in {name}") };
        assert_eq!(sim.list[ai].state, want);
        let (mut hits, mut health0, mut shots) = (0usize, sim.list[ai].health, 0usize);
        for _ in 0..30 {
            // The aim: the CURRENT torso box centre (part 4), from the eye.
            let boxes = sim.list[ai].hit_boxes(&d);
            let Some(torso) = boxes.iter().find(|b| b.part == 4) else { break };
            let c = [torso.m[3][0], torso.m[3][1], torso.m[3][2]];
            let e = p.eye;
            let to = [c[0] - e[0], c[1] - e[1], c[2] - e[2]];
            let n = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
            sim.fire_player(&d, e, [to[0] / n, to[1] / n, to[2] / n], 2, fx.sfx.get(0x403));
            let h0 = sim.list[ai].health;
            sim.tick(&d, &p);
            shots += 1;
            if sim.list[ai].health < h0 { hits += 1; }
            if sim.list[ai].health <= 0.0 { break; } // dead guards stop the rate
        }
        println!("== {name}: {hits}/{shots} hits (health {health0} → {})", sim.list[ai].health);
    }
}

#[test]
fn probe_slide_palette() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let p = bond_at(LANDING);
    let mut sim = fx.sim();
    let Some((ai, _)) = rope_guard_to(&mut sim, &d, &p, state::ABSEIL_SLIDE, 700) else { panic!("no slide") };
    for k in 0..5 {
        sim.tick(&d, &p);
        let dr = &sim.list[ai];
        let pal = dr.palette(&d);
        let boxes = dr.hit_boxes(&d);
        println!("== slide tick {k}: state {:x} pos {:?} danim {} script {:?} palette {} boxes {}",
            dr.state, dr.pos, dr.anim.danim, dr.anim.script, pal.as_ref().map_or(0, |p| p.len()), boxes.len());
    }
}

/// The anim chain through every abseil state: which danim/script/anim-state/call the port runs
/// while the DSTATE walks Init → Slide → Hang. The level-11-rope-guard-hitbox probe saw danim 11
/// (the Init rope-grab, script 0x6000892) persisting into Hang — this trace shows where the
/// port's `call_anim` stops producing danims.
#[test]
fn probe_abseil_anim_chain() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let p = bond_at(LANDING);
    let mut sim = fx.sim();
    for _ in 0..3 { sim.tick(&d, &p); }
    let Some(ai) = sim.list.iter().position(|x| x.present && x.mode == 0x1c) else { panic!("no rope guard") };
    sim.set_channel(0x13);
    let mut last: Option<(u8, u16, u16, Option<u32>, Option<(u16, u8, u8)>, bool)> = None;
    for t in 1..=900 {
        sim.tick(&d, &p);
        let dr = &sim.list[ai];
        let key = (dr.state, dr.anim.danim, dr.anim.state, dr.anim.script, dr.anim.call, dr.anim.reached);
        if last != Some(key) {
            println!("t {:4} DSTATE {:#04x} danim {:4} anim.state {:#04x} class {} script {:?} call {:?} reached {}",
                t, dr.state, dr.anim.danim, dr.anim.state, dr.anim.class, dr.anim.script, dr.anim.call, dr.anim.reached);
            last = Some(key);
        }
        if dr.state == state::ABSEIL_HANG && dr.anim.danim != 0xb { println!("== hang pose started at t {t}"); break; }
    }
    let dr = &sim.list[ai];
    println!("== final: DSTATE {:#04x} danim {} anim.state {:#04x} script {:?} call {:?} reached {}",
        dr.state, dr.anim.danim, dr.anim.state, dr.anim.script, dr.anim.call, dr.anim.reached);
    println!("== anim_set {} (saved {}) mode {:#x} attack_mode {:#x}", dr.anim_set, dr.saved_anim_set, dr.mode, dr.attack_mode);
}
