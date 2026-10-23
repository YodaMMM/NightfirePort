//! unported-guard-ai-other-4: the three states ported here (HostageDie 0x0b, CrouchCover
//! 0x6e, Investigate 0x7e) through the level load + `Drones::tick`, plus the bots' alert-outcome
//! mapping. Opt-in: `NF_ISO`. Addresses and evidence in docs/systems/guard-ai-other-4.md.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drone, Drones, Player as Bond, Rng, state};
use std::collections::HashMap;

struct LevelX {
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
    hud: nf_formats::hud::HudTables,
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    surfaces: nf_formats::sfx::SurfaceSounds,
    voices: Vec<nf_formats::sfx::VoiceLine>,
}

impl LevelX {
    fn load(code: u32) -> Option<LevelX> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let name = format!("{code:08x}.bin");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)).unwrap();
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok()).map(|k| (k.id as u8, k)).collect();
        let m = level.main_map().unwrap();
        let nav = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref().unwrap(), &world, code);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let volumes = nf_game::aivolume::build(&level, code);
        Some(LevelX {
            tables: drone::DroneAnimData::from_action_elf(&elf).unwrap(),
            props: drone::PropertyTables::from_action_elf(&elf).unwrap(),
            weapons: WeaponData::from_action_elf(&elf).unwrap(),
            sfx: nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap(),
            tuning: { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap(); nf_game::tuning::read_tuning_vars(&mut t, code); t },
            hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap(),
            sequences: disc::level_sequences(&iso, &name).unwrap(),
            models: nf_game::pickup::models(&level),
            hud: nf_formats::hud::HudTables::from_action_elf(&elf).unwrap(),
            surfaces: nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).unwrap(),
            voices: nf_formats::sfx::voice_lines(&elf).unwrap(),
            level, scripts, world, skeletons, nav, covers, volumes,
        })
    }

    fn data(&self, code: u32) -> Data<'_> {
        Data {
            tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: code,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1, ai_points: &[],
        }
    }

    fn drones(&self, code: u32) -> (Vec<Drone>, Rng) {
        let d = self.data(code);
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        (drones, rng)
    }
}

fn bond_far() -> Bond {
    let p = [0.0, -500.0, 0.0];
    Bond { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None }
}

fn bond_at(p: [f32; 3]) -> Bond {
    Bond { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None }
}

const L05: u32 = 0x0700_0005;

fn armed(w: u32) -> bool { (2..=0x46).contains(&w) || (0x6a..=0x71).contains(&w) }

fn guard_sim(l: &LevelX) -> (Drones, usize) {
    let d = l.data(L05);
    let (drones, rng) = l.drones(L05);
    let mut sim = Drones::new(drones, rng);
    sim.attach_level_id(&l.level, L05);
    for _ in 0..5 { sim.tick(&d, &bond_far()); }
    let gi = sim.list.iter().position(|dr| dr.started && dr.present && dr.enabled && !dr.corpse && armed(dr.weapon) && !dr.holstered)
        .expect("a running armed 07000005 guard");
    (sim, gi)
}

fn bond_near(sim: &Drones, gi: usize, dist: f32) -> Bond {
    let g = sim.list[gi].pos;
    bond_at([g[0] + dist, g[1], g[2]])
}

fn enter(sim: &mut Drones, d: &Data, gi: usize, s: u8, player: &Bond) {
    sim.list[gi].set_state(s);
    sim.tick(d, player);
}

fn entered(sim: &Drones, gi: usize, s: u8) -> bool { sim.list[gi].log.iter().any(|&(_, x)| x == s) }

/// The state ids ported here sit at the `NDrone2_StateFuncs` 0x29b300 indices the decompile
/// dispatches [M table read]: HostageDie 0x0b, CrouchCover 0x6e, Investigate 0x7e. Sister: the
/// neighbouring ids are NOT these states (a shifted constant would break every router arm).
#[test]
fn state_ids_match_the_game_table_indices() {
    assert_eq!(state::HOSTAGE_DIE, 0x0b);
    assert_eq!(state::CROUCH_COVER, 0x6e);
    assert_eq!(state::INVESTIGATE, 0x7e);
    assert_ne!(state::INVESTIGATE, state::DRONE_STUCK, "0x7e vs 0x7f are distinct states [M 0x168070 / 0x165ef0]");
    assert_ne!(state::CROUCH_COVER, state::AIM_CROUCH, "0x6e vs 0x6f are distinct states [M 0x166d90 / 0x166f28]");
}

/// `NDrone2_DSTATE_CrouchCover` 0x166d90: ENTER = vision on, alert status 0 (share 0.0), crouch
/// anim 0x11, timer C = now + 2 s + Rand(60) frames [M case 1]; at timer C → AimStand 0x67 [M case
/// 0xc]. Sister: AimCrouch with a non-0x11 mode does NOT route here (the mode gate [M 0x166f28:18]).
#[test]
fn crouch_cover_holds_two_seconds_then_stands() {
    let Some(l) = LevelX::load(L05) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L05);
    let (mut sim, gi) = guard_sim(&l);
    let player = bond_near(&sim, gi, 8.0);
    enter(&mut sim, &d, gi, state::CROUCH_COVER, &player);
    assert!(sim.list[gi].vision_on, "CrouchCover ENTER sets +0x4F8 0x10000 [M 0x166d90 case 1]");
    // `Drone_AlertStatusSet(0)` writes the STATUS byte +0x51C (and its prev/time fields) only, never
    // the alertness +0x50C [M 0x13a8d8; daily audit 2026-10-22 — the old pin read the alertness float].
    assert_eq!(sim.list[gi].alert_status, 0, "CrouchCover ENTER sets alert status 0 [M 0x166d90 case 1 → 0x13a8d8]");
    let t0 = sim.list[gi].timer_c.expect("timer C armed on enter");
    let dt = t0 - sim.frame;
    assert!((120..=181).contains(&dt), "timer C = 2 s (120) + Rand(60) frames [M 0x166d90 case 1]; got {dt}");
    let mut stood = false;
    for _ in 0..260 { sim.tick(&d, &player); if entered(&sim, gi, state::AIM_STAND) { stood = true; break; } }
    assert!(stood, "timer C → AimStand 0x67 [M 0x166d90 case 0xc]; log {:x?}", sim.list[gi].log);
    // Sister: a normal (non-0x11 mode) guard's AimCrouch plays the crouch-aim anim with follow-on
    // AimCrouchFire — it must not land in CrouchCover [M 0x166f28:18-22 else-branch].
    let (mut sim2, g2) = guard_sim(&l);
    let p2 = bond_near(&sim2, g2, 8.0);
    let mode0 = sim2.list[g2].mode;
    if mode0 != 0x11 {
        enter(&mut sim2, &d, g2, state::AIM_CROUCH, &p2);
        assert!(!entered(&sim2, g2, state::CROUCH_COVER), "mode {} != 0x11: AimCrouch does not route to CrouchCover [M 0x166f28:18]", mode0);
    }
}

/// `NDrone2_DSTATE_Investigate` 0x168070: ENTER = vision on, alert status 2 (the `+0x50C` share
/// 0.66), the alert-idle anim [M case 1]; UPDATE with no opponent pick and share < 0.66 → Idle 0x04
/// [M case 3]. Sister: a sighted opponent leaves for Attack.
#[test]
fn investigate_stands_down_to_idle_when_nothing_is_found() {
    let Some(l) = LevelX::load(L05) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L05);
    let (mut sim, gi) = guard_sim(&l);
    let far = bond_far();
    let alert_before = sim.list[gi].alert;
    enter(&mut sim, &d, gi, state::INVESTIGATE, &far);
    assert!(sim.list[gi].vision_on, "Investigate ENTER sets +0x4F8 0x10000 [M 0x168070 case 1]");
    // `Drone_AlertStatusSet(2)` writes the +0x51C status byte ONLY [M 0x13a8d8] (review 2026-10-21:
    // this test used to pin alertness 0.66 — the 0x2f5cc8 table is read only by DoModeSettingsNEW).
    assert_eq!(sim.list[gi].alert_status, 2, "alert status 2 [M 0x168070 case 1]");
    assert_eq!(sim.list[gi].alert, alert_before, "the alertness +0x50C is untouched [M 0x13a8d8]");
    for _ in 0..30 { sim.list[gi].ever_seen = false; sim.list[gi].alert = 0.0; sim.tick(&d, &far); }
    assert!(entered(&sim, gi, state::IDLE), "no pick, share < 0.66 → Idle 0x04 [M 0x168070 case 3]; log {:x?}", sim.list[gi].log);
    // Sister: with an opponent in sight (the game's `+0x170 != 0`) the pick wins over the idle
    // gate — Investigate leaves for Attack instead [M 0x168070 case 3, `iVar4 != 0` arm].
    let (mut sim2, g2) = guard_sim(&l);
    let near = bond_near(&sim2, g2, 6.0);
    enter(&mut sim2, &d, g2, state::INVESTIGATE, &near);
    sim2.list[g2].ever_seen = true;
    sim2.list[g2].seen = true;
    sim2.tick(&d, &near);
    assert!(entered(&sim2, g2, state::ATTACK), "a sighted opponent → Attack 0x56 [M 0x168070 case 3 / EnemyLookForOpponent]; log {:x?}", sim2.list[g2].log);
}

/// The bots' alert outcomes (`mp/bots.rs react_to_drone_alert_msg`, the SetState tail of
/// `NDrone2_ReactToDroneAlertMsg` 0x1437a0 [M LAB_00143e30]): the dead-body shout 0x15 →
/// SeenDeadBody 0x9e [M :298-326], the surrender shout 0x13 (`Surrender_Anim` 0x1626a0:74) →
/// SeenSurrenderedDrone 0x9f [M :267-292]; on 0x15 modes 9/10/0x12 attack (0x12 breaks to the 0x56 tail
/// [M :312, :363-366]) and 0x1e drops out; on 0x13 modes 9/10/0x12/0x13 attack; 0x16 attacks for every
/// mode but 0x1e, with no property gate [M :327-366] (daily audit 2026-10-22)
/// (the mode byte is the HANDLING drone's [M 0x1437a0:24]). Sister: the
/// property gates actually gate (no prop → None).
#[test]
fn bot_alert_shouts_resolve_to_the_seen_states() {
    let all = |_p: u32| true;
    let none = |_p: u32| false;
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x15, 0x0b, all, true), Some(0x9e), "dead-body shout, receiver mode 0x0b → 0x9e [M 0x1437a0:307-326]");
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x15, 0x12, all, true), Some(0x56), "receiver mode 0x12 attacks (break → the 0x56 tail) [M 0x1437a0:312, 363-366]");
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x15, 0x1e, all, true), None, "receiver mode 0x1e drops out [M 0x1437a0:320-321]");
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x13, 0x05, all, true), Some(0x9f), "surrender shout 0x13, ordinary receiver mode → 0x9f [M 0x1437a0:283-290]");
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x13, 0x13, all, true), Some(0x56), "receiver mode 0x13 attacks instead [M 0x1437a0:276-282]");
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x16, 0x05, none, true), Some(0x56), "alert 0x16: no property gate, Attack for mode 5 [M 0x1437a0:327-366]");
    // Review 2026-10-21: mode 10 is in the same attack group as 9/0x12/0x13 [M 0x1437a0:336-341] (the branch left it out).
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x13, 0x0a, all, true), Some(0x56), "receiver mode 10 attacks too");
    // Sister: without the receiver property the shout is ignored [M 0x1437a0:271-275 (0x4F), 300-303 (0x29)].
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x15, 0x0b, none, true), None);
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x13, 0x05, none, true), None);
    assert_eq!(nf_game::mp::bots::react_to_drone_alert_msg(0x15, 0x0b, all, false), None, "out of shouting range [M 0x1437a0:305]");
}
