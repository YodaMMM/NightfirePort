//! fx-npc-footsteps — the NPC footstep's `Effect_Create(1.0, obj+0xdc, obj, foot|0x42)` on real
//! level data: `AnimObjectUpdate` 0x1befd8:185-208's consumption of the anim-script footstep
//! markers (`AnimProcessScriptCmds` 0x1c1b38:372-401, op 4 events 0 / 4 / 5) — the surface
//! footstep sound (`Effect_Player` 0x1cec48:59-70) AND the footprint decal (:84-136, the audit's
//! missing arm), the ONE global `ScriptFootstepFoot` 0x30cde0, and the `ScriptCam != 0`
//! suppression (0x1befd8:191-195). The walk script is 07000001's real 060000ca (ev 4 at frame 30,
//! ev 5 at frame 64 [D]); the guard stands on material 0x4 whose EffectInfo row carries the disc's
//! foot meshes 0x02000627 (foot 4, +8) / 0x02000628 (foot 8, +0xc) [L the parsed SurfaceSounds].
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{state, Data, Drone, Drones, Player, Rng};
use nf_game::sound::Cmd;
use std::collections::HashMap;

/// Everything `Data` borrows, owned (the ai_live.rs fixture, generalized to the level name).
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
    models: std::collections::HashMap<u32, nf_game::pickup::Model>,
    hud: nf_formats::hud::HudTables,
    surfaces: nf_formats::sfx::SurfaceSounds,
    voices: Vec<nf_formats::sfx::VoiceLine>,
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
}

impl Fixture {
    fn load(name: &str, level_id: u32) -> Option<Fixture> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap();
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok()).map(|k| (k.id as u8, k)).collect();
        let m = level.main_map().unwrap();
        let nav = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref().unwrap(), &world, level_id);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let volumes = nf_game::aivolume::build(&level, level_id);
        Some(Fixture {
            tables: drone::DroneAnimData::from_action_elf(&elf).unwrap(),
            props: drone::PropertyTables::from_action_elf(&elf).unwrap(),
            weapons: WeaponData::from_action_elf(&elf).unwrap(),
            sfx: nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap(),
            tuning: { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap(); nf_game::tuning::read_tuning_vars(&mut t, level_id); t },
            hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap(),
            sequences: disc::level_sequences(&iso, name).unwrap(),
            models: nf_game::pickup::models(&level),
            hud: nf_formats::hud::HudTables::from_action_elf(&elf).unwrap(),
            surfaces: nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).unwrap(),
            voices: nf_formats::sfx::voice_lines(&elf).unwrap(),
            level, level_id, scripts, world, skeletons, nav, covers, volumes,
        })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: self.level_id,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1,
                ai_points: &[],
            }
    }

    fn sim(&self, drones: Vec<Drone>) -> Drones {
        let mut s = Drones::new(drones, Rng::new(1)); s.attach_level_id(&self.level, self.level_id);
        s.difficulty = 1;
        s
    }

    fn level_drones(&self) -> Vec<Drone> {
        let d = self.data();
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights)
    }
}

fn bond_away() -> Player { Player { yaw: 0.0, pos: [0.0, -500.0, 0.0], target: [0.0, -499.5, 0.0], eye: [0.0, -499.0, 0.0], aim: [-1.0, 0.0, 0.0], sph: [0.0, -500.0, 0.0], capsule: ([0.0, -498.9, 0.0], [0.0, -500.0, 0.0]), hanging: false, body: Default::default(), cel: None, obj_cel: None, move_local: [0.0; 2], riding: false, crouched: false } }

/// Points the drone's playing script at `hash` so the next tick's command window covers `frame`
/// (the anim_events_live.rs harness).
fn play_frame(dr: &mut Drone, hash: u32, frame: u32) {
    dr.anim.script = Some(hash);
    dr.anim.danim = 0;
    if frame <= 1 {
        dr.anim.time = 0;
        dr.anim.ftime = 0.0;
        dr.anim.started = false;
    } else {
        dr.anim.time = frame - 2;
        dr.anim.ftime = (frame - 1) as f32;
        dr.anim.started = true;
    }
    dr.anim.speed = 1.0;
    dr.anim.held = false;
    dr.anim.once = false;
    dr.anim.len_cap = None;
    dr.anim.call = None;
    dr.anim.blend_wait = 0;
    dr.drawn = true;
    dr.present = true;
    dr.enabled = true;
}

/// The floor under `pos` (the placement's own ground test, independent of the drain's ray).
fn floor_under(l: &Fixture, pos: [f32; 3]) -> f32 { l.world.floor_below(pos, 3.0).unwrap_or(pos[1] - 1.0) }

/// A live guard standing on a material whose EffectInfo row carries the disc's foot meshes.
fn pick_guard(sim: &Drones, d: &Data) -> usize {
    sim.list.iter().position(|dr| dr.health > 0.0 && !dr.corpse && {
        let feet = dr.feet();
        d.world.segment_surface_host([feet[0], feet[1] + 0.5, feet[2]], [feet[0], feet[1] - 1.0, feet[2]], true)
            .map_or(false, |(_, _, m, _, _)| (m & 0x3f) != 0
                && d.surfaces.and_then(|s| s.effects.get((m & 0x3f) as usize)).map_or(false, |r| r.foot_decal_4 != 0))
    }).expect("a live guard on a decal-carrying material")
}

/// The walk script 060000ca's own markers (ev 4 at 30, ev 5 at 64) drive two real footsteps on the
/// real guard's ground: the surface sound starts (`Effect_Player` 0x1cec48:59-70 — NOT the run
/// arm's 0x286, which drones never take [M 0x1befd8:159-167 bVar1 false]), one print decal lands
/// per step — foot 4's mesh 0x02000627 then foot 8's 0x02000628 [D EffectInfo mat 0x4], y = the
/// ground + 0.05 [M 0x1cec48:118-124], x/z at the foot bone (palette 87 / 51, [M :89-96]) — the
/// two steps' x/z differ — aligned up (normal.y > 0.9), cel-hosted [M :125-136].
#[test]
fn footstep_effect_create_sound_and_decal() {
    let Some(l) = Fixture::load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    // The disc values this test pins [D / L]: the script's markers and the material's foot meshes.
    let scr = d.scripts.get(&0x0600_00ca).expect("060000ca in 07000001");
    let marker = |ev: u16| scr.commands.iter().find_map(|c| match (c.op, c.args.as_slice()) {
        (4, [f, e, ..]) if *e == ev => Some(*f as u32), _ => None,
    }).unwrap_or_else(|| panic!("060000ca carries no ev-{ev} marker"));
    let (f4, f5) = (marker(4), marker(5));
    assert_eq!((f4, f5), (30, 64), "060000ca's markers");
    // The disc's foot-mesh rows for the island's concrete (mat 0x4) [L the parsed EffectInfo —
    // audit-effect-calls's parse; the audit's CONCRETE row] — pinned on the table itself.
    assert_eq!(l.surfaces.effects.get(0x4).unwrap().foot_decal_4, 0x0200_0567, "EffectInfo[0x4]+8");
    assert_eq!(l.surfaces.effects.get(0x4).unwrap().foot_decal_8, 0x0200_0568, "EffectInfo[0x4]+0xc");

    let mut sim = l.sim(l.level_drones());
    sim.tick(&d, &bond_away()); // the placements' ENTER
    let gi = pick_guard(&sim, &d);
    let gpos = sim.list[gi].pos;
    let floor = floor_under(&l, [gpos[0], gpos[1] - 1.0, gpos[2]]);
    let step_sound = nf_formats::sfx::SurfaceSounds::get(&l.surfaces.footstep, 0x4).expect("mat 0x4 carries a footstep sound");
    let decal_before = sim.effects.decals.len();
    // The sound driver's starts this tick (guard_barks_live.rs's drain: `update` emits `Cmd::Start`
    // for the just-started nodes, `take_commands` collects them). The listener stands at the guard:
    // a far one culls non-looped 3-D starts past 1.1×outer (`Sounds::update`).
    let mut starts = |sim: &mut Drones| {
        let lp = sim.list[gi].pos;
        sim.audio.update(Some(&l.sfx), (lp, [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]), &|_| false);
        sim.audio.take_commands().into_iter().filter_map(|c| match c {
            Cmd::Start { id, pos: Some(_), .. } => Some(id), _ => None,
        }).collect::<Vec<u32>>()
    };

    // A state whose UPDATE touches nothing with Bond away (the anim_events harness), so the forced
    // walk script survives to `update_anim` → `script_commands` → the queue → the drain.
    sim.list[gi].state = state::SURRENDERED;
    sim.list[gi].health = 100.0;
    play_frame(&mut sim.list[gi], 0x0600_00ca, f4);
    sim.tick(&d, &bond_away());

    // The foot-4 step: the sound starts, one print lands. (The walk's root motion can carry the
    // guard across a material boundary inside the tick, so the disc row is read under the guard's
    // post-tick feet — the drain's own ray is the same test.)
    let now = starts(&mut sim);
    assert_eq!(now.iter().filter(|&&id| id == step_sound).count(), 1, "exactly one footstep start: {now:?}");
    assert!(!now.iter().any(|&id| id == 0x286), "the run arm's 0x286 never fires for a drone");
    assert_eq!(sim.script_footstep_foot, false, "ev 4 → ScriptFootstepFoot false (foot 4)");
    let decals = &sim.effects.decals;
    assert_eq!(decals.len(), decal_before + 1, "one print decal");
    let print = &decals[decals.len() - 1];
    let gpos = sim.list[gi].pos;
    let feet = sim.list[gi].feet();
    let (t, n, m, _, _) = d.world.segment_surface_host([feet[0], feet[1] + 0.5, feet[2]], [feet[0], feet[1] - 1.0, feet[2]], true).expect("the guard stands on ground");
    let gy = feet[1] + 0.5 - 1.5 * t;
    let rec = l.surfaces.effects.get((m & 0x3f) as usize).expect("the row carries effects");
    assert_eq!(print.mesh, rec.foot_decal_4, "foot 4's decal = EffectInfo[mat]+8 (mat {m:#x})");
    assert!((print.pos[1] - (gy + 0.05)).abs() < 0.01, "print y = ground + 0.05: {} vs ground {gy}", print.pos[1]);
    assert!((print.pos[1] - (floor + 0.05)).abs() < 0.2, "and the ground is the placement's floor ± the walk's root motion");
    assert!((print.pos[0] - gpos[0]).abs() < 1.0 && (print.pos[2] - gpos[2]).abs() < 1.0, "print x/z at the foot bone, near the drone");
    assert!(print.normal[1] > 0.9, "aligned to the surface up (ny {:.2})", n[1]);
    assert!(matches!(print.host, nf_game::effect::DecalHost::Cel(_)), "hosted on the hit cel");
    let foot4_xz = (print.pos[0], print.pos[2]);

    // The foot-8 step (ev 5 pins the foot absolutely).
    play_frame(&mut sim.list[gi], 0x0600_00ca, f5);
    sim.tick(&d, &bond_away());

    let now = starts(&mut sim);
    assert_eq!(now.iter().filter(|&&id| id == step_sound).count(), 1, "the second step's sound: {now:?}");
    assert_eq!(sim.script_footstep_foot, true, "ev 5 → ScriptFootstepFoot true (foot 8)");
    let decals = &sim.effects.decals;
    assert_eq!(decals.len(), decal_before + 2, "the second print");
    let print8 = &decals[decals.len() - 1];
    let feet = sim.list[gi].feet();
    let (_, _, m, _, _) = d.world.segment_surface_host([feet[0], feet[1] + 0.5, feet[2]], [feet[0], feet[1] - 1.0, feet[2]], true).expect("the guard stands on ground");
    let rec = l.surfaces.effects.get((m & 0x3f) as usize).expect("the row carries effects");
    assert_eq!(print8.mesh, rec.foot_decal_8, "foot 8's decal = EffectInfo[mat]+0xc (mat {m:#x})");
    assert!((print8.pos[1] - (floor + 0.05)).abs() < 0.2, "print y = ground + 0.05");
    assert!(foot4_xz != (print8.pos[0], print8.pos[2]), "the two feet print at different x/z (palette slots 55 / 51)");
}

/// `ScriptCam != 0` kills the whole step [M 0x1befd8:191-195]: under a script camera the marker's
/// tick starts no sound and lays no print (the step is lost, not deferred — the flag is per-frame
/// [M :157]); with the camera gone the next marker fires normally.
#[test]
fn script_cam_suppresses_the_footstep() {
    let Some(l) = Fixture::load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let mut sim = l.sim(l.level_drones());
    sim.tick(&d, &bond_away());
    let gi = pick_guard(&sim, &d);
    let step_sound = nf_formats::sfx::SurfaceSounds::get(&l.surfaces.footstep, 0x4).unwrap();
    let mut step_starts = |sim: &mut Drones| {
        let lp = sim.list[gi].pos;
        sim.audio.update(Some(&l.sfx), (lp, [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]), &|_| false);
        sim.audio.take_commands().into_iter().filter(|c| matches!(c, Cmd::Start { id, pos: Some(_), .. } if *id == step_sound)).count()
    };
    let before = (step_starts(&mut sim), sim.effects.decals.len());

    sim.list[gi].state = state::SURRENDERED;
    sim.list[gi].health = 100.0;
    // A live script camera (`ScriptCam != 0`): the port's `scripts.camera()` is Some.
    sim.scripts.objects.list.push(Some(nf_game::nis::Obj::Camera {
        matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        fov: 1.0, src: (0x0600_00ca, 0),
    }));
    play_frame(&mut sim.list[gi], 0x0600_00ca, 30);
    sim.tick(&d, &bond_away());
    assert_eq!(step_starts(&mut sim), before.0, "no footstep sound under the script camera");
    assert_eq!(sim.effects.decals.len(), before.1, "no print under the script camera");
    assert_eq!(sim.script_footstep_foot, false, "the marker still flipped the global (the consumption, not the commands, is gated)");

    // The camera gone: the NEXT marker (ev 5, foot 8 — absolute) fires normally.
    sim.scripts.objects.list.pop();
    play_frame(&mut sim.list[gi], 0x0600_00ca, 64);
    sim.tick(&d, &bond_away());
    assert_eq!(step_starts(&mut sim), before.0 + 1, "the step fires with no script camera");
    assert_eq!(sim.effects.decals.len(), before.1 + 1, "and lays its print");
}
