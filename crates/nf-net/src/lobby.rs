//! The one-screen lobby's session messages (mp-lobby-one-screen).
//!
//! Port feature [S — the PS2 game has no network mode, docs/systems/net-online-lobby.md §6.4]:
//! the host publishes the whole lobby snapshot (`LobbyState`), a client pushes one own-row edit
//! (`LobbyEdit`). Both ride the lockstep session's app-to-app DATA tunnel
//! (`Session::send_data` / `Event::Data`, proto.rs type 10) tagged with the magics below, the
//! same way the match-start blob is tagged in the app layer. Little-endian, numbers and ids
//! only; codenames are 32-byte nul-padded like `proto::CODENAME_WIRE`. `decode` is strict:
//! wrong magic, unknown tag, truncated payload → `None`.

/// Host→clients snapshot magic (b"NFLS" + version 1).
pub const LOBBY_STATE_MAGIC: [u8; 4] = [0x4e, 0x46, 0x4c, 0x53];
/// Client→host single-row-edit magic (b"NFLE" + version 1).
pub const LOBBY_EDIT_MAGIC: [u8; 4] = [0x4e, 0x46, 0x4c, 0x45];

pub const CODENAME_WIRE: usize = 32;
pub const SLOTS: usize = 4;
pub const BOTS: usize = 4;
pub const ENVIROMODS: usize = 5;
pub const PLAYER_MODS: usize = 2;

/// `LobbyEdit.field` tags [S §6.4]: which own-row field the client changed.
pub const EDIT_CHARACTER: u8 = 0;
pub const EDIT_TEAM: u8 = 1;
pub const EDIT_READY: u8 = 2;

/// The host's full lobby snapshot — every word the one screen edits [S; the game offsets of the
/// settings words are in docs/systems/net-online-lobby.md §6.2 and the couch pages' handlers].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LobbyState {
    /// MPSettings+0x1A4 [M C_SBMPSCEN's confirm store].
    pub scenario: u32,
    /// MPSettings+0x1A8 [M C_SBMPMAP's confirm store].
    pub level: u32,
    /// MPSettings+0x1B4 [M P_MPPLAYERMODS 0x2104d0:13].
    pub weapon_set: u32,
    /// MPSettings+0x1C4 [M :16].
    pub radar: u32,
    /// MPSettings+0x198 [M :12].
    pub friendly_fire: u32,
    /// MPSettings+0x19C [M P_MPRULES 0x20f790:13].
    pub frag_limit: i32,
    /// MPSettings+0x1A0, minutes, -1 = Unlimited [M :12].
    pub time_limit: i32,
    /// +0x1BC, +0x1C8 [M P_MPPLAYERMODS 0x2104d0:14-15].
    pub player_mods: [u32; PLAYER_MODS],
    /// +0x1C0, +0x1B8, +0x1D4, +0x1D0, +0x1CC in page order [M P_MPENVIROMODS 0x210030].
    pub envmods: [u32; ENVIROMODS],
    /// The four mpbots records (0x2deea8's layout: enabled, team side, character, the seven
    /// stats [M the record layout in the port's MpBot]).
    pub bots: [LobbyBot; BOTS],
    /// The four player rows, slot 0 = the host's own agent [S §3].
    pub players: [LobbyPlayer; SLOTS],
}

/// One bot's record on the wire [M mpbots 0x2deea8 record layout].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LobbyBot {
    /// record+0x0E: enabled.
    pub enabled: bool,
    /// record+0x0F: the side (1 = MI6) [M C_SBMPBTCHOOSE 0x210f08:26-33].
    pub team_mi6: bool,
    /// record+0x10: the character (the mp_characters row id) [M :26].
    pub character: u32,
    /// record+0x00 aggression (Skill 8/5/3/1) [M P_MPBOTSETUP 0x211270:49-59].
    pub skill: i32,
    /// record+0x02 aim (2/3/4) [M :60-68].
    pub aim: i32,
    /// record+0x04 hit points 50..300 [M :69-80].
    pub hit_points: i32,
    /// record+0x06 speed (0/1/2) [M :81-89].
    pub speed: i32,
    /// record+0x07 recovery 1 [M :117-126].
    pub recovery1: i32,
    /// record+0x08 recovery 2 [M :127-135].
    pub recovery2: i32,
    /// record+0x0B trait 0..8 [M :90-116].
    pub trait_id: i32,
}

/// One player row on the wire [S §6.2].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LobbyPlayer {
    pub joined: bool,
    pub ready: bool,
    /// 0 Phoenix / 1 MI6 [M C_RBMPSETUP 0x20de58's team list].
    pub team: u32,
    /// The mp_characters row id [M P_MPBOTCHOOSE's selection over MP_CHARACTERS].
    pub character: u32,
    /// The session's round-trip in ms, display-only [S].
    pub ping_ms: u32,
    pub codename: String,
}

/// A client's own-row edit: slot + which field + the new value [S §6.4]. The host merges it
/// into its model; the next publish carries it to every client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LobbyEdit {
    /// The editing client's lobby slot (0..3).
    pub slot: u8,
    /// [`EDIT_CHARACTER`] / [`EDIT_TEAM`] / [`EDIT_READY`].
    pub field: u8,
    /// character: the mp_characters row id; team: 0/1; ready: 0/1.
    pub value: u32,
}

fn put_codename(out: &mut Vec<u8>, name: &str) {
    let mut buf = [0u8; CODENAME_WIRE];
    let bytes = name.as_bytes();
    let n = bytes.len().min(CODENAME_WIRE - 1);
    buf[..n].copy_from_slice(&bytes[..n]);
    out.extend_from_slice(&buf);
}

fn get_codename(buf: &[u8], off: usize) -> Option<String> {
    let raw = buf.get(off..off + CODENAME_WIRE)?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(CODENAME_WIRE);
    Some(String::from_utf8_lossy(&raw[..end]).into_owned())
}

fn put_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn get_i32(buf: &[u8], off: usize) -> Option<i32> {
    Some(i32::from_le_bytes(buf.get(off..off + 4)?.try_into().ok()?))
}

fn get_u32(buf: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(buf.get(off..off + 4)?.try_into().ok()?))
}

impl LobbyState {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(256);
        o.extend_from_slice(&LOBBY_STATE_MAGIC);
        o.push(1); // version
        for v in [self.scenario, self.level, self.weapon_set, self.radar, self.friendly_fire] {
            o.extend_from_slice(&v.to_le_bytes());
        }
        put_i32(&mut o, self.frag_limit);
        put_i32(&mut o, self.time_limit);
        for v in self.player_mods { o.extend_from_slice(&v.to_le_bytes()); }
        for v in self.envmods { o.extend_from_slice(&v.to_le_bytes()); }
        for b in &self.bots {
            o.push(b.enabled as u8);
            o.push(b.team_mi6 as u8);
            o.extend_from_slice(&b.character.to_le_bytes());
            for v in [b.skill, b.aim, b.hit_points, b.speed, b.recovery1, b.recovery2, b.trait_id] {
                put_i32(&mut o, v);
            }
        }
        for p in &self.players {
            o.push(p.joined as u8);
            o.push(p.ready as u8);
            o.extend_from_slice(&p.team.to_le_bytes());
            o.extend_from_slice(&p.character.to_le_bytes());
            o.extend_from_slice(&p.ping_ms.to_le_bytes());
            put_codename(&mut o, &p.codename);
        }
        o
    }

    pub fn decode(buf: &[u8]) -> Option<LobbyState> {
        let rest = buf.strip_prefix(&LOBBY_STATE_MAGIC)?;
        if rest.first() != Some(&1) { return None; }
        let mut off = LOBBY_STATE_MAGIC.len() + 1; // offsets index the full buffer
        let mut u32s = [0u32; 5 + PLAYER_MODS + ENVIROMODS];
        for slot in u32s.iter_mut() {
            *slot = get_u32(buf, off)?;
            off += 4;
        }
        let frag_limit = get_i32(buf, off)?; off += 4;
        let time_limit = get_i32(buf, off)?; off += 4;
        let mut bots = [LobbyBot {
            enabled: false, team_mi6: false, character: 0, skill: 0, aim: 0, hit_points: 0,
            speed: 0, recovery1: 0, recovery2: 0, trait_id: 0,
        }; BOTS];
        for b in bots.iter_mut() {
            if off + 2 > buf.len() { return None; }
            b.enabled = buf[off] != 0;
            b.team_mi6 = buf[off + 1] != 0;
            off += 2;
            b.character = get_u32(buf, off)?; off += 4;
            b.skill = get_i32(buf, off)?; off += 4;
            b.aim = get_i32(buf, off)?; off += 4;
            b.hit_points = get_i32(buf, off)?; off += 4;
            b.speed = get_i32(buf, off)?; off += 4;
            b.recovery1 = get_i32(buf, off)?; off += 4;
            b.recovery2 = get_i32(buf, off)?; off += 4;
            b.trait_id = get_i32(buf, off)?; off += 4;
        }
        let mut players = [LobbyPlayer {
            joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new(),
        }, LobbyPlayer {
            joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new(),
        }, LobbyPlayer {
            joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new(),
        }, LobbyPlayer {
            joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new(),
        }];
        for p in players.iter_mut() {
            if off + 2 > buf.len() { return None; }
            p.joined = buf[off] != 0;
            p.ready = buf[off + 1] != 0;
            off += 2;
            p.team = get_u32(buf, off)?; off += 4;
            p.character = get_u32(buf, off)?; off += 4;
            p.ping_ms = get_u32(buf, off)?; off += 4;
            p.codename = get_codename(buf, off)?; off += CODENAME_WIRE;
        }
        if off != buf.len() { return None; }
        let [scenario, level, weapon_set, radar, friendly_fire, m0, m1, e0, e1, e2, e3, e4] = u32s;
        Some(LobbyState {
            scenario, level, weapon_set, radar, friendly_fire, frag_limit, time_limit,
            player_mods: [m0, m1], envmods: [e0, e1, e2, e3, e4], bots, players,
        })
    }
}

impl LobbyEdit {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(10);
        o.extend_from_slice(&LOBBY_EDIT_MAGIC);
        o.push(1); // version
        o.push(self.slot);
        o.push(self.field);
        o.extend_from_slice(&self.value.to_le_bytes());
        o
    }

    pub fn decode(buf: &[u8]) -> Option<LobbyEdit> {
        let rest = buf.strip_prefix(&LOBBY_EDIT_MAGIC)?;
        if rest.first() != Some(&1) { return None; }
        if rest.len() != 1 + 1 + 1 + 4 { return None; }
        Some(LobbyEdit {
            slot: rest[1],
            field: rest[2],
            value: get_u32(rest, 3)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LobbyState {
        LobbyState {
            scenario: 0x2000_0002,
            level: 0x0700_0024,
            weapon_set: 3,
            radar: 1,
            friendly_fire: 1,
            frag_limit: 20,
            time_limit: 10,
            player_mods: [0, 1],
            envmods: [2, 1, 0x10, 0, 3],
            bots: [
                LobbyBot { enabled: true, team_mi6: false, character: 1, skill: 5, aim: 2, hit_points: 100, speed: 1, recovery1: 100, recovery2: 100, trait_id: 6 },
                LobbyBot { enabled: false, team_mi6: true, character: 6, skill: 8, aim: 2, hit_points: 100, speed: 1, recovery1: 100, recovery2: 100, trait_id: 0 },
                LobbyBot { enabled: false, team_mi6: false, character: 7, skill: 5, aim: 2, hit_points: 100, speed: 1, recovery1: 100, recovery2: 100, trait_id: 5 },
                LobbyBot { enabled: false, team_mi6: false, character: 8, skill: 5, aim: 2, hit_points: 100, speed: 1, recovery1: 100, recovery2: 100, trait_id: 0 },
            ],
            players: [
                LobbyPlayer { joined: true, ready: true, team: 1, character: 0, ping_ms: 12, codename: "Player 1".into() },
                LobbyPlayer { joined: true, ready: false, team: 0, character: 2, ping_ms: 47, codename: "Alec".into() },
                LobbyPlayer { joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new() },
                LobbyPlayer { joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new() },
            ],
        }
    }

    #[test]
    fn state_roundtrips() {
        let s = sample();
        let dec = LobbyState::decode(&s.encode()).expect("a full snapshot must decode");
        assert_eq!(dec, s, "every word, bot record and player row survives");
        // Sister: a truncated payload must not decode.
        let e = s.encode();
        assert!(LobbyState::decode(&e[..e.len() - 1]).is_none());
    }

    #[test]
    fn edit_roundtrips_and_rejects() {
        let e = LobbyEdit { slot: 2, field: EDIT_READY, value: 1 };
        assert_eq!(LobbyEdit::decode(&e.encode()), Some(e));
        assert!(LobbyEdit::decode(&e.encode()[..5]).is_none(), "truncated edit is none");
        let mut bad = e.encode();
        bad[1] = 9; // unknown version
        assert!(LobbyEdit::decode(&bad).is_none(), "a wrong version is none");
    }

    #[test]
    fn garbage_is_none() {
        assert!(LobbyState::decode(&[]).is_none());
        assert!(LobbyState::decode(b"XXXX0102").is_none());
        assert!(LobbyEdit::decode(&sample_marker().encode()).is_none(),
            "a state blob is not an edit");
    }

    fn sample_marker() -> LobbyState {
        LobbyState {
            scenario: 0, level: 0, weapon_set: 0, radar: 0, friendly_fire: 0,
            frag_limit: 0, time_limit: 0, player_mods: [0; 2], envmods: [0; 5],
            bots: std::array::from_fn(|_| LobbyBot { enabled: false, team_mi6: false, character: 0, skill: 0, aim: 0, hit_points: 0, speed: 0, recovery1: 0, recovery2: 0, trait_id: 0 }),
            players: std::array::from_fn(|_| LobbyPlayer { joined: false, ready: false, team: 0, character: 0, ping_ms: 0, codename: String::new() }),
        }
    }
}
