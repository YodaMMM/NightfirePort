//! g026-guard-ai: the stand reload ENTER of `NDrone2_DSTATE_AimStandReload` 0x1666d8, the
//! `Rand_Rand(0x1e)` draw, the 1-in-30 alternate reload anim 0x4a and `Drone_AlertStatusSet(1)`
//! [M 0x1666d8:17-27]; the crouch reload 0x167370 has neither [M case 1]. Opt-in: `NF_ISO`.


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


fn bond_far() -> Bond {
    let p = [0.0, -500.0, 0.0];
    Bond { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, body: Default::default(), cel: None, obj_cel: None, move_local: [0.0; 2], riding: false, crouched: false }
}

fn bond_at(p: [f32; 3]) -> Bond {
    Bond { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, body: Default::default(), cel: None, obj_cel: None, move_local: [0.0; 2], riding: false, crouched: false }
}
fn armed(w: u32) -> bool { (2..=0x46).contains(&w) || (0x6a..=0x71).contains(&w) }

/// The anim set 9 guards carry the 0x4a script (danim 0x3f), on 01/02/03/11/12/13/4a
/// [D g026_reload_census].
const SET9: &[u32] = &[0x0700_0001, 0x0700_0002, 0x0700_0003, 0x0700_0011, 0x0700_0012, 0x0700_0013, 0x0700_004a];
const L02: u32 = 0x0700_0002;

/// The level-start sim after a short warm-up, and the first running armed guard that `pick` accepts.
fn guard_sim(l: &LevelX, code: u32, pick: impl Fn(&Drone) -> bool) -> Option<(Drones, usize)> {
    let d = l.data(code);
    let (drones, rng) = l.drones(code);
    let mut sim = Drones::new(drones, rng);
    sim.attach_level_id(&l.level, code);
    for _ in 0..5 { sim.tick(&d, &bond_far()); }
    let gi = sim.list.iter().position(|dr| dr.present && !dr.corpse && armed(dr.weapon) && !dr.holstered && pick(dr))?;
    // A channel waiter (WaitSwitch, `NDrone2_Enable(0)`) is woken the way WaitSwitch's ENABLE arm does
    // it, the reload ENTER is the unit under test, not the wake.
    sim.list[gi].enabled = true;
    Some((sim, gi))
}

/// Advance the shared stream by `n` draws (any draw steps both seeds once).
fn skip(rng: &mut Rng, n: u32) { for _ in 0..n { rng.rand(2); } }

/// One reload entry of guard `gi` from a stream advanced by `t` draws: (anim state played, alert status).
fn reload_once(l: &LevelX, code: u32, s: u8, t: u32, pick: &impl Fn(&Drone) -> bool) -> (u16, u8, u64) {
    let d = l.data(code);
    let (mut sim, gi) = guard_sim(l, code, pick).expect("a running armed guard");
    skip(&mut sim.rng, t);
    sim.list[gi].alert_status_set(0, sim.frame);
    sim.list[gi].set_state(s);
    let p = bond_far();
    let before = sim.rng.draws;
    sim.tick(&d, &p);
    assert!(sim.list[gi].log.iter().any(|&(_, x)| x == s), "the guard entered {s:#x}; log {:x?}", sim.list[gi].log);
    // `anim.call` = the CallAnim request (anim state, follow-on AI state, done message) the ENTER
    // made; the playing state switches when the request's blend starts.
    let call = sim.list[gi].anim.call.expect("the reload ENTER's CallAnim");
    assert_eq!(call.1, if s == state::AIM_STAND_RELOAD { state::AIM_STAND_FIRE } else { state::AIM_CROUCH_FIRE }, "follow-on [M 0x1666d8:23/26, 0x167370:18]");
    (call.0, sim.list[gi].alert_status, sim.rng.draws - before)
}

/// `NDrone2_DSTATE_AimStandReload` 0x1666d8 case 1 [M :17-27]: `Drone_AlertStatusSet(1)`, then
/// `Rand_Rand(0x1e) == 0 && DroneAnim_CanDoAnimState(0x4a, 0)` → CallAnim(0x4a, follow-on 0x68),
/// else CallAnim(0x52, 0x68). Over a run of stream offsets a set-9 guard plays 0x4a at least once
/// and 0x52 otherwise, always with status 1. Sister: the crouch reload 0x167370 never plays 0x4a
/// and leaves the status alone [M 0x167370 case 1].
#[test]
fn stand_reload_rolls_the_alternate_anim_one_in_thirty() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return }
    // The first set-9 level whose 0x4a guard runs from the start (07000002's wait in WaitSwitch).
    let found = SET9.iter().find_map(|&c| {
        let l = LevelX::load(c)?;
        let ok = { let d = l.data(c); guard_sim(&l, c, |dr: &Drone| dr.can_do(&d, 0x4a)).is_some() };
        ok.then_some((l, c))
    });
    let (l, code) = found.expect("a running guard that can do anim 0x4a on a set-9 level");
    let d = l.data(code);
    let pick = |dr: &Drone| dr.can_do(&d, 0x4a);
    eprintln!("level {code:08x}");
    let (mut alt, mut plain) = (0, 0);
    for t in 0..240 {
        let (a, status, _) = reload_once(&l, code, state::AIM_STAND_RELOAD, t, &pick);
        assert_eq!(status, 1, "stand reload: Drone_AlertStatusSet(1) [M 0x1666d8:17] (offset {t})");
        match a { 0x4a => alt += 1, 0x52 => plain += 1, x => panic!("stand reload played anim state {x:#x} (offset {t})") }
        if alt > 0 && plain > 20 { break; }
    }
    assert!(alt > 0, "Rand_Rand(0x1e) == 0 → the alternate reload anim 0x4a [M 0x1666d8:21-24]; 0x52 x{plain}, 0x4a never");
    assert!(plain > alt, "the alternate is the 1-in-30 case: 0x4a x{alt} vs 0x52 x{plain}");
    for t in 0..60 {
        let (a, status, _) = reload_once(&l, code, state::AIM_CROUCH_RELOAD, t, &pick);
        assert_eq!(a, 0x52, "crouch reload: always 0x52 [M 0x167370 case 1] (offset {t})");
        assert_eq!(status, 0, "crouch reload writes no alert status [M 0x167370 case 1] (offset {t})");
    }
}

/// The draw is taken before the CanDoAnimState test [M 0x1666d8:21-22, `Rand_Rand` first, the
/// `&&` short-circuits on its result], so a guard WITHOUT the 0x4a script still moves the shared
/// stream by exactly one more draw than the crouch reload of the same guard from the same stream.
#[test]
fn stand_reload_draws_once_even_without_the_alternate_anim() {
    let Some(l) = LevelX::load(L02) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data(L02);
    let pick = |dr: &Drone| !dr.can_do(&d, 0x4a);
    if guard_sim(&l, L02, pick).is_none() { eprintln!("skip: no running guard without 0x4a on 07000002"); return; }
    for t in 0..8 {
        let (a, _, stand) = reload_once(&l, L02, state::AIM_STAND_RELOAD, t, &pick);
        let (_, _, crouch) = reload_once(&l, L02, state::AIM_CROUCH_RELOAD, t, &pick);
        assert_eq!(a, 0x52, "no 0x4a script → 0x52 (offset {t})");
        assert_eq!(stand, crouch + 1, "the stand reload's Rand_Rand(0x1e) is the one extra draw (offset {t})");
    }
}

/// Probe: the type-0xC drone of 07000008 (skin 0x0500003b, the escort VIP), the only `+0xD8 == 0xc`
/// placement on any level [D g026_reload_census], and the gates of CombatNoMove 0x1643e8's
/// Rand(3) → CrouchCover roll [M :103-111]. Assertion-free.
#[test]
#[ignore = "probe dump; run with --ignored --nocapture"]
fn type_c_drone_probe() {
    let Some(l) = LevelX::load(0x0700_0008) else { return };
    let d = l.data(0x0700_0008);
    let (drones, _) = l.drones(0x0700_0008);
    for dr in drones.iter().filter(|dr| dr.kind == 0xc) {
        eprintln!("kind {:#x} mode {:#x} weapon {:#x} start {:#x} stationary {} prop40 {} team {} armed {}", dr.kind, dr.mode, dr.weapon, dr.start_state(&d), dr.stationary, dr.prop(&d, 0x40), dr.team, armed(dr.weapon));
    }
}
