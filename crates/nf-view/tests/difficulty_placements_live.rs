//! difficulty-gated placements: for every on-foot level and each difficulty 1/2/3, the port's
//! spawned and locked sets match the [D] placements filtered by the [M] difficulty rules.
//!
//! The [M] laws — the complete set of placement creators reading the difficulty global
//! GameState+0x28 (0x2a3790; Ghidra's `GameState._40_4_` field is DECIMAL 40). The decompile
//! greps to 19 `GameState._40_4_` reader files in `work/decomp/ACTION.ELF`; the placement
//! creators among them are exactly these six (the rest are runtime scalars: HitDamage,
//! DoBulletAccuracy, Player_HandlePain, RamLoad, GetScore, AutoAim, Player_InitWeapon's
//! auto-aim distances, bootup/PTP/menus):
//! - `Drone_Create` 0x136be0:49 — the kind-0x0F placement's param 2 (level_tag+0x34) is the
//!   MINIMUM difficulty: the drone spawns iff `param2 <= difficulty` (u32 compare).
//! - `Lock_Create` 0x1ae1f8:67-69 — a kind-0x2E lock with param 6 (level_tag+0x44) non-zero
//!   starts UNLOCKED on Operative only (difficulty == 1) [M Lock_Unlock 0x1b0650:28-31].
//! - `SoundTrigger_Create` 0x1b2628:24-40 — the kind-0xF9 gate is param 6: refuse iff
//!   `gate != 0 && gate < difficulty`; then gate 5/6/7 fire only on difficulty 1/2/3.
//! - `Hint_Create` 0x1b3248:12-25 — the kind-0x33 stage is param 2, the same law.
//! - `Copter_Create` 0x17d340:76-94 — the kind-0x2F copter's HP = 299.0 × the difficulty
//!   multiplier (d1 1.0, d2 1.5, d3 2.0, d4 4.0).
//! - `NDrone2_DefaultInit` 0x14b300:504-535 — the captain flag: property 9 on d1, 5 on d2,
//!   8 on d3 (the captain skins 0x5000038/0x500002F/0x5000006/0x5000093/0x50000B2 by kind
//!   1/2/3/6/7; the flag swap is pinned here, the skin map in drone.rs's unit tests).
//!
//! Everything else the task lists reads something else: `Door_Create` 0x136100 and
//! `Switch_Create` 0x1adcc0 read the frame TIME (GameState+0x34 DECIMAL 52, `GameState._52_4_`),
//! `Car_Create` 0x1305b0 / `Drone_AIVolume_Create` 0x1376c0 / `Env_Tree_Create` 0x17c200 read the
//! mission word (GameState+0xC DECIMAL 12), the spawner path has no gate at create
//! (`DroneSpawner_Create` 0x1378d0 copies the placement bytes; `DroneSpawner_SpawnDrone` 0x137cb0
//! reads neither the global nor a gate), and no pickup/ammo/turret/camera creator
//! (`GunImp_Create`, `ThirdCam_Create`, `CamSubject_Create`, `Searchlight_Create`, `Monitor_Create`)
//! reads the difficulty at all — `Pickup_Create`'s kind-6 gate is the medal word
//! (`PlrStats_HasGoldMedal` 0x199a40), not the difficulty.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

/// The on-foot levels (the walkreal_*_live set; 0700004b is the driving level, no walk).
const LEVELS: &[&str] = &[
    "07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007",
    "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d", "07000011",
    "07000012", "07000013", "07000014", "07000015", "07000016", "0700001b", "07000041",
    "0700004a",
];

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_level(iso: &str, archive: &str) -> Level {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    let bytes = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    Level::parse(&bytes).expect("parse")
}

/// The behaviour-property tables, read once per test from the disc's ACTION.ELF symbol table —
/// the same [D] source the load path uses (nf-view lib.rs's DroneContext.props) and the same
/// construction (`PropertyTables::from_action_elf`).
fn tables(iso: &str) -> Option<nf_formats::drone::PropertyTables> {
    let img = IsoImage::open(iso).expect("open iso");
    let entry = img.find("/ACTION.ELF")?;
    let elf = img.read_file(entry).ok()?;
    nf_formats::drone::PropertyTables::from_action_elf(&elf).ok()
}

fn param_of(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

/// `Hint_Create`'s stage law, verbatim [M 0x1b3248:12-25] (the same words
/// `SoundTrigger_Create` 0x1b2628:24-40 reads at param 6).
fn stage_gate_passes(stage: u32, difficulty: u32) -> bool {
    if stage != 0 && stage < difficulty { return false; }
    if stage == 5 && difficulty != 1 { return false; }
    if stage == 6 && difficulty != 2 { return false; }
    if stage == 7 && difficulty != 3 { return false; }
    true
}

/// The [D]×[M] expected set for one level at one difficulty, computed straight from the disc's
/// placements — independent of the port's creators (which the load path then must reproduce).
struct Expected {
    drones: Vec<usize>,          // main-map placement indices that spawn [M 0x136be0:49]
    captain: Vec<bool>,          // per spawned drone, the 0x14b300:504-535 criterion
    locks: Vec<(usize, bool)>,   // (main-map placement index, starts unlocked) [M 0x1ae1f8:67-69]
    sound_triggers: Vec<[f32; 3]>, // kind-0xF9 survivors' translations [M 0x1b2628:24-40]
    hints: Vec<(u32, u32)>,      // kind-0x33 survivors' (label, stage) [M 0x1b3248:12-25]
    copters: Vec<(usize, f32)>,  // kind-0x2F kind-0 placements and their HP [M 0x17d340:76-94]
}

fn expected(level: &Level, level_word: u32, difficulty: u32, tables: Option<&nf_formats::drone::PropertyTables>) -> Expected {
    let mm = level.main_map().unwrap_or(0);
    let placements = &level.maps[mm].placements;
    let mut e = Expected { drones: Vec::new(), captain: Vec::new(), locks: Vec::new(),
        sound_triggers: Vec::new(), hints: Vec::new(), copters: Vec::new() };
    for (i, p) in placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        if kind == 0x0f {
            // Drone_Create's pre-body gate: no spawn (and no RNG draw) when param2 > difficulty [M 0x136be0:49].
            if param_of(p, 2) > difficulty { continue; }
            e.drones.push(i);
            // NDrone2_DefaultInit's captain criterion [M 0x14b300:504-535]: property 9 on d1,
            // 5 on d2, 8 on d3 — read through the disc's behaviour tables. The flag survives
            // only on the dtypes the skin switch maps [M 0x14b300:536-556: cases 1/2/3/6/7 swap
            // the skin, `default: +0x19 = 0` CLEARS the captain flag] — the kind gate comes from
            // the [D] skin table through `drone_setup` (the 0x07000014 skin rewrite included).
            let words = nf_formats::drone::behaviour_words(&|k| param_of(p, k)).0;
            let has = |id: usize| tables.map_or(false, |t| t.get(id, &words) != 0);
            let setup = nf_formats::drone::drone_setup(param_of(p, 0), param_of(p, 9), level_word);
            let mapped = matches!(setup.kind, 1 | 2 | 3 | 6 | 7);
            e.captain.push(mapped && ((has(9) && difficulty == 1) || (has(5) && difficulty == 2) || (has(8) && difficulty == 3)));
        } else if kind == 0x2e {
            // Lock_Create: param 6 non-zero + Operative → starts unlocked [M 0x1ae1f8:67-69].
            e.locks.push((i, param_of(p, 6) != 0 && difficulty == 1));
        } else if kind == 0x2f && param_of(p, 0) == 0 {
            // Copter_Create: a non-zero kind builds no copter [M 0x17d340:43-45]; HP × mult [M :76-94].
            let mult = match difficulty { 2 => 1.5, 3 => 2.0, 4 => 4.0, _ => 1.0 };
            e.copters.push((i, mult * 299.0));
        } else if kind == 0xf9 {
            // SoundTrigger_Create's own refusals [M 0x1b2628:14-40]: the id word (+0x34, param 2)
            // must be non-0 and non-0xffff, the on word (+0x2c, param 0) non-0, then the gate.
            let (on, id, gate) = (param_of(p, 0), param_of(p, 2), param_of(p, 6));
            if id == 0 || id == 0xffff || on == 0 { continue; }
            if !stage_gate_passes(gate, difficulty) { continue; }
            e.sound_triggers.push(p.translation);
        }
    }
    // Hints over every map [M Hint_Create 0x1b3248 via parsemap_create_dynamic_objects], the
    // stage law on param 2, the placement resolved against its map object (an unresolved
    // placement builds no object).
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            if p.flags & 0xff != 0x33 { continue; }
            if !stage_gate_passes(param_of(p, 2), difficulty) { continue; }
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let obj = obj.and_then(|(om, oi)| level.maps.get(om).and_then(|map| map.objects.get(oi)));
            if obj.is_some() { e.hints.push((param_of(p, 0), param_of(p, 2))); }
        }
    }
    e
}

fn near(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
}

fn check_level(level_name: &str, difficulty: u32, tables: Option<&nf_formats::drone::PropertyTables>) -> Result<(), String> {
    let Some(iso) = iso() else { eprintln!("skipping: NF_ISO not set"); return Ok(()) };
    let level = read_level(&iso, &format!("{level_name}.bin"));
    let loaded = nf_view::load_with_carry_difficulty(&iso, &format!("{level_name}.bin"), false, None, difficulty)
        .unwrap_or_else(|e| panic!("{level_name} d{difficulty}: load failed: {e}"));
    assert_eq!(loaded.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "{level_name}: the sim runs the picked difficulty [M GameState+0x28]");
    let e = expected(&level, u32::from_str_radix(&level_name[..8], 16).unwrap_or(0), difficulty, tables);
    let sim = loaded.sim.as_ref().unwrap();

    // The drone roster: the spawned placement-index SET must equal the filtered [D] set, and the
    // captain flags must equal the 0x14b300 criterion.
    let mut actual: Vec<usize> = sim.drones.list.iter().map(|d| d.placement).collect();
    actual.sort();
    let mut want = e.drones.clone();
    want.sort();
    assert_eq!(actual, want, "{level_name} d{difficulty}: the spawned drone set");
    assert_eq!(sim.drones.list.len(), e.captain.len(), "{level_name} d{difficulty}: captain-vector length");
    for (d, &want_cap) in sim.drones.list.iter().zip(&e.captain) {
        assert_eq!(d.captain, want_cap,
            "{level_name} d{difficulty}: drone on placement {} captain flag (skin {:#x})", d.placement, d.skin);
    }

    // The locks: same set, and the Operative create-unlock exactly on the param-6 tags.
    let mut actual_locks: Vec<(usize, bool)> = sim.drones.locks.iter().map(|l| (l.placement, l.unlocked)).collect();
    actual_locks.sort();
    let mut want_locks = e.locks.clone();
    want_locks.sort();
    assert_eq!(actual_locks, want_locks, "{level_name} d{difficulty}: the locked/unlocked set");

    // The sound triggers: same survivor set (positions copied verbatim from the placements).
    assert_eq!(sim.drones.sound_triggers.list.len(), e.sound_triggers.len(),
        "{level_name} d{difficulty}: the sound-trigger count");
    for want_pos in &e.sound_triggers {
        assert!(sim.drones.sound_triggers.list.iter().any(|t| near(t.pos, *want_pos)),
            "{level_name} d{difficulty}: sound trigger at {want_pos:?} missing");
    }

    // The copters: same set and the difficulty HP law.
    let mut actual_copters: Vec<(usize, u32)> = sim.drones.copters.iter()
        .map(|c| (c.placement, c.hp as u32)).collect();
    actual_copters.sort();
    let mut want_copters: Vec<(usize, u32)> = e.copters.iter().map(|&(i, hp)| (i, hp as u32)).collect();
    want_copters.sort();
    assert_eq!(actual_copters, want_copters, "{level_name} d{difficulty}: the copter set and HP");

    // The hints: same (label, stage) multiset.
    let hints = nf_game::player::hints(&level, difficulty);
    let mut actual_hints: Vec<(u32, u32)> = hints.iter().map(|h| (h.label, h.stage)).collect();
    actual_hints.sort();
    let mut want_hints = e.hints.clone();
    want_hints.sort();
    assert_eq!(actual_hints, want_hints, "{level_name} d{difficulty}: the hint set");
    Ok(())
}

/// Operative (difficulty 1): every placement whose min-difficulty tag is 0 or 1 spawns; the
/// param-6-tagged locks all start unlocked; the gate-5 sound triggers/hints fire.
#[test]
fn the_operative_sets_match_the_difficulty_rules() {
    let Some(iso) = iso() else { eprintln!("skipping: NF_ISO not set"); return };
    let t = tables(&iso);
    for level in LEVELS { check_level(level, 1, t.as_ref()).unwrap(); }
}

/// Agent (difficulty 2): the param-2-tagged drones join, the locks start LOCKED, the gate-6
/// markers fire instead of the gate-5 ones, the copter HP is ×1.5.
#[test]
fn the_agent_sets_match_the_difficulty_rules() {
    let Some(iso) = iso() else { eprintln!("skipping: NF_ISO not set"); return };
    let t = tables(&iso);
    for level in LEVELS { check_level(level, 2, t.as_ref()).unwrap(); }
}

/// 00 Agent (difficulty 3): the param-2 AND param-3 drones all spawn, the prop-8 captains
/// replace the prop-5 ones, the copter HP is ×2.0.
#[test]
fn the_00_agent_sets_match_the_difficulty_rules() {
    let Some(iso) = iso() else { eprintln!("skipping: NF_ISO not set"); return };
    let t = tables(&iso);
    for level in LEVELS { check_level(level, 3, t.as_ref()).unwrap(); }
}
