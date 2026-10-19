//! mp-hud-real — headless: the per-viewer score pane (`HUD_CreateMPScorePane` 0x19ed10 +
//! `HUD_MPUpdatePane` 0x19f0a0), the status line (`MP_SortOutWhoWon` 0x1848d0 / `MP_Update`
//! 0x184bd0 case 2) and the kill message queue (`MP_PlayerKilled` 0x188f70:100-107). The pane
//! positions are the runtime sprite rects read out of the real game [L PCSX2 EE snapshots
//! 2026-10-18, 1-human Arena + Team Arena on 07000023 — docs/systems/mp-hud-data.md §8]. Each
//! test carries a sister assertion that fails on the wrong rule.

use nf_game::drone::Rng;
use nf_game::mp::hud::{status_line, MpHudSnapshot, MpScorePane, StatusPart, SCORE_ICON_TEX};
use nf_game::mp::{scenario, MpEndStatus, MpMatch, MpSettings};

fn make(sc: u32) -> MpMatch {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.scenario = sc;
    s.gametype_mask = sc;
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.time_limit = 600;
    let mut m = MpMatch::init(s);
    m.start(&mut Rng::new(1));
    m
}

fn pane(m: &MpMatch, viewer: usize) -> MpScorePane {
    let snap = MpHudSnapshot::build(m, viewer, true);
    MpScorePane::build(&snap, m.settings.team_game, m.settings.bit30, m.settings.players[viewer].team)
}

/// Arena (FFA): the text "\n<kills>\n<deaths>" at (40,13), the star hidden, the gun at (16,17)
/// and the skull at (16,35) — exactly the PCSX2 snapshot's pane-13 sprites [L: text (40,13)
/// "\n0\n0", sprite 3 layer 0xff, sprite 4 (16,17), sprite 5 (16,35)]. Sister: Team Arena moves
/// the text to y 30 and shows the star at (16,16) with the team line first [L: (40,30)
/// "0 - 0\n0\n0", sprites 3/4/5 at y 16/34/52].
#[test]
fn the_pane_layout_matches_the_game() {
    let mut rng = Rng::new(3);
    let mut m = make(scenario::DEATHMATCH);
    m.player_killed(1, Some(0), false, false, &mut rng);
    let p = pane(&m, 0);
    assert_eq!((p.text_x, p.text_y), (40.0, 13.0));
    assert_eq!(p.text, "\n1\n0", "FFA: the blank team line, kills, deaths [M 0x189730 0x30cc18]");
    assert_eq!(p.outline, 0x0000_00ff, "black colour2 [M 0x19f0a0:60-63]");
    let ys: Vec<(u32, f32, f32)> = p.icons.iter().map(|i| (i.tex, i.x, i.y)).collect();
    assert_eq!(ys, vec![(SCORE_ICON_TEX[1], 16.0, 17.0), (SCORE_ICON_TEX[2], 16.0, 35.0)]);

    let mut t = make(scenario::TEAM_DEATHMATCH);
    t.settings.players[1].team = 1;
    t.player_killed(1, Some(0), false, false, &mut rng);
    let p = pane(&t, 0);
    assert_eq!((p.text_x, p.text_y), (40.0, 30.0), "team games add 0x11 [M 0x19ed10:114]");
    assert_eq!(p.text, "1 - 0\n1\n0", "own team first, then the player string [M 0x18b338, 0x30cc10]");
    assert_eq!(p.outline, 0xff00_00ff, "team 0's red colour2 [M 0x19f0a0:53-55; L the red-ringed capture]");
    let ys: Vec<f32> = p.icons.iter().map(|i| i.y).collect();
    assert_eq!(ys, vec![16.0, 34.0, 52.0]);
    // The other team's viewer reads its own team first, blue.
    let p1 = pane(&t, 1);
    assert_eq!(p1.text.lines().next(), Some("0 - 1"));
    assert_eq!(p1.outline, 0x0000_ffff);
}

/// Assassin is FFA but keeps the star line (0x400 is excluded from the hide) and prints three
/// player lines (score, kills, deaths) [M 0x19ed10:108-118, 0x189730:13-19]; the assassin /
/// target icons sit at y 384 / 368 [M 0x19ed10:61-66]. Sister: Arena's two-line form.
#[test]
fn assassin_keeps_the_star_and_three_lines() {
    let mut m = make(scenario::ASSASSIN);
    m.assassin = Some(0);
    m.assassin_target = Some(1);
    let p = pane(&m, 0);
    assert_eq!(p.text_y, 30.0);
    assert_eq!(p.text.lines().count(), 3);
    assert!(p.icons.iter().any(|i| i.sprite == 1 && i.tex == 0x0300_0180 && i.y == 384.0), "{:?}", p.icons);
    assert!(!p.icons.iter().any(|i| i.sprite == 2), "slot 0 is not the target");
    let p1 = pane(&m, 1);
    assert!(p1.icons.iter().any(|i| i.sprite == 2 && i.tex == 0x0300_0189 && i.y == 368.0));
    assert_eq!(pane(&make(scenario::DEATHMATCH), 0).text.lines().count(), 3, "\"\\n%d\\n%d\" = 3 lines, the first blank");
}

/// A kill queues the message to the killer with the victim's slot; a suicide queues nothing
/// [M 0x188f70:100-107 — only when the last shooter is not the victim]. Sister: the victim gets
/// no message.
#[test]
fn a_kill_queues_the_killers_message() {
    let mut rng = Rng::new(5);
    let mut m = make(scenario::DEATHMATCH);
    m.player_killed(1, Some(0), false, false, &mut rng);
    assert_eq!(m.kill_msgs, vec![(0, 1)]);
    m.player_killed(0, Some(0), false, false, &mut rng);
    assert_eq!(m.kill_msgs, vec![(0, 1)], "a suicide posts nothing");
    assert!(!m.kill_msgs.iter().any(|&(to, _)| to == 1));
}

/// The status line: nothing while playing; Arena's single winner prints "Game Over :" + the
/// codename + " " + "Won" (0x2000024, name, 0x2000027 [M 0x1848d0:62-70]); the lives scenario
/// 0x10 uses "Player %s Won" 0x2000022 [M :21-41]; a time-up outside 0x10 prints 0x2000028 +
/// 0x1c9 [M 0x184bd0:42-50]. Sister: a draw is 0x2000024 + 0x2000023.
#[test]
fn the_status_line_follows_the_sort() {
    let mut m = make(scenario::DEATHMATCH);
    assert!(status_line(&m).is_empty());
    m.game.state = 3;
    m.game.end_status = MpEndStatus::Won("Bond".into());
    assert_eq!(status_line(&m), vec![StatusPart::Label(0x0200_0024), StatusPart::Text("Bond".into()), StatusPart::Text(" ".into()), StatusPart::Label(0x0200_0027)]);
    m.game.end_status = MpEndStatus::Draw;
    assert_eq!(status_line(&m), vec![StatusPart::Label(0x0200_0024), StatusPart::Label(0x0200_0023)]);
    m.game.end_status = MpEndStatus::None;
    assert_eq!(status_line(&m), vec![StatusPart::Label(0x0200_0028), StatusPart::Label(0x1c9)]);
    m.settings.scenario = scenario::DEATH_LIMIT_FFA;
    m.game.end_status = MpEndStatus::Won("Bond".into());
    assert_eq!(status_line(&m), vec![StatusPart::Format(0x0200_0022, "Bond".into())]);
}
