//! mp-teams loopback: the team picks travel the session's DATA tunnel — a client's `TeamEdit`
//! reaches the host and the host's applied `MatchSettings` (with the teams in the player records)
//! reached every client through the handshake (`Event::Connected`), so both sides' effective teams
//! agree. Numbers/ids only (the main agent draws the pages).

use std::time::{Duration, Instant};

use nf_net::net::Transport;
use nf_net::session::{Event, Session};
use nf_net::teams::{start_check, scenario_has_teams, TeamEdit, TeamSelection, EDIT_TEAM, TEAM_MI6, TEAM_PHOENIX};
use nf_net::MatchSettings;

/// A plain loopback UDP transport (no injected loss — the tunnel's own delivery is not what's
/// under test here; the lossy variants live in loopback_e2e/lockstep).
struct Loop {
    sock: std::net::UdpSocket,
}

impl Loop {
    fn bind(addr: &str) -> Loop {
        let sock = std::net::UdpSocket::bind(addr).unwrap();
        sock.set_nonblocking(true).expect("the session's poll is a non-blocking drain");
        Loop { sock }
    }
}

impl Transport for Loop {
    fn send(&mut self, addr: std::net::SocketAddr, data: &[u8]) {
        let _ = self.sock.send_to(data, addr);
    }
    fn try_recv(&mut self) -> Option<(std::net::SocketAddr, Vec<u8>)> {
        let mut buf = [0u8; 2048];
        match self.sock.recv_from(&mut buf) {
            Ok((n, a)) => Some((a, buf[..n].to_vec())),
            Err(_) => None,
        }
    }
    fn local_addr(&self) -> std::net::SocketAddr {
        self.sock.local_addr().unwrap()
    }
}

fn host_addr(port: u16) -> std::net::SocketAddr {
    format!("127.0.0.1:{port}").parse().unwrap()
}

/// One host + one client over real loopback; returns both sessions plus the client's received
/// handshake settings (`Event::Connected`'s authoritative copy).
fn bring_up(port: u16, settings: MatchSettings) -> (Session<Loop>, Session<Loop>, MatchSettings) {
    let mut host = Session::new_host(
        nf_net::Config::default(),
        Loop::bind(&format!("127.0.0.1:{port}")),
        settings,
        0x1234_5678,
    );
    let mut client = Session::new_client(
        nf_net::Config::default(),
        Loop::bind("127.0.0.1:0"),
        host_addr(port),
    );
    let mut ms: u64 = 0;
    let mut connected: Option<MatchSettings> = None;
    let deadline = Instant::now() + Duration::from_secs(10);
    while client.local_slot() == 0 {
        ms += 5;
        for ev in client.poll(ms) {
            if let Event::Connected { settings, .. } = ev {
                connected = Some(settings);
            }
        }
        host.poll(ms);
        assert!(Instant::now() < deadline, "handshake must complete");
        std::thread::sleep(Duration::from_millis(1));
    }
    let client_settings = connected.expect("Connected carries the match settings");
    host.start_match(0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !client.is_started() {
        ms += 5;
        client.poll(ms);
        host.poll(ms);
        assert!(Instant::now() < deadline, "GO must reach the client");
        std::thread::sleep(Duration::from_millis(1));
    }
    (host, client, client_settings)
}

/// The scenario gates the lobby's column: team rows show it, FFA rows don't (unit-pinned in
/// teams.rs; here through the public re-exports the draw code would call).
#[test]
fn the_column_gate_and_start_check_are_the_public_flow() {
    assert!(scenario_has_teams(0x2000_0004)); // CTF [L MP_SCENARIO_ROWS]
    assert!(!scenario_has_teams(0x400)); // Assassin shows no team column [M 0x183438:41-42]
    // A CTF lobby: 2 humans (Phoenix + MI6) and 2 Phoenix bots pass the Start gate [M 0x20e818:50-130].
    let ts = TeamSelection::default();
    let mut s = MatchSettings::default();
    ts.apply(&mut s, 2);
    let mut teams = ts.effective_teams(&s, 2);
    teams[4] = TEAM_PHOENIX;
    teams[5] = TEAM_PHOENIX;
    assert_eq!(start_check(0x2000_0004, &teams, 2, 2), Ok(()));
    assert_eq!(s.players[0].0, TEAM_PHOENIX);
    assert_eq!(s.players[1].0, TEAM_MI6);
    assert_eq!(nf_net::team_label(TEAM_MI6), 0x1c8, "the row's label id [M 0x20de58:130-139]");
    assert_eq!(EDIT_TEAM, 1, "the lobby codec's team field id [S net-online-lobby §6]");
}

/// A client's TeamEdit rides the DATA tunnel to the host (Event::Data), which applies it into its
/// MatchSettings — and the host's settings the client received through the handshake carry the
/// same teams, so both sides' effective teams agree.
#[test]
fn team_edits_travel_the_data_tunnel_and_the_settings_carry_the_teams() {
    // The host's planned settings: CTF, 2 humans, 1 bot — the bot rides MI6 from its record.
    let ts = TeamSelection::default();
    let mut settings = MatchSettings::default();
    ts.apply(&mut settings, 2);
    settings.scenario = 0x2000_0004;
    settings.team_game = (settings.scenario >> 29) & 1;
    settings.human_count = 2;
    settings.bot_count = 1;
    settings.players[4].0 = TEAM_MI6; // the bot record's team [M 0x183a50:140-141]

    let (mut host, mut client, client_settings) = bring_up(45710, settings);

    // The client's pick: slot 1 switches to Phoenix — encoded and sent over the tunnel.
    let mut frame = Vec::new();
    TeamEdit { slot: 1, team: TEAM_PHOENIX }.encode(&mut frame);
    client.send_data(frame);

    // The host polls until the edit arrives.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut seen = None;
    'outer: while Instant::now() < deadline {
        let mut ms: u64 = 0;
        for _ in 0..40 {
            ms += 5;
            for ev in host.poll(ms) {
                if let Event::Data { bytes, .. } = ev {
                    if let Some(edit) = TeamEdit::decode(&bytes) {
                        seen = Some(edit);
                        break 'outer;
                    }
                }
            }
            client.poll(ms);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    let edit = seen.expect("the client's team edit must arrive over the DATA tunnel");
    assert_eq!(edit, TeamEdit { slot: 1, team: TEAM_PHOENIX });

    // The host applies it (the lobby's merge step) and the gate passes for CTF; the bot record
    // keeps its own team.
    let mut ts = ts;
    ts.set_team(edit.slot as usize, edit.team).unwrap();
    let mut applied = MatchSettings::default();
    applied.players[4].0 = TEAM_MI6;
    ts.apply(&mut applied, 2);
    let teams = ts.effective_teams(&applied, 2);
    assert_eq!(start_check(0x2000_0004, &teams, 2, 1), Ok(()));
    assert_eq!(applied.players[1].0, TEAM_PHOENIX, "the applied pick landed in the settings record");
    assert_eq!(applied.players[4].0, TEAM_MI6, "the bot record keeps its own team [M 0x183a50:140-141]");

    // Sister: the client's handshake copy of the settings carries the same teams (the slot records
    // ride the protocol-2 wire — settings.rs's players tuples).
    assert_eq!(client_settings.players[0].0, TEAM_PHOENIX);
    assert_eq!(client_settings.players[1].0, TEAM_MI6, "the pick made AFTER the handshake doesn't retro-edit it — the host republishes per the lobby flow");
    assert_eq!(client_settings.players[4].0, TEAM_MI6);
}
