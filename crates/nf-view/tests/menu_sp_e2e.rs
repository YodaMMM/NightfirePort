//! menu-e2e-singleplayer: every single-player path from boot to a level request, driven ONLY
//! through `action()` / `tick()` and the host-side calls the window makes (`set_profile_dir`,
//! `requested_level.take()` + the flow state the host owns around `mission_ended` [M lib.rs
//! demo_results 2076-2077], `results_enter`, `bonus_enter`, `sync_profile_progress`,
//! `win_movie_finished`, the card flow's dialog answers). No `page`/`cursor`/state-field
//! writes to get anywhere. The menus were once broken end to end even
//! though every page had passed its own unit tests — those set `fe.page = ...` directly, so
//! the real paths were never exercised.

use nf_view::menu::{self, Action, CAMPAIGN, FrontEnd, Outcome, Page, Stats};

/// One action, then the frames a player would wait: the page-change frames, and any intro /
/// attract movie run out (the headless movies are countdowns). The host's movie_tick consumes
/// each phase-2 resolve per leg [M lib.rs 2326], so the walker does the same.
fn act(fe: &mut FrontEnd, a: Action) -> bool {
    let r = fe.action(a);
    for _ in 0..60 {
        fe.tick();
        let _ = fe.flow.phase2_resolved.take();
    }
    for _ in 0..5000 {
        if !fe.intro_playing && !fe.attract_playing {
            break;
        }
        fe.tick();
        let _ = fe.flow.phase2_resolved.take();
    }
    r
}

/// A scratch card dir per test (the same pattern codename_entry / memcard_save_flow use).
fn card_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("nf-sp-e2e-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The d-pad actions that walk the grid focus from `from` to `to`: a breadth-first search over
/// the real moves (`menu::cnname_move`, the game's `Menu_FindControl` scoring over the page
/// data's cell rects), so every step the walk asks for lands.
fn grid_walk(from: u32, to: u32) -> Vec<Action> {
    use nf_view::menu_engine::FindDir;
    use std::collections::VecDeque;
    let pos = FrontEnd::cn_cell_index(from).expect("from is a drawn cell");
    let goal = FrontEnd::cn_cell_index(to).expect("to is a drawn cell");
    let mut prev: Vec<Option<(usize, Action)>> = vec![None; nf_view::menu::CNNAME_CELLS.len()];
    prev[pos] = Some((pos, Action::Up));
    let mut q = VecDeque::from([pos]);
    while let Some(i) = q.pop_front() {
        if i == goal { break; }
        for (a, d) in [(Action::Up, FindDir::Up), (Action::Down, FindDir::Down), (Action::Left, FindDir::Left), (Action::Right, FindDir::Right)] {
            let n = nf_view::menu::cnname_move(i, d);
            if n != i && prev[n].is_none() {
                prev[n] = Some((i, a));
                q.push_back(n);
            }
        }
    }
    let mut path = Vec::new();
    let mut cur = goal;
    while cur != pos {
        let (p, a) = prev[cur].expect("the grid walk is connected");
        path.push(a);
        cur = p;
    }
    path.reverse();
    path
}

/// Types `name` on the name grid the way the window feeds it: d-pad moves and presses.
fn type_on_grid(fe: &mut FrontEnd, name: &str) {
    for ch in name.chars() {
        let cell = ch.to_ascii_uppercase() as u32;
        for a in grid_walk(fe.cn_cursor_cell(), cell) {
            assert!(fe.action(a), "the grid walk to {ch} lands");
        }
        assert_eq!(fe.cn_cursor_cell(), cell, "the cursor stands on {ch}");
        assert!(fe.action(Action::Confirm), "the {ch} press types");
    }
    for a in grid_walk(fe.cn_cursor_cell(), FrontEnd::CN_KEY_CONFIRM) {
        assert!(fe.action(a), "the walk back to the ✓ lands");
    }
    assert_eq!(fe.cn_cursor_cell(), FrontEnd::CN_KEY_CONFIRM);
}

/// Runs a mission end's load out on ticks (the fade and the loader are headless-fast) until
/// the front-end map is back under the runner state. Returns the phase-2 resolve when the end
/// wrapped through an outro movie.
fn drive_end_to_map(fe: &mut FrontEnd) -> Option<u32> {
    let mut resolved = None;
    for _ in 0..400 {
        fe.tick();
        if let Some(r) = fe.flow.phase2_resolved.take() {
            resolved = Some(r);
        }
        if fe.flow.get_state() == menu::ST_RUNNER && fe.flow.current_level == menu::FRONTEND_LEVEL {
            return resolved;
        }
    }
    panic!("the front-end load never finished");
}

/// Boot → title → START → P_PARISENUM → P_ESTHERO (its 0x073B0048 plays out) → Main
/// [M 0x20a648:43, 0x20a060:16-45].
fn to_main(fe: &mut FrontEnd) {
    assert_eq!(fe.page, Page::Start, "the boot opens the title");
    assert!(fe.action(Action::Confirm), "the title's Enter starts the intro run");
    assert!(fe.intro_playing, "the intro run follows the title");
    assert_eq!(fe.page, Page::ParisEnum, "the up-page is P_PARISENUM, not Main [M 0x20a648]");
    for _ in 0..32 { fe.tick(); if fe.page == Page::EstHero { break; } } // G-048(4): the card walk
    assert_eq!(fe.page, Page::EstHero, "then P_ESTHERO while its movie plays [M 0x20a060:44-45]");
    assert!(!fe.action(Action::Down), "the intro swallows other input [M 0x20a060:25 msg 0x68/1]");
    for _ in 0..5000 {
        if !fe.intro_playing && !fe.attract_playing {
            break;
        }
        fe.tick();
    }
    assert!(!fe.intro_playing, "the intro ran out on its own ticks");
    assert_eq!(fe.page, Page::Main);
}

/// Main → NightFire (Select Codename).
fn to_codename(fe: &mut FrontEnd) {
    to_main(fe);
    assert!(act(fe, Action::Confirm), "NightFire");
    assert_eq!(fe.page, Page::Codename);
}

/// Codename page (opened from Main on row 1, Play without saving [M P_NFSELECT 0x20aec0:96-97]) →
/// two Confirms (the confirm box) → the difficulty wheel's first row [M P_NFDFCTY 0x216bf8:10-12]
/// → the mission list.
fn to_missions(fe: &mut FrontEnd) {
    to_codename(fe);
    assert_eq!(fe.cursor, 1, "Play without saving [M 0x20aec0:96-97]");
    assert!(act(fe, Action::Confirm), "the confirm box opens");
    assert!(act(fe, Action::Confirm), "the box's yes");
    assert_eq!(fe.page, Page::Difficulty);
    assert_eq!(fe.cursor, 0, "the wheel 0x100000bf opens on index 0 [M 0x216bf8:12]");
    assert!(act(fe, Action::Confirm), "the difficulty");
    assert_eq!(fe.page, Page::MissionSelect);
}

/// Fresh profile → the driving notice on row 0 → dismiss → The Exchange requested.
/// Returns the requested level (already taken, the host consumed it).
fn fresh_profile_to_the_exchange(fe: &mut FrontEnd) -> u32 {
    to_missions(fe);
    // The list opens on the HIGHEST unlocked row [M P_NFMAP 0x20b608:58-64]: rows 0-1 on a fresh
    // profile [D sp_level .data] → The Exchange. Up reaches Paris Prelude.
    assert_eq!(fe.cursor, 1, "a fresh profile opens on The Exchange [M 0x20b608:60-64]");
    assert!(act(fe, Action::Up), "Up to Paris Prelude");
    assert_eq!(fe.cursor, 0);
    assert!(act(fe, Action::Confirm), "Paris Prelude");
    assert!(fe.notice.is_some(), "the driving notice shows");
    assert_eq!(fe.requested_level, None, "a driving row requests nothing");
    assert!(act(fe, Action::Confirm), "Enter dismisses the notice");
    assert!(fe.notice.is_none());
    assert_eq!(fe.cursor, 0);
    assert!(act(fe, Action::Down), "Down to The Exchange");
    assert_eq!(fe.cursor, 1);
    assert!(fe.unlocked_row(1), "the row after a driving row opens through it");
    assert!(act(fe, Action::Confirm), "The Exchange starts");
    let level = fe.requested_level.expect("the level request");
    assert_eq!(level, CAMPAIGN[1].3);
    assert_eq!(level, 0x0700_0005);
    // A second Enter while the request stands is refused (the game deletes the menu manager
    // with it) — a second Enter during a load used to start a second load.
    assert!(!fe.action(Action::Confirm), "no second request");
    assert_eq!(fe.requested_level.take(), Some(level)); // [host] the level loads
    level
}

// ── boot / intro ─────────────────────────────────────────────────────────────────────────────

#[test]
fn boot_reaches_main_through_the_intro_both_ways() {
    // Skipped: the second Enter pops the movie state and Main is already up.
    let mut fe = FrontEnd::new();
    assert_eq!(fe.page, Page::Start);
    assert!(fe.action(Action::Confirm), "the title's Enter");
    assert!(fe.intro_playing);
    for _ in 0..32 { fe.tick(); if fe.page == Page::EstHero { break; } } // G-048(4): the card walk
    for _ in 0..2 { fe.tick(); }
    assert!(fe.intro_movie, "P_ESTHERO's 2nd 0x50 started the movie [M 0x20a060:16-24]");
    // The grace: a press in the first 180 ticks is refused and the intro keeps playing
    // [M 0x20a060:17 DAT_002e1c4c = 0xb4; Menu_StopMovie 0x215a50:6].
    assert!(!fe.action(Action::Confirm), "an early press is refused");
    assert!(fe.intro_movie, "the intro can't be skipped in its first 180 ticks");
    for _ in 0..nf_view::menu::ESTHERO_GRACE { fe.tick(); }
    assert!(fe.action(Action::Confirm), "the skip press");
    fe.tick();
    assert!(!fe.intro_playing, "Enter skipped the intro; the next 0x50 opened Main [M 0x20a060:28-45]");
    for _ in 0..60 {
        fe.tick();
    }
    assert_eq!(fe.page, Page::Main);
    assert!(act(&mut fe, Action::Confirm), "NightFire still opens after a skip");
    assert_eq!(fe.page, Page::Codename);

    // Played out: the movies end on their own ticks and Main stands.
    let mut fe = FrontEnd::new();
    to_main(&mut fe);
    assert!(act(&mut fe, Action::Confirm), "NightFire");
    assert_eq!(fe.page, Page::Codename);
}

// ── Back at every step ───────────────────────────────────────────────────────────────────────

#[test]
fn back_walks_every_single_player_step_to_the_title() {
    let mut fe = FrontEnd::new();
    to_missions(&mut fe);
    // The Dossier opens off the mission list with □ and returns with Back.
    assert!(act(&mut fe, Action::Side), "□ opens the Dossier");
    assert_eq!(fe.page, Page::Dossier);
    assert_eq!(fe.cursor, 0, "the Dossier wheel opens at Records [M 0x216de0:9-16]");
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::MissionSelect);
    // fe-audit 1019: the list's Back is ROUTE-DEPENDENT. 0x5f runs the page's msg-99 SCRIPT
    // first [M 0x21c858:604-608] — the disc's front-end def binds no msg-99 script on any page
    // (every page's only SCRIPT is 0x2000000f on the 0x4c-open slot 0 [D the 08000002 tag
    // stream]) — so the pushed SP chain takes the manager's default pop: reopen the wheel with
    // its saved cursor [M 0x21c858:592-598]. (The empty-stack results route is P_NFMAP's 99 arm
    // → MAIN [M 0x20b608:67-68] — pinned in a_completed_mission_routes… below.)
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Difficulty, "the pushed chain pops to the wheel [M 0x21c858:592-598]");
    assert_eq!(fe.cursor, 0, "the wheel re-opens on the chosen difficulty (Operative) [M 0x21c858:598]");
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Codename);
    assert_eq!(fe.cursor, 0);
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Main);
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Start, "Back walks all the way to the title [S — the port's Main Back]");
    // …and forward again still works.
    assert!(act(&mut fe, Action::Confirm), "the title");
    assert_eq!(fe.page, Page::Main);
    assert!(act(&mut fe, Action::Confirm), "NightFire");
    assert_eq!(fe.page, Page::Codename);
}

// ── the Select Codename branches ─────────────────────────────────────────────────────────────

#[test]
fn select_codename_play_without_saving_asks_and_back_cancels() {
    let mut fe = FrontEnd::new();
    to_codename(&mut fe);
    assert_eq!(fe.rows(), 2, "no card rows without a card");
    assert_eq!(fe.cursor, 1, "opened from Main on Play without saving [M P_NFSELECT 0x20aec0:96-97]");
    assert!(act(&mut fe, Action::Confirm), "the confirm box opens");
    assert!(act(&mut fe, Action::Back), "△ cancels the box");
    assert_eq!(fe.page, Page::Codename, "the cancel stays on the page");
    assert!(act(&mut fe, Action::Confirm), "the box opens again");
    assert!(act(&mut fe, Action::Confirm), "the box's yes");
    assert_eq!(fe.page, Page::Difficulty);
    assert_eq!(fe.cursor, 0, "the first row, Operative [M P_NFDFCTY 0x216bf8:10-12]");
}

#[test]
fn select_codename_new_codename_validates_creates_saves_and_reloads() {
    let dir = card_dir("new-codename");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone()); // [host] the card wiring before the menus run
    to_codename(&mut fe);
    // Row 0 "New Codename" (the page opened on row 1 [M 0x20aec0:96-97]; Up reaches it) → the name
    // grid: the game's proposal, the ✓ focused.
    assert!(act(&mut fe, Action::Up));
    assert_eq!(fe.cursor, 0);
    assert!(act(&mut fe, Action::Confirm), "New Codename");
    assert_eq!(fe.page, Page::CnName);
    assert_eq!(fe.cn_typed_name(), "BOND", "the game's proposal [L ee_snapshot 0x30d530]");
    assert_eq!(fe.cn_cursor_cell(), FrontEnd::CN_KEY_CONFIRM, "the ✓ holds the focus");
    // The empty-name refusal through the real path: Delete clears the proposal, ✓ refuses.
    for a in grid_walk(FrontEnd::CN_KEY_CONFIRM, FrontEnd::CN_KEY_DELETE) {
        assert!(fe.action(a), "the walk to ← lands");
    }
    assert!(fe.action(Action::Confirm), "← clears the proposal");
    assert_eq!(fe.cn_typed_name(), "");
    for a in grid_walk(FrontEnd::CN_KEY_DELETE, FrontEnd::CN_KEY_CONFIRM) {
        assert!(fe.action(a), "the walk back to the ✓ lands");
    }
    assert!(!fe.action(Action::Confirm), "the empty name is refused");
    assert_eq!(fe.cn_box, Some((6, 0x3bc)), "the validator's box [M 0x213a98:47-51]");
    assert!(fe.action(Action::Confirm), "the box's ok closes it");
    assert_eq!(fe.cn_box, None);
    // Type MONEYP on the grid, through the d-pad path, and submit on ✓.
    type_on_grid(&mut fe, "MONEYP");
    assert_eq!(fe.cn_typed_name(), "MONEYP");
    assert!(act(&mut fe, Action::Confirm), "the ✓ submits");
    // The create saved through the card flow and landed on the wheel.
    assert_eq!(fe.page, Page::CnMenu, "the completion routes to the wheel [M 0x213a98:58]");
    assert_eq!(fe.cursor, 1, "the wheel opens on Controller Setup [L PCSX2 m30]");
    assert_eq!(fe.active_profile.as_deref(), Some("MONEYP"));
    assert!(nf_game::profile::Profile::path_in(&dir, "MONEYP").exists(), "the file is on the card");
    // Back to the main menu and re-open: the saved codename is listed and loads.
    assert!(act(&mut fe, Action::Back), "the wheel goes back");
    assert_eq!(fe.page, Page::CnSelect);
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Main);
    assert_eq!(fe.cursor, 2, "the Codenames row");
    // fe-audit 1017: Main's wrap is [S unverified] (the rows are M_BUTTONs; the focus walk is
    // not in the decompile) — the walk selects NightFire directly instead of relying on it.
    fe.cursor = 0;
    assert!(act(&mut fe, Action::Confirm), "NightFire");
    assert_eq!(fe.page, Page::Codename);
    assert_eq!(fe.rows(), 3, "the card's codename is listed [M Menu_EnumSaves]");
    assert_eq!(fe.cursor, 1, "opened from Main on row 1 [M 0x20aec0:96-97]");
    assert!(act(&mut fe, Action::Down));
    assert_eq!(fe.cursor, 2, "MONEYP's row");
    assert!(act(&mut fe, Action::Confirm), "the saved codename loads");
    assert_eq!(fe.page, Page::Difficulty);
    assert_eq!(fe.active_profile.as_deref(), Some("MONEYP"));
    assert!(fe.unlocked_row(0) && fe.unlocked_row(1), "the fresh card keeps the boot unlocks");
    assert!(!fe.unlocked_row(2));
    assert!(act(&mut fe, Action::Confirm), "the difficulty");
    assert_eq!(fe.page, Page::MissionSelect);
    let _ = std::fs::remove_dir_all(&dir);
}

// ── every difficulty → the mission list ──────────────────────────────────────────────────────

#[test]
fn each_difficulty_reaches_the_mission_list() {
    let mut fe = FrontEnd::new();
    to_codename(&mut fe);
    assert!(act(&mut fe, Action::Confirm), "the confirm box");
    assert!(act(&mut fe, Action::Confirm), "the box's yes");
    assert_eq!(fe.page, Page::Difficulty);
    // fe-audit: the wheel opens on 0 [M 0x216bf8:12] and WRAPS — Scroll_Init sets the wrap flag,
    // only msg 0x69 clears it [M 0x2214f0:102-103, 0x221810:136-138; Scroll_Update 0x221b20:186-210;
    // C_SBNFDFCTY 0x216ca8:25-28 range 0..2].
    assert_eq!(fe.cursor, 0);
    assert!(act(&mut fe, Action::Down));
    assert!(act(&mut fe, Action::Down));
    assert_eq!(fe.cursor, 2);
    assert!(act(&mut fe, Action::Down), "Down at the bottom wraps");
    assert_eq!(fe.cursor, 0, "back on Operative [M 0x221b20:186-194]");
    // Every tier confirms into the mission list. The confirm stores GameState+0x40 =
    // difficulty[i*0x18+0xc] — the boot table's tier values 1/2/3, the port's +1 offset [D
    // ACTION.ELF .data 0x2df4c0: records 0..2 hold +0xc = 1/2/3, 0x18-byte stride] — and opens
    // 0x4000001c [M C_SBNFDFCTY 0x216ca8:10-17]. The gate byte +0x10 is 1 on all three rows, so
    // a fresh profile confirms every tier [D 0x2df4c0; the gate M 0x216ca8:12-15].
    for d in [2usize, 1, 0] {
        while fe.cursor < d {
            assert!(act(&mut fe, Action::Down));
        }
        while fe.cursor > d {
            assert!(act(&mut fe, Action::Up));
        }
        assert_eq!(fe.cursor, d);
        assert!(act(&mut fe, Action::Confirm), "the difficulty row confirms");
        assert_eq!(fe.page, Page::MissionSelect);
        assert_eq!(fe.difficulty, d, "the choice rode the confirm");
        assert_eq!(fe.cursor, 1, "the list opens on the highest unlocked row [M 0x20b608:60-64]");
        // fe-audit 1019: the pushed chain pops back to the wheel — the confirm's 0x44 carries
        // b = DAT_002e1adc [M 0x202190:9] — here 0 [S] unverified: the global is cleared by
        // C_SBNFCN 0x20b390:30 / P_NFSELECT 0x20aec0:65 and set by P_MPJOIN 0x20ca00:182 /
        // C_SBCNOPTIONS 0x213528:66 / P_CNNAME 0x213790:71 / C_KEYBOARD 0x213a98:61, so the
        // value depends on which menus ran — pushed it [M the push rule 0x21c858 case 0x44;
        // C_SBNFDFCTY 0x216ca8:10-17 only calls Menu_ChangePageCloseIris], and the pop
        // reopens the saved page AND cursor [M 0x21c858:592-598]. (No msg-99 script stands on
        // the disc [D the 08000002 tag stream] — the 99 arm only fires on an empty stack.)
        assert!(act(&mut fe, Action::Back));
        assert_eq!(fe.page, Page::Difficulty, "the pop re-opens the wheel [M 0x21c858:592-598]");
        assert_eq!(fe.cursor, d, "Back re-opens the wheel on the chosen row [M 0x21c858:598]");
    }
}

// ── every unlocked row → the exact level request ─────────────────────────────────────────────

#[test]
fn every_unlocked_row_requests_its_campaign_level() {
    // The two ports of the game's sp_level table agree [M CAMPAIGN 0x2df2e0; SP_LEVEL_HASHES].
    for (i, &(_, _, _, level)) in CAMPAIGN.iter().enumerate() {
        assert_eq!(level, nf_game::progress::SP_LEVEL_HASHES[i], "CAMPAIGN row {i} == sp_level[{i}]");
    }

    // A fresh profile: rows 0 and 1 are unlocked [D sp_level's boot state].
    let mut fe = FrontEnd::new();
    fresh_profile_to_the_exchange(&mut fe);

    // [host --unlock-all] every row answers with its own sp_level hash; the driving rows show
    // the notice instead and the next on-foot row opens through them.
    let mut fe = FrontEnd::new();
    fe.unlock_all = true;
    to_missions(&mut fe);
    assert_eq!(fe.cursor, 11, "every row unlocked: the list opens on row 11 [M 0x20b608:60-64]");
    for i in 0..CAMPAIGN.len() {
        // Down from row 11 wraps to row 0 (the port's list step wraps over the unlocked rows).
        assert!(act(&mut fe, Action::Down), "Down to row {i}");
        assert_eq!(fe.cursor, i, "the wheel walks one row");
        let level = CAMPAIGN[i].3;
        let driving = nf_game::ptp::is_driving_level(level);
        assert_eq!(driving, menu::is_driving_level(level), "row {i}: the driving set matches");
        assert!(act(&mut fe, Action::Confirm), "row {i} confirms");
        if driving {
            assert!(fe.notice.is_some(), "row {i}: the driving notice shows");
            assert_eq!(fe.requested_level, None, "row {i}: no level request");
            assert_eq!(fe.flow.get_state(), menu::ST_RUNNER, "row {i}: the flow stays");
            // Confirm dismisses; a second driving row's notice is dismissed with Back too.
            assert!(act(&mut fe, Action::Confirm), "row {i}: Enter dismisses");
            assert!(fe.notice.is_none());
        } else {
            assert!(fe.notice.is_none(), "row {i}: an on-foot row starts at once");
            assert_eq!(
                fe.requested_level.take(),
                Some(level),
                "row {i} requests its sp_level hash"
            );
        }
    }
}

// ── a completed mission: results → save → the next row ───────────────────────────────────────

#[test]
fn a_completed_mission_routes_the_results_save_and_the_next_row() {
    let dir = card_dir("complete");
    let mut fe = FrontEnd::new();
    fe.set_profile_dir(dir.clone());
    // A saved codename first, through the real branches (the save prompt needs one).
    to_codename(&mut fe);
    assert!(act(&mut fe, Action::Up), "the page opened on row 1 [M 0x20aec0:96-97]; Up to New Codename");
    assert!(act(&mut fe, Action::Confirm), "New Codename");
    assert_eq!(fe.page, Page::CnName);
    type_on_grid(&mut fe, "BOND007");
    assert!(act(&mut fe, Action::Confirm), "the ✓ submits");
    assert_eq!(fe.page, Page::CnMenu);
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::CnSelect);
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Main);
    // fe-audit 1017: Main's wrap is [S unverified] — select NightFire directly.
    fe.cursor = 0;
    assert!(act(&mut fe, Action::Confirm), "NightFire");
    assert_eq!(fe.cursor, 1, "opened from Main on row 1 [M 0x20aec0:96-97]");
    assert!(act(&mut fe, Action::Down));
    assert_eq!(fe.cursor, 2, "BOND007's row");
    assert!(act(&mut fe, Action::Confirm), "the saved codename loads");
    assert_eq!(fe.page, Page::Difficulty);
    assert!(act(&mut fe, Action::Confirm), "Operative (the wheel's first row [M 0x216bf8:12])");
    assert_eq!(fe.page, Page::MissionSelect);
    assert_eq!(fe.cursor, 1, "the list opens on the highest unlocked row [M 0x20b608:60-64]");

    // Paris Prelude is the driving row: the notice, dismiss, The Exchange. The second Enter
    // is refused while the request stands — a second Enter used to start a second load.
    let level = {
        assert!(act(&mut fe, Action::Up), "Up to Paris Prelude");
        assert!(act(&mut fe, Action::Confirm), "Paris Prelude");
        assert!(fe.notice.is_some());
        assert!(act(&mut fe, Action::Confirm), "dismiss");
        assert!(act(&mut fe, Action::Down));
        assert!(act(&mut fe, Action::Confirm), "The Exchange starts");
        let l = fe.requested_level.expect("the request");
        assert_eq!(l, 0x0700_0005);
        assert!(!fe.action(Action::Confirm), "no second request");
        assert_eq!(fe.requested_level.take(), Some(l)); // [host] the level loads
        l
    };

    // [host] the level plays (the runner state under Mission_Update 0x182c60) and ends complete.
    let mut progress = nf_game::progress::Progress::default();
    progress.nightfire_status = fe.packed_unlocks(); // swap_in_scene's seed [M lib.rs 2036]
    progress.stats[0].dispatched = 30;
    progress.stats[0].shots_fired = 100;
    progress.stats[0].shots_hit_enemy = 100;
    fe.flow.stack = vec![menu::ST_RUNNER];
    fe.flow.current_level = level;
    fe.mission_ended(
        Outcome::Complete,
        Stats { shots_fired: 30, shots_hit: 20, accuracy_pct: 66, time_s: 100, ..Stats::default() },
    );
    // The Exchange has no outro code: the front-end map loads plain, the results page stands.
    assert_eq!(fe.requested_level.take(), Some(menu::FRONTEND_LEVEL), "no outro for 07000005");
    assert_eq!(drive_end_to_map(&mut fe), None, "a plain map load has no phase-2 resolve");
    // G-048(3): the load tail's MenuManager_Create lands the queued page once
    // [M 0x1d6360:160-163]; a second call on the open page is a no-op.
    assert!(fe.reopen_queued_page(), "the results page opens with the manager");
    assert_eq!(fe.page, Page::Results);
    assert_eq!(fe.flow.page, menu::PAGE_RESULTS);

    // The results fields + the award [M P_NFRESULTS 0x208e78 case 0x4c].
    assert!(fe.results_enter(&mut progress, level, 6, fe.difficulty as u32, None), "the fields build");
    assert!(fe.results_improved, "the commit improved (first completion)");
    assert!(fe.results.is_some());
    assert!(fe.new_reward, "a first medal stands [M 0x1ffa10:74-79]");
    assert!(fe.unlocked_row(1), "the completed row stamps");
    assert!(fe.unlocked_row(2), "the next row unlocks");
    assert!(fe.unlocked_row(4), "Double Cross opens through the chained driving rows");

    // Confirm #1 opens the save prompt (improved + a saved codename in play); #2 answers Yes.
    assert!(act(&mut fe, Action::Confirm), "the results' first Confirm");
    assert!(fe.results_box, "the save prompt stands [M 0x1000206]");
    assert!(act(&mut fe, Action::Confirm), "Yes requests the save");
    assert_eq!(fe.card_message(), Some((4, 0x27d)), "ASK_OVERWRITE [M 0x10ee38]");
    assert!(fe.card_answer(menu::BTN_YES), "the overwrite question's Yes");
    fe.tick();
    assert_eq!(fe.card_message(), None, "the flow closed on completion");
    // The routing hops to the 007 Bonus page while the new reward stands.
    assert_eq!(fe.page, Page::NfBonus);
    assert_eq!(fe.flow.page, menu::PAGE_NF_BONUS);
    fe.bonus_enter(&progress, fe.results_level, fe.new_reward); // [host]
    assert!(fe.bonus.is_some(), "the bonus fields built on the page enter");
    assert!(act(&mut fe, Action::Confirm), "the bonus page's Confirm");
    // → the mission list, opened on the HIGHEST unlocked row [M P_NFMAP 0x20b608 case 0x4c].
    assert_eq!(fe.page, Page::MissionSelect);
    assert_eq!(fe.flow.page, menu::PAGE_MISSION_SELECT);
    let want_cursor = (0..CAMPAIGN.len()).rev().find(|&i| fe.unlocked_row(i)).unwrap();
    assert_eq!(fe.cursor, want_cursor, "the list opens on the highest unlocked row");
    assert_eq!(want_cursor, 4, "Double Cross, through the two driving rows");
    // The next on-foot row is reachable and starts.
    assert!(act(&mut fe, Action::Confirm), "Double Cross starts");
    assert_eq!(fe.requested_level.take(), Some(0x0700_0001));

    // The written file carries the run's progress.
    let path = nf_game::profile::Profile::path_in(&dir, "BOND007");
    let back = nf_game::profile::Profile::load_file(&path).expect("the card's file loads");
    let status = back.progress.nightfire_status();
    assert_eq!(status & 0b111, 0b111, "rows 0-2 ride the save");

    // Reloading the codename brings the unlocks back. fe-audit 1019: the post-results list
    // rides a FRESH manager (the front-end reload's MenuManager_Create [M 0x1d6360:160-163]) —
    // the results/bonus confirms open 0x4000001c with b=1, no push [M 0x208e78:43;
    // P_NFBONUS 0x209670:51-56] — so the list's Back finds an EMPTY stack and P_NFMAP's 99 arm
    // opens MAIN, stack-clearing [M 0x20b608:67-68]. (The SP chain's pushed Back is the wheel —
    // back_walks…/each_difficulty… pin that side.)
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Main, "the empty-stack △ opens MAIN [M 0x20b608:67-68]");
    // Forward again through NightFire (row 0 → C_GONIGHTFIRE's delayed 0x44(0x40000025, 0)
    // [M 0x2169b8]) — the Codenames row would open P_CNSELECT 0x4000001b, not this page
    // [M C_GOCODENAMES 0x216b38:36-37 — the delayed 0x44 to 0x4000001b].
    assert!(act(&mut fe, Action::Confirm), "NightFire");
    assert_eq!(fe.page, Page::Codename);
    while fe.cursor < 2 {
        assert!(act(&mut fe, Action::Down));
    }
    assert_eq!(fe.cursor, 2);
    assert!(act(&mut fe, Action::Confirm), "the saved codename loads");
    assert_eq!(fe.page, Page::Difficulty);
    assert!(fe.unlocked_row(1) && fe.unlocked_row(2), "the card's unlocks came back");
    let _ = std::fs::remove_dir_all(&dir);
}

// ── a failed mission: the fail menu's restart and quit ───────────────────────────────────────

#[test]
fn a_failed_mission_restarts_and_quits_from_the_results_page() {
    let mut fe = FrontEnd::new();
    let level = fresh_profile_to_the_exchange(&mut fe);

    // [host] the level runs and FAILS.
    fe.flow.stack = vec![menu::ST_RUNNER];
    fe.flow.current_level = level;
    fe.mission_ended(Outcome::Failed, Stats::default());
    // fe-audit: the fail menu 0x40000042 opens over the PAUSED mission and nothing loads [M
    // Mission_Update 0x182c60:77-80]; REPLAY reads GameState+0xc (the mission) at answer time
    // [M P_ENDMISSION 0x216618:19-21].
    assert_eq!(fe.requested_level, None, "the failure requests no level [M 0x182c60:77-80]");
    assert_eq!(fe.flow.current_level, level, "the mission stays loaded under the menu");
    assert_eq!(fe.page, Page::Results);
    assert_eq!(fe.page_id(), menu::PAGE_FAIL_MENU);
    // fe-audit 1017: the fail menu's two rows are the DISC's own values — REPLAY carries value 0
    // and QUIT value 2 (msg-0x11 bindings 0x5bc/0 and 0x16c/2 of control 0x100001a4, mission def
    // 08000001 page 0x40000042 [D]); no value-1 row exists on the page, so P_ENDMISSION's BaseMap
    // arm (value 1 [M 0x216618:14-17]) is unreachable from it. REPLAY (value 0) is the warm
    // checkpoint restart [M 0x216618:19-21].
    assert_eq!(fe.rows(), 2, "the fail menu's REPLAY / QUIT [D the mission def's 0x11 rows]");
    assert_eq!(fe.cursor, 0);
    assert!(fe.action(Action::Back), "△ lands on the second row [S unverified — the option list's △]");
    assert_eq!(fe.cursor, 1);
    assert!(fe.action(Action::Up));
    assert_eq!(fe.cursor, 0);
    assert!(fe.action(Action::Confirm), "REPLAY");
    assert_eq!(fe.requested_level, Some(0x0700_0005), "the current level rides the restart");
    assert!(fe.flow.warm_reset, "the warm arm");
    assert_eq!(fe.requested_level.take(), Some(0x0700_0005)); // [host] the restart loads
    for _ in 0..120 {
        fe.tick();
    } // the loads land

    // QUIT: the front end, cold (the same answer-before-the-load window).
    let mut fe2 = FrontEnd::new();
    let level2 = fresh_profile_to_the_exchange(&mut fe2);
    fe2.flow.stack = vec![menu::ST_RUNNER];
    fe2.flow.current_level = level2;
    fe2.mission_ended(Outcome::Failed, Stats::default());
    assert_eq!(fe2.requested_level, None);
    assert!(fe2.action(Action::Down));
    assert_eq!(fe2.cursor, 1, "the QUIT row");
    assert!(fe2.action(Action::Confirm), "QUIT");
    assert_eq!(fe2.requested_level, Some(menu::FRONTEND_LEVEL), "QUIT loads the front end");
    assert_eq!(fe2.flow.page, menu::PAGE_RESULTS_QUIT_TARGET, "+0x1c = the mission list [M 0x216618:25]");
    assert_eq!(fe2.page, Page::MissionSelect, "the front end reopens on the mission list [M P_NFMAP 0x20b608:58-64]");
    for _ in 0..120 {
        fe2.tick();
    }
}

// ── the final mission: results → bonus → win → credits ───────────────────────────────────────

#[test]
fn the_final_mission_routes_to_the_win_page_and_credits() {
    let mut fe = FrontEnd::new();
    fe.unlock_all = true; // [host --unlock-all] the only way to Equinox without the campaign
    to_missions(&mut fe);
    assert_eq!(fe.cursor, 11, "Equinox — the highest unlocked row [M 0x20b608:60-64]");
    assert!(act(&mut fe, Action::Confirm), "Equinox starts");
    let level = fe.requested_level.take().expect("the request");
    assert_eq!(level, 0x0700_001b);

    // [host] the level plays and completes — the game's final mission.
    let mut progress = nf_game::progress::Progress::default();
    progress.nightfire_status = fe.packed_unlocks();
    progress.stats[0].dispatched = 15;
    progress.stats[0].shots_fired = 200;
    progress.stats[0].shots_hit_enemy = 200;
    fe.flow.stack = vec![menu::ST_RUNNER];
    fe.flow.current_level = level;
    fe.mission_ended(
        Outcome::Complete,
        Stats { shots_fired: 30, shots_hit: 20, accuracy_pct: 66, time_s: 100, ..Stats::default() },
    );
    assert_eq!(fe.requested_level.take(), Some(0x0730_001b), "the final level wraps to its outro");
    assert_eq!(drive_end_to_map(&mut fe), Some(menu::FRONTEND_LEVEL), "the outro resolves to the map");
    // G-048(3): the load tail's MenuManager_Create lands the queued page once
    // [M 0x1d6360:160-163]; a second call on the open page is a no-op.
    assert!(fe.reopen_queued_page(), "the results page opens with the manager");
    assert_eq!(fe.page, Page::Results);
    assert_eq!(fe.flow.page, menu::PAGE_RESULTS);
    assert!(fe.results_enter(&mut progress, level, 6, fe.difficulty as u32, None));
    assert!(fe.just_completed, "the final mission sets just_completed [M 0x1ffa10 tail]");
    // The first Confirm routes: a new reward stands → the 007 Bonus page first.
    assert!(act(&mut fe, Action::Confirm), "the results' first Confirm");
    if fe.new_reward {
        assert_eq!(fe.page, Page::NfBonus, "the new-reward hop [M 0x208e78:48-53]");
        fe.bonus_enter(&progress, fe.results_level, fe.new_reward); // [host]
        assert!(fe.bonus.is_some());
        assert!(act(&mut fe, Action::Confirm), "the bonus page's Confirm");
    }
    // just_completed → the win page [M 0x209670:46-57].
    assert_eq!(fe.page, Page::WinGame);
    assert_eq!(fe.flow.page, menu::PAGE_WIN_GAME);
    assert!(!act(&mut fe, Action::Up), "the win page takes no page input [M P_WINGAME 0x215100:21 msg 0x68/1]");
    // The movie is skippable at once (c49 = 1, grace 0 [M 0x215100:13-15]); the skip's next 0x50
    // opens the credits [M :23-48].
    assert!(fe.action(Action::Confirm), "the skip");
    assert_eq!(fe.page, Page::Credits, "the credits open with the win source [M 0x215100:48]");
    assert!(!fe.win_movie_finished(), "sister: the host's movie-end call is inert after the skip");
}

// ── menu-dossier-navigation: the Dossier's page stack [M 0x21c858] ───────────────────────────

/// A completed mission's results page, through the real boot → main → codename → difficulty →
/// mission → runner → results path (the same walk a_completed_mission… takes, without the save).
fn completed_results(fe: &mut FrontEnd, dir: std::path::PathBuf) -> u32 {
    fe.set_profile_dir(dir);
    to_missions(fe);
    assert_eq!(fe.page, Page::MissionSelect);
    // The Exchange (row 1) completes: the driving notice on row 0 dismissed, the row picked.
    assert!(act(fe, Action::Up), "Up to Paris Prelude (the list opens on row 1 [M 0x20b608:60-64])");
    assert!(act(fe, Action::Confirm), "Paris Prelude's driving notice");
    assert!(fe.notice.is_some());
    assert!(act(fe, Action::Confirm), "dismiss the driving notice");
    assert!(act(fe, Action::Down));
    assert!(act(fe, Action::Confirm), "The Exchange starts");
    let level = fe.requested_level.take().expect("the level request");
    let mut progress = nf_game::progress::Progress::default();
    progress.nightfire_status = fe.packed_unlocks();
    fe.flow.stack = vec![menu::ST_RUNNER];
    fe.flow.current_level = level;
    fe.mission_ended(Outcome::Complete, Stats::default());
    assert_eq!(fe.requested_level.take(), Some(menu::FRONTEND_LEVEL));
    assert_eq!(drive_end_to_map(fe), None);
    assert!(fe.reopen_queued_page(), "the results page opens with the fresh manager [M 0x1d6360:160-163]");
    assert!(fe.results_enter(&mut progress, level, 6, fe.difficulty as u32, None), "the results build");
    assert_eq!(fe.page, Page::Results);
    level
}

/// The strand: Results → □/Tab (msg 0x5d) → the Dossier → Back MUST return to the Results page
/// the Tab came from — the game's 0x5d runs Manager_SendMessage(0x44, 0x4000002b, 0) whose
/// lparam 0 pushes the opener {page, cursor} onto the manager's stack [M P_NFRESULTS 0x208e78:
/// 222-230 + the push rule 0x21c858:262-274; the pop 0x21c858:582-602]. The old push-less hop
/// left the Dossier over an empty stack whose 0x5f does nothing [M 0x21c858:604-610 +
/// P_DOSSIER 0x216de0's 0x4c/0x50-only arms] — the port fell through to the mission list and
/// the player was stranded away from the results.
#[test]
fn the_results_tab_opens_the_dossier_and_back_returns_to_the_results() {
    let dir = card_dir("dossier-strand");
    let mut fe = FrontEnd::new();
    completed_results(&mut fe, dir.clone());
    let results_cursor = fe.cursor;
    // □/Tab (the "~Y Dossier" hint) opens the Dossier — the wheel resets to Records [M 0x216de0:9-16].
    assert!(act(&mut fe, Action::Side), "Tab opens the Dossier [M 0x208e78:222-230]");
    assert_eq!(fe.page, Page::Dossier);
    assert_eq!(fe.cursor, 0, "the Dossier wheel opens at Records [M 0x216de0:9-16]");
    // Back pops the opener: the RESULTS page with its saved cursor [M 0x21c858:592-600].
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::Results, "Back returns to the page the Tab came from [M 0x21c858:592-600]");
    assert_eq!(fe.cursor, results_cursor, "the results cursor rode the pop [M 0x21c858:598-600]");
    assert_eq!(fe.flow.page, menu::PAGE_RESULTS);
    // …and the results page still works (its own ✕/Tab arms re-run).
    assert!(act(&mut fe, Action::Extra), "the ~X Stats arm [M 0x208e78:225-230]");
    assert_eq!(fe.page, Page::Stats);
    assert!(act(&mut fe, Action::Back), "Stats' Back pops the Results opener");
    assert_eq!(fe.page, Page::Results);
    // The Results page's own empty-stack △ does nothing: P_NFRESULTS 0x208e78 arms only
    // 0x4b/0x4c/0x50/0x5d/0x5e [M] — the pop's msg-99 reaches no arm, the page stays.
    let _ = fe.action(Action::Back);
    assert_eq!(fe.page, Page::Results, "the empty-stack 0x5f strands nobody [M 0x21c858:604-610]");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The Dossier family walks the stack exactly as the game runs it [M C_SBDOSSIER 0x216eb0:12-32,
/// the push rule 0x21c858:262-274, the pop 0x21c858:582-602]: the mission list □ → the Dossier →
/// each sub-page (Records/Rewards immediate, Gadgets/Weapons through the CloseIris delay) →
/// Back → the Dossier wheel on the row that opened the page → Back → the mission list with ITS
/// saved cursor. No hop in the walk strands the player.
#[test]
fn the_dossier_family_walks_back_through_the_page_stack() {
    let mut fe = FrontEnd::new();
    to_missions(&mut fe);
    let list_cursor = fe.cursor;
    assert!(act(&mut fe, Action::Side), "□ opens the Dossier [M C_SBNFMAP 0x20b8d8:47-48]");
    assert_eq!(fe.page, Page::Dossier);
    for (row, want) in [(0usize, Page::Records), (1, Page::Rewards), (2, Page::Gadgets), (3, Page::Weapons)] {
        // Walk the wheel to the row (it wraps [M 0x221b20:186-210] — walk forward from 0).
        while fe.cursor != row {
            assert!(act(&mut fe, Action::Down), "the wheel wraps [M 0x2221b20:186-210]");
        }
        assert!(act(&mut fe, Action::Confirm), "row {row} opens its page [M 0x216eb0:12-32]");
        assert_eq!(fe.page, want, "the wheel's row {row} page");
        // Back pops the Dossier opener with the wheel's saved row [M 0x21c858:582-602].
        assert!(act(&mut fe, Action::Back));
        assert_eq!(fe.page, Page::Dossier);
        assert_eq!(fe.cursor, row, "the wheel returns to the row that opened the page [M 0x21c858:598-600]");
    }
    // The Dossier's own Back pops the mission list opener with its saved cursor
    // [M 0x21c858:582-602; the list's □ 0x44 carried lparam 0 through the CloseIris delay
    // [M 0x20b8d8:48 + 0x202190]].
    assert!(act(&mut fe, Action::Back));
    assert_eq!(fe.page, Page::MissionSelect);
    assert_eq!(fe.cursor, list_cursor, "the list cursor rode the pop [M 0x21c858:598-600]");
}
