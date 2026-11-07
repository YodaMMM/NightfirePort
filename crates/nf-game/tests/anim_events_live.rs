//! The anim-script event kinds on real disc scripts (`DroneAnim_EventFunc` 0x13b750,
//! `DroneWeap_CloseCombatImpact` 0x1789c0, `DroneAnim_PlayFiringAnim` 0x13da48) — G-030.
//! Each test plays a real kind-2 command through `update_anim` → `script_commands` → `event_func`
//! (the census: fix-anim-events, /tmp census of all 18,231 disc scripts) and asserts the effect.
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{state, Data, Drone, Drones, Player, Rng};
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

fn bond_away() -> Player { Player { yaw: 0.0, pos: [0.0, -500.0, 0.0], target: [0.0, -499.5, 0.0], eye: [0.0, -499.0, 0.0], aim: [-1.0, 0.0, 0.0], sph: [0.0, -500.0, 0.0], capsule: ([0.0, -498.9, 0.0], [0.0, -500.0, 0.0]), hanging: false, body: Default::default(), cel: None, move_local: [0.0; 2], riding: false, crouched: false } }

/// The frame of the first op-4 kind-2 command with the given event arg in `hash`.
fn event_frame(d: &Data, hash: u32, arg: u16) -> Option<(u32, u32)> {
    let s = d.scripts.get(&hash)?;
    s.commands.iter().find_map(|c| match (c.op, c.args.as_slice()) {
        (4, [f, 2, a, ..]) if *a == arg => Some((*f as u32, s.length as u32)),
        _ => None,
    })
}

/// Points the drone's playing script at `hash` so the next tick's command window covers `frame`.
/// Frame 1 commands are the script's start-frame commands (`AnimScriptTick` runs them on the first
/// tick — `AnimProcessScriptCmds`' first window is (0, 1]), so `started` is cleared for those.
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

/// The melee close-combat impact (events 5 / 10 / 0xB): the guard at 07000005's script 060002ba
/// (the census's kind-2 arg-10 example) fires a `Bullet_init` punch bullet from the weapon record
/// 0x65 when Bond stands in range and facing, and nothing outside it.
#[test]
fn melee_event_creates_the_close_combat_bullet() {
    let Some(l) = Fixture::load("07000005.bin", 0x0700_0005) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let Some((frame, len)) = event_frame(&d, 0x0600_02ba, 10) else { panic!("060002ba carries no arg-10 event") };
    assert!(frame + 1 < len, "event frame {frame} inside the {}-frame script", len);
    for dist in [1.5f32, 3.0] {
        let mut sim = l.sim(l.level_drones());
        sim.tick(&d, &bond_away()); // the placements' ENTER
        let gi = sim.list.iter().position(|dr| dr.health > 0.0 && !dr.corpse).unwrap();
        let gpos = sim.list[gi].pos;
        let g = &mut sim.list[gi];
        // A state whose UPDATE touches nothing (Bond away → CheckUnsurrender false), so the forced
        // script below survives to `update_anim`.
        g.state = state::SURRENDERED;
        g.health = 100.0;
        g.weapon = 1; // not the kick class 7 → the punch records
        g.yaw = 0.0; // atan2(0, +z) = 0 — facing Bond
        play_frame(g, 0x0600_02ba, frame);
        // Bond dist north/south of the guard: 1.5 (in range) / 3.0 (out of the 1.75 punch range).
        let mut player = bond_away();
        player.pos = [gpos[0], gpos[1], gpos[2] + dist];
        player.target = [player.pos[0], player.pos[1] + 0.5, player.pos[2]];
        player.eye = [player.pos[0], player.pos[1] + 1.0, player.pos[2]];
        player.sph = player.pos;
        player.capsule = ([player.pos[0], player.pos[1] + 1.086, player.pos[2]], player.pos);
        sim.tick(&d, &player);
        // The punch bullet flies and connects within its flight window: the record's speed is
        // 1.5/tick over the ≤1.75 gap, so the hit lands on the swing tick or the next one — under
        // the post-f1f37ba47 stream the first flight tick clamps at the capsule contact
        // (travelled 0.9467 of the 1.5) and the hit lands the tick after (the old same-tick pin
        // held one stream position of the contact path). The damage/owner/range pins stay exact.
        if dist < 1.75 {
            for _ in 0..2 {
                if sim.player_hits.len() == 1 { break; }
                sim.tick(&d, &player);
            }
            assert_eq!(sim.player_hits.len(), 1, "the punch connects in range (dist {dist})");
            assert_eq!(sim.player_hits[0].1, gi, "the hit is the melee guard's bullet");
        } else {
            sim.tick(&d, &player);
            assert!(sim.player_hits.is_empty(), "out of the 1.75 punch range (dist {dist}): {:?}", sim.player_hits);
        }
    }
}

/// The weapon-change event (datum 2): script 06000330 in 07000001 (the census's kind-2 arg-2
/// example) requests `DroneWeap_ChangeWeapon(drone+0x45)`; entry 6 = weapon 6, anim set 4,
/// 16 rounds (DrawWeapon 0x166000's EXIT). An unported table entry changes nothing.
#[test]
fn weapon_change_event_swaps_by_the_setup_byte() {
    let Some(l) = Fixture::load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let Some((frame, _)) = event_frame(&d, 0x0600_0330, 2) else { panic!("06000330 carries no arg-2 event") };
    for (swap, want) in [(6u8, Some(6u32)), (3, None)] {
        let mut sim = l.sim(l.level_drones());
        sim.tick(&d, &bond_away());
        let gi = sim.list.iter().position(|dr| dr.health > 0.0 && !dr.corpse).unwrap();
        {
            let g = &mut sim.list[gi];
            g.state = state::SURRENDERED;
            g.health = 100.0;
            g.weapon = 1;
            g.weapon_swap = swap;
            play_frame(g, 0x0600_0330, frame);
        }
        sim.tick(&d, &bond_away());
        let g = &sim.list[gi];
        match want {
            Some(w) => {
                assert_eq!(g.weapon, w, "ChangeWeapon({swap})");
                assert_eq!((g.anim_set, g.clip), (4, 16), "weapon 6's anim set and rounds");
            }
            None => assert_eq!(g.weapon, 1, "table entry {swap} is not ported — no change"),
        }
    }
}

/// The fire-flag events (0xF / 0x10 / 0x12 — 86% of the disc's kind-2 traffic): script 0600029f in
/// 07000005 carries the 0x12 (set `+0x20`/`+0x21`). `DroneAnim_PlayFiringAnim` (0x13da48) fires the
/// recoil weapon (`+0x3E`, the datum's weapon byte 0x1C/0x1D) only on `+0x21`, and the shot plays
/// the 0x5C recoil danim; a non-recoil weapon fires without the flag (the game's early-out).
#[test]
fn fire_flag_gates_the_recoil_weapon_and_plays_the_recoil() {
    let Some(l) = Fixture::load("07000005.bin", 0x0700_0005) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let Some((frame, _)) = event_frame(&d, 0x0600_029f, 18) else { panic!("0600029f carries no arg-18 (0x12) event") };
    // Through the real script: the event sets the flags.
    let mut sim = l.sim(l.level_drones());
    sim.tick(&d, &bond_away());
    let gi = sim.list.iter().position(|dr| dr.health > 0.0 && !dr.corpse).unwrap();
    {
        let g = &mut sim.list[gi];
        g.state = state::SURRENDERED;
        g.health = 100.0;
        g.event_20 = false;
        g.event_21 = false;
        play_frame(g, 0x0600_029f, frame);
    }
    sim.tick(&d, &bond_away());
    assert!(sim.list[gi].event_21 && sim.list[gi].event_20, "event 0x12 sets both flags");
}

/// A spot 5 u from the guard in the open — the accuracy harness's placement conditions
/// (`enemy_accuracy_live::measure`) condensed to the fixture's world: the same floor as the
/// guard, inside the guard's view cone (the visibility cone gates the LOS round robin
/// [M DroneVision_OpponentVisibility]), a clear corridor to him at three side offsets and
/// three heights, and no wall within 1.5 u of the spot at waist height in 8 directions.
fn open_spot_5u(world: &nf_game::world::CollisionWorld, gpos: [f32; 3], yaw: f32) -> Option<[f32; 3]> {
    let cone_ok = |p: [f32; 3]| {
        let a = (p[0] - gpos[0]).atan2(p[2] - gpos[2]);
        let mut d = a - yaw;
        while d > std::f32::consts::PI { d -= std::f32::consts::TAU; }
        while d < -std::f32::consts::PI { d += std::f32::consts::TAU; }
        d.abs() < 1.0
    };
    for k in 0..64 {
        let a = k as f32 * std::f32::consts::TAU / 64.0;
        let (s, c) = (a.sin(), a.cos());
        let p = [gpos[0] + 5.0 * s, gpos[1], gpos[2] + 5.0 * c];
        let Some(f) = world.floor_below([p[0], p[1] + 0.5, p[2]], 3.0) else { continue };
        if (f - (gpos[1] - 1.0)).abs() > 0.3 { continue; }
        let spot = [p[0], f + 1.0, p[2]];
        if !cone_ok(spot) { continue; }
        let side = [c, 0.0, -s];
        let clear = [-0.5f32, 0.0, 0.5].iter().all(|&o| [0.4f32, 0.8].iter().all(|&h| [0.3f32, 1.0, 1.8].iter().all(|&bh| {
            world.segment([gpos[0] + side[0] * o, gpos[1] + h, gpos[2] + side[2] * o],
                          [p[0] + side[0] * o * 0.5, f + bh, p[2] + side[2] * o * 0.5]).is_none()
        })));
        if !clear { continue; }
        let open = (0..8).all(|j| { let b = j as f32 * std::f32::consts::FRAC_PI_4;
            world.segment([p[0], f + 1.0, p[2]], [p[0] + 1.5 * b.sin(), f + 1.0, p[2] + 1.5 * b.cos()]).is_none() });
        if !open { continue; }
        return Some(spot);
    }
    None
}

/// Bond standing at `spot`, facing the guard (the fixture's Player shape, as the melee test builds it).
fn player_at(spot: [f32; 3], gpos: [f32; 3]) -> Player {
    let (dx, dz) = (gpos[0] - spot[0], gpos[2] - spot[2]);
    let yaw = dx.atan2(dz);
    let mut player = bond_away();
    player.yaw = yaw;
    player.pos = spot;
    player.target = [spot[0], spot[1] + 0.55, spot[2]]; // the aim point: 0.45 under the eye [M Drone_View_Bones]
    player.eye = [spot[0], spot[1] + 1.0, spot[2]];
    player.aim = [dx / 5.0, 0.0, dz / 5.0];
    player.sph = spot;
    player.capsule = ([spot[0], spot[1] + 1.086, spot[2]], spot);
    player
}

/// Drives one placement guard (by the `+0x3E` recoil class) through the real chain — perception,
/// the combat state machine, the anim scripts, the weapon pass — with nothing forced: Bond parked
/// 5 u away in the open, the other drones disabled (the accuracy harness's one-subject isolation),
/// ticks until the guard's first shot (the clip drops). Returns (weapon, a due-no-event tick was
/// seen, Some((danim, reached)) at the first shot).
///
/// The due shot is armed by the states themselves: the combat chain
/// (`CombatNewSighting` 0x173280 → `AimStandFire` 0x68) ENTER requests the fire
/// (`DroneWeap_Fire` 0x17a8c0:12-19: `+0x3B = 1`, `NextBulletTime` when not already firing), and
/// `DroneWeap_DoFiring` 0x17a080:24-29 sets the due latch `+0x3F` whenever `+0xBC4 <= now` and
/// `Ready2Fire` — for a recoil weapon (`+0x3E`) it then HOLDS (no `FireWeapon`, no
/// `NextBulletTime`) until `PlayFiringAnim` 0x13da48 sees the anim's fire event `+0x21`
/// (EventFunc 0x12 [M 0x13b750:93]) and fires with the 0x5C recoil danim [M 0x13da48:20-29].
fn drive_first_shot(l: &Fixture, d: &Data, recoil: bool) -> (u32, bool, Option<(u16, bool, bool)>) {
    let matches = |dr: &Drone| dr.recoil_weapon == recoil && dr.team == 1
        && dr.health > 0.0 && !dr.corpse && (2..=0x46).contains(&dr.weapon);
    let cands: Vec<usize> = l.level_drones().iter().enumerate().filter(|(_, dr)| matches(dr)).map(|(i, _)| i).collect();
    assert!(!cands.is_empty(), "no {} guard armed from the 07000001 placements", if recoil { "recoil (0x1C/0x1D)" } else { "non-recoil" });
    let trace = std::env::var_os("NF_REC_TRACE").is_some();
    // The accuracy harness's candidate walk: the first placements are often cinematic-held
    // (scripted to stand down until the intro releases), so drive each candidate in turn until
    // one reaches a fire state and shoots.
    for &gi in &cands {
        let mut sim = l.sim(l.level_drones());
        sim.tick(&d, &bond_away()); // the placements' ENTER
        for (i, dr) in sim.list.iter_mut().enumerate() { if i != gi { dr.present = false; dr.enabled = false; } }
        // The subject itself: the accuracy harness's activation (`d.enabled = true; d.present =
        // true`) — placements that the mission layer spawns later carry present/enabled off, and
        // the sight pass computes nothing for them.
        sim.list[gi].present = true;
        sim.list[gi].enabled = true;
        // The placements' guards start WAIT_SWITCHed — the state's ENTER is `NDrone2_Enable(0)`
        // [M 0x15b960:14-16] — until the level's mission script enables them, and the fixture has
        // no mission layer. The game's enable hands the guard a sight-capable state; the fixture
        // starts the subject in Idle (0x4): its ENTER opens vision (`+0x4F8 |= 0x10000` [M
        // 0x15bf18:30], the port's `(IDLE|ALERT|PATROL, msg::ENTER)`) and its UPDATE runs the
        // sight scan [M 0x15bf18 case 3]. `set_state` queues the real EXIT→ENTER transition
        // (a raw `.state =` write would skip the ENTER and the vision bit with it). Every tick
        // after this is the real chain.
        if sim.list[gi].state == state::WAIT_SWITCH { sim.list[gi].set_state(state::IDLE); }
        let gpos = sim.list[gi].pos;
        let weapon = sim.list[gi].weapon;
        let yaw = sim.list[gi].yaw;
        let Some(spot) = open_spot_5u(&l.world, gpos, yaw) else { continue };
        let player = player_at(spot, gpos);
        let clip0 = sim.list[gi].clip;
        let mut held = false;
        let mut shot = None;
        let mut prev_clip = clip0;
        let trace = std::env::var_os("NF_REC_TRACE").is_some();
        for step in 0..2400 {
            sim.tick(&d, &player);
            let (frame, dr) = (sim.frame, &sim.list[gi]);
            if trace && (step % 120 == 0 || dr.clip != prev_clip) {
                eprintln!("[rec] g{gi} s{step} f{frame} st {:#x} fire {} burst {} next {} clip {} ev21 {} danim {:#x} reached {}",
                    dr.state, dr.firing, dr.burst, dr.next_shot, dr.clip, dr.event_21, dr.anim.danim, dr.anim.reached);
            }
            // A due shot (`+0xBC4 <= now`, the port's `next_shot <= frame`) with the burst armed
            // and no shot this tick, while `+0x21` is clear: `PlayFiringAnim`'s gate is what
            // holds it (before the first shot and between the burst's rounds — the recoil anim's
            // 0x10 event clears `+0x21` [M 0x13b750:94-95] and its 0x12 re-opens it).
            if dr.firing && dr.burst > 0 && dr.next_shot <= frame && !dr.event_21 && dr.clip == prev_clip {
                held = true;
            }
            if dr.clip < prev_clip && shot.is_none() {
                shot = Some((dr.anim.danim, dr.anim.reached, dr.event_21));
            }
            prev_clip = dr.clip;
            if shot.is_some() && held { break; }
        }
        if shot.is_some() {
            return (weapon, held, shot);
        }
        if trace { eprintln!("[rec] g{gi} (weapon {weapon:#x}) never fired — next candidate"); }
    }
    panic!("no {} guard fired within the window (candidates {cands:?})", if recoil { "recoil" } else { "non-recoil" });
}

/// `DroneAnim_PlayFiringAnim` (0x13da48) through the real chain, all three rows from the level
/// data (the only setup choice is the subject and its start state — every shot field comes from
/// the machine; the old harness's `SURRENDER_ANIM` isolation never reached the fire block at
/// all, because a raw `.state =` write bypasses the ENTER and the weapon pass saw nothing due):
/// - recoil (`+0x3E`, 07000001's weapon-0x1C shotgun guards [D weapon_data; M DefaultInit
///   0x14b300:800-811]) with a due shot and no event: held (`DoFiring` 0x17a080:29 sets `+0x3F`,
///   skips `FireWeapon` for `+0x3E != 0`) — before the first shot and between the burst's
///   rounds (the recoil anim's 0x10 clears `+0x21` [M 0x13b750:94-95], its 0x12 re-opens);
/// - the event (`+0x21`, the fire anim's 0x12 [M 0x13b750:93]): the shot fires and plays the
///   0x5C recoil danim with `+0x57B` [M 0x13da48:20-29];
/// - non-recoil (the weapon-0x0E pistol guard): the `+0x3E == 0` early-out fires without any
///   event and without the recoil switch [M 0x13da48:10-12].
#[test]
fn play_firing_anim_gates_the_recoil_weapon_shot() {
    let Some(l) = Fixture::load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();

    let (weapon, held, shot) = drive_first_shot(&l, &d, true);
    assert!(shot.is_some(), "the recoil (weapon {weapon:#x}) guard never fired within the window");
    let (danim, reached, ev21) = shot.unwrap();
    assert!(held, "the recoil shot never stood due without the event (+0x21) — the gate is not holding it");
    assert!(ev21, "the recoil shot fired without the anim event (+0x21) [M 0x13da48:17-20]");
    assert_eq!(danim, 0x5c, "the recoil shot plays the 0x5C recoil danim [M 0x13da48:23]");
    assert!(reached, "+0x57B set by PlayFiringAnim [M 0x13da48:24-26]");

    let (weapon, _, shot) = drive_first_shot(&l, &d, false);
    assert!(shot.is_some(), "the non-recoil (weapon {weapon:#x}) guard never fired within the window");
    let (danim, _, _) = shot.unwrap();
    assert_ne!(danim, 0x5c, "the non-recoil shot keeps the playing danim — no recoil switch [M 0x13da48:10-12]");
}

/// The tertiary-placement event (datum 9): script 060007ac in 07000012 (the census's kind-2 arg-9
/// example). `DroneWeap_PlaceTertiary` (0x17a848) is a tail call to `DroneWeap_ThrowGrenade`, so the
/// event spawns the grenade projectile and spends one.
#[test]
fn tertiary_event_throws_the_grenade() {
    let Some(l) = Fixture::load("07000012.bin", 0x0700_0012) else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let Some((frame, _)) = event_frame(&d, 0x0600_07ac, 9) else { panic!("060007ac carries no arg-9 event") };
    let mut sim = l.sim(l.level_drones());
    sim.tick(&d, &bond_away());
    let gi = sim.list.iter().position(|dr| dr.health > 0.0 && !dr.corpse).unwrap();
    {
        let g = &mut sim.list[gi];
        g.state = state::SURRENDERED;
        g.health = 100.0;
        g.grenade_weapon = 0x36;
        g.grenade_count = 2;
        g.mode = 0; // no sniper stun-roll draw
        play_frame(g, 0x0600_07ac, frame);
    }
    sim.tick(&d, &bond_away());
    assert_eq!(sim.grenades.len(), 1, "PlaceTertiary = ThrowGrenade");
    assert_eq!(sim.list[gi].grenade_count, 1, "the throw spent one grenade");
    assert_eq!(sim.grenades[0].owner, gi);
}
