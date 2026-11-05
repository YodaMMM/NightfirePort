//! saved-records-across-parts: walk "Night Shift" — the mission of 07000009 → 0700000a →
//! 0700000b [D mission::LEVELS' parts] — through the App's own pieces: the front end (the
//! codename select → the difficulty → the mission list → the results/fail pages), the App's
//! level loads (`load_with_carry_medals` — the loader every menu pick and part transition
//! takes [M lib.rs menu_after]) and the App's swap seeding (the four-field copy at the scene
//! swap [M lib.rs swap_in_scene]) — and pin that the saved record (the ScoringTable, the
//! bonus word and the game-complete flag) survives EVERY part load and play, the fail menu's
//! REPLAY, and the fail menu's QUIT (which must leave the saved file byte-identical).
//!
//! The fail path is the game's own: the fail verdict (ch 99 [M 0x182c60:77-81]) opens menu
//! 0x40000042 over the paused mission — no level request, no fade [M 0x182c60:77-80] — whose
//! rows are REPLAY = 0, QUIT = 2 [L page 0x40000042; M P_ENDMISSION_Handler 0x216618:19-27].
//! Row 0 reloads the CURRENT level WARM on the entry checkpoint (`ResetMap_LevelToLoad(
//! GameState._12_4_, 1, 1)` + `Player_RamLoad(Bond, 1)` [M 0x216618:19-21]); row 2 goes to the
//! front end's mission list (+0x1c = 0x4000001c, ResetMap_LevelToLoad(0x7000048, 0, 0) [M
//! 0x216618:24-26]) with no save of any kind — the game writes the card only from the results
//! save prompt's chain [M 0x208e78:24-32 + Menu_UpdateMessageBox case 1].

use nf_game::mission::{self, Event};
use nf_game::player::RamSave;
use nf_view::menu::{self, Action, CAMPAIGN, FrontEnd, Outcome, Page, Stats};
use nf_view::scene::Scene;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The game's strings [M Txt_LoadLanguage 0x1caad0 loads USATxt.dat out of FILES.BIN].
fn chain_iso_lang(iso_path: &str) -> Option<nf_formats::hud::Lang> {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).ok()?;
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat"))?;
    let bytes = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
    nf_formats::hud::Lang::parse(&bytes).ok()
}

/// The saved record the journey pins: the values a real prior-session save would hold — the
/// campaign finished once (the game-complete flag [M GameState[0x52], Menu_SetLevelBonus
/// 0x1ffa10's tail sets it on 0x0700001b]), the bonus word's medal bits [M bonus[] 0x2e1b88,
/// LS_LoadBonus 0x1ce488 restores it] and a prior mission's best score in the table [M
/// ScoringTable 0x2bc640, the commit 0x1984b0:301-315].
const BONUS_MARKER: u64 = 0xC000_0000_0000_0000;
const TABLE_BEST_MARKER: u32 = 0x00C0_FEE0;

fn assert_record(progress: &nf_game::progress::Progress, what: &str) {
    assert_eq!(progress.game_complete, true, "{what}: the game-complete flag dropped");
    assert_eq!(progress.bonus[0], BONUS_MARKER, "{what}: the bonus word dropped");
    assert_eq!(progress.table[0].best, TABLE_BEST_MARKER, "{what}: the saved table dropped");
}

/// The App's swap seeding [M lib.rs swap_in_scene]: every fresh sim's Progress layer takes the
/// saved record's four fields — the same copy the App runs at every scene swap (the game keeps
/// ONE set of globals; the port splits them across the view and the sim).
fn seed_from_saved(progress: &mut nf_game::progress::Progress, saved: &nf_game::progress::Progress) {
    (progress.table, progress.bonus, progress.nightfire_status, progress.game_complete) =
        (saved.table.clone(), saved.bonus, saved.nightfire_status, saved.game_complete);
}

/// One part load through the App's own loader [M lib.rs menu_after → load_with_carry_medals],
/// then the swap seeding. Returns the scene with the record already seeded.
fn load_part(iso: &str, level: u32, save: Option<&RamSave>, difficulty: u32, bonus: u64,
             saved: &nf_game::progress::Progress) -> Scene {
    let name = nf_view::level_archive_name(level);
    let mut sc: Scene = nf_view::load_with_carry_medals(iso, &name, false, save, difficulty, bonus)
        .unwrap_or_else(|e| panic!("0x{level:08x}: load error: {e}"));
    let sim = sc.sim.as_mut().expect("the sim");
    seed_from_saved(&mut sim.drones.progress, saved);
    // The runner's forced verdicts ride the warm capture — a real warm entry never finds them
    // ON (the verdict arms end the level themselves — complete → the outro codes, fail → the
    // 0x40000042 menu — with the player paused, so no exit E can follow [M 0x182c60:77-135];
    // the campaign-flow runner's own warm-entry note). Bring the channels to what a real level
    // entry finds, before the mission layer's first tick.
    if save.is_some() {
        let f = sim.drones.frame;
        sim.drones.channels.set(100, 0, f);
        sim.drones.channels.set(99, 0, f);
    }
    sc
}

/// Fire a part's objectives the way the level's own triggers and scripts would (the objective
/// channels are the same switch channels [M 0x1827d8]) — the campaign-flow runner's fallback.
/// `verdict` = 100 (the mission-complete AND [M 0x1827d8:119-123]) or 99 (the fail verdict
/// [M 0x182c60:77-81]).
fn force_part(sc: &mut Scene, level: u32, verdict: u16) {
    let sim = sc.sim.as_mut().expect("the sim");
    let idx = mission::LEVELS.iter().position(|l| l.level == level).expect("the level table");
    for o in mission::LEVELS[idx].objectives {
        if o.flags & 2 != 0 { continue; } // the FAIL flag is record+0x10 bit 1 [M 0x1827d8:32-38]
        if o.channel == 100 || o.channel == 99 { continue; } // never fire a verdict as a stand-in
        if o.enable != 0 { sim.drones.set_channel(o.enable); }
        if o.channel != 0 { sim.drones.set_channel(o.channel); }
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    }
    sim.drones.set_channel(verdict as u8);
}

#[test]
fn night_shift_parts_keep_the_saved_record_through_the_fail_menu() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let lang = chain_iso_lang(&iso);
    // The card + the codename [M the create chain: the keyboard validates, the default template
    // maps (fresh unlocks 0x3 [M 0x2023a0]), Menu_UpdateMessageBox case 1 writes the chain].
    let dir = std::env::temp_dir().join(format!("nf-saved-records-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    fe.create_codename("SAVED01", &[]).expect("create the codename");
    // The prior-session stand-in: the file a real save would hold — rows 0..=5 beaten, the
    // campaign finished once, the medal bits and a prior best in the table.
    {
        let mut p = fe.profile.clone().expect("the codename's profile");
        p.progress.set_nightfire_status(0b11_1111);
        p.progress.game_complete = true;
        p.progress.bonus[0] = BONUS_MARKER;
        p.progress.table[0].best = TABLE_BEST_MARKER;
        p.save_to_dir(&dir).expect("the stand-in save writes");
    }
    // Select the codename the App's way: the list row 2 (the first card row [M Menu_EnumSaves
    // 0x1f7b78's row order — the two defaults, then the card's names]) loads it [M
    // Menu_UpdateMessageBox case 0 → LS_Load 0x1cdfc0] and the MSSN status IS the unlock mask
    // [M Menu_SetNightfireStatus 0x2023a0 unpacks it into sp_level[i]+0x10].
    assert!(fe.load_codename(2), "the codename loads");
    assert_eq!(fe.page, Page::Difficulty);
    let saved0 = fe.saved_progress().cloned().expect("the loaded record");
    assert_record(&saved0, "the loaded codename");

    // The difficulty wheel (cursor 1 = Agent [M the wheel starts centred]) confirms → the SP
    // list [M C_SBNFDFCTY 0x216ca8:10-17].
    assert!(fe.action(Action::Confirm), "the difficulty confirms");
    let difficulty = fe.requested_difficulty.take().expect("the pick rides the request [M 0x216ca8:15]");
    // G-048(1): the list opens through the delayed 0x44 — the input block holds until it lands
    // [M 0x202190:7; the port's delayed_page resolve], so the list pick waits the iris out.
    for _ in 0..24 {
        fe.tick();
        if !fe.close_iris_in_flight() { break; }
    }
    assert_eq!(fe.page, Page::MissionSelect);

    // The mission list: Night Shift's row confirms → the mission start rides the request
    // [M C_SBNFMAP 0x20b8d8:18-41; the lock gate :21-23].
    let row_ns = CAMPAIGN.iter().position(|c| c.3 == 0x0700_0009).expect("Night Shift's row");
    assert!(fe.unlocked_row(row_ns), "the prior saves unlocked Night Shift");
    fe.cursor = row_ns;
    assert!(fe.action(Action::Confirm), "the mission row confirms");
    assert_eq!(fe.requested_level.take(), Some(0x0700_0009), "the mission start rides the request");

    // The mission's parts, in part order [D mission::LEVELS].
    let m = mission::LEVELS.iter().find(|l| l.level == 0x0700_0009).expect("the level").mission;
    let mut parts: Vec<(u32, u32)> = mission::LEVELS.iter()
        .filter(|l| l.mission == m).map(|l| (l.part, l.level)).collect();
    parts.sort_by_key(|&(p, _)| p);
    let parts: Vec<u32> = parts.into_iter().map(|(_, l)| l).collect();
    assert!(parts.len() >= 2, "the walked mission has several parts: {parts:?}");
    assert_eq!(parts[0], 0x0700_0009, "the mission starts at its base map");

    // The bonus word rides every load [M lib.rs menu_after's pickup-medal-gate line — the
    // game's bonus[] is ONE store; only LS_LoadBonus and the award path write it].
    let bonus_rides = fe.bonus_bits();
    assert_eq!(bonus_rides, BONUS_MARKER, "the codename's bonus word rides the loads");

    let mut save: Option<RamSave> = None;
    let record = saved0.clone(); // the standing saved record — the keeps' source of truth
    let mut entry_saves: Vec<RamSave> = Vec::new();
    let mut failed_scene: Option<Scene> = None;
    let mut failed_level = 0u32;
    for (pi, &level) in parts.iter().enumerate() {
        let fail_leg = pi == 1; // the fail menu's legs run on the mission's middle part
        let mut sc = load_part(&iso, level, save.as_ref(), difficulty, bonus_rides, &record);
        {
            let sim = sc.sim.as_mut().expect("the sim");
            // THE PIN: the part's fresh layer took the saved record — no drop at the load.
            assert_record(&sim.drones.progress, &format!("part {pi} (0x{level:08x}) load"));
            // The entry checkpoint the App captures at every swap [M Player_InitWeapon's
            // unconditional Player_RamSave 0x1a7ef8:325].
            entry_saves.push(nf_view::entry_checkpoint_save(sim));
        }
        force_part(&mut sc, level, if fail_leg { 99 } else { 100 });
        let mut exit_next: Option<u32> = None;
        {
            let sim = sc.sim.as_mut().expect("the sim");
            // A real warm entry never finds the verdict channels ON; the fail arm opens the menu
            // (no level event), the complete arms end the level themselves [M 0x182c60:77-135].
            for t in 0..600u32 {
                sim.step_bond(Default::default(), false, false, false);
                for ev in std::mem::take(&mut sim.drones.mission_events) {
                    if let Event::LevelEnd { next } = ev { exit_next = Some(next); }
                }
                if sim.drones.level_exit.is_some() { exit_next = sim.drones.level_exit; break; }
                if fail_leg && t > 30 { break; } // the fail menu opens over the paused mission
            }
            // THE PIN: the record survived the part's own play — the globals stand under the
            // mission layer's ticking [M Mission_Init 0x182270 resets the tallies only].
            assert_record(&sim.drones.progress, &format!("part {pi} (0x{level:08x}) after play"));
        }
        if fail_leg {
            assert!(exit_next.is_none(), "the fail verdict loads no level [M 0x182c60:77-81]");
            failed_scene = Some(sc);
            failed_level = level;
            break; // the fail legs below; the walk resumes after them
        }
        save = Some(nf_view::warm_exit_save(sc.sim.as_ref().expect("the sim")));
    }
    // The walk reached the middle part and failed it (the legs below); the first part's warm
    // exit carried (07000009's exit loads 0700000a [D the LOAD_LEVEL trigger]).
    assert_eq!(failed_level, 0x0700_000a, "the fail leg runs on the mission's middle part");

    // ── the fail menu [M 0x182c60:77-80: menu 0x40000042 over the paused mission] ────────────
    // The saved file's bytes before the fail menu — the quit must leave them byte-identical.
    let file_before = std::fs::read(nf_game::profile::Profile::path_in(&dir, "SAVED01"))
        .expect("the saved file reads");
    // REPLAY (row 0 [L 0x40000042: REPLAY = 0; M P_ENDMISSION_Handler 0x216618:19-27]).
    {
        let fe = &mut fe;
        fe.flow.stack = vec![menu::ST_RUNNER];
        fe.flow.current_level = failed_level;
        fe.mission = row_ns;
        fe.mission_ended(Outcome::Failed, Stats::default());
        assert_eq!(fe.page, Page::Results, "the fail menu is the results page [M 0x182c60:77-80]");
        assert_eq!(fe.outcome, Outcome::Failed);
        assert!(fe.requested_level.is_none(), "the fail arm requests no level [M 0x182c60:77-80]");
        // The game's fail arm creates the manager DIRECTLY with page 0x40000042 and pauses —
        // it writes NO flow field [M 0x182c60 case 4, the MissionState != 6 arm:
        // MenuManager_Create(..., 0x40000042, ...) + GS_PauseGame(1); InternalState = 5]. The
        // port's fe.page is the manager page; the flow stays where the mission runs.
        assert_eq!(fe.up_page(), Some(Page::Results), "the manager's page is up [M MenuManager_Create 0x182c60:79]");
        assert_eq!(fe.flow.get_state(), menu::ST_RUNNER, "the flow stays where the mission runs");
        fe.cursor = 0;
        assert!(fe.action(Action::Confirm), "REPLAY confirms");
        assert_eq!(fe.requested_level.take(), Some(failed_level),
            "the REPLAY row reloads the current level [M 0x216618:19-21]");
        assert!(fe.requested_restart, "the entry-checkpoint half rides the REPLAY row [M 0x216618:21]");
        assert!(fe.closed, "the manager deletes [M 0x216618:28-30]");
        fe.closed = false;
        // The host serves it [M lib.rs menu_after's restart arm]: the ENTRY checkpoint with the
        // health floor (`Player_RamLoad(Bond, 1)` [M 0x216618:21]), and the swap re-seeds.
        let old = failed_scene.take().expect("the failed part's scene");
        let replay_save = nf_view::fail_restart_save(Some(&entry_saves[1]), old.sim.as_ref().expect("the sim"));
        let mut sc = load_part(&iso, failed_level, Some(&replay_save), difficulty, bonus_rides, &record);
        let sim = sc.sim.as_mut().expect("the sim");
        // The mission layer arms in the entry ticks [M Mission_Init 0x182270 — the campaign
        // runner's own 30-tick entry stand].
        for t in 0..30 {
            sim.step_bond(Default::default(), false, false, false);
            if t < 3 || t == 29 {
                eprintln!("[srap] replay tick {t}: internal {} state {} ch99 {} ch100 {} events {:?}",
                    sim.drones.mission.internal, sim.drones.mission.state,
                    sim.drones.channels.on(99), sim.drones.channels.on(100), sim.drones.mission_events);
            }
        }
        assert_record(&sim.drones.progress, "the REPLAY load");
        // The re-armed mission layer [M Mission_Init 0x182270:20-28] and the verdict channels
        // back to a real entry's state — the entry checkpoint's channels predate the fail
        // [M the restart arm serves the RamSave buffer as of level entry, not the dying capture].
        assert_eq!(sim.drones.mission.internal, 1, "the warm reload re-arms the mission layer");
        assert!(!sim.drones.channels.on(99) && !sim.drones.channels.on(100),
            "the carried verdict channels do not survive the entry-checkpoint reload");
    }
    // QUIT (row 2 — the cursor cycles 0/1 and [0usize, 2][cursor] maps cursor 1 to the QUIT row;
    // [L page 0x40000042: QUIT = 2; M 0x216618:24-26]).
    {
        let fe = &mut fe;
        fe.flow.stack = vec![menu::ST_RUNNER];
        fe.flow.current_level = failed_level;
        fe.mission_ended(Outcome::Failed, Stats::default());
        assert_eq!(fe.page, Page::Results);
        fe.cursor = 1;
        assert!(fe.action(Action::Confirm), "QUIT confirms");
        assert_eq!(fe.requested_level.take(), Some(menu::FRONTEND_LEVEL),
            "the QUIT row goes to the front end [M 0x216618:24-26]");
        assert_eq!(fe.page, Page::MissionSelect, "the mission list reopens [M +0x1c = 0x4000001c]");
        // THE PIN: the saved file is byte-identical — the quit wrote nothing [M 0x216618's
        // QUIT arm: no LS_Save anywhere; the card writes ride only the results save chain].
        let file_after = std::fs::read(nf_game::profile::Profile::path_in(&dir, "SAVED01"))
            .expect("the saved file reads");
        assert_eq!(file_before, file_after,
            "quitting from the fail menu must leave the saved file unchanged");
        // And the session's record still stands — the quit dropped nothing in memory either.
        assert_record(fe.saved_progress().expect("the session's record"), "after the fail-menu quit");
    }

    // ── the mission completes: the front-end leg the game runs at the mission end ────────────
    // [M Mission_Update case 4 → the outro code → the front-end map's load tail opens the
    // results page 0x40000036 [M 0x1d6360:160-163] → P_NFRESULTS case 0x4c.]
    let last = *parts.last().unwrap();
    {
        let fe = &mut fe;
        fe.flow.stack = vec![menu::ST_RUNNER];
        fe.flow.current_level = last;
        fe.mission = row_ns;
        fe.mission_ended(Outcome::Complete, Stats::default());
        assert_eq!(fe.requested_level.take(), Some(menu::outro_for(last)),
            "the outro code rode the load request [M 0x182c60:89-116]");
        let mut resolved = None;
        for _ in 0..64 {
            fe.tick();
            if let Some(r) = fe.flow.phase2_resolved.take() { resolved = Some(r); break; }
        }
        assert_eq!(resolved, Some(menu::FRONTEND_LEVEL), "the outro routes to the front end");
        assert_eq!(fe.flow.page, menu::PAGE_RESULTS, "phase 2 sets +0x1c = 0x40000036 [M 0x1d6360]");
        fe.tick();
        assert!(fe.reopen_queued_page(), "the manager create opens the queued results page");
        assert_eq!(fe.page, Page::Results);
    }
    // P_NFRESULTS case 0x4c against the live layer the mission ran with — the record's own
    // values, so the award lands ON the saved record (the game commits into the BSS globals
    // [M 0x1984b0:301-315] the save then carries).
    let mut live = record.clone();
    assert!(fe.results_enter(&mut live, 0x0700_0009, 6, difficulty, lang.as_ref()),
        "the results fields are built [M 0x208e78:77]");
    assert_eq!(live.nightfire_status() & (1 << row_ns), 1 << row_ns, "the completed row stamps");
    assert_eq!(live.game_complete, true, "the flag survives the award (only 1b sets it [M 0x1ffa10 tail])");
    // The award ADDS the run's medal bits to the bonus word [M 0x1ffa10:71-82's tier loop] —
    // the pin is that the SAVED bits survive it, not that the word is unchanged.
    assert_eq!(live.bonus[0] & BONUS_MARKER, BONUS_MARKER, "the bonus word's saved bits survive the award");
    assert!(live.bonus[0] & !BONUS_MARKER != 0, "the award added the run's medal bits");
    assert_eq!(live.table[0].best, TABLE_BEST_MARKER, "the table survives the award");
    // profile-session: the merge BEFORE the save prompt can run [M the game writes its MSSN/BNUS
    // from the same live state the results page reads — 0x1984b0:301-308 + case 1].
    fe.sync_profile_progress(&live);
    // The save prompt: the commit improved (the time row scores any run > 0 [M 0x1984b0:120-131])
    // and SAVED01 is a saved codename [M 0x208e78:24-32].
    assert!(fe.action(Action::Confirm), "the results confirm");
    assert!(fe.results_box, "the save prompt stands [M Txt 0x1000206]");
    assert!(fe.action(Action::Confirm), "Yes requests the save");
    assert!(fe.card_answer(menu::BTN_YES), "the overwrite question's Yes [M ASK_OVERWRITE]");
    fe.tick();
    assert_eq!(fe.card_message(), None, "the flow closed");
    // THE PIN: the file now holds the whole record — the award AND every marker.
    let p = nf_game::profile::Profile::load_file(&nf_game::profile::Profile::path_in(&dir, "SAVED01"))
        .expect("the saved file loads");
    assert_eq!(p.progress.nightfire_status() & (1 << row_ns), 1 << row_ns, "the award rode the save");
    assert_eq!(p.progress.game_complete, true, "the flag rode the save");
    assert_eq!(p.progress.bonus[0] & BONUS_MARKER, BONUS_MARKER, "the bonus word's saved bits rode the save");
    assert!(p.progress.bonus[0] & !BONUS_MARKER != 0, "the run's medal bits rode the save");
    assert_eq!(p.progress.table[0].best, TABLE_BEST_MARKER, "the table rode the save");
    let _ = std::fs::remove_dir_all(&dir);
}
