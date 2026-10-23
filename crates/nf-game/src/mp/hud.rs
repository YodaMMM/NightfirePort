//! The MP HUD's data model — what the in-match HUD reads, as a per-frame snapshot with getters.
//! Port-only delivery shape (the draw side reads it): the game's HUD reads these values straight
//! out of the MP globals each frame (`HUD_Update` 0x19a088 dispatches the per-pane updates over
//! the `MPPaneList` fn pointers [M 0x199b70:42, 0x199e88:36]); the port's snapshot folds the same
//! reads into one struct per frame. No rendering here — `nf_view::hud::radar_project` consumes
//! [`MpRadarObj`]s (pos/argb/kind) as-is.
//!
//! Evidence tags per item; the Ghidra `_NNN_` prints in these decomp files are DECIMAL, so
//! `MPGame._400_4_` = hex +0x190 (the clock) — the same convention the port's comments keep.

use super::{MpMatch, scenario};

/// The uplink terminal's live RGB, `MP_UplinkUpdate` 0x186d98:41-47,87-94,103-109: owner team 0
/// → (0xd2, 0x2d, 0x35), team 1 → (0x2d, 0x61, 0xd2), neutral/reset → the grey set — mirrored to
/// the level script via `SP_SetColour` and packed into the radar blip [M 0x189840:138-140]. The
/// port derives them from the owner word (the game stores the bytes at ext+0x103..5).
fn uplink_rgb(owner: i16) -> [u8; 3] {
    match owner {
        0 => [0xd2, 0x2d, 0x35],
        1 => [0x2d, 0x61, 0xd2],
        _ => [0x7f, 0x7f, 0x7f],
    }
}

/// `RadarObj` kinds — the u16 tag `MP_GetRadarObjects` 0x189840 writes at `RObj + i·0x20 + 0x14`
/// and `HUD_RadarUpdate` 0x19f6e8:137-150 feeds into the `TexUV.291` @0x2becf0 sheet (stride
/// 0x14; kind 5 is written by nothing — the sheet's record 5 is dead [M 0x189840 has no kind-5
/// arm]). Values are the game's, not renumbered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarKind {
    /// 0: a player blip (the viewer themself is never added [M 0x189840:37-40]).
    Player = 0,
    /// 1: a CTF flag (the two `Flags` entries, scenario 0x20000004 [M 0x189840:98-121]).
    Flag = 1,
    /// 2: an uplink terminal (scenario 0x60000008, per-terminal RGB [M 0x189840:123-144]).
    Uplink = 2,
    /// 3: the demolition / protection site (white [M 0x189840:145-172]).
    Target = 3,
    /// 4: a GoldenEye item not yet picked up (skip when `ext+0xf0 & 0x10` [M 0x189840:159-194]).
    GoldenEye = 4,
    /// 6: the blueprint (scenario 0x20000100 [M 0x189840:195-232]).
    Blueprint = 6,
    /// 7: an espionage delivery base (two, red then blue [M 0x189840:207-231]).
    Base = 7,
}

/// One radar entry: world-space position + the packed RGBA the game writes verbatim into
/// `RObj + i·0x20 + 0x10` (the draw reads it straight [M 0x19f6e8:151]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpRadarObj {
    /// `RObj + i·0x20 + 0x00` — the object's `+0xC0` position [M 0x189840:44-52,82-84].
    pub pos: [f32; 3],
    /// `RObj + i·0x20 + 0x10` — packed RGBA bytes (r<<24 | g<<16 | b<<8 | a) [M 0x189840:54-63,
    /// 138-140]. `nf_view::hud::RadarObj.argb` takes this as-is.
    pub argb: u32,
    /// `RObj + i·0x20 + 0x14` (u16) — [`RadarKind::as_u16`].
    pub kind: u16,
    /// The player slot for [`RadarKind::Player`] blips (None for objectives); the game carries
    /// only the obj pointer [M 0x189840:31,37] — the port keeps the slot too (the score row and
    /// the name tag lookups need it).
    pub slot: Option<usize>,
}

impl RadarKind {
    pub fn as_u16(self) -> u16 {
        self as u16
    }
    pub fn from_u16(v: u16) -> Option<Self> {
        Some(match v {
            0 => Self::Player,
            1 => Self::Flag,
            2 => Self::Uplink,
            3 => Self::Target,
            4 => Self::GoldenEye,
            6 => Self::Blueprint,
            7 => Self::Base,
            _ => return None,
        })
    }
}

/// The team colors as the game packs them for the radar blips: team 0 red `0xd22d35ff`, team 1
/// blue `0x2d61d2ff` [M 0x189840:58-62]; a non-team game tints every player white `0x7f7f7fff`
/// [M 0x189840:54-56]. (The score pane tints team 0 `0xff0000ff` and the name tags use dark
/// variants — three palettes over one team datum [M 0x19f0a0:53-55, 0x19f6e8:255-264]; each
/// getter here returns its own reader's value.)
pub fn radar_team_color(team_game: u32, team: u32) -> u32 {
    if team_game == 0 {
        0x7f7f_7fff
    } else if team == 0 {
        0xd22d_35ff
    } else {
        0x2d61_d2ff
    }
}

/// One agent's score row — the values `MP_GetPlayerScore` 0x189730 formats and the pane text
/// shows [M 0x19f0a0:56-70]. The offsets are the port's own `MpSlot` (+0x18 score, +0x04 kills,
/// +0x08 deaths).
#[derive(Clone, Debug, PartialEq)]
pub struct MpScoreRow {
    /// The MPGame slot (humans 0..human_count, then the bots — the game's fixed bot base 4 [M
    /// 0x212198:92-111] follows the human count in the port [S mp-16-slots]).
    pub slot: usize,
    /// `MPSettings + slot·0x30 + 0x20` — the team (the is-target marker in scenario 0x400
    /// [M 0x189840:59, 0x18aa70:38-41]; the port keeps the marker in `MpMatch::assassin_target`).
    pub team: u32,
    /// `MPGame + slot·0x30 + 0x18` (f32) — the scenario score [M 0x189730:17].
    pub score: f32,
    /// `MPGame + slot·0x30 + 0x04` (i32) [M 0x189730:18].
    pub kills: i32,
    /// `MPGame + slot·0x30 + 0x08` (i32) [M 0x189730:19].
    pub deaths: i32,
    /// The Assassin target marker (scenario 0x400 only) [M 0x18aa70:38-41, 0x19f0a0:102-113].
    pub is_target: bool,
}

impl MpScoreRow {
    /// The display score `Menu_GetMPScore` 0x203af8 sorts and prints: the slot's score for every
    /// scoring scenario, the KILLS for 0x1 (Arena) and 0x20000002 (Team Arena) [M 0x203af8:44-54].
    pub fn display_score(&self, sc: u32) -> f32 {
        match sc {
            scenario::DEATHMATCH | scenario::TEAM_DEATHMATCH => self.kills as f32,
            _ => self.score,
        }
    }

    /// The score text's line values, `MP_GetPlayerScore` 0x189730: scenario 0x400 (Assassin) and
    /// 0x40000800 (KOTH FFA) print THREE lines `score\nkills\ndeads` [M :13-19]; everything else
    /// prints `kills\ndeads` [M :20-27]. Returns `(lines, score, kills, deaths)`.
    pub fn score_lines(&self, sc: u32) -> (usize, f32, i32, i32) {
        if sc == scenario::ASSASSIN || sc == scenario::KOTH_FFA {
            (3, self.score, self.kills, self.deaths)
        } else {
            (2, self.kills as f32, self.kills, self.deaths)
        }
    }
}

/// The game's own MP participant layout: 4 humans + 4 bots in 8 `MPGame` slots [M `MP_Start`
/// 0x183a50:116-160]. The HUD keeps the game-sized layout up to this count [S mp-16-slots].
pub const GAME_SLOTS: usize = 8;

/// The radar pane's blip cap: `RadarPane` CREATE+0x16 = 24 runtime blips [M 0x19f6e8:108,153;
/// docs/fields/hud-panes.md]. With 16 participants the worst list is 15 player blips + the 8
/// uplink terminals = 23, so the game's cap still holds every blip (pinned by the tests).
pub const RADAR_BLIP_CAP: usize = 24;

/// One standings row ([`MpHudSnapshot::board`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpBoardRow {
    /// 0-based rank (equal display scores share a rank — the debriefing's walk [M 0x212198:131-143]).
    pub rank: usize,
    pub slot: usize,
    pub team: u32,
    /// [`MpScoreRow::display_score`].
    pub score: f32,
    pub kills: i32,
    pub deaths: i32,
    /// The reading machine's own row.
    pub viewer: bool,
}

/// The standings list the HUD draws ([`MpHudSnapshot::board`]).
#[derive(Clone, Debug, PartialEq)]
pub struct MpBoard {
    pub rows: Vec<MpBoardRow>,
    /// Participants not listed (the cut below `max_rows`).
    pub hidden: usize,
}

/// One uplink terminal's HUD state (`HUD_MPUpdatePane` colors the pane icons from it [M
/// 0x19f0a0:134-180]).
#[derive(Clone, Debug, PartialEq)]
pub struct MpUplinkHud {
    /// The terminal's index (the `Uplinks`[8] slot [M 0x189840:123-144]).
    pub index: usize,
    /// `MP_getUplinkStatus` 0x18ba80: the owner word — 0 team 0, 1 team 1, 2 neutral/reset
    /// [M 0x186d98's +0xf4 writes; 0x18ba80 returns it, 2 for an empty slot].
    pub owner: i16,
    /// `MPOBJECT+0x52` — the controlling player slot, −1 = none [M 0x186d98:61-65].
    pub controller: i16,
    /// The terminal's world position (the radar blip's [M 0x189840:126-135]).
    pub pos: [f32; 3],
    /// The live RGB the update mirrors to the level script and the radar [M 0x186d98:41-47,
    /// 87-94, 103-109; packed into the blip at 0x189840:138-140].
    pub rgb: [u8; 3],
}

/// The per-frame snapshot: everything the MP HUD draws, read off the match state the way the
/// game's pane updates read it. Build once per frame (`MpHudSnapshot::build`), then read.
#[derive(Clone, Debug, Default)]
pub struct MpHudSnapshot {
    /// The reading machine's slot (the game's panes are per-BLData, keyed `BLData+0x94e` [M
    /// 0x19f0a0:31]).
    pub viewer_slot: usize,
    /// Seconds remaining: `limit − clock`, clamped at 0 — the game feeds `(limit − clock)·100`
    /// to `Timer_Seconds2String` and negatives print 00:00 [M 0x184268:144-147, 0x1deaa8:21-25].
    pub time_left: f32,
    /// The score rows in the game's in-match order: the slots as they sit (the pause board walks
    /// humans then bots in slot order — no sort in the in-match paths [M 0x204408:327-399;
    /// 0x19f0a0 is one pane per viewer]).
    pub rows: Vec<MpScoreRow>,
    /// `MPGame + team·4 + 0x180` — the two team totals [M 0x18b338:20-34].
    pub team_scores: [f32; 2],
    /// Per CTF flag: the carrying slot (None = at base or dropped). The game's pane icon asks
    /// `MP_HasTeamFlag(obj, team)` 0x18b430 — the flag ext's `+0x1c` carrier pointer — NOT the
    /// slot's 0x26 bit (that one only bot AI reads) [M 0x19f0a0:89-100; 0x1861a8:173-178 writes
    /// the bit]. The port's `MpObjState::carrier` is that pointer's stand-in.
    pub flags: [Option<usize>; 2],
    /// The uplink terminals' HUD state (scenario 0x60000008; empty otherwise).
    pub uplinks: Vec<MpUplinkHud>,
    /// The Assassin marker pair (scenario 0x400): the local player's assassin and the target
    /// slots [M 0x19f0a0:102-113 — icons 0x3000180 / 0x3000189; the globals are `DAT_0030d774`
    /// (assassin) and `Assasin` 0x30d770 (target) — the names are inverted vs intuition [M
    /// 0x18b538:6, 0x18b558:6]].
    pub assassin: Option<usize>,
    pub target: Option<usize>,
    /// The radar name tags' gate: `MPSettings+0x1c4` (the port's `settings.radar`) set, or
    /// scenario 0x400 [M 0x19f6e8:177]. Colors: FFA `0x1e504bff`, teams `0x5a1414ff` /
    /// `0x14145aff` [M 0x19f6e8:255-264] — a getter, not stored per name.
    pub names_on_radar: bool,
    /// The scenario word (the getters key off it: the display score, the line count, the
    /// Assassin marker). Copied from `settings.scenario` at build [M 0x189730:13-27].
    pub scenario: u32,
    /// The radar's world-space objects, in the game's fill order (players, then the scenario's
    /// markers) — EMPTY when the viewer's radar toggle is off [M 0x189840:22; 0x19f6e8:102-105
    /// hides the ring the same way]. Feed to `nf_view::hud::radar_project`.
    pub radar: Vec<MpRadarObj>,
    /// mp-never-mentioned: the viewer carries the blueprint — `MP_HasEsponage` 0x18b468 (the
    /// `BluePrint` ext's `+0x1c` carrier is the viewer's obj [M]); the score pane re-textures
    /// its objective icon in scenario 0x20000100 [M 0x19f0a0:75-82].
    pub viewer_carries_blueprint: bool,
    /// mp-never-mentioned: the viewer's team holds GoldenEye item k —
    /// `MP_ObjTeamHasGEObj` 0x18b4a0 for items 0/1 (the item's `+0x1c` carrier's team word
    /// equals the viewer's [M]); the score pane re-textures sprites 1/2 in scenario 0x20000200
    /// [M 0x19f0a0:115-127].
    pub viewer_team_ge: [bool; 2],
}

impl MpHudSnapshot {
    /// Read the frame's HUD state off the match. `viewer_slot` = the reading machine's slot
    /// (the game keys the panes off `BLData+0x94e` [M 0x19f0a0:31]); `viewer_radar_on` = the
    /// viewer's `MPSettings + slot·0x30 + 0x28` toggle [M 0x189840:22] — the port keeps it on
    /// `MpPlayer::radar_on`.
    pub fn build(m: &MpMatch, viewer_slot: usize, viewer_radar_on: bool) -> Self {
        let sc = m.settings.scenario;
        let mut snap = MpHudSnapshot {
            viewer_slot,
            scenario: sc,
            // The clock accumulates dt per fixed tick; the game accumulates the
            // `psiGetTimeIn100ths` delta (the port's *Deviation*, mp/mod.rs's module doc).
            time_left: (m.game.limit - m.game.clock).max(0.0),
            team_scores: m.game.team_scores,
            flags: [None, None],
            uplinks: Vec::new(),
            assassin: m.assassin,
            target: m.assassin_target,
            names_on_radar: m.settings.radar != 0 || sc == scenario::ASSASSIN,
            radar: Vec::new(),
            rows: Vec::new(),
            // the esp/GoldenEye getter reads (populated after the flags pass below)
            viewer_carries_blueprint: false,
            viewer_team_ge: [false; 2],
        };

        // The rows: humans 0..human_count, then the bots — the game's slot walk with its
        // fixed bot base 4 [M 0x212198:92-111; the pause board 0x204408:327-399 buckets by the
        // same order] extended to follow the humans [S mp-16-slots].
        let humans = (m.settings.human_count as usize).min(super::MP_MAX_SLOTS);
        let bots = (m.settings.bot_count as usize).min(super::MP_MAX_SLOTS - humans);
        for slot in (0..humans).chain(humans..humans + bots) {
            let s = &m.game.slots[slot];
            snap.rows.push(MpScoreRow {
                slot,
                team: m.settings.players[slot].team,
                score: s.score,
                kills: s.kills,
                deaths: s.deaths,
                is_target: sc == scenario::ASSASSIN && m.assassin_target == Some(slot),
            });
        }

        // The radar: the viewer's toggle gates the WHOLE list [M 0x189840:22-24 — 0 returns an
        // empty list, and the ring hides with it 0x19f6e8:102-105].
        if viewer_radar_on {
            // Player blips: every live slot but the viewer [M 0x189840:29-96]. The game's
            // +0xff state gates (2/4 limbo, 3 the +0x894 arm) ride the BLData pipeline the
            // port's players don't have — the port's `MpSlot::dead` is the stand-in [S, see
            // the report's deviations].
            for slot in 0..humans {
                if slot == viewer_slot || m.game.slots[slot].dead {
                    continue;
                }
                let pos = m.game.slots[slot].pos.unwrap_or_default();
                let argb = radar_team_color(m.settings.team_game, m.settings.players[slot].team);
                snap.radar.push(MpRadarObj { pos, argb, kind: RadarKind::Player.as_u16(), slot: Some(slot) });
            }
            for slot in humans..humans + bots {
                if slot == viewer_slot || m.game.slots[slot].dead {
                    continue;
                }
                let pos = m.game.slots[slot].pos.unwrap_or_default();
                let argb = radar_team_color(m.settings.team_game, m.settings.players[slot].team);
                snap.radar.push(MpRadarObj { pos, argb, kind: RadarKind::Player.as_u16(), slot: Some(slot) });
            }
            // The scenario's markers, the game's order and colors [M 0x189840:97-232].
            match sc {
                scenario::CAPTURE_THE_FLAG => {
                    for (i, f) in m.objects_rt.flags.iter().enumerate() {
                        if let Some(f) = f {
                            // First flag blue, second red [M 0x189840:102-105 — uVar12 counts
                            // 0 then 1: 0 → 0xd22d35ff, 1 → 0x2d61d2ff].
                            let argb = if i == 0 { 0xd22d_35ff } else { 0x2d61_d2ff };
                            snap.radar.push(MpRadarObj { pos: f.pos, argb, kind: RadarKind::Flag.as_u16(), slot: None });
                        }
                    }
                }
                scenario::UPLINK => {
                    for (i, u) in m.objects_rt.uplinks.iter().enumerate() {
                        // The per-terminal RGB [M 0x189840:138-140 — the bytes at ext+0x103..5;
                        // `MP_UplinkUpdate` 0x186d98:41-47,87-94,103-109 sets them to the
                        // owner team's colors — the port derives them from the owner word].
                        let rgb = uplink_rgb(u.state);
                        let argb = (rgb[0] as u32) << 24 | (rgb[1] as u32) << 16 | (rgb[2] as u32) << 8 | 0xff;
                        snap.radar.push(MpRadarObj { pos: u.pos, argb, kind: RadarKind::Uplink.as_u16(), slot: None });
                    }
                }
                scenario::DEMOLITION | scenario::PROTECTION => {
                    if let Some(site) = &m.objects_rt.site {
                        snap.radar.push(MpRadarObj { pos: site.pos, argb: 0x7f7f_7fff, kind: RadarKind::Target.as_u16(), slot: None });
                    }
                }
                scenario::GOLDENEYE => {
                    for g in m.objects_rt.ge.iter().flatten() {
                        // Held items are skipped [M 0x189840:178 — `(ext+0xf0 & 0x10) == 0`
                        // draws; the port's state word: carried = 1/3 per the item's states].
                        if g.state == 0 || g.state == 2 {
                            snap.radar.push(MpRadarObj { pos: g.pos, argb: 0x7f7f_7fff, kind: RadarKind::GoldenEye.as_u16(), slot: None });
                        }
                    }
                }
                scenario::BLUEPRINT => {
                    if let Some(bp) = &m.objects_rt.blueprint {
                        snap.radar.push(MpRadarObj { pos: bp.pos, argb: 0x7f7f_7fff, kind: RadarKind::Blueprint.as_u16(), slot: None });
                    }
                    // The two delivery bases: red then blue [M 0x189840:207-231].
                    for (i, b) in m.objects_rt.esp_bases.iter().enumerate() {
                        if let Some(pos) = b {
                            let argb = if i == 0 { 0xd22d_35ff } else { 0x2d61_d2ff };
                            snap.radar.push(MpRadarObj { pos: *pos, argb, kind: RadarKind::Base.as_u16(), slot: None });
                        }
                    }
                }
                _ => {}
            }
        }

        // The objective status.
        for (i, f) in m.objects_rt.flags.iter().enumerate() {
            snap.flags[i] = f.as_ref().and_then(|f| f.carrier);
        }
        // mp-never-mentioned: the two getter reads the pane's esp/GoldenEye arms key off
        // (`MpMatch::mp_has_esponage` / `MpMatch::mp_obj_team_has_ge_obj` in `rules`).
        snap.viewer_carries_blueprint = m.mp_has_esponage(viewer_slot);
        for k in 0..2 {
            snap.viewer_team_ge[k] = m.mp_obj_team_has_ge_obj(viewer_slot, k);
        }
        if sc == scenario::UPLINK {
            for u in m.objects_rt.uplinks.iter().enumerate() {
                let (i, u) = u;
                snap.uplinks.push(MpUplinkHud {
                    index: i,
                    owner: u.state,
                    controller: u.controller,
                    pos: u.pos,
                    rgb: uplink_rgb(u.state),
                });
            }
        }
        snap
    }

    /// The timer text, `Timer_Seconds2String` 0x1deaa8 mode 2: whole seconds (rounded — the game
    /// passes centiseconds), `ss = secs % 60`, `mm = (secs / 60) % 60`, `"%02d:%02d"` [M
    /// 0x1deaa8:17-33]; negatives clamp to 00:00 [M :21-25]. Hours never show in mode 2.
    pub fn timer_string(&self) -> String {
        let secs = (self.time_left + 0.5).floor() as i64;
        let secs = secs.max(0);
        let ss = secs % 60;
        let mm = (secs / 60) % 60;
        format!("{mm:02}:{ss:02}")
    }

    /// The match's participant count (humans + bots) — the rows' length.
    pub fn participants(&self) -> usize {
        self.rows.len()
    }

    /// More participants than the game's own 8-slot layout (4 humans + 4 bots [M `MP_Start`
    /// 0x183a50:116-160]) — the port's 9–16 player matches [S mp-16-slots]. The HUD draw switches
    /// its standings list to the compact layout when this holds; ≤ 8 keeps the full-size rows.
    pub fn compact(&self) -> bool {
        self.rows.len() > GAME_SLOTS
    }

    /// The in-match standings list [S port-only: the game's in-match HUD has one score pane per
    /// viewer and no list — `HUD_MPUpdatePane` 0x19f0a0 shows the team line over the viewer's own
    /// line; the only ranked board is the debriefing's]. The rows are [`Self::sorted_rows`] (the
    /// debriefing's descending display-score order, equal scores in slot order) with the
    /// debriefing's rank walk (the rank advances only when the score differs from the row above
    /// [M 0x212198:131-143]). At most `max_rows` rows; when the viewer's own row falls below the
    /// cut it replaces the last visible row, so the viewer is ALWAYS listed with their true rank
    /// [S]. `hidden` counts the participants left out.
    pub fn board(&self, max_rows: usize) -> MpBoard {
        let sc = self.scenario;
        let sorted = self.sorted_rows();
        let mut ranked: Vec<MpBoardRow> = Vec::with_capacity(sorted.len());
        let mut rank = 0usize;
        for (i, r) in sorted.iter().enumerate() {
            if i > 0 && r.display_score(sc) != sorted[i - 1].display_score(sc) {
                rank = i;
            }
            ranked.push(MpBoardRow {
                rank,
                slot: r.slot,
                team: r.team,
                score: r.display_score(sc),
                kills: r.kills,
                deaths: r.deaths,
                viewer: r.slot == self.viewer_slot,
            });
        }
        let total = ranked.len();
        let max_rows = max_rows.max(1);
        if total > max_rows {
            let own = ranked.iter().position(|r| r.viewer);
            let mut shown: Vec<MpBoardRow> = ranked[..max_rows].to_vec();
            if let Some(i) = own.filter(|&i| i >= max_rows) {
                shown[max_rows - 1] = ranked[i];
            }
            MpBoard { rows: shown, hidden: total - max_rows }
        } else {
            MpBoard { rows: ranked, hidden: 0 }
        }
    }

    /// The rows sorted the way the game sorts its board:`P_MPDEBRIEFING_Handler` 0x212198's
    /// bubble pass, descending by `Menu_GetMPScore` 0x203af8's display score [M 0x212198:112-130;
    /// 0x203af8:44-54]. The bubble swaps only on a strictly greater score, so equal scores keep
    /// slot order — this getter is that stable order (a merge-safe re-derivation, not a port of
    /// the sort's swap loop).
    pub fn sorted_rows(&self) -> Vec<&MpScoreRow> {
        let sc = self.scenario;
        let mut rows: Vec<&MpScoreRow> = self.rows.iter().collect();
        rows.sort_by(|a, b| {
            let (sa, sb) = (a.display_score(sc), b.display_score(sc));
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        rows
    }
}

// ─── The per-viewer score pane (`MPScorePane` 13: `HUD_CreateMPScorePane` 0x19ed10 +
//     `HUD_MPUpdatePane` 0x19f0a0, `MPScoreInfo` 0x2be888, 7 sprites) ─────────────────────────
//
// The positions are the created runtime rects for one full-screen viewer, measured in the real
// game [L PCSX2 EE snapshots 2026-10-18 of a 1-human Arena (FFA) and Team Arena match on
// 07000023: pane 13's sprite array at `HUD+8+13·0x20+8`] and re-derived from the create code:
// every sprite is created at its template (0,0) / (0,448) and `HUD_FixMP` 0x19b110 clamps it into
// the viewer rect inset by 16 (MPSettings+0x1ac == 1 [M 0x19b110:27-31]) → (16,16) / (16,432−h);
// then 0x19ed10 moves them per scenario [M 0x19ed10:58-123].

/// One icon sprite of the score pane, in HUD units (512×448).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpPaneIcon {
    /// The pane sprite index (1/2/6 = the objective icons, 3/4/5 = the line icons).
    pub sprite: usize,
    /// The texture hash (`hashtable_set_sprite` [M 0x19f0a0] or the template's [L 0x2be888]).
    pub tex: u32,
    pub x: f32,
    pub y: f32,
    /// The sprite colour 0xRRGGBBAA [M 0x19f0a0 / L 0x2be888: 0x7f7f7fff].
    pub rgba: u32,
    /// The draw layer (0x1e — `HUD_MPUpdatePane` writes 0x1e to every shown icon [M]).
    pub layer: i32,
    /// Flag 0x100 (additive): the line icons 3/4/5 carry it, the objective icons not [L 0x2be888].
    pub additive: bool,
}

/// The score pane for one viewer: the text sprite (0) + the icons drawn this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct MpScorePane {
    /// The text, '\n'-separated lines (line 1 may be empty: the FFA player string starts "\n").
    pub text: String,
    /// The text sprite's pen (x, y): (40, 13) when the team line is suppressed, else (40, 30)
    /// [L; M 0x19ed10:101-118 — +0x18 x, −3 y, then +0x11 y unless FFA + not bit30 + not 0x400].
    pub text_x: f32,
    pub text_y: f32,
    /// The text colour (0x7f7f7fff [L MPScoreInfo[0]]) and its outline colour = the sprite's
    /// colour2 (+0x14): black, or with a team line the team's (team 0 `0xff0000ff`, else
    /// `0x0000ffff`) [M 0x19f0a0:52-70 — the store goes to +0x14, not the text colour; L the
    /// Team Arena capture: white digits, red outline for team 0].
    pub rgba: u32,
    pub outline: u32,
    pub icons: Vec<MpPaneIcon>,
}

/// The template textures [L MPScoreInfo @0x2be888]: 3 = 0x03000188 (the team star),
/// 4 = 0x03000187 (the gun = kills), 5 = 0x03000186 (the skull = deaths).
pub const SCORE_ICON_TEX: [u32; 3] = [0x0300_0188, 0x0300_0187, 0x0300_0186];

impl MpScorePane {
    /// `HUD_CreateMPScorePane` + `HUD_MPUpdatePane` for the snapshot's viewer (team
    /// `viewer_team`), the single full-screen viewer layout. `team_game`/`bit30` are
    /// MPSettings+0x18c/+0x190.
    pub fn build(snap: &MpHudSnapshot, team_game: u32, bit30: u32, viewer_team: u32) -> MpScorePane {
        let sc = snap.scenario;
        // The create's text move [M 0x19ed10:101-118]: x += 0x18, y −= 3, then y += 0x11 unless
        // (team_game == 0 && bit30 == 0 && scenario != 0x400) — where sprite 3 hides instead
        // (FuckOffSprite3 = 1, its y −= 0x11).
        let suppressed = team_game == 0 && bit30 == 0 && sc != scenario::ASSASSIN;
        let (text_y, s3_y) = if suppressed { (13.0, -1.0) } else { (30.0, 16.0) };
        // Sprites 4/5 follow 3 at +0x12 each [M 0x19ed10:120-123].
        let mut icons = Vec::new();
        for (k, tex) in SCORE_ICON_TEX.iter().enumerate() {
            if k == 0 && suppressed { continue; } // layer 0xff [M 0x19f0a0:44-46]
            icons.push(MpPaneIcon { sprite: 3 + k, tex: *tex, x: 16.0, y: s3_y + 18.0 * k as f32, rgba: 0x7f7f_7fff, layer: 0x1e, additive: true });
        }
        // The text [M 0x19f0a0:47-70; MP_GetTeamScore 0x18b338, MP_GetPlayerScore 0x189730].
        let row = snap.rows.iter().find(|r| r.slot == snap.viewer_slot);
        let (score, kills, deaths) = row.map_or((0.0, 0, 0), |r| (r.score, r.kills, r.deaths));
        let player = if sc == scenario::ASSASSIN || sc == scenario::KOTH_FFA {
            format!("{}\n{}\n{}", score as i32, kills, deaths)
        } else if team_game == 0 {
            format!("\n{kills}\n{deaths}")
        } else {
            format!("{kills}\n{deaths}")
        };
        let (text, outline) = if team_game != 0 {
            let t = (viewer_team as usize).min(1);
            let team = format!("{} - {}", snap.team_scores[t] as i32, snap.team_scores[1 - t] as i32);
            (format!("{team}\n{player}"), if viewer_team == 0 { 0xff00_00ff } else { 0x0000_ffff })
        } else {
            (player, 0x0000_00ff)
        };
        // The objective icons 1/2/6 [M 0x19f0a0:71-180; positions M 0x19ed10:58-96 + L].
        let me = snap.viewer_slot;
        let icon = |sprite: usize, tex: u32, x: f32, y: f32, rgba: u32| MpPaneIcon { sprite, tex, x, y, rgba, layer: 0x1e, additive: false };
        match sc {
            scenario::CAPTURE_THE_FLAG => {
                // MP_HasTeamFlag(obj, team == 0): carrying the other team's flag, tinted with that
                // flag's colour (viewer team 0 → 0x0040ffff, else 0xff0000ff) [M :89-100].
                let other = if viewer_team == 0 { 1 } else { 0 };
                if snap.flags[other] == Some(me) {
                    icons.push(icon(1, 0x0300_0182, 16.0, 376.0, if viewer_team == 0 { 0x0040_ffff } else { 0xff00_00ff }));
                }
            }
            scenario::ASSASSIN => {
                if snap.assassin == Some(me) { icons.push(icon(1, 0x0300_0180, 16.0, 384.0, 0x7f7f_7fff)); }
                if snap.target == Some(me) { icons.push(icon(2, 0x0300_0189, 16.0, 368.0, 0x7f7f_7fff)); }
            }
            scenario::UPLINK => {
                // All three shown, tinted by `MP_getUplinkStatus(0..2)` [M :134-180]; the create
                // re-textures them 0x03000185 and lines them up at x 16/32/48, y 352 [M 0x19ed10:76-96].
                for (k, sprite) in [1usize, 2, 6].iter().enumerate() {
                    let owner = snap.uplinks.iter().find(|u| u.index == k).map_or(2, |u| u.owner);
                    let rgba = match owner { 0 => 0xd22d_35ff, 1 => 0x2d61_d2ff, _ => 0x7f7f_7fff };
                    icons.push(icon(*sprite, 0x0300_0185, 16.0 + 16.0 * k as f32, 352.0, rgba));
                }
            }
            scenario::BLUEPRINT => {
                // mp-never-mentioned (`MP_HasEsponage` 0x18b468): the carrier's pane dims
                // sprite 1 to layer 0x1e and re-textures it 0x300018a [M 0x19f0a0:75-82]; the
                // template colour stays [L MPScoreInfo]. The create's else-arm (everything but
                // 0x400/0x60000008) moved sprites 1/2 down 0x18 — the same rect CTF's flag
                // icon shows at [M 0x19ed10:113-117].
                if snap.viewer_carries_blueprint {
                    icons.push(icon(1, 0x0300_018a, 16.0, 376.0, 0x7f7f_7fff));
                }
            }
            scenario::GOLDENEYE => {
                // mp-never-mentioned (`MP_ObjTeamHasGEObj` 0x18b4a0): each held item re-textures
                // sprite 1/2 to 0x300018b/0x300018c, layer 0x1e [M 0x19f0a0:115-127]. Both icons
                // take the same rect — the create's else-arm shifts sprites 1/2 by the same
                // −0x18 and moves no x, so a team holding both items stacks them [M; the
                // overlap is an open live check].
                for k in 0..2 {
                    if snap.viewer_team_ge[k] {
                        let tex = if k == 0 { 0x0300_018b } else { 0x0300_018c };
                        icons.push(icon(1 + k, tex, 16.0, 376.0, 0x7f7f_7fff));
                    }
                }
            }
            _ => {}
        }
        MpScorePane { text, text_x: 40.0, text_y, rgba: 0x7f7f_7fff, outline, icons }
    }
}

/// One piece of [`status_line`].
#[derive(Clone, Debug, PartialEq)]
pub enum StatusPart {
    Label(u32),
    /// A label used as a `%s` format with one argument.
    Format(u32, String),
    Text(String),
}

/// The MP status line (`StatusSpr`, `MPStatusInfo` @0x2a4740: (256,48), font 2 centred, white
/// [L]) as `MP_SortOutWhoWon` 0x1848d0 / `MP_Update` 0x184bd0 case 2 write it, in print order
/// (the caller binds and concatenates). Empty while playing.
pub fn status_line(m: &MpMatch) -> Vec<StatusPart> {
    use super::MpEndStatus;
    if m.game.state < 3 { return Vec::new(); }
    let lives = m.settings.scenario == scenario::DEATH_LIMIT_FFA;
    match &m.game.end_status {
        // 0x10: "Player %s Won" 0x2000022 [M :21-41]; else "%s%s %s" (0x2000024, name, 0x2000027) [M :62-70].
        MpEndStatus::Won(n) if lives => vec![StatusPart::Format(0x0200_0022, n.clone())],
        MpEndStatus::Won(n) => vec![StatusPart::Label(0x0200_0024), StatusPart::Text(n.clone()), StatusPart::Text(" ".into()), StatusPart::Label(0x0200_0027)],
        // "%s%s" (0x2000024, 0x2000023) [M :72-90]; 0x10 alone: 0x2000023 [M :43-52].
        MpEndStatus::Draw if lives => vec![StatusPart::Label(0x0200_0023)],
        MpEndStatus::Draw => vec![StatusPart::Label(0x0200_0024), StatusPart::Label(0x0200_0023)],
        // Time up outside 0x10: case 2 prints "%s%s" (0x2000028, 0x1c9) and skips the sort [M 0x184bd0:42-50].
        MpEndStatus::None => vec![StatusPart::Label(0x0200_0028), StatusPart::Label(0x1c9)],
    }
}

/// The kill message `MP_PlayerKilled` 0x188f70:100-107 posts: `Text_AddMsg(recipient, 0, 1,
/// sprintf(Txt_BindLabel(0x200004d), victim's name), 0, 0xb4)` — to the victim's last shooter
/// (`MPGame + victim·0x30 + 0x20`) when that is not the victim. Shown by the MP info pane
/// (`MPMsgInfoStatusPane`, `HUD_MPUpdateStatusPane` 0x19ca88).
pub const KILL_MSG_LABEL: u32 = 0x0200_004d;
/// Its life in ticks (0xb4 = 180).
pub const KILL_MSG_TICKS: u32 = 0xb4;
