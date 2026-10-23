//! mp-hud-16 — the in-match MP HUD data with the port's 16 participants (the port widened the
//! match to 16 slots [S mp-16-slots]; the game's own layout is 4 humans + 4 bots [M `MP_Start`
//! 0x183a50:116-160]). Numbers and ids only, headless, no ISO. Each test carries a sister
//! assertion that fails if the data model regressed (rows dropped past 8, the viewer cut from the
//! list, blips dropped past the game's 24 cap).

use nf_game::drone::Rng;
use nf_game::mp::hud::{MpHudSnapshot, RadarKind, GAME_SLOTS, RADAR_BLIP_CAP};
use nf_game::mp::{scenario, MpMatch, MpSettings, MP_MAX_SLOTS};

/// A match with `humans` humans then `bots` bots (slots human_count.. — the port's layout), every
/// slot placed on a distinct spot, kills = the slot number (so slot 15 leads, slot 0 is last).
fn make(sc: u32, humans: u32, bots: u32) -> MpMatch {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.scenario = sc;
    s.frag_limit = 99;
    s.time_limit = 600;
    s.human_count = humans;
    s.player_count = humans as u16;
    s.total_players = humans;
    s.bot_count = bots;
    let mut m = MpMatch::init(s);
    m.start(&mut Rng::new(1));
    for slot in 0..(humans + bots) as usize {
        m.settings.players[slot].team = (slot % 2) as u32;
        m.set_slot(slot, [5.0 * slot as f32, 0.0, 3.0]);
        m.game.slots[slot].kills = slot as i32;
    }
    m
}

/// 16 participants (2 humans + 14 bots): one row per participant in the slot walk, humans first;
/// the layout is compact past the game's 8 [S]; a full ≤ 8 match is not.
#[test]
fn sixteen_participants_give_sixteen_rows() {
    let m = make(scenario::DEATHMATCH, 2, 14);
    let snap = MpHudSnapshot::build(&m, 0, true);
    assert_eq!(snap.participants(), MP_MAX_SLOTS, "every participant has a row");
    let slots: Vec<usize> = snap.rows.iter().map(|r| r.slot).collect();
    assert_eq!(slots, (0..16).collect::<Vec<_>>(), "humans 0..2 then bots 2..16 in slot order");
    assert!(snap.compact(), "16 > the game's 8 → compact layout");
    // Sister: the game-sized match keeps the full-size layout.
    let eight = MpHudSnapshot::build(&make(scenario::DEATHMATCH, 4, 4), 0, true);
    assert_eq!(eight.participants(), GAME_SLOTS);
    assert!(!eight.compact(), "8 participants keep the game-sized rows");
    // Sister: 9 is the first compact count.
    assert!(MpHudSnapshot::build(&make(scenario::DEATHMATCH, 1, 8), 0, true).compact());
}

/// The board ranks by the display score (Arena = kills [M 0x203af8:44-54]) and lists everyone
/// when the rows fit.
#[test]
fn the_board_lists_all_sixteen_in_rank_order() {
    let m = make(scenario::DEATHMATCH, 2, 14);
    let snap = MpHudSnapshot::build(&m, 0, true);
    let b = snap.board(16);
    assert_eq!(b.rows.len(), 16);
    assert_eq!(b.hidden, 0);
    let order: Vec<usize> = b.rows.iter().map(|r| r.slot).collect();
    assert_eq!(order, (0..16).rev().collect::<Vec<_>>(), "kills = slot, so slot 15 leads");
    assert_eq!(b.rows.iter().map(|r| r.rank).collect::<Vec<_>>(), (0..16).collect::<Vec<_>>());
    assert_eq!(b.rows.iter().filter(|r| r.viewer).count(), 1, "exactly one viewer row");
    assert!(b.rows[15].viewer && b.rows[15].slot == 0, "the viewer (slot 0, 0 kills) is last");
}

/// A cut list (fewer rows than participants) still shows the viewer, with their true rank, in
/// the last visible row; the hidden count covers the rest. Sister: a viewer inside the cut keeps
/// the plain top-N.
#[test]
fn a_cut_board_always_shows_the_viewer() {
    let m = make(scenario::DEATHMATCH, 2, 14);
    // Viewer slot 0 is ranked 16th (index 15).
    let b = MpHudSnapshot::build(&m, 0, true).board(9);
    assert_eq!(b.rows.len(), 9);
    assert_eq!(b.hidden, 7);
    assert!(b.rows.iter().any(|r| r.viewer && r.slot == 0), "the viewer is listed");
    let last = b.rows[8];
    assert_eq!((last.slot, last.rank), (0, 15), "the viewer's own rank, not the cut position");
    assert_eq!(b.rows[..8].iter().map(|r| r.slot).collect::<Vec<_>>(), (8..16).rev().collect::<Vec<_>>(), "the top 8 stay");
    // Sister: viewer slot 15 is the leader — the top-N is untouched and slot 7 (rank 8) is shown.
    let b2 = MpHudSnapshot::build(&m, 15, true).board(9);
    assert!(b2.rows[0].viewer);
    assert_eq!(b2.rows[8].slot, 7);
    assert_eq!(b2.hidden, 7);
}

/// Equal scores share a rank (the debriefing's rank walk [M 0x212198:131-143]) and keep slot
/// order (the bubble's no-swap case).
#[test]
fn equal_scores_share_a_rank() {
    let mut m = make(scenario::DEATHMATCH, 2, 14);
    for s in 0..16 {
        m.game.slots[s].kills = if s < 4 { 5 } else { 1 };
    }
    let b = MpHudSnapshot::build(&m, 0, true).board(16);
    assert_eq!(b.rows[..4].iter().map(|r| (r.slot, r.rank)).collect::<Vec<_>>(), vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
    assert_eq!(b.rows[4].rank, 4, "the next score's rank is its position");
    assert!(b.rows[4..].iter().all(|r| r.rank == 4));
}

/// The radar carries one blip per other live participant — 15 with 16 — and every slot past the
/// game's 8 is present (the walk follows the human count, not the fixed base 4 [S mp-16-slots]).
/// Sisters: the viewer is never on it [M 0x189840:37-40]; a dead slot drops [port `MpSlot::dead`].
#[test]
fn the_radar_has_a_blip_for_every_other_participant() {
    let mut m = make(scenario::DEATHMATCH, 2, 14);
    let snap = MpHudSnapshot::build(&m, 0, true);
    let players: Vec<usize> = snap.radar.iter().filter(|o| o.kind == RadarKind::Player.as_u16()).filter_map(|o| o.slot).collect();
    assert_eq!(players.len(), 15);
    assert_eq!(players, (1..16).collect::<Vec<_>>());
    assert!(!players.contains(&0), "the viewer has no blip");
    assert!(snap.radar.len() <= RADAR_BLIP_CAP);
    m.game.slots[12].dead = true;
    let snap = MpHudSnapshot::build(&m, 0, true);
    assert_eq!(snap.radar.iter().filter(|o| o.kind == RadarKind::Player.as_u16()).count(), 14);
    assert!(snap.radar.iter().all(|o| o.slot != Some(12)));
    // The viewer is a bot-range slot too (a late joiner's slot 9): its own blip is skipped.
    let snap9 = MpHudSnapshot::build(&make(scenario::DEATHMATCH, 10, 6), 9, true);
    assert_eq!(snap9.radar.len(), 15);
    assert!(snap9.radar.iter().all(|o| o.slot != Some(9)));
}

/// The worst radar list at 16 participants: Uplink with the full 8 terminals — 15 player blips +
/// 8 uplinks = 23, inside the game's 24-blip cap [M 0x19f6e8:108,153], so no blip is dropped.
#[test]
fn sixteen_players_plus_eight_uplinks_fit_the_blip_cap() {
    let mut m = make(scenario::UPLINK, 2, 14);
    for i in 0..8 {
        m.register_mp_object([100.0 + 10.0 * i as f32, 0.0, 0.0], [0.0; 3], 2, [2, 0, 0, 0, 0, 0, 0], 0x2000 + i);
    }
    m.create_mode_objects(&mut Rng::new(3));
    let snap = MpHudSnapshot::build(&m, 0, true);
    let up = snap.radar.iter().filter(|o| o.kind == RadarKind::Uplink.as_u16()).count();
    assert_eq!(up, m.objects_rt.uplinks.len());
    assert_eq!(up, 8, "all 8 registered terminals are live");
    assert_eq!(snap.radar.len(), 15 + up);
    assert!(snap.radar.len() <= RADAR_BLIP_CAP, "{} blips, the cap is {RADAR_BLIP_CAP}", snap.radar.len());
}

/// The team scores and the clock are participant-count independent; the team tint follows each
/// slot's team past slot 8 (team game, `radar_team_color` [M 0x189840:58-62]).
#[test]
fn team_tints_reach_the_slots_past_eight() {
    let mut m = make(scenario::TEAM_DEATHMATCH, 2, 14);
    m.settings.team_game = 1;
    let snap = MpHudSnapshot::build(&m, 0, true);
    for o in snap.radar.iter().filter(|o| o.kind == RadarKind::Player.as_u16()) {
        let s = o.slot.unwrap();
        let want = if s % 2 == 0 { 0xd22d_35ff } else { 0x2d61_d2ff };
        assert_eq!(o.argb, want, "slot {s} team tint");
    }
    assert_eq!(snap.timer_string(), "10:00");
}
