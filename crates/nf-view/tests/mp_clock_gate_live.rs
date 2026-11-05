//! audit-sprite-set-text: the MP clock's write gate, drawn through the real overlay path
//! (`nf_view::load_mp_carry` + `nf_view::hud::overlay`). Opt-in via `NF_ISO`.
//!
//! Game evidence (ACTION.ELF, `MP_CheckForEndCondition` 0x184268): the demolition/protection
//! rounds run on the ROUND clock `MPGame+0x194` and `TimeSpr` is written every playing tick
//! whenever it is > 0 — NO −1 check [M :57, reached from the 0x20000040/0x20000080 arm at :50/:83]
//! — while every other scenario writes only when the match limit is set, `MPSettings+0x1a0 != -1`
//! [M :142] (and never after the 0xfe time-up latch). `MP_Start` 0x183a50:202-203 clears TimeSpr
//! to "", so an ungated scenario never shows a clock. The port's round default (60.0 when the
//! menu limit is negative [M `MP_Start` 0x183438:53 → the port's mod.rs:610-612]) lives in
//! `game.limit`; the old port keyed the whole gate on `settings.time_limit != -1`, which hid the
//! round clock on unlimited demolition/protection matches.

use nf_game::mp::MpSettings;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn settings(scenario: u32, time_limit: i32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = 3;
    s.time_limit = time_limit;
    s.scenario = scenario;
    s
}

/// The header-font slot's quad count — the clock is the only font-1 draw in a match
/// (`mp_panes`: the score pane and the status line use font 2, the radar no text).
fn clock_quads(scene: &nf_view::scene::Scene) -> usize {
    let frame = nf_view::hud::overlay(scene, 4.0 / 3.0);
    let slot = scene.hud_textures.iter().position(|(k, _)| *k == nf_view::hud::HEADER_FONT_TEX)
        .map(|i| i + 1).expect("the header font loads");
    frame.runs.iter().filter(|r| r.0 as usize == slot).map(|r| r.3 as usize / 6).sum()
}

/// Unlimited match limit + a demolition scenario: the round clock still draws (the game's round
/// arm gates on `MPGame+0x194 > 0` [M 0x184268:57], the port's `game.limit` 60.0) where the old
/// settings gate hid it. Sister: an unlimited deathmatch shows NO clock [M :142's -1 gate —
/// TimeSpr keeps MP_Start's "" clear, 0x183a50:202-203], and a limited deathmatch does.
#[test]
fn the_round_clock_shows_on_unlimited_demolition_and_not_on_unlimited_deathmatch() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    // Demolition, no match limit: the 60-second round clock [M 0x183438:53].
    let scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None,
        settings(nf_game::mp::scenario::DEMOLITION, -1)).expect("load demolition");
    assert!(clock_quads(&scene) > 0, "the round clock must draw on an unlimited demolition match");
    // Deathmatch, no match limit: TimeSpr stays "" [M 0x184268:142].
    let scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None,
        settings(nf_game::mp::scenario::DEATHMATCH, -1)).expect("load deathmatch");
    assert_eq!(clock_quads(&scene), 0, "an unlimited deathmatch shows no clock");
    // Deathmatch with a limit: the match clock draws [M 0x184268:142-147].
    let scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None,
        settings(nf_game::mp::scenario::DEATHMATCH, 600)).expect("load limited");
    assert!(clock_quads(&scene) > 0, "the match clock draws with a limit set");
}
