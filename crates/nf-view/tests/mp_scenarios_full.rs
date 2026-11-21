//! mp-scenarios-full: the scenario end results that the two-window play-outs found differing
//! from the game, headless (no ISO needed).

use nf_game::drone::Rng;
use nf_game::mp::debrief::MpDebriefResult;
use nf_game::mp::{scenario, MpMatch, MpSettings};

/// One human (slot 0) and `bots` bots on two registered spawn points, started the way the
/// live load starts a match (`start`, then the bots).
fn match_with_bots(scen: u32, bots: u32, frag_limit: i32) -> MpMatch {
    let mut s = MpSettings::deathmatch(0x0700_0024);
    s.scenario = scen;
    s.team_game = (scen >> 29) & 1;
    s.frag_limit = frag_limit;
    s.human_count = 1;
    s.player_count = 1;
    s.bot_count = bots;
    s.total_players = 1 + bots;
    s.players[0].codename = "Player 1".into();
    let mut m = MpMatch::init(s);
    let world = nf_game::world::CollisionWorld::from_triangles(Vec::new());
    m.register_spawn_point([10.0, 1.0, 0.0], [0.0, 0.0, 0.0], 0, &world);
    m.register_spawn_point([100.0, 1.0, 0.0], [0.0, 1.0, 0.0], 0, &world);
    let mut rng = Rng::new(0x5ce0_0001);
    m.start(&mut rng);
    m.set_slot(0, [30.0, 1.6, 0.0]);
    m.create_bots(&mut rng);
    m
}

/// A bot that wins a free-for-all match is named on the result line. The boot names the bot
/// records `sprintf("%s %d", "Bot", i − 3)` for records 4..7 [M bootup 0x1c7e78:50-61; D "%s %d"
/// @0x30ce18, "Bot" @0x30ce20]; the debriefing prints `MPSettings + top·0x30` into "%s wins the
/// match." [M P_MPDEBRIEFING 0x212198:227-228] and `MP_SortOutWhoWon` prints the same record
/// [M 0x1848d0:73-76]. The port's bot records had no name: the line read " wins the match."
#[test]
fn a_bot_winner_is_named_on_the_result_line() {
    let mut m = match_with_bots(scenario::DEATHMATCH, 2, 2);
    assert_eq!(m.settings.players[1].codename, "Bot 1", "the first bot record's boot name");
    assert_eq!(m.settings.players[2].codename, "Bot 2");
    // The second bot kills the human twice: +1.0 each [M MP_PlayerKilled 0x188f70:167-184].
    let mut rng = Rng::new(7);
    m.player_killed(0, Some(2), false, false, &mut rng);
    m.player_killed(0, Some(2), false, false, &mut rng);
    for _ in 0..3 { m.update(1.0 / 60.0, false, &mut rng); }
    assert_eq!(m.debriefing().result, MpDebriefResult::Wins("Bot 2".into()));
    assert_eq!(m.game.end_status, nf_game::mp::MpEndStatus::Won("Bot 2".into()));
}

/// A human's own codename stays (the fill touches only an empty bot record).
#[test]
fn a_human_winner_keeps_the_codename() {
    let mut m = match_with_bots(scenario::DEATHMATCH, 1, 1);
    let mut rng = Rng::new(7);
    m.player_killed(1, Some(0), false, false, &mut rng);
    for _ in 0..3 { m.update(1.0 / 60.0, false, &mut rng); }
    assert_eq!(m.debriefing().result, MpDebriefResult::Wins("Player 1".into()));
}

/// Top Agent: every slot's score is the lives copy, starting at the lives limit
/// (`MPGame + s·0x30 + 0x18 = (float)(int)MPSettings+0x19C` [M MP_Init 0x183438:56-62]) and
/// dropping 1.0 per death [M MP_PlayerKilled 0x188f70:96-98]; the debriefing's points column
/// reads it [M `Menu_GetMPScore` 0x203af8: 0x10 → (int)+0x18]. The port started it at 0, so a player
/// with one life left of three showed -2 points.
#[test]
fn top_agent_points_are_the_lives_left() {
    let mut m = match_with_bots(scenario::DEATH_LIMIT_FFA, 2, 3);
    for s in 0..3 { assert_eq!(m.game.slots[s].score, 3.0, "slot {s} starts with the lives limit"); }
    let mut rng = Rng::new(7);
    m.player_killed(0, Some(1), false, false, &mut rng);
    m.player_killed(0, Some(2), false, false, &mut rng);
    assert_eq!(m.game.slots[0].score, 1.0, "two deaths of three lives leave one");
    let b = m.debriefing();
    let row0 = b.rows.iter().find(|r| r.slot == 0).unwrap();
    assert_eq!((row0.points, row0.total, row0.deaths), (1, 1, 2));
    let row1 = b.rows.iter().find(|r| r.slot == 1).unwrap();
    assert_eq!((row1.points, row1.total, row1.victories), (3, 3, 1), "a kill adds nothing in 0x10");
}
