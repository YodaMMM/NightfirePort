//! mp-settings-2win: the multiplayer settings that the two-window lobby check found not reaching
//! a match, headless (no ISO needed).
//!
//! Health Handicap: the Setup page's state-4 pick lands in `MPSettings + slot·0x30 + 0x2c` [M
//! C_RBMPSETUP 0x20de58:112] (the codename's MP options row writes slot 0's word too [M
//! P_CNMPOPTIONS 0x2144d8:14]); the confirm's compaction copies the 0x30 record whole [M
//! P_MPCONFIRM 0x20ec00:113]; `Player_Init` and `MP_ReSpawn` spawn at `(float)(int)+0x2c + 100`
//! health [M 0x1910f8:128-131, 0x185368:71-72]. The port's confirm wrote a hard 0.

use nf_view::menu::{self, Action, FrontEnd, Page, MP_HANDICAPS};

fn iris_wait(fe: &mut FrontEnd) {
    for _ in 0..menu::CLOSE_IRIS_PAGE_TICKS + 2 { fe.tick(); }
}

/// Main → Multiplayer → Local → join agent 1 → Arena → Skyrail → Bond → the handicap list at
/// `pick` → Options hub → one bot (the FFA start check wants two players) → confirm → start.
fn couch_match_with_handicap(pick: usize) -> nf_game::mp::MpSettings {
    let mut fe = FrontEnd::new();
    fe.tick();
    fe.page = Page::Main;
    fe.cursor = 1;
    assert!(fe.action(Action::Confirm));
    fe.cursor = 0;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpJoin);
    fe.cursor = 0;
    for _ in 0..3 { assert!(fe.action(Action::Confirm)); } // join, codename, ready
    fe.mp_scenario_row = 1; // Arena
    assert!(fe.action(Action::Confirm));
    iris_wait(&mut fe);
    fe.mp_map_row = 0; // Skyrail
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_setup_state, 3, "FFA asks the character");
    fe.mp_setup_sel = 0;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_setup_state, 4, "then the Health Handicap list [M C_RBMPSETUP 3 → 4]");
    fe.mp_setup_sel = pick;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpOptions);
    assert_eq!(fe.mp_handicap, MP_HANDICAPS[pick].0, "the pick is stored [M 0x20de58:112]");
    fe.mp_bots[0].enabled = true;
    fe.mp_options_row = 0;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpConfirm);
    assert!(fe.action(Action::Confirm), "the summary's confirm starts the match");
    fe.mp_match_settings.clone().expect("the match settings land")
}

#[test]
fn the_health_handicap_reaches_the_match() {
    // -50 (list row 1): 50 health at the start and at every respawn.
    let s = couch_match_with_handicap(1);
    assert_eq!(s.players[0].armor_bonus as i32, -50, "agent 1's +0x2c carries the pick [M 0x20ec00:113]");
    assert_eq!(s.players[1].armor_bonus as i32, 0, "the bot slot keeps the boot 0 [M 0x1c7e78:52]");
    let m = nf_game::mp::MpMatch::init(s);
    assert_eq!(m.human_health(0), 50.0, "Player_Init / MP_ReSpawn: +0x2c + 100 [M 0x1910f8:129, 0x185368:71]");
    // +100 (row 7): 200.
    let s = couch_match_with_handicap(7);
    assert_eq!(s.players[0].armor_bonus as i32, 100);
    assert_eq!(nf_game::mp::MpMatch::init(s).human_health(0), 200.0);
    // The default "0" row keeps 100 (the sister: nothing changes without a pick).
    let s = couch_match_with_handicap(3);
    assert_eq!(nf_game::mp::MpMatch::init(s).human_health(0), 100.0);
}

#[test]
fn the_handicap_survives_the_wire() {
    // The online handover carries the word on every machine (mpnet's MatchSettings round trip).
    let s = couch_match_with_handicap(0); // -75
    let back = nf_view::mpnet::match_to_mp(&nf_view::mpnet::mp_to_match(&s), s.seed);
    assert_eq!(back.players[0].armor_bonus as i32, -75);
    assert_eq!(nf_game::mp::MpMatch::init(back).human_health(0), 25.0);
}

/// The one screen's lobby with the boot words and the four default bot records.
fn lobby() -> nf_view::mp_lobby::MpLobby {
    nf_view::mp_lobby::MpLobby::new_host(
        0x2000_0002, 0x0700_0024, 0, 1, 0, 10, 10, [0, 1], [2, 0, 0, 0, 0],
        std::array::from_fn(nf_view::menu::MpBot::fresh), "Player 1", true, 0)
}

fn focus(l: &mut nf_view::mp_lobby::MpLobby, id: nf_view::mp_lobby::MpLobbyRowId) {
    let at = l.rows().iter().position(|r| r.id == id).expect("the row exists");
    l.focus = at;
}

#[test]
fn a_bot_switched_on_plays_for_its_characters_side() {
    use nf_view::mp_lobby::MpLobbyRowId as R;
    // The fresh records: characters 6/7/8/9 (Snow Guard, Black Ops, Yakuza, Phoenix Commando),
    // team byte 0 = Phoenix [D the zeroed records; M C_SBMPOPTIONS 0x20e818:26-31 writes only +0x10].
    for slot in 0..4 {
        let b = nf_view::menu::MpBot::fresh(slot);
        assert_eq!(b.character, 6 + slot as u32);
        assert!(!b.team_mi6, "fresh record {slot} is on Phoenix (+0x0F = 0)");
    }
    let mut l = lobby();
    // ▶ on an Off bot switches it on: Snow Guard is a Phoenix character [M Menu_IsBotGood].
    focus(&mut l, R::Bot(0));
    assert!(l.apply(Action::Right));
    assert!(l.bots[0].enabled);
    assert!(!l.bots[0].team_mi6, "Snow Guard plays for Phoenix");
    // ✕ on an Off bot does the same (Black Ops, Phoenix).
    focus(&mut l, R::Bot(1));
    assert!(l.apply(Action::Confirm));
    assert!(l.bots[1].enabled && !l.bots[1].team_mi6, "Black Ops plays for Phoenix");
    // Sister: an MI6 character switched on lands on MI6 (Bond, row 0).
    l.bots[2].character = 0;
    focus(&mut l, R::Bot(2));
    assert!(l.apply(Action::Confirm));
    assert!(l.bots[2].team_mi6, "Bond plays for MI6 [M the preset's side byte 0]");
}

#[test]
fn the_lobby_bots_reach_the_match_on_their_sides() {
    // The front end's own records (fresh) through the host's one screen into the match's team words.
    let mut fe = FrontEnd::new();
    fe.mp_hosting = true;
    fe.mp_one = lobby();
    fe.mp_one.players[0].team = 1; // the host on MI6
    focus(&mut fe.mp_one, nf_view::mp_lobby::MpLobbyRowId::Bot(0));
    assert!(fe.mp_one.apply(Action::Right));
    focus(&mut fe.mp_one, nf_view::mp_lobby::MpLobbyRowId::Bot(1));
    assert!(fe.mp_one.apply(Action::Right));
    assert!(fe.mp_lobby_start(), "MI6 human + two Phoenix bots pass the team check [M 0x20e818:72-90]");
    let s = fe.mp_match_settings.clone().unwrap();
    assert_eq!(s.players[0].team, 1, "the host on MI6");
    assert_eq!(s.players[1].team, 0, "Snow Guard's slot on Phoenix");
    assert_eq!(s.players[2].team, 0, "Black Ops' slot on Phoenix");
}
