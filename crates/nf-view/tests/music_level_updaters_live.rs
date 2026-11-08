//! The six level music updaters behind the shared exit block, through the real load and tick
//! (opt-in via `NF_ISO`, silent): `UpdateMFX_02MayhewA` 0x111ed8 (07000001), `02MayhewD` 0x112bf8
//! (07000004), `03TowerA` 0x113558 (07000009), `03TowerB` 0x1137b0 (0700000a), `PowerStationA1`
//! 0x113c18 (0700000c), `PowerStationA2` 0x113fc0 (0700000d). Before they were ported these levels
//! had the exit sting only: the music never left the start section, alarm or not.

use nf_formats::iso9660::IsoImage;
use nf_game::music::Cmd;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn step(sim: &mut nf_view::scene::Sim) { sim.step_bond(Default::default(), false, false, false); }

fn jumps(sim: &nf_view::scene::Sim) -> Vec<i32> {
    sim.drones.music.out.iter().filter_map(|c| match c { Cmd::Jump { section, .. } => Some(*section), _ => None }).collect()
}

/// Boots `level` and runs it, skipping scenes, until Bond is free. `Sim::step_bond` queues the
/// music commands onto `music.out` and nothing drains them headless, so `out` is the whole log.
fn boot(path: &str, level: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(path, level, false).unwrap_or_else(|e| panic!("load {level}: {e}"));
    let sim = scene.sim.as_mut().unwrap();
    let mut n = 0;
    while n < 6000 {
        if sim.drones.scripts.cutscene_playing() { sim.drones.scripts.skip_request = true; }
        step(sim);
        n += 1;
        if n > 120 && !sim.disabled { break; }
    }
    assert!(!sim.disabled, "{level}: Bond is free ({n} ticks)");
    scene
}

/// `Music_Event(2, 5)` long enough for the alert count to run out: the board write
/// `NDrone2_SeenAndAttacking` makes each tick a guard engages [M 0x145150:68].
fn engage(sim: &mut nf_view::scene::Sim) {
    for _ in 0..30 { sim.drones.music.event(2, 5); step(sim); }
}

/// The rows the machines read, off the disc [D MapMusic 0x24f4d8], and the alarm row.
#[test]
fn the_rows_come_from_the_disc() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let img = IsoImage::open(&path).expect("open iso");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("no ACTION.ELF")).expect("read elf");
    let (mm, vars) = nf_formats::music::tables(&elf).expect("tables parse");
    let row = |lv: u32| mm.iter().find(|m| m.level == lv).expect("row").clone();
    let sets = |lv: u32| { let r = row(lv); (r.track, r.start, r.combat, r.stealth) };
    assert_eq!(sets(0x0700_0001), (19, 0, vec![3, 11, 14], vec![5, 7, 9]));
    assert_eq!(sets(0x0700_0004), (22, 0, vec![], vec![]));
    assert_eq!(sets(0x0700_0009), (23, 0, vec![10, 8, 6], vec![12, 2]));
    assert_eq!(sets(0x0700_000a), (23, 26, vec![8, 6, 10], vec![19, 14]));
    assert_eq!(sets(0x0700_000c), (36, 0, vec![7, 9, 10], vec![0, 15, 19, 0, 0]));
    assert_eq!(sets(0x0700_000d), (36, 32, vec![41, 43], vec![]));
    let alarm = vars.iter().find(|v| v.level as i32 == -99).expect("the -99 row");
    assert_eq!((alarm.count, alarm.floor, alarm.window, alarm.distance), (1, 0, 10, 50.0));
}

/// 07000001: the intro's end opens section 3 [M 0x111ed8:41, 0x1d9860:74].
#[test]
fn the_01_intro_end_starts_section_3() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = boot(&path, "07000001.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(jumps(sim).contains(&3), "the opening section: {:?}", sim.drones.music.out);
}

/// 07000009: a guard fight takes the music into the combat set, and it comes back out.
#[test]
fn the_09_fight_reaches_combat_music() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = boot(&path, "07000009.bin");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.music.out.clear();
    for _ in 0..60 { step(sim); }
    assert!(!jumps(sim).iter().any(|s| [10, 8, 6].contains(s)), "sister: quiet level, no combat section: {:?}", sim.drones.music.out);
    engage(sim);
    assert!(jumps(sim).contains(&10), "the first combat section: {:?}", sim.drones.music.out);
}

/// 0700000a and 0700000c: the same, on their own combat sets.
#[test]
fn the_0a_and_0c_fights_reach_combat_music() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    for (level, first) in [("0700000a.bin", 8), ("0700000c.bin", 7)] {
        let mut scene = boot(&path, level);
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.music.out.clear();
        engage(sim);
        assert!(jumps(sim).contains(&first), "{level}: {:?}", sim.drones.music.out);
    }
}

/// 0700000d: the intro's end jumps 0x2b [M 0x113fc0:42-46] and the machine wakes.
#[test]
fn the_0d_intro_end_starts_section_0x2b() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = boot(&path, "0700000d.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(jumps(sim).contains(&0x2b), "the opening section: {:?}", sim.drones.music.out);
}

/// 0700000c's trigger-driven creep event: a real `Music_Event(0xd, 1)` from the channel-13 music
/// trigger placement (ch 36 → id 13 [D]) reaches the updater's creep cue.
#[test]
fn the_0c_music_trigger_reaches_the_creep_cue() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = boot(&path, "0700000c.bin");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.music.out.clear();
    sim.drones.set_channel(36);
    for _ in 0..5 { step(sim); }
    assert!(jumps(sim).contains(&0x22), "{:?}", sim.drones.music.out);
}

/// `DroneFunc_CheckAlarmRaised` 0x1453a8: the alarm channel 0x61 posts the combat event on EVERY
/// level for its first ten seconds [M `LAB_001454d8`], not just 05/06. Sister: quiet before.
#[test]
fn the_alarm_channel_posts_the_combat_event_outside_the_castle() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = boot(&path, "07000009.bin");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..5 { step(sim); }
    assert_eq!(sim.drones.prev_event2, 0, "no alarm yet");
    sim.drones.set_channel(0x61);
    let mut seen = vec![];
    for _ in 0..5 { step(sim); seen.push((sim.drones.prev_event2, sim.drones.channels.on(0x61), sim.disabled)); }
    assert!(seen.iter().any(|s| s.0 == 1), "the combat event: {seen:?}");
}

/// The `Sound_Play` sites the port had as 3-D: the objective chimes are 2-D [M 0x19c310:221,233].
#[test]
fn the_objective_chimes_are_2d() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = boot(&path, "07000005.bin");
    let sim = scene.sim.as_mut().unwrap();
    // A fresh level announces its objectives; run the banner through its flash and steady arms.
    for _ in 0..1500 { step(sim); }
    let is_chime = |id: &u32| matches!(*id, 1532 | 469 | 468);
    assert!(sim.drones.audio.live_2d_ids().iter().any(is_chime), "2-D: {:?}", sim.drones.audio.live_ids());
    // Sister: none of them went out as 3-D.
    let all = sim.drones.audio.live_ids().into_iter().filter(is_chime).count();
    assert_eq!(all, sim.drones.audio.live_2d_ids().into_iter().filter(is_chime).count());
}
