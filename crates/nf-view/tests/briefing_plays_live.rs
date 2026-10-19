//! Playtest 1016: "cutscene didnt play when I started the exchange, it went straight to the loading
//! screen then to game". ResetMap_Load phase 0 pushes GameFlow state 6 with no frame budget of its own
//! [M 0x1d6360:101] and starts the movie in the same phase [M 0x1d6360:111 psiStartBackgroundMovie], so
//! BGFMVPlaying = 1 [M 0x1081a8] and case 6 holds until psiMovieFinished [M 0x1c98c8 case 6, 0x1082e8]. The port's
//! countdown popped state 6 on its first tick because the host had not reported the decode-ahead
//! player as playing.
//!
//! Mission-movies-e2e: EVERY single-player campaign row through the same path — the front end walk to
//! the Single Player list (`C_SBNFMAP` 0x20b8d8), the row's Confirm, then the window's own
//! `nf_view::mission_movie_glue` with the window's worst-case clock (up to 8 front-end ticks per
//! redraw): the briefing starts, state 6 holds while it plays, the level loads after it. And the
//! outros through the results path — `mission_ended` (the handler the window runs from the sim's
//! LevelEnd [M 0x182c60 case 4]) with the flow in the runner state at the mission's END level, the
//! outro code from case 6's switch [M 0x182e60-0x182eb4], through the same glue and real player.
//! The no-movie rows take the game's no-movie path (see docs/systems/mission-movies.md §2).
//!
//! Needs an image WITH movie data: the movie player reads `NF_MOVIE_ISO` first, then `NF_ISO`
//! (the same order movie.rs's own CLI uses) — the slim on-foot ISO's /MOVIES/30_FPS files are
//! 0-byte stubs [D both images, 2026-10-19], the full retail image carries all 23 [D]. Silent. Build with `--features nf-view/movies`.

#![cfg(feature = "movies")]

use nf_view::menu::{self, Action, FrontEnd, Outcome, Page, Stats, ST_MOVIE, ST_RUNNER};

/// The 12 campaign rows [M `sp_level` 0x2df2e0 +0x0c, menu.rs CAMPAIGN; `C_SBNFMAP` 0x20b8d8:21-24].
/// briefing/outro = None for the driving rows: the game pushes GameFlow state 9 (the DRIVING.ELF
/// handover) before the wrap switch is reached [M 0x1d3800:104-108], so no 0x71/0x73 code ever
/// exists for them. The end levels are the `MissionData` groups [L ee_start_05.bin 0x2a4350,
/// nf_game::mission::LEVELS]; the outro codes are case 6's switch [M 0x182e60-0x182eb4] — Countdown's
/// 0x07000016 → 0x07300017 is the one non-identity entry.
struct Row {
    cursor: usize,
    name: &'static str,
    start: u32,
    briefing: Option<u32>,
    end: u32,
    outro: Option<u32>,
}

const ROWS: [Row; 12] = [
    Row { cursor: 0, name: "Paris Prelude", start: 0x0900_0001, briefing: None, end: 0, outro: None },
    Row { cursor: 1, name: "The Exchange", start: 0x0700_0005, briefing: Some(0x0710_0005), end: 0x0700_0008, outro: Some(0x0730_0008) },
    Row { cursor: 2, name: "Alpine Escape", start: 0x0900_0005, briefing: None, end: 0, outro: None },
    Row { cursor: 3, name: "Enemies Vanquished", start: 0x0900_0006, briefing: None, end: 0, outro: None },
    Row { cursor: 4, name: "Double Cross", start: 0x0700_0001, briefing: Some(0x0710_0001), end: 0x0700_0004, outro: Some(0x0730_0004) },
    Row { cursor: 5, name: "Night Shift", start: 0x0700_0009, briefing: Some(0x0710_0009), end: 0x0700_000b, outro: Some(0x0730_000b) },
    Row { cursor: 6, name: "Chain Reaction", start: 0x0700_000c, briefing: Some(0x0710_000c), end: 0x0700_000d, outro: Some(0x0730_000d) },
    Row { cursor: 7, name: "Phoenix Fire", start: 0x0700_0011, briefing: Some(0x0710_0011), end: 0x0700_0013, outro: Some(0x0730_0013) },
    Row { cursor: 8, name: "Deep Descent", start: 0x0900_0002, briefing: None, end: 0, outro: None },
    Row { cursor: 9, name: "Island Infiltration", start: 0x0900_0003, briefing: None, end: 0, outro: None },
    Row { cursor: 10, name: "Countdown", start: 0x0700_0014, briefing: Some(0x0710_0014), end: 0x0700_0016, outro: Some(0x0730_0017) },
    Row { cursor: 11, name: "Equinox", start: 0x0700_001b, briefing: Some(0x0710_0018), end: 0x0700_001b, outro: Some(0x0730_001b) },
];

fn iso() -> String {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    // The movie player's image: NF_MOVIE_ISO first, then NF_ISO [the order movie.rs's CLI uses, 0x520].
    match std::env::var_os("NF_MOVIE_ISO").or_else(|| std::env::var_os("NF_ISO")) {
        Some(p) => p.to_string_lossy().to_string(),
        None => { eprintln!("NF_MOVIE_ISO/NF_ISO not set — skipping"); std::process::exit(0); }
    }
}

/// The front end walk the window runs: boot → the title → START → P_ESTHERO's intro (runs out
/// headless) → Main → NightFire → Play without saving → the box → difficulty → the Single Player
/// list — the same walk as movie_playback_flow / the pre-2026-10-19 briefing test.
fn walk_to_mission_select(fe: &mut FrontEnd, row: usize, name: &str) {
    for _ in 0..8 {
        fe.tick();
        if fe.flow.get_state() == ST_RUNNER && fe.flow.current_level == menu::FRONTEND_LEVEL { break; }
    }
    fe.action(Action::Confirm);
    for _ in 0..3 + menu::ESTHERO_TICKS { fe.tick(); }
    assert_eq!(fe.page, Page::Main, "{name}: the intro ran out; Main is up");
    fe.action(Action::Confirm); // NightFire → Codename after the 0x13 delay [M C_GONIGHTFIRE 0x2169b8]
    for _ in 0..menu::DELAYED_PAGE_TICKS + menu::MAIN_OPEN_BLOCK { fe.tick(); }
    fe.action(Action::Confirm); // Play without saving (row 1 [M P_NFSELECT 0x20aec0:96-97])
    fe.action(Action::Confirm); // the box
    fe.action(Action::Confirm); // difficulty → the Single Player list
    fe.unlock_all = true;
    fe.cursor = row;
    assert!(fe.action(Action::Confirm), "{name}: the row did not confirm");
}

/// One window redraw: the movie glue first, then the worst-case clock (up to 8 front-end ticks).
fn redraw(fe: &mut FrontEnd, slot: &mut Option<nf_view::movie::Player>, iso: &str) {
    nf_view::mission_movie_glue(fe, slot, iso);
    for _ in 0..8 { fe.tick(); }
}

/// The row's briefing: it starts (the decode-ahead opens it during the fade), state 6 holds while
/// it plays, and the level loads only after it — sister: the skip ends it and the load resolves to
/// the plain level [M 0x1d3560].
fn briefing_row(row: &Row) {
    let iso = iso();
    let code = row.briefing.expect("an on-foot row");
    let mut fe = FrontEnd::new();
    walk_to_mission_select(&mut fe, row.cursor, row.name);
    assert_eq!(fe.flow.level_to_load, code,
        "{}: ResetMap_LevelToLoad wrapped {} to its briefing [M 0x1d3800]", row.name, row.start);
    assert!(fe.flow.movie_pending, "{}: the host must hold the load until phase 2 [M 0x1d6360 phase 0]", row.name);

    let mut slot: Option<nf_view::movie::Player> = None;
    let mut state6_frames = 0u32;
    let mut saw_player = false;
    for _ in 0..600 {
        redraw(&mut fe, &mut slot, &iso);
        if let Some(p) = slot.as_mut() {
            if p.code() == code { saw_player = true; let _ = p.present(); }
        }
        if fe.flow.get_state() == ST_MOVIE { state6_frames += 1; }
        if state6_frames >= 120 { break; }
        if fe.flow.phase2_resolved.is_some() { break; }
    }
    assert!(saw_player, "{}: the briefing {code:08x} never started", row.name);
    assert!(state6_frames >= 120,
        "{}: state 6 lasted {state6_frames} redraws: the briefing was cut before it played [M 0x1c98c8 case 6 holds while BGFMVPlaying]", row.name);
    assert!(fe.flow.phase2_resolved.is_none(), "{}: the level load must wait for the briefing", row.name);

    // Sister: the skip still ends it and the load resolves to the plain level (the hold is not a hang).
    fe.flow.skip_movie();
    if let Some(p) = slot.take() { p.stop(); }
    let mut resolved = None;
    for _ in 0..20 {
        redraw(&mut fe, &mut slot, &iso);
        if let Some(r) = fe.flow.phase2_resolved { resolved = Some(r); break; }
    }
    if let Some(p) = slot.take() { p.stop(); } // the pending branch re-opens the code before phase 2 clears it
    assert_eq!(resolved, Some(row.start),
        "{}: after the skip the load resolves to the level [M 0x1d3560]", row.name);
}

/// The row's outro: the mission completes on its END level — the window's own end sequence. The
/// sim reported LevelEnd{next} [M 0x182c60 case 4]; the handler (`mission_ended`, the same call
/// the App makes from `pending_mission_end` [lib.rs]) captures the ended level, queues the results
/// page 0x40000036 [M 0x182c60:72], requests the outro code from case 6's switch [M
/// 0x182e60-0x182eb4] and pushes the 60-frame fade [M disasm 0x183054-60]. The glue's decode-ahead
/// opens the outro during the fade, state 6 holds while it plays, phase 2 routes the 0x73 code to
/// the front-end map with the results page [M 0x1d3560; 0x1d6360:173-177], and the load tail's
/// MenuManager_Create opens the page only then [M 0x1d6360:160-163].
fn outro_row(row: &Row) {
    let iso = iso();
    let code = row.outro.expect("an on-foot row");
    let mut fe = FrontEnd::new();
    // The flow sits in the runner state at the mission's end level — the state Mission_Update 6
    // runs in [M 0x182c60; lib.rs demo_results sets exactly this].
    fe.flow.stack = vec![ST_RUNNER];
    fe.flow.current_level = row.end;
    fe.mission_ended(Outcome::Complete, Stats::default());
    assert_eq!(fe.requested_level.take(), Some(code),
        "{}: Mission_Update case 6 maps the end level {:#x} to the outro [M 0x182e60-0x182eb4]", row.name, row.end);
    assert!(fe.flow.movie_pending, "{}: the 0x73 code wraps to a movie [M 0x1d3800's switch]", row.name);
    assert_eq!(fe.flow.page, menu::PAGE_RESULTS, "{}: the results page queued [M 0x182c60:72]", row.name);
    assert!(fe.results_queued, "{}: the page waits for the load tail [M 0x1d6360:160-163]", row.name);
    assert_ne!(fe.page, Page::Results, "{}: no manager while the movie is due [M 0x1d6360:160-163]", row.name);

    let mut slot: Option<nf_view::movie::Player> = None;
    let mut state6_frames = 0u32;
    let mut saw_player = false;
    for _ in 0..600 {
        redraw(&mut fe, &mut slot, &iso);
        if let Some(p) = slot.as_mut() {
            if p.code() == code { saw_player = true; let _ = p.present(); }
        }
        if fe.flow.get_state() == ST_MOVIE { state6_frames += 1; }
        if state6_frames >= 120 { break; }
        if fe.flow.phase2_resolved.is_some() { break; }
    }
    assert!(saw_player, "{}: the outro {code:08x} never started", row.name);
    assert!(state6_frames >= 120,
        "{}: state 6 lasted {state6_frames} redraws: the outro was cut before it played [M 0x1c98c8 case 6]", row.name);
    assert!(fe.flow.phase2_resolved.is_none(), "{}: the results load must wait for the outro", row.name);

    // The skip: state 6 pops, phase 2 routes the outro to the front-end map [M 0x1d3560] and sets
    // the results page [M 0x1d6360:173-177]; the load tail opens the page [M :160-163].
    fe.flow.skip_movie();
    if let Some(p) = slot.take() { p.stop(); }
    let mut resolved = None;
    for _ in 0..20 {
        redraw(&mut fe, &mut slot, &iso);
        if let Some(r) = fe.flow.phase2_resolved { resolved = Some(r); break; }
    }
    if let Some(p) = slot.take() { p.stop(); }
    assert_eq!(resolved, Some(menu::FRONTEND_LEVEL),
        "{}: the outro routes to the front-end map [M 0x1d3560]", row.name);
    assert_eq!(fe.flow.page, menu::PAGE_RESULTS, "{}: phase 2 set the results page [M 0x1d6360:173-177]", row.name);
    assert!(fe.reopen_queued_page(), "{}: the load tail's MenuManager_Create [M 0x1d6360:160-163]", row.name);
    assert_eq!(fe.page, Page::Results, "{}: the results page opens after the outro", row.name);
    assert!(!fe.results_queued);
}

// ── The briefings: every on-foot campaign row ─────────────────────────────────

#[test]
fn the_exchange_briefing_plays_before_the_level_loads() { briefing_row(&ROWS[1]); }

#[test]
fn double_cross_briefing_plays_before_the_level_loads() { briefing_row(&ROWS[4]); }

#[test]
fn night_shift_briefing_plays_before_the_level_loads() { briefing_row(&ROWS[5]); }

#[test]
fn chain_reaction_briefing_plays_before_the_level_loads() { briefing_row(&ROWS[6]); }

#[test]
fn phoenix_fire_briefing_plays_before_the_level_loads() { briefing_row(&ROWS[7]); }

#[test]
fn countdown_briefing_plays_before_the_level_loads() { briefing_row(&ROWS[10]); }

#[test]
fn equinox_briefing_wraps_to_07100018_and_plays() { briefing_row(&ROWS[11]); }

// ── The driving rows: no wrap, no movie — the game's no-movie path ────────────

/// The five driving rows never wrap [M 0x1d3800:104-108 — the DRIVING branch pushes state 9 with
/// GameState+0x8 = the hash and no 0x71 code] and their hashes are not in `UsingMovie` [M
/// 0x107eb8]. The port, which has no DRIVING.ELF, refuses the row with its driving notice instead
/// of pushing state 9 [S port choice, playtest 1015-2231] — either way no movie is due: no
/// movie_pending, no state 6, no player, no level request. This is the no-movie path every
/// not-in-USING_MOVIE row must take.
#[test]
fn the_driving_rows_take_the_no_movie_path() {
    let iso = iso();
    for row in ROWS.iter().filter(|r| r.briefing.is_none()) {
        let mut fe = FrontEnd::new();
        walk_to_mission_select(&mut fe, row.cursor, row.name);
        assert!(!fe.flow.movie_pending, "{}: no movie is due for a driving row", row.name);
        assert_eq!(fe.flow.level_to_load, menu::FRONTEND_LEVEL,
            "{}: the port requests no level for a refused driving row [S port choice — the game itself writes the driving hash to GameState+8, C_SBNFMAP 0x20b8d8:27 / 0x1d3800:105-108, and hands over to DRIVING.ELF]", row.name);
        assert!(fe.notice.is_some(),
            "{}: the port's driving notice stands in for the state-9 handover [S port choice]", row.name);
        let mut slot: Option<nf_view::movie::Player> = None;
        for _ in 0..40 {
            redraw(&mut fe, &mut slot, &iso);
            assert!(slot.is_none(), "{}: no movie may start for a driving row", row.name);
            assert_ne!(fe.flow.get_state(), ST_MOVIE, "{}: no state 6 for a driving row", row.name);
            assert!(fe.flow.phase2_resolved.is_none(), "{}: no level load rides a driving row", row.name);
        }
        assert!(fe.flow.get_state() == ST_RUNNER || fe.flow.get_state() == menu::ST_LAUNCH_DRIVING,
            "{}: the flow waits for the driving handover", row.name);
    }
}

// ── The outros: every on-foot mission's end level through the results path ────

#[test]
fn the_exchange_outro_plays_through_the_results_path() { outro_row(&ROWS[1]); }

#[test]
fn double_cross_outro_plays_through_the_results_path() { outro_row(&ROWS[4]); }

#[test]
fn night_shift_outro_plays_through_the_results_path() { outro_row(&ROWS[5]); }

#[test]
fn chain_reaction_outro_plays_through_the_results_path() { outro_row(&ROWS[6]); }

#[test]
fn phoenix_fire_outro_plays_through_the_results_path() { outro_row(&ROWS[7]); }

#[test]
fn countdown_outro_is_the_non_identity_07300017() { outro_row(&ROWS[10]); }

#[test]
fn equinox_outro_plays_through_the_results_path() { outro_row(&ROWS[11]); }

// ── The no-movie endings ──────────────────────────────────────────────────────

/// Mission 0x0700000e's end level 0x0700000f maps to 0x0730000f in case 6's switch [M
/// 0x182e60-0x182eb4] — but 0x073000f is NOT in `UsingMovie` [M 0x107eb8], so
/// psiStartBackgroundMovie returns without opening anything [M 0x1081a8], BGFMVPlaying stays 0 and
/// state 6 exits on frame 1 [M 0x1082e8 + 0x1c98c8 case 6]: the routing entry exists for exactly
/// that. No campaign row starts mission 0x0700000e (reachable only through the level chain), but
/// the no-movie path it takes is a hard game behaviour: no player, phase 2 resolves to the map
/// with the results page, no hold. (No mission has an end level the switch maps to a whitelisted
/// movie yet fails to play — every on-foot campaign row's outro is whitelisted.)
#[test]
fn the_073000f_outro_code_takes_the_frame1_no_movie_path() {
    let iso = iso();
    let mut fe = FrontEnd::new();
    fe.flow.stack = vec![ST_RUNNER];
    fe.flow.current_level = 0x0700_000f;
    fe.mission_ended(Outcome::Complete, Stats::default());
    assert_eq!(fe.requested_level.take(), Some(0x0730_000f), "the switch entry [M 0x182e60-0x182eb4]");
    assert!(fe.flow.movie_pending);
    let mut slot: Option<nf_view::movie::Player> = None;
    let mut saw_player = false;
    let mut resolved = None;
    for _ in 0..40 {
        redraw(&mut fe, &mut slot, &iso);
        if slot.is_some() { saw_player = true; }
        if let Some(r) = fe.flow.phase2_resolved { resolved = Some(r); break; }
    }
    assert!(!saw_player, "0x073000f is not whitelisted: psiStartBackgroundMovie must refuse it [M 0x107eb8]");
    assert_eq!(resolved, Some(menu::FRONTEND_LEVEL), "the resolve lands on the map [M 0x1d3560]");
    assert_eq!(fe.flow.page, menu::PAGE_RESULTS, "the results page opens [M 0x1d6360:173-177]");
    assert!(fe.reopen_queued_page(), "the load tail's MenuManager_Create [M 0x1d6360:160-163]");
    assert_eq!(fe.page, Page::Results);
}

/// The default arm: completing on a NON-end part (early completion) maps to the front-end map
/// [M 0x182c60:93-95] — a PLAIN 0x07000048 load, no wrap, no state 6 at all; the load tail opens
/// the results page directly [M 0x1d6360:160-163]. The Exchange's first part (0x07000005) is the
/// example [L nf_game::mission::LEVELS: 05 is part 0 of mission 0x07000005].
#[test]
fn completing_a_non_end_part_takes_the_plain_map_path_with_no_movie() {
    let iso = iso();
    let mut fe = FrontEnd::new();
    fe.flow.stack = vec![ST_RUNNER];
    fe.flow.current_level = 0x0700_0005;
    fe.mission_ended(Outcome::Complete, Stats::default());
    assert_eq!(fe.requested_level.take(), Some(menu::FRONTEND_LEVEL),
        "0x07000005 is not in case 6's switch: LevelToEndTo = the map [M 0x182c60:93-95]");
    assert!(!fe.flow.movie_pending, "a plain map load wraps to no movie [M 0x1d3800]");
    let mut slot: Option<nf_view::movie::Player> = None;
    let mut saw_state6 = false;
    let mut saw_player = false;
    for _ in 0..40 {
        redraw(&mut fe, &mut slot, &iso);
        saw_state6 |= fe.flow.get_state() == ST_MOVIE;
        saw_player |= slot.is_some();
    }
    assert!(!saw_player && !saw_state6, "the plain path plays no movie [M 0x1d6360:39 the 0xff0000 byte]");
    assert_eq!(fe.flow.current_level, menu::FRONTEND_LEVEL, "the map loaded [M 0x1d6360 phase 2]");
    assert!(fe.reopen_queued_page(), "the load tail's MenuManager_Create [M 0x1d6360:160-163]");
    assert_eq!(fe.page, Page::Results);
}

/// A FAILED mission opens the 0x40000042 fail menu OVER the paused mission — no load request, no
/// fade, no movie, the flow stays where the mission runs [M 0x182c60:77-80,
/// P_ENDMISSION_Handler 0x216618]. Sister to every outro row: the outro only rides completion.
#[test]
fn a_failed_mission_opens_the_fail_menu_and_plays_no_outro() {
    let iso = iso();
    let mut fe = FrontEnd::new();
    fe.flow.stack = vec![ST_RUNNER];
    fe.flow.current_level = 0x0700_0008; // The Exchange's end level — the outro row's flow state
    fe.mission_ended(Outcome::Failed, Stats::default());
    assert_eq!(fe.page, Page::Results, "the fail menu 0x40000042 over the paused mission [M 0x182c60:77-80]");
    assert_eq!(fe.outcome, Outcome::Failed);
    assert!(fe.requested_level.is_none(), "no load request on failure [M 0x182c60:77-80]");
    assert!(!fe.flow.movie_pending, "no movie is due on failure [M 0x182c60:77-80]");
    assert!(!fe.results_queued, "the fail menu never queues [M 0x182c60:77-80]");
    assert_eq!(fe.flow.get_state(), ST_RUNNER, "the flow stays where the mission runs [M 0x182c60:77-80]");
    let mut slot: Option<nf_view::movie::Player> = None;
    for _ in 0..40 {
        redraw(&mut fe, &mut slot, &iso);
        assert!(slot.is_none(), "no movie on the fail path");
        assert_ne!(fe.flow.get_state(), ST_MOVIE, "no state 6 on the fail path");
    }
}
