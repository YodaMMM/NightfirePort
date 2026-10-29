//! The one-screen online lobby model (mp-lobby-one-screen, docs/systems/net-online-lobby.md §6).
//!
//! One page state holds every MpSettings word the couch pages set, the per-player rows and the
//! host-only Start. Nothing here draws — the draw side renders through [`MpLobby::rows`],
//! [`MpLobby::players`] and [`MpLobby::start_enabled`]. The couch flow is untouched; this model
//! is the ONLINE path only [S — port-only, the PS2 game has no network mode].
//!
//! Evidence: every field cites the couch page that sets the same word [M]; the port-only choices
//! (row layout, the edit-refusal mapping) are [S]. The map wheel is NOT filtered per scenario —
//! nothing in P_MPMAP 0x217468 / C_SBMPMAP 0x217538 reads MPSettings+0x1A4 [M; an earlier
//! "filtered by scenario" note loses to the decompile].

use crate::menu::{
    Action, MpBot, MP_CHARACTERS, MP_MAP_ROWS, MP_RULES_DURATIONS, MP_RULES_POINTS,
    MP_SCENARIO_ROWS, MP_WEAPON_SETS,
};

/// Which row a [`MpLobbyRow`] is — the action mapping and the draw code's hit columns key on
/// this, never on the label text [S].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MpLobbyRowId {
    Scenario,
    Map,
    WeaponSet,
    /// The Points/Lives limit (MPSettings+0x19C).
    Limit,
    /// The duration in minutes (MPSettings+0x1A0).
    Duration,
    FriendlyFire,
    /// The radar row (MPSettings+0x1C4) [M P_MPPLAYERMODS 0x2104d0:16].
    Radar,
    PlayerMod(u8),
    Enviro(u8),
    /// Bot `usize` (0..14): the Enabled toggle + the character wheel on ◀ ▶.
    Bot(usize),
    /// Bot `usize`'s Skill row (the difficulty proxy).
    BotSkill(usize),
    /// Player `usize` (0..15): the character wheel on ◀ ▶, the ready toggle on Confirm.
    Player(usize),
    /// Player `usize`'s team row — only in team games [M the 0x2000_0000 bit, P_MPSETUP 0x20d7a8].
    PlayerTeam(usize),
    /// The host-only Start row.
    Start,
}

/// One row of the one screen, as the draw code and the tests read it [S §6.3].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MpLobbyRow {
    pub id: MpLobbyRowId,
    /// The game's Txt label id when the game has one for the row (the confirm page's own pairs
    /// [M P_MPCONFIRM's enter]); 0 = the port's own literal in [`MpLobbyRow::label_text`] [S].
    pub label_id: u32,
    pub label_text: &'static str,
    /// The value's Txt id when the game binds one (On/Off, the list names); 0 = use
    /// [`MpLobbyRow::value_text`].
    pub value_id: u32,
    pub value_text: String,
    /// This role may edit this row (host rows for the host, the own player row for a client) [S].
    pub editable: bool,
    pub focused: bool,
    /// The Start row's gate; false greys it [S §6.2 — every joined human ready].
    pub enabled: bool,
}

/// One per-player row [S §6.2]. The codename and the ping are display-only; a client edits
/// character / team (team games only) / ready on its own row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MpLobbyPlayer {
    pub codename: String,
    /// The mp_characters row id [M P_MPBOTCHOOSE's selection over MP_CHARACTERS].
    pub character: u32,
    /// 0 Phoenix / 1 MI6 [M C_RBMPSETUP 0x20de58's team list].
    pub team: u32,
    pub ready: bool,
    pub joined: bool,
    /// The session's round-trip in ms, display-only [S].
    pub ping_ms: u32,
}

impl Default for MpLobbyPlayer {
    fn default() -> Self {
        MpLobbyPlayer { codename: String::new(), character: 0, team: 0, ready: false, joined: false, ping_ms: 0 }
    }
}

/// The one-screen lobby state [S; every field's [M] source is on the doc line].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MpLobby {
    /// The page's role: the host edits everything, a client only its own row [S].
    pub host: bool,
    /// MPSettings+0x1A4 [M C_SBMPSCEN's confirm store].
    pub scenario: u32,
    /// MPSettings+0x1A8 [M C_SBMPMAP's confirm store].
    pub level: u32,
    /// MPSettings+0x1B4, 0..10 [M P_MPPLAYERMODS 0x2104d0:13, 26-49].
    pub weapon_set: u32,
    /// MPSettings+0x1C4 [M :16].
    pub radar: u32,
    /// MPSettings+0x198 [M :12].
    pub friendly_fire: u32,
    /// MPSettings+0x19C, the MP_RULES_POINTS list [M P_MPRULES 0x20f790:45-76].
    pub frag_limit: i32,
    /// MPSettings+0x1A0, the MP_RULES_DURATIONS list, -1 = Unlimited [M :21-42].
    pub time_limit: i32,
    /// +0x1BC, +0x1C8 [M P_MPPLAYERMODS 0x2104d0:14-15].
    pub player_mods: [u32; 2],
    /// +0x1C0, +0x1B8, +0x1D4, +0x1D0, +0x1CC in page order [M P_MPENVIROMODS 0x210030].
    pub envmods: [u32; 5],
    /// The mpbots records [M mpbots 0x2deea8 — the layout is MpBot's] — 15 rows, the port's
    /// bot cap [S mp-16-slots; the game's own block is 4 records].
    pub bots: [MpBot; nf_net::lobby::BOTS],
    /// The per-player rows, HOST-relative: slot 0 = the host, 1..15 = the clients [S §6.4 —
    /// the same mapping MpSettings.players and the session's slots use] — 16 rows [S
    /// mp-16-slots].
    pub players: [MpLobbyPlayer; nf_net::lobby::SLOTS],
    /// Which slot is this machine's own row: 0 for the host, the session slot for a client
    /// [S §6.4]. The rows mark it editable; the edits carry it.
    pub own_slot: usize,
    /// The client's own-row fields the local user edited and the host's publish has not
    /// echoed yet (character, team, ready) — the echo clears them [S §6.4].
    own_dirty: (bool, bool, bool),
    /// The focused row index over [`MpLobby::rows`].
    pub focus: usize,
    /// The unlock-everything cheat [M the couch gates read the same flag].
    pub unlock_all: bool,
    /// The reward mask — the Explosive-Scenery gate [M Menu_UnlockMPSettings 0x1ff698's kind-3
    /// id-0x3e bit; the couch model keeps the same word].
    pub bonus_mask: u64,
}

/// `mp_scenario`+0x10's reward column ≠ 0 and the cheat is off [M C_SBMPSCEN 0x20c600:40-42's
/// gate over the same bytes].
fn scenario_locked(unlock_all: bool, row: usize) -> bool {
    let (_, _, _, _, reward) = MP_SCENARIO_ROWS[row.min(MP_SCENARIO_ROWS.len() - 1)];
    reward != 0 && !unlock_all
}

/// A locked character row [M C_SBMPBTCHOOSE 0x210f08:18-20 over mp_characters+0x10].
fn character_locked(unlock_all: bool, row: usize) -> bool {
    MP_CHARACTERS[row.min(MP_CHARACTERS.len() - 1)].3 != 0 && !unlock_all
}

impl MpLobby {
    /// A fresh host model from the front end's boot-default words [S; the defaults are the
    /// couch front end's own — boot sets scenario 1 [M bootup_bootup 0x1c7e78:30] and the
    /// limits/weapon set carry the couch defaults].
    pub fn new_host(
        scenario: u32, level: u32, weapon_set: u32, radar: u32, friendly_fire: u32,
        frag_limit: i32, time_limit: i32, player_mods: [u32; 2], envmods: [u32; 5],
        bots: [MpBot; nf_net::lobby::BOTS], host_codename: &str, unlock_all: bool, bonus_mask: u64,
    ) -> MpLobby {
        let mut lobby = MpLobby {
            host: true,
            scenario, level, weapon_set, radar, friendly_fire, frag_limit, time_limit,
            player_mods, envmods, bots,
            players: std::array::from_fn(|_| MpLobbyPlayer::default()),
            own_slot: 0,
            own_dirty: (false, false, false),
            focus: 0,
            unlock_all,
            bonus_mask,
        };
        lobby.players[0] = MpLobbyPlayer {
            codename: host_codename.to_string(),
            character: 0, team: 0, ready: true, joined: true, ping_ms: 0,
        };
        lobby
    }

    /// The scenario has the team bit — the per-player team rows exist only then [M P_MPSETUP
    /// 0x20d7a8's state-2 arm picks teams from the same bit].
    pub fn team_game(&self) -> bool { self.scenario & 0x2000_0000 != 0 }

    /// The Start gate: at least one joined human and every joined human ready [S §6.2 — the
    /// task's gate; the game's own ready gate is per-panel Menu_MPAreWeReady 0x1f78d8].
    pub fn start_enabled(&self) -> bool {
        if !self.host { return false; }
        let joined: Vec<&MpLobbyPlayer> = self.players.iter().filter(|p| p.joined).collect();
        !joined.is_empty() && joined.iter().all(|p| p.ready)
    }

    /// The Explosive-Scenery gate [M Menu_UnlockMPSettings 0x1ff698's kind-3 id-0x3e bit — the
    /// same word the couch model keeps as mp_bonus_mask].
    ///
    /// mp-lobby-settings: the port's unlock-everything default (`UNLOCK_ALL`, on unless
    /// `--locked`) opens it too — it already opens every locked scenario/character row, and
    /// without it no player could ever reach Explosive Scenery (the bonus word needs the
    /// campaign's reward) [S].
    pub fn explosive_unlocked(&self) -> bool { self.unlock_all || self.bonus_mask >> 0x3e & 1 != 0 }

    /// The row list in screen order [S §6.3]. Settings rows first, then the bots, then the
    /// players, then Start.
    pub fn rows(&mut self) -> Vec<MpLobbyRow> {
        let mut v: Vec<MpLobbyRow> = Vec::new();
        let mut idx = 0usize;
        // The editable matrix [S §6.2]: the settings rows (scenario..bots) are the HOST's; a
        // client edits only its own player row (character / team / ready).
        let host_can = self.host;
        let mut push = |id: MpLobbyRowId, label_id: u32, label_text: &'static str, value_id: u32,
                         value_text: String, editable: bool, enabled: bool, focus: &mut usize| {
            let focused = idx == *focus;
            v.push(MpLobbyRow {
                id, label_id, label_text, value_id, value_text,
                editable: editable && enabled, focused, enabled,
            });
            idx += 1;
        };
        // The values bind through the shared tables — the same words the couch pages show [M
        // the tables carry the game's own label ids].
        let scen_pos = MP_SCENARIO_ROWS.iter().position(|r| r.3 == self.scenario);
        let scen_row = scen_pos.map(|p| MP_SCENARIO_ROWS[p]);
        let scen_open = scen_pos.map_or(false, |p| !scenario_locked(self.unlock_all, p));
        push(MpLobbyRowId::Scenario, 0x3d5, "Scenario",
             scen_row.map_or(0, |r| r.1),
             if scen_row.is_none() { format!("<{:#x}>", self.scenario) } else { String::new() },
             host_can, scen_open, &mut self.focus);
        let map_row = MP_MAP_ROWS.iter().find(|r| r.3 == self.level);
        push(MpLobbyRowId::Map, 0x3d4, "Map", map_row.map_or(0, |r| r.1),
             if map_row.is_none() { format!("<{:#x}>", self.level) } else { String::new() },
             host_can, true, &mut self.focus);
        push(MpLobbyRowId::WeaponSet, 0x24d, "Weapon Set",
             MP_WEAPON_SETS[self.weapon_set.min(10) as usize], String::new(),
             host_can, true, &mut self.focus);
        let limit_id = if self.scenario == 0x10 { 0x3d7 } else { 0x3d8 }; // [M mp_rules_lives]
        push(MpLobbyRowId::Limit, limit_id, "Limit", 0,
             if self.frag_limit >= 0 { self.frag_limit.to_string() } else { "Unlimited".into() },
             host_can, true, &mut self.focus);
        push(MpLobbyRowId::Duration, 0x3d6, "Duration", 0,
             if self.time_limit == -1 { "Unlimited".into() } else { self.time_limit.to_string() },
             host_can, true, &mut self.focus);
        push(MpLobbyRowId::FriendlyFire, 0x3da, "Friendly Fire",
             if self.friendly_fire != 0 { 0x1b7 } else { 0x1b6 }, String::new(),
             host_can, true, &mut self.focus);
        for i in 0..2u8 {
            // The couch Player Mods page's own labels [L USATxt]: 0x1ac "Professional Mode", 0x24e "Location Damage"
            // (the lobby used to show "Professional Mode 1/2").
            push(MpLobbyRowId::PlayerMod(i), [0x1ac, 0x24e][i as usize], "Player Mod",
                 if self.player_mods[i as usize] != 0 { 0x1b7 } else { 0x1b6 }, String::new(),
                 host_can, true, &mut self.focus);
        }
        // The couch page labels this setting 0x1e5 "Team ID" [L USATxt; P_MPPLAYERMODS row 5].
        push(MpLobbyRowId::Radar, 0x1e5, "Radar",
             if self.radar != 0 { 0x1b7 } else { 0x1b6 }, String::new(),
             host_can, true, &mut self.focus);
        for i in 0..5u8 {
            let (vid, enabled) = match i {
                0 => ([0x1b0, 0x1b1, 0x1b2][self.envmods[0].min(2) as usize], true),
                1 => (if self.envmods[1] != 0 { 0x1b7 } else { 0x1b6 }, true),
                2 => (if !self.explosive_unlocked() { 0x0100_0076 } else if self.envmods[2] == 0 { 0x1b6 } else { 0x1b7 }, self.explosive_unlocked()),
                3 => (if self.envmods[3] != 0 { 0x1b7 } else { 0x1b6 }, true),
                _ => ([0x1b6, 0x0100_02d8, 0x0100_02d9, 0x1b2][self.envmods[4].min(3) as usize], true),
            };
            // The couch Enviro-Mods page's labels [L USATxt]: Respawn, Fixed Gun Emplacements, Explosive Scenery,
            // Grapple, Mini Vehicles (the lobby showed "Enviro-Mod 1..5").
            push(MpLobbyRowId::Enviro(i), [0x1af, 0x1ab, 0x1aa, 0x328, 0x5b9][i as usize], "Enviro-Mod", vid, String::new(), host_can, enabled, &mut self.focus);
        }
        // Ravine takes no bots: the game locks the AI Bots row there [M P_MPMAP 0x217678:20-25] and the match start
        // zeroes them [M Menu_PrepareBots 0x1f94a0:85-88] (`FrontEnd::mp_bot_count`). The lobby used to let the host
        // turn bots on for Ravine and then start with none (16 bots set, no bots
        // spawned). On Ravine the bot rows show "Not on this map" and can't be edited.
        let no_bots = self.level == 0x0700_004b;
        for (i, b) in self.bots.iter().enumerate() {
            if no_bots {
                push(MpLobbyRowId::Bot(i), 0, "Bot", 0, "Not on this map".to_string(), false, false, &mut self.focus);
                continue;
            }
            push(MpLobbyRowId::Bot(i), 0, "Bot",
                 if b.enabled { MP_CHARACTERS[b.character.min(28) as usize].1 } else { 0x1b6 },
                 String::new(), host_can, true, &mut self.focus);
            push(MpLobbyRowId::BotSkill(i), 0, "Skill", 0,
                 if b.enabled { skill_name(b.skill).to_string() } else { String::new() },
                 host_can && b.enabled, host_can && b.enabled, &mut self.focus);
        }
        for (i, p) in self.players.iter().enumerate() {
            if !p.joined { continue; }
            let mine = i == self.own_slot; // this machine's own row [S §6.4]
            push(MpLobbyRowId::Player(i), 0, "Player",
                 MP_CHARACTERS[p.character.min(28) as usize].1,
                 format!("{} | {} ms{}", p.codename, p.ping_ms, if p.ready { " | ready" } else { "" }),
                 mine, true, &mut self.focus);
            if self.team_game() {
                push(MpLobbyRowId::PlayerTeam(i), 0, "Team",
                     if p.team == 1 { 0x1c8 } else { 0x1c7 }, String::new(),
                     mine, true, &mut self.focus);
            }
        }
        push(MpLobbyRowId::Start, 0, "Start Match", 0, String::new(),
             self.host, self.start_enabled(), &mut self.focus);
        // The focus stays inside the list when rows disappear (a bot disabled, a team row gone).
        if self.focus >= v.len() { self.focus = v.len().saturating_sub(1); }
        v
    }

    /// Pad navigation over the row list [S §6.2]. Returns whether the state changed.
    pub fn apply(&mut self, a: Action) -> bool {
        let n = self.row_count();
        match a {
            Action::Up if self.focus > 0 => { self.focus -= 1; true }
            Action::Down if self.focus + 1 < n => { self.focus += 1; true }
            Action::Left => self.step(-1),
            Action::Right => self.step(1),
            Action::Confirm => self.confirm(),
            _ => false,
        }
    }

    /// The number of rows `rows()` yields (the join-aware count, without building it) [S].
    pub fn row_count(&self) -> usize {
        let mut n = 7 + 2 + 5; // scenario, map, weapon, limit, duration, ff, radar + 2 mods + 5 enviro
        // Every bot lists its Skill row (blank and disabled while the bot is off) — `rows()`
        // pushes both, so a disabled bot still counts two (counting one stranded the focus on
        // Bot 4: the player rows and Start were unreachable — the two-window check).
        n += self.bots.len() * 2;
        let joined = self.players.iter().filter(|p| p.joined).count();
        n += joined * if self.team_game() { 2 } else { 1 };
        n + 1 // Start
    }

    /// Which row id sits at `focus` (and whether it is editable) — building the row list [S].
    pub fn focused_row(&mut self) -> Option<(MpLobbyRowId, bool)> {
        let rows = self.rows();
        rows.get(self.focus).map(|r| (r.id, r.editable))
    }

    /// ◀ ▶ on the focused row [S — each step cites its couch handler].
    fn step(&mut self, dir: i32) -> bool {
        let (id, editable) = match self.focused_row() { Some(x) => x, None => return false };
        if !editable { return false; }
        match id {
            MpLobbyRowId::Scenario => {
                // Step over the unlocked rows only: the game's confirm refuses a locked row
                // [M C_SBMPSCEN 0x20c600:40-42]; the one screen refuses the step onto it [S].
                let cur = MP_SCENARIO_ROWS.iter().position(|r| r.3 == self.scenario).unwrap_or(0);
                let mut r = cur;
                // Row 0 (Quick Game, word 0) is the couch's preset shortcut, not a scenario — the
                // one screen's wheel steps over it (a lobby match on scenario word 0 has no rules)
                // [S; the couch's Quick Game is `mp_quick_start`].
                for _ in 0..MP_SCENARIO_ROWS.len() {
                    r = (r as i32 + dir).rem_euclid(MP_SCENARIO_ROWS.len() as i32) as usize;
                    if MP_SCENARIO_ROWS[r].3 != 0 && !scenario_locked(self.unlock_all, r) { break; }
                }
                self.scenario = MP_SCENARIO_ROWS[r].3;
                true
            }
            MpLobbyRowId::Map => {
                // All 8 rows at every scenario: the wheel is NOT filtered [M mp-menus.md —
                // nothing in P_MPMAP 0x217468 / C_SBMPMAP 0x217538 reads MPSettings+0x1A4].
                let cur = MP_MAP_ROWS.iter().position(|r| r.3 == self.level).unwrap_or(0);
                let r = (cur as i32 + dir).rem_euclid(MP_MAP_ROWS.len() as i32) as usize;
                self.level = MP_MAP_ROWS[r].3;
                true
            }
            MpLobbyRowId::WeaponSet => {
                self.weapon_set = (self.weapon_set as i32 + dir).rem_euclid(11) as u32; // [M 0x2104d0:26-49's 0..10]
                true
            }
            MpLobbyRowId::Limit => {
                let cur = MP_RULES_POINTS.iter().position(|&t| t == self.frag_limit).unwrap_or(0) as i32;
                self.frag_limit = MP_RULES_POINTS[(cur + dir).clamp(0, MP_RULES_POINTS.len() as i32 - 1) as usize]; // [M 0x20f790:45-76]
                true
            }
            MpLobbyRowId::Duration => {
                let cur = MP_RULES_DURATIONS.iter().position(|&t| t == self.time_limit).unwrap_or(0) as i32;
                self.time_limit = MP_RULES_DURATIONS[(cur + dir).clamp(0, MP_RULES_DURATIONS.len() as i32 - 1) as usize]; // [M 0x20f790:21-42]
                true
            }
            MpLobbyRowId::FriendlyFire => { self.friendly_fire ^= 1; true } // [M 0x2104d0:12's two-item wrap]
            MpLobbyRowId::PlayerMod(i) => {
                let m = &mut self.player_mods[i as usize & 1];
                *m ^= 1; // [M :14-15's On/Off lists]
                true
            }
            MpLobbyRowId::Radar => { self.radar ^= 1; true } // [M :16]
            MpLobbyRowId::Enviro(i) => {
                match i {
                    0 => { self.envmods[0] = (self.envmods[0] as i32 + dir).rem_euclid(3) as u32; } // [M 0x210030:12-16]
                    1 => { self.envmods[1] ^= 1; }
                    // The locked row's list holds one node — ◀ ▶ cannot move it [M :35-61].
                    2 if self.explosive_unlocked() => {
                        self.envmods[2] = if self.envmods[2] > 1 { 1 } else { self.envmods[2] ^ 1 };
                    }
                    3 => { self.envmods[3] ^= 1; }
                    _ if i == 4 => { self.envmods[4] = (self.envmods[4] as i32 + dir).rem_euclid(4) as u32; }
                    _ => return false,
                }
                true
            }
            MpLobbyRowId::Bot(i) => {
                // The character wheel over mp_characters; a locked character refuses [M
                // C_SBMPBTCHOOSE 0x210f08:18-20] — the step skips it [S, the scenario mapping].
                let b = &mut self.bots[i.min(nf_net::lobby::BOTS - 1)];
                // ◀ ▶ on an Off bot switches it on with its current character (the row reads
                // "< Off >": stepping it did nothing, so a bot could only be added by Confirm —
                // the two-window check's bot never joined) [S].
                if !b.enabled { b.enabled = true; return true; }
                let mut r = b.character.min(28) as i32;
                for _ in 0..MP_CHARACTERS.len() {
                    r = (r + dir).rem_euclid(MP_CHARACTERS.len() as i32);
                    if !character_locked(self.unlock_all, r as usize) { break; }
                }
                b.character = r as u32;
                // The couch pick's record writes [M C_SBMPBTCHOOSE 0x210f08:26-43]: +0x0F team =
                // Menu_IsBotGood (the preset's side), the stats reset to the character's preset
                // (`default_bot_stats`), +0x11 dirty = 0. The lobby's bot team was never written,
                // so every bot stayed on its fresh record's side in a team game [S].
                b.team_mi6 = crate::menu::MP_CHARACTER_SIDE_MI6[r as usize];
                b.dirty = false;
                let (skill, aim, hp, speed, r1, r2, trait_id) = MpBot::preset(r as usize);
                (b.skill, b.aim, b.hit_points, b.speed, b.recovery1, b.recovery2, b.trait_id) = (skill, aim, hp, speed, r1, r2, trait_id);
                true
            }
            MpLobbyRowId::BotSkill(i) => {
                // Poor 8 / Average 5 / Good 3 / Very Good 1 [M P_MPBOTSETUP 0x211270:49-59].
                const SKILLS: [i32; 4] = [1, 3, 5, 8];
                let b = &mut self.bots[i.min(nf_net::lobby::BOTS - 1)];
                let cur = SKILLS.iter().position(|&s| s == b.skill).unwrap_or(0) as i32;
                b.skill = SKILLS[(cur + dir).rem_euclid(SKILLS.len() as i32) as usize];
                b.dirty = true; // a hand edit [M P_MPBOTSETUP's store sets +0x11]
                true
            }
            MpLobbyRowId::Player(i) if i.min(nf_net::lobby::SLOTS - 1) == self.own_slot => {
                // The character wheel, own row only [S §6.2]; the edit stays dirty until the
                // host's publish echoes it.
                self.own_dirty.0 = true;
                let p = &mut self.players[i.min(nf_net::lobby::SLOTS - 1)];
                let mut r = p.character.min(28) as i32;
                for _ in 0..MP_CHARACTERS.len() {
                    r = (r + dir).rem_euclid(MP_CHARACTERS.len() as i32);
                    if !character_locked(self.unlock_all, r as usize) { break; }
                }
                p.character = r as u32;
                true
            }
            MpLobbyRowId::PlayerTeam(i) if i.min(nf_net::lobby::SLOTS - 1) == self.own_slot => {
                self.own_dirty.1 = true;
                let t = &mut self.players[i.min(nf_net::lobby::SLOTS - 1)].team;
                *t ^= 1; // 0 Phoenix / 1 MI6 [M C_RBMPSETUP 0x20de58's two-item list]
                true
            }
            MpLobbyRowId::Start => false, // ◀ ▶ do nothing on Start
            MpLobbyRowId::Player(_) | MpLobbyRowId::PlayerTeam(_) => false, // not this machine's row
        }
    }

    /// Confirm on the focused row: the ready toggles, the bot enable, Start [S §6.2].
    fn confirm(&mut self) -> bool {
        let (id, editable) = match self.focused_row() { Some(x) => x, None => return false };
        if !editable { return false; }
        match id {
            MpLobbyRowId::Player(i) if i.min(nf_net::lobby::SLOTS - 1) == self.own_slot => {
                self.own_dirty.2 = true;
                let p = &mut self.players[i.min(nf_net::lobby::SLOTS - 1)];
                p.ready = !p.ready;
                true
            }
            // Confirm on the own team row flips it like ◀ ▶ (this arm sat after the catch-all
            // below and never ran) [S].
            MpLobbyRowId::PlayerTeam(i) if i.min(nf_net::lobby::SLOTS - 1) == self.own_slot => {
                self.own_dirty.1 = true;
                let t = &mut self.players[i.min(nf_net::lobby::SLOTS - 1)].team;
                *t ^= 1;
                true
            }
            MpLobbyRowId::Player(_) | MpLobbyRowId::PlayerTeam(_) => false, // not this machine's row
            MpLobbyRowId::Bot(i) => {
                let b = &mut self.bots[i.min(nf_net::lobby::BOTS - 1)];
                b.enabled = !b.enabled; // the Bot Setup page's own Enabled Yes/No row [M P_MPBOTSETUP row 0]
                true
            }
            _ => false, // Start rides the front end (it hands the match over), not the bare model
        }
    }

    /// The client's own-row edits as the session messages [S §6.4] — its session slot
    /// ([`MpLobby::own_slot`]); empty on the host (the host edits the model directly).
    pub fn own_row_edits(&self) -> Vec<nf_net::LobbyEdit> {
        if self.host { return Vec::new(); }
        let p = &self.players[self.own_slot.min(nf_net::lobby::SLOTS - 1)];
        let slot = self.own_slot.min(nf_net::lobby::SLOTS - 1) as u8;
        let mut v = Vec::new();
        if self.own_dirty.0 {
            v.push(nf_net::LobbyEdit { slot, field: nf_net::lobby::EDIT_CHARACTER, value: p.character });
        }
        if self.own_dirty.1 {
            v.push(nf_net::LobbyEdit { slot, field: nf_net::lobby::EDIT_TEAM, value: p.team });
        }
        if self.own_dirty.2 {
            v.push(nf_net::LobbyEdit { slot, field: nf_net::lobby::EDIT_READY, value: p.ready as u32 });
        }
        v
    }

    /// The own row's dirty fields as one comparable word-triple — the push diff rides on it
    /// [S §6.4].
    pub fn own_row_words(&self) -> (u32, u32, u32) {
        let p = &self.players[self.own_slot.min(nf_net::lobby::SLOTS - 1)];
        (p.character, p.team, p.ready as u32)
    }

    /// One comparable line of every setting word + the rows (the menu script's `lobby` step —
    /// the two-window check diffs the host's line against the client's) [S test aid].
    pub fn debug_line(&self) -> String {
        let bots: Vec<String> = self.bots.iter().enumerate().filter(|(_, b)| b.enabled)
            .map(|(i, b)| format!("b{i}:c{}/s{}/t{}/a{}/hp{}", b.character, b.skill, b.team_mi6 as u8, b.aim, b.hit_points)).collect();
        let players: Vec<String> = self.players.iter().enumerate().filter(|(_, p)| p.joined)
            .map(|(i, p)| format!("p{i}:{}/c{}/t{}/r{}", p.codename, p.character, p.team, p.ready as u8)).collect();
        format!("host={} own={} scen={:#x} map={:#x} ws={} limit={} dur={} ff={} pm={:?} radar={} env={:?} bots=[{}] players=[{}] focus={}",
            self.host, self.own_slot, self.scenario, self.level, self.weapon_set, self.frag_limit, self.time_limit,
            self.friendly_fire, self.player_mods, self.radar, self.envmods, bots.join(" "), players.join(" "), self.focus)
    }

    /// The wire snapshot [S §6.4].
    pub fn to_state(&self) -> nf_net::LobbyState {
        nf_net::LobbyState {
            scenario: self.scenario,
            level: self.level,
            weapon_set: self.weapon_set,
            radar: self.radar,
            friendly_fire: self.friendly_fire,
            frag_limit: self.frag_limit,
            time_limit: self.time_limit,
            player_mods: self.player_mods,
            envmods: self.envmods,
            bots: self.bots.each_ref().map(|b| nf_net::LobbyBot {
                enabled: b.enabled, team_mi6: b.team_mi6, character: b.character,
                skill: b.skill, aim: b.aim, hit_points: b.hit_points, speed: b.speed,
                recovery1: b.recovery1, recovery2: b.recovery2, trait_id: b.trait_id,
            }),
            players: self.players.each_ref().map(|p| nf_net::LobbyPlayer {
                joined: p.joined, ready: p.ready, team: p.team, character: p.character,
                ping_ms: p.ping_ms, codename: p.codename.clone(),
            }),
        }
    }

    /// The client applies the host's snapshot; the own row's local picks survive until the
    /// host's publish confirms them [S §6.4 — the client never edits host words].
    pub fn apply_state(&mut self, s: &nf_net::LobbyState) {
        self.scenario = s.scenario;
        self.level = s.level;
        self.weapon_set = s.weapon_set;
        self.radar = s.radar;
        self.friendly_fire = s.friendly_fire;
        self.frag_limit = s.frag_limit;
        self.time_limit = s.time_limit;
        self.player_mods = s.player_mods;
        self.envmods = s.envmods;
        for (b, sb) in self.bots.iter_mut().zip(s.bots.iter()) {
            b.enabled = sb.enabled;
            b.team_mi6 = sb.team_mi6;
            b.character = sb.character;
            b.skill = sb.skill;
            b.aim = sb.aim;
            b.hit_points = sb.hit_points;
            b.speed = sb.speed;
            b.recovery1 = sb.recovery1;
            b.recovery2 = sb.recovery2;
            b.trait_id = sb.trait_id;
        }
        for (i, (p, sp)) in self.players.iter_mut().zip(s.players.iter()).enumerate() {
            let own = i == self.own_slot; // this machine's own row [S §6.4]
            let (character, team, ready) = (p.character, p.team, p.ready);
            *p = MpLobbyPlayer {
                codename: sp.codename.clone(),
                character: sp.character,
                team: sp.team,
                ready: sp.ready,
                joined: sp.joined,
                ping_ms: sp.ping_ms,
            };
            // The own row keeps the dirty local picks (the host's publish has not echoed them
            // yet); a publish that already carries the local value confirms it — the dirty flag
            // clears. The other rows take the host's words [S §6.4].
            if own && !self.host {
                if self.own_dirty.0 { p.character = character; if sp.character == character { self.own_dirty.0 = false; } }
                if self.own_dirty.1 { p.team = team; if sp.team == team { self.own_dirty.1 = false; } }
                if self.own_dirty.2 { p.ready = ready; if sp.ready == ready { self.own_dirty.2 = false; } }
            }
        }
    }

    /// A client's edit merges into the host's model [S §6.4].
    pub fn apply_edit(&mut self, e: &nf_net::LobbyEdit) -> bool {
        let slot = e.slot as usize;
        if slot >= nf_net::lobby::SLOTS { return false; }
        let p = &mut self.players[slot];
        if !p.joined { return false; }
        match e.field {
            nf_net::lobby::EDIT_CHARACTER => { p.character = e.value.min(28); }
            nf_net::lobby::EDIT_TEAM => { p.team = e.value & 1; }
            nf_net::lobby::EDIT_READY => { p.ready = e.value != 0; }
            _ => return false,
        }
        true
    }
}

/// The Skill list's names [M P_MPBOTSETUP 0x211270:49-59's labels — the values are the record's
/// aggression byte: Very Good 1 / Good 3 / Average 5 / Poor 8].
fn skill_name(skill: i32) -> &'static str {
    match skill { 1 => "Very Good", 3 => "Good", 5 => "Average", 8 => "Poor", _ => "Average" }
}

/// A blank wire snapshot for tests and for the client's "waiting" state [S §6.4 — port-only].
pub fn lobby_state_template() -> nf_net::LobbyState {
    nf_net::LobbyState {
        scenario: 1,
        level: 0x0700_0024,
        weapon_set: 0,
        radar: 1,
        friendly_fire: 0,
        frag_limit: 10,
        time_limit: 10,
        player_mods: [0, 1],
        envmods: [2, 0, 0, 0, 0],
        bots: std::array::from_fn(|_| nf_net::LobbyBot {
            enabled: false, team_mi6: false, character: 0, skill: 5, aim: 2, hit_points: 100,
            speed: 1, recovery1: 100, recovery2: 100, trait_id: 0,
        }),
        players: std::array::from_fn(|_| nf_net::LobbyPlayer {
            joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new(),
        }),
    }
}
