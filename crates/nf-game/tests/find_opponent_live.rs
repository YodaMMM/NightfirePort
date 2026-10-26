//! `986-find-opponent-exact`: NDrone2_FindOpponent's team-2 scan (0x1424e0) — the exact alive
//! filter and the shared per-call LOS budget, against the decompile + the scan-loop disassembly.
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drone, Drones, Player, Rng};
use std::collections::HashMap;

struct Level05 {
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
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    surfaces: nf_formats::sfx::SurfaceSounds,
    voices: Vec<nf_formats::sfx::VoiceLine>,
}

impl Level05 {
    fn load() -> Option<Level05> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).unwrap();
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok()).map(|k| (k.id as u8, k)).collect();
        let nav = nf_game::nav::Network::build(level.maps[level.main_map().unwrap()].ai_network.as_ref().unwrap(), &world, 0x0700_0005);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let volumes = nf_game::aivolume::build(&level, 0x0700_0005);
        Some(Level05 {
            tables: drone::DroneAnimData::from_action_elf(&elf).unwrap(),
            props: drone::PropertyTables::from_action_elf(&elf).unwrap(),
            weapons: WeaponData::from_action_elf(&elf).unwrap(),
            sfx: nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap(),
            tuning: { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap(); nf_game::tuning::read_tuning_vars(&mut t, 0x0700_0005); t },
            hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap(),
            sequences: disc::level_sequences(&iso, "07000005.bin").unwrap(),
            models: nf_game::pickup::models(&level),
            surfaces: nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).unwrap(),
            voices: nf_formats::sfx::voice_lines(&elf).unwrap(),
            level, scripts, world, skeletons, nav, covers, volumes,
        })
    }

    fn data(&self) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: 0x0700_0005,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1,
                ai_points: &[],
        }
    }

    fn drones(&self) -> (Vec<Drone>, Rng) {
        let d = self.data();
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        (nf_game::drone::level_drones(&d, &self.level, &mut rng, &mut lights), rng)
    }

    fn sim(&self, drones: Vec<Drone>, rng: Rng) -> Drones {
        let mut s = Drones::new(drones, rng); s.attach_level_id(&self.level, 0x0700_0005);
        s.difficulty = 1;
        s
    }
}

fn bond_at(p: [f32; 3]) -> Player { Player { yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, cel: None, crouched: false } }

/// Two known mutually-visible spots (ai_live's guard/Bond pair: the guard at x 61.342 sees Bond at
/// (67.31, 26.5, 99.47) through the open floor — the notice test's geometry).
const OPEN_ALLY: [f32; 3] = [67.31, 26.5, 99.47];

fn guard_spot(l: &Level05) -> [f32; 3] {
    let (drones, _) = l.drones();
    drones.iter().find(|g| (g.pos[0] - 61.342).abs() < 0.01).map(|g| g.pos).expect("the 61.342 guard")
}

/// The scan's exact alive filter [M 0x1424e0 disasm 0x143098-0x1430f8], pinned two ways: the
/// gate method directly, and through the scan — with the cursor parked ON the guard, the healthy
/// case picks him (his LOS passes, ai_live's pair), and each dead arm leaves him unpicked: a
/// cheap-but-dead guard would have been probed and picked the same way, so the pick difference IS
/// the gate.
#[test]
fn scan_skips_dead_candidates_and_picks_the_live_one() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, rng) = l.drones();
    let g = drones.iter().position(|g| (g.pos[0] - 61.342).abs() < 0.01).expect("the 61.342 guard");
    let mut sim = l.sim(drones, rng);
    let a = (0..sim.list.len()).find(|&i| i != g && sim.list[i].present).expect("a live drone");
    sim.list[a].team = 2;
    sim.list[a].pos = OPEN_ALLY;
    // The gate method, each arm [M 0x143098-0x1430f8]:
    assert!(sim.list[g].find_opponent_alive(), "the live guard passes");
    let health = sim.list[g].health;
    sim.list[g].health = 0.0;
    assert!(!sim.list[g].find_opponent_alive(), "health 0 fails the +0xAC arm (PostDroneControl 0x14aff8 flags him 0x400)");
    sim.list[g].health = health;
    sim.list[g].corpse = true;
    assert!(!sim.list[g].find_opponent_alive(), "SetAsDead 0x200 (0x14a2f0:9) fails");
    sim.list[g].corpse = false;
    sim.list[g].delete_me = true;
    assert!(!sim.list[g].find_opponent_alive(), "obj+0xFE bit 0 fails [M 0x1430e4-0x1430f0]");
    sim.list[g].delete_me = false;
    // Through the scan: the cursor parked on him, so he is the FIRST candidate.
    let cases: [(&str, Box<dyn Fn(&mut Drone)>); 4] = [
        ("healthy", Box::new(|_: &mut Drone| {})),
        ("health 0", Box::new(|g: &mut Drone| { g.health = 0.0; })),
        ("corpse (SetAsDead 0x200)", Box::new(|g: &mut Drone| { g.corpse = true; })),
        ("delete-me (+0xFE bit 0)", Box::new(|g: &mut Drone| { g.delete_me = true; })),
    ];
    for (name, setup) in cases.iter() {
        let before = sim.los_probes;
        {
            let ally = &mut sim.list[a];
            ally.opp_drone = None;
            ally.opp_cursor = g; // the walk starts at the guard: the first cheap candidate IS him
        }
        setup(&mut sim.list[g]);
        let (picks, cursors) = sim.team2_opponent_picks(&d);
        let picked = picks[a].map(|(c, _, _, _)| c);
        if *name == "healthy" {
            assert_eq!(picked, Some(g), "{name}: the live guard in range with LOS is picked");
            assert_eq!(sim.los_probes - before, 1, "{name}: one probe (the scan's first cheap candidate) [M 0x1424e0:231-242]");
        } else {
            assert_ne!(picked, Some(g), "{name}: the dead candidate is not picked");
            // The budget caps the call at ONE scan probe [M 0x143144-0x14314c] — the walk went on
            // to some other in-range candidate (the map has several) or found none.
            assert!(sim.los_probes - before <= 1, "{name}: at most one probe per call");
        }
        let _ = cursors;
        // Restore the guard for the next case.
        sim.list[g].health = sim.list[g].max_health;
        sim.list[g].corpse = false;
        sim.list[g].delete_me = false;
    }
}

/// The shared budget: per FindOpponent call at most ONE scan probe (the entry snapshot
/// [M 0x142554] vs the bail `snapshot < DAT_00299092` [M 0x143144-0x14314c]) plus the held
/// opponent's probe [M :152, taken before the scan and after the same snapshot]. Each scenario
/// runs on ONE ally with the cursor pinned, so the counts are exact.
#[test]
fn los_budget_matches_the_counter_rule() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, rng) = l.drones();
    let g = drones.iter().position(|g| (g.pos[0] - 61.342).abs() < 0.01).expect("the 61.342 guard");
    let mut sim = l.sim(drones, rng);
    let a = (0..sim.list.len()).find(|&i| i != g && sim.list[i].present).expect("a live drone");
    sim.list[a].team = 2;
    sim.list[a].pos = OPEN_ALLY;

    // (1) no held opponent: exactly one scan probe, the live guard wins.
    sim.list[a].opp_drone = None;
    sim.list[a].opp_cursor = g;
    let before = sim.los_probes;
    let (picks, _) = sim.team2_opponent_picks(&d);
    assert_eq!(sim.los_probes - before, 1, "one scan probe [M 0x142554 + 0x143144-0x14314c]");
    assert_eq!(picks[a].map(|(c, _, _, _)| c), Some(g));

    // (2) holding the guard himself: the re-validation probe only (the scan skips the held node
    // [M :223-226] and no other candidate — the cursor sits on the held one).
    sim.list[a].opp_drone = Some(g);
    sim.list[a].opp_cursor = g;
    let before = sim.los_probes;
    let (picks, _) = sim.team2_opponent_picks(&d);
    assert_eq!(sim.los_probes - before, 1, "held + no other candidate: only the re-validation probe [M 0x1424e0:152]");
    assert_eq!(picks[a].map(|(c, _, _, _)| c), Some(g), "the re-validated opponent stays");

    // (3) holding him with a second cheap candidate first in the walk: the re-validation probe
    // AND the scan's probe — two [M :152 + :231-242].
    let g2 = sim.list.iter().position(|x| x.present && x.team == 1 && x.index != g).expect("a second guard");
    sim.list[g2].pos = [OPEN_ALLY[0] + 1.0, OPEN_ALLY[1], OPEN_ALLY[2] + 1.0];
    sim.list[a].opp_drone = Some(g);
    sim.list[a].opp_cursor = g2;
    let before = sim.los_probes;
    let (_, _) = sim.team2_opponent_picks(&d);
    assert_eq!(sim.los_probes - before, 2, "held + a second candidate: two probes");

    // (4) the bail: the first cheap candidate's LOS fails (he is buried) — exactly one probe, no
    // pick; the next call probes the NEXT candidate, still one probe [M :239-242 saves the cursor].
    let buried = {
        let g2 = sim.list.iter().position(|x| x.present && x.team == 1 && x.index != g).expect("a guard");
        sim.list[g2].pos = [OPEN_ALLY[0], OPEN_ALLY[1] - 100.0, OPEN_ALLY[2]];
        sim.list[g2].team = 1;
        sim.list[g2].health = 100.0;
        g2
    };
    sim.list[a].opp_drone = None;
    sim.list[a].opp_cursor = buried;
    let before = sim.los_probes;
    let (picks, _) = sim.team2_opponent_picks(&d);
    assert_eq!(sim.los_probes - before, 1, "the buried first candidate: one failed probe, then the walk stops");
    assert_eq!(picks[a], None, "no pick on the bail tick");
    sim.list[a].opp_cursor = buried + 1; // the resumed walk starts past the probed candidate
    let before = sim.los_probes;
    let (picks, _) = sim.team2_opponent_picks(&d);
    assert_eq!(sim.los_probes - before, 1, "the resumed call still spends one probe");
    // The resumed walk's winner is whichever in-range candidate follows the cursor (the map has
    // several) — the pinned facts are the ONE probe and a live pick that is not the buried one.
    let won = picks[a].map(|(c, _, _, _)| c);
    assert!(won.is_some_and(|c| c != buried && sim.list[c].team == 1), "a live guard wins on the resumed walk, got {won:?}");

    // (5) N allies scanning in one call: the counter grows by N — each ally's call re-snapshots
    // [M 0x142554] and spends its own one scan probe.
    let mut n_allies = 0;
    let extra: Vec<usize> = (0..sim.list.len()).filter(|&i| i != g && sim.list[i].present && i != a).take(2).collect();
    for (k, &i) in extra.iter().enumerate() {
        sim.list[i].team = 2;
        sim.list[i].pos = [OPEN_ALLY[0] + 0.1 + k as f32 * 0.05, OPEN_ALLY[1], OPEN_ALLY[2]];
        sim.list[i].opp_drone = None;
        sim.list[i].opp_cursor = g;
        n_allies += 1;
    }
    sim.list[a].opp_drone = None;
    sim.list[a].opp_cursor = g;
    let before = sim.los_probes;
    let (_, _) = sim.team2_opponent_picks(&d);
    assert_eq!(sim.los_probes - before, 1 + n_allies, "each ally's call spends its own one scan probe");
}

/// Through the real tick (the ControlSTANDARD call shape): an ally notices and holds the guard.
#[test]
fn ally_picks_the_guard_through_the_tick() {
    let Some(l) = Level05::load() else { eprintln!("skip: NF_ISO not set"); return };
    let d = l.data();
    let (drones, rng) = l.drones();
    let g = drones.iter().position(|g| (g.pos[0] - 61.342).abs() < 0.01).expect("the 61.342 guard");
    let mut sim = l.sim(drones, rng);
    let a = (0..sim.list.len()).find(|&i| i != g && sim.list[i].present).expect("a live drone");
    sim.list[a].team = 2;
    sim.list[a].pos = OPEN_ALLY;
    let away = bond_at([0.0, -500.0, 0.0]);
    for _ in 0..5 { sim.tick(&d, &away); }
    let mut picked = None;
    for _ in 0..30 {
        sim.tick(&d, &away);
        if sim.list[a].opp_drone.is_some() { picked = sim.list[a].opp_drone; break; }
    }
    assert_eq!(picked, Some(g), "the ally's opponent is the live guard after the tick's pick");
}
