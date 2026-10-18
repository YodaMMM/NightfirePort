//! mp-teams: the team-selection state for the online lobby's team modes — the module the lobby
//! page (700v1, unmerged) calls. No drawing here: the page shows a team column only when
//! [`scenario_has_teams`] says so, reads/writes picks through this module, and applies them into
//! the match settings so the ported rules use them exactly as the game's team modes do.
//!
//! Game evidence:
//! - **Which modes are team-based**: the scenario word's bit 29 (0x2000_0000). `MP_Init` 0x183438
//!   derives `MPSettings+0x18C = (scenario >> 29) & 1` (the team-game flag the rules read) and
//!   `+0x190 = (scenario >> 30) & 1` [M 0x183438:41-42]; the couch menus test the bit raw
//!   (`scenario & 0x20000000`) to offer the team pick [M C_RBMPSETUP 0x20de58:62,130-139,
//!   C_SBMPOPTIONS 0x20e818:84, P_MPDEBRIEFING 0x20ec00:177]. Team rows of the scenario wheel
//!   [L `MP_SCENARIO_ROWS` in mp_menu_lists_live.rs, the mp_scenario table @0x2df508]: Team Arena
//!   0x2000_0002, CTF 0x2000_0004, Uplink 0x6000_0008, Demolition 0x2000_0040, Protection
//!   0x2000_0080, Industrial Espionage 0x2000_0100, GoldenEye Strike 0x2000_0200, Team KOTH
//!   0x6000_1000; the FFA rows: Quick Game/Deathmatch 0x1, Top Agent 0x10, Assassin 0x400,
//!   KOTH 0x4000_0800. (Assassin is an FFA that reuses the team word as its role marker and still
//!   splits the spawn table [M 0x184f20:54-67] — the lobby shows NO team column for it: its
//!   "teams" are assigned by the rules, not picked.)
//! - **Teams are 2, named Phoenix (0) and MI6 (1)**: the team-pick list is items 0=Phoenix
//!   (label 0x1c7), 1=MI6 (0x1c8) [M 0x20de58:130-139]; the HUD/radar tints are team0 red
//!   0xd22d35ff / team1 blue 0x2d61d2ff [M 0x20ec00:195-207; L mp-hud-data.md].
//! - **Storage**: `MPSettings + slot·0x30 + 0x20` (u32 team) per slot [M 0x1857c0 and every
//!   rules read]; humans pick on the Join page into `mpjoin+4`, the confirm copies
//!   `mpjoin+4 → MPSettings+0x20+i·0x30` [M P_MPCONFIRM 0x20ec00:109-144]; bots carry the team
//!   byte in their record (+0x0F) and `MP_Start` copies it into the bot slots [M 0x183a50:140-141].
//!   There is NO auto-balance in the game — the only team rule at start is validation [M
//!   C_SBMPOPTIONS 0x20e818:50-130]; the port's optional auto-balance below is [S] unverified.
//! - **Start gates** [M 0x20e818:50-130]: a team game rejects when either team has 0 members
//!   (the "%s team" error with the 0x1c7/0x1c8 labels); an FFA rejects more than ONE MI6-side
//!   participant (0x1000311); any game needs ≥ 2 participants total.
//! - **What the rules do with the teams** (already ported in nf-game; the flow test pins it
//!   through this module's apply): kill credit only when FFA or teams differ, TEAM_DEATHMATCH
//!   team-score effects [M MP_PlayerKilled 0x188f70:157-163,246-274]; the friendly-fire gate
//!   blocks same-team bullet hits unless `friendly_fire != 0` [M MP_RegisterBulletHit 0x1857c0,
//!   disasm 0x185864-0x1858a0]; the spawn table splits [0,0x20)/[0x20,0x40) per team in team
//!   games (and Assassin) [M 0x184f20:54-67, 0x185098:85-98]; every objective update scores
//!   `MPGame + team·4 + 0x180/0x184` [M 0x1861a8 CTF, 0x186d98 Uplink, 0x186ae0 KOTH, 0x1870a0
//!   Demo/Prot].
//!
//! [S] unverified items are marked inline; online play is a port feature (the PS2 has none [D —
//! no netcode in ACTION.ELF]), so the lobby flow itself is port design.

/// The scenario-word bit that makes a mode team-based (`MPSettings+0x18C`'s source) [M 0x183438:41].
pub const SCENARIO_TEAM_BIT: u32 = 0x2000_0000;

/// Team 0 = Phoenix [M 0x20de58:130-139 (item 0), 0x20ec00:195-207 (the red tint)].
pub const TEAM_PHOENIX: u32 = 0;
/// Team 1 = MI6 [M 0x20de58:130-139 (item 1), 0x20ec00:195-207 (the blue tint)].
pub const TEAM_MI6: u32 = 1;

/// The team-pick rejections (the state is left unchanged in every case).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamError {
    /// Slots 4..8 are the bots' records — they don't pick [M 0x183a50:140-141].
    NotAPickingSlot,
    /// Only Phoenix (0) and MI6 (1) exist [M 0x20de58:130-139].
    BadTeam,
}

/// The team-name label ids (`Txt_BindLabel` on the couch team pick and the debrief's team lines)
/// [M 0x20de58:130-139; mp-debriefing.md's TEAM_PHOENIX_LABEL/TEAM_MI6_LABEL].
pub const TEAM_LABELS: [u32; 2] = [0x1c7, 0x1c8];

/// Whether the lobby shows a team column for this scenario [M 0x183438:41-42, 0x20de58:62].
/// Assassin (0x400) deliberately returns false — its team word is a rules-internal role marker
/// [M 0x188f70's assassin bypass, 0x1832b8], not a picked team.
pub fn scenario_has_teams(scenario: u32) -> bool {
    scenario & SCENARIO_TEAM_BIT != 0
}

/// The name label of a team, for the page's `Txt_BindLabel` [M 0x20de58:130-139].
pub fn team_label(team: u32) -> u32 {
    TEAM_LABELS[(team == TEAM_MI6) as usize]
}

/// The humans' team picks (slots 0..4 — bots' teams ride their bot records [M 0x183a50:140-141],
/// so they never enter this state). The lobby edits it; `apply` writes it into the wire settings.
#[derive(Clone, Debug, PartialEq)]
pub struct TeamSelection {
    /// Per human slot: `TEAM_PHOENIX` or `TEAM_MI6`.
    picks: [u32; 4],
}

impl Default for TeamSelection {
    fn default() -> Self {
        // [S] unverified (port default): a team game starts Phoenix/MI6 alternating so the game's
        // both-teams-non-empty gate [M 0x20e818:84-103] passes for 2+ humans with no edits; an FFA
        // default of all-Phoenix keeps the ≤1-MI6 gate [M 0x20e818:85-89] trivially true. The
        // couch flow's fixed Quick-Start teams are the closest game shape (mp-menus.md §Quick Start).
        TeamSelection { picks: [TEAM_PHOENIX, TEAM_MI6, TEAM_PHOENIX, TEAM_MI6] }
    }
}

impl TeamSelection {
    /// The pick of human slot `slot` (0..4).
    pub fn team_of(&self, slot: usize) -> u32 {
        self.picks[slot.min(3)]
    }

    /// Set a pick. Returns `Err` (leaving the state unchanged) for a slot ≥ 4 (bots don't pick
    /// [M 0x183a50:140-141]) or a team id outside {0, 1} [M the two-item pick list].
    pub fn set_team(&mut self, slot: usize, team: u32) -> Result<(), TeamError> {
        if slot >= 4 {
            return Err(TeamError::NotAPickingSlot);
        }
        if team != TEAM_PHOENIX && team != TEAM_MI6 {
            return Err(TeamError::BadTeam);
        }
        self.picks[slot] = team;
        Ok(())
    }

    /// Toggle one slot's pick (the lobby row's tap action).
    pub fn toggle(&mut self, slot: usize) -> Result<(), TeamError> {
        let t = self.team_of(slot);
        self.set_team(slot, if t == TEAM_PHOENIX { TEAM_MI6 } else { TEAM_PHOENIX })
    }

    /// [S] unverified (port-only): optional auto-balance — the game has none [M 0x20e818 only
    /// validates]. Deterministic: fill Phoenix/MI6 alternately over `humans` slots. The lobby can
    /// call this on a "balance teams" control before Start.
    pub fn auto_balance(&mut self, humans: usize) {
        let humans = humans.min(4);
        for i in 0..humans {
            self.picks[i] = if i % 2 == 0 { TEAM_PHOENIX } else { TEAM_MI6 };
        }
    }

    /// Write the picks into the wire settings' human records (slots 0..`humans`). Bot slots are
    /// NOT touched here — their teams arrive from the bot records at `MP_Start` [M 0x183a50:140-141]
    /// and the lobby's bot rows carry them separately. The app-side converter then flows them into
    /// `MpSettings.players[slot].team` (nf-view mpnet.rs `match_to_mp`), where the ported rules
    /// read them [M the rules cites at the top of this file].
    pub fn apply(&self, s: &mut MatchSettings, humans: usize) {
        for (i, p) in s.players.iter_mut().enumerate().take(humans.min(4)) {
            p.0 = self.picks[i];
        }
    }

    /// The effective team of every slot: the humans' picks plus the bots' record teams (slot
    /// `4+i` = `bots[i]`, `TEAM_PHOENIX`/`TEAM_MI6`). This is the array the start gate scores —
    /// the same composition `MP_Start` builds from `mpjoin+4` and the bot records [M 0x20ec00:109-144,
    /// 0x183a50:140-141].
    pub fn effective_teams(&self, s: &MatchSettings, humans: usize) -> [u32; 8] {
        let mut t = [TEAM_PHOENIX; 8];
        for (i, p) in s.players.iter().enumerate() {
            t[i] = p.0;
        }
        for (i, pick) in self.picks.iter().enumerate().take(humans.min(4)) {
            t[i] = *pick;
        }
        t
    }
}

/// The start gate's verdicts, in the game's own error order [M C_SBMPOPTIONS 0x20e818:50-130].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartError {
    /// Fewer than 2 participants [M 0x20e818:104-111].
    TooFewPlayers,
    /// A team game with an empty side — the "%s team" error, label 0x1c7 (Phoenix) or 0x1c8 (MI6)
    /// [M 0x20e818:84-103].
    EmptyTeam(u32),
    /// An FFA with more than one MI6-side participant (error label 0x1000311) [M 0x20e818:85-89].
    TooManyMi6,
}

/// The couch Start gates over the EFFECTIVE teams (humans' picks + bots' records), ported from
/// `C_SBMPOPTIONS` 0x20e818:50-130 [M]. The participants are the humans' slots 0..`humans` plus
/// the bots' slots 4..4+`bots` — the game's slot layout [M 0x183a50:116-141: humans inited first,
/// then the bots in slots 4..].
pub fn start_check(
    scenario: u32,
    effective_teams: &[u32; 8],
    humans: usize,
    bots: usize,
) -> Result<(), StartError> {
    let total = humans + bots;
    if total < 2 {
        return Err(StartError::TooFewPlayers);
    }
    let mut participants: Vec<u32> = Vec::with_capacity(total);
    for t in effective_teams.iter().take(humans.min(4)) {
        participants.push(*t);
    }
    for t in effective_teams.iter().skip(4).take(bots.min(4)) {
        participants.push(*t);
    }
    if scenario_has_teams(scenario) {
        let (mut phoenix, mut mi6) = (0usize, 0usize);
        for t in participants {
            match t {
                TEAM_PHOENIX => phoenix += 1,
                TEAM_MI6 => mi6 += 1,
                _ => {} // Assassin-style role markers can't appear in a team scenario
            }
        }
        if phoenix == 0 {
            return Err(StartError::EmptyTeam(TEAM_PHOENIX));
        }
        if mi6 == 0 {
            return Err(StartError::EmptyTeam(TEAM_MI6));
        }
        Ok(())
    } else {
        // FFA: more than one MI6-side participant is rejected [M 0x20e818:85-89].
        let mi6 = participants.iter().filter(|&&t| t == TEAM_MI6).count();
        if mi6 > 1 {
            return Err(StartError::TooManyMi6);
        }
        Ok(())
    }
}

// ── The wire message ────────────────────────────────────────────────────────────────────────────
// A client's team edit rides the session's app-to-app DATA tunnel (`Session::send_data` →
// `Event::Data`), framed exactly like the lobby codec's one-row edit (magic "NFLE", version 1,
// `{slot, field, value}`) with the team field id — wire-compatible with 700v1's `LobbyEdit`
// EDIT_TEAM row, so whichever codec lands first decodes both [S — port design; the framing mirrors
// docs/systems/net-online-lobby.md §6].

/// The frame magic "NFLE" (the lobby row-edit frame this message reuses) [S].
pub const TEAM_EDIT_MAGIC: u32 = 0x4E46_4C45;
/// The frame version [S].
pub const TEAM_EDIT_VERSION: u8 = 1;
/// The row-edit field id for the team pick (the lobby codec's EDIT_TEAM) [S].
pub const EDIT_TEAM: u8 = 1;

/// A client→host team pick: human `slot` switches to `team`. 10 bytes on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeamEdit {
    pub slot: u8,
    pub team: u32,
}

impl TeamEdit {
    pub fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&TEAM_EDIT_MAGIC.to_le_bytes());
        out.push(TEAM_EDIT_VERSION);
        out.push(self.slot);
        out.push(EDIT_TEAM);
        out.extend_from_slice(&self.team.to_le_bytes());
    }

    /// Strict decode: `None` unless the buffer is exactly one frame with the right magic, version
    /// and field id (the lobby codec's rule — a short or trailing-garbage buffer is not a team edit).
    pub fn decode(buf: &[u8]) -> Option<TeamEdit> {
        if buf.len() != 11 {
            return None;
        }
        if u32::from_le_bytes(buf[0..4].try_into().ok()?) != TEAM_EDIT_MAGIC || buf[4] != TEAM_EDIT_VERSION || buf[6] != EDIT_TEAM {
            return None;
        }
        Some(TeamEdit { slot: buf[5], team: u32::from_le_bytes(buf[7..11].try_into().ok()?) })
    }
}

use crate::settings::MatchSettings;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_column_follows_the_scenario_bit() {
        // The wheel's team rows [L MP_SCENARIO_ROWS] — the column shows for bit 29.
        for s in [0x2000_0002u32, 0x2000_0004, 0x6000_0008, 0x2000_0040, 0x2000_0080, 0x2000_0100, 0x2000_0200, 0x6000_1000] {
            assert!(scenario_has_teams(s), "{s:#x} is a team row");
        }
        // The FFA rows.
        for s in [0x1u32, 0x10, 0x400, 0x4000_0800] {
            assert!(!scenario_has_teams(s), "{s:#x} is an FFA row");
        }
    }

    #[test]
    fn team_labels_are_phoenix_then_mi6() {
        assert_eq!(team_label(TEAM_PHOENIX), 0x1c7);
        assert_eq!(team_label(TEAM_MI6), 0x1c8);
        assert_eq!(team_label(7), 0x1c7, "anything not MI6 reads as Phoenix");
    }

    #[test]
    fn start_checks_match_the_couch_gates() {
        let mut ts = TeamSelection::default();
        let mut s = MatchSettings::default();
        // 2 humans, Phoenix/MI6 → a team game starts.
        ts.apply(&mut s, 2);
        let teams = ts.effective_teams(&s, 2);
        assert_eq!(start_check(0x2000_0004, &teams, 2, 0), Ok(()));
        // Both humans on Phoenix → the MI6 "%s team" error [M 0x20e818:84-103].
        ts.set_team(1, TEAM_PHOENIX).unwrap();
        ts.apply(&mut s, 2);
        let teams = ts.effective_teams(&s, 2);
        assert_eq!(start_check(0x2000_0004, &teams, 2, 0), Err(StartError::EmptyTeam(TEAM_MI6)));
        // FFA with two MI6-side picks → rejected [M 0x20e818:85-89]; one is fine.
        assert_eq!(start_check(0x1, &teams, 2, 0), Ok(()), "both Phoenix is a legal FFA");
        ts.set_team(0, TEAM_MI6).unwrap();
        ts.set_team(1, TEAM_MI6).unwrap();
        ts.apply(&mut s, 2);
        let teams = ts.effective_teams(&s, 2);
        assert_eq!(start_check(0x1, &teams, 2, 0), Err(StartError::TooManyMi6));
        // Fewer than 2 participants [M 0x20e818:104-111].
        assert_eq!(start_check(0x1, &teams, 1, 0), Err(StartError::TooFewPlayers));
        // Bots count: a Phoenix/MI6 human pair + 2 Phoenix bots keeps the team game legal.
        let mut teams8 = ts.effective_teams(&s, 2);
        teams8[4] = TEAM_PHOENIX;
        teams8[5] = TEAM_PHOENIX;
        assert_eq!(start_check(0x2000_0004, &teams8, 2, 2), Ok(()));
    }

    #[test]
    fn picks_reject_bogus_slots_and_teams() {
        let mut ts = TeamSelection::default();
        assert_eq!(ts.set_team(4, TEAM_MI6), Err(TeamError::NotAPickingSlot), "bots don't pick [M 0x183a50:140-141]");
        assert_eq!(ts.set_team(0, 5), Err(TeamError::BadTeam));
        assert_eq!(ts.team_of(0), TEAM_PHOENIX, "the state is unchanged after a rejection");
        ts.toggle(0).unwrap();
        assert_eq!(ts.team_of(0), TEAM_MI6);
    }

    #[test]
    fn auto_balance_is_deterministic_and_even() {
        let mut ts = TeamSelection::default();
        ts.set_team(0, TEAM_MI6).unwrap();
        ts.set_team(1, TEAM_MI6).unwrap();
        ts.auto_balance(3);
        assert_eq!((ts.team_of(0), ts.team_of(1), ts.team_of(2)), (TEAM_PHOENIX, TEAM_MI6, TEAM_PHOENIX));
        assert_eq!(ts.team_of(3), TEAM_MI6, "beyond the balanced humans the default picks stay");
    }

    #[test]
    fn apply_writes_humans_and_leaves_bot_records() {
        let mut ts = TeamSelection::default();
        let mut s = MatchSettings::default();
        s.players[4].0 = TEAM_MI6; // a bot record's team [M 0x183a50:140-141]
        s.players[5].0 = TEAM_MI6;
        ts.set_team(1, TEAM_PHOENIX).unwrap();
        ts.apply(&mut s, 2);
        assert_eq!(s.players[0].0, TEAM_PHOENIX);
        assert_eq!(s.players[1].0, TEAM_PHOENIX);
        assert_eq!(s.players[2].0, TEAM_PHOENIX, "unpicked human slots keep their record value");
        assert_eq!(s.players[4].0, TEAM_MI6, "bot slots untouched");
        assert_eq!(s.players[5].0, TEAM_MI6, "bot slots untouched");
    }

    #[test]
    fn team_edit_frame_roundtrips_strictly() {
        let mut buf = Vec::new();
        TeamEdit { slot: 2, team: TEAM_MI6 }.encode(&mut buf);
        assert_eq!(buf.len(), 11);
        assert_eq!(TeamEdit::decode(&buf), Some(TeamEdit { slot: 2, team: TEAM_MI6 }));
        // Trailing garbage → None (the lobby codec's strict-length rule).
        let mut long = buf.clone();
        long.push(0);
        assert_eq!(TeamEdit::decode(&long), None);
        // A different field id → None (not a team edit).
        let mut other = buf.clone();
        other[6] = 0;
        assert_eq!(TeamEdit::decode(&other), None);
        // A wrong magic/version → None.
        let mut bad = buf.clone();
        bad[0] = 0;
        assert_eq!(TeamEdit::decode(&bad), None);
    }
}
