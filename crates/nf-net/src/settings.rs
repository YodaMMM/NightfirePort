//! The host's match configuration sent to clients in the handshake.
//!
//! `nf-net` does not depend on `nf-game`, so this is the netcode's own
//! encoding of the `MpSettings` values that matter to the sim. The mapping
//! below is to `MpSettings` (`crates/nf-game/src/mp/mod.rs:76-124`), whose
//! authoritative PS2 hex layout is `docs/systems/mp-match-rules.md:24-29` and
//! `docs/systems/mp-menus.md:176-252` [M]. No serialization of the full
//! struct existed anywhere in the port before this crate (the only save code
//! is `LS_MakeMPSettings` 0x1ce508's 13-byte `MPSG` per-slot block, which
//! carries two unrelated profile fields) — this wire layout is new, tagged
//! [S] as a port feature (the PS2 game has no network mode).
//!
//! Wire layout (338 bytes, little-endian, protocol 5): 20 `u32` scalars in a
//! fixed order, then the per-player records of 4 `u32` (team, skin,
//! armor_bonus, radar_on), then a `u16` registered_pickups + 2 reserved
//! bytes. Offsets in comments are the game's `MpSettings` hex offsets for
//! cross-reference. Protocol 1 carried 176 bytes with 3-field player records
//! and no registered_pickups — the consolidation (net-consolidate 2026-10-16)
//! added the two fields the match-host layer carried and this one dropped.
//! Protocol 2/3 carried 8 records (208 B, the game's own slot count [M]);
//! protocol 4 (mp-16-slots) carries 16 — the port's agent cap [S]; protocol 5
//! (mp-combat-weapons) adds the 20th scalar, Location Damage (+0x1C8).
//! Protocol 7 (mp-lobby-settings) appends the four enviro-mod words the match
//! reads (+0x1B8 gun emplacements, +0x1CC vehicles, +0x1D0 grapple, +0x1D4
//! explosive scenery [M the readers named on `nf_game::mp::MpSettings`]) and the
//! prepared bot records (a count byte + 15 × 0x12-byte `mpbots` records [M
//! Menu_PrepareBots 0x1f94a0]) — the lobby's bot characters/skills and those mods
//! never reached a client before [S].

/// The match configuration the host sends in `WELCOME`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchSettings {
    /// MpSettings.started (0x188)
    pub started: u32,
    /// gametype_mask, same word as scenario (0x1A4)
    pub gametype_mask: u32,
    /// player_count / human joined count, "the viewports" (0x1AC low half)
    pub player_count: u32,
    /// master (0x180)
    pub master: u32,
    /// master_copy (0x184)
    pub master_copy: u32,
    /// team_game = (scenario>>29)&1 (0x18C)
    pub team_game: u32,
    /// bit30 = (scenario>>30)&1 (0x190)
    pub bit30: u32,
    /// total_players (0x194)
    pub total_players: u32,
    /// frag_limit (0x19C)
    pub frag_limit: i32,
    /// time_limit in the game's confirmed unit — menu seconds x60 (0x1A0)
    pub time_limit: i32,
    /// scenario (0x1A4)
    pub scenario: u32,
    /// level_code (0x1A8)
    pub level_code: u32,
    /// human_count (0x1AC)
    pub human_count: u32,
    /// bot_count, clamped <= 4 in the game (0x1B0)
    pub bot_count: u32,
    /// weapon_set (0x1B4)
    pub weapon_set: u32,
    /// capture_reward (0x1BC) — the Player Mods "Professional Mode" word (label 0x1ac [L
    /// USATxt.dat]; `Player_HandlePain` 0x1902c8 multiplies MP damage by 3 when set [M])
    pub capture_reward: u32,
    /// spawn_policy (0x1C0)
    pub spawn_policy: u32,
    /// radar (0x1C4)
    pub radar: u32,
    /// friendly_fire (0x198 hex; Ghidra `_408_` decimal 0x198 was once
    /// misread as decimal +0x408 — the merge-review trap this field caused)
    pub friendly_fire: u32,
    /// location_damage (0x1C8): the Player Mods "Location Damage" word (label 0x24e [L
    /// USATxt.dat]) — the MP pain path's per-part multipliers [M 0x1902c8:62-78]. Protocol 5.
    pub location_damage: u32,
    /// Per-slot records: (team 0x20, skin 0x24, armor_bonus 0x2c, radar_on
    /// +0x28) per MpPlayer; name (+0x00) and saved-flag (+0x28's bit) are not
    /// carried by the port's MpPlayer. The game's own block is 8 records
    /// (slots 0..8) [M MPSettings+0x000..0x17F]; protocol 4 carries 16 — the
    /// port's agent cap, humans first then bots [S]. radar_on rides since
    /// protocol 2 (net-consolidate): `MP_GetRadarObjects` 0x189840 and
    /// `HUD_RadarUpdate` 0x19f6e8 read it per slot [M].
    pub players: [(u32, u32, u32, u32); crate::MAX_AGENTS],
    /// +0x472 (u16): the registered pickup count (protocol 2).
    pub registered_pickups: u16,
    /// +0x1B8, +0x1CC, +0x1D0, +0x1D4 (protocol 7): gun emplacements, vehicles,
    /// grapple, explosive scenery.
    pub mods: [u32; 4],
    /// The prepared bot records, slot order (protocol 7) — at most [`BOT_RECORDS`].
    pub bot_records: Vec<[u8; BOT_RECORD_SIZE]>,
}

/// Bytes occupied by a [`MatchSettings`] on the wire: 20 u32 scalars +
/// [`crate::MAX_AGENTS`] × 16-byte records + the u16 registered_pickups
/// (protocol ≤ 3: 208 with 8 records [M the game's 8-slot block]) + the protocol-7 tail.
pub const SETTINGS_WIRE_SIZE: usize = TAIL + 4 * 4 + 1 + BOT_RECORDS * BOT_RECORD_SIZE;

/// The scalar words before the per-player records (protocol 5: 20).
const SCALARS: usize = 20;

/// One `mpbots` record's size [M 0x2deea8's 0x12 stride].
pub const BOT_RECORD_SIZE: usize = 0x12;
/// The wire's bot-record room: the 16-slot cap minus the host [S mp-16-slots].
pub const BOT_RECORDS: usize = crate::MAX_AGENTS - 1;
/// Where the protocol-7 tail starts (after the u16 registered_pickups + 2 reserved bytes).
const TAIL: usize = SCALARS * 4 + crate::MAX_AGENTS * 16 + 4;

impl Default for MatchSettings {
    fn default() -> Self {
        MatchSettings {
            started: 0,
            gametype_mask: 0,
            player_count: 0,
            master: 1,       // the game writes +0x180 = 1 (mp-menus.md menu confirm)
            master_copy: 0,
            team_game: 0,
            bit30: 0,
            total_players: 0,
            frag_limit: 20,
            time_limit: 10 * 60, // menu hands seconds, x60 at confirm
            scenario: 0,
            level_code: 0,
            human_count: 1,
            bot_count: 0,
            weapon_set: 0,
            capture_reward: 0,
            spawn_policy: 0,
            radar: 0,
            friendly_fire: 0,
            location_damage: 1, // the boot value [M bootup_bootup 0x1c7e78 @0x1c7ff8]
            players: [(0, 0, 0, 1); crate::MAX_AGENTS],
            registered_pickups: 0,
            mods: [0; 4],
            bot_records: Vec::new(),
        }
    }
}

impl MatchSettings {
    /// Encode into `out` (must be at least [`SETTINGS_WIRE_SIZE`] bytes).
    pub fn encode(&self, out: &mut [u8]) {
        for b in out[..SETTINGS_WIRE_SIZE].iter_mut() {
            *b = 0;
        }
        let mut put = |i: usize, v: u32| {
            out[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        };
        put(0, self.started);
        put(1, self.gametype_mask);
        put(2, self.player_count);
        put(3, self.master);
        put(4, self.master_copy);
        put(5, self.team_game);
        put(6, self.bit30);
        put(7, self.total_players);
        put(8, self.frag_limit as u32);
        put(9, self.time_limit as u32);
        put(10, self.scenario);
        put(11, self.level_code);
        put(12, self.human_count);
        put(13, self.bot_count);
        put(14, self.weapon_set);
        put(15, self.capture_reward);
        put(16, self.spawn_policy);
        put(17, self.radar);
        put(18, self.friendly_fire);
        put(19, self.location_damage);
        for (k, &(team, skin, armor, radar_on)) in self.players.iter().enumerate() {
            let base = SCALARS + k * 4;
            put(base, team);
            put(base + 1, skin);
            put(base + 2, armor);
            put(base + 3, radar_on);
        }
        // The registered pickup count (u16) in the old reserved tail.
        let tail = SCALARS * 4 + crate::MAX_AGENTS * 16;
        out[tail..tail + 2].copy_from_slice(&self.registered_pickups.to_le_bytes());
        for (k, v) in self.mods.iter().enumerate() {
            out[TAIL + k * 4..TAIL + k * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        let n = self.bot_records.len().min(BOT_RECORDS);
        out[TAIL + 16] = n as u8;
        for (k, r) in self.bot_records.iter().take(n).enumerate() {
            let at = TAIL + 17 + k * BOT_RECORD_SIZE;
            out[at..at + BOT_RECORD_SIZE].copy_from_slice(r);
        }
    }

    /// Decode from `buf`; `None` if too short.
    pub fn decode(buf: &[u8]) -> Option<MatchSettings> {
        if buf.len() < SETTINGS_WIRE_SIZE {
            return None;
        }
        let get = |i: usize| {
            u32::from_le_bytes([buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2], buf[i * 4 + 3]])
        };
        let mut players = [(0u32, 0u32, 0u32, 0u32); crate::MAX_AGENTS];
        for (k, p) in players.iter_mut().enumerate() {
            let base = SCALARS + k * 4;
            *p = (get(base), get(base + 1), get(base + 2), get(base + 3));
        }
        let tail = SCALARS * 4 + crate::MAX_AGENTS * 16;
        let registered_pickups = u16::from_le_bytes([buf[tail], buf[tail + 1]]);
        let mods: [u32; 4] = std::array::from_fn(|k| {
            let a = TAIL + k * 4;
            u32::from_le_bytes([buf[a], buf[a + 1], buf[a + 2], buf[a + 3]])
        });
        let n = (buf[TAIL + 16] as usize).min(BOT_RECORDS);
        let bot_records = (0..n).map(|k| {
            let at = TAIL + 17 + k * BOT_RECORD_SIZE;
            let mut r = [0u8; BOT_RECORD_SIZE];
            r.copy_from_slice(&buf[at..at + BOT_RECORD_SIZE]);
            r
        }).collect();
        Some(MatchSettings {
            started: get(0),
            gametype_mask: get(1),
            player_count: get(2),
            master: get(3),
            master_copy: get(4),
            team_game: get(5),
            bit30: get(6),
            total_players: get(7),
            frag_limit: get(8) as i32,
            time_limit: get(9) as i32,
            scenario: get(10),
            level_code: get(11),
            human_count: get(12),
            bot_count: get(13),
            weapon_set: get(14),
            capture_reward: get(15),
            spawn_policy: get(16),
            radar: get(17),
            friendly_fire: get(18),
            location_damage: get(19),
            players,
            registered_pickups,
            mods,
            bot_records,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut s = MatchSettings {
            ..Default::default()
        };
        s.scenario = 0x1234_5678;
        s.friendly_fire = 1;
        s.frag_limit = -1;
        s.players[3] = (2, 5, 300, 0);
        s.registered_pickups = 0x1234;
        s.mods = [1, 3, 1, 1];
        s.bot_records = vec![[7u8; BOT_RECORD_SIZE], [9u8; BOT_RECORD_SIZE]];
        let mut b = [0u8; SETTINGS_WIRE_SIZE];
        s.encode(&mut b);
        assert_eq!(MatchSettings::decode(&b).unwrap(), s);
        // Sister assertions: a flipped byte must change the decoded value.
        b[18 * 4] ^= 1; // friendly_fire low bit
        assert_ne!(MatchSettings::decode(&b).unwrap(), s);
        b[18 * 4] ^= 1;
        let tail = SCALARS * 4 + crate::MAX_AGENTS * 16;
        b[tail] ^= 1; // registered_pickups low byte
        assert_ne!(MatchSettings::decode(&b).unwrap(), s);
        b[tail] ^= 1;
        // Sister: the protocol-5 tail — a mod word and a bot-record byte both decode.
        b[TAIL + 8] ^= 1;
        assert_ne!(MatchSettings::decode(&b).unwrap(), s);
        b[TAIL + 8] ^= 1;
        b[TAIL + 17 + BOT_RECORD_SIZE] ^= 1;
        assert_ne!(MatchSettings::decode(&b).unwrap(), s);
    }

    #[test]
    fn short_buffer_is_none() {
        assert!(MatchSettings::decode(&[0u8; 10]).is_none());
    }
}
