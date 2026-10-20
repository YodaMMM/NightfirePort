//! ai-missing-states (847): the four guard states the call-tree check found unported — AimStand 0x67
//! (completed: DrawWeapon/AltAttack routing), AltAttack 0x72 (+ NDrone2_CanAltAttack, PlrStat_OkToUpdate),
//! CombatWait 0x5e (the port's former COMBAT_DRONE_AHEAD), NoOpponent 0x60, HostageDead 0x0f —
//! through the level load + `Drones::tick`. Opt-in: `NF_ISO`.


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
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: std::collections::HashMap<u32, nf_game::pickup::Model>,
    hud: nf_formats::hud::HudTables,
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

    /// The level-start drone list.
    fn drones(&self, code: u32) -> (Vec<Drone>, Rng) {
        let d = self.data(code);
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        (drones, rng)
    }
}

fn full_data(l: &LevelX, code: u32) -> Data<'_> { l.data(code) }

fn bond_far() -> Bond {
    let p = [0.0, -500.0, 0.0];
    Bond { pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None }
}

fn bond_at(p: [f32; 3]) -> Bond {
    Bond { pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None }
}


const L05: u32 = 0x0700_0005;

fn armed(w: u32) -> bool { (2..=0x46).contains(&w) || (0x6a..=0x71).contains(&w) }

/// A running (started, present, enabled), armed 07000005 guard (index) after a short warm-up from the
/// level start (the level's own scripts delete some placements on the first ticks).
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

/// Bond `dist` in front of guard `gi` on its own floor.
fn bond_near(sim: &Drones, gi: usize, dist: f32) -> Bond {
    let g = sim.list[gi].pos;
    bond_at([g[0] + dist, g[1], g[2]])
}

/// Put guard `gi` into state `s` (the router's exit → enter on the next delivery) and tick once.
fn enter(sim: &mut Drones, d: &Data, gi: usize, s: u8, player: &Bond) {
    sim.list[gi].set_state(s);
    sim.tick(d, player);
}

fn entered(sim: &Drones, gi: usize, s: u8) -> bool { sim.list[gi].log.iter().any(|&(_, x)| x == s) }

/// `NDrone2_DSTATE_AimStand` 0x166178 case 1 [M :18-20]: a drone that is not itself armed
/// (`NDrone2_ObjectIsArmed(self)` 0) leaves for DrawWeapon 0x66. Sister: an armed one stays and aims.
#[test]
fn aim_stand_sends_an_unarmed_guard_to_draw_weapon() {
    let Some(l) = LevelX::load(L05) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L05);
    let (mut sim, gi) = guard_sim(&l);
    let player = bond_near(&sim, gi, 6.0);
    sim.list[gi].holstered = true; // weapon datum swapped out — unarmed for ObjectIsArmed
    enter(&mut sim, &d, gi, state::AIM_STAND, &player);
    sim.tick(&d, &player);
    assert!(entered(&sim, gi, state::DRAW_WEAPON), "unarmed → DrawWeapon 0x66 [M 0x166178:18-20]; log {:x?}", sim.list[gi].log);
    // Sister: armed, Bond out of the 1.5 alt-attack reach → the aim (no DrawWeapon, no AltAttack).
    let (mut sim2, g2) = guard_sim(&l);
    let p2 = bond_near(&sim2, g2, 6.0);
    enter(&mut sim2, &d, g2, state::AIM_STAND, &p2);
    assert!(!entered(&sim2, g2, state::DRAW_WEAPON) && !entered(&sim2, g2, state::ALT_ATTACK),
        "armed and 6 away: AimStand aims [M 0x166178:21-45]; log {:x?}", sim2.list[g2].log);
    assert!(sim2.list[g2].vision_on, "AimStand's enter sets +0x4F8 0x10000 (the port's vision_on) [M 0x166178:16]");
}

/// `NDrone2_CanAltAttack` 0x1549e0: within 1.5 and the 0x3D anim available (and `PlrStat_OkToUpdate`
/// in SP: the mission running) → AimStand routes to AltAttack 0x72 [M 0x166178:21-24]. Sister: the
/// mission over (`Mission_Status` ≥ 2 → `PlrStat_OkToUpdate` 0) → no alt attack.
#[test]
fn aim_stand_close_in_takes_the_alt_attack_while_the_mission_runs() {
    let Some(l) = LevelX::load(L05) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L05);
    let (mut sim, gi) = guard_sim(&l);
    if !sim.list[gi].can_do(&d, 0x3d) { eprintln!("skip: this guard's anim set has no 0x3D alt attack"); return; }
    let player = bond_near(&sim, gi, 1.0);
    sim.tick(&d, &player); // opponent info: dist ≈ 1.0
    enter(&mut sim, &d, gi, state::AIM_STAND, &player);
    sim.tick(&d, &player);
    assert!(entered(&sim, gi, state::ALT_ATTACK), "1.0 away, mission running → AltAttack 0x72 [M 0x1549e0]; log {:x?}", sim.list[gi].log);
    // Sister: the mission over — PlrStat_OkToUpdate false → plain AimStand.
    let (mut sim2, g2) = guard_sim(&l);
    sim2.mission.set_state(2);
    let p2 = bond_near(&sim2, g2, 1.0);
    sim2.tick(&d, &p2);
    enter(&mut sim2, &d, g2, state::AIM_STAND, &p2);
    sim2.tick(&d, &p2);
    assert!(!entered(&sim2, g2, state::ALT_ATTACK), "Mission_Status 2 → PlrStat_OkToUpdate 0 → no alt attack [M 0x1993e0]");
}

/// `NDrone2_DSTATE_NoOpponent` 0x1650a0: with no opponent object the drone stays (alert idle,
/// a 1 s timer that re-arms) instead of the port's old drop to Idle; once there is one it goes to
/// Attack 0x56 [M :24-42]. Sister: an opponent already known → Attack.
#[test]
fn no_opponent_waits_then_attacks_when_one_exists() {
    let Some(l) = LevelX::load(L05) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L05);
    let (mut sim, gi) = guard_sim(&l);
    let far = bond_far();
    sim.list[gi].ever_seen = false;
    let mark = sim.list[gi].log.len();
    enter(&mut sim, &d, gi, state::NO_OPPONENT, &far);
    for _ in 0..150 { sim.list[gi].ever_seen = false; sim.tick(&d, &far); }
    assert_eq!(sim.list[gi].state, state::NO_OPPONENT, "no opponent: NoOpponent holds (timer C re-arms) [M 0x1650a0:36-42]; log {:x?}", sim.list[gi].log);
    assert!(!sim.list[gi].log[mark..].iter().any(|&(_, x)| x == state::IDLE), "the old port's straight drop to Idle is gone; log {:x?}", &sim.list[gi].log[mark..]);
    // Sister: an opponent object exists → Attack on the next update [M 0x1650a0:24-27].
    sim.list[gi].ever_seen = true;
    sim.tick(&d, &far);
    sim.tick(&d, &far);
    assert!(entered(&sim, gi, state::ATTACK), "an opponent → Attack 0x56; log {:x?}", sim.list[gi].log);
}

/// `NDrone2_DSTATE_HostageDead` 0x172628: SetAsDead (the corpse, health 0), timer C 10 s → Fade 0x48
/// [M :13-35]. Sister: prop 0x53 off → no mission-fail reason.
#[test]
fn hostage_dead_is_a_corpse_that_fades_after_ten_seconds() {
    let Some(l) = LevelX::load(L05) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L05);
    let (mut sim, gi) = guard_sim(&l);
    let far = bond_far();
    enter(&mut sim, &d, gi, state::HOSTAGE_DEAD, &far);
    assert!(sim.list[gi].corpse, "SetAsDead: a corpse [M 0x172628:15]");
    assert_eq!(sim.list[gi].health, 0.0, "SetAsDead: health 0");
    assert!(sim.list[gi].fail_reason.is_none() || sim.list[gi].prop(&d, 0x53),
        "the fail reason 9 only with prop 0x53 [M 0x172628:16-19]");
    let mut faded = false;
    for _ in 0..320 { sim.tick(&d, &far); if entered(&sim, gi, state::FADE) || !sim.list[gi].present { faded = true; break; } }
    assert!(faded, "timer C (10 s) → Fade 0x48 [M 0x172628:30-33]; log {:x?}", sim.list[gi].log);
}

/// `NDrone2_DSTATE_CombatWait` 0x164960 (the port's former COMBAT_DRONE_AHEAD): enter = vision on +
/// the 3 s timer C; at the timer `DroneFunc_CombatState` decides (0 → wait again) [M :20-31, :76-86].
/// Sister: in sight, the update aims and fires (`+0x228 & 4` set) [M :43-58].
#[test]
fn combat_wait_holds_then_reconsiders_at_the_three_second_timer() {
    let Some(l) = LevelX::load(L05) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L05);
    let (mut sim, gi) = guard_sim(&l);
    let far = bond_far();
    enter(&mut sim, &d, gi, state::COMBAT_WAIT, &far);
    assert!(sim.list[gi].vision_on, "CombatWait's enter sets +0x4F8 0x10000 [M 0x164960:22]");
    let t0 = sim.list[gi].timer_c.expect("timer C armed on enter");
    assert!(t0 >= 170 && t0 <= sim.frame + 181, "timer C = now + 3 s (180 ticks) [M 0x164960:27]; got {t0} at frame {}", sim.frame);
    // Past the timer the state is reconsidered: either a new combat state, or CombatWait again with
    // a fresh 3 s timer.
    for _ in 0..200 { sim.tick(&d, &far); }
    let s = sim.list[gi].state;
    assert!(s != state::COMBAT_WAIT || sim.list[gi].timer_c.is_some_and(|t| t > t0),
        "CombatState at timer C: a new state, or CombatWait re-armed [M 0x164960:76-86]; state {s:#x}");
}

