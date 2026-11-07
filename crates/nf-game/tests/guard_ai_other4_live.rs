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
    Bond { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, body: Default::default(), cel: None, move_local: [0.0; 2], riding: false, crouched: false }
}

fn bond_at(p: [f32; 3]) -> Bond {
    Bond { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, body: Default::default(), cel: None, move_local: [0.0; 2], riding: false, crouched: false }
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
/// dispatches [M NDrone2_ProcessStateMachine 0x175778:11 — `NDrone2_StateFuncs + id*4` call].
/// The table entries are disc data: entry 0x0b → 0x1725a0 HostageDie, 0x6e → 0x166d90
/// CrouchCover, 0x7e → 0x168070 Investigate, 0x7f → 0x165ef0 DroneStuck [D ACTION.ELF bytes at
/// 0x29b300+id·4]. Sister: the neighbouring ids are NOT these states (a shifted constant would
/// break every router arm).
#[test]
fn state_ids_match_the_game_table_indices() {
    assert_eq!(state::HOSTAGE_DIE, 0x0b, "table entry 0x0b holds HostageDie 0x1725a0 [D 0x29b300+0x2c = 0x001725a0]");
    assert_eq!(state::CROUCH_COVER, 0x6e, "table entry 0x6e holds CrouchCover 0x166d90 [D 0x29b300+0x1b8 = 0x00166d90]");
    assert_eq!(state::INVESTIGATE, 0x7e, "table entry 0x7e holds Investigate 0x168070 [D 0x29b300+0x1f8 = 0x00168070; the id·4 dispatch M 0x175778:11]");
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

// ==== guard-dead-body-shout (2026-11-07): the shout-range law, the SP receiver jumptable,
// ==== and the shout-receiver property census ====================================================

/// `NDrone2_InShoutingRange` 0x1788a0 (the port's `mp::bots::in_shouting_range`) and the SP
/// receiver tail of `DroneVision_ShoutFromOtherDrone` 0x177a30 (the port's `sp_shout_outcome`).
/// The decompiler dropped 0x177a30's body (23 "removing unreachable block" warnings, an empty
/// `return 0`), so the pins cite the disassembly and the jumptable bytes:
/// 0x2fb6c0 → 0x177b80 (0x11), 0x2fb6c4 → 0x177b80 (0x12), 0x2fb6c8 → 0x177bf4 (0x13, return 0),
/// 0x2fb6cc → 0x177bc4 (0x14), 0x2fb6d0 → 0x177bf4 (0x15, return 0), 0x2fb6d4 → 0x177b80 (0x16)
/// [D ACTION.ELF, read with tools/crosscheck/mipsdis.py].
#[test]
fn shouting_range_law_and_the_sp_shout_jumptable() {
    use nf_game::mp::bots::{in_shouting_range, sp_shout_outcome};
    // The level bands [M 0x1788a0:14-25]: level > 0x0700000f or < 0x0700000c → 15.0/20.0;
    // 0x0700000c..=0x0700000f → 40.0/100.0. Inside the first bound always in; inside the
    // second only with the cel word `*(sender obj+0x20) == *(receiver obj+0x20)`; else out
    // [M 0x1788a0:30-33, 41-44; the f32 c.lt.s compares].
    assert!(in_shouting_range(14.9, 0x0700_0001, false), "under the near bound (15.0) — in range with no cel match [M 0x1788a0:47-53]");
    assert!(!in_shouting_range(15.0, 0x0700_0001, false), "at 15.0 the c.lt.s fails → the cel band [M 0x1788a0:53]");
    assert!(in_shouting_range(15.0, 0x0700_0001, true), "15.0..20.0 is the same-cel band [M 0x1788a0:55-61]");
    assert!(in_shouting_range(19.9, 0x0700_0001, true));
    assert!(!in_shouting_range(19.9, 0x0700_0001, false), "the mid band without the cel word → out [M 0x1788a0:56]");
    assert!(!in_shouting_range(20.0, 0x0700_0001, true), "≥ 20.0 out of range at any cel [M 0x1788a0:60]");
    assert!(in_shouting_range(39.9, 0x0700_000c, false), "level 0x0700000c switches to the 40/100 band [M 0x1788a0:23-35]");
    assert!(in_shouting_range(40.0, 0x0700_000c, true), "40.0..100.0 same-cel [M 0x1788a0:37-44]");
    assert!(!in_shouting_range(40.0, 0x0700_000c, false));
    assert!(in_shouting_range(99.9, 0x0700_000f, true));
    assert!(!in_shouting_range(99.9, 0x0700_000f, false));
    assert!(!in_shouting_range(100.0, 0x0700_000f, true), "≥ 100.0 out [M 0x1788a0:42]");
    assert!(!in_shouting_range(39.9, 0x0700_0010, true), "level 0x07000010 leaves the band → 15/20 again, so 39.9 is out [M 0x1788a0:16-19]");
    // The SP jumptable [D 0x2fb6c0]: the dead-body 0x15 and surrender 0x13 shouts land in the
    // return-0 entries — an SP guard NEVER reacts to them; the armed-intruder/scream/team
    // shouts take the Attack arm (the receiver's `+0x4F8 |= 0x4000000 | 0x20000000`, 0x56 [M
    // 0x177bc8-bf0], the caller's SM pending slot +0x1C [M 0x15bf18 Idle's alert case]).
    assert_eq!(sp_shout_outcome(0x15), None, "the dead-body shout: jumptable 0x2fb6d0 → return 0 [D]");
    assert_eq!(sp_shout_outcome(0x13), None, "the surrender shout: jumptable 0x2fb6c8 → return 0 [D]");
    assert_eq!(sp_shout_outcome(0x11), Some(0x56), "armed intruder → the Attack arm [D 0x2fb6c0]");
    assert_eq!(sp_shout_outcome(0x12), Some(0x56), "scream → the Attack arm [D 0x2fb6c4]");
    assert_eq!(sp_shout_outcome(0x16), Some(0x56), "team shout → the Attack arm [D 0x2fb6d4]");
    // Sister: the state table itself — 0x9E/0x9F are the two Seen states, 0x56 Attack [D
    // 0x29b300+id·4: 0x9e → 0x16acd0 SeenDeadBody, 0x9f → 0x16ade0 SeenSurrenderedDrone,
    // 0x56 → 0x164210 Attack]. The port never reaches them: nothing SetStates 0x9E/0x9F
    // (the only computor of those ids, `NDrone2_ReactToDroneAlertMsg` 0x1437a0, drops its
    // only possible callers — mode-0x1E bots — in both the 0x15 [M :320-321] and 0x13 [M
    // :283-290] arms).
    assert_eq!(nf_game::drone::state::ATTACK, 0x56);
}

/// The shout-receiver property census on the disc — `behaviour_util_getProperty(0x29/0x4F,
/// Drone+0x4D8)` read from the levels' placement data through the port's parser (`Drone::prop`
/// = the game's property tables) [D ISO level bins via the port's nf-formats loader; the gate
/// sites are 0x1437a0:300-303 (0x29, the dead-body receiver) and :271-275 (0x4F, the surrender
/// receiver)]. The two levels the task names; the counts are the cast list any live pin uses.
#[test]
fn shout_receiver_props_census_07000001_and_07000009() {
    for code in [0x0700_0001u32, 0x0700_0009] {
        let l = LevelX::load(code).expect("NF_ISO");
        let d = l.data(code);
        let (drones, _rng) = l.drones(code);
        let p29 = drones.iter().filter(|g| g.prop(&d, 0x29)).count();
        let p4f = drones.iter().filter(|g| g.prop(&d, 0x4f)).count();
        println!("level {code:08x}: {p29} guards with prop 0x29, {p4f} with prop 0x4f, {} guards total", drones.len());
        assert!(p29 > 0, "{code:08x} carries prop-0x29 (dead-body receiver) guards [D]");
    }
}
