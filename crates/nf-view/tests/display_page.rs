//! window-size-fullscreen: the port-only Display page (PAGE_DISPLAY 0x5000_0006). The whole
//! feature is the port's own surface — the PS2 had no PC window, and the main agent's note pins
//! the route: NO row is added to a game page (the reachable options page is the game's own
//! P_CNAVOPTIONS 0x214818), so the page is reached only through the port's `open_display` (the
//! host's Alt+D, or the headless `--page display` render aid). The picks ride
//! `FrontEnd::display_request` for the host, which applies them to the winit window and writes
//! the SAME display.toml the Alt+Enter / Alt+PageUp keys write (settings::display_path — no
//! second file; the file maths' round trip is settings.rs's own tests). The aspect-fit maths is
//! `cutscene_frame`'s (fix_cutscene_letterbox_live.rs pins the 4:3 band) and
//! `settings::next_window_preset`'s (the fit rule). Everything here pins the port design [S] —
//! kept out of game-behaviour pass/fail.

use nf_view::menu::{Action, FrontEnd, Page};
use nf_view::settings::Display;

/// A FrontEnd with the Display page open, as the host's Alt+D does it.
fn fe_open(monitor: (u32, u32), now: Display) -> FrontEnd {
    let mut fe = FrontEnd::new();
    assert!(fe.open_display(now, monitor, Page::Main), "the page opens from Main");
    assert_eq!(fe.page, Page::Display);
    fe
}

#[test]
fn the_page_offers_the_presets_that_fit_the_monitor() {
    let fe = fe_open((1920, 1080), Display { fullscreen: false, size: Some((800, 600)) });
    // 1600x1200 and 1920x1440 do not fit 1080 lines; 640x480 / 800x600 / 1280x960 do.
    assert_eq!(fe.display_presets(), vec![(640, 480), (800, 600), (1280, 960)]);
    assert_eq!(fe.rows(), 5, "the toggle + the 3 fitting presets + Back");
    assert_eq!(fe.row_label(None, 0), "Fullscreen  Off", "the row shows the live state");
    assert_eq!(fe.row_label(None, 1), "640 x 480");
    assert_eq!(fe.row_label(None, 3), "1280 x 960");
    assert_eq!(fe.row_label(None, 4), "Back");
    assert_eq!(fe.row_label(None, 9), "Back", "an off-page row clamps onto the last one");
}

#[test]
fn every_preset_is_offered_without_a_monitor_and_1920x1440_fits_1440_lines() {
    let mut fe = FrontEnd::new();
    assert!(fe.open_display(Display::default(), (0, 0), Page::Main));
    assert_eq!(fe.display_presets().len(), 5, "(0, 0) = the host could not tell: everything fits");
    assert_eq!(fe.rows(), 7);
    let fe = fe_open((2560, 1440), Display::default());
    assert!(fe.display_presets().contains(&(1920, 1440)), "1920x1440 fits 1440 lines exactly");
    assert_eq!(fe.display_presets().len(), 5);
}

#[test]
fn confirm_on_the_fullscreen_row_requests_the_toggle_and_keeps_the_size() {
    let mut fe = fe_open((1920, 1080), Display { fullscreen: false, size: Some((800, 600)) });
    assert!(fe.action(Action::Confirm), "the page opens on the toggle row");
    assert_eq!(fe.display_request, Some(Display { fullscreen: true, size: Some((800, 600)) }),
        "the request carries the kept size — the host applies + writes display.toml");
    assert_eq!(fe.row_label(None, 0), "Fullscreen  On", "the row shows the picked state");
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.display_request, Some(Display { fullscreen: false, size: Some((800, 600)) }));
}

#[test]
fn confirm_on_a_preset_requests_windowed_at_that_size() {
    let mut fe = fe_open((2560, 1440), Display { fullscreen: true, size: Some((800, 600)) });
    assert!(fe.action(Action::Down), "row 1");
    assert!(fe.action(Action::Confirm), "the first preset row");
    assert_eq!(fe.display_request, Some(Display { fullscreen: false, size: Some((640, 480)) }),
        "a size row means WINDOWED at that size — fullscreen drops");
}

#[test]
fn back_leaves_to_the_page_the_host_opened_from() {
    let mut fe = fe_open((1920, 1080), Display::default());
    for _ in 0..4 { fe.action(Action::Down); }   // the last row (Back), 5 rows
    assert_eq!(fe.cursor, 4);
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::Main, "the confirm on the Back row leaves");
    let mut fe = fe_open((1920, 1080), Display::default());
    assert!(fe.action(Action::Back), "the keyboard's ○ leaves from anywhere on the page");
    assert_eq!(fe.page, Page::Main);
    // And a pause-page opener returns to the pause, not Main.
    let mut fe = FrontEnd::new();
    assert!(fe.open_display(Display::default(), (1280, 960), Page::Pause));
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::Pause);
}

#[test]
fn the_mouse_walks_and_clicks_the_rows_like_the_keyboard() {
    let mut fe = fe_open((1920, 1080), Display::default());
    // The drawn rows' bands (list_rows_y: 138 + 26k, ±(−6, +22)) — row 1 is the 640 x 480 preset.
    assert!(fe.mouse_move(320.0, 164.0), "hovering the second row selects it");
    assert_eq!(fe.cursor, 1);
    assert!(fe.mouse_click(320.0, 164.0), "a click on an action row confirms it");
    assert_eq!(fe.display_request, Some(Display { fullscreen: false, size: Some((640, 480)) }));
}

#[test]
fn a_blocked_page_cannot_open_twice_or_over_a_notice() {
    let mut fe = FrontEnd::new();
    assert!(fe.open_display(Display::default(), (1920, 1080), Page::Main));
    assert!(!fe.open_display(Display::default(), (1920, 1080), Page::Main),
        "the page is already up — the second Alt+D is a no-op");
    assert_eq!(fe.page, Page::Display);
    // A notice box blocks the game's page input the way the keyboard is blocked.
    let mut fe = FrontEnd::new();
    fe.notice = Some("Checking the memory card...".into());
    assert!(!fe.open_display(Display::default(), (1920, 1080), Page::Main),
        "a notice up blocks the open");
    assert_ne!(fe.page, Page::Display);
}

#[test]
fn the_page_id_lands_in_the_port_only_block() {
    assert_eq!(Page::Display.id(), nf_view::menu::PAGE_DISPLAY);
    assert_eq!(Page::from_id(nf_view::menu::PAGE_DISPLAY), Some(Page::Display));
    assert_eq!(nf_view::menu::PAGE_DISPLAY, 0x5000_0006, "next to the online pages' spare block");
}
