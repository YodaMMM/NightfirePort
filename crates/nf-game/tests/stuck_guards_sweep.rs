//! stuck-guards-sweep (976): a headless sweep over every on-foot SP level for guards that are in
//! a MOVING state yet do not move. The real alert path only — no manual state sets:
//! - Phase A: Bond far away (`bond_far`), 15 s — patrol/route movers measured;
//! - Phase B: for each armed, alive guard, Bond 6 units in front of it (inside its sight range
//!   and FOV), 15 s — the game's own vision pass alerts it and the real combat flow runs.
//! The sweep logs every drone whose horizontal displacement over the last 6 s is < 0.1 while its
//! state is one of the states the decompile moves through, with its goal, the straight-line
//! blocker to that goal, and whether it ended parked in the game's own give-up states
//! (CombatWait 0x5e parks by design when `DroneMove_NoBunching` 0x13cfb0:35-39 flags a drone
//! ahead; CombatNoRoute 0x5f and DroneStuck 0x7f are the route-failure parks [M]).
//!
//! The MOVING set, each with its decompile state (NDrone2_StateFuncs 0x29b300 [M table]):
//! Patrol 0x07, Attack 0x56, Combat 0x58, CombatOutOfRange 0x5a (approach [M 0x164690]),
//! CombatNoSight 0x5c (move to the last-seen point), AlertToPosition 0x62, GoToGoalPosition 0x63,
//! HostageGoToGoal 0x64 (0x165a68), SearchArea 0x65, HeardNoise 0xa2-0xa5, RunForCover 0x8c.
//!
//! Opt-in: `NF_ISO`. Diagnostic: prints the hit table and a TOTAL line; asserts nothing so the
//! suite never blocks on a census.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drones, Player as Bond, Rng, state};
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
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))?;
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).ok()?;
        let lb = LevelBin::parse(&data).unwrap();
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok()).map(|k| (k.id as u8, k)).collect();
        let m = level.main_map()?;
        let nav = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref()?, &world, code);
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

use nf_game::drone::Drone;

fn bond_at(p: [f32; 3]) -> Bond {
    Bond { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None, crouched: false }
}

fn bond_far() -> Bond {
    bond_at([0.0, -500.0, 0.0])
}

fn armed(w: u32) -> bool { (2..=0x46).contains(&w) || (0x6a..=0x71).contains(&w) }

/// The states the decompile moves the drone through (see the file docstring).
const MOVING: &[u8] = &[
    state::PATROL, state::ATTACK, state::COMBAT, state::COMBAT_OUT_OF_RANGE, state::COMBAT_NO_SIGHT,
    state::ALERT_TO_POSITION, state::GO_TO_GOAL, state::HOSTAGE_GO_TO_GOAL, state::SEARCH_AREA,
    state::HEARD_NOISE, state::HEARD_NOISE_AWARE, state::HEARD_NOISE_SUSPECT, state::HEARD_NOISE_ALERT,
    state::RUN_FOR_COVER,
];

const PHASE_TICKS: usize = 600; // 10 s at the fixed 60 Hz
const WINDOW: usize = 360; // the last 6 s
const STILL: f32 = 0.1;

fn state_name(s: u8) -> &'static str {
    match s {
        state::PATROL => "PATROL 0x07", state::ATTACK => "ATTACK 0x56", state::COMBAT => "COMBAT 0x58",
        state::COMBAT_OUT_OF_RANGE => "COMBAT_OUT_OF_RANGE 0x5a", state::COMBAT_NO_SIGHT => "COMBAT_NO_SIGHT 0x5c",
        state::COMBAT_WAIT => "COMBAT_WAIT 0x5e", state::COMBAT_NO_ROUTE => "COMBAT_NO_ROUTE 0x5f",
        state::GO_TO_GOAL => "GO_TO_GOAL 0x63", state::ALERT_TO_POSITION => "ALERT_TO_POSITION 0x62",
        state::HOSTAGE_GO_TO_GOAL => "HOSTAGE_GO_TO_GOAL 0x64", state::SEARCH_AREA => "SEARCH_AREA 0x65",
        state::HEARD_NOISE => "HEARD_NOISE 0xa2", state::HEARD_NOISE_AWARE => "HEARD_NOISE_AWARE 0xa3",
        state::HEARD_NOISE_SUSPECT => "HEARD_NOISE_SUSPECT 0xa4", state::HEARD_NOISE_ALERT => "HEARD_NOISE_ALERT 0xa5",
        state::RUN_FOR_COVER => "RUN_FOR_COVER 0x8c", state::INVESTIGATE => "INVESTIGATE 0x7e",
        state::DRONE_STUCK => "DRONE_STUCK 0x7f", state::IDLE => "IDLE 0x04", state::ALERT => "ALERT 0x05",
        state::COMBAT_NO_MOVE => "COMBAT_NO_MOVE 0x59", state::NO_OPPONENT => "NO_OPPONENT 0x60",
        state::DEAD => "DEAD 0x47", state::FADE => "FADE 0x48",
        _ => "?",
    }
}

struct Track { disp: f32, moved: bool, states: Vec<u8>, pos_end: [f32; 3], goal_end: Option<[f32; 3]> }

fn track(dr: &Drone, start: [f32; 3], states: Vec<u8>) -> Track {
    let dx = dr.pos[0] - start[0];
    let dz = dr.pos[2] - start[2];
    Track { disp: (dx * dx + dz * dz).sqrt(), moved: true, states, pos_end: dr.pos, goal_end: dr.goal }
}

fn main_sweep() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let (entries, _bin) = disc::files_bin_entries(&iso).unwrap();
    let mut codes: Vec<u32> = entries.iter()
        .filter(|e| e.name.starts_with("0700") && e.name.ends_with(".bin"))
        .filter_map(|e| u32::from_str_radix(e.name.trim_end_matches(".bin"), 16).ok())
        .collect();
    codes.sort();
    println!("SWEEP levels: {}", codes.iter().map(|c| format!("{c:08x}")).collect::<Vec<_>>().join(" "));
    let mut total_hits = 0usize;
    for code in codes {
        let Some(l) = LevelX::load(code) else { println!("SWEEP {code:08x}: load failed"); continue };
        let d = l.data(code);
        let (drones, rng) = l.drones(code);
        let mut sim = Drones::new(drones, rng);
        sim.attach_level_id(&l.level, code);
        for _ in 0..5 { sim.tick(&d, &bond_far()); } // the spawn scripts must run before `started` means anything
        let armed_alive: Vec<usize> = (0..sim.list.len()).filter(|&i| {
            let g = &sim.list[i];
            g.started && g.present && g.enabled && !g.corpse && armed(g.weapon) && !g.holstered
        }).collect();
        println!("SWEEP {code:08x}: {} drones ({} armed alive), {} ticks/phase x {}", sim.list.len(), armed_alive.len(), PHASE_TICKS, 1 + armed_alive.len().min(20));
        let mut hits: Vec<String> = Vec::new();
        let mut phase_bonds: Vec<Bond> = vec![bond_far()];
        for &gi in armed_alive.iter().take(16) {
            let g = &sim.list[gi];
            let yaw = g.desired_yaw;
            let p = [g.pos[0] + yaw.sin() * 6.0, g.pos[1], g.pos[2] + yaw.cos() * 6.0];
            phase_bonds.push(bond_at(p));
        }
        // The disabled WAIT_SWITCH guards the scripts enable only when the player reaches them:
        // teleport Bond to each of those too (cap 20) — the proximity + sight is the game's own
        // enabling path (the level scripts' volumes + the vision pass), never a manual state set.
        let idle: Vec<usize> = (0..sim.list.len()).filter(|&i| {
            let g = &sim.list[i];
            g.started && g.enabled && !g.corpse && !armed_alive.contains(&i) && g.weapon != 0 && (g.state == state::IDLE || g.state == state::WAIT_SWITCH)
        }).take(20 - armed_alive.len().min(16)).collect();
        for &gi in &idle {
            let g = &sim.list[gi];
            let yaw = if g.desired_yaw != 0.0 { g.desired_yaw } else { 0.0 };
            let p = [g.pos[0] + yaw.sin() * 2.5, g.pos[1], g.pos[2] + yaw.cos() * 2.5];
            phase_bonds.push(bond_at(p));
        }
        for (phase, bond) in phase_bonds.iter().enumerate() {
            // Fresh sim per phase so each phase measures one alerting configuration cleanly.
            let (drones, rng) = l.drones(code);
            let mut sim = Drones::new(drones, rng);
            sim.attach_level_id(&l.level, code);
            sim.scripts.skip_request = true; // the game's own intro skip; the phases measure combat, not cinematics
            for _ in 0..5 { sim.tick(&d, &bond_far()); }
            let n = sim.list.len();
            let mut starts = vec![[0.0f32; 3]; n];
            let mut statelog: Vec<Vec<u8>> = vec![Vec::new(); n];
            for i in 0..n { starts[i] = sim.list[i].pos; }
            for t in 0..PHASE_TICKS {
                sim.tick(&d, bond);
                if t >= PHASE_TICKS - WINDOW {
                    for i in 0..n { statelog[i].push(sim.list[i].state); }
                }
            }
            for i in 0..n {
                let g = &sim.list[i];
                if !g.started || !g.present || !g.enabled || g.corpse || g.weapon == 0 { continue; }
                let tr = track(g, starts[i], statelog[i].clone());
                let moving_frac = tr.states.iter().filter(|s| MOVING.contains(s)).count();
                let end_moving = tr.states.last().is_some_and(|s| MOVING.contains(s));
                let parked = matches!(tr.states.last(), Some(&state::COMBAT_NO_ROUTE) | Some(&state::DRONE_STUCK));
                let bunched = tr.states.last() == Some(&state::COMBAT_WAIT);
                // HIT-A: a mover that never moved; HIT-B: parked in a route-failure state.
                if (end_moving && tr.goal_end.is_some() && tr.disp < STILL && moving_frac >= WINDOW * 3 / 4)
                    || (parked && tr.disp < STILL) {
                    let goal = tr.goal_end;
                    let los = goal.map(|g| {
                        let from = [tr.pos_end[0], tr.pos_end[1] + 1.0, tr.pos_end[2]];
                        let to = [g[0], g[1] + 1.0, g[2]];
                        match l.world.segment_hit_scoped(None, from, to) { Some((f, _)) => format!("blocked@{f:.2}"), None => "clear".to_string() }
                    }).unwrap_or_else(|| "n/a".to_string());
                    let gdist = goal.map(|g| ((g[0] - tr.pos_end[0]).powi(2) + (g[2] - tr.pos_end[2]).powi(2)).sqrt()).unwrap_or(0.0);
                    hits.push(format!(
                        "{code:08x} | ph{phase} | dr{i} | pos ({:.1},{:.1},{:.1}) | {} | goal {:?} d={gdist:.1} | LOS {los} | disp {:.3} | moving {moving_frac}/{WINDOW} | bunched {bunched}",
                        tr.pos_end[0], tr.pos_end[1], tr.pos_end[2], state_name(g.state), goal.map(|g| format!("({:.1},{:.1},{:.1})", g[0], g[1], g[2])), tr.disp));
                }
            }
        }
        for h in &hits { println!("HIT {}", h); }
        total_hits += hits.len();
    }
    println!("SWEEP TOTAL: {total_hits} hits");
}

#[test]
fn sweep_all_on_foot_sp_levels_for_stuck_moving_guards() {
    main_sweep();
}
