//! fx-npc-footsteps — the Script_Interp case-2 footstep on real level data: the NIS anim stream's
//! `AnimObjectUpdate` raises `hitdata+0x60 & 4` from the walk script's op-4 footstep markers
//! (`AnimProcessScriptCmds` 0x1c1b38:372-401) and the stream's own trailing block consumes it —
//! the 4-unit downward ray from obj+0xc0 [M 0x1d9a08 0x1d9f7c-0x1d9fd0] and
//! `Effect_Create(rayhit, obj, foot|0x42, 1.0)` [M 0x1da01c] with NO ScriptCam gate [M
//! 0x1d9f30-0x1da028 reads only ScriptFootstepFoot + glb_world]. The scene is 07000005's ledge
//! cut-scene (ledge_live.rs's ScriptPlayer 298, NIS 0x06000994, camera length 1400): stream 14 is
//! Bond's `Anim { script: 0x06000995, skin: 0x05000007 }` whose markers are ev 4 at frame 8 and
//! ev 5 at frame 56 [D the parsed script — the probe census]. Both guard streams (0x06000996 on
//! skin 0x05000030, 0x06000997 on 0x0500002e, markers at 577+ [D]) walk later in the cut-scene.
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drones, Player, Rng};
use nf_game::sound::Cmd;
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
    sfx: Option<nf_formats::sfx::SfxTable>,
    surfaces: nf_formats::sfx::SurfaceSounds,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: HashMap<u32, nf_game::pickup::Model>,
    skins: HashMap<u32, nf_formats::character::Skin>,
}

impl Level05 {
    fn load() -> Option<Level05> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let read = |name: &str| -> Option<Vec<u8>> {
            let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
            Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
        };
        let data = read("07000005.bin")?;
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let skeletons: HashMap<u8, _> = lb.entries.iter().filter(|e| e.kind == 6)
            .filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok().map(|s| (s.id as u8, s))).collect();
        let mut sequences = HashMap::new();
        if let Some(anim) = read("07F00005.bin") {
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
        let nav = nf_game::nav::Network::build(level.maps[level.main_map().unwrap()].ai_network.as_ref().unwrap(), &world, 0x0700_0005);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let hitboxes = nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap();
        let volumes = nf_game::aivolume::build(&level, 0x0700_0005);
        let models = nf_game::pickup::models(&level);
        let sfx = nf_formats::sfx::SfxTable::from_action_elf(&elf).ok();
        let surfaces = nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).unwrap();
        let tuning = nf_formats::tuning::Tuning::default();
        let skins = level.skins.clone();
        Some(Level05 { tables, props, weapons, level, scripts, world, sequences, skeletons, nav, covers, sfx, surfaces, tuning, hitboxes, volumes, models, skins })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_0005, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: self.sfx.as_ref(), models: &self.models, surfaces: Some(&self.surfaces), voices: None, difficulty: 1,
            ai_points: &[],
        }
    }

    fn sim(&self) -> Drones {
        let d = self.data();
        let mut rng = Rng::new(0x1234_5678);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        let mut s = Drones::new(drones, rng);
        s.attach_level_id(&self.level, 0x0700_0005);
        s
    }
}

fn bond_standing(p: [f32; 3]) -> Player { Player { pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, ..Default::default() } }

/// The ledge cut-scene's footsteps: Bond's stream-14 walk script 0x06000995 carries ev 4 at frame
/// 8, ev 5 at frame 56 and ev 4 at frame 90 [D the parsed script], so within the first ~90 ticks
/// of the cut-scene the case-2 consumption fires three times on the real ground under the stream
/// object: the surface's footstep sound starts (`Effect_Player` 0x1cec48:59-70 at the ray hit),
/// and ONE footprint decal per step lands — foot 4's mesh (`EffectInfo[mat]+8`) on the ev-4
/// steps, foot 8's (`+0xc`) on the ev-5 step [M 0x1cec48:84-89 + the global alternator the
/// markers set, 0x1c1b38:392-395], y = the ray hit + 0.05 [M :118-124], no ScriptCam gate (the
/// cut-scene RUNS a script camera and still steps — the inner AnimObjectUpdate consumption is
/// the suppressed one [M 0x1befd8:191-195, 0x1d9f30-0x1da028 reads no ScriptCam]).
#[test]
fn the_ledge_cutscene_leaves_footsteps() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    // The disc values this test pins [D]: stream 14's anim script and its two first markers.
    let scr = d.scripts.get(&0x0600_0995).expect("06000995 in 07000005");
    let marker = |ev: u16| scr.commands.iter().find_map(|c| match (c.op, c.args.as_slice()) {
        (4, [f, e, ..]) if *e == ev => Some(*f as u32), _ => None,
    }).unwrap_or_else(|| panic!("06000995 carries no ev-{ev} marker"));
    let (f4, f5) = (marker(4), marker(5));
    assert_eq!((f4, f5), (8, 56), "06000995's first markers");

    let mut sim = l.sim();
    // The plate's lock channel on, then the plate's output 0x43 (Bond stepping on plate 301) —
    // ledge_live.rs's chain for SP 298's NIS 0x06000994.
    sim.set_channel(0xb3);
    sim.set_channel(0x43);

    // The sound driver's starts, at Bond's stream object (npc_footsteps_live.rs's drain; a far
    // listener culls non-looped 3-D starts past 1.1×outer, so the listener RIDES the object).
    let mut starts: Vec<(u32, u32)> = Vec::new();
    let mut steps: Vec<(u32, [f32; 3], f32)> = Vec::new(); // (mesh, pos, obj y at the step)
    let mut step_sound = 0u32;
    let mut obj: Option<(usize, [f32; 3])> = None;
    let mut seen = 0usize;
    for t in 0..90 {
        sim.tick(&d, &bond_standing([104.0, 26.0, 108.0]));
        // The stream object (Bond's 0x06000995) and its live position.
        for (oi, o) in sim.scripts.objects.list.iter().enumerate() {
            if let Some(nf_game::nis::Obj::Anim { skin: 0x0500_0007, script: 0x0600_0995, matrix, .. }) = o {
                obj = Some((oi, [matrix[3][0], matrix[3][1], matrix[3][2]]));
            }
        }
        let Some((_, p)) = obj else { continue };
        // The material under the stream object this tick (the drain's own ray shape, 4 units [M 0x1d9f90]).
        let Some((hit_t, _, m, _, _)) = d.world.segment_surface_host([p[0], p[1], p[2]], [p[0], p[1] - 4.0, p[2]], true) else { continue };
        if m & 0x3f == 0 { continue; }
        let mat = (m & 0x3f) as usize;
        let rec = l.surfaces.effects.get(mat).expect("the row carries effects");
        if rec.foot_decal_4 == 0 && rec.foot_decal_8 == 0 { continue; }
        step_sound = nf_formats::sfx::SurfaceSounds::get(&l.surfaces.footstep, m).expect("the row carries a footstep sound");
        let listener = p;
        if let Some(tbl) = &l.sfx {
            sim.audio.update(Some(tbl), (listener, [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]), &|_| false);
            for c in sim.audio.take_commands() {
                if let Cmd::Start { id, pos: Some(_), .. } = c { starts.push((id, t)); }
            }
        }
        // The decals this tick: each new print is a step (mesh, y, the object's ray-hit y).
        while seen < sim.effects.decals.len() {
            let print = &sim.effects.decals[seen];
            steps.push((print.mesh, print.pos, p[1] - 4.0 * hit_t));
            seen += 1;
        }
    }
    assert!(obj.is_some(), "Bond's stream object was never created");
    // The markers within the window: ev 4 @8, ev 5 @56, ev 4 @90 [D] — three steps, foot 4/8/4.
    assert_eq!(steps.len(), 3, "three footstep decals from 06000995's markers at 8/56/90 (got {steps:?})");
    // The foot meshes: ev 4 → ScriptFootstepFoot false → foot 4's +8; ev 5 → true → foot 8's +0xc.
    let (_, p) = obj.unwrap();
    let (_, _, m, _, _) = d.world.segment_surface_host([p[0], p[1], p[2]], [p[0], p[1] - 4.0, p[2]], true).unwrap();
    let m = (m & 0x3f) as usize;
    let rec = l.surfaces.effects.get(m).unwrap();
    assert_eq!(steps[0].0, rec.foot_decal_4, "step 1 = EffectInfo[{m}]+8 (foot 4)");
    assert_eq!(steps[1].0, rec.foot_decal_8, "step 2 = EffectInfo[{m}]+0xc (foot 8)");
    assert_eq!(steps[2].0, rec.foot_decal_4, "step 3 = EffectInfo[{m}]+8 (foot 4, the ev-4 @90)");
    // y = the ray hit + 0.05 [M 0x1cec48:118-124]; the two steps sit at the walk's ground.
    for (i, (_, pos, gy)) in steps.iter().enumerate() {
        assert!((pos[1] - (gy + 0.05)).abs() < 0.01, "step {i} y {} = ray hit {gy} + 0.05", pos[1]);
    }
    // The surface sound started at the steps (the cut-scene RUNS a script camera — the case-2
    // block still consumes [M no ScriptCam read in 0x1d9f30-0x1da028]).
    assert!(starts.iter().any(|&(id, _)| id == step_sound), "the surface footstep sound started; starts {starts:?}");
}
