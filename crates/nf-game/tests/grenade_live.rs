//! Grenade tests on level 07000002 (the throw mechanics; the guards get their grenades set the way
//! `NDrone2_DefaultInit` decodes placement param 11 — drone-port-check.md #1 fixed the param index; 07000002's
//! own placements carry none). Opt-in: set `NF_ISO` to the disc image. See docs/systems/levels.md
//! "Grenade-carrying guards".

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drone, Drones, Player, Rng};
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
        let anim_name = "07F00002.bin";
        let mut sequences = HashMap::new();
        if let Some(anim) = read(anim_name) {
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
        let surfaces = None;
        let voices = Vec::new();
        let skins = level.skins.clone();
        Some(Level02 { tables, props, weapons, level, scripts, world, sequences, skeletons, nav, covers, sfx, tuning, hitboxes, volumes, models, hud, surfaces, voices, skins })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_0002, sequences: &self.sequences, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: self.sfx.as_ref(), models: &self.models, surfaces: self.surfaces.as_ref(), voices: None, difficulty: 1,
            ai_points: &[],
        }
    }

    fn drones(&self) -> (Vec<Drone>, Rng) {
        let d = self.data();
        let mut rng = Rng::new(0x1234_5678);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights);
        (drones, rng)
    }

    fn sim(&self, drones: Vec<Drone>, rng: Rng) -> Drones { let mut s = Drones::new(drones, rng); s.attach_level_id(&self.level, 0x0700_0002); s }
}

fn bond_at(p: [f32; 3]) -> Player { Player { pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], ..Default::default() } }

/// On 07000002 (whose own placements grant no grenades — param 11 = 0 everywhere), a guard given the grenade
/// fields the way `NDrone2_DefaultInit` decodes them (raw 0x43: `(0x43 >> 4) & 0xffff` = 4 → the frag 0x34, count 3 —
/// fix-explosions2's disc names: sel 1 → 0x36 is the SMOKE grenade, which never explodes, so the old 0x13 setup no
/// longer describes an exploding throw)
/// and put within 8–20 units should throw: the throw anim plays, a grenade entity spawns, flies under gravity,
/// bounces off the world, and explodes. Sister: Bond out of range (dist > 20) → no throw.
#[test]
fn guard_throws_a_grenade_on_level_02() {
    use nf_game::drone::state;
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, rng) = l.drones();
    let mut sim = l.sim(drones, rng);

    // A present normal combat guard (not mode 7 / 0x1e); 07000002's placements carry no grenades
    // (param 11 = 0), so the fields are set the way `NDrone2_DefaultInit` decodes a 0x43 word.
    let gi = sim.list.iter().position(|x| {
        x.present && x.mode != 7 && x.mode != 0x1e && !x.corpse
    }).expect("level 07000002 should have a normal combat guard");
    let g = sim.list[gi].pos;
    sim.list[gi].grenade_weapon = 0x34; // (0x43 >> 4) & 0xffff == 4 → the frag 0x34
    sim.list[gi].grenade_count = 3;     // 0x43 & 0xf == 3
    let gw = sim.list[gi].grenade_weapon;
    let gc = sim.list[gi].grenade_count;
    assert!(gw == 0x34 && gc > 0, "guard has frag grenades: weapon={:#x} count={}", gw, gc);

    // Bond 15 units in front of the guard, in view.
    let bond = bond_at([g[0] + 15.0, g[1], g[2]]);
    // Run the sim so the guard sees Bond and reaches AIM_STAND_FIRE.
    for _ in 0..300 { sim.tick(&d, &bond); }
    // Now force the grenade throw: set up the conditions and set_state.
    {
        let dr = &mut sim.list[gi];
        dr.state = state::AIM_STAND_FIRE;
        dr.next_state = None;
        dr.anim.class = 2;
        dr.anim.state = nf_game::drone::anim_state::AIM_STAND;
        dr.dist = 15.0;
        dr.engage_range = 25.0;
        dr.opp_target = bond.pos;
        dr.seen = true; dr.ever_seen = true; dr.vision_on = true; dr.unseen_ticks = 0;
        dr.clip = 5; dr.burst_done = true;
        dr.cover_claimed = false; dr.has_cover = false; dr.cover = None;
        d.props.set(0x30, &mut dr.props, 0);
        if let Some(ref mut ap) = dr.alert_props { d.props.set(0x30, ap, 0); }
        dr.state = state::GRENADE_THROW;
        dr.pending_grenade = true;
    }
    let before = sim.grenades.len();
    sim.tick(&d, &bond);
    // A grenade entity should have been created.
    assert!(sim.grenades.len() > before, "a grenade was spawned after the throw state");

    // Run the grenade to completion (it should bounce and explode).
    let mut exploded = false;
    for _ in 0..600 {
        sim.tick(&d, &bond);
        if !sim.grenades.is_empty() && sim.grenades[0].state >= 2 { exploded = true; break; }
        if sim.grenades.is_empty() && !sim.explosions.is_empty() { exploded = true; break; }
    }
    assert!(exploded, "the grenade exploded (bounced + detonated)");

    // Sister: Bond out of range (dist > 20) — the guard should NOT throw.
    let (drones2, rng2) = l.drones();
    let mut sim2 = l.sim(drones2, rng2);
    let gi2 = sim2.list.iter().position(|x| x.present && x.mode != 7 && x.mode != 0x1e && !x.corpse).unwrap();
    sim2.list[gi2].grenade_weapon = 0x34; // param 11 raw 0x43 → the frag 0x34 × 3
    sim2.list[gi2].grenade_count = 3;
    let g2 = sim2.list[gi2].pos;
    let bond_far = bond_at([g2[0] + 30.0, g2[1], g2[2]]); // 30 units — outside the 8–20 window
    for _ in 0..300 { sim2.tick(&d, &bond_far); }
    let grenades_before = sim2.grenades.len();
    for _ in 0..300 { sim2.tick(&d, &bond_far); }
    assert!(sim2.grenades.len() == grenades_before, "guard out of range (dist 30) does not throw");
}

/// Unforced: a grenade guard on 07000002 (the test loads Level02 — the old doc said 07000009), Bond in
/// range (15 m), run 1800 ticks (30 s at 60 Hz). The guard should decide to throw through its own
/// decision tree, with the game's RNG. [S unverified: the dtype=0xd / mode 9 note below was written
/// for 07000009 and is not re-checked for 07000002.]
/// (07000009 note: guards have dtype=0xd, mode_normal=0 → mode 9 (HostageKillerAttack — a combat guard).)
/// Sister: Bond at 30 m (out of range) → no throw in the same number of ticks.
#[test]
fn guard_decides_to_throw_grenade_unforced() {
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, rng) = l.drones();
    let mut sim = l.sim(drones, rng);

    // Find a present normal combat guard. On 07000002 no placement carries grenades (param 11 = 0), so the
    // guard's grenade fields are set the way `NDrone2_DefaultInit` decodes param 11 (raw 0x13 → 0x36 × 3; 0x36 is the smoke grenade per fix-explosions2 — the throw decision is the same for every type).
    let gi = sim.list.iter().position(|x| {
        x.present && x.mode == 0 && x.team == 1
    }).expect("level 07000002 should have a normal combat guard");
    let g = sim.list[gi].pos;
    // Give the guard grenades (the way param 11 decodes in `NDrone2_DefaultInit`).
    sim.list[gi].grenade_weapon = 0x36;
    sim.list[gi].grenade_count = 3;
    let gc_before = sim.list[gi].grenade_count;
    eprintln!("unforced: combat guard #{} at ({:.1}, {:.1}, {:.1}), given 3 smoke grenades (0x36), mode={}", gi, g[0], g[1], g[2], sim.list[gi].mode);

    // Bond 12 units in front of the guard (in +x), on the floor. The guard faces roughly -x (yaw -1.5),
    // so Bond is behind the guard. But visibility still fires (dist < sight_range, alert >= 1 after first
    // contact). The key is LOS: we need a clear path. Place Bond in -z direction instead (guard faces
    // -x, but the level's corridors may be clearer in z).
    // Try Bond at guard's z - 12 (in -z direction).
    let bond_offsets: [[f32; 3]; 4] = [
        [12.0, 0.0, 0.0],   // +x
        [-12.0, 0.0, 0.0],  // -x
        [0.0, 0.0, 12.0],   // +z
        [0.0, 0.0, -12.0],  // -z
    ];
    let mut bond = bond_at(g);
    let mut best_los = false;
    for off in &bond_offsets {
        let p = [g[0] + off[0], g[1] + off[1], g[2] + off[2]];
        let fy = l.world.floor_below(p, 4.0).unwrap_or(g[1] - 1.0) + 1.0;
        let bp = [p[0], fy, p[2]];
        if l.world.line_of_sight([g[0], g[1] + 0.6, g[2]], [bp[0], bp[1] + 0.5, bp[2]], nf_game::world::LOS_GUARD_VISION) {
            bond = bond_at(bp);
            best_los = true;
            eprintln!("[scenario] Bond at ({:.1}, {:.1}, {:.1}) — clear LOS from guard at ({:.1}, {:.1}, {:.1})",
                bp[0], bp[1], bp[2], g[0], g[1], g[2]);
            break;
        }
    }
    if !best_los {
        // Fallback: place Bond at +x anyway (the visibility check will still fire).
        let p = [g[0] + 12.0, g[1], g[2]];
        let fy = l.world.floor_below(p, 4.0).unwrap_or(g[1] - 1.0) + 1.0;
        bond = bond_at([p[0], fy, p[2]]);
        eprintln!("[scenario] WARNING: no clear LOS found — Bond at ({:.1}, {:.1}, {:.1})", bond.pos[0], bond.pos[1], bond.pos[2]);
    }

    // Enable the switch channels EXCEPT the mission's own. The old blanket `1..=255` also set the
    // standing fail-hook channels 0x60/0x61 [M `Mission_MonitorObjectives` 0x1827d8:98-117 — each
    // sets ch 99] and the mission-internal channels 99/100 [M `Mission_Update` 0x182c60:23-33], so
    // Mission_Status left the in-mission range within a tick; `PlrStat_OkToUpdate` then failed
    // [M 0x1993e0 — SP: Mission_Status < 2], `NDrone2_FindOpponent` zeroed +0x170 every tick
    // [M 0x1424e0:106-108], the combat router's head returned NoOpponent 0x60 [M 0x148338:20-31]
    // and `DroneWeap_CanThrowGrenade`'s own +0x170 gate [M 0x17a798:16] could never pass: the
    // harness itself ended the mission, and a guard holds no player opponent on a mission that is
    // over. The level's objective channels 0x48/0x4b/0x4d announce but do not move Mission_Status
    // [L ee_start_05.bin 0x2a3ed0 — three complete-armed records, flags 0], so they and the
    // guards' wake channels stay enabled.
    for c in 1..=255u8 {
        if !matches!(c, 0x60 | 0x61 | 0x62 | 99 | 100) { sim.set_channel(c); }
    }

    // Run 1800 ticks (30 s). The guard should see Bond, enter combat, and eventually throw.
    let mut threw = false;
    for t in 0..1800 {
        sim.tick(&d, &bond);
        if t % 60 == 0 && t < 300 {
            let dr = &sim.list[gi];
            let vis = if dr.vision_on || dr.alert >= 1.0 { dr.visibility() } else { 0.0 };
            eprintln!("[trace] tick {}: guard #{} state={:#x} dist={:.1} seen={} vision_on={} vis={:.3} alert={:.2} yaw={:.1} yaw_to_opp={:.1} los={} grenades={}",
                t, gi, dr.state, dr.dist, dr.seen, dr.vision_on, vis, dr.alert, dr.yaw, dr.yaw_to_opp, dr.los, dr.grenade_count);
        }
        if !sim.grenades.is_empty() || sim.list[gi].grenade_count < gc_before {
            threw = true;
            eprintln!("grenade thrown at tick {} (guard state {:#x})", t, sim.list[gi].state);
            break;
        }
    }
    assert!(threw, "guard with grenades (count {}) decided to throw within 1800 ticks — \
        check NF_AILOG=1 for why can_throw_grenade never passed", gc_before);

    // Sister: Bond at 30 m — out of the 8–20 throw window. No throw in 1800 ticks.
    let (drones2, rng2) = l.drones();
    let mut sim2 = l.sim(drones2, rng2);
    let gi2 = sim2.list.iter().position(|x| x.present && x.mode == 0 && x.team == 1)
        .expect("level 07000002 should have a normal combat guard");
    let g2 = sim2.list[gi2].pos;
    sim2.list[gi2].grenade_weapon = 0x36;
    sim2.list[gi2].grenade_count = 3;
    let floor_y2 = l.world.floor_below([g2[0] + 30.0, g2[1], g2[2]], 4.0).unwrap_or(g2[1] - 1.0) + 1.0;
    let bond_far = bond_at([g2[0] + 30.0, floor_y2, g2[2]]);
    let gc_before2 = sim2.list[gi2].grenade_count;
    // The sister runs under the same harness rule: the mission's own channels stay off (see the
    // unforced test's blanket comment — the fail hooks 0x60/0x61 [M 0x1827d8:98-117] and 99/100
    // [M 0x182c60:23-33] would end the mission and the pointer fold [M 0x1424e0:106-108] would
    // zero +0x170, masking the range pin with a no-opponent park).
    for c in 1..=255u8 {
        if !matches!(c, 0x60 | 0x61 | 0x62 | 99 | 100) { sim2.set_channel(c); }
    }
    for _ in 0..1800 { sim2.tick(&d, &bond_far); }
    assert!(sim2.list[gi2].grenade_count == gc_before2,
        "guard out of range (30 m) did not throw — grenades {} == {}", sim2.list[gi2].grenade_count, gc_before2);
}

/// The ConsiderExplosive STATE GATE (GAPS G-055): the game refuses the flee only from 0x28, 0x2d,
/// 0x4a-0x4b, 0x53-0x55 (the < 0x56 arm) and 0x63, 0x86-0x88 (the >= 0x56 arm) [M 0x1431c0:40-67] —
/// every other state considers the flee. The old port gate also refused 0x2e-0x49, 0x52, 0x56 and
/// 0x62-0x85, so a guard in AimStandFire 0x68 never fled a live grenade. These two tests pin the
/// gate's both sides with the suite's pin-then-run convention (the state write is the premise, the
/// msg-0x21 delivery — the bounce broadcast [M 0x1435f0 + 0x138b10] — the gate and the flee are the
/// sim's own path; nothing is written after the bounce).
///
/// The frag setup: 07000002's placements carry no grenades (param 11 = 0), so the fields are set the
/// way `NDrone2_DefaultInit` decodes a 0x43 word (the frag 0x34 × 3), and the fear property 0x3A the
/// broadcast requires [M 0x1435f0:30-34]. Bond stands 8.5 u out — inside the 8–20 throw window and
/// inside the frag's 12.0 scan radius [M 0x138b10:67-87] — so the thrower's own grenade lands within
/// reach of his own flee scan (the corridor-11 d3 lob that "rolls back to the guards", G-055).
fn force_frag_throw(
    d: &Data, sim: &mut Drones, gi: usize, bond: &Player,
) {
    use nf_game::drone::state;
    {
        let dr = &mut sim.list[gi];
        dr.enabled = true; // a fighting guard is AI-active (the game's obj+0x4F8 & 0x100); the
                           // WAIT_SWITCH wake leaves the port's `enabled` cleared and the broadcast
                           // (like the game's HandleExplosives) scans active guards only
        dr.grenade_weapon = 0x34; // (0x43 >> 4) & 0xffff == 4 → the frag 0x34
        dr.grenade_count = 3;     // 0x43 & 0xf == 3
        d.props.set(0x3a, &mut dr.props, 1); // the fear prop the broadcast scans [M 0x1435f0:30-34]
        if let Some(ref mut ap) = dr.alert_props { d.props.set(0x3a, ap, 1); }
        d.props.set(0x34, &mut dr.props, 1); // ConsiderExplosive's own prop gate [M 0x1431c0:22-30]
        if let Some(ref mut ap) = dr.alert_props { d.props.set(0x34, ap, 1); }
        dr.state = state::AIM_STAND_FIRE;
        dr.next_state = None;
        dr.anim.class = 2;
        dr.anim.state = nf_game::drone::anim_state::AIM_STAND;
        dr.dist = 10.0;
        dr.engage_range = 25.0;
        dr.opp_target = bond.pos;
        dr.seen = true; dr.ever_seen = true; dr.vision_on = true; dr.unseen_ticks = 0;
        dr.clip = 5; dr.burst_done = true;
        dr.cover_claimed = false; dr.has_cover = false; dr.cover = None;
        d.props.set(0x30, &mut dr.props, 0);
        if let Some(ref mut ap) = dr.alert_props { d.props.set(0x30, ap, 0); }
        dr.state = state::GRENADE_THROW;
        dr.pending_grenade = true;
    }
}


#[test]
fn a_guard_in_aim_stand_fire_0x68_flees_his_own_landed_frag() {
    use nf_game::drone::state;
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    // The law under test: a delivered msg 0x21 makes an AimStandFire 0x68 guard flee his own
    // landed frag [M 0x1431c0:40-67 — 0x68 is not in the refuse set]. The DELIVERY premise is
    // trajectory-dependent: the frag must come to rest inside the thrower's 12 u radius AND in
    // his sight, and the joined tree's RNG streams (gas draws + muzzle draws + guard radii)
    // roll the corridor-11 lob away and out of sight (stream-batch-2: stale-harness scenario,
    // the scan itself is the game's [M 0x1435f0:30-52 + 0x138b10:18-19]). So the standoff is a
    // searched harness lever, not a pin: the first standoff in the list whose frag delivers
    // within the 30-tick window carries the asserts; every candidate runs the same law.
    let mut ran = None;
    for standoff in [10.0f32, 8.5, 12.0, 14.0, 16.0] {
        let (drones, rng) = l.drones();
        let mut sim = l.sim(drones, rng);
        let gi = sim.list.iter().position(|x| x.present && x.mode == 0 && x.team == 1 && !x.corpse)
            .expect("level 07000002 should have a normal combat guard");
        let g = sim.list[gi].pos;
        let bpy = l.world.floor_below([g[0] + standoff, g[1], g[2]], 4.0).unwrap_or(g[1] - 1.0) + 1.0;
        let bond = bond_at([g[0] + standoff, bpy, g[2]]);
        for _ in 0..300 { sim.tick(&d, &bond); }
        force_frag_throw(&d, &mut sim, gi, &bond);
        sim.tick(&d, &bond);
        if sim.grenades.len() != 1 { continue; } // the throw refused on this standoff — next lever
        {
            let dr = &mut sim.list[gi];
            dr.grenade_weapon = 0;
            dr.grenade_count = 0;
            dr.pending_grenade = false;
        }
        // Flight: the thrower is in GRENADE_THROW 0x2d, which ConsiderExplosive's state gate
        // REFUSES [M 0x1431c0:40-67] — the per-frame broadcast [M 0x1435f0:20-52] cannot scare
        // him mid-flight. From the bounce on he stands in AimStandFire 0x68 (the state the
        // game's AimStandFire case 0x21 passes as the resume [M 0x166328:184-187]).
        let mut bounced_at = None;
        for t in 0..240u32 {
            {
                let dr = &mut sim.list[gi];
                dr.state = state::GRENADE_THROW; // the throw itself — refused by the flee gate [M 0x1431c0:40-67]
                dr.next_state = None;
                dr.firing = false;
                dr.unseen_ticks = 0;
            }
            sim.tick(&d, &bond);
            if sim.grenades.iter().any(|gr| gr.state < 2 && gr.bounces >= 1) { bounced_at = Some(t); break; }
            if sim.grenades.is_empty() { break; }
        }
        let bounced_at = match bounced_at { Some(t) => t, None => continue };
        let mut frag_at_delivery = None;
        {
            let dr = &sim.list[gi];
            let gr = &sim.grenades[0];
            let dist = ((dr.pos[0] - gr.pos[0]).powi(2) + (dr.pos[2] - gr.pos[2]).powi(2)).sqrt();
            eprintln!("gate: standoff {standoff}: frag bounced at t+{bounced_at}, thrower state {:#x}, frag {dist:.2} u away (scan radius 12), guard {:?}, frag {:?}",
                dr.state, dr.pos, gr.pos);
            if dist >= 12.0 { continue; } // out of radius at the bounce — next lever
        }
        // From the bounce on: the 0x68 premise, pinned until the msg-0x21 delivery latches +0x2B
        // (scared — set by ConsiderExplosive 0x1431c0:194).
        for _ in 0..30u32 {
            {
                let dr = &mut sim.list[gi];
                dr.state = state::AIM_STAND_FIRE;
                dr.next_state = None;
                dr.firing = true;
                dr.burst_done = false;
                dr.unseen_ticks = 0;
            }
            sim.tick(&d, &bond);
            if sim.list[gi].scared {
                frag_at_delivery = Some(sim.grenades[0].pos);
                break;
            }
        }
        let frag_at_delivery = match frag_at_delivery { Some(p) => p, None => continue };
        // Unpinned from the delivery on: the game's gate lets 0x68 consider the flee [M 0x1431c0:40-67 —
        // only 0x28/0x2d/0x4a-0x4b/0x53-0x55/0x63/0x86-0x88 refuse], so the thrower must leave 0x68 for
        // the flee family (RunAwayFromObject 0x86 [M 0x1431c0:194-195], whose arrival parks in the
        // +0x354 return 0x87).
        let mut fled_at = None;
        for t in 0..10u32 {
            let dr = &sim.list[gi];
            if matches!(dr.state, state::RUN_AWAY_FROM_OBJECT | state::HIDE_FROM_SCARY_OBJECT) {
                fled_at = Some(t);
                break;
            }
            sim.tick(&d, &bond);
        }
        let fled_at = fled_at.unwrap_or_else(|| panic!(
            "a guard in AimStandFire 0x68 flees his own landed frag [M 0x1431c0:40-67 — the gate refuses \
             only 0x28/0x2d/0x4a-0x4b/0x53-0x55/0x63/0x86-0x88; the old port gate also refused 0x62-0x85] — \
             state after the delivery {:#x}", sim.list[gi].state));
        eprintln!("gate: standoff {standoff}: the 0x68 guard fled at delivery+{fled_at} (state {:#x})", sim.list[gi].state);
        // The flee aims away from the explosive: +0x330 holds the grenade's position at
        // ConsiderExplosive's own run [M 0x1431c0:172-175] — the frag still rolls, so the check is
        // against the position captured at the delivery tick, not the later rest spot.
        let dr = &sim.list[gi];
        let ep = dr.explosive_pos;
        let dpos = ((ep[0] - frag_at_delivery[0]).powi(2) + (ep[2] - frag_at_delivery[2]).powi(2)).sqrt();
        assert!(dpos < 3.0, "the flee's explosive position is the grenade's spot at the delivery ({dpos:.2} u off)");
        ran = Some((standoff, gi));
        break;
    }
    let (standoff, gi) = ran.expect("no standoff in {10, 8.5, 12, 14, 16} delivered the 0x21 premise — the harness needs a new lever (the law itself is pinned by the sister 0x63 test's gate asserts)");
    eprintln!("gate: the premise held at standoff {standoff} on guard list[{gi}]");
}

/// Sister: the same landed frag, but the guard is pinned in GoToGoalPosition 0x63 — which the game's
/// gate REFUSES [M 0x1431c0:63] — so he does not flee it. The pin is the same premise write each
/// tick BEFORE it: a broken gate would still set the flee state inside the tick and be caught.
#[test]
fn a_guard_in_go_to_goal_0x63_does_not_flee_the_landed_frag() {
    use nf_game::drone::state;
    let Some(l) = Level02::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, rng) = l.drones();
    let mut sim = l.sim(drones, rng);
    let gi = sim.list.iter().position(|x| x.present && x.mode == 0 && x.team == 1 && !x.corpse)
        .expect("level 07000002 should have a normal combat guard");
    let g = sim.list[gi].pos;
    let bpy = l.world.floor_below([g[0] + 10.0, g[1], g[2]], 4.0).unwrap_or(g[1] - 1.0) + 1.0;
    let bond = bond_at([g[0] + 10.0, bpy, g[2]]);
    for _ in 0..300 { sim.tick(&d, &bond); }
    force_frag_throw(&d, &mut sim, gi, &bond);
    sim.tick(&d, &bond);
    assert!(sim.grenades.len() == 1, "the frag spawned (grenades {})", sim.grenades.len());
    {
        let dr = &mut sim.list[gi];
        dr.grenade_weapon = 0;
        dr.grenade_count = 0;
        dr.pending_grenade = false;
    }

    // Pin 0x63 through the whole live window (the game rescans every frame while the object exists
    // [M 0x1435f0 — the port's once-at-the-bounce is the documented deviation], so the refusal must
    // hold at every delivery, not just the first). Bond is pulled out of sight so the 0x63 walk is
    // the natural premise.
    let bond_gone = bond_at([g[0] + 60.0, g[1], g[2]]);
    let mut bounced = false;
    for t in 0..260u32 {
        {
            let dr = &mut sim.list[gi];
            dr.state = state::GO_TO_GOAL;
            dr.next_state = None;
            // Lock the premise through the tick: the guard's own update runs BEFORE the bounce's
            // broadcast inside a tick, and a long in-progress route is the walk's own stay —
            // GoToGoal keeps pathing to a far goal [M 0x165640 the GoToGoalPosition UPDATE].
            dr.goal = Some([g[0] - 80.0, g[1], g[2]]);
            dr.croute = None;
        }
        sim.tick(&d, &bond_gone);
        let dr = &sim.list[gi];
        assert!(!matches!(dr.state, state::RUN_AWAY_FROM_OBJECT | state::HIDE_FROM_SCARY_OBJECT | state::RECOVER_FROM_SCARY_OBJECT),
            "a guard in GoToGoal 0x63 does not flee the landed frag [M 0x1431c0:63] — fled at t+{t} (state {:#x})", dr.state);
        if sim.grenades.iter().any(|gr| gr.state < 2 && gr.bounces >= 1) { bounced = true; break; }
        if sim.grenades.is_empty() { bounced = true; break; }
    }
    assert!(bounced, "the frag bounced within 260 ticks");
    eprintln!("gate: the 0x63 guard held through the bounce (state {:#x})", sim.list[gi].state);
    // Keep the window open past the bounce: no flee-family state may appear while the pin holds.
    for t in 0..130u32 {
        {
            let dr = &mut sim.list[gi];
            dr.state = state::GO_TO_GOAL;
            dr.next_state = None;
        }
        sim.tick(&d, &bond_gone);
        let dr = &sim.list[gi];
        assert!(!matches!(dr.state, state::RUN_AWAY_FROM_OBJECT | state::HIDE_FROM_SCARY_OBJECT | state::RECOVER_FROM_SCARY_OBJECT),
            "a guard in GoToGoal 0x63 does not flee the landed frag [M 0x1431c0:63] — fled at bounce+{t} (state {:#x})", dr.state);
    }
}
