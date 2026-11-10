//! NF_ISO test for `NDrone2_ControlDTYPE_Zoe` 0x14b058 — the escort-abandonment monitor.
//!
//! The 07000008 escort VIP (skin 0x0500003b, placement dtype 12 + kind 0xc) lands `+0xC5 = 0x11`
//! through `NDrone2_GetDTYPENEW` 0x14c938 case 0xc [M 0x14c938:54-59; docs/levels/07000008.md §3],
//! and `Drone_Control` 0x139268:60-64 dispatches `DroneTypeSettings` row 17's control fn for her
//! [D table 0x29aee8+204] — 0x14b058, whose body is the monitor + the `ControlSTANDARD` tail-call:
//! while switch channel 1 is on, `switch_channels[0x9b] = 1` [M 0x14b058:5-8]; while 0x9b is armed
//! and a player exists, `Vec_Dist3D(obj+0x30, glb_players+0x30) > 10.0` fails the escort EVERY tick
//! through `DroneFunc_SetMissionFailReason` 0x13e560's default arm (reason 0xf: switch channel 99,
//! no FailLabel [M 0x13e560:20-25]). 07000008's own scripts gate on channel 1 (ScriptPlayer #295
//! CableCar, mask 1 on ch 1 [D docs/levels/07000008.md §2/§4]) — the monitor arms in real play.
//! Opt-in: `NF_ISO=<iso>`.

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
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: HashMap<u32, nf_game::pickup::Model>,
    skins: HashMap<u32, nf_formats::character::Skin>,
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
        Some(Fixture { tables, props, weapons, level, level_id, scripts, world, sequences, skeletons, nav, covers, tuning, hitboxes, volumes, models, skins })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world,
            level: self.level_id, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav,
            covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: None, models: &self.models,
            surfaces: None, voices: None, difficulty: 1,
                ai_points: &[],
            }
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

/// The monitor: ch 1 arms 0x9b; while 0x9b is armed, Bond > 10 (3D) from Zoe fails the escort
/// (ch 99, no FailLabel); inside the band it does not; and the arm LATCHES after ch 1 clears
/// (the game's switch_channels[0x9b] byte — nothing in the game clears it).
#[test]
fn zoe_escort_abandonment_arms_on_channel_1_and_fails_past_ten_units() {
    let Some(f) = Fixture::load("07000008.bin", 0x0700_0008) else { eprintln!("skip: NF_ISO not set"); return };
    let d = f.data();
    let mut sim = f.sim();
    // The escort VIP: exactly one placed mode-0x11 drone (the 07000008 ally).
    let zoe_idx = sim.list.iter().position(|dr| dr.present && dr.mode == 0x11)
        .expect("07000008 must place the escort VIP (dtype 12 + kind 0xc → GetDTYPENEW 0x11)");
    let zoe = sim.list[zoe_idx].pos;
    // Bond starts at her side (the escort keeps him within the band).
    let near = bond_at([zoe[0] + 3.0, zoe[1], zoe[2]]);
    // The level script's ch-1 raise (07000008's ScriptPlayer #295 gate channel [D]).
    sim.set_channel(1);
    sim.tick(&d, &near);
    sim.tick(&d, &near);
    assert!(sim.channels.on(0x9b), "ch 1 must arm switch_channels[0x9b] through the Zoe control fn [M 0x14b058:5-8]");
    assert!(!sim.channels.on(99), "within the 10-unit band the monitor must not fail the escort [M 0x14b058:11-14]");
    // The arm latches: the game's byte survives ch 1 clearing.
    sim.channels.set(1, 0, sim.frame);
    sim.tick(&d, &near);
    assert!(sim.channels.on(0x9b), "switch_channels[0x9b] has no clearer in the game — it latches [M]");
    // Bond abandons her: > 10.0 in 3D → the default arm of SetMissionFailReason(0xf): ch 99.
    let far = bond_at([zoe[0] + 12.0, zoe[1], zoe[2]]);
    sim.tick(&d, &far);
    assert!(sim.channels.on(99), "Vec_Dist3D > 10.0 while 0x9b is armed must raise ch 99 (reason 0xf) [M 0x14b058:11-16, 0x13e560 default arm]");
    // Reason 0xf sets NO FailLabel (the default arm skips Mission_SetFailLabel [M 0x13e560]).
    assert_eq!(sim.mission.fail_label, 0, "reason 0xf rides the default arm — no FailLabel write [M 0x13e560:20-25]");
}

/// The band is exclusive at 10.0 and measured in 3D: 10.0 exactly does not fail, 10.5 does.
#[test]
fn zoe_band_is_strictly_greater_than_ten_in_3d() {
    let Some(f) = Fixture::load("07000008.bin", 0x0700_0008) else { eprintln!("skip: NF_ISO not set"); return };
    let d = f.data();
    let mut sim = f.sim();
    let zoe_idx = sim.list.iter().position(|dr| dr.present && dr.mode == 0x11)
        .expect("the escort VIP");
    let zoe = sim.list[zoe_idx].pos;
    sim.set_channel(1);
    // Exactly 10.0: the game's `10.0 < fVar1` refuses (float 10.0 vs 10.0 → not greater [M 0x14b058:13]).
    let edge = bond_at([zoe[0] + 10.0, zoe[1], zoe[2]]);
    sim.tick(&d, &edge);
    sim.tick(&d, &edge);
    assert!(!sim.channels.on(99), "exactly 10.0 is not `10.0 < dist` — no fail [M 0x14b058:13]");
    // 10.5 horizontal (same y): 3D distance 10.5 > 10.0 → fail.
    let out = bond_at([zoe[0] + 10.5, zoe[1], zoe[2]]);
    sim.tick(&d, &out);
    assert!(sim.channels.on(99), "10.5 > 10.0 must fail [M 0x14b058:13-16]");
}
