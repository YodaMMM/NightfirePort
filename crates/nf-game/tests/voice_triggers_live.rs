//! audit-voice-triggers live pins: the state-arm callers of the drone voice path, driven on real
//! level data with the real `Drone_SfxInfo` (ACTION.ELF, `nf_formats::sfx::voice_lines`).
//! Run with: `NF_ISO=… cargo test -p nf-game --test voice_triggers_live -- --test-threads=1`
//!
//! The audited fixes (each cites its game address in `crates/nf-game/src/drone.rs`):
//! - `NDrone2_DSTATE_RunAwayFromObject` 0x168aa8 case 1 → `NDrone2_DeathTalk` [M 0x168aa8:21];
//! - `NDrone2_DSTATE_UnderCoverFire` 0x16a3e0 case 1 → `NDrone2_AttackTalk` [M 0x16a3e0:33];
//! - `NDrone2_DSTATE_UnderCoverSniperFire` 0x16a610 case 1 → `NDrone2_AttackTalk` [M 0x16a610:20];
//! - `NDrone2_DSTATE_PlayScript` 0x15ba68 (0x6000168/0x6000169 arms) → `NDrone2_AttackTalk` at
//!   ENTER [M :72], msg 3 after +0x11c+0x1e [M :120], msg 5 [M :157].
//!
//! 07000005 is the proof level: all 13 drones are character 0x0500002e/2f/30 → voice type 2
//! [L speech.md §2.1], whose death line is entry 5 (sfxId 139, repeat 30) and whose attack lines
//! are 0x37..0x3f (stream, repeat 180) [L speech.md §3].

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{msg, state, Data, Drones, Player, Rng};
use std::collections::HashMap;

struct Fixture {
    tables: drone::DroneAnimData,
    weapons: WeaponData,
    props: drone::PropertyTables,
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
    voices: Vec<nf_formats::sfx::VoiceLine>,
}

impl Fixture {
    fn load(level_name: &str, level_id: u32) -> Option<Fixture> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let voices = nf_formats::sfx::voice_lines(&elf).unwrap();
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
        Some(Fixture { tables, weapons, props, level, level_id, scripts, world, sequences, skeletons, nav, covers, tuning, hitboxes, volumes, models, skins, voices })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world,
            level: self.level_id, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav,
            covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: None, models: &self.models,
            surfaces: None, voices: Some(&self.voices), difficulty: 1, ai_points: &[] }
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

const LEVEL: &str = "07000005.bin";
const LEVEL_ID: u32 = 0x0700_0005;

/// The real `Drone_SfxInfo` rows the audited barks resolve to on the type-2 voice set:
/// death 5 → sfxId 139 / repeat 30, pain 0x13 → 138 / repeat 0, attack 0x37..0x3f → stream ids
/// 175..198 / repeat 180, seen 0x49..0x4e → repeat 0 [L speech.md §3, read back from ACTION.ELF].
#[test]
fn the_type2_voice_rows_match_the_table() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
    let v = nf_formats::sfx::voice_lines(&elf).unwrap();
    assert_eq!(v[5].sfx, 139, "the type-2 death line [L speech.md §3]");
    assert_eq!(v[5].repeat, 30, "entry 5's repeatDelay 30 [L speech.md §1.1]");
    assert_eq!(v[0x13].sfx, 138, "the type-2 pain line [L speech.md §3]");
    for k in 0x37usize..=0x3f {
        assert_eq!(v[k].repeat, 180, "the attack stream entries repeat at 180 [L speech.md §3]");
        assert!(v[k].sfx >= 175 && v[k].sfx <= 198, "attack entry {k} is a stream id [L speech.md §3]");
    }
    for k in 0x49usize..=0x4e {
        assert_eq!(v[k].repeat, 0, "the seen entries have no repeat delay [L speech.md §3]");
    }
}

/// A real 07000005 guard, scared by a frag blast through the real tick drain
/// (`HandleExplosives` 0x1435f0's queue → msg 0x21 → `DroneFunc_ConsiderExplosive` 0x1431c0 →
/// `RunAwayFromObject` 0x86), barks `NDrone2_DeathTalk` on the ENTER [M 0x168aa8:21] — the
/// type-2 death line 5 (sfxId 139 [L speech.md §3]), forced.
#[test]
fn a_frag_blast_flees_a_guard_with_his_death_bark() {
    let Some(fx) = Fixture::load(LEVEL, LEVEL_ID) else { eprintln!("skip: NF_ISO not set"); return };
    let mut sim = fx.sim();
    let d = fx.data();
    for _ in 0..3 { sim.tick(&d, &bond_at([0.0, 0.0, 0.0])); }
    // A present guard whose type row grants the flee props (0x3a/0x3b — the type-2 rows read
    // through the drone's own property words [M 0x1431c0:40-67 via the drain's gate]).
    let ai = sim.list.iter().position(|x| x.present && x.enabled && !x.corpse && x.health > 0.0
        && (x.prop_value(&fx.props, 0x3a) != 0 || x.prop_value(&fx.props, 0x3b) != 0))
        .unwrap_or_else(|| panic!("no flee-capable guard on 07000005 [D: props read from ACTION.ELF tables]"));
    let at = sim.list[ai].pos;
    // The blast 1.0 off the guard: inside the 12.0 scare radius, a non-degenerate LOS ray.
    // gas-draws-8 (ffcb70a5d): the HandleExplosives broadcast walks the LIVE grenades every
    // frame [M 0x1435f0:20-52 + 0x138b10:18-19] — the take-once explosive_queue is gone — so
    // the scare is driven by a live frag at rest beside the guard (state 1, the 0x34 fuse is
    // 210 ticks [L record 0x34], one tick can't kill it).
    sim.grenades.push(nf_game::grenade::Grenade {
        pos: [at[0] + 1.0, at[1], at[2]], velocity: [0.0, 0.0, 0.0], speed: 0.0, travelled: 0.0,
        range: 210.0, damage: 8.0, radius: 12.0, weapon_type: 0x34, mesh: 0, owner: usize::MAX,
        state: 1, bounces: 0, smoke_popped: false, resting: true, stuck: false, seek_target: None,
        phase: None, burn: 0, defused: false, born: 0,
        loop_handle: 0,
    }); // the frag weapon type [M 0x2a/0x34 drain map, drone.rs tick]
    sim.tick(&d, &bond_at([0.0, 0.0, 0.0]));
    assert_eq!(sim.list[ai].state, state::RUN_AWAY_FROM_OBJECT,
        "the scary blast flees the guard [M 0x1431c0:194-195]");
    assert_eq!(sim.list[ai].talks, vec![(5usize, true)],
        "the ENTER barks the type-2 death line 5, forced [M 0x168aa8:21; 0x13eff8]");
}
