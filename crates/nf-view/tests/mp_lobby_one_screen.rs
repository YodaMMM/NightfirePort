//! mp-lobby-one-screen: the ONE-screen online lobby, headless loopback (no ISO, no sockets, no
//! threads) — numbers and ids only. Two FrontEnds (host + client) share one in-memory net so
//! the host's publish reaches the client exactly as the session tunnel would (the real
//! transport is the lockstep DATA tunnel, docs/systems/net-online-lobby.md §6.4):
//!
//! - host edit → the client's rows show it on the next tick;
//! - the client cannot edit the host's rows (the editable matrix);
//! - Start is gated on everyone ready, then the handover rides the session;
//! - the built MpSettings equals what the couch pages build for the same choices
//!   [M P_MPCONFIRM 0x20ec00:94-157, the msg-0x4b start arm; mp_confirm_start is the port.s name for it];
//! - the LobbyState / LobbyEdit codec round-trips (nf-net).

use std::cell::RefCell;
use std::rc::Rc;

use nf_game::mp::MpSettings;
use nf_view::menu::{
    Action, FrontEnd, MpNetAgent, MpNetChoices, MpNetError, MpNetLink, MpSession, MpNetState,
    Page, MP_MAP_ROWS, MP_SCENARIO_ROWS, PAGE_MP_ONLINE_LOBBY,
};
use nf_view::mp_lobby::MpLobbyRowId;

// ── the shared fake net (host-relative slots: 0 = the host, 1..3 = the clients) ─────────────

struct Net {
    link_host: MpNetLink,
    link_client: MpNetLink,
    agents: Vec<MpNetAgent>,
    choices: Option<MpNetChoices>,
    started: Option<MpSettings>,
    lobby: Option<nf_net::LobbyState>,
    edits: Vec<nf_net::LobbyEdit>,
    client_slot: usize,
    closed: usize,
    codename_host: String,
    codename_client: String,
    /// poll() turns a Dialing client link into Connected.
    advise_connected: bool,
}

struct HostHandle(Rc<RefCell<Net>>);
struct ClientHandle(Rc<RefCell<Net>>);

impl MpSession for HostHandle {
    fn poll(&mut self) {}
    fn link(&self) -> MpNetLink { self.0.borrow().link_host }
    fn listen(&mut self, _port: u16) -> Result<(), MpNetError> {
        self.0.borrow_mut().link_host = MpNetLink::Listening;
        let mut n = self.0.borrow_mut();
        if !n.agents.iter().any(|a| a.slot == 0) {
            let name = n.codename_host.clone();
            n.agents.insert(0, MpNetAgent { slot: 0, codename: name, ready: true });
        }
        Ok(())
    }
    fn connect(&mut self, _addr: &str) -> Result<(), MpNetError> { Ok(()) }
    fn agents(&self) -> Vec<MpNetAgent> { self.0.borrow().agents.clone() }
    fn host_choices(&self) -> Option<MpNetChoices> { self.0.borrow().choices.clone() }
    fn publish_choices(&mut self, c: &MpNetChoices) { self.0.borrow_mut().choices = Some(c.clone()); }
    fn start_match(&mut self, s: &MpSettings) { self.0.borrow_mut().started = Some(s.clone()); }
    fn match_started(&self) -> Option<MpSettings> { self.0.borrow().started.clone() }
    fn set_codename(&mut self, name: &str) { self.0.borrow_mut().codename_host = name.into(); }
    fn local_slot(&self) -> usize { 0 }
    fn lobby_state(&self) -> Option<nf_net::LobbyState> { self.0.borrow().lobby.clone() }
    fn publish_lobby(&mut self, s: &nf_net::LobbyState) { self.0.borrow_mut().lobby = Some(s.clone()); }
    fn push_row_edit(&mut self, _e: &nf_net::LobbyEdit) {}
    fn take_row_edit(&mut self) -> Option<nf_net::LobbyEdit> { self.0.borrow_mut().edits.pop() }
    fn close(&mut self) { self.0.borrow_mut().closed += 1; }
}

impl MpSession for ClientHandle {
    fn poll(&mut self) {
        let mut n = self.0.borrow_mut();
        if n.link_client == MpNetLink::Dialing && n.advise_connected {
            n.link_client = MpNetLink::Connected;
            // The handshake lands the client in the host's agent list (host-relative slot).
            let slot = n.client_slot as usize;
            let name = n.codename_client.clone();
            n.agents.retain(|a| a.slot != slot);
            n.agents.push(MpNetAgent { slot, codename: name, ready: false });
            n.agents.sort_by_key(|a| a.slot);
        }
    }
    fn link(&self) -> MpNetLink { self.0.borrow().link_client }
    fn listen(&mut self, _port: u16) -> Result<(), MpNetError> { Ok(()) }
    fn connect(&mut self, _addr: &str) -> Result<(), MpNetError> {
        self.0.borrow_mut().link_client = MpNetLink::Dialing;
        Ok(())
    }
    fn agents(&self) -> Vec<MpNetAgent> { self.0.borrow().agents.clone() }
    fn host_choices(&self) -> Option<MpNetChoices> { self.0.borrow().choices.clone() }
    fn publish_choices(&mut self, _c: &MpNetChoices) {}
    fn start_match(&mut self, s: &MpSettings) { self.0.borrow_mut().started = Some(s.clone()); }
    fn match_started(&self) -> Option<MpSettings> { self.0.borrow().started.clone() }
    fn set_codename(&mut self, name: &str) { self.0.borrow_mut().codename_client = name.into(); }
    fn local_slot(&self) -> usize { self.0.borrow().client_slot }
    fn lobby_state(&self) -> Option<nf_net::LobbyState> { self.0.borrow().lobby.clone() }
    fn publish_lobby(&mut self, _s: &nf_net::LobbyState) {}
    fn push_row_edit(&mut self, e: &nf_net::LobbyEdit) { self.0.borrow_mut().edits.push(e.clone()); }
    fn take_row_edit(&mut self) -> Option<nf_net::LobbyEdit> { None }
    fn close(&mut self) { self.0.borrow_mut().closed += 1; }
}

/// Host + client front ends over one shared net.
fn pair() -> (FrontEnd, FrontEnd, Rc<RefCell<Net>>) {
    let net = Rc::new(RefCell::new(Net {
        link_host: MpNetLink::Idle,
        link_client: MpNetLink::Idle,
        agents: Vec::new(),
        choices: None,
        started: None,
        lobby: None,
        edits: Vec::new(),
        client_slot: 1,
        closed: 0,
        codename_host: String::new(),
        codename_client: String::new(),
        advise_connected: false,
    }));
    let mut host = FrontEnd::new();
    host.mp_session = Some(Box::new(HostHandle(net.clone())));
    let mut client = FrontEnd::new();
    client.mp_session = Some(Box::new(ClientHandle(net.clone())));
    (host, client, net)
}

/// The host: Main → Multiplayer → Host Online → the host page → Start Listening → the one screen.
fn host_to_lobby(fe: &mut FrontEnd) {
    fe.tick();
    fe.page = Page::Main;
    fe.cursor = 1;
    assert!(fe.action(Action::Confirm), "Multiplayer confirm must navigate");
    assert_eq!(fe.page, Page::MpMode, "the mode choice is up [S]");
    fe.cursor = 1;
    assert!(fe.action(Action::Confirm), "Host Online confirm navigates");
    assert_eq!(fe.page, Page::MpHost);
    fe.cursor = 1;
    assert!(fe.action(Action::Confirm), "Start Listening navigates");
    assert_eq!(fe.page, Page::MpOnlineLobby, "the online path lands on the ONE screen [S §6.1]");
    assert_eq!(fe.page.id(), PAGE_MP_ONLINE_LOBBY);
    assert!(fe.mp_hosting, "host mode armed");
    assert!(fe.mp_one.host, "the model is the host's");
}

/// The client: Main → Multiplayer → Join Online → type → Connect → (advise connected) → tick.
fn client_to_lobby(fe: &mut FrontEnd, net: &Rc<RefCell<Net>>) {
    // NOTE: the caller pairs this with a host tick when it wants the rows live; `live_pair`
    // below does both.
    fe.tick();
    fe.page = Page::Main;
    fe.cursor = 1;
    assert!(fe.action(Action::Confirm));
    fe.cursor = 2;
    assert!(fe.action(Action::Confirm), "Join Online confirm navigates");
    assert_eq!(fe.page, Page::MpNetJoin);
    for c in "127.0.0.1:27420".chars() { assert!(fe.mp_net_type(c)); }
    assert!(fe.action(Action::Confirm), "Connect dials");
    assert_eq!(fe.mp_net_state, MpNetState::Dialing);
    net.borrow_mut().advise_connected = true;
    fe.tick();
    assert_eq!(fe.page, Page::MpOnlineLobby, "connected → the same ONE screen [S §6.1]");
    assert!(!fe.mp_one.host, "the model is the client's");
    assert_eq!(fe.mp_one.own_slot, 1, "the client's own row is its session slot [S §6.4]");
    // The lobby goes live: the host's next tick mirrors the client into its model and
    // publishes, the client's next tick applies the snapshot [S §6.4 — the tick cadence].
    fe.tick();
}

/// The focus lands on a row with the given id; returns its index.
fn focus_on(fe: &mut FrontEnd, id: MpLobbyRowId) -> usize {
    let rows = fe.mp_lobby_rows().expect("the lobby page is up");
    let idx = rows.iter().position(|r| r.id == id).unwrap_or_else(|| panic!("no {id:?} row"));
    fe.mp_one.focus = idx;
    idx
}

/// A pair with the lobby live both ways (host mirrored + published, client applied).
fn live_pair() -> (FrontEnd, FrontEnd, Rc<RefCell<Net>>) {
    let (mut host, mut client, net) = pair();
    host_to_lobby(&mut host);
    client_to_lobby(&mut client, &net);
    host.tick();
    client.tick();
    (host, client, net)
}

// ── the tests ────────────────────────────────────────────────────────────────────────────────

#[test]
fn host_edit_reaches_the_client_on_the_next_tick() {
    let (mut host, mut client, net) = pair();
    host_to_lobby(&mut host);
    // The host steps the map row ◀ (Skyrail row 0 → Ravine row 7, the wheel wraps backwards).
    focus_on(&mut host, MpLobbyRowId::Map);
    let before = host.mp_one.level;
    assert!(host.mp_one.apply(Action::Left), "the map row is editable for the host");
    assert_ne!(host.mp_one.level, before, "the map word moved");
    assert_eq!(host.mp_one.level, MP_MAP_ROWS[7].3, "◀ from row 0 wraps to row 7 — [S] unverified: the port.s wheel walk; no decompile line pinned");
    // The publish rides the tick (the diff path).
    host.tick();
    assert!(net.borrow().lobby.is_some(), "the host published its snapshot");
    // The client, in the lobby, sees it on its next tick.
    client_to_lobby(&mut client, &net);
    assert!(client.mp_one_published.is_some(), "the client applied a snapshot already");
    let rows = client.mp_lobby_rows().expect("the lobby page is up");
    let map = rows.iter().find(|r| r.id == MpLobbyRowId::Map).expect("the map row");
    assert_eq!(map.value_id, MP_MAP_ROWS[7].3.name_id_helper(), "the client's map value is the host's word");
    // Sister: the client sees the host's scenario too.
    let scen = rows.iter().find(|r| r.id == MpLobbyRowId::Scenario).expect("the scenario row");
    assert_eq!(scen.label_id, 0x3d5, "the confirm page's own Scenario label [M P_MPCONFIRM]");
}

/// The map row's value is the mp_map name label; a tiny helper keeps the test's intent readable.
trait NameIdHelper { fn name_id_helper(&self) -> u32; }
impl NameIdHelper for u32 {
    fn name_id_helper(&self) -> u32 {
        MP_MAP_ROWS.iter().find(|r| r.3 == *self).map(|r| r.1).unwrap_or(0)
    }
}

#[test]
fn the_client_cannot_edit_host_rows() {
    let (mut host, mut client, net) = live_pair();
    // The getter says the host's rows are not the client's to edit.
    for r in client.mp_lobby_rows().unwrap() {
        if !matches!(r.id, MpLobbyRowId::Player(1) | MpLobbyRowId::PlayerTeam(1)) {
            assert!(!r.editable, "{:?} must not be editable for the client [S §6.2]", r.id);
        }
    }
    // And the actions refuse: the map word does not move.
    focus_on(&mut client, MpLobbyRowId::Map);
    let before = client.mp_one.level;
    assert!(!client.mp_one.apply(Action::Right), "the client's ◀ ▶ on a host row refuses");
    assert_eq!(client.mp_one.level, before, "the map word stayed");
    assert!(!client.mp_one.apply(Action::Confirm), "Confirm on a host row refuses too");
    // Sister: the client CAN edit its own row — the ready toggle flips.
    focus_on(&mut client, MpLobbyRowId::Player(1));
    let was_ready = client.mp_one.players[1].ready;
    assert!(client.mp_one.apply(Action::Confirm), "the own row's Confirm toggles ready");
    assert_ne!(client.mp_one.players[1].ready, was_ready, "the own ready flag moved");
    // The edits ride out on the tick and the host's model takes them.
    client.tick();
    host.tick();
    assert_ne!(host.mp_one.players[1].ready, was_ready, "the host merged the client's edit");
    // The host's rows are editable for the host.
    focus_on(&mut host, MpLobbyRowId::WeaponSet);
    let w = host.mp_one.weapon_set;
    assert!(host.mp_one.apply(Action::Right), "the host edits its own screen");
    assert_eq!(host.mp_one.weapon_set, (w + 1) % 11, "the weapon set wraps 0..10 [M 0x2104d0:26-49]");
    let _ = &net; // the shared net kept both sides honest
}

#[test]
fn start_is_gated_on_ready_and_hands_the_match_over() {
    let (mut host, mut client, net) = live_pair();
    // The client arrived NOT ready (the fake joins remote agents ready = false) — sister: the
    // getter's gate is closed.
    assert!(host.mp_one.players[1].joined, "the client's row joined on the mirror");
    assert!(!host.mp_lobby_start_enabled(), "a not-ready client holds Start [S §6.2]");
    focus_on(&mut host, MpLobbyRowId::Start);
    assert!(!host.action(Action::Confirm), "Start refuses while someone is not ready");
    assert!(host.mp_match_settings.is_none(), "no match was built");
    assert!(host.requested_level.is_none(), "no level request either");
    // The client readies itself through its own row; the edit rides out.
    focus_on(&mut client, MpLobbyRowId::Player(1));
    assert!(client.action(Action::Confirm), "the own row's ready toggle");
    client.tick();
    host.tick();
    assert!(host.mp_one.players[1].ready, "the host sees the ready");
    assert!(host.mp_lobby_start_enabled(), "everyone ready opens Start");
    // The Start: the model's words build the settings and the handover rides the session.
    assert!(host.action(Action::Confirm), "Start hands the match over");
    assert!(host.mp_match_settings.is_some(), "the host built the settings");
    assert_eq!(host.requested_level, Some(host.mp_one.level), "the level request rides the load");
    assert!(net.borrow().started.is_some(), "the settings crossed the session");
    // The client's next tick hands its match over too (the same confirm-page tail).
    client.tick();
    assert!(client.mp_match_settings.is_some(), "the client's handover ran");
    assert_eq!(client.requested_level, host.requested_level, "both sides load the same arena");
    // Sister: the fake's edit path carried exactly one field change (ready), not noise.
    assert_eq!(net.borrow().edits.len(), 0, "the host drained every edit");
}

#[test]
fn the_built_settings_equal_the_couch_build() {
    // The same choices through both paths: the couch walk's mp_confirm_start vs the lobby's
    // mp_lobby_start [M P_MPCONFIRM 0x20ec00:94-157, the msg-0x4b start arm; mp_confirm_start is the port.s name for it].
    let choices = |fe: &mut FrontEnd| {
        fe.mp_scenario = MP_SCENARIO_ROWS[2].3; // Team Arena (a team game, the team rows exist)
        fe.mp_level = MP_MAP_ROWS[3].3;
        fe.mp_weapon_set = 4;
        fe.mp_frag_limit = 15;
        fe.mp_time_limit = 7;
        fe.mp_friendly_fire = 1;
        fe.mp_radar = 0;
    };
    // The couch path: one joined, ready human + one bot (the two-player minimum [M C_SBMPOPTIONS 0x20e818:104-109 (humans + bots > 1 sends the 0x44/0x40000049 start); 0x39e is a label id]).
    let mut couch = FrontEnd::new();
    choices(&mut couch);
    couch.mp_join[0] = nf_view::menu::MpJoinAgent {
        state: 5, joined: true, ready: true, team: 1, skin: 2, codename_row: 0, codename: "Player 1".into(),
    };
    // The profile-less client is named by its slot on the host ("Player 2" — two machines both
    // reading "Player 1" was the two-window check's duplicate) [S].
    couch.mp_join[1] = nf_view::menu::MpJoinAgent {
        state: 5, joined: true, ready: true, team: 0, skin: 1, codename_row: 0, codename: "Player 2".into(),
    };
    assert!(couch.mp_confirm_start(), "the couch confirm starts");
    let couch_s = couch.mp_match_settings.clone().expect("the couch build");

    // The lobby path: the same words through the model.
    let (mut host, mut client, net) = live_pair();
    // The client readies (the two humans the couch build carried).
    focus_on(&mut client, MpLobbyRowId::Player(1));
    client.mp_one.apply(Action::Confirm);
    client.tick();
    host.tick();
    // The host's choices = the couch choices, through the model's own words.
    host.mp_one.scenario = MP_SCENARIO_ROWS[2].3;
    host.mp_one.level = MP_MAP_ROWS[3].3;
    host.mp_one.weapon_set = 4;
    host.mp_one.frag_limit = 15;
    host.mp_one.time_limit = 7;
    host.mp_one.friendly_fire = 1;
    host.mp_one.radar = 0;
    // The player rows mirror the couch walk's join records (codename/team/skin).
    host.mp_one.players[0] = nf_view::mp_lobby::MpLobbyPlayer {
        codename: "Player 1".into(), character: 2, team: 1, ready: true, joined: true, ping_ms: 0,
    };
    host.mp_one.players[1].team = 0;
    host.mp_one.players[1].character = 1;
    host.mp_one.players[1].ready = true;
    focus_on(&mut host, MpLobbyRowId::Start);
    assert!(host.action(Action::Confirm), "the lobby Start builds");
    let lobby_s = host.mp_match_settings.clone().expect("the lobby build");
    // The structs are equal — the same fields, the same values.
    assert_eq!(lobby_s.scenario, couch_s.scenario, "the scenario word [M +0x1A4]");
    assert_eq!(lobby_s.level_code, couch_s.level_code, "the arena [M +0x1A8]");
    assert_eq!(lobby_s.weapon_set, couch_s.weapon_set, "the weapon set [M +0x1B4]");
    assert_eq!(lobby_s.frag_limit, couch_s.frag_limit, "the limit [M +0x19C]");
    assert_eq!(lobby_s.time_limit, couch_s.time_limit, "the confirmed time (minutes ×60) [M the ×0x3c]");
    assert_eq!(lobby_s.friendly_fire, couch_s.friendly_fire, "friendly fire [M +0x198]");
    assert_eq!(lobby_s.radar, couch_s.radar, "radar (the port.s +0x1C4 mapping — [S] unverified; P_MPPLAYERMODS 0x2104d0:16 writes the control-0x1000008d on/off item there)");
    assert_eq!(lobby_s.human_count, couch_s.human_count, "the humans");
    assert_eq!(lobby_s.master, 1, "master = 1 [M +0x180 = 1]");
    assert_eq!(lobby_s.team_game, couch_s.team_game, "the team flag re-derives [M MP_Init 0x183438:41]");
    let p = |r: &nf_game::mp::MpPlayer| (r.team, r.skin, r.radar_on, r.codename.clone());
    assert_eq!(p(&lobby_s.players[0]), p(&couch_s.players[0]), "the host's player record");
    assert_eq!(p(&lobby_s.players[1]), p(&couch_s.players[1]), "the client's player record");
    let _ = &net;
}

#[test]
fn the_model_rows_carry_the_game_labels_and_gates() {
    // The rows' game-side label ids and gates, standalone (no session needed) [S §6.2; the
    // label ids are the confirm page's own pairs (0x3d4 Map / 0x3d5 Scenario / 0x24d Weapon
    // Set / 0x3d8-0x3d7 limit / 0x3d6 Duration / 0x3da Friendly Fire [M P_MPCONFIRM's enter])].
    let mut fe = FrontEnd::new();
    fe.page = Page::MpOnlineLobby;
    fe.mp_one = nf_view::mp_lobby::MpLobby::new_host(1, 0x0700_0024, 0, 1, 0, 10, 10, [0, 1], [2, 0, 0, 0, 0],
        std::array::from_fn(nf_view::menu::MpBot::fresh), "Player 1", false, 0);
    let rows = fe.mp_lobby_rows().unwrap();
    let ids: Vec<(MpLobbyRowId, u32)> = rows.iter().map(|r| (r.id, r.label_id)).collect();
    assert!(ids.contains(&(MpLobbyRowId::Map, 0x3d4)), "the Map label [M P_MPCONFIRM]");
    assert!(ids.contains(&(MpLobbyRowId::Scenario, 0x3d5)), "the Scenario label [M]");
    assert!(ids.contains(&(MpLobbyRowId::WeaponSet, 0x24d)), "the Weapon Set label [M]");
    assert!(ids.contains(&(MpLobbyRowId::Duration, 0x3d6)), "the Duration label [M]");
    assert!(ids.contains(&(MpLobbyRowId::FriendlyFire, 0x3da)), "the Friendly Fire label [M]");
    // The limit row's unit: "Points" 0x3d8 in a deathmatch, "Lives" 0x3d7 for Top Agent [M
    // mp_rules_lives — P_MPRULES 0x20f790:78-95].
    assert!(rows.iter().any(|r| r.id == MpLobbyRowId::Limit && r.label_id == 0x3d8), "a deathmatch's limit label");
    fe.mp_one.scenario = 0x10;
    let rows = fe.mp_lobby_rows().unwrap();
    assert!(rows.iter().any(|r| r.id == MpLobbyRowId::Limit && r.label_id == 0x3d7), "Top Agent's limit label");
    // A locked scenario refuses the step onto it [M C_SBMPSCEN 0x20c600:40-42 — the port maps
    // the refusal onto ◀ ▶ [S]]; Uplink (row 4) is reward-locked on a fresh profile.
    fe.mp_one.scenario = MP_SCENARIO_ROWS[3].3; // CTF, unlocked; row 4 (Uplink) is locked
    fe.mp_one.focus = 0;
    assert!(fe.mp_one.apply(Action::Right), "CTF steps");
    assert_eq!(fe.mp_one.scenario, MP_SCENARIO_ROWS[5].3, "the step skipped locked Uplink row 4");
    // The team rows exist only in team games [M P_MPSETUP 0x20d7a8's state-2 arm].
    fe.mp_one.scenario = 1; // Arena, FFA
    let rows = fe.mp_lobby_rows().unwrap();
    assert!(!rows.iter().any(|r| matches!(r.id, MpLobbyRowId::PlayerTeam(_))), "no team rows in an FFA");
    // Sister: with the host alone joined and ready, Start is open.
    assert!(rows.iter().any(|r| r.id == MpLobbyRowId::Start && r.enabled), "the ready host opens Start");
    fe.mp_one.scenario = MP_SCENARIO_ROWS[2].3; // Team Arena
    fe.mp_one.players[1].joined = true;
    let rows = fe.mp_lobby_rows().unwrap();
    assert!(rows.iter().any(|r| matches!(r.id, MpLobbyRowId::PlayerTeam(1))), "team rows in a team game");
}

#[test]
fn every_row_is_reachable_with_the_bots_off() {
    // row_count must equal the row list: a disabled bot still lists its (blank) Skill row, and
    // counting it once stranded the focus on Bot 4 — the player rows and Start unreachable
    // (the two-window menu check) [S].
    let mut one = nf_view::mp_lobby::MpLobby::new_host(1, 0x0700_0024, 0, 1, 0, 10, 10, [0, 1], [2, 0, 0, 0, 0],
        std::array::from_fn(nf_view::menu::MpBot::fresh), "Player 1", false, 0);
    for enabled in [false, true] {
        for b in one.bots.iter_mut() { b.enabled = enabled; }
        let n = one.rows().len();
        assert_eq!(one.row_count(), n, "row_count with bots enabled={enabled}");
        for _ in 0..n + 5 { one.apply(nf_view::menu::Action::Down); }
        assert_eq!(one.focused_row().map(|(id, _)| id), Some(MpLobbyRowId::Start), "Down reaches Start (bots enabled={enabled})");
        for _ in 0..n + 5 { one.apply(nf_view::menu::Action::Up); }
    }
}

/// A match set to 16 bots spawned none — the match was on Ravine, where the
/// game takes no bots [M P_MPMAP 0x217678:20-25; Menu_PrepareBots 0x1f94a0:85-88], but the lobby let the host turn
/// them on. On Ravine the bot rows read "Not on this map" and can't be edited; the start carries no bots.
#[test]
fn ravine_shows_its_bot_rows_as_unavailable() {
    let lobby_on = |level: u32| {
        let mut fe = FrontEnd::new();
        fe.page = Page::MpOnlineLobby;
        let mut bots: [nf_view::menu::MpBot; nf_net::lobby::BOTS] = std::array::from_fn(nf_view::menu::MpBot::fresh);
        for b in bots.iter_mut().take(3) { b.enabled = true; }
        fe.mp_one = nf_view::mp_lobby::MpLobby::new_host(1, level, 0, 1, 0, 10, 10, [0, 1], [2, 0, 0, 0, 0],
            bots, "Player 1", false, 0);
        fe
    };
    let mut ravine = lobby_on(0x0700_004b);
    let rows = ravine.mp_lobby_rows().unwrap();
    let bot_rows: Vec<_> = rows.iter().filter(|r| matches!(r.id, MpLobbyRowId::Bot(_))).collect();
    assert!(!bot_rows.is_empty() && bot_rows.iter().all(|r| !r.editable && r.value_text == "Not on this map"),
        "every Ravine bot row is locked and says why");
    assert!(!rows.iter().any(|r| matches!(r.id, MpLobbyRowId::BotSkill(_))), "no skill rows on Ravine");
    // The start copies the lobby's map and bots into the couch fields (mp_lobby_start); with three enabled bots on
    // Ravine the count is still 0.
    ravine.mp_level = ravine.mp_one.level;
    ravine.mp_bots = ravine.mp_one.bots;
    assert_eq!(ravine.mp_bots.iter().filter(|b| b.enabled).count(), 3);
    assert_eq!(ravine.mp_bot_count(), 0, "the start carries no bots on Ravine [M]");
    // Sister: on Skyrail the same lobby offers its bots, and all three enabled ones count.
    let mut skyrail = lobby_on(0x0700_0024);
    let rows = skyrail.mp_lobby_rows().unwrap();
    assert!(rows.iter().filter(|r| matches!(r.id, MpLobbyRowId::Bot(_))).all(|r| r.editable), "Skyrail's bot rows are editable");
    skyrail.mp_level = skyrail.mp_one.level;
    skyrail.mp_bots = skyrail.mp_one.bots;
    assert_eq!(skyrail.mp_bot_count(), 3);
}
