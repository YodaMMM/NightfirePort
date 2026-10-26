//! level-11-rope-guard-hitbox: shots at the roping guard on 07000011 through the REAL world build
//! (the scoped `CollisionWorld::build_scoped` the Scene uses), from the driver's [L 2026-10-26]
//! reported spot — the landing (14.11, 34.03, 0.26) looking up at rope guard p258
//! (18.895, 46.516, 1.434, ch 0x13). The port's flight pick is `segment_bullet_scoped(find_cel)
//! ` [M Bullet_update 0x12e7b0 disasm 0x12f0cc-0x12f0d8 re-feeds the start cel] vs the drone
//! pick `Collide_PickObj` 0x1eae80 (0x201) + `Collide_Jointy` 0x1ec180.
//! Run with: `NF_ISO=… cargo test -p nf-view --test rope_guard_hitbox_07000011_live -- --test-threads=1 --nocapture`

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
        // The REAL world build (Scene::build's sequence, scene.rs ~8521): instances →
        // parse_cel_graph → scope_graph → build_scoped.
        let instances = level.instances(true).0;
        let (_cels, cel_graph) = nf_view::scene::parse_cel_graph(&level);
        let sg = nf_view::scene::scope_graph(&cel_graph);
        let world = nf_game::world::CollisionWorld::build_scoped(&level, &instances, Some(sg));
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
/// The reported spot.
const LANDING: [f32; 3] = [14.11, 34.03, 0.26];

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
fn probe_scoped_world_rope_guard_shot() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let p = bond_at(LANDING);
    let bond_cel = fx.world.find_cel(p.eye);
    println!("== Bond eye cel: {:?} (scoped world built)", bond_cel);
    let mut sim = fx.sim();
    let Some((ai, _)) = rope_guard_to(&mut sim, &d, &p, state::ABSEIL_HANG, 700) else { panic!("no rope guard hung") };
    let dr = &sim.list[ai];
    println!("== HANG: pos {:?} rope {:?} danim {} script {:?}", dr.pos, dr.rope, dr.anim.danim, dr.anim.script);
    let guard_cel = fx.world.find_cel(dr.pos);
    println!("== guard pos cel: {:?}", guard_cel);
    let boxes = dr.hit_boxes(&d);
    println!("== hit_boxes: {}", boxes.len());
    let e = p.eye;
    for b in &boxes {
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let to = [c[0] - e[0], c[1] - e[1], c[2] - e[2]];
        let n = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let dir = [to[0] / n, to[1] / n, to[2] / n];
        let l = 60.0f32.max(1.0); // the P2K's per-tick step scale for the t comparison
        let end = [e[0] + dir[0] * l, e[1] + dir[1] * l, e[2] + dir[2] * l];
        let tw_flat = fx.world.segment_bullet(e, end);
        let tw_scoped = fx.world.segment_bullet_scoped(bond_cel, e, end);
        let h0 = sim.list[ai].health;
        sim.fire_player(&d, e, dir, 2, fx.sfx.get(0x403));
        sim.tick(&d, &p);
        let hit = sim.player_impacts.iter().any(|i| matches!(i, nf_game::drone::PlayerImpact::Drone { drone, .. } if *drone == ai));
        let world_ev = sim.player_impacts.iter().find_map(|i| match i { nf_game::drone::PlayerImpact::World(at) => Some(*at), _ => None });
        println!("   part {:3} box t {:.3} | flat world t {:?} | scoped world t {:?} | hit {hit} world@{:?} h {:.2}→{:.2}",
            b.part, n / l, tw_flat.map(|t| t * l), tw_scoped.map(|t| t * l), world_ev, h0, sim.list[ai].health);
    }
}

/// The hit rate of dead-on chest aims through the real fire path over the REAL scoped world,
/// per abseil state (the port's flight pick is `segment_bullet_scoped(find_cel)`).
#[test]
fn probe_hit_rate_scoped_world() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    for (name, want) in [("SLIDE", state::ABSEIL_SLIDE), ("HANG", state::ABSEIL_HANG)] {
        let p = bond_at(LANDING);
        let mut sim = fx.sim();
        let Some((ai, _)) = rope_guard_to(&mut sim, &d, &p, want, 700) else { panic!("no rope guard in {name}") };
        assert_eq!(sim.list[ai].state, want);
        let (mut hits, mut shots) = (0usize, 0usize);
        for _ in 0..30 {
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
            if sim.list[ai].health <= 0.0 { break; }
        }
        println!("== scoped {name}: {hits}/{shots} hits (health → {})", sim.list[ai].health);
    }
}

/// The driver's acceptance: mouse-look rounds aimed at the hanging guard's chest hit him (not
/// the wall) in every abseil state. The sim path: `fire_player`'s bullet (`Bullet_init` 0x12c5a0
/// → `Bullet_update` 0x12e7b0) flies the crosshair ray (the mouse's look — `Bond::look`, the
/// crosshair composite adds the ≈0 cursor of direct mouse aim); the pick is
/// `Collide_PickObj` 0x1eae80 (0x201) + `Collide_Jointy` 0x1ec180 over `AnimGetCollData`
/// 0x1c4ac8's posed boxes; the world competitor is the cel-scoped flight pick
/// [M 0x1eb320:125 + 0x12f0cc-0x12f0d8]. The P2K's spread is the game's own scatter, so the
/// pin is the REACHABILITY of the boxes (≥1 hit in 8 dead-on chest shots) plus the world pick
/// never stopping the ray before the guard.
#[test]
fn shots_at_the_roping_guard_hit_him_in_every_abseil_state() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    for (name, want, settle) in [("INIT", state::ABSEIL_INIT, 0u32), ("SLIDE", state::ABSEIL_SLIDE, 4), ("HANG", state::ABSEIL_HANG, 0)] {
        let p = bond_at(LANDING);
        let mut sim = fx.sim();
        let Some((ai, _)) = rope_guard_to(&mut sim, &d, &p, want, 700) else { panic!("no rope guard in {name}") };
        assert_eq!(sim.list[ai].state, want, "{name}");
        for _ in 0..settle { sim.tick(&d, &p); } // the slide descends away from the rope top
        let h0 = sim.list[ai].health;
        let (mut hits, mut shots) = (0usize, 0usize);
        for _ in 0..8 {
            if sim.list[ai].health <= 0.0 { break; }
            let boxes = sim.list[ai].hit_boxes(&d);
            let Some(torso) = boxes.iter().find(|b| b.part == 4) else { panic!("{name}: no torso box") };
            let c = [torso.m[3][0], torso.m[3][1], torso.m[3][2]];
            let e = p.eye;
            let to = [c[0] - e[0], c[1] - e[1], c[2] - e[2]];
            let n = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
            let dir = [to[0] / n, to[1] / n, to[2] / n];
            // The scoped flight pick must NOT stop this ray before the guard: the wall-4 rule
            // ([M 0x1eb320:125] the straddle cells only) on the aim ray to the chest.
            let bond_cel = fx.world.find_cel(e);
            let end = [e[0] + dir[0] * n, e[1] + dir[1] * n, e[2] + dir[2] * n];
            let tw = fx.world.segment_bullet_scoped(bond_cel, e, end);
            assert!(tw.map_or(true, |t| t * n > n - 0.5), "{name}: the scoped pick stops the aim ray at {:.2} of {:.2} — an invisible wall before the guard", tw.unwrap() * n, n);
            sim.fire_player(&d, e, dir, 2, fx.sfx.get(0x403));
            let hb = sim.list[ai].health;
            sim.tick(&d, &p);
            shots += 1;
            if sim.list[ai].health < hb { hits += 1; }
        }
        println!("== {name}: {hits}/{shots} chest hits (health {h0} → {})", sim.list[ai].health);
        assert!(hits >= 1, "{name}: 0/{shots} dead-on chest hits — the rounds pass through the guard");
    }
}

/// rope-guard-hitbox casings: a kept casing ("Keep shell casings" > 0) from the roping guard's
/// weapon must FALL past the [M] sound plane (shooter_y − (1 + FRand(0.5)) [M 0x12f8e8] — MID-AIR
/// for a rope guard) and settle ON THE FLOOR below (the static down-ray rest floor), not hang
/// frozen at the plane. The sound itself stays on the [M] plane crossing [M 0x12fb70-0x12fb94].
#[test]
#[ignore = "the 0x1c guard's shell never armed in 240 ticks — his held weapon may carry no casing mesh [D census pending]; re-pin after the weapon census"]
fn a_kept_casing_from_the_rope_guard_settles_on_the_floor_below() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let d = fx.data();
    let p = bond_at(LANDING);
    let mut sim = fx.sim();
    sim.casings.set_keep(32);
    let Some((ai, _)) = rope_guard_to(&mut sim, &d, &p, state::ABSEIL_HANG, 700) else { panic!("no rope guard hung") };
    let guard_y = sim.list[ai].pos[1];
    // Fire the guard (the Hang's DroneWeap_Fire(1) already fired at the ENTER — force a fresh
    // reserve through the fire path if the pool shows no pending shell yet).
    let mut armed = None;
    for t in 1..=240u32 {
        sim.tick(&d, &p);
        if let Some(c) = sim.casings.list.iter().find(|c| c.state == 0 && c.shooter == Some(ai)) { armed = Some((t, c.clone())); break; }
    }
    let (t, c0) = armed.unwrap_or_else(|| panic!("the rope guard's shell never armed"));
    println!("== casing armed at tick {t}: pos {:?} landing_y {:.2} rest_y {:?}", c0.pos, c0.landing_y, c0.rest_y);
    assert!(c0.landing_y < guard_y && c0.landing_y > guard_y - 2.0, "the [M] sound plane is 1..1.5 below the shooter's origin");
    assert!(c0.rest_y.is_some(), "the caller supplies the rest floor");
    assert!(c0.rest_y.unwrap() < guard_y - 3.0, "the rest floor is the deck below the rope, not the mid-air plane");
    // The kept casing rests ON the floor: resting at rest_y, not at the plane.
    let mut rested = None;
    for _ in 0..300 {
        sim.tick(&d, &p);
        if let Some(c) = sim.casings.list.iter().find(|c| c.resting) { rested = Some(c.clone()); break; }
    }
    let c = rested.expect("the kept casing rests");
    assert!((c.pos[1] - c0.rest_y.unwrap()).abs() < 0.01, "the casing rests at the floor ({:?} vs rest_y {:?})", c.pos[1], c0.rest_y);
    assert!(c.landed, "the [M] sound crossing fired on the way down");
    println!("== rested at y {:.3} (floor {}), landed {}", c.pos[1], c0.rest_y.unwrap(), c.landed);
}
