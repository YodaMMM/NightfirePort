//! gold-medal-realpath: the G-029 weapon upgrades reach Bond's kit through the app's own path,
//! not a headless `--bonus`-style load.
//!
//! The path: a codename saved on the port's memory card (`<config dir>/<name>.profile`, the
//! PLRS/MSSN/MPSG/GSET/CHET/BNUS chain, docs/systems/profile-session.md) whose bonus word holds
//! gold medals, selected from the boot through the menus the window drives (title → Main →
//! Select Codename (row 0 New, 1 Play without saving, 2+ the card) → the card row → Difficulty → the mission list → the mission), then the level
//! load the window runs for that pick (`load_front_end` with the front end's `bonus_bits()` [M
//! lib.rs's pickup-medal-gate line: the profile's `bonus[0]` rides every front-end load]).
//!
//! The medals: `Menu_SetLevelBonus` 0x1ffa10 (`Progress::set_level_bonus`, medal 3 = gold sets
//! counters 3, 2, 1) on every one of the 12 missions. The upgrade row that word rebuilds
//! (`Menu_SetBonus` 0x1ff898, `progress::upgrades_from_bonus`) is [3, 1, 1, 2, 1, 1, 1, 1, 1]:
//! handgun gold group bits 0..2 (07000005, 07000009, 07000011) → `UpgradedHandguns[3]` = 8;
//! the sniper group bits 9..10 (07000001, 0700001B) → 0x1e → 0x22; the stunner bit 18
//! (09000003) → 0x4a → 0x4c; the dart gun bit 12 (0700000C) → 0x43 → 0x44; the decryptor bit 15
//! (09000002) → 0x56 → 0x57 [M `Upgrade_Weapon` 0x1a8f30's jump table, docs/systems/pickups.md].
//!
//! The control walk uses the same boot and card but picks Play without saving (bonus 0): the base kit.
//!
//! `real_exe_gold_kit` (ignored, opt-in, `NF_REAL_EXE=<exe>`) runs the built exe in a window with
//! `NF_CONFIG_DIR` pointing at a throwaway card and `NF_MENU_SCRIPT` presses, and reads its
//! `loadout:` line. It never touches the player's real card: the card dir is a fresh temp dir.

use nf_view::menu::{Action, CAMPAIGN, FrontEnd, Page};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The throwaway codename: gold (and so silver and bronze) on all 12 missions.
const NAME: &str = "GOLDALL";

fn gold_profile() -> nf_game::profile::Profile {
    let mut p = nf_game::profile::Profile::default_codename(NAME);
    for &h in nf_game::progress::SP_LEVEL_HASHES.iter() {
        p.progress.set_level_bonus(h, 3, 0);
    }
    p
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("nf-gold-realpath-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// One action, then the frames a player would wait (create_save_live's harness).
fn act(fe: &mut FrontEnd, a: Action) -> bool {
    let r = fe.action(a);
    for _ in 0..60 { fe.tick(); }
    for _ in 0..5000 {
        if !fe.intro_playing && !fe.attract_playing { break; }
        fe.tick();
    }
    for _ in 0..24 {
        if !fe.close_iris_in_flight() { break; }
        fe.tick();
    }
    r
}

/// The boot through the menus to the picked mission's request. `card_row` = the Select Codename
/// row (1 = Play without saving, 2 = the first card codename [M Menu_EnumSaves 0x1f7b78's row order]).
/// Returns (level, difficulty, bonus word riding the load).
fn walk_to_mission(dir: &std::path::Path, card_row: usize, level: u32) -> (u32, u32, u64) {
    let mut fe = FrontEnd::new();
    nf_view::menu::ProfileSession { dir: Some(dir.to_path_buf()), name: None, profile: None, session_progress: None }
        .apply_to(&mut fe);
    assert_eq!(fe.page, Page::Start);
    assert!(act(&mut fe, Action::Confirm), "the title's Enter");
    assert_eq!(fe.page, Page::Main);
    assert!(act(&mut fe, Action::Confirm), "Main row 0 → Select Codename");
    assert_eq!(fe.page, Page::Codename);
    assert_eq!(fe.cursor, 1, "the page opens on row 1");
    while fe.cursor < card_row { assert!(act(&mut fe, Action::Down)); }
    assert!(act(&mut fe, Action::Confirm), "the codename row loads");
    // Row 1 "Play without saving" pops its confirm box first [M C_SBNFCN 0x20b390:19-21].
    if card_row == 1 { assert!(act(&mut fe, Action::Confirm), "the box's confirm"); }
    assert_eq!(fe.page, Page::Difficulty);
    assert!(act(&mut fe, Action::Confirm), "the difficulty confirms");
    let difficulty = fe.requested_difficulty.take().expect("the difficulty pick");
    assert_eq!(fe.page, Page::MissionSelect);
    let row = CAMPAIGN.iter().position(|c| c.3 == level).expect("the mission row");
    let start = fe.cursor;
    for _ in 0..CAMPAIGN.len() {
        if fe.cursor == row { break; }
        let a = if row > fe.cursor { Action::Down } else { Action::Up };
        act(&mut fe, a);
    }
    assert_eq!(fe.cursor, row, "the wheel reached the row (opened on {start})");
    assert!(fe.unlocked_row(row), "row {row} is unlocked");
    assert!(act(&mut fe, Action::Confirm), "the mission confirms");
    let req = fe.requested_level.take().expect("the mission start rides the request");
    (req, difficulty, fe.bonus_bits())
}

/// The kit the window's load builds for that pick: (switch request, owned ids).
fn kit(iso: &str, level: u32, difficulty: u32, bonus: u64) -> (u8, Vec<usize>) {
    let name = nf_view::level_archive_name(level);
    let sc = nf_view::load_front_end(iso, &name, false, None, false, difficulty, bonus)
        .unwrap_or_else(|e| panic!("{level:08x}: {e}"));
    let sim = sc.sim.as_ref().expect("the sim");
    let owned = sim.drones.inventory.owned.iter().enumerate().filter(|(_, &o)| o).map(|(i, _)| i).collect();
    (sim.gun.switch_request, owned)
}

#[test]
fn gold_codename_walk_upgrades_the_kit() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let dir = temp_dir("walk");
    gold_profile().save_to_dir(&dir).expect("the card file writes");
    // The weapon bytes `Upgrade_Weapon` reads: [0] handguns 3 (three golds), [2] the 0x54 zoom,
    // [3] snipers 2 (two golds), [4] dart gun, [5] decryptor, [6] stunner, [7] laser.
    let row = nf_game::progress::upgrades_from_bonus(gold_profile().progress.bonus[0]);
    assert_eq!((row[0], row[2], row[3], row[4], row[5], row[6], row[7]), (3, 1, 2, 1, 1, 1, 1), "{row:?}");

    // The Exchange: the handgun (request) and the stunner.
    let (lv, d, bonus) = walk_to_mission(&dir, 2, 0x0700_0005);
    assert_eq!(lv, 0x0700_0005);
    assert_eq!(bonus, gold_profile().progress.bonus[0], "the card's bonus word rides the load");
    let (req, owned) = kit(&iso, lv, d, bonus);
    eprintln!("gold 07000005: request {req:#x} owned {owned:x?}");
    assert_eq!(req, 8, "the level-3 handgun is drawn [M UpgradedHandguns[3]]");
    assert!(owned.contains(&8) && !owned.contains(&2), "the upgraded handgun replaces the base one");
    assert!(owned.contains(&0x4c) && !owned.contains(&0x4a), "the upgraded stunner");

    // Chain Reaction: the sniper (request), the stunner.
    let (lv, d, bonus) = walk_to_mission(&dir, 2, 0x0700_000c);
    let (req, owned) = kit(&iso, lv, d, bonus);
    eprintln!("gold 0700000c: request {req:#x} owned {owned:x?}");
    assert_eq!(req, 0x22, "the level-2 sniper is drawn");
    assert!(owned.contains(&8) && owned.contains(&0x22) && owned.contains(&0x4c));

    // Night Shift: the dart gun (request) and the decryptor.
    let (lv, d, bonus) = walk_to_mission(&dir, 2, 0x0700_0009);
    let (req, owned) = kit(&iso, lv, d, bonus);
    eprintln!("gold 07000009: request {req:#x} owned {owned:x?}");
    assert_eq!(req, 0x44, "the upgraded dart gun is drawn");
    assert!(owned.contains(&0x57) && !owned.contains(&0x56), "the upgraded decryptor");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn play_without_saving_walk_keeps_the_base_kit() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let dir = temp_dir("default");
    // The same card holds the gold codename; the walk picks Play without saving (row 1), bonus 0.
    gold_profile().save_to_dir(&dir).expect("the card file writes");
    let (lv, d, bonus) = walk_to_mission(&dir, 1, 0x0700_0005);
    assert_eq!(bonus, 0);
    let (req, owned) = kit(&iso, lv, d, bonus);
    eprintln!("no-save 07000005: request {req:#x} owned {owned:x?}");
    assert_eq!(req, 2);
    assert!(owned.contains(&2) && owned.contains(&0x4a) && !owned.contains(&8));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Helper for audits/tools-wip/gold-medal-realpath.sh: write the gold codename into
/// `NF_GOLD_DIR` (a throwaway card dir; refused when it already holds a settings file, so the
/// player's own config dir is never written).
#[test]
#[ignore]
fn write_gold_profile() {
    let Some(dir) = std::env::var_os("NF_GOLD_DIR").map(std::path::PathBuf::from) else { panic!("NF_GOLD_DIR not set") };
    assert!(!dir.join("settings.toml").exists(), "{} looks like a real config dir", dir.display());
    let path = gold_profile().save_to_dir(&dir).expect("the card file writes");
    println!("wrote {} bonus {:#018x}", path.display(), gold_profile().progress.bonus[0]);
}

/// Opt-in: the built exe, a window, the menus pressed by NF_MENU_SCRIPT, the card in a temp dir.
#[test]
#[ignore]
fn real_exe_gold_kit() {
    let Some(exe) = std::env::var_os("NF_REAL_EXE") else { panic!("NF_REAL_EXE not set") };
    let Some(iso) = iso() else { panic!("NF_ISO not set") };
    let dir = temp_dir("exe");
    let card = dir.join("card");
    gold_profile().save_to_dir(&card).expect("the card file writes");
    // Title, Main → Select Codename (opens on row 1), down to the card row, load →
    // Difficulty → the mission list (opens on the highest unlocked row, Equinox; two downs wrap
    // to The Exchange), confirm, skip the briefing, then the capture once in the level.
    let script = "0:page=Start,500:confirm,0:page=EstHero,4000:confirm,0:page=Main,1200:confirm,0:page=Codename,\
                  800:down,800:confirm,0:page=Difficulty,800:confirm,0:page=MissionSelect,1500:down,800:down,\
                  1500:confirm,2500:confirm,0:page=None,4000:shot,500:sleep";
    let out = std::process::Command::new(&exe)
        .arg(&iso).arg("--game")
        .current_dir(&dir)
        .env("NF_SILENT", "1").env("NF_CONFIG_DIR", &card).env("NF_MENU_SCRIPT", script)
        .stdin(std::process::Stdio::null())
        .output_with_timeout(std::time::Duration::from_secs(150));
    let lines: Vec<&str> = out.lines().filter(|l| l.starts_with("loadout:") || l.contains("[spot]")).collect();
    for l in &lines { eprintln!("{l}"); }
    let lo = lines.iter().find(|l| l.starts_with("loadout: level 07000005")).expect("the level loaded from the menus");
    assert!(lo.contains("switch request 8 "), "the upgraded handgun is the request: {lo}");
}

trait RunFor { fn output_with_timeout(&mut self, t: std::time::Duration) -> String; }
impl RunFor for std::process::Command {
    /// Run, kill at `t`, return stderr + stdout (the app logs to stderr and never exits on its own).
    fn output_with_timeout(&mut self, t: std::time::Duration) -> String {
        let mut child = self.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().expect("spawn the exe");
        let err = child.stderr.take().unwrap();
        let reader = std::thread::spawn(move || { let mut s = String::new(); let _ = std::io::Read::read_to_string(&mut { err }, &mut s); s });
        let start = std::time::Instant::now();
        while start.elapsed() < t {
            if let Ok(Some(_)) = child.try_wait() { break; }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        let _ = child.kill();
        let _ = child.wait();
        reader.join().unwrap_or_default()
    }
}
