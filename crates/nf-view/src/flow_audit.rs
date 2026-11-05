//! The start-to-mission path as a player takes it, driven through the App at real display rates:
//! boot logos, title, attract, the menus, the briefing, the mission's pause, fail and end, and
//! back to the menus. Lengths come off the image, so these need the movie-bearing `NF_ISO`.
use super::*;
use crate::headless_app_harness::HeadlessApp;
use menu::{Outcome, Page};
use winit::keyboard::KeyCode as K;

struct Player { h: HeadlessApp, hz: u32 }

impl Player {
    fn boot(iso: &str, hz: u32) -> Player { Player { h: HeadlessApp::boot(iso), hz } }
    fn mission(iso: &str, level: &str, hz: u32) -> Player { Player { h: HeadlessApp::mission(iso, level), hz } }

    fn app(&self) -> &App { &self.h.app }
    fn fe(&self) -> &menu::FrontEnd { self.h.app.fe.as_ref().expect("a front end") }
    fn page(&self) -> Option<Page> { self.h.app.fe.as_ref().map(|f| f.page) }
    fn sim(&mut self) -> &mut scene::Sim { self.h.app.scene.sim.as_mut().expect("a sim") }
    fn key(&mut self, k: K) { self.h.key(k); }
    fn exit_requested(&self) -> bool { self.h.exit_requested }

    /// `secs` of game time at the display rate.
    fn wait(&mut self, secs: f32) {
        for _ in 0..(secs * self.hz as f32).round() as u32 { self.h.frame(1.0 / self.hz as f32); }
    }

    /// Game seconds until `pred`, None past `max`.
    fn until(&mut self, max: f32, mut pred: impl FnMut(&App) -> bool) -> Option<f32> {
        let n = (max * self.hz as f32) as u32;
        for i in 0..n {
            if pred(&self.h.app) { return Some(i as f32 / self.hz as f32); }
            self.h.frame(1.0 / self.hz as f32);
        }
        None
    }

    /// Presses `k` every tenth of a second until the front end satisfies `pred`: the input block
    /// swallows a press right after a page change [M 0x21c3d8:70].
    fn press(&mut self, k: K, what: &str, mut pred: impl FnMut(&menu::FrontEnd) -> bool) {
        for _ in 0..80 {
            if self.h.app.fe.as_ref().is_some_and(&mut pred) { return; }
            self.key(k);
            self.wait(0.1);
        }
        panic!("{what}: page {:?}", self.page());
    }

    /// The load's hold is wall time (`LOADING_HOLD`).
    fn finish_load(&mut self) {
        let t0 = std::time::Instant::now();
        while self.h.app.async_load.is_some() {
            assert!(t0.elapsed().as_secs_f32() < 60.0, "the load never finished");
            self.h.frame(1.0 / self.hz as f32);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    fn title(&mut self) -> f32 {
        self.until(40.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::Start && !f.intro_playing)).expect("the title")
    }

    fn main_menu(&mut self) {
        self.title();
        self.key(K::Enter);
        self.until(5.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::EstHero)).expect("the start movie");
        self.wait(3.2); // past its 180-tick grace [M 0x215a50:6]
        self.press(K::Enter, "the main menu", |f| f.page == Page::Main && !f.intro_playing);
    }

    /// Main → Night Fire → Play without saving → Operative → the mission list.
    fn mission_list(&mut self) {
        self.press(K::Enter, "the codename page", |f| f.page == Page::Codename);
        self.press(K::Enter, "the difficulty page", |f| f.page == Page::Difficulty);
        self.press(K::Enter, "the mission list", |f| f.page == Page::MissionSelect);
        self.wait(0.3);
    }

    fn pick(&mut self, row: usize) {
        let k = if self.fe().cursor > row { K::ArrowUp } else { K::ArrowDown };
        self.press(k, "the row", |f| f.cursor == row);
        self.press(K::Enter, "the pick", |f| f.flow.movie_pending);
    }

    fn briefing_up(&mut self) {
        self.until(5.0, |a| a.fe.as_ref().is_some_and(|f| f.flow.get_state() == menu::ST_MOVIE)).expect("the briefing");
    }

    /// Pause > QUIT > YES, back on the mission list.
    fn quit_to_list(&mut self) {
        self.key(K::Escape);
        self.wait(0.1);
        assert_eq!(self.page(), Some(Page::Pause));
        self.press(K::ArrowDown, "the QUIT row", |f| f.cursor == 2);
        self.key(K::Enter);
        self.wait(0.1);
        assert!(self.fe().confirming_quit);
        self.key(K::ArrowUp);
        self.key(K::Enter);
        self.wait(0.1);
        self.finish_load();
        self.wait(2.0);
        assert_eq!(self.app().level_code, menu::FRONTEND_LEVEL);
        assert_eq!(self.page(), Some(Page::MissionSelect));
    }

    fn sound_ids(&self) -> Vec<u32> { self.app().sound.as_ref().map_or(vec![], |s| s.driver.ids()) }
    fn jingle_up(&self) -> bool { self.sound_ids().iter().any(|i| matches!(i, 0x1d7 | 0x470)) }
    fn music_up(&self) -> bool { self.app().sound.as_ref().is_some_and(|s| s.music.is_some()) }
    fn movie(&self) -> Option<(u32, bool)> { self.app().movie.as_ref().map(|p| (p.code(), p.is_looping())) }
}

fn iso() -> Option<String> {
    let iso = std::env::var("NF_ISO").ok()?;
    movie::Player::start(&iso, menu::INTRO_MOVIES[0], 100).is_ok().then_some(iso)
}

/// The movie's length in seconds, off the image.
fn secs(iso: &str, code: u32) -> f32 {
    match movie::Player::start(iso, code, 100) {
        Ok(p) => p.length_ticks() as f32 / 60.0,
        Err(_) => panic!("the movie {code:08x}"),
    }
}

fn near(got: f32, want: f32, what: &str) {
    // No picture is pulled here, so each hold waits out its half second for the first one.
    assert!(got > want - 0.25 && got < want + 1.25, "{what}: {got:.2} s, the movie runs {want:.2} s");
}

/// Boot to the main menu, the long way: both logos, the title's 45 s, the trailer, the start movie.
#[test]
fn the_front_end_movies_run_their_length_and_the_music_comes_back() {
    let Some(iso) = iso() else { return };
    for hz in [60, 240] {
        let mut p = Player::boot(&iso, hz);
        let logos = p.title();
        near(logos, secs(&iso, menu::INTRO_MOVIES[0]) + secs(&iso, menu::INTRO_MOVIES[1]), "the boot logos");
        p.wait(0.1);
        assert!(p.jingle_up(), "{hz} Hz: the title has its music: {:x?}", p.sound_ids());

        let idle = p.until(50.0, |a| a.fe.as_ref().is_some_and(|f| f.attract_playing)).expect("the attract");
        assert!((idle - 44.9).abs() < 0.3, "{hz} Hz: the attract opened after {idle} s [M 0xa8c ticks]");
        p.wait(0.1);
        assert!(!p.jingle_up(), "{hz} Hz: the trailer has its own sound");
        let code = p.fe().attract_movie;
        let run = p.until(200.0, |a| a.fe.as_ref().is_some_and(|f| !f.attract_playing)).expect("the trailer's end");
        near(run + 0.1, secs(&iso, code), "the trailer");
        p.wait(0.2);
        assert_eq!(p.page(), Some(Page::Start));
        assert!(p.jingle_up(), "{hz} Hz: the title's music is back after the trailer [M 0x20a1f0:45-50]: {:x?}", p.sound_ids());

        p.key(K::Enter);
        p.until(5.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::EstHero)).expect("the start movie");
        let run = p.until(100.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::Main)).expect("the main menu");
        near(run, secs(&iso, menu::ESTHERO_MOVIE), "the start movie");
        p.wait(0.2);
        assert_eq!(p.movie(), Some((0x0733_0048, true)), "{hz} Hz: the main menu's backdrop");
        assert!(p.jingle_up(), "{hz} Hz: the main menu's music");
    }
}

/// The trailer from the main menu, skipped: the menu is as it was left.
#[test]
fn a_skipped_trailer_returns_to_the_main_menu() {
    let Some(iso) = iso() else { return };
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.until(50.0, |a| a.fe.as_ref().is_some_and(|f| f.attract_playing)).expect("the attract");
    p.wait(2.0);
    p.key(K::Enter);
    p.wait(0.3);
    assert_eq!(p.page(), Some(Page::Main));
    assert!(!p.fe().attract_playing);
    assert!(p.jingle_up(), "{:x?}", p.sound_ids());
    assert_eq!(p.movie(), Some((0x0733_0048, true)));
}

/// State 6 holds the briefing for its length at any rate. Below 60 Hz the old poll ran it out
/// early; at 120 Hz and up it never counted at all.
#[test]
fn the_briefing_holds_its_length_at_any_display_rate() {
    let Some(iso) = iso() else { return };
    let want = secs(&iso, 0x0710_0005);
    for hz in [30, 60, 144, 240] {
        let mut p = Player::boot(&iso, hz);
        p.main_menu();
        p.mission_list();
        p.pick(1);
        p.briefing_up();
        let run = p.until(want + 10.0, |a| a.fe.as_ref().is_none_or(|f| f.flow.get_state() != menu::ST_MOVIE));
        near(run.unwrap_or(f32::MAX), want, &format!("{hz} Hz briefing"));
        p.wait(0.1);
        p.finish_load();
        assert_eq!(p.app().level_code, 0x0700_0005);
        assert!(p.app().fe.is_none(), "{hz} Hz: the mission is up");
    }
}

/// The mission's end: the outro at its length with the mission's sound gone, then the results,
/// the list and the next briefing.
#[test]
fn the_outro_runs_alone_then_the_results_lead_to_the_next_mission() {
    let Some(iso) = iso() else { return };
    let want = secs(&iso, 0x0730_0004);
    for hz in [30, 240] {
        let mut p = Player::mission(&iso, "07000004.bin", hz);
        p.wait(1.0);
        assert!(p.music_up(), "the level's music is up");
        p.sim().drones.mission_end = Some(0x0730_0004);
        p.until(5.0, |a| a.fe.as_ref().is_some_and(|f| f.flow.get_state() == menu::ST_MOVIE)).expect("the outro");
        p.wait(0.2);
        assert!(p.sound_ids().is_empty(), "{hz} Hz: the mission's loops under the outro [M 0x1d6360:24]: {:x?}", p.sound_ids());
        assert!(!p.music_up(), "{hz} Hz: the mission's music under the outro");
        let run = p.until(want + 10.0, |a| a.fe.as_ref().is_some_and(|f| f.flow.get_state() != menu::ST_MOVIE));
        near(run.unwrap_or(f32::MAX) + 0.2, want, &format!("{hz} Hz outro"));
        p.wait(0.1);
        p.finish_load();
        p.wait(0.5);
        assert_eq!(p.page(), Some(Page::Results), "{hz} Hz: the results open after the outro");
        assert_eq!(p.fe().outcome, Outcome::Complete);
        assert!(p.jingle_up(), "{hz} Hz: the results page's music: {:x?}", p.sound_ids());
        p.press(K::Enter, "the mission list", |f| f.page == Page::MissionSelect);
        p.wait(0.2);
        p.pick(5);
        p.briefing_up();
        p.key(K::ArrowDown);
        p.until(2.0, |a| a.async_load.is_some()).expect("the skip");
        p.finish_load();
        assert_eq!(p.app().level_code, 0x0700_0009, "{hz} Hz: the next mission");
        assert!(p.app().fe.is_none());
    }
}

/// A dead Bond meets the fail menu over the paused mission [M 0x182c60:77-80]; its rows replay
/// the level or quit to the list [M 0x216618:19-27].
#[test]
fn a_death_opens_the_fail_menu_and_both_rows_work() {
    let Some(iso) = iso() else { return };
    let mut p = Player::mission(&iso, "07000006.bin", 240);
    p.wait(1.0);
    p.sim().drones.health.health = 0.0;
    let failed = |a: &App| a.fe.as_ref().is_some_and(|f| f.page == Page::Results && f.outcome == Outcome::Failed);
    p.until(30.0, failed).expect("the fail menu");
    assert!(p.app().async_load.is_none() && p.app().pending_level.is_none(), "the menu waits for the player");
    let frame = p.sim().drones.frame;
    p.wait(1.0);
    assert_eq!(p.sim().drones.frame, frame, "the mission is paused under the menu");
    assert_eq!(p.movie(), None, "no menu backdrop over the mission");
    p.key(K::Enter); // REPLAY
    p.wait(0.1);
    p.finish_load();
    assert!(p.app().fe.is_none());
    assert_eq!(p.app().level_code, 0x0700_0006, "REPLAY reloads the level");
    p.wait(2.0);
    assert!(p.sim().drones.health.health > 0.0, "Bond is back");
    assert!(!p.sim().drones.fail_menu_open());

    // A failed objective takes the same menu.
    let frame = p.sim().drones.frame;
    p.sim().drones.channels.set(99, 1, frame);
    p.until(30.0, failed).expect("the fail menu after the fail channel");
    p.key(K::ArrowDown);
    p.wait(0.1);
    p.key(K::Enter); // QUIT
    p.wait(0.1);
    p.finish_load();
    p.wait(2.0);
    assert_eq!(p.app().level_code, menu::FRONTEND_LEVEL);
    assert_eq!(p.page(), Some(Page::MissionSelect));
    assert!(p.jingle_up(), "{:x?}", p.sound_ids());
}

/// Esc on the pause's YES/NO box or hint box closes that box, not the pause [M 0x2022a0].
#[test]
fn esc_closes_the_pause_boxes_before_the_pause() {
    let Some(iso) = iso() else { return };
    let mut p = Player::mission(&iso, "07000005.bin", 240);
    p.wait(0.5);
    p.key(K::Escape);
    p.wait(0.1);
    p.press(K::ArrowDown, "the QUIT row", |f| f.cursor == 2);
    p.key(K::Enter);
    p.wait(0.1);
    assert!(p.fe().confirming_quit);
    p.key(K::Escape);
    p.wait(0.1);
    assert_eq!(p.page(), Some(Page::Pause), "Esc on the QUIT box stays on the pause page");
    assert!(!p.fe().confirming_quit);
    p.key(K::ArrowUp);
    p.key(K::Enter);
    p.wait(0.1);
    assert!(p.fe().confirming_restart);
    p.key(K::Escape);
    p.wait(0.1);
    assert_eq!(p.page(), Some(Page::Pause), "Esc on the RESTART box stays on the pause page");
    p.h.app.fe.as_mut().unwrap().pause_hint = Some(0x31d);
    p.key(K::Escape);
    p.wait(0.1);
    assert_eq!(p.page(), Some(Page::Pause), "Esc on the hint box stays on the pause page");
    assert!(p.fe().pause_hint.is_none());
    p.key(K::Escape);
    p.wait(0.1);
    assert!(p.app().fe.is_none(), "the next Esc resumes");
}

/// Pause holds the world and the sound, resume lets go, RESTART reloads the mission.
#[test]
fn pause_holds_the_mission_and_restart_reloads_it() {
    let Some(iso) = iso() else { return };
    let mut p = Player::mission(&iso, "07000005.bin", 240);
    p.wait(1.0);
    let f0 = p.sim().drones.frame;
    p.wait(1.0);
    let ran = p.sim().drones.frame - f0;
    assert!((58..=62).contains(&ran), "60 ticks a second at 240 Hz, got {ran}");
    p.key(K::Escape);
    p.wait(0.1);
    assert!(p.app().audio_paused, "the pause holds the sound [M 0x21b908:121-124]");
    let f0 = p.sim().drones.frame;
    p.wait(1.0);
    assert_eq!(p.sim().drones.frame, f0, "the pause holds the sim");
    p.key(K::Escape);
    p.wait(0.5);
    assert!(!p.app().audio_paused && p.app().fe.is_none());
    assert!(p.sim().drones.frame > f0);
    p.key(K::Escape);
    p.wait(0.1);
    p.press(K::ArrowDown, "the RESTART row", |f| f.cursor == 1);
    p.key(K::Enter);
    p.wait(0.1);
    p.key(K::ArrowUp);
    p.key(K::Enter);
    p.wait(0.1);
    p.finish_load();
    assert_eq!(p.app().level_code, 0x0700_0005);
    assert!(p.app().fe.is_none());
    assert!(p.sim().drones.frame < 60, "a fresh mission");
    p.wait(0.5);
    assert!(p.music_up(), "the mission's music restarted");
    assert!(!p.app().audio_paused);
}

/// A mouse button let go over the pause page reaches no game arm: it must not stay down.
#[test]
fn a_held_button_does_not_survive_the_pause() {
    let Some(iso) = iso() else { return };
    let mut p = Player::mission(&iso, "07000005.bin", 60);
    p.wait(0.2);
    p.h.app.trigger = true;
    p.h.app.zoom = true;
    p.key(K::Escape);
    p.wait(0.1);
    p.key(K::Escape);
    p.wait(0.1);
    assert!(!p.app().trigger, "the gun fires on after the pause");
    assert!(!p.app().zoom, "the scope stays up after the pause");
}

/// The refused pause (a script camera is up [M 0x193600:72]) leaves the mouse with the mission.
#[test]
fn esc_in_a_cinema_keeps_the_mouse() {
    let Some(iso) = iso() else { return };
    let mut p = Player::mission(&iso, "0700001b.bin", 60);
    p.h.app.grabbed = true;
    p.until(60.0, |a| a.scene.sim.as_ref().is_some_and(|s| s.script_camera().is_some())).expect("the intro cinema");
    p.key(K::Escape);
    p.wait(0.1);
    assert!(p.app().fe.is_none(), "no pause over the cinema");
    assert!(p.app().grabbed, "the mouse stays grabbed");
}

/// A saved gold medal rides the load the briefing hands over: the kind-6 gate reads it
/// [M 0x18dda0:93-95]. The front end is gone by then.
#[test]
fn the_saved_medals_reach_a_mission_started_from_the_menus() {
    let Some(iso) = iso() else { return };
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.mission_list();
    let word = (0..64).map(|b| 1u64 << b).find(|&w| nf_game::progress::has_gold_medal(w, 0x0700_0005)).expect("the gold bit");
    let mut profile = nf_game::profile::Profile::default_codename("BOND");
    profile.progress.bonus[0] = word;
    let dir = std::env::temp_dir().join(format!("nf-flow-medal-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let fe = p.h.app.fe.as_mut().unwrap();
    fe.set_profile_dir(dir.clone());
    fe.profile = Some(profile);
    p.pick(1);
    p.briefing_up();
    p.key(K::ArrowDown);
    p.until(2.0, |a| a.async_load.is_some()).expect("the skip");
    p.finish_load();
    let count = |s: scene::Scene| s.sim.as_ref().unwrap().drones.pickups.len();
    let with = count(crate::load_with_carry_medals(&iso, "07000005.bin", false, None, 1, word).unwrap());
    let without = count(crate::load_with_carry_medals(&iso, "07000005.bin", false, None, 1, 0).unwrap());
    assert!(with > without, "the level has a medal-gated pickup: {with} vs {without}");
    assert_eq!(p.sim().drones.pickups.len(), with, "the medal word reached the load");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Every page the main menu leads to, in and back out by Esc.
#[test]
fn every_menu_page_opens_and_backs_out() {
    let Some(iso) = iso() else { return };
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.wait(0.5);

    // Night Fire: down to the list, the Dossier's four pages, back up to the main menu.
    p.mission_list();
    p.press(K::Tab, "the Dossier", |f| f.page == Page::Dossier);
    p.wait(0.3);
    for (row, page) in [(0, Page::Records), (1, Page::Rewards), (2, Page::Gadgets), (3, Page::Weapons)] {
        p.press(K::ArrowDown, "the Dossier row", |f| f.cursor == row);
        p.press(K::Enter, "a Dossier page", |f| f.page == page);
        p.wait(0.3);
        p.press(K::Escape, "back to the Dossier", |f| f.page == Page::Dossier);
        assert_eq!(p.fe().cursor, row, "the wheel keeps its row");
        p.wait(0.1);
    }
    p.press(K::Escape, "back to the list", |f| f.page == Page::MissionSelect);
    p.press(K::Escape, "back to the difficulty", |f| f.page == Page::Difficulty);
    p.press(K::Escape, "back to the codenames", |f| f.page == Page::Codename);
    p.press(K::Escape, "back to the main menu", |f| f.page == Page::Main);
    p.wait(0.2);
    assert!(p.jingle_up(), "{:x?}", p.sound_ids());

    // Multiplayer's first page.
    p.press(K::ArrowDown, "the MP row", |f| f.cursor == 1);
    p.press(K::Enter, "the MP mode page", |f| f.page == Page::MpMode);
    p.press(K::Escape, "back to the main menu", |f| f.page == Page::Main);
    p.wait(0.1);

    // Edit Codename: the wheel's pages, the AV's Screen Adjust and Credits.
    p.press(K::ArrowDown, "the codename row", |f| f.cursor == 2);
    p.press(K::Enter, "Edit Codename", |f| f.page == Page::CnSelect);
    p.wait(0.3);
    p.press(K::Enter, "the wheel", |f| f.page == Page::CnMenu);
    for (row, page) in [(1, Page::CnControls), (3, Page::CnAdvanced), (4, Page::CnMp), (5, Page::CnAv)] {
        let k = if p.fe().cursor < row { K::ArrowDown } else { K::ArrowUp };
        p.press(k, "the wheel row", |f| f.cursor == row);
        p.press(K::Enter, "an options page", |f| f.page == page);
        p.wait(0.1);
        p.press(K::Escape, "back to the wheel", |f| f.page == Page::CnMenu);
        assert_eq!(p.fe().cursor, row, "the wheel keeps its row");
    }
    p.press(K::Enter, "the AV page", |f| f.page == Page::CnAv);
    let adjust = menu::AV_ROWS.iter().position(|r| r.ctrl == 0x1000_0137).unwrap();
    p.press(K::ArrowDown, "the Screen Adjust row", |f| f.cursor == adjust);
    p.press(K::Enter, "Screen Adjust", |f| f.page == Page::ScreenAdjust);
    p.press(K::Escape, "back to the AV page", |f| f.page == Page::CnAv);
    let credits = menu::AV_ROWS.iter().position(|r| r.ctrl == 0x1000_0138).unwrap();
    let k = if p.fe().cursor < credits { K::ArrowDown } else { K::ArrowUp };
    p.press(k, "the Credits row", |f| f.cursor == credits);
    p.press(K::Enter, "the credits", |f| f.page == Page::Credits);
    p.wait(0.5);
    assert!(!p.jingle_up(), "the credits have their own track");
    assert!(p.music_up(), "the credits' track");
    // The roll cannot be skipped [M 0x2152b0's switch]: it ends on its own.
    p.key(K::Escape);
    p.wait(1.0);
    assert_eq!(p.page(), Some(Page::Credits), "Esc does not cut the roll");
    let mut lowest = i32::MAX;
    p.until(900.0, |a| {
        lowest = lowest.min(a.sound.as_ref().map_or(i32::MAX, |s| s.driver.music_master));
        a.fe.as_ref().is_some_and(|f| f.page == Page::CnAv)
    }).expect("the roll's end");
    assert!(lowest < 10, "the roll's music fades out [M 0x2152b0:165-168], lowest {lowest}");
    p.wait(0.3);
    assert_eq!(p.app().sound.as_ref().unwrap().driver.music_master, p.fe().opt_value(0x1000_0135), "the music level is back");
    assert!(p.jingle_up(), "the menu music after the credits: {:x?}", p.sound_ids());
    assert!(!p.music_up(), "the credits' track stopped");
    p.press(K::Escape, "back to the wheel", |f| f.page == Page::CnMenu);
    p.press(K::Escape, "back to Edit Codename", |f| f.page == Page::CnSelect);
    p.press(K::Escape, "back to the main menu", |f| f.page == Page::Main);
    p.wait(0.1);
    p.key(K::Escape);
    p.wait(0.2);
    assert_eq!(p.page(), Some(Page::Main), "Esc on Main holds [M P_MAIN_Handler 0x20a7b0]");
    assert!(!p.exit_requested(), "no page quits the game");
}

/// An AV edit kept by the confirm reaches the option file; one backed out by Esc does not.
#[test]
fn an_option_edit_persists_and_a_backed_out_one_does_not() {
    let Some(iso) = iso() else { return };
    let dir = std::env::temp_dir().join(format!("nf-flow-options-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.wait(0.5);
    p.press(K::ArrowDown, "the codename row", |f| f.cursor == 2);
    p.press(K::Enter, "Edit Codename", |f| f.page == Page::CnSelect);
    p.wait(0.3);
    p.press(K::Enter, "the wheel", |f| f.page == Page::CnMenu);
    p.press(K::ArrowDown, "the AV row", |f| f.cursor == 5);
    p.press(K::Enter, "the AV page", |f| f.page == Page::CnAv);
    let music = 0x1000_0135;
    let row = menu::AV_ROWS.iter().position(|r| r.ctrl == music).unwrap();
    let k = if p.fe().cursor < row { K::ArrowDown } else { K::ArrowUp };
    p.press(k, "the music row", |f| f.cursor == row);
    let was = p.fe().opt_value(music);
    p.key(K::ArrowLeft);
    p.wait(0.1);
    let edited = p.fe().opt_value(music);
    assert_ne!(edited, was, "the left key moves the slider");
    assert_eq!(p.app().sound.as_ref().unwrap().driver.music_master, edited, "the slider reaches the sound");
    // Set beside each save: another test in this process may move it.
    std::env::set_var("NF_CONFIG_DIR", &dir);
    p.key(K::Enter);
    assert_eq!(p.page(), Some(Page::CnMenu));
    assert_eq!(settings::load_default().0.get(&music), Some(&edited), "the confirm keeps the edit in the file");

    p.wait(0.1);
    p.key(K::Enter);
    p.wait(0.1);
    assert_eq!(p.page(), Some(Page::CnAv));
    let k = if p.fe().cursor < row { K::ArrowDown } else { K::ArrowUp };
    p.press(k, "the music row", |f| f.cursor == row);
    p.key(K::ArrowLeft);
    p.wait(0.1);
    assert_ne!(p.fe().opt_value(music), edited);
    std::env::set_var("NF_CONFIG_DIR", &dir);
    p.key(K::Escape);
    assert_eq!(p.fe().opt_value(music), edited, "Esc restores the value");
    assert_eq!(settings::load_default().0.get(&music), Some(&edited), "the file holds the kept value");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A codename made on the name page saves, lists, and loads back.
#[test]
fn a_new_codename_saves_and_loads_from_the_list() {
    let Some(iso) = iso() else { return };
    let dir = std::env::temp_dir().join(format!("nf-flow-card-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.h.app.fe.as_mut().unwrap().set_profile_dir(dir.clone());
    p.press(K::Enter, "the codename page", |f| f.page == Page::Codename);
    p.press(K::ArrowUp, "New Codename", |f| f.cursor == 0);
    p.press(K::Enter, "the name page", |f| f.page == Page::CnName);
    p.wait(0.3);
    for k in [K::KeyQ, K::KeyZ] { p.key(k); p.wait(0.05); }
    assert_eq!(p.fe().cn_typed_name(), "QZ");
    p.press(K::Enter, "the save", |f| f.page != Page::CnName);
    p.wait(1.0);
    eprintln!("after the name page: {:?}", p.page());
    assert_eq!(p.fe().active_profile.as_deref(), Some("QZ"), "page {:?}", p.page());
    assert!(p.fe().profiles.iter().any(|n| n == "QZ"), "the card lists it");

    // Back out, pick it on the Night Fire list, finish a mission, save at the results.
    p.press(K::Escape, "the main menu", |f| f.page == Page::Main);
    p.wait(0.2);
    p.press(K::ArrowUp, "the Night Fire row", |f| f.cursor == 0);
    p.press(K::Enter, "the codename page", |f| f.page == Page::Codename);
    p.wait(0.3);
    p.press(K::ArrowDown, "the QZ row", |f| f.cursor == 2);
    p.press(K::Enter, "the difficulty page", |f| f.page == Page::Difficulty);
    p.press(K::Enter, "the mission list", |f| f.page == Page::MissionSelect);
    p.wait(0.3);
    p.pick(1);
    p.briefing_up();
    p.key(K::ArrowDown);
    p.until(2.0, |a| a.async_load.is_some()).expect("the skip");
    p.finish_load();
    p.wait(1.0);
    p.sim().drones.mission_end = Some(menu::FRONTEND_LEVEL);
    p.until(5.0, |a| a.async_load.is_some()).expect("the end load");
    p.finish_load();
    p.wait(0.5);
    assert_eq!(p.page(), Some(Page::Results));
    assert_eq!(p.fe().active_profile.as_deref(), Some("QZ"), "the codename rides the mission");
    let saved = |d: &std::path::Path| std::fs::metadata(nf_game::profile::Profile::path_in(d, "QZ")).unwrap().modified().unwrap();
    let before = saved(&dir);
    std::thread::sleep(std::time::Duration::from_millis(50));
    p.key(K::Enter);
    p.wait(0.2);
    assert!(p.fe().results_box, "the save prompt: improved {}", p.fe().results_improved);
    p.key(K::Enter); // YES
    p.wait(0.2);
    assert!(p.fe().card.awaiting, "the overwrite question [M 0x10ee38]");
    p.key(K::Enter); // YES
    p.wait(0.5);
    assert!(saved(&dir) > before, "the results saved the codename");
    assert_eq!(p.page(), Some(Page::NfBonus), "the save routes on to the new reward");
    assert!(p.fe().bonus.is_some(), "the page built with no key pressed");
    p.press(K::Enter, "the mission list", |f| f.page == Page::MissionSelect);

    // The Edit Codename wheel's Save row asks the same question.
    p.press(K::Escape, "the difficulty page", |f| f.page != Page::MissionSelect);
    p.press(K::Escape, "the main menu", |f| f.page == Page::Main);
    p.wait(0.2);
    p.press(K::ArrowDown, "the codename row", |f| f.cursor == 2);
    p.press(K::Enter, "Edit Codename", |f| f.page == Page::CnSelect);
    p.wait(0.3);
    p.press(K::ArrowDown, "the QZ row", |f| f.cursor == 2);
    p.press(K::Enter, "the wheel", |f| f.page == Page::CnMenu);
    p.press(K::ArrowDown, "the Save row", |f| f.cursor == 6);
    let before = saved(&dir);
    std::thread::sleep(std::time::Duration::from_millis(50));
    p.key(K::Enter);
    p.wait(0.2);
    assert!(p.fe().card.awaiting, "the overwrite question");
    p.key(K::Enter);
    p.wait(0.5);
    assert!(saved(&dir) > before, "the wheel saved the codename");
    assert_eq!(p.page(), Some(Page::CnSelect));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Each on-foot row: the pick, the skip, the level, the quit back to the list.
#[test]
fn every_on_foot_mission_starts_from_the_list_and_quits_back() {
    let Some(iso) = iso() else { return };
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.mission_list();
    for row in (0..menu::CAMPAIGN.len()).filter(|&r| !menu::is_driving_level(menu::CAMPAIGN[r].3)) {
        let level = menu::CAMPAIGN[row].3;
        p.pick(row);
        p.briefing_up();
        assert!(p.movie().is_some_and(|m| !m.1), "row {row}: the briefing plays");
        p.key(K::ArrowDown);
        p.until(2.0, |a| a.async_load.is_some()).unwrap_or_else(|| panic!("row {row}: the skip"));
        p.finish_load();
        assert_eq!(p.app().level_code, level, "row {row}");
        assert!(p.app().fe.is_none());
        p.wait(1.0);
        assert!(p.sim().drones.frame > 0, "row {row}: the level runs");
        // A script camera refuses the pause: wait for the player.
        p.until(120.0, |a| a.scene.sim.as_ref().is_some_and(|s| s.script_camera().is_none())).expect("the player");
        p.quit_to_list();
        assert!(p.jingle_up(), "row {row}: the menu music on the way back: {:x?}", p.sound_ids());
        assert_eq!(p.movie(), Some((0x0735_0048, true)), "row {row}: the list's backdrop");
    }
    assert!(!p.exit_requested());
}

// ---------------------------------------------------------------------------
// movie-skip-keys: the skip action's PC keys (ArrowDown = the pad mask's D-pad-down half,
// R = R1 [M psiInput_MapInputs 0x10cd80:415-417]) must skip EVERY skippable movie kind
// through the real App key path, the way the game's ONE poll does. The game reads the skip
// action while a front-end movie plays — `Menu_InputAction(-1, 0x19, 4)` → `Menu_StopMovie(0)`
// [M MenuManager_Update 0x21c3d8:37-43] — and `Menu_StopMovie` stops the movie only when it
// accepts the skip (DAT_002e1c49) and the grace (DAT_002e1c4c, `Menu_UpdateMovieCounter`
// 0x215970:6-8) has run out [M 0x215a50:6]. The kinds differ only in their gates:
//   boot logos — c49 = 0, unskippable [M P_INTRO_Handler 0x209e88:41,:55];
//   the start movie (P_ESTHERO) — c49 = 1, grace 0xb4 = 180 ticks [M 0x20a060:17-18];
//   the attract trailer — c49 = 1, no grace [M P_ATTRACT_Handler 0x20a1f0:17-19,:27-29];
//   state 6 — the case-6 read `Input_Action(-1, 0x19, 1)`, no gate [M 0x1c98c8:148-150].
// The headless countdown carries the intro/state-6 movies on an image without PSS data, so
// those legs run on any NF_ISO at any display rate; the attract's key leg needs the real
// decoder (a declined attract closes at once [M BGFMVPlaying == 0 — the glue's Err arm]) and
// gates on `iso()`.
// ---------------------------------------------------------------------------

/// The image without the movie-data gate: the front-end state machine and the keys run on it.
fn nf_iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The boot logos refuse every skip — the action keys AND Confirm [M 0x209e88:41,:55].
#[test]
fn the_boot_logos_refuse_every_skip_at_240hz() {
    let Some(iso) = nf_iso() else { return };
    let mut p = Player::boot(&iso, 240);
    p.until(30.0, |a| a.fe.as_ref().is_some_and(|f| f.intro_playing && f.intro_movie)).expect("the first logo");
    p.key(K::ArrowDown); p.wait(0.3);
    p.key(K::KeyR); p.wait(0.3);
    p.key(K::Enter); p.wait(0.3);
    let fe = p.fe();
    assert!(fe.intro_playing && fe.intro_movie && fe.page == Page::Intro,
        "the boot logo is unskippable [M 0x209e88:41,:55]");
}

/// The start movie skips on the action keys — but only past its 180-tick grace, at 240 Hz.
#[test]
fn the_start_movie_skips_on_the_action_keys_past_the_grace_at_240hz() {
    let Some(iso) = nf_iso() else { return };
    // Inside the grace (180 ticks = 3 s) the keys refuse [M 0x215a50:6].
    let mut p = Player::boot(&iso, 240);
    p.title();
    p.key(K::Enter);
    p.until(8.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::EstHero && f.intro_movie)).expect("the start movie");
    p.wait(1.0);
    p.key(K::ArrowDown); p.wait(0.3);
    p.key(K::KeyR); p.wait(0.3);
    assert!(p.fe().intro_movie, "the grace still holds the movie (96 ticks < 180 [M 0x20a060:17])");
    // Past the grace (240 ticks in) D-pad-down's key skips, straight to the main menu
    // [M 0x21c3d8:40-43].
    p.wait(2.4);
    p.key(K::ArrowDown);
    p.until(3.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::Main && !f.intro_playing)).expect("the skip");
    // R (R1) skips too — a fresh boot.
    let mut p = Player::boot(&iso, 240);
    p.title();
    p.key(K::Enter);
    p.until(8.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::EstHero && f.intro_movie)).expect("the start movie");
    p.wait(3.4);
    p.key(K::KeyR);
    p.until(3.0, |a| a.fe.as_ref().is_some_and(|f| f.page == Page::Main && !f.intro_playing)).expect("the R skip");
}

/// State 6 skips on the action keys at once — the case-6 read has no gate [M 0x1c98c8:148-150].
#[test]
fn the_briefing_skips_on_the_action_keys_at_once_at_240hz() {
    let Some(iso) = nf_iso() else { return };
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.mission_list();
    p.pick(1);
    p.briefing_up();
    p.wait(0.5);
    p.key(K::ArrowDown);
    p.until(3.0, |a| a.async_load.is_some()).expect("the skip");
    p.finish_load();
    assert_eq!(p.app().level_code, 0x0700_0005, "the skip returns to the level load");
}

/// The attract trailer skips on the action keys at once — c49 = 1, c4c = 0 [M 0x20a1f0:17-19].
/// Needs the real decoder: a declined attract closes at once, so this runs only where the
/// image carries the PSS files (the sister legs above run everywhere).
#[test]
fn the_attract_trailer_skips_on_the_action_keys_at_240hz() {
    let Some(iso) = iso() else { return };
    let mut p = Player::boot(&iso, 240);
    p.main_menu();
    p.until(50.0, |a| a.fe.as_ref().is_some_and(|f| f.attract_playing)).expect("the attract");
    p.wait(0.5);
    p.key(K::ArrowDown);
    p.wait(0.3);
    assert!(!p.fe().attract_playing, "the trailer skipped on D-pad-down's key");
    assert_eq!(p.page(), Some(Page::Main));
    // The next cycle takes R (R1).
    p.until(60.0, |a| a.fe.as_ref().is_some_and(|f| f.attract_playing)).expect("the second attract");
    p.wait(0.5);
    p.key(K::KeyR);
    p.wait(0.3);
    assert!(!p.fe().attract_playing && p.page() == Some(Page::Main), "the R skip");
}

/// The poll's own rule, FrontEnd-level (the attract branch needs the real decoder at App level,
/// so the gate itself is pinned here): the attract skips with no grace, the START movie refuses
/// inside its 180-tick grace and passes after it, the boot logos are never armed [M
/// 0x20a1f0:17-19, 0x20a060:17-18, 0x215a50:6, 0x209e88:41,:55]. The fields are the state
/// `start_intro_movie` / `attract_open` build.
#[test]
fn the_skip_poll_honours_each_kind_gate() {
    let mut fe = menu::FrontEnd::new();
    // The attract trailer: c49 = 1, c4c = 0 — skips at once, back on the opener.
    fe.page = Page::Main;
    fe.attract_opener = Page::Main;
    fe.attract_movie = 0x073a_0048;
    fe.attract_playing = true;
    assert!(fe.movie_skip_armed());
    assert!(fe.movie_skip_poll(), "the attract skips at once [M 0x20a1f0:17-19]");
    assert!(!fe.attract_playing && fe.page == Page::Main, "back on the opener [M 0x20a1f0:64]");
    // The START movie: the grace refuses one tick short, passes on the tick [M 0x215a50:6].
    fe.page = Page::EstHero;
    fe.intro_playing = true;
    fe.intro_movie = true;
    fe.intro_skippable = true;
    fe.intro_ticks = menu::ESTHERO_GRACE - 1;
    assert!(!fe.movie_skip_poll(), "the grace holds at tick 179");
    assert!(fe.intro_movie, "the movie is still up");
    fe.intro_ticks = menu::ESTHERO_GRACE;
    assert!(fe.movie_skip_poll(), "past the grace it skips [M 0x20a060:17-18]");
    assert!(!fe.intro_movie);
    // The boot logos: c49 = 0 — never armed, however long they run [M 0x209e88:41,:55].
    fe.intro_movie = true;
    fe.intro_skippable = false;
    fe.intro_ticks = menu::ESTHERO_GRACE + 100;
    assert!(!fe.movie_skip_armed() && !fe.movie_skip_poll(), "the logos refuse");
    assert!(fe.intro_movie);
}
