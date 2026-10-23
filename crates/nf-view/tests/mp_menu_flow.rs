//! mp-menu-flow: the multiplayer front-end flow, headless (no ISO needed).
//!
//! The page graph is the decompile's [M]: Main menu row 1 → C_GOMULTIPLAYER 0x216a78's delayed
//! page change → P_MPJOIN 0x40000019; the join page's ready arm (C_RBMPSTART 0x20d048's state-5
//! ✕ + P_MPJOIN 0x20ca00's Menu_MPAreWeReady) → P_MPSCENARIO 0x4000001a; C_SBMPSCEN 0x20c600's
//! confirm → P_MPMAP 0x40000013; C_SBMPMAP 0x217538's confirm → P_MPSETUP 0x40000051; the
//! per-agent picks → P_MPOPTIONS 0x40000012; C_SBMPOPTIONS row 0 → P_MPCONFIRM 0x40000049, whose
//! confirm (P_MPCONFIRM 0x20ec00:53-94) hands the match over. Each step keeps a sister assertion
//! that can fail.

use nf_view::menu::{self, Action, FrontEnd, Page, MP_MAP_ROWS, MP_OPTIONS_ROWS, MP_SCENARIO_ROWS};

/// Main menu → Multiplayer → Local → Join (the main menu's row 1 confirm; net-online-lobby put
/// the port's Local / Host Online / Join Online choice between them, and row 0 Local keeps the
/// game's page [S docs/systems/net-online-lobby.md §2]).
fn enter_mp(fe: &mut FrontEnd) {
    fe.tick(); // boot → the runner state [M 0x1c98c8 case 1] — the flow machine must be live for
               // the confirm's ResetMap_LevelToLoad push to be accepted
    fe.page = Page::Main;
    fe.cursor = 1; // Multiplayer [M MAIN_ITEMS 0x149; C_GOMULTIPLAYER fires on row 1]
    assert!(fe.action(Action::Confirm), "Multiplayer confirm must navigate");
    assert_eq!(fe.page, Page::MpMode, "the port's mode choice is up [S net-online-lobby]");
    assert_eq!(fe.page.id(), 0x5000_0001);
    fe.cursor = 0; // Local [S — the game's own flow]
    assert!(fe.action(Action::Confirm), "Local confirm must navigate");
    assert_eq!(fe.page, Page::MpJoin, "Local lands on C_GOMULTIPLAYER's 0x40000019");
    assert_eq!(fe.page.id(), 0x4000_0019);
}

/// The keyboard join: agent 1 joins, picks the default codename, readies.

/// G-048(1): the CloseIris walks land 10 ticks after the action [M 0x202190:8-9].
fn iris_wait(fe: &mut FrontEnd) {
    for _ in 0..menu::CLOSE_IRIS_PAGE_TICKS + 2 { fe.tick(); }
}

fn join_agent1(fe: &mut FrontEnd) {
    fe.cursor = 0;
    assert!(fe.action(Action::Confirm), "✕ joins agent 1");
    assert_eq!(fe.mp_join[0].state, 1, "the panel's state 1 = Choose Codename [M mp_stuff+0x360]");
    assert_eq!(fe.mp_join[1].state, 0, "agents 2-4 stay unjoined (no pads — keyboard only)");
    assert!(fe.action(Action::Confirm), "✕ picks the codename");
    assert_eq!(fe.mp_join[0].state, 5, "state 5 = joined [M C_RBMPCNAME's write]");
    assert_eq!(fe.mp_join[0].codename, "Player 1", "the template codename [M Txt 0x1c3 + slot]");
    assert!(fe.action(Action::Confirm), "✕ readies");
    assert!(fe.mp_join[0].ready, "mpjoin+0x01 set [M C_RBMPSTART's state-5 arm]");
}

#[test]
fn main_to_join_to_scenario_flows() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    assert_eq!(fe.page, Page::MpScenario, "the ready arm reaches the scenario page [M → 0x4000001a]");
    assert_eq!(fe.page.id(), 0x4000_001a);
    // Sister: the scenario wheel opens on the row whose +0xc word is MPSettings+0x1A4 [M
    // P_MPSCENARIO 0x217348:24-25]; boot sets +0x1A4 = 1 [M bootup 0x1c7e78:30] — row 1, Arena.
    assert_eq!(fe.mp_scenario, 1, "the boot scenario word [M 0x1c7e78:30]");
    assert_eq!(fe.mp_scenario_row, 1, "a fresh boot's scenario cursor is Arena [M 0x217348:25]");
}

#[test]
fn scenario_confirm_moves_to_map_and_back_out() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    // Walk to a real scenario (row 1 = Arena / Deathmatch) and confirm.
    fe.mp_scenario_row = 1;
    assert!(fe.action(Action::Confirm), "the scenario confirm navigates");
    iris_wait(&mut fe); // [M C_SBMPSCEN 0x20c600:45 Menu_ChangePageCloseIris — G-048(1)]
    assert_eq!(fe.page, Page::MpMap, "C_SBMPSCEN's confirm opens the map page [M → 0x40000013]");
    assert_eq!(fe.mp_scenario, 1, "the confirm stores the scenario word [M MPSettings+0x1A4]");
    // Sister: the map confirm stores the level code and lands on the setup page.
    fe.mp_map_row = 0; // Skyrail 0x07000024
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpSetup);
    assert_eq!(fe.mp_level, 0x0700_0024, "C_SBMPMAP's confirm stores the level code [M +0x1A8]");
    // Back out the whole chain [M the pages' own backs].
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::MpMap, "setup's Back returns to the map page");
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::MpScenario, "map's Back returns to the scenario page");
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::Main, "scenario's Back returns to the main menu [M 0x217348:28-30 — the 99 arm opens 0x40000002]");
    assert_eq!(fe.cursor, 1, "back on the Multiplayer row [S — P_MAIN's 0x4c writes only the write-only DAT_002a3cce (0x20a7b0:39); the landing row is the port's choice]");
}

#[test]
fn setup_ffa_then_options_then_rules_back_out() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    fe.mp_scenario_row = 1; // Arena (FFA)
    fe.action(Action::Confirm);
    iris_wait(&mut fe); // [M C_SBMPSCEN 0x20c600:45 Menu_ChangePageCloseIris — G-048(1); the walk must land before the map row's confirm]
    fe.mp_map_row = 0;
    fe.action(Action::Confirm);
    assert_eq!(fe.mp_setup_state, 3, "FFA asks Choose Character [M P_MPSETUP's mp_stuff state 3]");
    // FFA: the character confirm asks the Health Handicap, whose confirm lands on the hub [M
    // C_RBMPSETUP 0x20de58's 0x4b: 3 → 4 → 5 + Menu_MPAreWeReady → 0x40000012].
    fe.mp_setup_sel = 0; // Bond
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_setup_state, 4, "then the Health Handicap [M the state-4 list −75..+100]");
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpOptions, "the picks flow to P_MPOPTIONS [M → 0x40000012]");
    assert_eq!(fe.mp_join[0].skin, 0);
    // Rules page: change the limits, close, and see the values kept.
    assert!(fe.action(Action::Down)); // AI Bots row
    assert!(fe.action(Action::Down)); // Game Rules
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpRules);
    let (t0, f0) = (fe.mp_time_limit, fe.mp_frag_limit);
    assert!(fe.action(Action::Right), "▶ steps Duration");
    assert_ne!(fe.mp_time_limit, t0, "the duration list moved");
    assert!(fe.action(Action::Down));
    assert!(fe.action(Action::Right));
    assert_eq!(f0, 10, "the boot Points value [M bootup 0x1c7e78:29 MPSettings+0x19c = 10]");
    assert_eq!(fe.mp_frag_limit, 15, "the points list steps 10 → 15 [M P_MPRULES 0x20f790:54-55]");
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpOptions, "the rules page closes to the hub [M Manager 0x5f]");
    // Sister: Player Mods and Enviro-Mods close the same way.
    fe.mp_options_row = 3;
    fe.action(Action::Confirm);
    assert_eq!(fe.page, Page::MpPlayerMods);
    fe.action(Action::Confirm);
    assert_eq!(fe.page, Page::MpOptions);
    fe.mp_options_row = 4;
    fe.action(Action::Confirm);
    assert_eq!(fe.page, Page::MpEnviroMods);
    fe.action(Action::Back);
    assert_eq!(fe.page, Page::MpOptions);
}

#[test]
fn locked_scenario_and_ravine_bots_gate() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    // Uplink is row 4 — reward-locked on a fresh profile.
    fe.mp_scenario_row = 4;
    assert!(!fe.action(Action::Confirm), "a locked scenario refuses [M C_SBMPSCEN:40-42]");
    assert_eq!(fe.page, Page::MpScenario, "the page did not change");
    // Sister: the unlock_all shortcut opens Uplink [M the gate's other arm].
    fe.unlock_all = true;
    assert!(fe.action(Action::Confirm));
    iris_wait(&mut fe); // [M C_SBMPSCEN 0x20c600:45]
    assert_eq!(fe.page, Page::MpMap);
    fe.unlock_all = false;   // back to the fresh profile for the Ravine checks
    // Ravine (row 7) hides the AI Bots row.
    fe.mp_map_row = 7;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_level, 0x0700_004b);
    fe.mp_setup_state = 4;
    fe.action(Action::Confirm); // the handicap pick → options
    assert_eq!(fe.page, Page::MpOptions);
    assert!(!fe.mp_options_ai_bots_open(), "Ravine clears the AI Bots row [M 0x217678:20-25]");
    fe.mp_options_row = 1;
    assert!(!fe.action(Action::Confirm), "the locked AI Bots row refuses");
    assert_eq!(fe.page, Page::MpOptions);
}

#[test]
fn bots_flow_picks_and_counts() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    fe.mp_scenario_row = 1;
    fe.action(Action::Confirm);
    iris_wait(&mut fe); // [M C_SBMPSCEN 0x20c600:45 — G-048(1)]
    fe.mp_map_row = 1; // Fort Knox — bots allowed
    fe.action(Action::Confirm);
    fe.action(Action::Confirm); // FFA character pick → the handicap
    fe.action(Action::Confirm); // the handicap pick → options [M 0x4b's 3 → 4 → 5]
    assert_eq!(fe.page, Page::MpOptions);
    // → AI Bots → Setup Bot 1 → the wheel opens on the record's character [M 6 = Snow Guard].
    fe.mp_options_row = 1;
    fe.action(Action::Confirm);
    assert_eq!(fe.page, Page::MpBots);
    assert_eq!(fe.mp_bots_row, 1);
    fe.action(Action::Confirm);
    assert_eq!(fe.page, Page::MpBotChoose);
    assert_eq!(fe.mp_bot_char, 6, "the wheel opens on the slot's character [M P_MPBOTCHOOSE]");
    // A locked character (Jaws, row 24) refuses; a free one (Ninja, row 11) lands on bot setup.
    fe.mp_bot_char = 24;
    assert!(!fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpBotChoose);
    fe.mp_bot_char = 11;
    assert!(fe.action(Action::Confirm), "Ninja (preset 11) is unlocked");
    assert_eq!(fe.page, Page::MpBotSetup);
    assert!(fe.mp_bots[0].enabled, "the choose sets record+0x0E [M C_SBMPBTCHOOSE]");
    assert_eq!(fe.mp_bots[0].character, 11);
    assert!(!fe.mp_bots[0].team_mi6, "Ninja's preset side is Phoenix [M Menu_IsBotGood]");
    assert_eq!(fe.mp_bot_count(), 1, "the enabled bot counts [M Menu_PrepareBots]");
    // Confirm closes back to the bots list [M Manager 0x5f twice].
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpBots);
    // Continue → the hub → the start checks: 1 human + 1 bot = 2 players, FFA ok.
    fe.mp_bots_row = 0;          // the Continue row (the confirm left the cursor on the slot row)
    fe.action(Action::Confirm); // row 0 = Continue
    assert_eq!(fe.page, Page::MpOptions);
    fe.mp_options_row = 0;
    assert!(fe.action(Action::Confirm), "the start checks pass with 2 players");
    assert_eq!(fe.page, Page::MpConfirm);
}

#[test]
fn start_checks_refuse_single_player() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    fe.mp_scenario_row = 1;
    fe.action(Action::Confirm);
    iris_wait(&mut fe); // [M C_SBMPSCEN 0x20c600:45 — G-048(1)]
    fe.mp_map_row = 1;
    fe.action(Action::Confirm);
    fe.action(Action::Confirm); // character → handicap
    fe.action(Action::Confirm); // handicap → options
    fe.mp_options_row = 0;
    assert!(!fe.action(Action::Confirm), "one player alone cannot start [M the 0x39e box]");
    assert_eq!(fe.mp_error, 0x39e, "the error label id lands in mp_error");
    assert_eq!(fe.page, Page::MpOptions, "the page stays");
    // Sister: the same single human WITH a bot passes (the bots flow test covers that path
    // end-to-end; here enable a bot directly and retry).
    fe.mp_bots[0].enabled = true;
    // The refusal's message box stands (it was never drawn before — playtest 1015-2231): Enter
    // dismisses it first, then Continue passes.
    assert!(fe.action(Action::Confirm), "Enter dismisses the message box");
    assert_eq!(fe.mp_error, 0);
    assert_eq!(fe.page, Page::MpOptions, "dismissing doesn't also continue");
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpConfirm);
}

#[test]
fn quick_game_starts_the_games_fixed_match() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    // Quick Game is row 0 — the fixed quick-start config, straight to the confirm page [M
    // C_SBMPSCEN 0x20c600:48-145 + disasm 0x20c72c-0x20c868, docs/systems/quick-game.md §2].
    // The arena is the game's Rand_Random() % 7 sample over mp_level rows 0..6 [M 0x20c75c-0x20c7b4];
    // the port takes row 0 (Skyrail) for headless determinism [S — quick-game.md §3].
    fe.mp_scenario_row = 0;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpConfirm, "Quick Game skips the setup pages [M → 0x40000049]");
    // Every store of the quick-start block, on the menu fields [M quick-game.md §2's table].
    assert_eq!(fe.mp_scenario, 1, "+0x1A4 = 1 Deathmatch [M 0x20c740]");
    assert_eq!(fe.mp_time_limit, 10, "+0x1A0 = 10 [M 0x20c72c]");
    assert_eq!(fe.mp_frag_limit, 10, "+0x19C = 10 [M 0x20c738]");
    assert_eq!(fe.mp_friendly_fire, 0, "+0x198 = 0 [M 0x20c754]");
    assert_eq!(fe.mp_weapon_set, 0, "+0x1B4 = 0 Normal [M 0x20c760]");
    assert_eq!(fe.mp_radar, 1, "+0x1C4 = 1 [M 0x20c750]");
    assert_eq!(fe.mp_envmods[4], 2, "+0x1CC = 2 Helicopters [M 0x20c734]");
    assert_eq!(fe.mp_envmods[2], 0, "+0x1D4 = 0 [M 0x20c748]");
    assert_eq!(fe.mp_envmods[3], 0, "+0x1D0 = 0 [M 0x20c758]");
    // The fields the quick start does NOT write keep their values [M the store list has no
    // 0x1B8/0x1BC/0x1C0/0x1C8; boot 0x1c7e78:28,38,45,46] — location damage stays ON.
    assert_eq!(fe.mp_player_mods[1], 1, "+0x1C8 keeps the boot 1 [M 0x1c7e78:45]");
    assert_eq!(fe.mp_player_mods[0], 0, "+0x1BC keeps the boot 0 [M 0x1c7e78:46]");
    assert_eq!(fe.mp_envmods[0], 2, "+0x1C0 keeps the boot 2 [M 0x1c7e78:28]");
    assert_eq!(fe.mp_envmods[1], 0, "+0x1B8 keeps the boot 0 [M 0x1c7e78:38]");
    // The bots: count 3, Drake/Kiko/Rook, Phoenix, enabled [M 0x20c864/0x20c86c + the records].
    assert_eq!(fe.mp_bot_count(), 3, "0x2deea9 = 3, +0x1B0 = 3 [M 0x20c864/0x20c86c]");
    assert_eq!(fe.mp_bots[0].character, 1, "Drake [M DAT_002deeba]");
    assert_eq!(fe.mp_bots[1].character, 3, "Kiko [M DAT_002deecc]");
    assert_eq!(fe.mp_bots[2].character, 2, "Rook [M DAT_002deede]");
    assert!(fe.mp_bots.iter().take(3).all(|b| !b.team_mi6), "all three Phoenix [M record+0x0F = 0]");
    assert!(fe.mp_bots.iter().take(3).all(|b| b.enabled), "record+0x0E = 1 [M]");
    // Sister A: Back from the summary pops to the scenario page and the cursor re-derives from
    // MPSettings+0x1A4 = 1 [M P_MPCONFIRM's 0x5f pop + P_MPSCENARIO 0x217348:25's re-selection].
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::MpScenario);
    assert_eq!(fe.mp_scenario_row, 1, "the quick start's Deathmatch drives the cursor [M +0x1A4]");
    fe.mp_scenario_row = 0;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpConfirm);
    assert!(fe.action(Action::Confirm), "the confirm page's ✕ starts the match");
    let s = fe.mp_match_settings.as_ref().expect("the match config lands for the host");
    // The match's fields, each cited [M quick-game.md §2-§4].
    assert_eq!(s.scenario, 1, "+0x1A4 = 1 [M 0x20c740]");
    assert_eq!(s.gametype_mask, 1, "the same word — the game's gametype dispatch reads the scenario word itself: team = bit 29, assassin = bit 30 [M MP_Init 0x183438:41-42]");
    assert_eq!(s.level_code, MP_MAP_ROWS[0].3, "the port's row-0 sample [S quick-game.md §3]");
    assert!(MP_MAP_ROWS[..7].iter().all(|r| r.3 != 0x0700_004b), "rows 0..6 never Ravine [M the %7]");
    assert_eq!(s.frag_limit, 10, "+0x19C [M 0x20c738]");
    assert_eq!(s.time_limit, 600, "10 min ×0x3c at the confirm [M 0x20c72c + P_MPCONFIRM]");
    assert_eq!(s.human_count, 1, "+0x1AC the joined count [M 0x20c858]");
    assert_eq!(s.player_count, 1, "the same word [M 0x20c858]");
    assert_eq!(s.bot_count, 3, "+0x1B0 [M 0x20c86c]");
    assert_eq!(s.total_players, 4, "humans + bots [M MP_Start 0x183a50:164]");
    assert_eq!(s.weapon_set, 0, "+0x1B4 [M 0x20c760]");
    assert_eq!(s.radar, 1, "+0x1C4 [M 0x20c750]");
    assert_eq!(s.friendly_fire, 0, "+0x198 [M 0x20c754]");
    assert_eq!(s.location_damage, 1, "the boot value persists [M 0x1c7e78:45 — no 0x1C8 store]");
    assert_eq!(s.capture_reward, 0, "the boot value persists [M 0x1c7e78:46 — no 0x1BC store]");
    assert_eq!(s.spawn_policy, 2, "the boot view distance persists [M 0x1c7e78:28 — no 0x1C0 store]");
    assert_eq!(s.gun_emplacements, 0, "the boot value persists [M 0x1c7e78:38 — no 0x1B8 store]");
    assert_eq!(s.explosive_scenery, 0, "+0x1D4 [M 0x20c748]");
    assert_eq!(s.grapple, 0, "+0x1D0 [M 0x20c758]");
    assert_eq!(s.vehicles, 2, "+0x1CC Helicopters [M 0x20c734]");
    assert_eq!(s.master, 1, "+0x180 [M 0x20c73c]");
    assert_eq!(s.team_game, 0, "scenario 1's bit 29 [M MP_Init 0x183438:41]");
    assert_eq!(s.players[0].team, 1, "slot 0 MI6 [M 0x20c810 sw $v1, 4($a3)]");
    assert_eq!(s.players[0].skin, 0, "slot 0 Bond [M 0x20c818 sw $zero, 8($a3)]");
    assert_eq!(s.players[0].codename, "Player 1", "the template codename [M Txt 0x1c3 + slot]");
    for k in 1..4 {
        assert_eq!(s.players[k].team, 0, "bot slot {k} is Phoenix [M record+0x0F = 0]");
    }
    assert_eq!(s.bot_records.len(), 3, "three prepared records [M Menu_PrepareBots + the count]");
    assert_eq!(s.bot_records[0].character(), 1, "Drake [M DAT_002deeba]");
    assert_eq!(s.bot_records[1].character(), 3, "Kiko [M DAT_002deecc]");
    assert_eq!(s.bot_records[2].character(), 2, "Rook [M DAT_002deede]");
    assert!(s.bot_records.iter().all(|r| r.enabled() && r.team() == 0), "enabled, Phoenix [M]");
    assert_eq!(s.bot_records[0].trait_id(), 6, "Drake's preset trait [M BOT_getDefaultStats]");
    assert_eq!(s.bot_records[2].trait_id(), 7, "Rook's preset trait [M BOT_getDefaultStats]");
    assert_eq!(fe.requested_level, Some(fe.mp_level), "the arena request rides the loader");
    // The flow machine runs the load to the arena [M ResetMap_LevelToLoad + the phases; the
    // confirm's fade hold sits on top of the load state: P_MPCONFIRM's PushState(84.0f, 7) — 84
    // frames + the finished-frame pop (gameflow-state-stack) [M 0x20ec00, 0x1bada0]].
    for _ in 0..=84 { fe.tick(); }
    while fe.flow.get_state() == nf_view::menu::ST_MAPLOAD { fe.flow.load_tick(); }
    assert_eq!(fe.flow.current_level, fe.mp_level, "the loader reaches the arena");
}

#[test]
fn custom_game_keeps_its_chosen_scenario() {
    // G-053(6) sister: the quick start stomps MPSettings+0x1A4 to 1; a CUSTOM pick must still
    // carry ITS row's word to the match (C_SBMPSCEN's row-n arm writes mp_scenario[n].id [M
    // :44, disasm 0x20c9c8 `sw $v0, 0x4944($t0)`]).
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    // A quick game first (the stomp context), then back out to the wheel.
    fe.mp_scenario_row = 0;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_scenario, 1, "the quick start forced Deathmatch [M 0x20c740]");
    assert!(fe.action(Action::Back), "the summary pops to its opener");
    assert_eq!(fe.page, Page::MpScenario);
    // The custom pick: row 2 = Team Arena 0x20000002 — neither the sentinel 0 nor the quick 1.
    fe.mp_scenario_row = 2;
    assert!(fe.action(Action::Confirm), "an unlocked custom row navigates");
    iris_wait(&mut fe); // [M C_SBMPSCEN 0x20c600:45 — G-048(1)]
    assert_eq!(fe.page, Page::MpMap);
    assert_eq!(fe.mp_scenario, 0x2000_0002, "the row's word is stored [M +0x1A4 = mp_scenario[2].id]");
    fe.mp_map_row = 0; // Skyrail
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_setup_state, 2, "a team game asks Choose Team [M P_MPSETUP]");
    fe.mp_setup_sel = 1; // MI6
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_setup_state, 3, "then the character [M C_RBMPSETUP 2 → 3]");
    fe.mp_setup_sel = 4; // Alura, MI6 side
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_setup_state, 4, "then the Health Handicap [M C_RBMPSETUP 3 → 4]");
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpOptions);
    // A lone MI6 human fails the both-sides start check [M the 0x389 box]; a Phoenix bot (the
    // fresh record's Snow Guard) completes the sides.
    fe.mp_bots[0].enabled = true;
    fe.mp_options_row = 0;
    assert!(fe.action(Action::Confirm), "1 MI6 human + 1 Phoenix bot passes");
    assert_eq!(fe.page, Page::MpConfirm);
    assert!(fe.action(Action::Confirm), "the custom game's confirm starts it");
    let s = fe.mp_match_settings.as_ref().expect("the custom match config lands");
    assert_eq!(s.scenario, 0x2000_0002, "the custom scenario reaches the match [M +0x1A4]");
    assert_eq!(s.gametype_mask, 0x2000_0002, "the same word — bit 29 set makes it a team game [M MP_Init 0x183438:41-42]");
    assert_eq!(s.team_game, 1, "bit 29 set [M MP_Init 0x183438:41]");
    assert_ne!(s.scenario, 1, "the quick start's Deathmatch did not leak into the custom game");
}

#[test]
fn quick_game_keeps_a_custom_visits_mods() {
    // Sister C: the quick start's store list writes neither +0x1BC nor +0x1C0 (nor +0x1B8/
    // +0x1C8) [M 0x20c72c-0x20c768] — a custom visit's Professional Mode and View Distance
    // survive a later Quick Game.
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    // The custom walk to the hub [M the setup page chain — see setup_ffa_... above].
    fe.mp_scenario_row = 1;
    fe.action(Action::Confirm);
    iris_wait(&mut fe);
    fe.mp_map_row = 0;
    fe.action(Action::Confirm);
    fe.mp_setup_sel = 0; // Bond (FFA)
    fe.action(Action::Confirm);
    fe.action(Action::Confirm); // the handicap → options
    assert_eq!(fe.page, Page::MpOptions);
    // Player Mods row 2 (+0x1BC Professional Mode) ◀▶ toggles it on [M P_MPPLAYERMODS 0x1ac].
    fe.mp_options_row = 3;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpPlayerMods);
    fe.cursor = 2;
    assert!(fe.action(Action::Right));
    assert_eq!(fe.mp_player_mods[0], 1, "+0x1BC on [M 0x2104d0:14]");
    assert!(fe.action(Action::Confirm), "the page closes to the hub [M Manager 0x5f]");
    assert_eq!(fe.page, Page::MpOptions);
    // Enviro-Mods row 0 (View Distance +0x1C0) ◀▶ steps 2 → 0 Near [M P_MPENVIROMODS 0x1b0].
    fe.mp_options_row = 4;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpEnviroMods);
    fe.cursor = 0;
    assert!(fe.action(Action::Right));
    assert_eq!(fe.mp_envmods[0], 0, "+0x1C0 = Near [M 0x210030]");
    assert!(fe.action(Action::Confirm), "the page closes to the hub");
    assert_eq!(fe.page, Page::MpOptions);
    // Back out to the scenario wheel and run Quick Game.
    assert!(fe.action(Action::Back), "the hub pops to the setup page");
    assert_eq!(fe.page, Page::MpSetup);
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::MpMap);
    assert!(fe.action(Action::Back));
    assert_eq!(fe.page, Page::MpScenario);
    fe.mp_scenario_row = 0;
    assert!(fe.action(Action::Confirm), "Quick Game");
    assert_eq!(fe.page, Page::MpConfirm);
    assert!(fe.action(Action::Confirm), "start");
    let s = fe.mp_match_settings.as_ref().expect("the match config lands");
    assert_eq!(s.capture_reward, 1, "+0x1BC survives the quick start [M no 0x1BC store]");
    assert_eq!(s.spawn_policy, 0, "+0x1C0 survives the quick start [M no 0x1C0 store]");
    assert_eq!(s.location_damage, 1, "+0x1C8 untouched [M no 0x1C8 store]");
    assert_eq!(s.vehicles, 2, "+0x1CC = the quick start's own write [M 0x20c734]");
}

#[test]
fn team_scenario_asks_team_then_character() {
    let mut fe = FrontEnd::new();
    enter_mp(&mut fe);
    join_agent1(&mut fe);
    // Team Arena (row 2, scenario 0x20000002) has the team bit.
    fe.mp_scenario_row = 2;
    fe.action(Action::Confirm);
    iris_wait(&mut fe); // [M C_SBMPSCEN 0x20c600:45 — G-048(1)]
    fe.mp_map_row = 1;
    fe.action(Action::Confirm);
    assert_eq!(fe.mp_setup_state, 2, "a team game asks Choose Team [M P_MPSETUP's state 2]");
    // The team list has exactly two items (Phoenix/MI6).
    fe.mp_setup_sel = 1;
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_join[0].team, 1, "the selection IS the team [M mpjoin+4]");
    assert_eq!(fe.mp_setup_state, 3, "then the character list [M C_RBMPSETUP's 2 → 3]");
    fe.mp_setup_sel = 4; // Alura — MI6 side
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.mp_setup_state, 4, "then the Health Handicap [M C_RBMPSETUP's 3 → 4]");
    assert!(fe.action(Action::Confirm));
    assert_eq!(fe.page, Page::MpOptions);
    // Sister: a lone MI6 human in a team game fails the both-sides check.
    fe.mp_options_row = 0;
    assert!(!fe.action(Action::Confirm), "an empty Phoenix side refuses [M the 0x389 box]");
    assert_eq!(fe.mp_error, 0x389);
}

#[test]
fn the_lists_match_the_game_rows_headless() {
    // The tables the flow walks carry the game's own row counts and ids (the ISO-gated
    // mp_menu_lists_live.rs re-pins the bytes from the disc).
    assert_eq!(MP_SCENARIO_ROWS.len(), 13, "mp_scenario's 13 rows [M symtab size 312 = 13×0x18]");
    assert_eq!(MP_MAP_ROWS.len(), 8, "mp_level's 8 rows [M symtab 192]");
    assert_eq!(MP_OPTIONS_ROWS.len(), 5, "mp_options' 5 rows [M the confirm's switch]");
    assert_eq!(MP_SCENARIO_ROWS[1].3, 1, "Arena = Deathmatch [M MPSettings+0x1A4 = 1]");
    assert_eq!(MP_SCENARIO_ROWS[2].3, 0x2000_0002, "Team Arena");
    assert_eq!(MP_SCENARIO_ROWS[12].3, 0x6000_1000, "Team KOTH");
    // Sister: the map rows' level codes are the arenas the ISO ships.
    for r in MP_MAP_ROWS { assert_eq!((r.3 >> 16) & 0xffff, 0x0700, "an on-foot arena code"); }
    assert!(MP_MAP_ROWS.iter().any(|r| r.3 == 0x0700_004b), "Ravine is the 8th arena");
    assert!(!MP_MAP_ROWS.iter().any(|r| r.3 == 0x0700_0021), "0x07000021 is absent from the table [D]");
}
