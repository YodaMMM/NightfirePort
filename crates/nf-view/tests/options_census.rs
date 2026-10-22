//! options-census: every value row of the codename options pages is classified in
//! `menu::OPT_CENSUS` — Wired (the port drains it to a consumer) or NoPortReader (the game's
//! reader has no PC counterpart, with the reason). The census test below fails when a row of
//! ADV_ROWS / MP_ROWS / AV_ROWS / CN_CONTROLS_ROWS is missing from the list, so a future
//! options row cannot land without a reader (the Auto Aim regression this task fixes).
//! Evidence: docs/systems/options-census.md; the game's writers are P_CNOPTIONS_Handler
//! 0x213f08:13-21, P_CNMPOPTIONS_Handler 0x2144d8:13-15, P_CNAVOPTIONS_Handler 0x214818:128-134
//! and P_CNCONTROLS_Handler 0x213cd0 / C_RBCONTROL_Handler 0x217908.
use nf_view::menu::{self, OptCensus};

/// Every non-action row of the four pages has a census entry.
#[test]
fn every_option_row_has_a_census_entry() {
    let pages = [
        ("Advanced 0x4000002d", menu::ADV_ROWS),
        ("MP 0x4000002e", menu::MP_ROWS),
        ("AV 0x40000031", menu::AV_ROWS),
        ("Controls 0x40000022", menu::CN_CONTROLS_ROWS),
    ];
    for (name, rows) in pages {
        assert!(!rows.is_empty(), "{name}: the page has no rows at all");
        for row in rows.iter() {
            if row.kind == menu::OptKind::Action {
                continue; // navigation rows carry no profile byte (0x214818:91-125)
            }
            let kind = menu::opt_census(row.ctrl);
            assert!(
                kind.is_some(),
                "{name}: row {:#x} has no reader classification — add it to OPT_CENSUS with its game reader",
                row.ctrl
            );
        }
    }
}

/// The two census kinds both occur, and each NoPortReader cites its evidence — the test fails
/// when a future change wires one of these (then the entry must move to Wired) or strips a
/// reason away.
#[test]
fn the_no_port_reader_entries_cite_their_reason() {
    assert!(
        menu::OPT_CENSUS.iter().any(|(_, k)| matches!(k, OptCensus::Wired)),
        "at least one row must be wired"
    );
    for (ctrl, k) in menu::OPT_CENSUS {
        if let OptCensus::NoPortReader(reason) = k {
            assert!(!reason.is_empty(), "{ctrl:#x}: a NoPortReader entry must cite why");
            // The reason carries the game address of the reader it stands in for.
            assert!(reason.contains("0x1"), "{ctrl:#x}: the reason names the game reader's address");
        }
    }
}

/// The Auto Aim pair is classified Wired — the user report 2026-10-21 that started the census
/// ("Auto Aim did nothing") is the regression this list guards.
#[test]
fn the_auto_aim_rows_are_wired() {
    assert_eq!(menu::opt_census(0x1000_011b), Some(OptCensus::Wired), "SP auto aim [M 0x1a7c98:41]");
    assert_eq!(menu::opt_census(0x1000_0129), Some(OptCensus::Wired), "MP auto aim [M 0x1a7c98:44]");
}

/// The MP options rows move the front end's MP fields — the game's page and lobby write the
/// same MPSettings record [M P_CNMPOPTIONS_Handler 0x2144d8:13-14 / C_RBMPSETUP 0x20de58:112].
#[test]
fn the_mp_rows_move_the_mp_fields() {
    let mut fe = menu::FrontEnd::new();
    fe.opts = menu::default_opts();
    assert_eq!(fe.opt_value(0x1000_0128), 1, "radar defaults On [M bootup 0x1c7e78:50]");
    // One ◀/▶ step wraps the On/Off row to 0 and the FrontEnd field follows.
    fe.opt_step(&menu::MP_ROWS[0], 1);
    assert_eq!(fe.opt_value(0x1000_0128), 0, "the step moved the row to Off");
    assert_eq!(fe.mp_player_radar, 0, "MPSettings+slot·0x30+0x28 (fe.mp_player_radar) moved with it [M 0x2144d8:13]");
    assert_eq!(fe.mp_radar, 1, "sister: the match radar +0x1C4 (Player Mods) is a different setting [M 0x2104d0:16]");
    // And back.
    fe.opt_step(&menu::MP_ROWS[0], 1);
    assert_eq!((fe.opt_value(0x1000_0128), fe.mp_player_radar), (1, 1));
}
