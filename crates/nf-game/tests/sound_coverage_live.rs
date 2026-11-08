//! sound-coverage ports (audit-sound-calls): each MISSING sound the task ported reaches the
//! Sim's sound pool (`Drones.audio`, the `DYNAMICSOUNDS` node pool) at the game's moment, through
//! the real input path. Opt-in: set `NF_ISO` to the disc image. Tests stay silent (NF_SILENT
//! semantics: the pool's `out` commands go nowhere unless the driver drains them).
//!
//! Ports and their game sites:
//! - the grenade-ammo launch whoosh 0x10E [M Bullet_init 0x12c5a0:285-291] via `Drones::fire_player`;
//! - the jump launch 0xA3 [M Player_HandleJump 0x1916b0:184] through `Bond::tick_ctx` (the
//!   Sim's `step_bond` drain lives in nf-view scene.rs; here the outbox itself is asserted);
//! - the laser burn-through 0x249 [M Lock_Update 0x1ae3d8:90-93] through the watch-bullet pass
//!   (0x5DD is the satchel burn-out's, not the lock's: `Bullet_DoTrails` 0x12d890:472-474 is its
//!   only caller — pinned by nf-view's satchel_laser_4a_live);
//! - the special-death cry 0x148 [M NDrone2_DSTATE_SpecialDeath_Anim 0x163c68 case 5].

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drone, Drones, Player, Rng, state};
use nf_game::player::{Bond, Input, LadderCtx};
use std::collections::HashMap;

struct Level02 {
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
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: HashMap<u32, nf_game::pickup::Model>,
    hud: nf_formats::hud::HudTables,
    surfaces: Option<nf_formats::sfx::SurfaceSounds>,
    voices: Vec<nf_formats::sfx::VoiceLine>,
    skins: HashMap<u32, nf_formats::character::Skin>,
}

impl Level02 {
    fn load() -> Option<Level02> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let read = |name: &str| -> Option<Vec<u8>> {
            let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
            Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
        };
        let data = read("07000002.bin")?;
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let skeletons: HashMap<u8, _> = lb.entries.iter().filter(|e| e.kind == 6)
            .filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok().map(|s| (s.id as u8, s))).collect();
        let mut sequences = HashMap::new();
        if let Some(anim) = read("07F00002.bin") {
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
        let nav = nf_game::nav::Network::build(level.maps[level.main_map().unwrap()].ai_network.as_ref().unwrap(), &world, 0x0700_0002);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let hitboxes = nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap();
        let volumes = nf_game::aivolume::build(&level, 0x0700_0002);
        let models = nf_game::pickup::models(&level);
        let sfx = nf_formats::sfx::SfxTable::from_action_elf(&elf).ok();
        let tuning = nf_formats::tuning::Tuning::default();
        let hud = nf_formats::hud::HudTables::from_action_elf(&elf).unwrap();
        let skins = level.skins.clone();
        Some(Level02 { tables, props, weapons, level, scripts, world, sequences, skeletons, nav, covers, sfx, tuning, hitboxes, volumes, models, hud, surfaces: None, voices: Vec::new(), skins })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_0002, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: self.sfx.as_ref(), models: &self.models, surfaces: self.surfaces.as_ref(), voices: None, difficulty: 1,
            ai_points: &[],
        }
    }

    fn sim(&self) -> Drones {
        let d = self.data();
        let mut rng = Rng::new(0x1234_5678);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        let mut s = Drones::new(drones, rng);
        s.attach_level_id(&self.level, 0x0700_0002);
        s
    }
}

fn bond_at(p: [f32; 3]) -> Player { Player { pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], ..Default::default() } }

/// The first weapon whose ammo type is 0x0e (the grenade pool — [L] docs/fields/ammo-data.md
/// rec 14 ← weapons 0x2c..0x31, 0x6d) and one whose is not, from the live weapon_data.
fn grenade_and_normal_weapon(wd: &WeaponData) -> (Option<u8>, u8) {
    let mut g = None;
    for w in 0..0x73u8 {
        if wd.records.get(w as usize).map(|r| r[0x90]) == Some(0x0e) { g = Some(w); break; }
    }
    (g, 0x02) // 0x02 = the P2K (ammo type 2 per [L] rec 2 ← weapons 6-13,18-21)
}

/// [M Bullet_init 0x12c5a0:285-291]: firing a grenade-ammo weapon plays the launch whoosh 0x10E
/// at the shooter; a normal-ammo weapon does not. Through `Drones::fire_player` — the port's
/// `Player_WeaponInitBullet` path.
#[test]
fn grenade_ammo_launch_plays_0x10e() {
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (gw, nw) = grenade_and_normal_weapon(&d.weapons);
    let Some(gw) = gw else { eprintln!("skip: no grenade-ammo weapon in weapon_data"); return };
    let mut sim = l.sim();
    let from = [10.0, 2.0, 10.0];
    let hearing = nf_formats::sfx::SfxHearing { id: 0x403, inner: 0.0, outer: 0.0, weight: 0.0, looped: false, cull: false };
    sim.fire_player(&d, from, [-1.0, 0.0, 0.0], gw, hearing);
    assert!(sim.audio.active_ids().contains(&0x10e),
        "the grenade-ammo launch ({gw:#x}) plays the 0x10e whoosh");

    // Sister: a normal-ammo weapon fires without the whoosh.
    let mut sim2 = l.sim();
    sim2.fire_player(&d, from, [-1.0, 0.0, 0.0], nw, hearing);
    assert!(!sim2.audio.active_ids().contains(&0x10e), "a normal-ammo weapon ({nw:#x}) plays no 0x10e");
}

/// [M Player_HandleJump 0x1916b0:184]: the jump press (action 7) launches with 0xA3 at the
/// player — through the real physics tick (`Bond::tick_ctx`, the path `Sim::step_bond` drives).
#[test]
fn jump_launch_plays_0xa3() {
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let world = &l.world;
    // A flat outdoor spot on 07000002 (the courtyard spawn); Bond stands, then jumps.
    let mut bond = Bond::new([226.0, 2.0, 312.0], 0.0);
    let ctx = LadderCtx::default();
    let input = Input { jump: true, ..Default::default() };
    bond.tick_ctx(world, &ctx, &input);
    assert!(bond.sfx_out.iter().any(|&(id, _)| id == 0xa3),
        "the jump launch queues the 0xa3 thud (sfx_out: {:?})", bond.sfx_out);

    // Sister: a tick with no jump press queues nothing.
    let mut bond2 = Bond::new([226.0, 2.0, 312.0], 0.0);
    let idle = Input::default();
    bond2.tick_ctx(world, &ctx, &idle);
    assert!(bond2.sfx_out.is_empty(), "an idle tick queues no jump sound");
}

/// [M Lock_Update 0x1ae3d8:90-93]: a laser-watch bullet (0x4F) completing the burn plays 0x249
/// (the release clunk) at the lock — through the port's watch-bullet pass in `Drones::tick`.
/// 0x5DD is NOT a lock sound: its only caller is `Bullet_DoTrails`' 0x3b satchel tail [M
/// 0x12d890:472-474], which a lock never runs.
#[test]
fn lock_burn_through_plays_0x249_not_0x5dd() {
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let mut sim = l.sim();
    // A lock on the bullet's path: the watch shot flies -x from Bond's head.
    let lock_pos = [10.0, 1.5, 10.0];
    sim.locks.push(nf_game::lock::Lock { placement: 0, pos: lock_pos, radius: 1.0, out: 0, key_channels: [0xffff; 2], burn: 0, burned: false, unlock_on_operative: false, unlocked: false, kind: 1, code: 0, scan_progress: 0, scan_solved: 0, scanning: false, burn_a: 0xff, burn_b: 0xff, grace: 0, lod_next: false, pending_sounds: Vec::new(), facing: [0.0, 0.0, 1.0], open_pending: false, equip_pending: false });
    let hearing = nf_formats::sfx::SfxHearing { id: 0x403, inner: 0.0, outer: 0.0, weight: 0.0, looped: false, cull: false };
    // The real burn: the 0x4F takes 10/tick off the 0xff bytes and completes at +0x105 < 4 — 26
    // contact ticks [M Lock_Update 0x1ae3d8:66,82-101; the bytes start 0xff, control_init_object
    // 0x134630]. The old instant-burn pin (one shot) contradicted the decompile — fixed here.
    let bond = bond_at([14.0, 1.0, 10.0]);
    for _ in 0..30 {
        sim.fire_player(&d, [14.0, 1.5, 10.0], [-1.0, 0.0, 0.0], 0x4f, hearing);
        sim.tick(&d, &bond);
        if sim.locks[0].burned { break }
    }
    assert!(sim.locks[0].burned, "the watch bullet stream burns the lock (26 contact ticks [M 0x1ae3d8:82-101])");
    assert!(sim.audio.active_ids().contains(&0x249), "the burn-through clunk 0x249 plays");
    assert!(!sim.audio.active_ids().contains(&0x5dd), "0x5dd is the satchel burn-out's sound, not the lock's [M 0x12d890:472-474]");
    // Sister: a fresh bullet into the already-burned lock plays neither again.
    sim.fire_player(&d, [14.0, 1.5, 10.0], [-1.0, 0.0, 0.0], 0x4f, hearing);
    sim.tick(&d, &bond);
    let n_249 = sim.audio.active_ids().iter().filter(|&&i| i == 0x249).count();
    assert_eq!(n_249, 1, "a burned lock does not re-burn (one 0x249 node)");
}

/// [M NDrone2_DSTATE_SpecialDeath_Anim 0x163c68 case 5, disasm 0x163f38-0x163f80]: the cry 0x148
/// rides the death script's COMPLETION (case 5 = the ANIM_DONE message), plays at the PLAYER's
/// position (`glb_players+0x30`), gated on the 0x0600021F script and `glb_players != 0`. The
/// state's ENTER queues nothing (sound-gates-sweep: it used to fire the cry on ENTER, at the
/// drone — right id, wrong event).
#[test]
fn special_death_0x0600021f_plays_0x148() {
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let mut sim = l.sim();
    let gi = sim.list.iter().position(|x| x.present && !x.corpse)
        .expect("07000002 has a guard");
    {
        let dr = &mut sim.list[gi];
        dr.special_death = 0x0600_021f;
        dr.special_spot = dr.pos;
        dr.special_yaw = dr.yaw;
        dr.dist = 5.0;
        dr.state = state::SPECIAL_DEATH;
        // Deliver the machine's msg::ENTER directly (send is the state machine's public entry;
        // a plain next_state race loses to the live state's UPDATE arm re-setting it).
        let mut rng = Rng::new(0x0bad_f00d);
        dr.send(&d, &mut rng, 100, nf_game::drone::msg::ENTER);
    }
    assert!(sim.list[gi].sfx_reqs.is_empty(), "the ENTER queues nothing — the cry is the ANIM_DONE event");
    let bond = bond_at([sim.list[gi].pos[0] + 5.0, sim.list[gi].pos[1], sim.list[gi].pos[2]]);
    {
        let dr = &mut sim.list[gi];
        dr.player_pos = bond.pos;
        let mut rng = Rng::new(0x0bad_f00d);
        // The completion message, delivered the same way the machine delivers it when the death
        // script ends (play_script's msg::ANIM_DONE tail).
        dr.send(&d, &mut rng, 140, nf_game::drone::msg::ANIM_DONE);
    }
    let queued = sim.list[gi].sfx_reqs.clone();
    assert!(queued.iter().any(|&(id, p)| id == 0x148 && p == bond.pos),
        "the completion queues 0x148 at the PLAYER's position, got {queued:?}");
    sim.tick(&d, &bond);
    assert!(sim.audio.active_ids().contains(&0x148),
        "the scripted special-death cry 0x148 plays at the player");

    // Sister: the game's gate — a special death whose script is NOT 0x0600021F gets no cry
    // ([M 0x163c68 case 5's `+0xbd4 == 0x600021f` check]).
    let mut sim2 = l.sim();
    let gj = sim2.list.iter().position(|x| x.present && !x.corpse).expect("a guard");
    {
        let dr = &mut sim2.list[gj];
        dr.special_death = 0x0600_0020; // another script id
        dr.special_spot = dr.pos;
        dr.special_yaw = dr.yaw;
        dr.dist = 5.0;
        dr.state = state::SPECIAL_DEATH;
        let mut rng = Rng::new(0x0bad_f00d);
        dr.send(&d, &mut rng, 100, nf_game::drone::msg::ENTER);
        dr.send(&d, &mut rng, 140, nf_game::drone::msg::ANIM_DONE);
    }
    assert!(sim2.list[gj].sfx_reqs.is_empty(), "no cry for other special-death scripts");
}

/// [M Player_WeaponInitBullet 0x1a4bf0:261-263]: a weapon record whose base id (+2 u16) is 0x54
/// — the Micro-Camera pair 0x54/0x55 [L weapon_data] — plays the 2-D shutter 0xEC per bullet.
/// Sister: the P2K 0x02 (base 0x02) fires without it.
#[test]
fn micro_camera_shot_plays_the_0xec_shutter() {
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let base = |w: usize| u16::from_le_bytes([d.weapons.records[w][2], d.weapons.records[w][3]]);
    assert_eq!((base(0x54), base(0x55)), (0x54, 0x54), "both Micro-Camera records carry base 0x54 [L weapon_data +2]");
    let hearing = nf_formats::sfx::SfxHearing { id: 0xffff, inner: 0.0, outer: 0.0, weight: 0.0, looped: false, cull: false };
    for w in [0x54u8, 0x55] {
        let mut sim = l.sim();
        sim.fire_player(&d, [10.0, 2.0, 10.0], [-1.0, 0.0, 0.0], w, hearing);
        let shutters = sim.audio.active_ids().iter().filter(|&&id| id == 0xec).count();
        let pellets = u32::from_le_bytes(d.weapons.records[w as usize][0x18..0x1c].try_into().unwrap()).clamp(1, 16) as usize;
        assert_eq!(shutters, pellets, "{w:#x}: one 0xEC per bullet");
        sim.audio.update(None, ([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), &|_| true);
        let starts: Vec<_> = sim.audio.take_commands().into_iter().filter_map(|c| match c {
            nf_game::sound::Cmd::Start { id: 0xec, pos, .. } => Some(pos),
            _ => None,
        }).collect();
        assert!(!starts.is_empty() && starts.iter().all(|p| p.is_none()), "{w:#x}: the shutter is 2-D (`Sound_Play`)");
    }
    let mut sim = l.sim();
    sim.fire_player(&d, [10.0, 2.0, 10.0], [-1.0, 0.0, 0.0], 0x02, hearing);
    assert!(!sim.audio.active_ids().contains(&0xec), "sister: the P2K plays no shutter");
}
