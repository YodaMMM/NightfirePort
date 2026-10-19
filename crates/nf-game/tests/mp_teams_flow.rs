//! mp-teams: the lobby's team picks flow into MpSettings → the game's team rules. The picks are
//! what `nf_net::teams::TeamSelection::apply` writes into the settings records (the wire's player
//! tuples; nf-view's `match_to_mp` converts them into `MpSettings.players[slot].team`); this test
//! builds the same records and runs the ported rules, pinning that the flow uses the teams exactly
//! as the game's team modes do. Rules and addresses: docs/systems/mp-match-rules.md, net-teams.md.

use nf_game::drone::Rng;
use nf_game::mp::{scenario, MpMatch, MpSettings};

const DT: f32 = 1.0 / 60.0;

/// The default picks of `TeamSelection` (Phoenix/MI6 alternating) written into the records — the
/// same write `apply` does on the wire side [S net-teams.md; the game's storage is
/// `MPSettings + slot·0x30 + 0x20` [M 0x1857c0]].
fn join_picked(m: &mut MpMatch, slot: usize, team: u32) {
    m.settings.players[slot].team = team;
    m.set_slot(slot, [10.0 + 10.0 * slot as f32, 0.0, 0.0]);
}

fn team_match() -> MpMatch {
    let mut s = MpSettings::deathmatch(0x07000011);
    s.scenario = scenario::TEAM_DEATHMATCH;
    s.friendly_fire = 0; // the couch default: same-team bullet hits block [M 0x1857c0:43-57]
    let mut m = MpMatch::init(s);
    m.start(&mut Rng::new(1));
    m
}

fn tick(m: &mut MpMatch, frames: u32, rng: &mut Rng) {
    for _ in 0..frames {
        m.update_objects(DT, &[false; 8], rng);
        m.update(DT, false, rng);
    }
}

/// A cross-team kill moves the KILLER's team score and credits the killer personally; a same-team
/// kill pays the VICTIM's team and credits nobody [M MP_PlayerKilled 0x188f70:157-163, 246-274 —
/// scenario 0x20000002's team-score block]. The picks here are the lobby's defaults: slot 0
/// Phoenix, slot 1 MI6.
#[test]
fn the_picked_teams_score_per_the_game() {
    let mut rng = Rng::new(3);
    let mut m = team_match();
    join_picked(&mut m, 0, 0); // Phoenix pick
    join_picked(&mut m, 1, 1); // MI6 pick
    // Phoenix kills MI6: the enemy kill → the attacker's (Phoenix, team 0) score += |Δ| where Δ is
    // the KILLER's personal delta (1 for an enemy kill, −1 suicide, 2 the bonus case) → +0x180
    // (team 0) [M 0x188f70:166-178, 266-271] (was "the victim's delta" — daily audit 2026-10-19).
    m.player_killed(1, Some(0), false, false, &mut rng);
    assert_eq!(m.game.team_scores[0], 1.0, "the Phoenix pick's team scores the enemy kill");
    assert_eq!(m.game.team_scores[1], 0.0);
    assert_eq!(m.game.slots[0].score, 1.0, "the personal kill lands too (bit30 = 0) [M 0x188f70:179-184]");
    // MI6 kills Phoenix: team 1 catches up.
    m.player_killed(0, Some(1), false, false, &mut rng);
    assert_eq!(m.game.team_scores[1], 1.0);
    assert_eq!(m.game.team_scores[0], 1.0);
    tick(&mut m, 1, &mut rng);
}

/// The lobby can put both picks on one side (the gate allows it: both teams need only be non-empty
/// ACROSS the participants — a 1-human test lobby pairs with bots) — the rules then read them as
/// TEAM MATES: no personal credit for the kill, the victim's team pays, and with friendly_fire = 0
/// the same-team bullet hit is blocked [M 0x188f70:157-163 + 0x1857c0 disasm 0x185864-0x1858a0].
#[test]
fn same_picked_side_means_teammate_rules() {
    let mut rng = Rng::new(5);
    let mut m = team_match();
    join_picked(&mut m, 0, 0);
    join_picked(&mut m, 1, 0); // both Phoenix — the lobby allows it (bots/backfill fill MI6)
    m.player_killed(1, Some(0), false, false, &mut rng);
    // The same-team gate covers kills/streak ONLY [M 0x188f70:157-163]; the personal score add sits
    // OUTSIDE it — bit30==0 ∧ scenario ∈ {1, 0x20000002, 0x40000800, 0x60001000} [M :179-184] — so
    // the team kill still pays the killer +1.0 personally while the victim's team pays |Δ|.
    assert_eq!(m.game.slots[0].score, 1.0, "the personal score add is outside the same-team gate [M 0x188f70:179-184]");
    assert_eq!(m.game.slots[0].kills, 0, "a team kill adds no kill [M 0x188f70:157-163]");
    assert_eq!(m.game.team_scores[0], -1.0, "the victim's team pays |Δ| [M 0x188f70:262-274]");
    // The friendly-fire gate: same team + friendly_fire 0 → blocked [M 0x1857c0].
    assert!(!m.register_bullet_hit(1, 0), "a Phoenix bullet does not hurt a Phoenix pick");
    // Sister: with friendly fire on, the same hit is allowed and warns [M 0x1857c0:46-56].
    m.settings.friendly_fire = 1;
    assert!(m.register_bullet_hit(1, 0), "friendly_fire 1 allows the same-team hit");
    assert_eq!(m.game.slots[1].timers[0], 3 * 60, "the victim's warn timer arms for 3 s [M 0x1857c0:52-56]");
}

/// The spawns split per the picked team [M MP_RegisterSpawnPoint 0x184f20:54-67 /
/// MP_GetSpawnPoint 0x185098:85-98: team games split the 64-point table [0,0x20)/[0x20,0x40)]:
/// every pick-Phoenix draw lands in team 0's half, every pick-MI6 draw in team 1's half.
#[test]
fn the_spawn_table_splits_per_the_picked_team() {
    let mut rng = Rng::new(9);
    let mut m = team_match();
    join_picked(&mut m, 0, 0);
    join_picked(&mut m, 1, 1);
    let world = nf_game::world::CollisionWorld::from_triangles(vec![]);
    for i in 0..64u32 {
        let team = if i < 0x20 { 0u16 } else { 1u16 };
        m.register_spawn_point([i as f32, 0.0, 0.0], [0.0, 0.0, 1.0], team, &world);
    }
    for _ in 0..16 {
        let t0 = m.get_spawn_point(m.settings.players[0].team as u16, 0, &mut rng);
        assert!(t0 < 0x20, "a Phoenix pick spawns in team 0's half, got {t0}");
        let t1 = m.get_spawn_point(m.settings.players[1].team as u16, 1, &mut rng);
        assert!(t1 >= 0x20 && t1 < 0x40, "an MI6 pick spawns in team 1's half, got {t1}");
    }
}

/// Sister: the picks survive the match's slot records — the HUD/debrief read the same
/// `players[slot].team` (the score rows and the team result line) [M mp-hud-data.md,
/// mp-debriefing.md; the game reads `MPSettings + slot·0x30 + 0x20` everywhere].
#[test]
fn the_picks_reach_the_hud_and_debrief_view() {
    let mut rng = Rng::new(11);
    let mut m = team_match();
    join_picked(&mut m, 0, 1); // the lobby's pick: slot 0 on MI6
    join_picked(&mut m, 1, 1);
    assert_eq!(m.settings.players[0].team, 1);
    // A cross-team kill cannot exist here (both MI6) — the FFA-style personal score is gated by
    // the same-team rule instead [M 0x188f70:157-163].
    m.player_killed(1, Some(0), false, false, &mut rng);
    assert_eq!(m.game.slots[0].score, 1.0, "the personal add is outside the same-team gate [M 0x188f70:179-184]");
    tick(&mut m, 1, &mut rng);
    assert_eq!(m.game.team_scores[1], -1.0, "the shared MI6 pick's team pays together");
    assert_eq!(m.game.team_scores[0], 0.0, "Phoenix (no members) is untouched");
}
