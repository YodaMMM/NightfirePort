//! NF_ISO test for `NDrone2_initDTYPE_SniperAlert` 0x14e580 (`DroneTypeSettings` row 25, mode 0x19):
//! 0700004a's elevator-shaft garrison carries a mode-2 behaviour block with alert level 3, so
//! `NDrone2_GetDTYPENEW` 0x14c938:105-110 lands +0xC5 = 0x19 at creation and the row-25 init fn (whose
//! only observable effect is the +0x4F8 |= 0x10 stationary flag [M disasm 0x14e580]) must fire. Opt-in: `NF_ISO=<iso>`.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{state, Data, Drone, Drones, Player, Rng};
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

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None } }


/// 35 of 0700004a's 38 drone placements (disc census, ai-dstates-uncited report) have mode 2 +
/// alert level 3 (behaviour-block prop 0x20 = 3 → the 1.0 entry of the alert table) → mode 0x19
/// → row 25: start state 0x2a SniperAim [L 0x29aee8+300] and the init fn's stationary flag.
#[test]
fn alert_snipers_spawn_in_sniper_aim_with_the_stationary_flag() {
    let Some(f) = Fixture::load("0700004a.bin", 0x0700_004a) else { eprintln!("skip: NF_ISO not set"); return };
    let d = f.data();
    let mut sim = f.sim();
    let away = bond_at([0.0, -500.0, 0.0]);
    sim.tick(&d, &away);
    // Five of them stand in the shaft at level start; the rest of the 35-placement garrison are
    // spawner templates released by the floor plates during the ride (docs/levels/0700004a.md §7).
    let alerts: Vec<_> = sim.list.iter().filter(|dr| dr.mode == 0x19).collect();
    assert!(alerts.len() >= 5, "the shaft garrison must spawn as alert snipers (mode 0x19), got {}", alerts.len());
    for dr in &alerts {
        assert!(dr.stationary, "row 25's init fn (NDrone2_initDTYPE_SniperAlert 0x14e580) sets +0x4F8 |= 0x10");
        assert_eq!(dr.start_state(&d), 0x2a, "row 25 start = 0x2a SniperAim [L 0x29aee8+25*12]");
    }
    for dr in sim.list.iter().filter(|dr| dr.present && dr.mode == 0x19) {
        // Level start parks them on a switch channel (state 1) like 07000005's waiters — 0x2a is
        // the start state they take when enabled, which start_state above pins.
        assert!(!dr.enabled, "level-start drones wait for their channel");
    }
    // The filter is not vacuous: the same level spawns the dtype-3 abseil pair (mode 0x1c) whose
    // normal block is mode 0 — the alert mapping only fires on mode-2 blocks.
    let abseil = sim.list.iter().find(|dr| dr.mode == 0x1c).expect("the abseil garrison");
    assert!(!abseil.stationary && abseil.start_state(&d) != 0x2a, "non-mode-2 drones stay out of the sniper-alert row");
}
