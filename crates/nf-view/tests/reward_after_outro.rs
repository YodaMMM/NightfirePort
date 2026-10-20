//! reward-after-outro (playtest 2026-10-18: "reward screen appearing before ending cutscene"):
//! the game's mission-end order is Mission_Update case 6 → case 4's load request + fade [M
//! 0x182c60:65-76] → the loader wraps the 0x7300xx outro code into GameFlow state 6 and plays the
//! ending movie [M ResetMap_Load 0x1d6360:101-112 + GameFlow_Main 0x1c98c8 case 6, which pops on
//! psiMovieFinished 0x1082e8] → phase 2 resolves the code to the front-end map [M
//! GetLevelToLoadFromMovieCode 0x1d3560; 0x1d6360:173-177] → the load's tail MenuManager_Create
//! opens the queued results page 0x40000036 [M 0x1d6360:160-163]. The reward page therefore opens
//! only AFTER the ending script ends. The front half (the real touches raising ch 0x64 → mission
//! state 6 → LevelToEndTo 0x07300013) is pinned through the sim by walk_07000013_live.

use nf_view::menu::{Action, FrontEnd, Outcome, Page, Stats};

/// The reward page opens only after the ending movie finishes — 07000013 (the playtest's tower
/// chain; outro code 0x7300013 [M 0x182c60:109-110]).
#[test]
fn the_reward_page_opens_only_after_the_ending_script_ends() {
    let mut fe = FrontEnd::new();
    fe.flow.current_level = 0x0700_0013;
    fe.flow.stack = vec![nf_view::menu::ST_RUNNER]; // the host's demo_results boots the runner state [lib.rs]
    fe.mission_ended(Outcome::Complete, Stats::default());
    // Case 4 queued the page (GameState+0x1c (Ghidra `_28_4_`, decimal) = 0x40000036 [M 0x182c60:72]) — no manager is up.
    assert_eq!(fe.flow.page, 0x4000_0036, "the page id queued [M 0x182c60:72]");
    assert!(fe.results_queued);
    assert_ne!(fe.page, Page::Results, "the reward is NOT up [M 0x1d6360:160-163]");
    assert_eq!(fe.requested_level.take(), Some(0x0730_0013), "LevelToEndTo [M 0x182c60:109-110]");
    assert!(fe.flow.movie_pending, "the 0x73 code wraps to a movie [M ResetMap_Load 0x1d6360:39 (the GS+8 & 0xff0000 movie decision) + :101 (the state-6 push)]");
    assert!(!fe.action(Action::Confirm), "no menu handlers while the manager is down [M 0x1d6360:160-163]");
    // The outro plays: the fade pops, the loader's phase 0 pushes state 6 with the movie code
    // [M 0x1d6360:101-112]; the decoder holds it (BGFMVPlaying = 1 [M 0x1081a8]).
    fe.flow.movie_playing = true;
    for _ in 0..70 { fe.tick(); } // the 60-frame fade [M disasm 0x183054-60] + the wrap
    assert_eq!(fe.flow.get_state(), nf_view::menu::ST_MOVIE, "the ending movie is up");
    assert_eq!(fe.flow.movie_code, 0x0730_0013);
    assert!(fe.flow.movie_pending, "the wrapped load still defers [M 0x1d6360 phases 0/3]");
    assert_ne!(fe.page, Page::Results, "the reward waits behind the ending script [M 0x1c98c8 case 6]");
    assert!(fe.results_queued);
    // The movie finishes — state 6 pops, phase 2 resolves 0x7300013 to the front-end map [M
    // 0x1d3560; 0x1d6360:173-177], the load's tail opens the page [M 0x1d6360:160-163].
    fe.flow.movie_playing = false;
    fe.tick();
    assert_eq!(fe.flow.phase2_resolved.take(), Some(nf_view::menu::FRONTEND_LEVEL), "the outro routes to the map [M 0x1d3560]");
    assert!(fe.flow.load_tick(), "phase 3 done");
    assert!(fe.reopen_queued_page(), "the load tail's MenuManager_Create [M 0x1d6360:160-163]");
    assert_eq!(fe.page, Page::Results, "NOW the reward page is up");
    assert!(!fe.results_queued);
}

/// Sister: a mission with NO ending NIS opens the reward directly — 07000011 is not in
/// Mission_Update case 6's compiled switch, so LevelToEndTo stays 0x7000048 [M 0x182c60:93-95
/// default arm]; the loader runs no movie branch (the 0xff0000 byte is clear [M 0x1d6360:39,179])
/// and the load's tail opens the page right after the fade + load.
#[test]
fn a_mission_with_no_ending_nis_opens_the_reward_directly() {
    assert_eq!(nf_view::menu::outro_for(0x0700_0011), nf_view::menu::FRONTEND_LEVEL, "no outro [M 0x182c60:93-95]");
    let mut fe = FrontEnd::new();
    fe.flow.current_level = 0x0700_0011;
    fe.flow.stack = vec![nf_view::menu::ST_RUNNER]; // the host's demo_results boots the runner state [lib.rs]
    fe.mission_ended(Outcome::Complete, Stats::default());
    assert_eq!(fe.flow.page, 0x4000_0036);
    assert!(fe.results_queued);
    assert_ne!(fe.page, Page::Results, "still behind the fade + load — but behind NO movie");
    assert_eq!(fe.requested_level.take(), Some(nf_view::menu::FRONTEND_LEVEL));
    // The walk: fade pops, the loader loads the front end directly — no movie state ever; the
    // headless load completes inside the window (the game's load-tail MenuManager_Create follows
    // the fade + load, nothing else [M 0x1d6360:160-163]).
    for _ in 0..70 { fe.tick(); }
    assert_eq!(fe.flow.get_state(), nf_view::menu::ST_RUNNER, "the plain load completed — no movie gate");
    assert_eq!(fe.flow.movie_code, 0, "no movie wrap [M 0x1d6360:39]");
    assert_ne!(fe.flow.get_state(), nf_view::menu::ST_MOVIE, "no ending NIS played");
    assert!(fe.reopen_queued_page(), "the load tail opens the page");
    assert_eq!(fe.page, Page::Results, "the reward page — directly, with no outro");
    assert!(!fe.results_queued);
}

/// The queued window swallows input and holds P_START's attract counter (a long outro must not
/// attract-open) [M the game has no MenuManager until 0x1d6360:160-163].
#[test]
fn the_queued_window_takes_no_input_and_holds_the_attract_counter() {
    let mut fe = FrontEnd::new();
    fe.flow.current_level = 0x0700_0013;
    fe.flow.stack = vec![nf_view::menu::ST_RUNNER]; // the host's demo_results boots the runner state [lib.rs]
    fe.mission_ended(Outcome::Complete, Stats::default());
    for a in [Action::Confirm, Action::Back, Action::Up, Action::Down] {
        assert!(!fe.action(a), "queued: {a:?} is swallowed");
    }
    for _ in 0..100 { fe.tick(); }
    assert_eq!(fe.no_input_frames, 0, "the no-input counter is not live while queued [M 0x1d6360:160-163]");
}
