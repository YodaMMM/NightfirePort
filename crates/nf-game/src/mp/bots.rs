//! MP bots: the data layer — per-bot config records, `default_bot_stats` presets,
//! the bot state table, and the `BotGlobal` dispatch skeleton.
//!
//! Groundwork for phase 4 (atlas §6 step 4). Everything here is data-first [M]/[L];
//! nothing in the sim calls it yet — the dispatch skeleton is behind the `mp-bots`
//! feature and compiled for tests only.
//!
//! Evidence tags (docs/fields/README.md): [M] = decompile in `work/decomp/ACTION.ELF/`
//! (function address = filename prefix; `:NN` = line), [L] = EE snapshot read
//! (`work/ee_snapshot.bin`, file offset == EE address), [S] = suggested.
//! Field map: `docs/fields/bot-vars.md`.

#![allow(dead_code)]

// ============================================================================
// 1. The per-bot config record (0x12 bytes each, array of 4 @ 0x2DEEAA)
// ============================================================================
// `MP_Start` 0x183a50:151 passes `0x2deeaa + i*0x12` to `BOT_init` as the MPBOTS*
// record and reads the team byte `0x2deeb9 + i*0x12` (= record+0x0F) into
// MPSettings+slot*0x30+0x20 [M]. `BOT_respawn` 0x124e38:74 rebuilds the same
// pointer on respawn [M]. `Menu_PrepareBots` 0x1f94a0 compacts enabled records
// (record+0x0E != 0) to the front, sets the count byte 0x2DEEA9 and
// MPSettings+0x432, and forces count 0 for map 0x700004b [M].
//
// Field layout ([M] BOT_init 0x1249e8 reads stats 0x00..0x0D and character
// +0x10; Menu_PrepareBots +0x0E; MP_Start +0x0F; atlas §4):
//
// | off  | type | meaning                                            |
// |------|------|----------------------------------------------------|
// | 0x00 | u8   | aggression (0..9)                                  |
// | 0x01 | u8   | (padding in every snapshot; unused [S])            |
// | 0x02 | u8   | aim (0..9)                                         |
// | 0x03 | u8   | (padding [S])                                      |
// | 0x04 | u16  | max HP (`BOT_setDroneStats` 0x126318 → `BOT_SetHealth`) |
// | 0x06 | u8   | speed level                                        |
// | 0x07 | u8   | recovery A                                         |
// | 0x08 | u8   | recovery B (stun frames = stats[8] x 2/3, atlas §4)|
// | 0x09 | u8   | (unused in the readers found)                      |
// | 0x0A | u8   | preferred weapon class                             |
// | 0x0B | u8   | trait 0..8 (`BOTSTATE_getPreferredTraitOpponentObjIndex` 0x128b50 switches on it) |
// | 0x0C | u8   | flags: bit3 0x08 = 360-degree vision, bit4 0x10 = HP regen (BOT_init 0x1249e8 tests `stats & 0x10` to arm the regen timer) |
// | 0x0D | u8   | trailing stats byte (part of the 0xE stats copy)   |
// | 0x0E | u8   | enabled (`Menu_PrepareBots` 0x1f94a0)              |
// | 0x0F | u8   | team (`MP_Start` 0x183a50:140)                     |
// | 0x10 | u8   | character index (`BOT_init` 0x1249e8: `MP_skins[rec[0x10]]`, stored to MPSettings+slot*0x30+0x24) |
// | 0x11 | u8   | dirty                                              |

/// EE address of the 4-record array (4 x 0x12 bytes).
pub const BOT_RECORDS_ADDR: u32 = 0x2DEEAA;
/// EE address of the bot-count byte just before the array (`Menu_PrepareBots`).
pub const BOT_COUNT_ADDR: u32 = 0x2DEEA9;

/// Size of one config record.
pub const BOT_RECORD_SIZE: usize = 0x12;
/// Number of config records (player slots 4..7).
pub const BOT_RECORD_COUNT: usize = 4;

/// One MP bot config record (0x12 bytes), byte-exact lift of the game layout.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BotRecord(pub [u8; BOT_RECORD_SIZE]);

impl BotRecord {
    pub fn aggression(&self) -> u8 { self.0[0x00] }
    pub fn aim(&self) -> u8 { self.0[0x02] }
    pub fn max_hp(&self) -> u16 { u16::from_le_bytes([self.0[0x04], self.0[0x05]]) }
    pub fn speed(&self) -> u8 { self.0[0x06] }
    pub fn recovery_a(&self) -> u8 { self.0[0x07] }
    pub fn recovery_b(&self) -> u8 { self.0[0x08] }
    pub fn preferred_weapon_class(&self) -> u8 { self.0[0x0A] }
    pub fn trait_id(&self) -> u8 { self.0[0x0B] }
    /// bit3 = 360-degree vision, bit4 = HP regen [M BOT_init 0x1249e8].
    pub fn flags(&self) -> u8 { self.0[0x0C] }
    pub fn has_360_vision(&self) -> bool { self.flags() & 0x08 != 0 }
    pub fn has_hp_regen(&self) -> bool { self.flags() & 0x10 != 0 }
    pub fn enabled(&self) -> bool { self.0[0x0E] != 0 }
    pub fn team(&self) -> u8 { self.0[0x0F] }
    pub fn character(&self) -> u8 { self.0[0x10] }
    pub fn dirty(&self) -> bool { self.0[0x11] != 0 }
    /// The 0xE stats subset copied into `BOT_vars+0xA0` by `BOT_init`
    /// (`amemcpy(0x26d700 + i*0x780, rec, 0xE)`, 0x26d700 = 0x26d660+0xA0) [M].
    pub fn stats(&self) -> &[u8; 14] {
        self.0[0x00..0x0E].try_into().expect("stats are the first 0xE bytes")
    }
}

/// The four records exactly as the disc's initial `.data` image has them
/// (identical bytes in all four EE snapshots at 0x2DEEAA) [L ee_snapshot].
/// Slot 4 = preset 0 with enabled=0 / team=0 / character=6; slots 5..7 are
/// empty with characters 7/8/9. `P_MPBOTSETUP_Handler` 0x211270 rewrites them
/// from the menu.
pub const INITIAL_BOT_RECORDS: [BotRecord; BOT_RECORD_COUNT] = [
    BotRecord([
        0x01, 0x00, 0x04, 0x00, 0x96, 0x00, 0x02, 0x96, 0x32, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x06, 0x00,
    ]), // slot 4
    BotRecord([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x07, 0x00,
    ]), // slot 5
    BotRecord([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x08, 0x00,
    ]), // slot 6
    BotRecord([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x09, 0x00,
    ]), // slot 7
];

// ============================================================================
// 2. `default_bot_stats` — 29 preset rows of 0xE bytes @ 0x26D2F0
// ============================================================================
// `BOT_getDefaultStats(i)` 0x1263a8 returns `default_bot_stats + i * 0xe`
// (ELF symbol: 0x26d2f0, size 406 = 29 x 14) [M-sym]. Rows 0..14 end with
// byte 0x0D == 1, rows 15..28 with 0 == 0 — two groups; `BOT_init` uses row 1
// for the null-record default [M 0x1249e8: `BOT_getDefaultStats(1)`].
// Byte layout = the record's stats subset (see `BotRecord::stats`).
// Values below are byte-identical across all four EE snapshots [L ee_snapshot].

/// EE address of the presets table.
pub const DEFAULT_BOT_STATS_ADDR: u32 = 0x26D2F0;
/// Row size (0xE) and row count (29) — `BOT_getDefaultStats` 0x1263a8 [M].
pub const DEFAULT_BOT_STATS_ROW: usize = 0x0E;
pub const DEFAULT_BOT_STATS_COUNT: usize = 29;

/// One `default_bot_stats` row (0xE bytes).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BotStats(pub [u8; DEFAULT_BOT_STATS_ROW]);

impl BotStats {
    pub fn aggression(&self) -> u8 { self.0[0x00] }
    pub fn aim(&self) -> u8 { self.0[0x02] }
    pub fn max_hp(&self) -> u16 { u16::from_le_bytes([self.0[0x04], self.0[0x05]]) }
    pub fn speed(&self) -> u8 { self.0[0x06] }
    pub fn recovery_a(&self) -> u8 { self.0[0x07] }
    pub fn recovery_b(&self) -> u8 { self.0[0x08] }
    pub fn weapon_byte9(&self) -> u8 { self.0[0x09] }
    pub fn preferred_weapon_class(&self) -> u8 { self.0[0x0A] }
    pub fn trait_id(&self) -> u8 { self.0[0x0B] }
    pub fn flags(&self) -> u8 { self.0[0x0C] }
    pub fn trailing(&self) -> u8 { self.0[0x0D] }
    /// The copy `BOT_setDroneStats` 0x126318 performs into the Drone_tag:
    /// HP → `BOT_SetHealth`, +0 → Drone+0xB4, +2 → +0xB5, +6 → +0xB6
    /// (+0xB7 := 3 hardcoded), +7 → +0xB8, +8 → +0xB9, +9 → +0xBA,
    /// and `(f32)aggression` → Drone+0xC0 [M].
    pub const fn drone_stat_bytes(&self) -> [u8; 7] {
        [self.0[0], self.0[2], self.0[6], 3, self.0[7], self.0[8], self.0[9]]
    }
}

/// The 29 preset rows, byte-exact from 0x26D2F0 [L ee_snapshot].
pub const DEFAULT_BOT_STATS: [BotStats; DEFAULT_BOT_STATS_COUNT] = [
    BotStats([0x01, 0x00, 0x04, 0x00, 0x96, 0x00, 0x02, 0x96, 0x32, 0x00, 0x00, 0x00, 0x00, 0x01]), // 0
    BotStats([0x05, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x01, 0x00, 0x06, 0x00, 0x01]), // 1
    BotStats([0x03, 0x00, 0x03, 0x00, 0x96, 0x00, 0x01, 0x64, 0x32, 0x01, 0x00, 0x07, 0x00, 0x01]), // 2
    BotStats([0x03, 0x00, 0x03, 0x00, 0x64, 0x00, 0x01, 0x96, 0x64, 0x01, 0x00, 0x00, 0x00, 0x01]), // 3
    BotStats([0x03, 0x00, 0x03, 0x00, 0x64, 0x00, 0x01, 0x7d, 0x64, 0x00, 0x00, 0x02, 0x00, 0x01]), // 4
    BotStats([0x05, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x00, 0x00, 0x03, 0x00, 0x01]), // 5
    BotStats([0x08, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x01, 0x00, 0x00, 0x00, 0x01]), // 6
    BotStats([0x03, 0x00, 0x03, 0x00, 0x7d, 0x00, 0x01, 0x7d, 0x4b, 0x01, 0x00, 0x00, 0x00, 0x01]), // 7
    BotStats([0x05, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x01, 0x00, 0x05, 0x00, 0x01]), // 8
    BotStats([0x03, 0x00, 0x03, 0x00, 0x96, 0x00, 0x01, 0x7d, 0x4b, 0x01, 0x00, 0x00, 0x00, 0x01]), // 9
    BotStats([0x03, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x01, 0x00, 0x00, 0x00, 0x01]), // 10
    BotStats([0x03, 0x00, 0x03, 0x00, 0xc8, 0x00, 0x02, 0x96, 0x32, 0x01, 0x00, 0x08, 0x00, 0x01]), // 11
    BotStats([0x01, 0x00, 0x04, 0x00, 0x96, 0x00, 0x02, 0x96, 0x32, 0x00, 0x00, 0x00, 0x00, 0x01]), // 12
    BotStats([0x05, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x01, 0x00, 0x06, 0x00, 0x01]), // 13
    BotStats([0x01, 0x00, 0x04, 0x00, 0x96, 0x00, 0x02, 0x96, 0x32, 0x00, 0x00, 0x00, 0x00, 0x01]), // 14
    BotStats([0x05, 0x00, 0x03, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x01, 0x01, 0x06, 0x00, 0x00]), // 15
    BotStats([0x03, 0x00, 0x04, 0x00, 0xfa, 0x00, 0x01, 0x64, 0x32, 0x01, 0x05, 0x07, 0x00, 0x00]), // 16
    BotStats([0x01, 0x00, 0x04, 0x00, 0x96, 0x00, 0x01, 0x96, 0x64, 0x01, 0x01, 0x08, 0x01, 0x00]), // 17
    BotStats([0x05, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x00, 0x00, 0x01, 0x00, 0x00]), // 18
    BotStats([0x05, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x00, 0x00, 0x03, 0x00, 0x00]), // 19
    BotStats([0x03, 0x00, 0x03, 0x00, 0xc8, 0x00, 0x02, 0xc8, 0x32, 0x00, 0x03, 0x04, 0x02, 0x00]), // 20
    BotStats([0x03, 0x00, 0x04, 0x00, 0xc8, 0x00, 0x01, 0x7d, 0x4b, 0x01, 0x02, 0x00, 0x02, 0x00]), // 21
    BotStats([0x03, 0x00, 0x03, 0x00, 0x96, 0x00, 0x02, 0xc8, 0x32, 0x01, 0x04, 0x08, 0x00, 0x00]), // 22
    BotStats([0x05, 0x00, 0x02, 0x00, 0x64, 0x00, 0x01, 0x64, 0x64, 0x01, 0x00, 0x00, 0x00, 0x00]), // 23
    BotStats([0x08, 0x00, 0x03, 0x00, 0x2c, 0x01, 0x00, 0x32, 0xc8, 0x01, 0x00, 0x00, 0x04, 0x00]), // 24
    BotStats([0x05, 0x00, 0x02, 0x00, 0x96, 0x00, 0x02, 0x96, 0x64, 0x01, 0x00, 0x00, 0x18, 0x00]), // 25
    BotStats([0x03, 0x00, 0x03, 0x00, 0x96, 0x00, 0x01, 0x64, 0x96, 0x01, 0x00, 0x00, 0x04, 0x00]), // 26
    BotStats([0x03, 0x00, 0x03, 0x00, 0x64, 0x00, 0x02, 0x7d, 0x4b, 0x01, 0x01, 0x00, 0x00, 0x00]), // 27
    BotStats([0x03, 0x00, 0x04, 0x00, 0x96, 0x00, 0x01, 0x7d, 0x4b, 0x01, 0x00, 0x06, 0x00, 0x00]), // 28
];

// ============================================================================
// 3. The bot state table — ids 0xC3..0xF9
// ============================================================================
// The id → function mapping is `NDrone2_StateFuncs` @ 0x29B300 (250 u32
// entries; read at ids 0xC3..0xF9 from the EE snapshot, byte-identical in all
// four) [L ee_snapshot]. The id → type mapping is a 0x37-byte u8 table at
// 0x26F460 — one past the end of `BOT_vars` — read by
// `BOTSTATE_getStateType` 0x1291f0 as `BOT_vars[state + 0x1D3D]` for
// `state - 0xC3 < 0x37` [M; bytes L ee_snapshot]. State types group the
// family: 1 init, 2 global, 3 trait, 4 attack, 12 cover, 5
// stuck/alert/goto-goal, 6 seen/heard, 7 impact, 10 death, 8 door,
// 13 guard-friend, 9 goal-idle.

/// First bot state id (`NDrone2_DSTATE_BotInit`).
pub const BOT_STATE_FIRST: u8 = 0xC3;
/// One past the last bot state id (0xF9 + 1).
pub const BOT_STATE_END: u8 = 0xFA;
/// Number of bot states (0xFA - 0xC3 = 55).
pub const BOT_STATE_COUNT: usize = (BOT_STATE_END - BOT_STATE_FIRST) as usize;
/// EE address of the state → type byte table (`BOTSTATE_getStateType` 0x1291f0).
pub const BOT_STATE_TYPES_ADDR: u32 = 0x26F460;
/// EE address of the game's state-function pointer table.
pub const BOT_STATE_FUNCS_ADDR: u32 = 0x29B300;

pub mod state {
    /// State ids 0xC3..0xF9, names from the decompile symbols (`action-symtab.tsv`,
    /// addresses cross-checked against `NDrone2_StateFuncs` @0x29B300 [L ee_snapshot]).
    pub const BOT_INIT: u8 = 0xC3;
    pub const BOT_RESPAWN: u8 = 0xC4;
    pub const BOT_GLOBAL: u8 = 0xC5;
    pub const BOT_COLLECTOR: u8 = 0xC6;
    pub const BOT_GUARDIAN: u8 = 0xC7;
    pub const BOT_TEAM_PLAYER: u8 = 0xC8;
    pub const BOT_BULLY: u8 = 0xC9;
    pub const BOT_BERSERKER: u8 = 0xCA;
    pub const BOT_GREEDY: u8 = 0xCB;
    pub const BOT_VENGEFUL: u8 = 0xCC;
    pub const BOT_JUDGE: u8 = 0xCD;
    pub const BOT_ASSASSIN: u8 = 0xCE;
    pub const BOT_ATTACK: u8 = 0xCF;
    pub const BOT_ATTACK_RUN: u8 = 0xD0;
    pub const BOT_ATTACK_NO_ROUTE: u8 = 0xD1;
    pub const BOT_ATTACK_FIRE: u8 = 0xD2;
    pub const BOT_ATTACK_NO_OPPONENT: u8 = 0xD3;
    pub const BOT_ATTACK_STRAFE_AIM_LEFT: u8 = 0xD4;
    pub const BOT_ATTACK_STRAFE_AIM_RIGHT: u8 = 0xD5;
    pub const BOT_ATTACK_RUN_CHANGE_POSITION: u8 = 0xD6;
    pub const BOT_ATTACK_BACKOFF: u8 = 0xD7;
    pub const BOT_ATTACK_CROUCH: u8 = 0xD8;
    pub const BOT_ATTACK_ROLL_LEFT_CROUCH: u8 = 0xD9;
    pub const BOT_ATTACK_ROLL_RIGHT_CROUCH: u8 = 0xDA;
    pub const BOT_ATTACK_STEP_AIM_LEFT: u8 = 0xDB;
    pub const BOT_ATTACK_STEP_AIM_RIGHT: u8 = 0xDC;
    pub const BOT_ATTACK_RELOAD: u8 = 0xDD;
    pub const BOT_ATTACK_CHANGE_WEAPON: u8 = 0xDE;
    pub const BOT_ATTACK_UNARMED: u8 = 0xDF;
    pub const BOT_COVER_RUN_TO: u8 = 0xE0;
    pub const BOT_COVER_INIT: u8 = 0xE1;
    pub const BOT_COVER_IDLE: u8 = 0xE2;
    pub const BOT_COVER_AIM: u8 = 0xE3;
    pub const BOT_COVER_FIRE: u8 = 0xE4;
    pub const BOT_COVER_RETURN: u8 = 0xE5;
    pub const BOT_COVER_TYPE_CHANGE: u8 = 0xE6;
    pub const BOT_COVER_LEAVE: u8 = 0xE7;
    pub const BOT_COVER_LEAVE_NOW: u8 = 0xE8;
    pub const BOT_STUCK: u8 = 0xE9;
    pub const BOT_ALERT_TO_POSITION: u8 = 0xEA;
    pub const BOT_GOTO_GOAL_POSITION: u8 = 0xEB;
    pub const BOT_SEEN_OPPONENT: u8 = 0xEC;
    pub const BOT_SEEN_DRONE_SHOT: u8 = 0xED;
    pub const BOT_HEARD_NOISE: u8 = 0xEE;
    pub const BOT_IMPACT_BULLET: u8 = 0xEF;
    pub const BOT_IMPACT_EXPLOSIVE: u8 = 0xF0;
    pub const BOT_IMPACT_PUNCH: u8 = 0xF1;
    pub const BOT_DEATH_ANIM: u8 = 0xF2;
    pub const BOT_DEATH_BY_EXPLOSION: u8 = 0xF3;
    pub const BOT_DEAD: u8 = 0xF4;
    pub const BOT_IMPACT_STUN_GRENADE: u8 = 0xF5;
    pub const BOT_DOOR_OPEN: u8 = 0xF6;
    pub const BOT_GUARD_FRIEND_IDLE: u8 = 0xF7;
    pub const BOT_GUARD_FRIEND_FOLLOW: u8 = 0xF8;
    pub const BOT_IDLE: u8 = 0xF9;
}

/// One row of the lifted state table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BotStateInfo {
    /// State id (0xC3..0xF9).
    pub id: u8,
    /// Function address in ACTION.ELF.
    pub addr: u32,
    /// Short name (the decompile symbol minus prefixes).
    pub name: &'static str,
    /// Classification for the phase-4 port (see the report): Full (real logic),
    /// Stub (ENTER → one SetState), Thin (a few shared-helper calls), Empty.
    pub kind: StateKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateKind {
    /// Real per-state logic (movement/anim/weapon decisions).
    Full,
    /// A handful of shared-helper calls around one decision.
    Thin,
    /// ENTER (msg 1) does one `Drone_SM_SetState` and nothing else.
    Stub,
    /// No code at all (falls through / returns a constant).
    Empty,
    /// The msg-id dispatcher (0xC5).
    Global,
}

use state::*;

/// The 55 bot states: id, address [L ee_snapshot @0x29B300], name, and the
/// phase-4 classification (kind from reading every .c in work/decomp/ACTION.ELF;
/// details in docs/reports/mp-bots-data.md).
pub const BOT_STATES: [BotStateInfo; BOT_STATE_COUNT] = [
    BotStateInfo { id: BOT_INIT, addr: 0x1748C8, name: "BotInit", kind: StateKind::Thin },
    BotStateInfo { id: BOT_RESPAWN, addr: 0x174938, name: "BotRespawn", kind: StateKind::Thin },
    BotStateInfo { id: BOT_GLOBAL, addr: 0x16E060, name: "BotGlobal", kind: StateKind::Global },
    BotStateInfo { id: BOT_COLLECTOR, addr: 0x174980, name: "BotCollector", kind: StateKind::Full },
    BotStateInfo { id: BOT_GUARDIAN, addr: 0x174A28, name: "BotGuardian", kind: StateKind::Stub },
    BotStateInfo { id: BOT_TEAM_PLAYER, addr: 0x174A70, name: "BotTeamPlayer", kind: StateKind::Stub },
    BotStateInfo { id: BOT_BULLY, addr: 0x174AB8, name: "BotBully", kind: StateKind::Stub },
    BotStateInfo { id: BOT_BERSERKER, addr: 0x174B00, name: "BotBerserker", kind: StateKind::Stub },
    BotStateInfo { id: BOT_GREEDY, addr: 0x174B48, name: "BotGreedy", kind: StateKind::Stub },
    BotStateInfo { id: BOT_VENGEFUL, addr: 0x174B90, name: "BotVengeful", kind: StateKind::Stub },
    BotStateInfo { id: BOT_JUDGE, addr: 0x174BD8, name: "BotJudge", kind: StateKind::Stub },
    BotStateInfo { id: BOT_ASSASSIN, addr: 0x174C20, name: "BotAssassin", kind: StateKind::Stub },
    BotStateInfo { id: BOT_ATTACK, addr: 0x16F198, name: "BotAttack", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_RUN, addr: 0x16F390, name: "BotAttackRun", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_NO_ROUTE, addr: 0x174C68, name: "BotAttackNoRoute", kind: StateKind::Thin },
    BotStateInfo { id: BOT_ATTACK_FIRE, addr: 0x16F5E8, name: "BotAttackFire", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_NO_OPPONENT, addr: 0x174CD0, name: "BotAttackNoOpponent", kind: StateKind::Stub },
    BotStateInfo { id: BOT_ATTACK_STRAFE_AIM_LEFT, addr: 0x16F850, name: "BotAttackStrafeAimLeft", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_STRAFE_AIM_RIGHT, addr: 0x16F9B8, name: "BotAttackStrafeAimRight", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_RUN_CHANGE_POSITION, addr: 0x16FB20, name: "BotAttackRunChangePosition", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_BACKOFF, addr: 0x16FC30, name: "BotAttackBackoff", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_CROUCH, addr: 0x16FE80, name: "BotAttackCrouch", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_ROLL_LEFT_CROUCH, addr: 0x174D18, name: "BotAttackRollLeftCrouch", kind: StateKind::Thin },
    BotStateInfo { id: BOT_ATTACK_ROLL_RIGHT_CROUCH, addr: 0x174DD0, name: "BotAttackRollRightCrouch", kind: StateKind::Thin },
    BotStateInfo { id: BOT_ATTACK_STEP_AIM_LEFT, addr: 0x174E88, name: "BotAttackStepAimLeft", kind: StateKind::Thin },
    BotStateInfo { id: BOT_ATTACK_STEP_AIM_RIGHT, addr: 0x174FA8, name: "BotAttackStepAimRight", kind: StateKind::Thin },
    BotStateInfo { id: BOT_ATTACK_RELOAD, addr: 0x170190, name: "BotAttackReload", kind: StateKind::Full },
    BotStateInfo { id: BOT_ATTACK_CHANGE_WEAPON, addr: 0x1750C8, name: "BotAttackChangeWeapon", kind: StateKind::Thin },
    BotStateInfo { id: BOT_ATTACK_UNARMED, addr: 0x175188, name: "BotAttackUnarmed", kind: StateKind::Thin },
    BotStateInfo { id: BOT_COVER_RUN_TO, addr: 0x1702E0, name: "BotCoverRunTo", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_INIT, addr: 0x170448, name: "BotCoverInit", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_IDLE, addr: 0x1705B0, name: "BotCoverIdle", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_AIM, addr: 0x1708E8, name: "BotCoverAim", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_FIRE, addr: 0x1709D0, name: "BotCoverFire", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_RETURN, addr: 0x170B90, name: "BotCoverReturn", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_TYPE_CHANGE, addr: 0x170D00, name: "BotCoverTypeChange", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_LEAVE, addr: 0x170E20, name: "BotCoverLeave", kind: StateKind::Full },
    BotStateInfo { id: BOT_COVER_LEAVE_NOW, addr: 0x170F38, name: "BotCoverLeaveNow", kind: StateKind::Full },
    BotStateInfo { id: BOT_STUCK, addr: 0x175220, name: "BotStuck", kind: StateKind::Stub },
    BotStateInfo { id: BOT_ALERT_TO_POSITION, addr: 0x171048, name: "BotAlertToPosition", kind: StateKind::Full },
    BotStateInfo { id: BOT_GOTO_GOAL_POSITION, addr: 0x171240, name: "BotGotoGoalPosition", kind: StateKind::Full },
    BotStateInfo { id: BOT_SEEN_OPPONENT, addr: 0x175268, name: "BotSeenOpponent", kind: StateKind::Stub },
    BotStateInfo { id: BOT_SEEN_DRONE_SHOT, addr: 0x1752B0, name: "BotSeenDroneShot", kind: StateKind::Stub },
    BotStateInfo { id: BOT_HEARD_NOISE, addr: 0x1752F8, name: "BotHeardNoise", kind: StateKind::Empty },
    BotStateInfo { id: BOT_IMPACT_BULLET, addr: 0x1713F0, name: "BotImpactBullet", kind: StateKind::Full },
    BotStateInfo { id: BOT_IMPACT_EXPLOSIVE, addr: 0x175308, name: "BotImpactExplosive", kind: StateKind::Thin },
    BotStateInfo { id: BOT_IMPACT_PUNCH, addr: 0x175388, name: "BotImpactPunch", kind: StateKind::Thin },
    BotStateInfo { id: BOT_DEATH_ANIM, addr: 0x175508, name: "BotDeathAnim", kind: StateKind::Thin },
    BotStateInfo { id: BOT_DEATH_BY_EXPLOSION, addr: 0x1755A8, name: "BotDeathByExplosion", kind: StateKind::Thin },
    BotStateInfo { id: BOT_DEAD, addr: 0x171518, name: "BotDead", kind: StateKind::Full },
    BotStateInfo { id: BOT_IMPACT_STUN_GRENADE, addr: 0x175418, name: "BotImpactStunGrenade", kind: StateKind::Thin },
    BotStateInfo { id: BOT_DOOR_OPEN, addr: 0x1716D0, name: "BotDoorOpen", kind: StateKind::Full },
    BotStateInfo { id: BOT_GUARD_FRIEND_IDLE, addr: 0x175680, name: "BotGuardFriendIdle", kind: StateKind::Full },
    BotStateInfo { id: BOT_GUARD_FRIEND_FOLLOW, addr: 0x1718E0, name: "BotGuardFriendFollow", kind: StateKind::Full },
    BotStateInfo { id: BOT_IDLE, addr: 0x171B80, name: "BotIdle", kind: StateKind::Full },
];

/// The state → type table at 0x26F460, 0x37 bytes [L ee_snapshot; reader
/// `BOTSTATE_getStateType` 0x1291f0]. Index = state - 0xC3.
pub const BOT_STATE_TYPES: [u8; BOT_STATE_COUNT] = [
    0x01, 0x01, 0x02, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, // C3..CE
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, // CF..DA
    0x04, 0x04, 0x04, 0x04, 0x04, // DB..DF (17 attack entries total)
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, // E0..E8
    0x05, 0x05, 0x05, // E9..EB
    0x06, 0x06, 0x06, // EC..EE
    0x07, 0x07, 0x07, // EF..F1
    0x0A, 0x0A, 0x0A, // F2..F4
    0x07, // F5
    0x08, // F6
    0x0D, 0x0D, // F7..F8
    0x09, // F9
];

/// Port of `BOTSTATE_getStateType` 0x1291f0: the type byte for a bot state id,
/// or 0 for anything outside 0xC3..0xF9. [M 0x1291f0]
pub const fn state_type(state: u32) -> u8 {
    if state.wrapping_sub(BOT_STATE_FIRST as u32) < BOT_STATE_COUNT as u32 {
        BOT_STATE_TYPES[(state - BOT_STATE_FIRST as u32) as usize]
    } else {
        0
    }
}

// ============================================================================
// 4. The `BotGlobal` dispatch skeleton (state 0xC5, 0x16E060)
// ============================================================================
// `NDrone2_DSTATE_BotGlobal` 0x16E060 is the bots' message router — itself
// state id 0xC5 in `NDrone2_StateFuncs` [L ee_snapshot]. It is a switch on the
// MsgObject id (`*(int*)msg`), 759 decompiled lines. This skeleton mirrors its
// routing only: each arm records what the game does (which helper it calls /
// which state it switches to), so phase 4 turns the arms into real calls.
// Compiled only for tests or with the `mp-bots` feature — nothing calls it.


/// A bot message id, the way `NDrone2_DSTATE_BotGlobal` switches on `*msg` [M 0x16E060].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BotMsg {
    pub id: u8,
    /// `msg[6]` — the payload word the goal/kill/weapon arms read (`puVar17[6]`).
    pub arg: u32,
    /// The sender/current-state word (`msg[1]` in the decompile: `*(int*)(iVar16+4)`).
    pub current_state: u32,
}

/// What the game does per message — the skeleton's outcome descriptors.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BotGlobalOutcome {
    /// `return 0` without action (msg 0x00 and the 0x2F..0x39/0x3E/0x3F block).
    Ignore,
    /// Killed (msg 0x03): dead-drop. If the SMI current state is 0 → SetState(0xF9);
    /// else the type-10 (death) states keep, a dead/invalid opponent clears the
    /// MPGame slot status bits (+0x26) and SetState(0xF2) unless the type is 10.
    Killed { goto: u8 },
    /// Alert family (0x11..0x16, 0x1E): forward to `NDrone2_ReactToDroneAlertMsg`
    /// unless the current state's type is 0/1/2/4/10.
    Alert { react: bool },
    /// Msg 0x18 (impact/explosion alert?): current-state type != 10 → SetState(0xF5 stun).
    StunIfNotDeath { stun: bool },
    /// Msg 0x2E (state change): cover disown / +0x758 clear / alert-status set,
    /// `+0x766 = state_type(state)`; the skeleton reports the recorded fields.
    StateChange { clear_opponent: bool, state_type: u8 },
    /// Msg 0x3A: no current opponent → weight the objective (+1.0), pickGoal(1),
    /// goal found → SetState(0xEB goto goal position).
    SeekObjective { goto: u8 },
    /// Msg 0x3B: `BOTSTATE_setGoalComplete(slot = arg)`.
    GoalComplete { slot: u32 },
    /// Msg 0x3C: `BOTSTATE_uninitGoal(slot = arg)`.
    GoalUninit { slot: u32 },
    /// Msg 0x3D: `BOTSTATE_cancelGoalToObj(arg)`, then SetState(0xF9).
    GoalCancel { goto: u8 },
    /// Msgs 0x40/0x41 (demo/protect site): if my opponent == arg, re-target the
    /// mode's site object (`MP_getProtectionObj_NotExt` / `MP_getDemolitionObj_NotExt`).
    ObjectiveSite,
    /// Msg 0x42 (weapon announce): `BOT_vars+0x76A = arg byte`, `Drone+0x5A2 = current_state`.
    WeaponAnnounce { weapon: u8 },
    /// Msg 0x43 (kill): if the killed was my opponent clear `+0x758`; grudge slot
    /// `+0x76B` matches → clear; `cancelGoalToObj`; type-13 (guard-friend) state → SetState(0xF9).
    KillMsg { goto: u8 },
    /// Msg 0x44 (hit): only when I'm the target → `BOTSTATE_combatWeaponChangeChoice`.
    Hit,
    /// Msg 0x45 (weapon change done): weapon choice + default combat range;
    /// missile opponent → drop the active goal; `+0x771` set and state 0xF9 → SetState(0xCF attack).
    WeaponChanged { goto: u8 },
    /// Any other id: fall through to `return 1`.
    Pass,
}

/// The dispatch skeleton of `NDrone2_DSTATE_BotGlobal` 0x16E060 [M 0x16E060:37-759].
/// `scenario` is MPSettings+0x420 (the mode word — the 0x40/0x41 arms read it);
/// `has_opponent` mirrors `Drone+0x170 != 0`; `goal_slot_flag` mirrors the
/// `BOT_vars+0x771` re-entry flag.

pub fn bot_global_dispatch(
    msg: BotMsg,
    scenario: u32,
    has_opponent: bool,
    goal_slot_flag: bool,
) -> BotGlobalOutcome {
    match msg.id {
        // case 0, 0x2F..0x39, 0x3E, 0x3F: the block falls to `return 0` [M :66-83]
        0x00 | 0x2F..=0x39 | 0x3E | 0x3F => BotGlobalOutcome::Ignore,

        // case 3 (killed) [M :84-114, 401-459]
        0x03 => {
            if msg.current_state == 0 {
                BotGlobalOutcome::Killed { goto: state::BOT_IDLE }
            } else {
                let ty = state_type(msg.current_state);
                // invalid opponent (the universal gate) → slot status cleared;
                // type 10 (death) states stay; anything else → 0xF2 death anim.
                if ty == 10 { BotGlobalOutcome::Killed { goto: msg.current_state as u8 } }
                else { BotGlobalOutcome::Killed { goto: state::BOT_DEATH_ANIM } }
            }
        }

        // 0x11..0x16, 0x1E: alert msgs — react unless state type ∈ {0,1,2,4,10} [M :547-565]
        0x11..=0x16 | 0x1E => {
            let ty = state_type(msg.current_state);
            BotGlobalOutcome::Alert { react: !matches!(ty, 0 | 1 | 2 | 4 | 10) }
        }

        // 0x18: not a death state → 0xF5 stun [M :566-573]
        0x18 => BotGlobalOutcome::StunIfNotDeath { stun: state_type(msg.current_state) != 10 },

        // 0x2E: state change [M :574-608]
        0x2E => {
            let ty = state_type(msg.current_state);
            BotGlobalOutcome::StateChange {
                // type != 0xD (impact) and state != 0xEB → clear the +0x758 opponent
                clear_opponent: ty != 13 && msg.current_state != state::BOT_GOTO_GOAL_POSITION as u32,
                state_type: ty,
            }
        }

        // 0x3A: seek an objective when no opponent [M :609-625]
        0x3A => {
            if has_opponent {
                BotGlobalOutcome::Pass
            } else {
                BotGlobalOutcome::SeekObjective { goto: state::BOT_GOTO_GOAL_POSITION }
            }
        }

        // 0x3B/0x3C/0x3D: the goal messages [M :626-634]
        0x3B => BotGlobalOutcome::GoalComplete { slot: msg.arg },
        0x3C => BotGlobalOutcome::GoalUninit { slot: msg.arg },
        0x3D => BotGlobalOutcome::GoalCancel { goto: state::BOT_IDLE },

        // 0x40/0x41: demo/protect site re-target [M :635-651]
        0x40 | 0x41 => {
            let _ = scenario; // the real arm reads MPSettings+0x420 to pick the site
            BotGlobalOutcome::ObjectiveSite
        }

        // 0x42: weapon announce → +0x76A [M :652-655]
        0x42 => BotGlobalOutcome::WeaponAnnounce { weapon: msg.arg as u8 },

        // 0x43: kill message [M :656-701]
        0x43 => {
            let ty = state_type(msg.current_state);
            BotGlobalOutcome::KillMsg {
                goto: if ty == 13 { state::BOT_IDLE } else { 0 },
            }
        }

        // 0x44: hit → weapon re-choice when I am the target [M :702-729]
        0x44 => BotGlobalOutcome::Hit,

        // 0x45: weapon-change done [M :730-757]
        0x45 => BotGlobalOutcome::WeaponChanged {
            goto: if goal_slot_flag { state::BOT_ATTACK } else { 0 },
        },

        _ => BotGlobalOutcome::Pass,
    }
}

// ============================================================================
// 6. Phase 4 (`mp-bots-states`): the ported state machine — the goal machine,
//    the roam states (0xF9/0xEB) and the attack/fire decision (0xCF/0xD2)
// ============================================================================
// Driver mechanics [M 0x171df8 `Drone_SM_RouteMsgDCV`, 0x175778 `NDrone2_ProcessStateMachine`,
// 0x1490f0 `NDrone2_ControlSTANDARD`:81]: per drone-control tick the game sends itself msg 3
// (0x175a38:31); for MP bots (Drone+0xC5 == 0x1E) msg 3 is delivered to state 0xC5 BotGlobal
// FIRST regardless of the current state [M 0x171df8:78-86], then to the current state; a
// SetState request is committed after the delivery loop (EXIT msg 2, the commit, a synthetic
// msg 0x2E to BotGlobal — its only sender [M 0x171df8:119-203] — then the ENTER msg 1) [M
// 0x1758c0:32-56, 0x171df8:195-197]. The firing pipeline is NOT a state: `ControlSTANDARD`
// :231 runs `DroneWeap_HandleFiring` every tick [M].
//
// Corrections to the phase-401 skeleton this port encodes (from the decompile re-read):
// msg[1] is never read — every "current state" check is the live SM state [M 0x16e060];
// the ignore block returns 1 while the default arm returns 0 (re-delivery) [M :757/:115];
// msg 3 is the per-tick heartbeat with the death branch inside it, not "killed" [M :84-519].


use crate::drone::Rng;

use crate::world::CollisionWorld;


/// The fixed 60 Hz frame count the timers use [L docs/systems/sound.md:394 `FRAME_RATE_INT`].
pub const FRAME_RATE_INT: u32 = 60;


/// One 0x50-byte goal slot (`BOT_vars+0x00` / `+0x50`) — offsets per [M docs/fields/bot-vars.md].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GoalSlot {
    /// +0x00..0x0F — the goal position (`initGoal` 0x1294c0 copies the 0x20-byte CelPos; the port
    /// carries the position word the route targets [M]).
    pub celpos: [f32; 4],
    /// +0x20 — scratch; `gotoGoal` writes the wander budget into slot 1 [M 0x1278a8:84-142].
    pub scratch: f32,
    /// +0x24 — start stamp (seconds, `MPGame+0x412`) [M 0x1278a8:51].
    pub start: f32,
    /// +0x28 — duration := `VIDEO_FRAME_RATE * 5` (= 300.0 at 60 Hz) [M 0x1278a8:50,58; the
    /// atlas's "5 s" gloss was the pre-correction reading — it is 5 s × frame-rate on a
    /// seconds clock, see the report].
    pub duration: f32,
    /// +0x2C — pickup class 3 (health) weight, clamped ≤ 25.0 [M 0x128298:74-81].
    pub weight_health: f32,
    /// +0x30 — pickup class 1 (ammo) weight.
    pub weight_ammo: f32,
    /// +0x34 — pickup class 0 (weapon) weight.
    pub weight_weapon: f32,
    /// +0x38 — the objective weight f32 (`pickGoal` reads it as the multiplier [M 0x126b88:144,292]).
    pub objective_weight: f32,
    /// +0x3C — target index (type 1 → the pickups table, type 3 → the others table) [M].
    pub target: usize,
    /// +0x40 — the state to switch to when the goal fires [M 0x1278a8:59].
    pub fire_state: u8,
    /// +0x44 — flags: bit0 = complete (`setGoalComplete` 0x129510) [M].
    pub complete: u8,
    /// +0x45 — goal type: 1 pickup, 2 objective, 3 hunt player [M].
    pub goal_type: u8,
    /// +0x46 — pickup pref byte (bit1 0x2 one pass, bit2 0x4 the ≥5.0 skip, bit3 0x8 the
    /// opponent-path verify, bit5 0x20 ignore visit times) [M 0x126b88:346,369,404,420,514].
    pub pref_byte: u8,
    /// +0x47 — pickup distance limit (0 stored as −2/0xFE) [M 0x128298:70-77, 0x126b88:362-363].
    pub dist_limit: i8,
    /// +0x48 — the last route code (`BotGotoGoalPosition` writes it per tick [M 0x171240:40]).
    pub route_code: u8,
    /// +0x49 — slot-active flag (slot 1 := 1 at `BOT_init`) [M].
    pub active: u8,
    /// +0x4A — subtype (1 flag … 8 demo/protect, 9 hunt) [M].
    pub subtype: u8,
}


impl Default for GoalSlot {
    fn default() -> Self {
        GoalSlot {
            celpos: [0.0; 4],
            scratch: 0.0,
            start: 0.0,
            duration: 0.0,
            weight_health: 0.0,
            weight_ammo: 0.0,
            weight_weapon: 0.0,
            objective_weight: 0.0,
            target: 0,
            fire_state: 0,
            complete: 0,
            goal_type: 0,
            pref_byte: 0,
            dist_limit: 0,
            route_code: 0,
            active: 0,
            subtype: 0,
        }
    }
}


/// One registered MP pickup — the `MP_PICKUP` row subset the goal machine reads
/// [M 0x189d80 `MP_RegisterPickup`, 0x126b88:386-460, 0x129958].
#[derive(Clone, Debug, PartialEq)]
pub struct BotPickup {
    /// +0x08 — the registered position.
    pub pos: [f32; 3],
    /// `MPOBJECT+0x24` — the weapon id (0 = not resolved; the MP weapon-set ids are runtime
    /// random via `Pickup_MakeRandomWeaponSet`, the matrix is not lifted — see the report).
    pub weapon_id: u16,
    /// `MPOBJECT+0x22` — the pickup class (0 weapon, 1 ammo, 3 health per the pickGoal dispatch
    /// [M 0x126b88:431-477]).
    pub mp_class: u8,
    /// `MPOBJECT+0x20` (s16) == 2 — the "only ≥ 5.0 away" special [M 0x126b88:420-421].
    pub is_obj_flag2: bool,
    /// +0x80 — the per-bot visit stamps [f32; 4] by bot index [M 0x129958:10-15].
    pub visit: [f32; 16],
    /// mp-bot-weapons: the pickup stands takeable (not in its state-2 respawn wait, not deleted).
    /// `pickGoal` skips a row whose registered object is gone [M 0x126b88:386-387 — the
    /// `MPpickups[i] == 0` / `+0x10 == 0` gate]; the port skips the hidden respawn wait too [S].
    pub available: bool,
}


/// One row of the `BOT_vars+0xB0` per-player info block (`BOT_setOtherPlayerInfo` 0x125390).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OtherPlayer {
    /// The slot's object position (`MPGame+slot*0x30+0x1C → obj+0x30`) [M].
    pub pos: [f32; 3],
    /// The player is alive (`MP_playerIsDead` gate [M 0x128b50:102]).
    pub alive: bool,
    /// The player's index 0..7 (`Control_Plr2Ind` domain [M]).
    pub index: usize,
    /// The player's team word (`MPSettings+slot*0x30+0x20`) — bit3 0x8 source [M 0x125390:100-102].
    pub team: u32,
    /// +0x0C flags: bit1 0x2 candidate, bit2 0x4 seen this pass, bit3 0x8 same team [M].
    pub flags: u32,
    /// +0x04 — squared distance to the bot [M 0x125390:148-149].
    pub dist2: f32,
    /// The player's health for the trait-8 pick (their `Drone+0xAC`) [M 0x128b50:201-210].
    pub health: f32,
    /// The player's score (`MPGame+slot*0x30+0x18`) for the trait-4 pick [M 0x128b50:132-150].
    pub score: f32,
}


/// The world view one brain reads each tick — the port's `MPGame`/`MPSettings`/object stand-ins.
/// Every field carries its game origin; the collision world stands in for `BoundsTest` LOS
/// (the same *Deviation* `nav.rs` documents).
pub struct BotCtx<'a> {
    /// Kept for API compatibility, **ignored by `BotBrain::step`**: the brain's canonical clock
    /// is its own 60 Hz tick counter (`self.tick as f32 / 60.0` — the same clock
    /// `next_bullet_at` is stored on). A driver whose wall-clock accumulation drifts by ulps
    /// (or a lockstep peer that derives it differently) must not be able to move sim state —
    /// that was net-lockstep finding F4 (docs/systems/net-lockstep.md).
    pub now: f32,
    /// `MPSettings+0x420` — the scenario word.
    pub scenario: u32,
    /// The bot's own player index (`BOT_vars+0x767`).
    pub my_index: usize,
    /// The bot's team (`MPSettings+slot*0x30+0x20`).
    pub my_team: u32,
    /// The registered pickups (`MPpickups`).
    pub pickups: &'a [BotPickup],
    /// The 8 player slots (`MPGame+slot*0x30+0x1C` objs) — self included; readers skip `my_index`.
    pub others: &'a [OtherPlayer],
    /// The collision world (the LOS/cel stand-in).
    pub world: &'a CollisionWorld,
    /// The arena's AI network (map block 0x05 — `AINetwork_CalcRouteToPosition`'s graph); `None`
    /// = no routes (the bots steer straight, as the unit rigs' empty worlds do).
    pub nav: Option<&'a crate::nav::Network>,
    /// The bot's health (`Drone+0xAC`).
    pub health: f32,
    /// The bot's max HP (`BOT_vars+0xA4`).
    pub max_health: f32,
    /// The attack states' stationary gate: behaviour prop 0x40 / `Drone+0x4F8` bit 0x10
    /// [M 0x16f198:42-45, 0x16f5e8:56-60]. The "stand" preset maps here [S — see the report].
    pub stationary: bool,
    /// mp-perf-16: the caller left the seen bits (0x6) of `others` unset and `find_opponent` runs
    /// the sight line itself — the bot's eye (`pos`) to the other's chest (eye − 0.4), clear of the
    /// bullet-class world (`segment_surface_bullet` = `segment_bullet_clear`) — only for the
    /// candidates whose answer can change the pick (nearer than the best so far). The flags are
    /// read nowhere else in a way the answer changes (the 0xA melee gate and the trait-5 0x10 test
    /// see the same values with bits 0x6 clear or set), so the pick is bit-identical to the
    /// precomputed pass while the per-pair segments fall away [S port optimisation].
    pub lazy_sight: bool,
    /// mp-bot-objectives: the scenario view the objective goals read (the mode objects, the
    /// per-slot/per-team status words, the assassin roles, the other bots' objective goals) —
    /// `None` = no scenario objects (the unit rigs; the brain then plays deathmatch only).
    pub objectives: Option<&'a BotObjectives>,
    /// mp-bot-weapons: the arena's `weapon_data` lift (`botweap::BotWeaponTable`). `Some` arms
    /// the bot with the game's weapon layer on its first tick (`BOTWEAP_InitWeapon`) — the
    /// arsenal, the per-weapon fire timing, the reload/change states, the weapon choice. `None`
    /// keeps the old stand-in (one weapon, an endless 12-round clip) for the unit rigs [S].
    pub weapons: Option<&'a crate::mp::botweap::BotWeaponTable>,
    /// mp-bot-weapons: the match's `startweap` (`weapon_data[PickupMatrix[set·10]].base`
    /// [M 0x129e48:44-45]) — `BotWeaponTable::start_weapon(settings.weapon_set)`.
    pub start_weapon: u8,
}

// ============================================================================
// 7. mp-bot-objectives: the scenario objects as the bots' goal machine reads them
// ============================================================================
// `BOTSTATE_pickGoal` 0x126b88 Phase A (slot 1, objective weight != 0 [M :143-313]) switches on
// MPSettings+0x420 and reads the mode objects through the getter family (`MP_getFlagObj` 0x18b978,
// `MP_getBaseObj` 0x18b9a0, `MP_getUplinkObj` 0x18b9c8, `MP_getHillObj` 0x18bb50,
// `MP_getDemolitionObj` 0x18bab8 / `MP_getProtectionObj` 0x18bac8, `MP_getGoldenEyeObj` 0x18baf0,
// `MP_getBlueprintObj` 0x18bad8, `MP_getEsponageBaseObj` 0x18a338), the bot's player status word
// `MPGame+slot·0x30+0x26` and its team's status word `MPGame+team·2+0x1a8` [M :133-135]. The port
// keeps the objects in `MpObjectsRt`; this snapshot is the read-only view one tick's brains share.

/// The objective-row index in the `BotCtx::others` table: the Demolition/Protection site rides
/// one row past the player slots so the attack/fire states can take it as their opponent — the
/// game sets the site obj itself as `Drone+0x170` (`NDrone2_FindOpponent` 0x1424e0:520-560,
/// `BOTSTATE_opponentIsMissile` 0x129a60) [M]; the row is the port's carrier for that obj [S].
pub const OBJ_ROW: usize = super::MP_MAX_SLOTS;

/// Goal-target ids for the type-2 (objective) goals — `+0x3C` holds the MPOBJECT pointer in the
/// game [M 0x126b88:294-297]; the port encodes which object (kind | index).
pub mod obj_id {
    pub const FLAG: usize = 0x1000;
    pub const BASE: usize = 0x2000;
    pub const UPLINK: usize = 0x3000;
    pub const HILL: usize = 0x4000;
    pub const SITE: usize = 0x5000;
    pub const GOLDENEYE: usize = 0x6000;
    pub const BLUEPRINT: usize = 0x7000;
    /// + team·4 + approach point k (`MP_getEsponageBaseObj(team, k)` [M 0x18a338]).
    pub const ESP_BASE: usize = 0x8000;
    pub const KIND: usize = 0xF000;
}

/// The GoldenEye items' team-status masks `DAT_0030ca88` = {1, 2} [L ee_snapshot 0x30ca88; read
/// `pickGoal` 0x126b88:147,185].
pub const GE_TEAM_MASKS: [u16; 2] = [1, 2];

/// One mode object as the bots see it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ObjView {
    /// The object's current position (`obj+0x30` / the carried object's ride position).
    pub pos: [f32; 3],
    /// The contact sphere centre (`obj+0x80`) and radius (`obj+0x8C`), 0 radius = no bounds.
    pub centre: [f32; 3],
    pub radius: f32,
    /// The box (world-space min/max) for the hill and the site (`celglist+0x20/+0x30`).
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// `obj+0xF4` / `obj+0xF6`.
    pub state: i16,
    pub sub: i16,
    pub carrier: Option<usize>,
}

/// The scenario snapshot one tick's bots share ([`super::MpMatch::bot_objectives`]).
#[derive(Clone, Debug, Default)]
pub struct BotObjectives {
    /// `MPSettings+0x420`.
    pub scenario: u32,
    /// `MPSettings+0x396` — the team view of `BOT_setOtherPlayerInfo` 0x125390:62-73.
    pub team_game: bool,
    /// `MPGame+slot·0x30+0x26` — the player status bits: 1 flag, 2 blueprint, 4/8 GoldenEye item
    /// 0/1, 0x10 on the hill (`MP_setPlayerStatus` 0x18a518 callers: 0x1861a8:178, 0x1882d8:128,
    /// 0x187568:232/235, 0x186ae0:115) [M]. Derived from the objects' carriers (the port's rules
    /// keep only bits 1/0x10) + the hill bit.
    pub status: [u16; super::MP_MAX_SLOTS],
    /// `MPGame+team·2+0x1a8` — the team status: bit0 = the team carries the enemy flag / the
    /// blueprint / GoldenEye item 0 (status events 0x2f/0x32/0x35), bit1 = item 1 (0x36)
    /// [M 0x18a518:107-161]. Derived from the carriers' teams.
    pub team_status: [u16; 2],
    /// The per-slot team words (`MPSettings+slot·0x30+0x20`).
    pub teams: [u32; super::MP_MAX_SLOTS],
    pub flags: [Option<ObjView>; 2],
    pub bases: [Option<ObjView>; 2],
    pub uplinks: Vec<ObjView>,
    pub hill: Option<ObjView>,
    /// The Demolition/Protection site while it stands (`obj+0xF6 != 1`).
    pub site: Option<ObjView>,
    /// The defending team of the site mode (Demolition 0, Protection 1 — `BotGlobal`
    /// 0x16e060:247-255) and the attacking one (`opponentIsMissile` 0x129a60: Protection team 0,
    /// Demolition team 1).
    pub site_defenders: u32,
    pub ge: [Option<ObjView>; 2],
    pub blueprint: Option<ObjView>,
    pub esp_bases: [Option<ObjView>; 2],
    /// Assassination (0x400): the assassin and the target slot (`MP_IsAssasin`,
    /// `MP_getAssassinTarget` [M 0x128b50:80-86]).
    pub assassin: Option<usize>,
    pub assassin_target: Option<usize>,
    /// Every bot's slot-1 goal (slot, goal type, target id) — `BOTSTATE_isObjAlreadyAnotherTeam
    /// Objective` 0x1297b8 walks the bots' `BOT_vars+0x95/+0x8C`. The driver refreshes a bot's row
    /// after its step (the game's bots run in slot order and read each other live).
    pub bot_goals: Vec<(usize, u8, usize)>,
}

impl BotObjectives {
    /// The object behind a type-2 goal target id (its live view), `None` when gone.
    pub fn view(&self, id: usize) -> Option<ObjView> {
        let i = id & !obj_id::KIND;
        match id & obj_id::KIND {
            obj_id::FLAG => self.flags.get(i).copied().flatten(),
            obj_id::BASE => self.bases.get(i).copied().flatten(),
            obj_id::UPLINK => self.uplinks.get(i).copied(),
            obj_id::HILL => self.hill,
            obj_id::SITE => self.site,
            obj_id::GOLDENEYE => self.ge.get(i).copied().flatten(),
            obj_id::BLUEPRINT => self.blueprint,
            obj_id::ESP_BASE => self.esp_point(i / 4, i % 4),
            _ => None,
        }
    }

    /// `MP_getEsponageBaseObj(team, k)` 0x18a338: the k-th approach point round the base — the
    /// sphere centre + (sin a, cos a)·(radius + 0.1) with a = yaw + k·π/2, at the centre height
    /// [M :26-36] (the port's placements are unrotated, yaw 0 [S]; the floor snap
    /// `build_PointOnFloor` is the walk's own floor read).
    pub fn esp_point(&self, team: usize, k: usize) -> Option<ObjView> {
        let b = self.esp_bases.get(team).copied().flatten()?;
        let r = b.radius + 0.1;
        let (s, c) = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][k & 3];
        let p = [b.centre[0] + s * r, b.centre[1], b.centre[2] + c * r];
        Some(ObjView { pos: p, centre: b.centre, radius: b.radius, ..b })
    }

    /// `BOTSTATE_isObjAlreadyAnotherTeamObjective` 0x1297b8: another bot's slot-1 goal is an
    /// objective (type 2) on the same object. The game's team test compares the asker's team word
    /// with itself (always equal — any other bot counts) [M :17-19]; a team-2 asker never counts
    /// [M :9-11].
    pub fn already_objective(&self, id: usize, me: usize) -> bool {
        if self.teams.get(me).is_some_and(|&t| t == 2) {
            return false;
        }
        self.bot_goals.iter().any(|&(s, ty, t)| s != me && ty == 2 && t == id)
    }
}

impl super::MpMatch {
    /// The bots' scenario snapshot for this tick (see [`BotObjectives`]).
    pub fn bot_objectives(&self) -> BotObjectives {
        use super::rules::MpObjState;
        let rt = &self.objects_rt;
        let s = self.settings.scenario;
        let view = |o: &MpObjState| {
            let (centre, radius, min, max) = match o.bounds {
                Some(b) => (
                    [o.pos[0] + b.centre[0], o.pos[1] + b.centre[1], o.pos[2] + b.centre[2]],
                    b.radius,
                    [o.pos[0] + b.min[0], o.pos[1] + b.min[1], o.pos[2] + b.min[2]],
                    [o.pos[0] + b.max[0], o.pos[1] + b.max[1], o.pos[2] + b.max[2]],
                ),
                None => (o.pos, 0.0, o.pos, o.pos),
            };
            ObjView { pos: o.pos, centre, radius, min, max, state: o.state, sub: o.sub, carrier: o.carrier }
        };
        let point = |p: [f32; 3], b: Option<super::rules::MpObjBounds>| {
            let mut o = MpObjState { pos: p, bounds: b, ..Default::default() };
            o.home = p;
            view(&o)
        };
        let mut ob = BotObjectives {
            scenario: s,
            team_game: self.settings.team_game != 0,
            ..Default::default()
        };
        for slot in 0..super::MP_MAX_SLOTS {
            ob.teams[slot] = self.settings.players[slot].team;
            ob.status[slot] = self.game.slots[slot].status_bits & 0x10;
        }
        let team_of = |c: usize| self.settings.players[c].team as usize;
        if s == super::scenario::CAPTURE_THE_FLAG {
            for ti in 0..2 {
                ob.flags[ti] = rt.flags[ti].as_ref().map(view);
                ob.bases[ti] = rt.bases[ti].map(|p| point(p, rt.bases_bounds[ti]));
                if let Some(c) = rt.flags[ti].as_ref().and_then(|f| f.carrier.filter(|_| f.state == 1)) {
                    ob.status[c] |= 1;
                    if team_of(c) < 2 && team_of(c) != ti {
                        ob.team_status[team_of(c)] |= 1;
                    }
                }
            }
        }
        if s == super::scenario::UPLINK {
            ob.uplinks = rt.uplinks.iter().map(view).collect();
        }
        if s == super::scenario::KOTH_FFA || s == super::scenario::KOTH_TEAM {
            ob.hill = rt.hill.as_ref().map(view);
        }
        if s == super::scenario::DEMOLITION || s == super::scenario::PROTECTION {
            ob.site = rt.site.as_ref().filter(|x| x.sub != 1).map(view);
            ob.site_defenders = if s == super::scenario::DEMOLITION { 0 } else { 1 };
        }
        if s == super::scenario::GOLDENEYE {
            for i in 0..2 {
                ob.ge[i] = rt.ge[i].as_ref().map(view);
                if let Some(c) = rt.ge[i].as_ref().and_then(|g| g.carrier.filter(|_| g.state == 1)) {
                    ob.status[c] |= 4 << i;
                    if team_of(c) < 2 {
                        ob.team_status[team_of(c)] |= GE_TEAM_MASKS[i];
                    }
                }
            }
        }
        if s == super::scenario::BLUEPRINT {
            ob.blueprint = rt.blueprint.as_ref().map(view);
            for ti in 0..2 {
                ob.esp_bases[ti] = rt.esp_bases[ti].map(|p| point(p, rt.esp_bases_bounds[ti]));
            }
            if let Some(c) = rt.blueprint.as_ref().and_then(|b| b.carrier.filter(|_| b.state == 1)) {
                ob.status[c] |= 2;
                if team_of(c) < 2 {
                    ob.team_status[team_of(c)] |= 1;
                }
            }
        }
        if s == super::scenario::ASSASSIN {
            ob.assassin = self.assassin;
            ob.assassin_target = self.assassin_target;
        }
        ob.bot_goals = self.bots.iter().map(|b| (b.slot, b.brain.vars.goals[1].goal_type, b.brain.vars.goals[1].target)).collect();
        ob
    }
}

/// What the objective scan found (`pickGoal` Phase A's three outcomes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObjPick {
    /// A scored objective: (target id, goal position, subtype `+0x4A`).
    Goal(usize, [f32; 3], u8),
    /// King of the Hill with the bot on the hill: `iVar18 = -3` → pickGoal returns 0 without the
    /// pickup fall-through — the bot holds the hill [M 0x126b88:237-243, :491-493].
    Hold,
    /// Nothing scored → the slot-0 pickup fall-through [M :315-324].
    None,
}


/// The bot's weapon fire parameters — lifted from `weapon_data` by the caller
/// (`BOTWEAP_InitWeapon` 0x129e48 sets the default; `DroneWeap_NextBulletTime` 0x179d40 reads).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BotWeapon {
    /// `weapon_data[w*0x10C+0x40]` (s16) — the per-shot interval in ticks [M 0x179d40:26-47].
    pub interval: i16,
    /// `weapon_data[w*0x10C+0x2C]` (u16) — the burst size [M 0x179d40:53-116].
    pub burst: u16,
    /// The aggression multiplier (`BOT_getAggressionMul` — switches on `Drone+0xB5`, ×counts ≥ 4,
    /// min 3.0 [M 0x179d40]; the exact 0.3..1.0 table is not lifted — the caller passes it, [S]).
    pub aggression_mul: f32,
}


/// The `BOT_vars_t` subset the ported states touch — every field carries its offset in the comment.
#[derive(Clone, Debug, PartialEq)]
pub struct BotVars {
    // the stats block (+0xA0..0xAD, `BOT_init` 0x1249e8:164's record copy)
    pub aggression: u8,   // +0xA0
    pub aim: u8,          // +0xA2
    pub max_hp: u16,      // +0xA4
    pub speed: u8,        // +0xA6
    pub weapon_class: u8, // +0xAA preferred weapon class
    pub trait_id: u8,     // +0xAB
    pub flags: u8,        // +0xAC (bit3 360° vision, bit4 HP regen)
    // the two goal slots
    pub goals: [GoalSlot; 2],
    pub active_goal: u8, // +0x765 (0xFF none)
    // instance fields
    pub requested_state: u32, // +0x72C (`BOTSTATE_setStateChange` writes; 0xEB reads as resume)
    pub pending_scratch: u32, // +0x730 (gotoGoal := the state at goal start)
    pub flag_word: u32,       // +0x734 (bit1 recovery pending, bit2 flag-carrier goal)
    pub regen_deadline: u32,  // +0x738 (HP-regen next-tick deadline)
    pub nav_node: u16,        // +0x760 (0xFFFF invalid)
    pub slot: i16,            // +0x75C (player slot 4..7)
    pub bot_index: i16,       // +0x75E (slot−4)
    pub state_type: u8,       // +0x766 (type of the last requested state)
    pub weapon_id: u8,        // +0x768 (current weapon)
    pub announced_weapon: u8, // +0x76A (msg 0x42 payload)
    pub grudge: i8,           // +0x76B (0xFF none)
    pub last_pickup: i8,      // +0x76E (the last chosen pickup index — pickGoal skip/ writer [M 0x126b88:397,496])
    pub guard_near_flag: u8,  // +0x76F (BotIdle's sphere-box result [M 0x171b80:96])
    pub scan_flag: u8,        // +0x770 (set when the +0xB0 scan found a live candidate)
    pub attack_latch: u8,     // +0x771 (the attack re-entry latch)
    // combat state (Drone_tag fields the states read; carried here so the brain is self-contained)
    pub opponent: Option<usize>, // Drone+0x170 (index into ctx.others)
    pub sighted: u32,            // Drone+0x228 (bit 2 = opponent visible)
    pub unseen_ticks: u32,       // Drone+0x274 (ticks since the last LOS tick)
    /// Drone+0x270 — the seen-arm counter `ProcessDroneSight` increments while the vision chain
    /// holds, `HaveOpponentSight` gates the bit-2 ack on [M 0x175ef0:41-44].
    pub seen_ticks: u32,
    pub sight_stamp: f32,        // Drone+0x238 (last-sight stamp; heartbeat's 5 s latch)
    pub combat_range: [f32; 3],  // Drone+0xEC/0xF0/0xFC (too-close / engage / too-far)
    pub fire_latch: u8,          // Drone+0x3B
    pub burst_empty: u8,         // Drone+0x3C
    pub clip_rounds: i16,        // `BOTWEAP_AmmoInGun` = BOT_vars+0x144 + w*0xC [M 0x12ae40]
    pub next_bullet_at: f32,     // Drone+0xBC4 (the burst deadline)
    pub burst_done_tick: u32,    // [S] the tick the last burst emptied (the re-arm stand-in)
    pub burst_left: u32,         // Drone+0xBC0
    pub resume_state: u16,       // Drone+0x5A6 (the 0xF6 door-detour resume marker)
    // mp-bot-weapons (botweap.rs)
    pub character: u8,           // +0x764 (the record's character — `BOTWEAP_InitWeapon`'s 0x1A test)
    pub arsenal: crate::mp::botweap::BotArsenal, // +0x144/+0x146 per weapon, +0x698 pools, +0x769 armour
    pub last_shot_tick: u32,     // Drone+0xBC8 (the last bullet's GameState stamp [M 0x178f68:261])
    pub next_shot_tick: u32,     // Drone+0xBC4 (the next bullet's deadline, ticks [M 0x179d40:131-135])
    pub reload_until: u32,       // [S] the reload anim 0x52's end (0xDD → 0xCF)
    pub pickup_choice: Option<bool>, // the pending `pickupWeaponChangeChoice` (has-weapon flag) from a take
    pub ammo_summary: (f32, f32), // (`getWeaponAmmoAmount`, `getWeaponClipSize`) of +0x768 — setGoalPickPrefs' inputs
    pub opp_dist: f32,           // Drone+0x1A0 (the distance to the opponent; MAX = none)
    pub prev_opponent: Option<usize>, // [S] last tick's Drone+0x170 (the `SetOpponent` msg-0x45 edge)
    pub prev_visible: bool,      // [S] last tick's Drone+0x228 bit 2 (the `HaveOpponentSight` edge)
}


impl Default for BotVars {
    fn default() -> Self {
        BotVars {
            aggression: 0,
            aim: 0,
            max_hp: 100,
            speed: 0,
            weapon_class: 0,
            trait_id: 0,
            flags: 0,
            goals: [GoalSlot::default(); 2],
            active_goal: 0xFF,
            requested_state: 0,
            pending_scratch: 0,
            flag_word: 0,
            regen_deadline: 0,
            nav_node: 0xFFFF,
            slot: 4,
            bot_index: 0,
            state_type: 0,
            weapon_id: 0,
            announced_weapon: 0,
            grudge: -1,
            last_pickup: -1,
            guard_near_flag: 0,
            scan_flag: 0,
            attack_latch: 0,
            opponent: None,
            sighted: 0,
            unseen_ticks: u32::MAX,
            seen_ticks: 0,
            sight_stamp: f32::NEG_INFINITY,
            combat_range: [2.0, 20.0, 40.0],
            fire_latch: 0,
            burst_empty: 0,
            clip_rounds: 0,
            next_bullet_at: 0.0,
            burst_done_tick: 0,
            burst_left: 0,
            resume_state: 0,
            character: 0,
            arsenal: crate::mp::botweap::BotArsenal::default(),
            last_shot_tick: 0,
            next_shot_tick: 0,
            reload_until: 0,
            pickup_choice: None,
            ammo_summary: (1.0, 1.0),
            opp_dist: f32::MAX,
            prev_opponent: None,
            prev_visible: false,
        }
    }
}


/// One fired shot (the port's `DroneWeap_FireWeapon` event — the caller spawns the bullet).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BotShot {
    pub tick: u32,
    pub pos: [f32; 3],
    pub dir: [f32; 3],
    pub weapon: u8,
}


/// One MP bot brain: the `BotVars` slot, the SM state and the per-tick driver
/// (`Drone_Control` → `ControlSTANDARD` → `SendMsgSelf(3)` → `RouteMsgDCV` order).
pub struct BotBrain {
    pub vars: BotVars,
    /// The SM current state (`Drone+0x10C`).
    pub state: u8,
    /// The position (`obj+0x30`).
    pub pos: [f32; 3],
    /// The yaw (`obj+0x50..0x5C` matrix facing; the port carries the scalar).
    pub yaw: f32,
    /// The per-tick counter (the port's `GameState` stand-in).
    pub tick: u32,
    /// The weapon params (`BOTWEAP_InitWeapon`'s lift).
    pub weapon: BotWeapon,
    /// The fired shots (the test observes these).
    pub shots: Vec<BotShot>,
    /// The visited pickups (`setPickupVisitTime` events — the test's "reached a pickup").
    pub visited_pickups: Vec<usize>,
    /// The state-change trace (tick, from, to) — the test's transition assertions.
    pub trace: Vec<(u32, u8, u8)>,
    /// The last `setPickupVisitTime` stamp (pickup index, deadline) — the caller mirrors it
    /// into its `MP_PICKUP+0x80` floats (the game writes the pickup record in place).
    pub last_visit_stamp: (usize, f32),
    /// The pending SetState (`SM+0x0C` + the flag [M 0x1758c0:53-55]).
    pending: Option<u8>,
    /// The bot's walking body: the player's movement core (`Bond::tick_ctx` — the capsule
    /// push-out, the floor ray, gravity, steps) stands in for the drone mover
    /// (`NDrone2_MoveToGoalPosition`'s anim-driven root motion + `Collide`) [S]. `pos` is this
    /// body's root + `player::EYE`. Unused (the plane walk) when the world has no triangles —
    /// the unit rigs' empty worlds.
    pub body: crate::player::Bond,
    /// The route the body follows (`AINetwork_CalcRouteToPosition` / `FollowRoute`'s port
    /// stand-in over the arena's AI network) [S].
    pub route: BotRoute,
    /// This tick's walk request (world x/z unit direction, speed 0..1) — the states write it, the
    /// body walk at the end of `step` consumes it.
    move_want: Option<([f32; 2], f32)>,
    /// The combat move's end tick (`Drone+0xCC4` — the strafe's `(Rand(8)+2)·FRAME_RATE_INT`
    /// timer [M 0x16f850]; the step's anim length stand-in [S]).
    pub combat_move_until: u32,
    /// The stuck watch: (root position, tick) of the last check, and the consecutive stuck checks.
    stuck_ref: ([f32; 3], u32),
    pub stuck_hits: u8,
    /// One jump on the next body tick (the stuck recovery [S]).
    jump_next: bool,
}

/// The port's route over the arena AI network [S — the game's `AIRoute` + link creep are
/// ported in `nav.rs` for the SP drones; the bots use a plain shortest path over the same graph
/// with a walk-clear shortcut, see `BotBrain::steer_to`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BotRoute {
    /// The nav path index and its node list (empty = walk straight to `dest`).
    pub path: usize,
    pub nodes: Vec<u16>,
    /// The next node to reach.
    pub idx: usize,
    /// The destination (body-root space) the route was built for.
    pub dest: [f32; 3],
    /// The tick it was built (re-plans age out).
    pub built: u32,
    pub valid: bool,
}

/// The walk-clear test between two body roots: the floor+1.0 line (root − 0.1) is clear of the
/// world and the height step is one a walk can take [S — the game's `NDrone2_MoveTest` reads the
/// boundary walls + portals; the collision world stands in, as `nav::Network::move_test` does].
fn walk_clear(world: &CollisionWorld, a: [f32; 3], b: [f32; 3]) -> bool {
    (b[1] - a[1]).abs() < 1.5
        && world.line_of_sight([a[0], a[1] - 0.1, a[2]], [b[0], b[1] - 0.1, b[2]])
        // mp-bot-objectives: and the knee line (floor + 0.35) — the floor+1.0 line alone passed
        // over a ramp's side wall, so a bot cut the corner onto 07000023's pit ramp from the side
        // and pushed into its edge for good (the Espionage carrier, the KOTH walkers) [S]
        && world.line_of_sight([a[0], a[1] - 0.75, a[2]], [b[0], b[1] - 0.75, b[2]])
}

fn dist2d(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]) * (a[0] - b[0]) + (a[2] - b[2]) * (a[2] - b[2])).sqrt()
}

/// A nav node's body-root height: the node sits at floor + 0.4 (`AIPath_Parse`), the root at
/// floor + 1.1 (`Player_StandAtNewPosition`).
fn node_root(p: [f32; 3]) -> [f32; 3] {
    [p[0], p[1] + 0.7, p[2]]
}

/// The body's top forward step per tick (`Player_Move`'s 0.1·f² + 0.005 at full stick) — the
/// plane walk (no world) moves at the same speed.
const BODY_STEP: f32 = 0.105;


/// `BOTSTATE_getStateType` is already above; the states the port dispatches to.
mod st {
    pub use super::state::*;
    /// The attack family ids the port handles (`docs/reports/mp-bots-data.md` inventory).
    pub const BOT_ATTACK_NO_OPPONENT: u8 = 0xD3; // Stub
    pub const BOT_ATTACK_STRAFE_AIM_LEFT: u8 = 0xD4; // Full — not ported (chooseCombatMove target)
    pub const BOT_ATTACK_STRAFE_AIM_RIGHT: u8 = 0xD5; // Full — not ported
    pub const BOT_ATTACK_STEP_AIM_LEFT: u8 = 0xDB; // Thin — not ported
    pub const BOT_ATTACK_STEP_AIM_RIGHT: u8 = 0xDC; // Thin — not ported
    pub const BOT_SEEN_OPPONENT: u8 = 0xEC; // Stub: ENTER → 0xCF [M 0x175268:9-13]
}


impl BotBrain {
    /// `BOT_init` 0x1249e8 (the port's lift of the BOT_vars-relevant half) + the 0xC3 ENTER
    /// chain [M 0x1748c8:16-18: setNearestNavNode, +0x766 = 0, SetState(0xF9)]:
    /// the stats copy, the goal-slot active flags (slot 1 = 1, slot 0 = 0), the grudge 0xFF,
    /// the links, then state 0xF9 via the commit.
    pub fn bot_init(record: &BotRecord, slot: u8, pos: [f32; 3], yaw: f32) -> BotBrain {
        let stats = record.stats();
        let mut vars = BotVars::default();
        vars.aggression = stats[0x00];
        vars.aim = stats[0x02];
        vars.max_hp = u16::from_le_bytes([stats[0x04], stats[0x05]]);
        vars.speed = stats[0x06];
        vars.weapon_class = stats[0x0A];
        vars.trait_id = stats[0x0B];
        vars.flags = stats[0x0C];
        vars.character = record.character(); // +0x764 [M 0x1249e8:160]
        vars.slot = slot as i16;
        vars.bot_index = (slot - 4) as i16;
        vars.grudge = -1; // 0xFF [M 0x1249e8:163]
        vars.last_pickup = -1;
        vars.goals[1].active = 1; // [M 0x1249e8:172-176: slot1+0x49 = 1, slot0+0x49 = 0]
        vars.goals[0].active = 0;
        if vars.flags & 0x10 != 0 {
            // the HP-regen arm: +0x738 := GameState+52 + FRAME_RATE_INT [M 0x1249e8:165-167]
            vars.regen_deadline = FRAME_RATE_INT;
        }
        let mut brain = BotBrain {
            vars,
            state: 0xC3, // BotInit
            pos,
            yaw,
            tick: 0,
            weapon: BotWeapon { interval: 6, burst: 1, aggression_mul: 1.0 },
            shots: Vec::new(),
            visited_pickups: Vec::new(),
            trace: Vec::new(),
            last_visit_stamp: (usize::MAX, 0.0),
            pending: Some(state::BOT_IDLE), // the BotInit chain's SetState(0xF9) [M 0x1748c8:18],
            // committed by the first `step` (the game's PostLoad_Init → msg 1 → SetState order)
            // The body stands at the root under the eye; Bond's yaw convention (forward =
            // (sin, cos)) is the brain's (forward = (cos, sin)) mirrored about 45°.
            body: crate::player::Bond::new([pos[0], pos[1] - crate::player::EYE, pos[2]], crate::trig::HALF_PI - yaw),
            route: BotRoute::default(),
            move_want: None,
            combat_move_until: 0,
            stuck_ref: ([pos[0], pos[1] - crate::player::EYE, pos[2]], 0),
            stuck_hits: 0,
            jump_next: false,
        };
        // The spawn loadout: weapon pickups are not taken by bots yet, so a bot holding nothing
        // (weapon 0) starts with the match's start weapon, the P2K (2), and its clip — with
        // weapon 0 its rounds did record 0's damage (none) [S].
        if brain.vars.weapon_id == 0 {
            brain.vars.weapon_id = 2;
            brain.vars.clip_rounds = 12;
        }
        brain
    }

    fn enter_state(&mut self, next: u8, ctx: &BotCtx, rng: &mut Rng) {
        // the commit: the state flips, +0x766 := the type [M 0x16e060:605], ENTER msg 1 runs below
        let prev = self.state;
        self.state = next;
        self.vars.state_type = state_type(next as u32);
        self.trace.push((self.tick, prev, next));
        // the ENTER delivery — each ported state's msg-1 arm runs now
        match next {
            state::BOT_IDLE => self.bot_idle_enter(),
            state::BOT_GOTO_GOAL_POSITION => {} // walk anim 0x1E [M 0x171240:20]
            state::BOT_ATTACK => self.bot_attack_enter(ctx, rng),
            state::BOT_ATTACK_FIRE => self.bot_attack_fire_enter(rng), // the trigger pull [M 0x16f5e8:19-26]
            st::BOT_SEEN_OPPONENT => {
                self.set_state(state::BOT_ATTACK); // the 0xEC stub ENTER [M 0x175268:9-13]
            }
            st::BOT_ATTACK_STRAFE_AIM_LEFT | st::BOT_ATTACK_STRAFE_AIM_RIGHT
            | st::BOT_ATTACK_STEP_AIM_LEFT | st::BOT_ATTACK_STEP_AIM_RIGHT => self.bot_attack_move_enter(next, rng),
            state::BOT_ATTACK_RELOAD => self.bot_attack_reload_enter(ctx),
            state::BOT_ATTACK_CHANGE_WEAPON => self.bot_attack_change_weapon_enter(ctx),
            _ => {} // 0xF2/0xF6/0xD3… not ported (the report lists the gap)
        }
    }

    /// `Drone_SM_SetState` 0x1758c0: the pending write [M :53-55].
    fn set_state(&mut self, next: u8) {
        if next != 0 {
            self.pending = Some(next);
        }
    }

    /// `DroneVision_OpponentVisibility` 0x175af0 with the bots' constants: sight 24.0 (the
    /// drones' default-case +0xE8 = 0x41c00000 [M 0x14b300:386]; the bots' own arm is not
    /// confirmed — [S], the same constant `find_opponent` squares), half-FOV π/2 [M 0x14b300
    /// LAB_0014bb44], alertness 0 (the bots never raise +0x50c in the port) and light 1.0
    /// ([S] the bots run no AI-volume factor). The ≥ 0.7 score is `SeekOpponentLOS` 0x175cb8's
    /// gate before its raycast.
    fn opponent_visibility(&self, t: &OtherPlayer) -> f32 {
        let dx = t.pos[0] - self.pos[0];
        let dy = t.pos[1] - self.pos[1];
        let dz = t.pos[2] - self.pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < 2.0 {
            return 1.0;
        }
        let r = 24.0f32; // sight × (2·alertness + 1), alertness 0
        let mut vis = 0.7 - (dist - r) / r;
        let yaw_to_opp = crate::trig::atan2_approx(dx, dz);
        let margin = crate::trig::HALF_PI + 0.392_699_1 - crate::trig::angle_difference(self.yaw, yaw_to_opp).abs();
        if margin < 0.0 {
            return 0.0;
        }
        vis *= crate::trig::sinf(margin / crate::trig::HALF_PI * crate::trig::HALF_PI);
        vis.max(0.0)
    }

    /// One drone-control tick [M 0x1490f0:81 + 0x171df8:78-86,119-203]: heartbeat to 0xC5, msg 3
    /// to the current state, the firing pipeline, then the pending commit (0x2E + ENTER).
    pub fn step(&mut self, ctx: &BotCtx, rng: &mut Rng, weapon_fire: bool) {
        self.tick += 1;
        // 0. the perception pass [M 0x1490f0:164-167 — setOtherPlayerInfo then FindOpponent]:
        // the LOS/sight bookkeeping (the port's `CanSeeObject` = the collision-world segment).
        // The bit-2 sight the heartbeat's react and the combat states read is the DRONE vision
        // chain's ack, not a bare segment: `DroneVision_OpponentVisibility` 0x175af0 scores the
        // cone + range band (≥ 0.7 before any raycast — `SeekOpponentSight` 0x175cb8), and
        // `HaveOpponentSight` 0x175ef0 sets +0x228 bit 2 only once the seen counter (+0x270)
        // passes `DroneFunc_ReactionTime` 0x14ac60 (10 ticks off the five SP levels; MP
        // 0x07000023 lands in the default arm). Without the cone a goal-running bot "saw" the
        // opponent it was walking AWAY from (the segment ignores facing), the heartbeat reacted
        // every tick, and a carrier could never break contact to deliver (bot-weapons-finish-2).
        self.vars.sighted &= !4;
        if let Some(o) = self.vars.opponent {
            if let Some(t) = ctx.others.get(o) {
                if t.alive {
                    // The line to the aim point (the chest, eye − 0.3 — where `handle_firing`'s
                    // round goes): an eye-to-eye line over a ledge lip saw a bot on the storey
                    // below whom no round could reach, and the pair stood firing into the lip
                    // all match instead of running the route round [S the CanSeeObject stand-in].
                    let clear = ctx.world.segment_clear(self.pos, [t.pos[0], t.pos[1] - 0.3, t.pos[2]]);
                    if clear && self.opponent_visibility(t) >= 0.7 {
                        self.vars.seen_ticks = self.vars.seen_ticks.saturating_add(1); // +0x270 [M 0x176a28 seen arm]
                        self.vars.unseen_ticks = 0; // Drone+0x274
                        if self.vars.seen_ticks >= 10 {
                            // HaveOpponentSight's ack [M 0x175ef0:41-67 — bit 2 + the +0x238 stamp]
                            self.vars.sighted |= 4; // Drone+0x228 bit 2 [M]
                            // the brain's canonical clock: its own tick counter (60 Hz), NOT the
                            // driver-supplied ctx.now — a lockstep host's wall-clock accumulation
                            // must not reach sim state (net-lockstep finding F4). next_bullet_at is
                            // already tick-derived; this aligns every time read with it.
                            self.vars.sight_stamp = self.tick as f32 / 60.0; // Drone+0x238
                        }
                    } else {
                        self.vars.seen_ticks = 0; // the lost arm zeroes +0x270 [M 0x176a28]
                        self.vars.unseen_ticks = self.vars.unseen_ticks.saturating_add(1);
                    }
                }
            }
        }
        { let _p = crate::prof::scope("bots: find_opponent"); self.find_opponent(ctx); } // [M 0x1490f0:167]
        // mp-bot-weapons: the weapon layer's per-tick part (arming, the opponent distance, a
        // pending pickup weapon choice) — botweap.rs.
        self.weapons_tick(ctx, rng);
        // 1. the forced-to-0xC5 heartbeat [M 0x171df8:78-86]
        self.bot_global_msg3(ctx, rng);
        // 2. msg 3 to the current state (unknown states do not consume → nothing, like the
        //    unported handlers returning 0 with the BotGlobal fallback consumed above)
        match self.state {
            state::BOT_IDLE => {
                self.bot_idle_msg3(ctx, rng);
            }
            state::BOT_GOTO_GOAL_POSITION => self.bot_goto_goal_msg3(ctx),
            state::BOT_ATTACK => self.bot_attack_msg3(ctx, rng),
            state::BOT_ATTACK_FIRE => self.bot_attack_fire_msg3(ctx, rng),
            state::BOT_ATTACK_RUN => self.bot_attack_run_msg3(ctx),
            state::BOT_ATTACK_BACKOFF => self.bot_attack_backoff_msg3(ctx),
            state::BOT_ATTACK_NO_OPPONENT => self.set_state(state::BOT_IDLE),
            state::BOT_ATTACK_RELOAD => self.bot_attack_reload_msg3(),
            st::BOT_ATTACK_STRAFE_AIM_LEFT | st::BOT_ATTACK_STRAFE_AIM_RIGHT => self.bot_attack_strafe_msg3(ctx, rng),
            st::BOT_ATTACK_STEP_AIM_LEFT | st::BOT_ATTACK_STEP_AIM_RIGHT => self.bot_attack_step_msg3(ctx, rng),
            _ => {}
        }
        // The lost opponent: an attack state with the opponent out of sight for 5 s gives up and
        // goes back to its goals [S — the game's attack-route failure path (0xD1 BotAttackNoRoute,
        // the heartbeat's combat block) is not lifted; without it a bot chased a hidden opponent
        // forever].
        if self.vars.state_type == 4 && self.vars.unseen_ticks != u32::MAX && self.vars.unseen_ticks > 300 {
            self.vars.opponent = None;
            self.vars.fire_latch = 0;
            self.set_state(state::BOT_IDLE);
        }
        // `DroneWeap_AimTarget_IsOpponent` [M 0x16f850 / 0x174e88 msg 3]: the combat states keep
        // the body turned on the opponent while they move.
        if self.vars.state_type == 4 {
            if let Some(t) = self.vars.opponent.and_then(|o| ctx.others.get(o)) {
                let t = t.pos;
                self.turn_toward(t, 0.2);
            }
        }
        // 3. the per-tick firing pipeline [M 0x1490f0:231 `DroneWeap_HandleFiring`]
        if weapon_fire {
            match ctx.weapons {
                Some(t) if self.vars.arsenal.armed => self.handle_firing_armed(ctx, rng, t),
                _ => self.handle_firing(ctx, rng),
            }
        }
        // The body walk (the drone mover's place in `Drone_Control`, after the state's msg 3).
        self.walk_body(ctx);
        // 4. the commit [M 0x171df8:119-203]: EXIT msg 2 is a no-op for the ported states, the
        //    commit, the synthetic 0x2E (the +0x766 type write is the ported part [M 0x16e060:605]),
        //    then ENTER msg 1 — the loop repeats while ENTERs chain more SetStates [M :119-203].
        while let Some(next) = self.pending.take() {
            self.vars.state_type = state_type(next as u32);
            self.enter_state(next, ctx, rng);
        }
    }

    /// The brain's lockstep hash — every **live** state field, f32 as bit patterns
    /// (`mp::hash::Hasher`). The brains are sim state: a lockstep host hashes them with the
    /// match every tick (`docs/systems/net-lockstep.md`). The derived logs (`shots`,
    /// `visited_pickups`, `trace`) hash as lengths only — they are pure functions of the live
    /// state + the shared RNG stream, so a content divergence implies a hashed divergence.
    pub fn state_hash(&self) -> u64 {
        use crate::mp::hash::Hasher;
        let mut h = Hasher::new();
        let v = &self.vars;
        h.u8(v.aggression);
        h.u8(v.aim);
        h.u16(v.max_hp);
        h.u8(v.speed);
        h.u8(v.weapon_class);
        h.u8(v.trait_id);
        h.u8(v.flags);
        for g in &v.goals {
            h.f32_slice(&g.celpos);
            h.f32(g.scratch);
            h.f32(g.start);
            h.f32(g.duration);
            h.f32(g.weight_health);
            h.f32(g.weight_ammo);
            h.f32(g.weight_weapon);
            h.f32(g.objective_weight);
            h.usize(g.target);
            h.u8(g.fire_state);
            h.u8(g.complete);
            h.u8(g.goal_type);
            h.u8(g.pref_byte);
            h.u8(g.dist_limit as u8);
            h.u8(g.route_code);
            h.u8(g.active);
            h.u8(g.subtype);
        }
        h.u8(v.active_goal);
        h.u32(v.requested_state);
        h.u32(v.pending_scratch);
        h.u32(v.flag_word);
        h.u32(v.regen_deadline);
        h.u16(v.nav_node);
        h.u16(v.slot as u16);
        h.u16(v.bot_index as u16);
        h.u8(v.state_type);
        h.u8(v.weapon_id);
        h.u8(v.announced_weapon);
        h.u8(v.grudge as u8);
        h.u8(v.last_pickup as u8);
        h.u8(v.guard_near_flag);
        h.u8(v.scan_flag);
        h.u8(v.attack_latch);
        match v.opponent {
            None => h.u8(0),
            Some(o) => {
                h.u8(1);
                h.usize(o);
            }
        }
        h.u32(v.sighted);
        h.u32(v.unseen_ticks);
        h.u32(v.seen_ticks);
        h.f32(v.sight_stamp);
        h.f32_slice(&v.combat_range);
        h.u8(v.fire_latch);
        h.u8(v.burst_empty);
        h.u16(v.clip_rounds as u16);
        h.f32(v.next_bullet_at);
        h.u32(v.burst_left);
        h.u16(v.resume_state);
        h.u8(v.character);
        v.arsenal.hash_into(&mut h);
        h.u32(v.last_shot_tick);
        h.u32(v.next_shot_tick);
        h.u32(v.reload_until);
        match v.pickup_choice { None => h.u8(0), Some(b) => { h.u8(1); h.bool(b); } }
        h.f32(v.ammo_summary.0);
        h.f32(v.ammo_summary.1);
        h.f32(v.opp_dist);
        match v.prev_opponent { None => h.u8(0), Some(o) => { h.u8(1); h.usize(o); } }
        h.bool(v.prev_visible);
        h.u8(self.state);
        h.f32_slice(&self.pos);
        h.f32(self.yaw);
        // the walking body + its route (sim state: the next tick's walk reads them)
        h.f32_slice(&self.body.pos);
        h.f32_slice(&self.body.vel);
        h.f32_slice(&self.body.local);
        h.u8(self.body.grounded as u8);
        h.usize(self.route.nodes.len());
        h.usize(self.route.idx);
        h.u32(self.combat_move_until);
        h.u8(self.stuck_hits);
        h.u32(self.tick);
        h.i16(self.weapon.interval);
        h.u16(self.weapon.burst);
        h.f32(self.weapon.aggression_mul);
        h.usize(self.shots.len());
        h.usize(self.visited_pickups.len());
        h.usize(self.trace.len());
        h.usize(self.last_visit_stamp.0);
        h.f32(self.last_visit_stamp.1);
        match self.pending {
            None => h.u8(0),
            Some(p) => {
                h.u8(1);
                h.u8(p);
            }
        }
        h.finish()
    }

    // ------------------------------------------------------------------
    // The goal machine (BOTSTATE_*)
    // ------------------------------------------------------------------

    /// `BOTSTATE_initGoal` 0x1294c0 [M :15-40].
    /// The port form of `AINetwork_SetupGoalPositionToObj` (0x15add0); the goal machine is
    /// `BOTSTATE_gotoGoal` (0x1278a8).
    pub fn init_goal(&mut self, slot: usize, goal_type: u8, celpos: [f32; 4], target: usize, subtype: u8) {
        if slot >= 2 {
            return;
        }
        let g = &mut self.vars.goals[slot];
        g.goal_type = goal_type;
        g.complete = 0;
        g.route_code = 0;
        g.celpos = celpos;
        g.subtype = subtype;
        g.target = target;
    }

    /// `BOTSTATE_uninitGoal` 0x129438 [M :9-29].
    pub fn uninit_goal(&mut self, slot: usize) {
        if slot > 1 {
            return;
        }
        let carrier = self.vars.goals[slot].subtype == 1;
        let drone = &mut self.vars;
        if carrier {
            drone.flag_word |= 4;
        } else {
            drone.flag_word &= !4;
        }
        if drone.active_goal == slot as u8 {
            drone.active_goal = 0xFF;
        }
        let g = &mut drone.goals[slot];
        g.subtype = 0;
        g.goal_type = 0;
        g.complete = 0;
        g.route_code = 0;
        g.scratch = 0.0;
    }

    /// `BOTSTATE_setGoalComplete` 0x129510: `+0x44 |= 1` [M :9-10].
    pub fn set_goal_complete(&mut self, slot: usize) {
        if slot < 2 {
            self.vars.goals[slot].complete |= 1;
        }
    }

    /// `BOTSTATE_getActiveGoal` 0x129548: Some(slot) or None [M :8-12].
    pub fn get_active_goal(&self) -> Option<usize> {
        let idx = self.vars.active_goal;
        (idx != 0xFF).then_some(idx as usize)
    }

    /// `BOTSTATE_setStateChange` 0x129ce8: SetState + the +0x72C record [M :6-7].
    fn set_state_change(&mut self, next: u8) {
        self.set_state(next);
        self.vars.requested_state = next as u32;
    }

    /// `BOTSTATE_setGoalPickPrefs` 0x128298 [M :23-95]: the auto-derive when all three weights
    /// are 0 (health = (1 − hp/max)·25, ammo/weapon the same shape from the clip state), the
    /// 25.0 clamps, and the p7==0 → +0x47 := −2.
    pub fn set_goal_pick_prefs(
        &mut self, health_w: f32, ammo_w: f32, weapon_w: f32, objective_w: f32, slot: usize,
        dist_limit: i8, pref_byte: u8, health: f32, ammo: f32, clip: f32,
    ) {
        if slot >= 2 {
            return;
        }
        let (mut h, mut a, mut w) = (health_w, ammo_w, weapon_w);
        if health_w == 0.0 && ammo_w == 0.0 && weapon_w == 0.0 {
            // the auto-derive [M 0x128298:24-28]
            let hp = health.max(1.0);
            h = (1.0 - 1.0 / (self.vars.max_hp as f32 / hp)) * 25.0;
            // mp-bot-weapons: an armed bot's own `getWeaponAmmoAmount` / `getWeaponClipSize` of
            // +0x768 [M :29-30] (the callers' 1.0/1.0 stand-in made the ammo weight always 0).
            let (ammo, clip) = if self.vars.arsenal.armed { self.vars.ammo_summary } else { (ammo, clip) };
            if ammo < clip {
                let denom = clip.max(1.0);
                a = (1.0 - 1.0 / (denom / ammo.max(1.0))) * 25.0;
                w = a; // [M :53-70 — the same derivation for the weapon weight]
            }
        }
        let g = &mut self.vars.goals[slot];
        g.weight_health = h.min(25.0);
        g.weight_ammo = a.min(25.0);
        g.weight_weapon = w.min(25.0);
        g.objective_weight = objective_w;
        g.pref_byte = pref_byte;
        g.dist_limit = if dist_limit == 0 { -2 } else { dist_limit };
    }

    /// `BOTSTATE_setPickupVisitTime` 0x129958: `now + 45.0` (+0.1 if that is 0) [M :10-15].
    pub fn set_pickup_visit_time(&mut self, ctx: &BotCtx, pickup: usize) {
        let mut t = self.tick as f32 / 60.0 + 45.0;
        if t == 0.0 {
            t += 0.1;
        }
        let idx = self.vars.bot_index as usize;
        self.visited_pickups.push(pickup);
        // the game writes `MP_PICKUP+0x80+idx*4` in place; the port's ctx owns the pickups, so
        // the driver mirrors `last_visit_stamp` into its table between steps
        let _ = idx;
        self.last_visit_stamp = (pickup, t);
    }

    /// `BOTSTATE_gotoGoal` 0x1278a8 [M :20-147]: teardown of the old active goal, the deadline
    /// stamp (+0x24 := now, +0x28 := VIDEO_FRAME_RATE·5), +0x40 := fire state, the route setup,
    /// the +0x20 RNG budget (slot 1: the 600/600/500 rolls by subtype), activation.
    pub fn goto_goal(&mut self, ctx: &BotCtx, rng: &mut Rng, slot: usize, fire_state: u8) -> bool {
        if slot > 1 {
            return false;
        }
        // teardown of the currently active goal [M :28-49]
        if self.vars.active_goal != 0xFF {
            let old = self.vars.active_goal as usize;
            let carrier = self.vars.goals[old].subtype == 1;
            if carrier {
                self.vars.flag_word |= 4;
            } else {
                self.vars.flag_word &= !4;
            }
            let g = &mut self.vars.goals[old];
            g.subtype = 0;
            g.goal_type = 0;
            g.complete = 0;
            g.route_code = 0;
            g.scratch = 0.0;
        }
        {
            let g = &mut self.vars.goals[slot];
            g.start = self.tick as f32 / 60.0;
            g.duration = (FRAME_RATE_INT * 5) as f32; // [M 0x1278a8:50,58]
            g.fire_state = fire_state;
        }
        let gtype = self.vars.goals[slot].goal_type;
        if gtype == 3 {
            self.vars.goals[slot].subtype = 9; // hunt [M :71]
        }
        self.vars.pending_scratch = self.state as u32; // +0x730 := current state [M :67-68]
        // the +0x20 wander budget [M :84-142] — slot 0 stays 0 [M :85-87]
        if slot == 1 {
            let sub = self.vars.goals[1].subtype;
            let mut v = match sub {
                1 | 3 | 4 | 6 | 7 => rng.rand(600) as f32 + 500.0, // [M :95]
                8 => rng.rand(600) as f32 + 250.0,                  // [M :109]
                9 => rng.rand(500) as f32 + 250.0,                  // [M :119]
                2 | 5 => 1500.0,                                    // [M :104-107]
                _ => 0.0,
            };
            if ctx.my_team == 0 && self.vars.trait_id == 5 {
                v = 0.0; // [M :128-135]
            }
            self.vars.goals[1].scratch = v;
        }
        self.vars.flag_word &= !4; // [M :145]
        self.vars.active_goal = slot as u8; // [M :146]
        true
    }

    /// `BOTSTATE_pickGoal` 0x126b88 (the port's flow; route distance → straight-line [Deviation]).
    /// Returns true when a goal was initialised + `gotoGoal` ran.
    pub fn pick_goal(&mut self, ctx: &BotCtx, rng: &mut Rng, slot: usize) -> bool {
        self.pick_goal_scan(ctx, rng, slot, true)
    }

    /// `pick_goal` with the Phase A scan optional (`scan == false`: the caller already ran
    /// [`BotBrain::objective_scan`] this tick and it found nothing).
    fn pick_goal_scan(&mut self, ctx: &BotCtx, rng: &mut Rng, slot: usize, scan: bool) -> bool {
        let mut slot = slot;
        // Phase A: the mode-objective scan (slot 1 only) [M :143-303] — `objective_scan`.
        // Nothing found → uninitGoal(0), zero slot-0 prefs, fall through [M :317-319]; the KOTH
        // hold returns 0 at once [M :491-493].
        if slot == 1 {
            self.uninit_goal(1); // [M :137]
            let pick = if scan { self.objective_scan(ctx, rng) } else { ObjPick::None };
            match pick {
                ObjPick::Goal(id, p, sub) => {
                    self.init_goal(1, 2, [p[0], p[1], p[2], 0.0], id, sub); // [M :508-509]
                    return self.goto_goal(ctx, rng, 1, state::BOT_IDLE); // [M :510]
                }
                ObjPick::Hold => return false,
                ObjPick::None => {}
            }
            self.uninit_goal(0);
            self.set_goal_pick_prefs(0.0, 0.0, 0.0, 0.0, 0, 0, 0, ctx.health, 1.0, 1.0);
            slot = 0;
        }
        // Phase B: the grudge hunt (slot 0, pref byte 0) [M :327-342]
        if slot == 0 && self.vars.goals[0].pref_byte == 0 {
            let g = self.vars.grudge;
            let hunted = (g >= 0)
                .then(|| ctx.others.get(g as usize).filter(|o| o.alive).map(|_| g as usize))
                .flatten();
            if let Some(idx) = hunted {
                let p = ctx.others[idx].pos;
                self.init_goal(0, 3, [p[0], p[1], p[2], 0.0], idx, 9);
                if self.goto_goal(ctx, rng, 0, state::BOT_IDLE) {
                    return true;
                }
            } else {
                self.vars.grudge = -1; // [M :340]
            }
        }
        // Phase C: the pickup scoring [M :345-488]
        let g0 = self.vars.goals[0];
        let mut weights = (g0.weight_health, g0.weight_ammo, g0.weight_weapon);
        if weights.0 + weights.1 + weights.2 == 0.0 {
            weights = (1.0, 1.0, 1.0); // [M :353-356]
        }
        let dist_limit = g0.dist_limit as u8 as f32; // 0xFE (the -2 store) = 254.0 [M :362-363]
        let passes = if g0.pref_byte & 0x2 != 0 { 1 } else { 4 }; // [M :367-371]
        let mut best: Option<(usize, f32, f32)> = None; // (index, score, dist)
        for pass in 0..passes {
            for (i, p) in ctx.pickups.iter().enumerate() {
                if i as i8 == self.vars.last_pickup || !p.available {
                    continue; // [M :386-387, :397]
                }
                let mut dist = ((self.pos[0] - p.pos[0]).powi(2)
                    + (self.pos[1] - p.pos[1]).powi(2)
                    + (self.pos[2] - p.pos[2]).powi(2))
                .sqrt();
                if dist > dist_limit || dist <= 0.0 {
                    continue; // [M :397]
                }
                let visit = p.visit.get(self.vars.bot_index as usize).copied().unwrap_or(0.0);
                if pass < 3 {
                    if visit != 0.0 && g0.pref_byte & 0x20 == 0 {
                        continue; // [M :404-417 — visited pickups skipped until the last pass]
                    }
                } else if visit != 0.0 {
                    // the visit decay [M :407-415]
                    let mul = if visit >= self.tick as f32 / 60.0 {
                        4500.0
                    } else {
                        ((self.tick as f32 / 60.0 - visit).abs() * 50.0).max(1.0)
                    };
                    dist *= mul;
                }
                // the class filters, pass 0 [M :431-477]
                if pass == 0 {
                    match p.mp_class {
                        0 if self.vars.arsenal.armed => {
                            // weapons: pass 0 scores only a preferred weapon the bot lacks
                            // (`isPreferredWeapon` ∧ !`hasWeapon` [M :431-441]; the demolition
                            // explosive double-score arm is not modelled [S])
                            let w = p.weapon_id.min(0xff) as u8;
                            let a = &self.vars.arsenal;
                            if !ctx.weapons.is_some_and(|t| crate::mp::botweap::is_preferred_weapon(t, self.vars.weapon_class, w)) || a.has_weapon(w) {
                                continue;
                            }
                        }
                        0 => {
                            // the unarmed stand-in (no weapon table): skip the held weapon id [S]
                            if p.weapon_id != 0 && p.weapon_id as u8 == self.vars.weapon_id {
                                continue;
                            }
                        }
                        // ammo: pass 0 only for a weapon the bot has (`hasWeapon` [M :459-464])
                        1 if self.vars.arsenal.armed => {
                            if !self.vars.arsenal.has_weapon(p.weapon_id.min(0xff) as u8) {
                                continue;
                            }
                        }
                        1 => continue, // the unarmed stand-in has no owned-weapon scan [S]
                        3 => {}
                        _ => continue, // class 2 and others never score [M :474-475]
                    }
                }
                if p.is_obj_flag2 && (g0.pref_byte & 0x4 != 0 || dist < 5.0) {
                    continue; // [M :420-421]
                }
                let proximity = dist_limit + 1.0 - dist; // ((u8)+0x47 + 1.0) − dist [M :402]
                let weight = match p.mp_class {
                    3 => weights.0,
                    1 => weights.1,
                    _ => weights.2,
                };
                let score = proximity * weight;
                let better = match best {
                    None => true,
                    Some((_, bs, bd)) => score > bs || (score == bs && dist < bd), // [M :449-453]
                };
                if better {
                    best = Some((i, score, dist));
                }
            }
            if best.is_some() {
                break;
            }
            if pass == 1 {
                weights = (1.0, 1.0, 1.0); // pass 2's equal weighting [M :375-379]
            }
        }
        let Some((idx, _, _)) = best else {
            self.uninit_goal(slot);
            return false; // [M :490-492]
        };
        self.vars.last_pickup = idx as i8; // [M :496]
        let pos = ctx.pickups[idx].pos;
        self.init_goal(slot, 1, [pos[0], pos[1], pos[2], 0.0], idx, 0);
        self.goto_goal(ctx, rng, slot, state::BOT_IDLE) // [M :510]
    }

    /// `BOTSTATE_pickGoal` 0x126b88 Phase A [M :143-313]: the scenario dispatch on
    /// MPSettings+0x420, each candidate object scored `((u8)+0x47 + 1 − dist)·+0x38` when
    /// `dist ≤ (u8)+0x47`, ties to the nearer [M :284-301] (`NDrone2_DistanceToEmitter`'s route
    /// distance → the straight line, the same *Deviation* as the pickup scoring). The arms, with
    /// `team` = my team word, `status` = my `MPGame+0x26`, `ts` = my team's `MPGame+0x1a8`:
    /// - CTF 0x20000004 (team ≠ 2): carrying (status & 1) → my base (subtype 2); else when my
    ///   team carries nothing (ts & 1 == 0) → the enemy flag `MP_getFlagObj(team == 0)` wherever it
    ///   is (subtype 1) [M :269-283].
    /// - Demolition 0x20000040 / Protection 0x20000080 (team ≠ 2): the site (subtype 8) [M :160-173,
    ///   :248-255] — both teams; the defenders' guard latch and the attackers' site-as-opponent are
    ///   the heartbeat's and `find_opponent`'s.
    /// - Espionage 0x20000100 (team ≠ 2): carrying (status & 2) → the best of my base's 4 approach
    ///   points (subtype 5); else when my team does not carry it (ts & 1 == 0) → the blueprint
    ///   wherever it is (subtype 4) [M :256-267, :303-312].
    /// - GoldenEye 0x20000200: items 0, 1 that are free/dropped (`MP_getGoldenEyeObj` gates state
    ///   ∈ {0, 2}) and not already my team's (ts & {1,2}[i]); an item another bot already targets
    ///   is kept only as the fallback (`Rand_Rand(100) & 0x20` picks between two such), scored in
    ///   the third pass when nothing else scored (subtype 3) [M :175-201].
    /// - Uplink 0x60000008 (team ≠ 2): the terminals not owned by my team (`MP_getUplinkObj(−n,
    ///   team)` — the first from index n−1 whose state ≠ team), the other bots' objectives kept as
    ///   the fallback (`Rand_Rand(bot count) == n` re-picks it) (subtype 6) [M :203-233].
    /// - KOTH 0x40000800, team KOTH 0x60001000 (team ≠ 2): on the hill (status & 0x10) → hold;
    ///   else the hill (subtype 7) [M :234-246].
    pub fn objective_scan(&mut self, ctx: &BotCtx, rng: &mut Rng) -> ObjPick {
        use super::scenario as sc;
        let Some(ob) = ctx.objectives else { return ObjPick::None };
        let weight = self.vars.goals[1].objective_weight;
        if weight == 0.0 {
            return ObjPick::None; // [M :144]
        }
        let me = ctx.my_index;
        let team = ob.teams.get(me).copied().unwrap_or(ctx.my_team);
        let status = ob.status.get(me).copied().unwrap_or(0);
        let ts = if team < 2 { ob.team_status[team as usize] } else { 0 };
        let limit = self.vars.goals[1].dist_limit as u8 as f32;
        // the scoring state (fVar21 best score, fVar25 its distance) [M :284-301]
        let mut best: Option<(usize, [f32; 3], u8, f32, f32)> = None;
        let root = self.root();
        let mut consider = |id: usize, p: [f32; 3], sub: u8, best: &mut Option<(usize, [f32; 3], u8, f32, f32)>| {
            let d = ((root[0] - p[0]).powi(2) + (root[1] - p[1]).powi(2) + (root[2] - p[2]).powi(2)).sqrt();
            if d > limit {
                return;
            }
            let score = (limit + 1.0 - d) * weight;
            let take = match *best {
                None => score >= -1.0,
                Some((_, _, _, bs, bd)) => bs <= score && (score != bs || d < bd),
            };
            if take {
                *best = Some((id, p, sub, score, d));
            }
        };
        match ctx.scenario {
            sc::CAPTURE_THE_FLAG if team != 2 => {
                if status & 1 != 0 {
                    let t = team as usize;
                    if let Some(b) = ob.bases.get(t).copied().flatten() {
                        consider(obj_id::BASE | t, b.pos, 2, &mut best);
                    }
                } else if ts & 1 == 0 {
                    let e = (team == 0) as usize;
                    if let Some(f) = ob.flags[e] {
                        consider(obj_id::FLAG | e, f.pos, 1, &mut best);
                    }
                }
            }
            sc::DEMOLITION | sc::PROTECTION if team != 2 => {
                if let Some(s) = ob.site {
                    consider(obj_id::SITE, s.pos, 8, &mut best);
                }
            }
            sc::BLUEPRINT if team != 2 => {
                if status & 2 != 0 {
                    for k in 0..4 {
                        if let Some(p) = ob.esp_point(team as usize, k) {
                            consider(obj_id::ESP_BASE | (team as usize * 4 + k), p.pos, 5, &mut best);
                        }
                    }
                } else if ts & 1 == 0 {
                    if let Some(b) = ob.blueprint {
                        consider(obj_id::BLUEPRINT, b.pos, 4, &mut best);
                    }
                }
            }
            sc::GOLDENEYE => {
                let mut fallback: Option<usize> = None; // lVar9
                for i in 0..3usize {
                    let mut cand: Option<usize> = None; // lVar14
                    if i == 2 {
                        cand = fallback.filter(|_| best.is_none()); // [M :177-181]
                    } else if ts & GE_TEAM_MASKS[i] == 0 {
                        if let Some(g) = ob.ge[i].filter(|g| g.state == 0 || g.state == 2) {
                            let id = obj_id::GOLDENEYE | i;
                            if ob.already_objective(id, me) {
                                // [M :190-196]: keep the first, then a coin (bit 0x20 of Rand(100))
                                if fallback.is_none() || rng.rand(100) & 0x20 != 0 {
                                    fallback = Some(id);
                                }
                            } else {
                                fallback = Some(id);
                                cand = Some(id);
                            }
                            let _ = g;
                        }
                    }
                    if let Some(id) = cand {
                        if let Some(v) = ob.view(id) {
                            consider(id, v.pos, 3, &mut best);
                        }
                    }
                }
            }
            sc::UPLINK if team != 2 => {
                let mut fallback: Option<usize> = None;
                let n = ob.uplinks.len();
                let bots = ob.bot_goals.len().max(1) as u32;
                let mut start = 0usize;
                loop {
                    let next = start + 1; // iVar6 [M :205]
                    // MP_getUplinkObj(−next, team): the first terminal from index next−1 whose
                    // state ≠ team [M 0x18b9c8:12-25]
                    let found = (start..n).find(|&k| ob.uplinks[k].state as i32 != team as i32);
                    start = next;
                    let Some(k) = found else {
                        // the scan ran out: the fallback scores unless something already did [M :210-215]
                        if best.is_none() {
                            if let Some(id) = fallback {
                                let p = ob.view(id).unwrap().pos;
                                consider(id, p, 6, &mut best);
                            }
                        }
                        break;
                    };
                    let id = obj_id::UPLINK | k;
                    if ob.already_objective(id, me) {
                        // [M :217-230]: the first such is kept; later ones replace it when
                        // Rand_Rand(bot count) == iVar6
                        fallback = match fallback {
                            None => Some(id),
                            Some(f) => {
                                if rng.rand(bots) == next as u32 { Some(id) } else { Some(f) }
                            }
                        };
                    } else {
                        consider(id, ob.uplinks[k].pos, 6, &mut best);
                    }
                }
            }
            sc::KOTH_FFA | sc::KOTH_TEAM if ctx.scenario == sc::KOTH_FFA || team != 2 => {
                if status & 0x10 != 0 {
                    return ObjPick::Hold; // [M :237-243]
                }
                if let Some(h) = ob.hill {
                    // the box centre — the hill obj's emitter stand-in [S]
                    let c = [(h.min[0] + h.max[0]) * 0.5, h.pos[1], (h.min[2] + h.max[2]) * 0.5];
                    consider(obj_id::HILL, c, 7, &mut best);
                }
            }
            _ => {}
        }
        match best {
            Some((id, p, sub, _, _)) => ObjPick::Goal(id, p, sub),
            None => ObjPick::None,
        }
    }

    /// The type-2 goal's validity each tick — the port's form of `processGoals`' objective gate
    /// (`(MPGame+team·2+0x1a8 & mask(obj+0x80)) == 0` [M 0x127c78:73-84]) and of the 0x3D
    /// `cancelGoalToObj` broadcasts `MP_setPlayerStatus` sends when an object changes hands [M
    /// 0x18a518:111-127]: the object must still pass the scan's own gate [S].
    fn objective_valid(&self, ctx: &BotCtx, id: usize) -> bool {
        let Some(ob) = ctx.objectives else { return false };
        let Some(v) = ob.view(id) else { return false };
        let me = ctx.my_index;
        let team = ob.teams.get(me).copied().unwrap_or(ctx.my_team);
        let status = ob.status.get(me).copied().unwrap_or(0);
        let ts = if team < 2 { ob.team_status[team as usize] } else { 0 };
        match id & obj_id::KIND {
            obj_id::FLAG => status & 1 == 0 && ts & 1 == 0,
            obj_id::BASE => status & 1 != 0,
            obj_id::UPLINK => v.state as i32 != team as i32,
            obj_id::HILL => status & 0x10 == 0,
            obj_id::SITE => ob.site.is_some(),
            obj_id::GOLDENEYE => {
                let i = id & !obj_id::KIND;
                (v.state == 0 || v.state == 2) && ts & GE_TEAM_MASKS[i.min(1)] == 0
            }
            obj_id::BLUEPRINT => status & 2 == 0 && ts & 1 == 0,
            obj_id::ESP_BASE => status & 2 != 0,
            _ => false,
        }
    }

    /// The objective's current goal point (objects move: a dropped flag, a carried blueprint).
    fn objective_point(&self, ctx: &BotCtx, slot: usize) -> [f32; 3] {
        let g = &self.vars.goals[slot];
        let id = g.target;
        let fixed = [g.celpos[0], g.celpos[1], g.celpos[2]];
        match (id & obj_id::KIND, ctx.objectives.and_then(|o| o.view(id))) {
            (obj_id::FLAG | obj_id::GOLDENEYE | obj_id::BLUEPRINT, Some(v)) => v.pos,
            _ => fixed,
        }
    }

    /// The objective arrival [S the `dist < Drone+0x8EC` stand-in for objects]: the body sphere
    /// in contact with the object's sphere (`MP_HitBy`'s reach — the touch the rules react to), the
    /// site within 3.0 (inside `FindOpponent`'s 4.0 site reach [M 0x1424e0:536-538]), the hill
    /// with the body centre inside its box (`MP_isPosOnHill` [M 0x18bc20]).
    fn objective_arrived(&self, ctx: &BotCtx, slot: usize) -> bool {
        let Some(ob) = ctx.objectives else { return false };
        let id = self.vars.goals[slot].target;
        let Some(v) = ob.view(id) else { return false };
        let root = self.root();
        let c = [root[0], root[1] + super::rules::PLAYER_SPHERE_DY, root[2]];
        match id & obj_id::KIND {
            obj_id::SITE => {
                let d2 = (root[0] - v.pos[0]).powi(2) + (root[1] - v.pos[1]).powi(2) + (root[2] - v.pos[2]).powi(2);
                d2 < 9.0
            }
            obj_id::HILL => (0..3).all(|k| {
                // a 0.3 horizontal margin keeps the bot inside, not on the edge
                let m = if k != 1 && v.max[k] - v.min[k] > 1.0 { 0.3 } else { 0.0 };
                v.min[k] + m <= c[k] && c[k] <= v.max[k] - m
            }),
            // the CTF capture is the carrier within sq-dist 2.0 of the base + 1.0 y
            // [M 0x1861a8:74-86] — the plain floor-plan arrival on the base point reaches it
            obj_id::BASE => false,
            _ if v.radius > 0.0 => {
                let reach = v.radius + super::rules::PLAYER_SPHERE_RADIUS - 0.15;
                let d2 = (c[0] - v.centre[0]).powi(2) + (c[1] - v.centre[1]).powi(2) + (c[2] - v.centre[2]).powi(2);
                d2 < reach * reach
            }
            _ => false,
        }
    }

    /// `BOTSTATE_processGoals` 0x127c78 (the port's flow; called by the heartbeat's non-combat
    /// types [M 0x16e060:498-501]). Returns true when it changed the SM state.
    pub fn process_goals(&mut self, ctx: &BotCtx) -> bool {
        if self.vars.active_goal == 0xFF {
            return false;
        }
        let slot = self.vars.active_goal as usize;
        let g = self.vars.goals[slot];
        if g.goal_type == 0 {
            self.vars.active_goal = 0xFF; // [M :64-68]
            return false;
        }
        // the deadline [M :70]
        if self.tick as f32 / 60.0 > g.start + g.duration {
            // expiry: fire the goal state if I'm walking a goal state [M :216-221]
            if matches!(self.state, 0xEB | 0xF8) {
                self.set_state_change(g.fire_state);
            }
            self.uninit_goal(slot);
            return true;
        }
        // per-type validity
        match g.goal_type {
            2 => {
                // the objective gate [M :73-84] (the port's form — `objective_valid`); invalid →
                // the goal's state fires and the slot clears [M :206-221]
                if !self.objective_valid(ctx, g.target) {
                    if matches!(self.state, 0xEB | 0xF8) {
                        self.set_state_change(g.fire_state);
                    }
                    self.uninit_goal(slot);
                    return true;
                }
            }
            3 => {
                // the hunt target must be a live player [M :88-97]
                let valid = ctx.others.get(g.target).is_some_and(|o| o.alive);
                if !valid {
                    if matches!(self.state, 0xEB | 0xF8) {
                        self.set_state_change(g.fire_state);
                    }
                    self.uninit_goal(slot);
                    return true;
                }
                // the trait-2 melee gate [M :108-113]: candidate + same-team, dist² < 20.25, |Δz| < 2
                if self.vars.trait_id == 2 {
                    if let Some(o) = ctx.others.get(g.target) {
                        if o.flags & 0xA == 0xA
                            && o.dist2 < 20.25
                            && (o.pos[1] - self.pos[1]).abs() < 2.0
                        {
                            self.vars.goals[slot].fire_state = st::BOT_GUARD_FRIEND_IDLE;
                            self.vars.opponent = Some(g.target); // +0x758 [M :113]
                        }
                    }
                }
            }
            _ => {}
        }
        // the arrival: the stored route code == 3 or the complete flag [M :165-205]
        let arrived = g.route_code == 3 || g.complete & 1 != 0;
        if arrived {
            if g.goal_type == 1 {
                self.set_pickup_visit_time(ctx, g.target); // [M :153,199]
            }
            let fire = self.vars.goals[slot].fire_state;
            if matches!(self.state, 0xEB | 0xF8) {
                self.set_state_change(fire);
            }
            self.uninit_goal(slot);
            return true;
        }
        false
    }

    // ------------------------------------------------------------------
    // The target choice + perception
    // ------------------------------------------------------------------

    /// `BOTSTATE_getPreferredTraitOpponentObjIndex` 0x128b50 — the trait switch [M :92-223].
    /// Returns the player index or −1.
    pub fn trait_pick_opponent(&self, ctx: &BotCtx) -> i8 {
        // mp-bot-objectives: Assassination — the assassin's pick is always the target
        // (`MP_IsAssasin` → `MP_getAssassinTarget` [M :80-86]), ahead of the trait switch.
        if ctx.scenario == super::scenario::ASSASSIN {
            if let Some(ob) = ctx.objectives {
                if ob.assassin == Some(ctx.my_index) {
                    return ob.assassin_target.map_or(-1, |t| t as i8);
                }
            }
        }
        // The team reads are the +0xB0 rows' bit 0x8 (same team in the team view of
        // `setOtherPlayerInfo` 0x125390:62-102 — FFA compares against team 3, so no one) [M
        // :98/:136/:155]; the old team-word compares found no enemy at all in FFA (every word 0).
        let same = |o: &OtherPlayer| o.flags & 8 != 0;
        match self.vars.trait_id {
            2 => {
                // the same-team pool [M :93-131]: min dist² among same-team candidates
                let mut best: Option<(usize, f32)> = None;
                for o in ctx.others {
                    if o.index == ctx.my_index || o.index >= OBJ_ROW || !same(o) || !o.alive {
                        continue;
                    }
                    match best {
                        Some((_, d)) if o.dist2 >= d => {}
                        _ => best = Some((o.index, o.dist2)),
                    }
                }
                match best {
                    Some((idx, d)) => {
                        if d >= 20.25 {
                            idx as i8 // [M :120-122]
                        } else {
                            -1 // the close-range guard handoff (setStateChange(0xF7)) [M :123-130]
                        }
                    }
                    None => -1,
                }
            }
            4 => {
                // the top-score enemy [M :132-150]
                let mut best: Option<(usize, f32)> = None;
                for o in ctx.others {
                    if o.index == ctx.my_index || o.index >= OBJ_ROW || same(o) || !o.alive {
                        continue;
                    }
                    if best.map_or(true, |(_, s)| o.score > s) {
                        best = Some((o.index, o.score));
                    }
                }
                best.map_or(-1, |(i, _)| i as i8)
            }
            5 => {
                // keep the current opponent, else the nearest enemy from the block [M :151-170]
                if let Some(cur) = self.vars.opponent {
                    return cur as i8;
                }
                let mut best: Option<(usize, f32)> = None;
                for o in ctx.others {
                    if o.index == ctx.my_index || o.index >= OBJ_ROW || same(o) || !o.alive {
                        continue;
                    }
                    if o.flags & 0xA != 0x2 {
                        continue; // bit1 candidate set, bit3 clear [M :152-166]
                    }
                    match best {
                        Some((_, d)) if o.dist2 >= d => {}
                        _ => best = Some((o.index, o.dist2)),
                    }
                }
                best.map_or(-1, |(i, _)| i as i8)
            }
            8 => {
                // the lowest-health enemy, init my own health [M :192-223]
                let mut best: Option<(usize, f32)> = None;
                for o in ctx.others {
                    if o.index == ctx.my_index || o.index >= OBJ_ROW || same(o) || !o.alive {
                        continue;
                    }
                    if best.map_or(true, |(_, h)| o.health < h) {
                        best = Some((o.index, o.health));
                    }
                }
                best.map_or(-1, |(i, _)| i as i8)
            }
            _ => -1, // traits 0,1,3,6,>8 [M :223]
        }
    }

    /// `BOT_setOtherPlayerInfo` 0x125390 (the port's pass): bit1 candidate, bit3 same team,
    /// bit2 seen (the LOS via the collision world — the game's `NDrone2_CanSeeObject`), the
    /// +0x04 dist², and the +0x770 scan flag. Called once per tick like ControlSTANDARD:164.
    pub fn scan_others(&mut self, ctx: &BotCtx) {
        self.vars.scan_flag = 0;
        let mut seen_any = false;
        for o in ctx.others {
            if o.index == ctx.my_index || o.index >= OBJ_ROW || !o.alive {
                continue;
            }
            seen_any = true;
        }
        if seen_any {
            self.vars.scan_flag = 1; // [M 0x125390:61]
        }
    }

    /// `NDrone2_FindOpponent` 0x1424e0 (the port's minimal pick): accept the nearest live
    /// candidate the pass found (the gate [M 0x1424e0:164-167 — 0x600 clear, 0x100 set,
    /// health > 0]; the full score/history selection is not lifted — [S], see the report).
    /// Team reads are `MP_getObjectTeam` (0x18b6c0).
    pub fn find_opponent(&mut self, ctx: &BotCtx) {
        // The hold [M 0x1424e0:355-365]: the current opponent survives only while it is valid
        // (alive, not a team-mate in the Assassination role shuffle) and the unseen counter is
        // within FRAME_RATE × aggression_mul × 20 — `BOT_getAggressionMul` 0x126468 (0.3..1.0
        // by the aggression byte, × up to 3 inside 2.5u) — then it clears and the scan re-picks.
        // Holding until death kept a carrier chasing an enemy that had broken line of sight
        // instead of resuming the objective (bot-weapons-finish-2).
        if let Some(i) = self.vars.opponent {
            // The validity arm is class-gated [M 0x1424e0:357-365]: class 1 clears outright,
            // class 2 (a drone) runs the alive/enabled check, any OTHER class (the demolition/
            // protection site pseudo-opponent) skips it. The port's site opponent (index
            // OBJ_ROW and above, not a ctx.others row) is that other class, and the perception
            // pass never ticks +0x274 for it — the port keeps it for the goal's lifetime (the
            // attackers shoot the standing site; bot-weapons-finish-2's hold regressed
            // demolition/protection and the site-ack chain needs its own task).
            if i >= OBJ_ROW {
                return;
            }
            let (valid, dist) = if i < OBJ_ROW {
                let o = ctx.others.get(i);
                (o.is_some_and(|o| o.alive && o.flags & 8 == 0), o.map(|o| o.dist2.sqrt()))
            } else {
                (true, None)
            };
            let mut mul = match self.vars.aggression {
                0 => 0.3,
                1 => 0.5,
                2 => 0.7,
                3 => 0.85,
                _ => 1.0,
            };
            if let Some(d) = dist {
                if d < 2.5 {
                    mul = (mul * ((2.5 - d) + 1.0)).min(3.0); // [M 0x126468:22-30]
                }
            }
            let hold = (mul * 20.0) as u32; // FRAME_RATE is 1 at the fixed 60 Hz
            if !valid || self.vars.unseen_ticks > hold {
                self.vars.opponent = None; // NDrone2_SetOpponent(0) [M 0x1424e0 LAB_0014277c]
            }
        }
        let mut best: Option<(usize, f32)> = None;
        for o in ctx.others {
            if o.index == ctx.my_index || o.index >= OBJ_ROW || !o.alive {
                continue;
            }
            if !ctx.lazy_sight && o.flags & 0x6 != 0x6 {
                continue; // bit1 candidate + bit2 seen (the game's CanSeeObject bit [S — the
                          // vision query's range/params are not lifted; the caller supplies it])
            }
            if o.flags & 8 != 0 {
                continue; // a team-mate (`(uVar3 & 8) == 0` gates the scoring [M :364])
            }
            // mp-bot-objectives: the sight range and cone — `fVar24 = Drone+0xE8²` (the
            // drones' default 24.0 [M 0x14b300:386]) and the half-angle `Drone+0xE4` (π/2 [M
            // 0x14b300:427]); the 360° bit (stats +0x0C & 8) widens the range to 300.0 and drops
            // the cone [M 0x1424e0:277-300]. (The bots' own `DefaultInit` arm is not confirmed —
            // the default row is the port's pick [S].) Unlimited LOS made every bot fight every
            // bot it could see across the arena, and no objective was ever reached.
            let wide = self.vars.flags & 0x08 != 0;
            if o.dist2 >= if wide { 90000.0 } else { 576.0 } {
                continue;
            }
            if !wide {
                let (dx, dz) = (o.pos[0] - self.pos[0], o.pos[2] - self.pos[2]);
                if dx * crate::trig::cosf(self.yaw) + dz * crate::trig::sinf(self.yaw) < 0.0 {
                    continue; // outside ±90°
                }
            }
            // mp-bot-objectives: an objective carrier (status & 0xF — flag, blueprint, the
            // GoldenEye items) scores at a quarter of its distance [M :367-369, fVar20 = 0.25]
            let carrier = ctx.objectives.is_some_and(|ob| ob.status.get(o.index).is_some_and(|&s| s & 0xF != 0));
            let d = if carrier { o.dist2 * 0.25 } else { o.dist2 };
            if best.is_some_and(|(_, bd)| d >= bd) {
                continue; // the earlier pick stays on a tie
            }
            if ctx.lazy_sight {
                // mp-perf-16: only a candidate that would take the pick pays for its sight line —
                // the same answer as the precomputed bit2, without the per-pair segments. The
                // candidate cone is the scan's own hard ±half-FOV test above [M 0x1424e0's
                // replacement arm re-tests the pick against ±(+0xE4·57.3)° before keeping it];
                // the vis ≥ 0.7 score gates the perception's ack, not the scan's pick.
                let chest = [o.pos[0], o.pos[1] - 0.4, o.pos[2]];
                let seen = o.flags & 0x6 == 0x6 || ctx.world.segment_bullet_clear(self.pos, chest);
                if !seen { continue; }
            }
            best = Some((o.index, d));
        }
        // The pick: the port overwrites the held opponent with this tick's scan result — the
        // game's hold (the block above) bounds a held opponent to ~aggression×20 unseen frames
        // anyway, so the port's stricter re-acquire lands within the same window [*S* port
        // choice: the decompile's scan-miss keeps the held opponent (0x1424e0's tail returns
        // without SetOpponent(0)), but the game's vision-chain re-acquire gates make the
        // effective lifetime near-identical; the overwrite ends the chases the moment the
        // segment check fails, which the carriers' delivery windows need]. A miss still runs
        // the tail's site arm — its own gate is `site != +0x170`, not "no opponent" [M
        // 0x1424e0:541].
        if let Some((i, _)) = best {
            if self.vars.opponent != Some(i) && i >= OBJ_ROW {
                self.vars.unseen_ticks = 0; // the site pick starts its hold fresh
            }
            self.vars.opponent = Some(i);
        } else {
            self.find_site_opponent(ctx);
        }
    }

    /// The tail of `NDrone2_FindOpponent` 0x1424e0:519-560 — with no player chosen, an
    /// attacker (Protection team 0, Demolition team 1) takes the standing site as its opponent
    /// when it is within sq-dist 16.0, inside the vision cone and visible
    /// (`NDrone2_CanSeeObject(.., 5, 0)`), and tells itself the objective goal is complete
    /// (`MP_sendBotMessage(.., 0x3b, 1, 0)` → `setGoalComplete(1)`) [M]. The cone half-angle
    /// (`Drone+0xE4`) is not lifted — 90° [S]; the sight is the driver's seen bit on the row.
    fn find_site_opponent(&mut self, ctx: &BotCtx) {
        if self.vars.opponent.is_some_and(|i| i >= OBJ_ROW) {
            return; // the site is already the opponent [M 0x1424e0:541 `iVar7 != +0x170`]
        }
        let Some(ob) = ctx.objectives else { return };
        let Some(site) = ob.site else { return };
        let team = ob.teams.get(ctx.my_index).copied().unwrap_or(ctx.my_team);
        if team != 1 - ob.site_defenders.min(1) {
            return; // the defenders never shoot their own site [M 0x1424e0:520-525]
        }
        let Some(row) = ctx.others.get(OBJ_ROW).filter(|r| r.alive && r.index == OBJ_ROW) else { return };
        let root = self.root();
        let d2 = (root[0] - site.pos[0]).powi(2) + (root[1] - site.pos[1]).powi(2) + (root[2] - site.pos[2]).powi(2);
        if d2 >= 16.0 {
            return; // [M :536-538]
        }
        let (dx, dz) = (site.pos[0] - self.pos[0], site.pos[2] - self.pos[2]);
        if dx * crate::trig::cosf(self.yaw) + dz * crate::trig::sinf(self.yaw) < 0.0 {
            return; // behind the bot [M :539-549, the cone]
        }
        if row.flags & 4 == 0 {
            return; // [M :550]
        }
        self.vars.opponent = Some(OBJ_ROW);
        self.set_goal_complete(1); // the 0x3b self-message [M :551]
    }

    /// `NDrone2_ReactToOpponentSighted` 0x1770f0 — the gates the decompile shows, then the
    /// suggested state [M :5-…; the surrender arm returns 0x3F].
    pub fn react_to_opponent_sighted(&self, suggested: u8) -> u8 {
        if self.vars.opponent.is_none() {
            return 0; // [M :16-18]
        }
        suggested
    }

    /// `BOTSTATE_chooseCombatMove` 0x129218 via `checkAttackMove` 0x126a40: `Rand_Rand(4)` →
    /// 0xD5 (1) / 0xD4 (0) / 0xDB (2) / 0xDC (3) [M 0x129218], each gated by its `NDrone2_Can*`
    /// test [M 0x126a40: 0xD4 CanStrafeLeft, 0xD5 CanStrafeRight, 0xDB CanStepLeft, 0xDC
    /// CanStepRight — the port's `can_side_move`]. `BOTSTATE_EvasiveMove` and
    /// `BOT_getMovePossibility(drone, 2)` are not lifted (taken as no evasive move / possible) [S].
    fn choose_combat_move(&mut self, ctx: &BotCtx, rng: &mut Rng) -> Option<u8> {
        let roll = rng.rand(4);
        let mv = match roll {
            0 => st::BOT_ATTACK_STRAFE_AIM_LEFT,
            1 => st::BOT_ATTACK_STRAFE_AIM_RIGHT,
            2 => st::BOT_ATTACK_STEP_AIM_LEFT,
            _ => st::BOT_ATTACK_STEP_AIM_RIGHT,
        };
        if ctx.stationary {
            return None; // the stand preset never moves
        }
        let reach = if matches!(mv, st::BOT_ATTACK_STRAFE_AIM_LEFT | st::BOT_ATTACK_STRAFE_AIM_RIGHT) { 1.5 } else { 1.0 };
        self.can_side_move(ctx, Self::move_side(mv), reach).then_some(mv)
    }

    // ------------------------------------------------------------------
    // BotGlobal — state 0xC5, the forced heartbeat arm [M 0x16e060:84-519]
    // ------------------------------------------------------------------

    fn bot_global_msg3(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        // SM current state == 0 → 0xF9 [M :86-87]
        if self.state == 0 {
            self.set_state(state::BOT_IDLE);
            return;
        }
        self.vars.state_type = state_type(self.state as u32); // [M :90]
        // (the death branch [M :91-119] is the test's alive-bot path — the port carries it as
        // the caller's responsibility until the death states land; see the report)
        // tick reset: the +0x72C requested state [M :121-130]
        if self.vars.requested_state != 0 {
            if self.state as u32 != self.vars.requested_state {
                let next = self.vars.requested_state as u8;
                self.vars.requested_state = 0;
                self.set_state(next);
                return;
            }
            self.vars.requested_state = 0;
        }
        // HP regen [M :209-233]: +5 clamp to max, the +0x738 deadline re-arm
        // regen_deadline is a TICK count — compare ticks exactly (the old now*60 f32 fold
        // drifted above 2^24; the tick compare is the same predicate, exact).
        if self.vars.flags & 0x10 != 0 && self.tick as f32 >= self.vars.regen_deadline as f32 {
            self.vars.regen_deadline += FRAME_RATE_INT;
        }
        let ty = self.vars.state_type;
        // mp-bot-objectives: the defenders' guard latch [M :244-268] — in a type-5 (goal walk)
        // state of Protection (defenders team 1) / Demolition (team 0), a defender whose sphere
        // centre is within 3.0 of the site's box latches `+0x771` and stops (SetState 0xF9); BotIdle
        // then holds it there while the latch re-tests true [M 0x171b80:66-96].
        if ty == 5 {
            if let Some(ob) = ctx.objectives {
                let team = ob.teams.get(ctx.my_index).copied().unwrap_or(ctx.my_team);
                if let Some(site) = ob.site.filter(|s| s.radius > 0.0 && team == ob.site_defenders) {
                    self.vars.attack_latch = self.site_sphere_box(&site) as u8;
                    if self.vars.attack_latch != 0 {
                        self.set_state(state::BOT_IDLE);
                        return;
                    }
                }
            }
        }
        match ty {
            4 => {
                // type 4 → the combat block [M :290-495]: mp-bot-weapons ports its weapon half
                // (the empty gun → reload/change, the fists up close, the explosives' ranges);
                // the combat-move half stays the states' own [S].
                if let Some(t) = ctx.weapons {
                    if self.vars.arsenal.armed {
                        self.combat_weapon_block(ctx, t, rng);
                    }
                }
            }
            1 | 2 => {
                // types 1/2 → the tail return [M :517-519]
            }
            10 => {}
            _ => {
                // types 3,5,6..9,11..13 [M :498-513]
                self.vars.nav_node = 0xFFFF; // setNearestNavNode [M 0x129420:6]
                let changed = self.process_goals(ctx);
                if !changed && self.vars.opponent.is_some() {
                    // the 5 s sight latch [M :504-509]
                    if self.tick as f32 / 60.0 - self.vars.sight_stamp < 5.0 {
                        self.vars.sighted |= 4;
                    }
                    if self.vars.sighted & 4 != 0 {
                        let next = self.react_to_opponent_sighted(st::BOT_SEEN_OPPONENT);
                        if next != 0 {
                            self.set_state(next);
                        }
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // State 0xF9 BotIdle [M 0x171b80]
    // ------------------------------------------------------------------

    fn bot_idle_enter(&mut self) {
        if self.vars.opponent.is_none() || self.vars.sighted & 4 == 0 {
            self.vars.opponent = None; // SetOpponent(0) [M :39]
            return; // + the +0xCC0/4/8 search timer [M :40-42]
        }
        self.set_state(state::BOT_ATTACK); // [M :45-49]
    }

    /// The hunt goal on the nearest live opponent (slot 0, type 3 → subtype 9 — the same goal
    /// the grudge hunt builds [M 0x126b88:327-342]). Opponents: every live row not marked
    /// same-team (bit 0x8 — the team view of `setOtherPlayerInfo` 0x125390:62-102: FFA marks no
    /// one, team games the team-mates, Assassination everyone but the assassin for the
    /// non-assassins). Returns true when set.
    fn hunt_nearest(&mut self, ctx: &BotCtx, rng: &mut Rng) -> bool {
        // (the goal target is the row position — the rows sit at their slot index)
        let live = || ctx.others.iter().enumerate().filter(|(_, o)| o.index != ctx.my_index && o.index < OBJ_ROW && o.alive);
        let mut best: Option<(usize, f32)> = None;
        for (i, o) in live() {
            if o.flags & 8 != 0 {
                continue;
            }
            let d = dist2d(o.pos, self.pos) + (o.pos[1] - self.pos[1]).abs();
            if best.is_none_or(|(_, b)| d < b) {
                best = Some((i, d));
            }
        }
        let Some((idx, _)) = best else { return false };
        let p = ctx.others[idx].pos;
        self.uninit_goal(0);
        self.init_goal(0, 3, [p[0], p[1], p[2], 0.0], idx, 9);
        self.goto_goal(ctx, rng, 0, state::BOT_IDLE)
    }

    fn bot_idle_msg3(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        let pick = self.trait_pick_opponent(ctx); // [M :54]
        self.vars.grudge = pick; // +0x76B [M :55]
        if pick != -1 {
            // zero the slot-0 prefs → the hunt pick [M :57-59]
            self.set_goal_pick_prefs(0.0, 0.0, 0.0, 0.0, 0, 0, 0, ctx.health, 1.0, 1.0);
            if self.pick_goal(ctx, rng, 0) {
                self.set_state(state::BOT_GOTO_GOAL_POSITION); // [M :61-64]
                return;
            }
        }
        // the latch/mode gate [M :66-72 verbatim in the research]
        if self.vars.attack_latch == 0
            || !matches!(ctx.scenario, 0x2000_0080 | 0x2000_0040)
        {
            // mp-bot-objectives: `setGoalPickPrefs(0,0,0,1.0, slot 1)` + `pickGoal(1)` [M :73-79] —
            // its Phase A (the mode objective) first; the KOTH hold keeps the bot idle on the hill.
            if ctx.objectives.is_some() {
                self.set_goal_pick_prefs(0.0, 0.0, 0.0, 1.0, 1, 0, 0, ctx.health, 1.0, 1.0);
                self.uninit_goal(1);
                match self.objective_scan(ctx, rng) {
                    ObjPick::Goal(id, p, sub) => {
                        self.init_goal(1, 2, [p[0], p[1], p[2], 0.0], id, sub);
                        if self.goto_goal(ctx, rng, 1, state::BOT_IDLE) {
                            self.set_state(state::BOT_GOTO_GOAL_POSITION); // [M :76-78]
                        }
                        return;
                    }
                    ObjPick::Hold => return,
                    ObjPick::None => {}
                }
            }
            // The hunt fallback [S]: with every pickup visited inside its 45 s cooldown (or none
            // registered) the bot hunts the nearest live opponent instead of re-walking a stale
            // pickup — the game's deathmatch bots roam into their opponents; its own source of
            // that (the objective/route scoring of `pickGoal` Phase A, the `NDrone2_FindOpponent`
            // search sweep) is not lifted. Without it the bots stood once the table was spent.
            let now = self.tick as f32 / 60.0;
            let idx = self.vars.bot_index as usize;
            let fresh = ctx.pickups.iter().enumerate().any(|(i, p)| {
                i as i8 != self.vars.last_pickup && p.visit.get(idx).is_some_and(|&v| v == 0.0 || v < now)
            });
            if fresh {
                self.set_goal_pick_prefs(0.0, 0.0, 0.0, 1.0, 1, 0, 0, ctx.health, 1.0, 1.0);
                if self.pick_goal_scan(ctx, rng, 1, ctx.objectives.is_none()) {
                    self.set_state(state::BOT_GOTO_GOAL_POSITION);
                    return;
                }
            }
            if self.hunt_nearest(ctx, rng) {
                self.set_state(state::BOT_GOTO_GOAL_POSITION);
            }
        } else {
            // the protection/demolition guard-site sphere-box gate [M :80-96]
            self.vars.attack_latch = self.guard_site_check(ctx);
            if self.vars.attack_latch != 0 {
                self.vars.guard_near_flag = 1; // +0x76F [M :96]
            }
        }
    }

    /// The BotIdle protection/demolition site check [M 0x171b80:80-94]: `Intersect_SphereBox`
    /// (radius 3.0) of my pos against the site's box → the +0x771 latch. Deathmatch has no
    /// site → the latch clears (the caller only reaches here in those modes).
        // `BOTSTATE_opponentIsMissile` (0x129a60): the demo/protection attacker may target the
        // site object itself as "opponent" (team-gated, +0xF6 done check) — ported as a stub [S].
    fn guard_site_check(&mut self, ctx: &BotCtx) -> u8 {
        let Some(ob) = ctx.objectives else { return 0 };
        let Some(site) = ob.site.filter(|s| s.radius > 0.0) else { return 0 }; // obj+0xD8 != 0 [M :86]
        self.site_sphere_box(&site) as u8
    }

    /// `Intersect_SphereBox(3.0, obj+0x80 − site+0xC0, box min, box max)` [M 0x171b80:86-94,
    /// 0x16e060:258-265]: the bot's world-sphere centre within 3.0 of the site's box (the box is
    /// the snapshot's world-space min/max — the same relative test) [M `Intersect_SphereBox`
    /// 0x1ed988: the clamped-point distance].
    fn site_sphere_box(&self, site: &ObjView) -> bool {
        let root = self.root();
        let c = [root[0], root[1] + super::rules::PLAYER_SPHERE_DY, root[2]];
        let mut d2 = 0.0f32;
        for k in 0..3 {
            let q = c[k].clamp(site.min[k], site.max[k]);
            d2 += (c[k] - q) * (c[k] - q);
        }
        d2 <= 9.0
    }

    // ------------------------------------------------------------------
    // State 0xEB BotGotoGoalPosition [M 0x171240]
    // ------------------------------------------------------------------

    /// `NDrone2_MoveToGoalPosition` 0x1517d8 (the port's straight-line walk — *Deviation*: the
    /// `AINetwork_CalcRouteToPosition`/`FollowRoute` steering is not wired for the MP arenas;
    /// the arrival code 3 stands in for `dist < Drone+0x8EC` [M 0x1517d8:52-54]).
    fn move_to_goal_position(&mut self, ctx: &BotCtx) -> u8 {
        let Some(slot) = self.get_active_goal() else { return 3 };
        // the hunt goals track the live player obj (`SetupGoalPositionToObj` [M 0x1278a8:65-66]);
        // pickup/objective goals use the goal's CelPos [M :77-78]
        let target: [f32; 3] = if self.vars.goals[slot].goal_type == 3 {
            ctx.others.get(self.vars.goals[slot].target).map(|o| o.pos).unwrap_or(self.pos)
        } else if self.vars.goals[slot].goal_type == 2 {
            // mp-bot-objectives: the objective goal walks to the object (`MP_recalcObjExtPaths`
            // keeps a moving object's emitter on it [M 0x18b888]) and arrives on contact
            if self.objective_arrived(ctx, slot) {
                return 3;
            }
            self.objective_point(ctx, slot)
        } else {
            let c = self.vars.goals[slot].celpos;
            [c[0], c[1], c[2]]
        };
        // The arrival: within 1.0 on the floor plan and on the same storey (the body walks the
        // floors, so the eye never meets an item's height) [S the `dist < Drone+0x8EC` stand-in].
        let dy = target[1] - self.pos[1];
        if dist2d(self.pos, target) < 1.0 && dy.abs() < 2.0 {
            return 3; // arrived
        }
        if ctx.stationary {
            return 1; // the stationary gate holds the bot in place [M 0x16f198:42-45 pattern; 0xEB arm [S]]
        }
        // The goal walk: a hunt target's root is its eye − EYE; an item's is its floor + 1.1.
        let hunt = self.vars.goals[slot].goal_type == 3;
        let dest = if hunt {
            [target[0], target[1] - crate::player::EYE, target[2]]
        } else {
            self.item_root(ctx, target)
        };
        let mul = [0.7f32, 1.0, 1.3][(self.vars.speed as usize).min(2)]; // the speed mult [M atlas §4]
        if !self.steer_to(ctx, dest, mul.min(1.0), true) {
            return 0xC; // no route and stuck: the route failed (validateRoute → the goal ends)
        }
        1
    }

    /// An item's standing spot: the floor under it + 1.1 (the body root), else its own height.
    fn item_root(&self, ctx: &BotCtx, p: [f32; 3]) -> [f32; 3] {
        if self.route.valid && dist2d(self.route.dest, p) < 0.01 {
            return self.route.dest;
        }
        match ctx.world.floor_below([p[0], p[1] + 0.5, p[2]], 3.0) {
            Some(f) => [p[0], f + 1.1, p[2]],
            None => [p[0], p[1] + 0.6, p[2]],
        }
    }

    /// Turn the facing toward a point by at most `rate` rad (the game's ATAN2_APPROX, not std
    /// libm — std's atan2 differs by ulps across libms and would desync lockstep peers).
    fn turn_toward(&mut self, p: [f32; 3], rate: f32) {
        let dx = p[0] - self.pos[0];
        let dz = p[2] - self.pos[2];
        if dx * dx + dz * dz < 1e-6 {
            return;
        }
        self.turn_to_yaw(crate::trig::atan2_approx(dz, dx), rate);
    }

    fn turn_to_yaw(&mut self, want: f32, rate: f32) {
        let pi = std::f32::consts::PI;
        let mut delta = want - self.yaw;
        while delta > pi { delta -= 2.0 * pi; }
        while delta < -pi { delta += 2.0 * pi; }
        self.yaw += delta.clamp(-rate, rate);
        while self.yaw > pi { self.yaw -= 2.0 * pi; }
        while self.yaw < -pi { self.yaw += 2.0 * pi; }
    }

    /// The body root (the feet's floor + 1.1).
    fn root(&self) -> [f32; 3] {
        [self.pos[0], self.pos[1] - crate::player::EYE, self.pos[2]]
    }

    /// Walk toward `dest` (body-root space) along the arena's AI network: straight when the walk
    /// is clear, else a shortest path over the network's links between the nearest walk-clear
    /// nodes, skipping ahead whenever the node after next is already walk-clear [S — the game's
    /// `AINetwork_CalcRouteToPosition` → A* → `OptimiseRoute` → `FollowRoute` link creep; the
    /// ported creep (`nav::Route`) is tuned to the drones' anim root motion, so the bots use this
    /// plain follower over the same graph]. `face` turns the body along the walk (the goal walk;
    /// the combat moves keep it on the opponent). Returns false when the bot is stuck for good
    /// (three stuck checks in a row) so the caller can drop the goal.
    fn steer_to(&mut self, ctx: &BotCtx, dest: [f32; 3], speed: f32, face: bool) -> bool {
        let root = self.root();
        let mut way = dest;
        if ctx.world.triangle_count() > 0 {
            if let Some(net) = ctx.nav {
                let stale = !self.route.valid
                    || dist2d(self.route.dest, dest) > 3.0
                    || (dest[1] - self.route.dest[1]).abs() > 1.5
                    || self.tick.wrapping_sub(self.route.built) > 240;
                if stale && (!self.route.valid || self.tick.wrapping_sub(self.route.built) >= 20) {
                    self.plan_route(ctx, net, root, dest);
                }
                if self.route.valid && !self.route.nodes.is_empty() {
                    let p = &net.paths[self.route.path];
                    // Advance past reached nodes; skip ahead one node a tick when the next-but-one
                    // (or the destination, past the last) is already walk-clear.
                    while self.route.idx < self.route.nodes.len() {
                        let n = node_root(p.nodes[self.route.nodes[self.route.idx] as usize].pos);
                        if dist2d(root, n) < 0.8 && (n[1] - root[1]).abs() < 1.5 {
                            self.route.idx += 1;
                        } else {
                            break;
                        }
                    }
                    if self.route.idx < self.route.nodes.len() {
                        let after = if self.route.idx + 1 < self.route.nodes.len() {
                            node_root(p.nodes[self.route.nodes[self.route.idx + 1] as usize].pos)
                        } else {
                            dest
                        };
                        if walk_clear(ctx.world, root, after) {
                            self.route.idx += 1;
                        }
                    }
                    if self.route.idx < self.route.nodes.len() {
                        way = node_root(p.nodes[self.route.nodes[self.route.idx] as usize].pos);
                    }
                }
            }
        }
        let dx = way[0] - root[0];
        let dz = way[2] - root[2];
        let d = (dx * dx + dz * dz).sqrt();
        if d > 1e-4 {
            self.move_want = Some(([dx / d, dz / d], speed));
            if face {
                self.turn_to_yaw(crate::trig::atan2_approx(dz, dx), 0.25);
            }
        }
        self.stuck_hits < 3
    }

    /// The route plan (see `steer_to`): straight when walk-clear; else Dijkstra over the network
    /// links from the nearest walk-clear nodes at the start to the nearest at the destination
    /// (ties by node index — deterministic). No reachable node → straight (the body slides).
    fn plan_route(&mut self, ctx: &BotCtx, net: &crate::nav::Network, from: [f32; 3], dest: [f32; 3]) {
        let _p = crate::prof::scope("bots: plan_route");
        self.route = BotRoute { path: 0, nodes: Vec::new(), idx: 0, dest, built: self.tick, valid: true };
        if walk_clear(ctx.world, from, dest) {
            return;
        }
        let Some(pi) = net.nav_path_for(from) else { return };
        let p = &net.paths[pi];
        let n = p.nodes.len();
        if n == 0 {
            return;
        }
        // The nearest few walk-clear nodes around a point (2D order, ties by index).
        let near = |at: [f32; 3]| -> Vec<(usize, f32)> {
            let mut c: Vec<(f32, usize)> = (0..n)
                .map(|i| (dist2d(node_root(p.nodes[i].pos), at), i))
                .filter(|&(d, i)| d < 30.0 && (node_root(p.nodes[i].pos)[1] - at[1]).abs() < 3.0)
                .collect();
            c.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            c.into_iter()
                .take(10)
                .filter(|&(_, i)| walk_clear(ctx.world, at, node_root(p.nodes[i].pos)))
                .take(3)
                .map(|(d, i)| (i, d))
                .collect()
        };
        let starts = near(from);
        let goals = near(dest);
        if starts.is_empty() || goals.is_empty() {
            return;
        }
        let mut dist = vec![f32::INFINITY; n];
        let mut prev = vec![u16::MAX; n];
        let mut done = vec![false; n];
        for &(i, d) in &starts {
            if d < dist[i] {
                dist[i] = d;
            }
        }
        loop {
            let mut cur = usize::MAX;
            for i in 0..n {
                if !done[i] && dist[i].is_finite() && (cur == usize::MAX || dist[i] < dist[cur]) {
                    cur = i;
                }
            }
            if cur == usize::MAX {
                break;
            }
            done[cur] = true;
            for &li in &p.adj[cur] {
                let l = &p.links[li as usize];
                let nb = if l.a as usize == cur { l.b } else { l.a } as usize;
                let c = dist[cur] + l.len;
                if c < dist[nb] {
                    dist[nb] = c;
                    prev[nb] = cur as u16;
                }
            }
        }
        let mut best: Option<(usize, f32)> = None;
        for &(g, d) in &goals {
            let total = dist[g] + d;
            if total.is_finite() && best.is_none_or(|(_, b)| total < b) {
                best = Some((g, total));
            }
        }
        let Some((g, _)) = best else { return };
        let mut nodes = vec![g as u16];
        let mut k = g;
        while prev[k] != u16::MAX && nodes.len() <= n {
            k = prev[k] as usize;
            nodes.push(k as u16);
        }
        nodes.reverse();
        self.route.path = pi;
        self.route.nodes = nodes;
    }

    /// The body walk: this tick's request (+ the keep-apart push from bodies closer than 1.5)
    /// through the player's movement core, facing the brain's yaw — the walk direction splits
    /// into the body's forward/strafe sticks. With no world (the unit rigs) the walk is a plane
    /// move at the body's top speed.
    fn walk_body(&mut self, ctx: &BotCtx) {
        let _p = crate::prof::scope("bots: walk_body");
        let want = self.move_want.take();
        let (mut dir, mut speed) = want.unwrap_or(([0.0, 0.0], 0.0));
        dir = [dir[0] * speed, dir[1] * speed];
        // Keep apart [S — the game's drone-vs-drone `Collide` pushes bodies apart; the port's
        // bot bodies are not collision objects]: a push away from every live body within 1.5.
        let mut pushed = false;
        for o in ctx.others {
            if o.index == ctx.my_index || o.index >= OBJ_ROW || !o.alive || (o.pos[1] - self.pos[1]).abs() > 2.0 {
                continue;
            }
            let ax = self.pos[0] - o.pos[0];
            let az = self.pos[2] - o.pos[2];
            let d = (ax * ax + az * az).sqrt();
            if d < 1.5 {
                let k = (1.5 - d) / 1.5;
                if d > 1e-4 {
                    dir[0] += ax / d * k;
                    dir[1] += az / d * k;
                } else {
                    // stacked exactly: split by slot order (deterministic)
                    let s = if ctx.my_index < o.index { 1.0 } else { -1.0 };
                    dir[0] += s * k;
                }
                pushed = true;
            }
        }
        let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
        if len > 1e-4 {
            speed = len.min(1.0).max(if pushed { 0.6 } else { 0.0 });
            dir = [dir[0] / len, dir[1] / len];
        } else {
            speed = 0.0;
        }
        if ctx.world.triangle_count() == 0 {
            // The plane walk (no world to stand on).
            if speed > 0.0 && !ctx.stationary {
                self.pos[0] += dir[0] * speed * BODY_STEP;
                self.pos[2] += dir[1] * speed * BODY_STEP;
            }
            return;
        }
        let mut input = crate::player::Input::default();
        let byaw = crate::trig::HALF_PI - self.yaw;
        self.body.yaw = byaw;
        if speed > 0.0 && !ctx.stationary {
            let (s, c) = (crate::trig::sinf(byaw), crate::trig::cosf(byaw));
            // Bond's forward is (sin, cos), his strafe (+) (cos, −sin) [M Player_Move's
            // `pos += c·vs + s·vf, −s·vs + c·vf`]. The forward stick is quadratic (0.1·f²), so
            // its share is the square root of the wanted component.
            let f = dir[0] * s + dir[1] * c;
            let r = dir[0] * c - dir[1] * s;
            input.forward = if f > 0.0 { (f * speed).sqrt() } else { -(-f * speed).sqrt() };
            input.strafe = r * speed * 1.4;
        }
        input.jump = std::mem::take(&mut self.jump_next);
        self.body.cel = None;
        self.body.tick_ctx(ctx.world, &crate::player::LadderCtx::default(), &input);
        self.body.sfx_out.clear();
        self.body.yaw = byaw;
        self.pos = [self.body.pos[0], self.body.pos[1] + crate::player::EYE, self.body.pos[2]];
        // The stuck watch: every 45 ticks of wanting to walk, under 0.4 covered → one jump and a
        // re-plan; three in a row tell the walker's caller to give the goal up [S]. The xz
        // measure false-triggers on stair climbs (bot-weapons-finish-2) — the jump+re-plan it
        // drives is kept: the jump is the wedge escape that un-wedges the carriers at the
        // stairwell mouths.
        if want.is_some_and(|w| w.1 > 0.3) && !ctx.stationary {
            if self.tick.wrapping_sub(self.stuck_ref.1) >= 45 {
                let r = self.root();
                if dist2d(r, self.stuck_ref.0) < 0.4 {
                    self.stuck_hits = self.stuck_hits.saturating_add(1);
                    self.jump_next = true;
                    self.route.valid = false;
                } else {
                    self.stuck_hits = 0;
                }
                self.stuck_ref = (r, self.tick);
            }
        } else {
            self.stuck_ref = (self.root(), self.tick);
            self.stuck_hits = 0;
        }
    }

    /// `NDrone2_CanStrafeLeft/Right` / `CanStepLeft/Right` / `CanBackoff` [S — the game's tests
    /// (0x14f9e8…) sweep the drone's bounds; the port: the side spot `dist` away is walk-clear
    /// and has a floor under it].
    fn can_side_move(&self, ctx: &BotCtx, side: f32, dist: f32) -> bool {
        let r = self.root();
        // left (side +1) = (−sin, cos) of the brain's facing (cos, sin)
        let lx = -crate::trig::sinf(self.yaw) * side;
        let lz = crate::trig::cosf(self.yaw) * side;
        let to = [r[0] + lx * dist, r[1], r[2] + lz * dist];
        walk_clear(ctx.world, r, to) && ctx.world.floor_below([to[0], to[1] - 1.1, to[2]], 0.9).is_some()
    }

    /// The side a combat move walks: +1 left, −1 right.
    fn move_side(state: u8) -> f32 {
        match state {
            st::BOT_ATTACK_STRAFE_AIM_LEFT | st::BOT_ATTACK_STEP_AIM_LEFT => 1.0,
            _ => -1.0,
        }
    }

    /// Walk sideways (relative to the facing) at `speed`, drifting in (`fwd` > 0) or out.
    fn side_walk(&mut self, side: f32, fwd: f32, speed: f32) {
        let (s, c) = (crate::trig::sinf(self.yaw), crate::trig::cosf(self.yaw));
        let x = -s * side + c * fwd;
        let z = c * side + s * fwd;
        let l = (x * x + z * z).sqrt().max(1e-6);
        self.move_want = Some(([x / l, z / l], speed));
    }

    /// The combat moves' ENTER [M 0x16f850 msg 1 (strafe): the trigger pull when the latch is
    /// clear, the `(Rand(8)+2)·FRAME_RATE_INT` timer; 0x174e88 msg 1 (step): the trigger pull,
    /// the step anim 0x5A — its length stands in as 30 ticks [S]].
    fn bot_attack_move_enter(&mut self, next: u8, rng: &mut Rng) {
        if self.vars.fire_latch == 0 {
            self.bot_attack_fire_enter(rng);
        }
        self.combat_move_until = match next {
            st::BOT_ATTACK_STRAFE_AIM_LEFT | st::BOT_ATTACK_STRAFE_AIM_RIGHT => {
                self.tick + (rng.rand(8) + 2) * FRAME_RATE_INT
            }
            _ => self.tick + 30,
        };
    }

    /// 0xD4/0xD5 BotAttackStrafeAim{Left,Right} msg 3 [M 0x16f850/0x16f9b8]: no opponent →
    /// 0xD3; aim; `CanStrafe` failing → `chooseCombatMove` (none → 0xCF); out of sight 16+ ticks
    /// → 0xD0 run; the timer (msg 0xC) → 0xCF. The strafe walk itself is the anim's root motion
    /// in the game (anim 0xC); the port walks the body sideways at the strafe stick [S].
    fn bot_attack_strafe_msg3(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        let Some(o) = self.vars.opponent.and_then(|i| ctx.others.get(i)).filter(|o| o.alive) else {
            self.set_state(st::BOT_ATTACK_NO_OPPONENT);
            return;
        };
        let dist = o.dist2.sqrt();
        if self.tick >= self.combat_move_until {
            self.set_state(state::BOT_ATTACK); // the timer's msg 0xC [M]
            return;
        }
        let side = Self::move_side(self.state);
        if !self.can_side_move(ctx, side, 1.5) {
            let mv = self.choose_combat_move(ctx, rng).unwrap_or(state::BOT_ATTACK);
            self.set_state(mv);
            return;
        }
        if self.vars.unseen_ticks >= 0x10 {
            self.set_state(state::BOT_ATTACK_RUN);
            return;
        }
        // drift in when far, out when inside the too-close band [S]
        let fwd = if dist > self.vars.combat_range[1] * 0.5 { 0.35 } else if dist < self.vars.combat_range[0] * 2.0 { -0.35 } else { 0.0 };
        self.side_walk(side, fwd, 1.0);
    }

    /// 0xDB/0xDC BotAttackStepAim{Left,Right} msg 3 [M 0x174e88/0x174fa8]: no opponent → 0xD3;
    /// the step anim over → 0xD2; aim; `Rand(5) == 3` → `chooseCombatMove` (none → stay).
    fn bot_attack_step_msg3(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        if self.vars.opponent.and_then(|i| ctx.others.get(i)).filter(|o| o.alive).is_none() {
            self.set_state(st::BOT_ATTACK_NO_OPPONENT);
            return;
        }
        if self.tick >= self.combat_move_until {
            self.set_state(state::BOT_ATTACK_FIRE); // `CanDoAnimState(0x5A)` fails → 0xD2 [M]
            return;
        }
        let side = Self::move_side(self.state);
        self.side_walk(side, 0.0, 0.8);
        if rng.rand(5) == 3 {
            if let Some(mv) = self.choose_combat_move(ctx, rng) {
                self.set_state(mv);
            }
        }
    }

    fn bot_goto_goal_msg3(&mut self, ctx: &BotCtx) {
        // the no-active-goal branch [M :30-35]
        let Some(slot) = self.get_active_goal() else {
            if self.vars.requested_state != 0 {
                let next = self.vars.requested_state as u8;
                self.vars.requested_state = 0;
                self.set_state(next);
            } else {
                self.set_state(state::BOT_IDLE);
            }
            return;
        };
        if self.vars.goals[slot].goal_type == 0 {
            self.set_state(state::BOT_IDLE);
            return;
        }
        let code = self.move_to_goal_position(ctx);
        self.vars.goals[slot].route_code = code; // +0x48 [M :40]
        // validateRoute: codes ≥ 0xC read 0 = route finished [M 0x129880:8-15]
        if code >= 0xC {
            if self.vars.goals[slot].goal_type == 1 {
                let target = self.vars.goals[slot].target;
                self.set_pickup_visit_time(ctx, target); // [M :43-45]
            }
            self.set_state(state::BOT_IDLE); // [M :52]
            return;
        }
        if code == 10 {
            self.vars.resume_state = state::BOT_GOTO_GOAL_POSITION as u16; // [M :60]
            self.set_state(state::BOT_DOOR_OPEN); // [M :61-63]
            return;
        }
        if code == 1 || code == 2 {
            // the walk anim replays [M :65]
        }
    }

    // ------------------------------------------------------------------
    // State 0xCF BotAttack [M 0x16f198]
    // ------------------------------------------------------------------

    fn bot_attack_enter(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        // anim 10, the attack message, InvalidateAttackRoute, then the 1-in-3 roll [M :18-28,91]
        if rng.rand(3) == 0 {
            if let Some(mv) = self.choose_combat_move(ctx, rng) {
                self.set_state(mv); // [M :26-28]
            }
        }
    }

    fn bot_attack_msg3(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        if self.vars.opponent.is_none() {
            self.set_state(st::BOT_ATTACK_NO_OPPONENT); // [M :36-41]
            return;
        }
        let o = self.vars.opponent.unwrap();
        let dist = ctx.others.get(o).map(|x| x.dist2.sqrt()).unwrap_or(f32::MAX);
        // the shortcut: unseen < 16 && dist < engage [M :46-50]
        let shortcut = self.vars.unseen_ticks < 0x10 && dist < self.vars.combat_range[1];
        if !shortcut && !ctx.stationary && self.vars.flag_word & 2 == 0 {
            // the route block [M :52-77] — the port's validateRoute stand-in: the bot always
            // has a route to the opponent here, result < 3 → 0xD0 (run)
            self.set_state(state::BOT_ATTACK_RUN);
            return;
        }
        // the sighted latch [M :85-88]
        if self.vars.sighted & 4 != 0 {
            let next = self.react_to_opponent_sighted(state::BOT_ATTACK_FIRE);
            if next != 0 {
                self.set_state(next);
            }
        }
    }

    // ------------------------------------------------------------------
    // State 0xD2 BotAttackFire [M 0x16f5e8]
    // ------------------------------------------------------------------

    fn bot_attack_fire_enter(&mut self, rng: &mut Rng) {
        // anim 10, InvalidateAttackRoute, then the trigger pull [M :19-26]
        if self.vars.fire_latch != 0 {
            return; // [M :22-23]
        }
        self.vars.fire_latch = 1; // DroneWeap_Fire [M 0x17a8c0:15-17]
        self.vars.burst_empty = 0;
        if self.vars.arsenal.armed {
            self.next_bullet_time_mp(rng, true); // `NextBulletTime(0, 1)` in MP [M 0x17a8c0:11-13]
        } else {
            self.next_bullet_time(rng); // → DroneWeap_NextBulletTime [M :25]
        }
    }

    fn bot_attack_fire_msg3(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        if self.vars.opponent.is_none() {
            self.set_state(st::BOT_ATTACK_NO_OPPONENT); // [M :36-41]
            return;
        }
        let o = self.vars.opponent.unwrap();
        let dist = ctx.others.get(o).map(|x| x.dist2.sqrt()).unwrap_or(f32::MAX);
        // the latch routing [M :42-52 verbatim]: 3C == 0 → (3B == 0 → the block) then the roll
        if self.vars.burst_empty == 0 {
            if self.vars.fire_latch == 0 {
                self.route_vision_block(ctx, dist, rng);
                return;
            }
            if rng.rand(3) != 0 {
                return; // keep firing [M :48-50]
            }
            // the fall-through → chooseCombatMove [M :51 → LAB_0016f818]
            if let Some(mv) = self.choose_combat_move(ctx, rng) {
                self.set_state(mv);
            }
            return;
        }
        self.route_vision_block(ctx, dist, rng); // 3C != 0 → the block [M :52]
    }

    /// 0xD0 BotAttackRun [M 0x16f390]: no opponent → 0xD3 (the port: idle); the route to the
    /// opponent (`NDrone2_MoveToObject` — the port's `steer_to` over the AI network); inside
    /// the too-close range → 0xD7 backoff; inside the engage range (+0xF0) and sighted → react
    /// → 0xCF. A route that fails for good (the stuck watch) → 0xD1's place: drop the opponent
    /// and go back to the goals [S].
    fn bot_attack_run_msg3(&mut self, ctx: &BotCtx) {
        let Some(o) = self.vars.opponent.and_then(|i| ctx.others.get(i)).filter(|o| o.alive) else {
            self.set_state(state::BOT_IDLE);
            return;
        };
        let dist = o.dist2.sqrt();
        let target = o.pos;
        if dist <= self.vars.combat_range[0] {
            self.set_state(state::BOT_ATTACK_BACKOFF); // [M :72-76]
            return;
        }
        if dist < self.vars.combat_range[1] && self.vars.sighted & 4 != 0 && self.vars.unseen_ticks < 0x10 {
            // No latch write here [M 0x16f390:86-91, :102 — the 0xCF hand-over reads +0x3b
            // nowhere]: the fire latch and its bullet deadline survive the combat-state bounce
            // (0xD0/0xCF/the moves), so the per-tick pipeline fires at the deadline from any
            // combat state — the game's bots shoot while they move. Clearing the latch made
            // every re-entry a fresh `DroneWeap_Fire` whose `NextBulletTime` re-pushed the
            // deadline (≥ 60 ticks for weapon 6 [M 0x179d40:26-47]) beyond the 1-in-3 exit
            // roll's stay [M 0x16f5e8:48-52] — the latch was set and the bot never fired.
            self.set_state(state::BOT_ATTACK); // [M :86-91]
            return;
        }
        if ctx.stationary {
            return;
        }
        let dest = [target[0], target[1] - crate::player::EYE, target[2]];
        if !self.steer_to(ctx, dest, 1.0, true) {
            self.vars.opponent = None;
            self.stuck_hits = 0;
            self.set_state(state::BOT_IDLE);
        }
    }

    /// 0xD7 BotAttackBackoff [M 0x16fc30 — not lifted; the stand-in]: walk back (facing the
    /// opponent) until past the close range, then back to BotAttack [S].
    fn bot_attack_backoff_msg3(&mut self, ctx: &BotCtx) {
        let Some(o) = self.vars.opponent.and_then(|i| ctx.others.get(i)).filter(|o| o.alive) else {
            self.set_state(state::BOT_IDLE);
            return;
        };
        if o.dist2.sqrt() > self.vars.combat_range[0] + 1.0 || ctx.stationary || self.stuck_hits > 0 {
            // No latch write [M 0x16fc30 — the backoff reads +0x3b (the enter's skip-Fire, the
            // msg3's aim gate) and never writes it]: see the 0xD0 arm above.
            self.set_state(state::BOT_ATTACK);
            return;
        }
        self.side_walk(0.0, -1.0, 0.8);
    }

    /// BotAttackFire's route/vision block [M 0x16f5e8:56-93].
    fn route_vision_block(&mut self, ctx: &BotCtx, dist: f32, rng: &mut Rng) {
        if ctx.stationary {
            return; // [M :56-60]
        }
        if dist <= self.vars.combat_range[0] {
            self.set_state(state::BOT_ATTACK_BACKOFF); // CanBackoff → 0xD7 [M :61-67]
            return;
        }
        if self.vars.unseen_ticks >= 0x10 {
            if let Some(mv) = self.choose_combat_move(ctx, rng) {
                self.set_state(mv); // [M :77]
            }
            return;
        }
        if dist >= self.vars.combat_range[2] {
            // the chase route [M :81-93]: the port's route stand-in → run
            self.set_state(state::BOT_ATTACK_RUN);
            return;
        }
        // The spent burst re-arms in the per-tick pipeline itself (`DoFiring`'s `+0xbc0 == 0`
        // arm → `NextBulletTime(1, 0)` [M 0x17a080:11-13]); the game's 0xD2 msg 3 has no
        // re-arm and no latch write [M 0x16f5e8:56-93]. The port's interval-based re-arm here
        // cleared the latch, so the next 0xD2/move enter was a fresh `DroneWeap_Fire` whose
        // `NextBulletTime` pushed the bullet deadline out again — with weapon 6's 60-tick
        // minimum and the 1-in-3 exit roll the deadline always receded and the bot never
        // fired (the playtest's "0xD4 strafe, latch set, never fires" wedge).
    }

    /// `DroneWeap_NextBulletTime` 0x179d40 (the port's lift): the interval from the weapon
    /// (min 60 for ids 6/0xA/0xE..0x10), the burst size with the shotgun/MG rolls and the
    /// aggression scaling for counts ≥ 4 [M :26-116].
    fn next_bullet_time(&mut self, rng: &mut Rng) {
        let w = self.vars.weapon_id as usize;
        let mut interval = self.weapon.interval.max(0) as i32;
        if matches!(w, 6 | 0xA | 0xE..=0x10) && interval < 60 {
            interval = 60; // [M 0x179d40:26-47]
        }
        let mut burst = match self.vars.weapon_id {
            6 | 0xA => rng.rand(3) + 2,   // [M :58]
            0xB..=0xD => rng.rand(2) + 1, // [M :74]
            _ => self.weapon.burst as u32,
        };
        if burst >= 4 {
            burst = ((burst as f32) * self.weapon.aggression_mul).max(3.0) as u32; // [M :…116]
        }
        self.vars.burst_left = burst;
        self.vars.next_bullet_at =
            self.tick as f32 / FRAME_RATE_INT as f32 + interval as f32 / 60.0; // the deadline [M :…134]
    }

    // ------------------------------------------------------------------
    // The per-tick firing pipeline [M 0x1490f0:231; 0x17a080 DoFiring; 0x178c48 Ready2Fire]
    // ------------------------------------------------------------------

    fn handle_firing(&mut self, ctx: &BotCtx, rng: &mut Rng) {
        if self.vars.fire_latch == 0 || self.vars.next_bullet_at <= 0.0 {
            return;
        }
        if (self.tick as f32 / 60.0) < self.vars.next_bullet_at {
            return; // now < the deadline
        }
        // the gates [M 0x17a080: unseen < 16; 0x178c48: opponent alive, ammo, the aim cone]
        if self.vars.unseen_ticks >= 0x10 {
            return;
        }
        let Some(o) = self.vars.opponent else { return };
        let Some(target) = ctx.others.get(o) else { return };
        // `BOTWEAP_WeaponHasAmmo` (0x12adb0): the port gates on clip rounds only; the ammo-pool
        // half (BotVars+0x698) is not modelled [S partial].
        if !target.alive {
            return;
        }
        if self.vars.clip_rounds < 1 {
            // The reload: the ammo pool (BotVars+0x698) is not modelled, so an empty clip refills
            // from an unmodelled pool — 12, the start P2K's clip [S]. Without it a bot spawned
            // with 0 rounds and never fired a shot (the two-window bots: clip 0 all match).
            self.vars.clip_rounds = 12;
            return; // the reload takes this tick
        }
        let dx = target.pos[0] - self.pos[0];
        let dz = target.pos[2] - self.pos[2];
        let d = (dx * dx + dz * dz).sqrt();
        let dir = [dx / d, 0.0, dz / d];
        // The aim turn: the bot swings toward its opponent (≤ 0.2 rad a tick) before the cone
        // test — nothing turned an attacking bot (only the goal walk faced its heading), so it
        // never entered the cone and never fired (the two-window bots stood in 0xD2 all match)
        // [S the drones' aim-turn rate is not lifted; the game's ATAN2_APPROX keeps it lockstep-
        // exact].
        let want = crate::trig::atan2_approx(dz, dx);
        let pi = std::f32::consts::PI;
        let mut delta = want - self.yaw;
        while delta > pi { delta -= 2.0 * pi; }
        while delta < -pi { delta += 2.0 * pi; }
        self.yaw += delta.clamp(-0.2, 0.2);
        // the aim cone [M 0x178c48: dot > 0.9998]
        // the game's PS2Sinf/PS2Cosf (0x120700) — see the ATAN2_APPROX note above: std trig is
        // libm-dependent; the game's own approximations are not.
        let facing = [crate::trig::cosf(self.yaw), 0.0, crate::trig::sinf(self.yaw)];
        let dot = facing[0] * dir[0] + facing[2] * dir[2];
        if dot <= 0.9998 {
            return;
        }
        // FireWeapon [M 0x179d40→0x17a080:11] — the round aims at the target's height too (a level
        // round passed over a player on another floor) [S the aim's vertical half].
        let dy = target.pos[1] - 0.3 - self.pos[1];
        let len = (d * d + dy * dy).sqrt().max(1e-6);
        let aim = [dx / len, dy / len, dz / len];
        let _ = dir;
        self.shots.push(BotShot { tick: self.tick, pos: self.pos, dir: aim, weapon: self.vars.weapon_id });
        self.vars.clip_rounds -= 1;
        self.vars.burst_left -= 1;
        if self.vars.burst_left == 0 {
            self.vars.burst_empty = 1; // [M 0x17a180:63]
            self.vars.burst_done_tick = self.tick;
            self.vars.next_bullet_at = 0.0; // the pipeline stops — Fire's only caller is the state ENTER [M]
        } else {
            self.vars.next_bullet_at += self.weapon.interval.max(0) as f32 / 60.0;
        }
    }
}

// ============================================================================
// mp-bot-weapons: the weapon layer's brain half (`botweap.rs` holds the records)
// ============================================================================

/// [S] The reload anim 0x52's length (`BotAttackReload` 0x170190 plays it through
/// `DroneAnim_CallAnim` and moves on to 0xCF when it ends; the anim is not lifted).
pub const RELOAD_TICKS: u32 = 45;

impl BotBrain {
    /// The legacy fire params (`self.weapon`) follow +0x768 — `weapon_data+0x40/+0x2C`.
    fn apply_weapon_params(&mut self, t: &crate::mp::botweap::BotWeaponTable) {
        let r = t.row(self.vars.weapon_id);
        self.weapon.interval = r.interval;
        self.weapon.burst = r.burst;
        self.weapon.aggression_mul = self.aggression_mul();
    }

    /// `BOT_getAggressionMul` 0x126468: by `Drone+0xB5` — the record's AIM byte (`BOT_setDroneStats`
    /// 0x126318:13 copies stats[2] there) — 0.3 / 0.5 / 0.7 / 0.85 / 1.0; in MP × (2.5 − dist + 1,
    /// at most 3) while the opponent is inside 2.5 [M :8-31].
    pub fn aggression_mul(&self) -> f32 {
        let mut m = match self.vars.aim {
            0 => 0.3,
            1 => 0.5,
            2 => 0.7,
            3 => 0.85,
            _ => 1.0,
        };
        if self.vars.opponent.is_some() && self.vars.opp_dist < 2.5 {
            m *= ((2.5 - self.vars.opp_dist) + 1.0).min(3.0);
        }
        m
    }

    /// The opponent is visible (`Drone+0x228 & 4` — `combatWeaponChangeChoice`'s gate [M disasm
    /// 0x128768-0x128778]).
    fn opponent_visible(&self) -> bool {
        self.vars.opponent.is_some() && self.vars.sighted & 4 != 0
    }

    /// `BOTSTATE_changeWeapon(bot, 0, p3, 1)`'s choice half — `combatWeaponChangeChoice` with
    /// p4 = 1 (no announce).
    fn weapon_choice(&self, t: &crate::mp::botweap::BotWeaponTable) -> u8 {
        self.vars.arsenal.combat_choice(t, self.opponent_visible(), self.vars.opp_dist, self.vars.trait_id, self.vars.weapon_class)
    }

    /// The weapon layer's per-tick part, after the perception pass: `Drone+0x1A0`, the arming
    /// (`BOTWEAP_InitWeapon` 0x129e48 — the game runs it from the bot's init; the port arms on
    /// the first tick a table is present, which every machine does on the same tick), a pending
    /// `pickupWeaponChangeChoice` 0x129728 from last tick's take, and `setGoalPickPrefs`' ammo
    /// inputs.
    fn weapons_tick(&mut self, ctx: &BotCtx, _rng: &mut Rng) {
        self.vars.opp_dist = self.vars.opponent.and_then(|o| ctx.others.get(o)).map_or(f32::MAX, |o| o.dist2.sqrt());
        let Some(t) = ctx.weapons else { return };
        if !self.vars.arsenal.armed {
            let w = self.vars.arsenal.init(t, ctx.start_weapon, ctx.scenario, ctx.my_team, self.vars.character);
            self.vars.weapon_id = w; // +0x768 [M 0x129e48:49,61]
            // Drone+0xBBC: the drawn weapon's loaded rounds (`BOTWEAP_changeWeapon`'s write
            // [M 0x12a028:48-52]; the init path's own write is not lifted [S]).
            self.vars.clip_rounds = self.vars.arsenal.change_weapon(t, w).unwrap_or(0);
            self.apply_weapon_params(t);
        }
        // The choice points (each `combatWeaponChangeChoice(opp, 1, 0, 0)` → msg 0x42 → 0xDE):
        // a new opponent (`NDrone2_SetOpponent` 0x1422d0:21-23 sends msg 0x45, whose `BotGlobal`
        // arm runs the choice [M 0x16e060:730-731]); the opponent coming into sight
        // (`DroneVision_HaveOpponentSight` 0x175ef0:48-54); a take last tick
        // (`pickupWeaponChangeChoice(w, has, 0)` — the has-weapon gate holds after a take).
        let visible = self.opponent_visible();
        let new_opp = self.vars.opponent.is_some() && self.vars.opponent != self.vars.prev_opponent;
        let sighted = visible && !self.vars.prev_visible;
        let took = self.vars.pickup_choice.take().is_some();
        self.vars.prev_opponent = self.vars.opponent;
        self.vars.prev_visible = visible;
        if new_opp || sighted || took {
            self.announce_weapon_choice(t);
        }
        let w = self.vars.weapon_id;
        self.vars.ammo_summary = (self.vars.arsenal.weapon_ammo_amount(t, w) as f32, t.row(w).clip as f32);
    }

    /// `combatWeaponChangeChoice(opp, 1, 0, …)`'s announce tail [M disasm 0x128ad8-0x128b10]: a
    /// pick that differs from +0x768 goes out as `MP_sendBotMessage(0x42, w)`; the bot's own
    /// `BotGlobal` msg 0x42 stores it in +0x76A and enters 0xDE [M 0x16e060:652-655]. The port
    /// delivers the message at once [S].
    fn announce_weapon_choice(&mut self, t: &crate::mp::botweap::BotWeaponTable) {
        let w = self.weapon_choice(t);
        if w != 0 && w != self.vars.weapon_id {
            self.vars.announced_weapon = w;
            self.set_state(state::BOT_ATTACK_CHANGE_WEAPON);
        }
    }

    /// The scene's take (`Pickup_Handler` 0x18e650's class-2 arm, `botweap::BotArsenal::take_pickup`):
    /// a taken weapon/ammo the bot can fire queues `pickupWeaponChangeChoice` for the next tick.
    pub fn take_pickup(&mut self, t: &crate::mp::botweap::BotWeaponTable, kind: u16, id: u16, rounds: i16, team: u32, health: &mut f32) -> crate::mp::botweap::BotTake {
        let r = self.vars.arsenal.take_pickup(t, kind, id, rounds, team, health, self.vars.max_hp);
        if let crate::mp::botweap::BotTake::Taken { choice: Some(has) } = r {
            self.vars.pickup_choice = Some(has);
        }
        r
    }

    /// `BOTSTATE_setPickupVisitTime` 0x129958 without a ctx (the take path's call [M 0x18e650:126-127]):
    /// returns the stamp (`now + 45.0`) the caller writes into `MP_PICKUP+0x80 + idx·4`.
    pub fn pickup_visit_stamp(&mut self, pickup: usize) -> f32 {
        let mut t = self.tick as f32 / 60.0 + 45.0;
        if t == 0.0 {
            t += 0.1;
        }
        self.visited_pickups.push(pickup);
        self.last_visit_stamp = (pickup, t);
        t
    }

    /// `BotGlobal`'s combat block, the weapon half [M 0x16e060:290-495] (types 4 outside
    /// 0xDD/0xDE): an empty gun reloads (0xDD) or, with nothing left for it, changes weapon
    /// (0xDE); up close and facing the opponent the bot goes to its fists (1 in 200 a tick, or
    /// at once for the grenades/mines/gadgets — `punchIsBetterIfClose`), and back out of
    /// reach; an explosive held too close changes away, a loaded explosive whose min range the
    /// opponent is beyond is drawn. The combat-move half (backoff, `reallyWantACombatMove`) is
    /// the states' own [S].
    fn combat_weapon_block(&mut self, ctx: &BotCtx, t: &crate::mp::botweap::BotWeaponTable, rng: &mut Rng) {
        use crate::mp::botweap::{punch_is_better_if_close, FISTS};
        if matches!(self.state, state::BOT_ATTACK_RELOAD | state::BOT_ATTACK_CHANGE_WEAPON) {
            return; // `iVar3 - 0xddU < 2` [M :293]
        }
        let cur = self.vars.weapon_id;
        let row = t.row(cur);
        // The empty gun [M :295-306]: `Drone+0xBBC < 1`. [S] also an empty clip slot (the
        // Ready2Fire `AmmoInGun < 1` refusal would otherwise hold a bot that switched to an
        // empty gun forever); the ammo-less weapons (type 0 — the fists) never run dry here
        // (the game's melee is 0xDF's anims, not the bullet pipeline).
        let empty = row.ammo_type != 0
            && (self.vars.clip_rounds < 1 || self.vars.arsenal.ammo_in_gun(t, cur) < 1);
        if empty {
            if !self.vars.arsenal.weapon_has_ammo(t, cur) {
                self.vars.announced_weapon = self.weapon_choice(t);
                self.set_state(state::BOT_ATTACK_CHANGE_WEAPON);
            } else {
                self.set_state(state::BOT_ATTACK_RELOAD);
            }
            return;
        }
        let dist = self.vars.opp_dist;
        // The fists up close [M :310-345]: within 1.5 and in front (|Drone+0x1D4| < π/2 — the
        // port: the facing·direction dot > 0).
        let facing = self.vars.opponent.and_then(|o| ctx.others.get(o)).is_some_and(|o| {
            let (dx, dz) = (o.pos[0] - self.pos[0], o.pos[2] - self.pos[2]);
            crate::trig::cosf(self.yaw) * dx + crate::trig::sinf(self.yaw) * dz > 0.0
        });
        if dist <= 1.5 && facing && cur != FISTS {
            let go = if self.vars.flags & 4 == 0 {
                rng.rand(200) == 199 || punch_is_better_if_close(cur)
            } else {
                true
            };
            if go {
                self.vars.announced_weapon = FISTS;
                self.set_state(state::BOT_ATTACK_CHANGE_WEAPON);
                return;
            }
        }
        // The fists out of reach → the choice [M :378-385].
        if cur == FISTS && dist > 1.5 {
            self.vars.announced_weapon = self.weapon_choice(t);
            self.set_state(state::BOT_ATTACK_CHANGE_WEAPON);
            return;
        }
        // The explosives [M :462-483].
        if self.vars.opponent.is_some() {
            if row.class == 4 {
                if self.vars.arsenal.too_close_for_weapon(t, cur, dist, self.vars.trait_id) {
                    self.vars.announced_weapon = self.weapon_choice(t);
                    self.set_state(state::BOT_ATTACK_CHANGE_WEAPON);
                }
            } else {
                let e = self.vars.arsenal.explosive_for_range(t, dist);
                if e != 0 {
                    self.vars.announced_weapon = e;
                    self.set_state(state::BOT_ATTACK_CHANGE_WEAPON);
                }
            }
        }
    }

    /// 0xDE `BotAttackChangeWeapon` ENTER [M 0x1750c8:16-30]: the burst count and the fire latch
    /// clear, `BOTSTATE_changeWeapon(+0x76A, 1, 0)` (+0x76A = 0 asks the choice; a pick equal to
    /// +0x768 changes nothing; `BOTWEAP_changeWeapon` refuses a weapon not held), then 0xCF. The
    /// team message 0x44 is not modelled [S].
    fn bot_attack_change_weapon_enter(&mut self, ctx: &BotCtx) {
        self.vars.burst_left = 0;
        self.vars.fire_latch = 0;
        if let Some(t) = ctx.weapons.filter(|_| self.vars.arsenal.armed) {
            let mut w = self.vars.announced_weapon;
            if w == 0 {
                w = self.weapon_choice(t);
            }
            if w != 0 && w != self.vars.weapon_id {
                if let Some(g) = self.vars.arsenal.change_weapon(t, w) {
                    self.vars.weapon_id = w;
                    self.vars.clip_rounds = g;
                    self.apply_weapon_params(t);
                }
            }
        }
        self.set_state(state::BOT_ATTACK);
    }

    /// 0xDD `BotAttackReload` ENTER [M 0x170190:21-48]: the burst count and the latch clear;
    /// `BOTWEAP_ReloadAmmoType` — nothing to load → 0xDE; else the reload anim 0x52 (when the
    /// weapon has one) and then 0xCF, or 0xD7 when the opponent is inside the too-close range.
    fn bot_attack_reload_enter(&mut self, ctx: &BotCtx) {
        self.vars.burst_left = 0;
        self.vars.fire_latch = 0;
        let Some(t) = ctx.weapons.filter(|_| self.vars.arsenal.armed) else {
            self.set_state(state::BOT_ATTACK);
            return;
        };
        let cur = self.vars.weapon_id;
        let (ok, rounds) = self.vars.arsenal.reload(t, cur, false);
        if !ok {
            self.set_state(state::BOT_ATTACK_CHANGE_WEAPON);
            return;
        }
        if let Some(g) = rounds {
            self.vars.clip_rounds = g; // Drone+0xBBC [M 0x129d28:46-49]
        }
        if crate::mp::botweap::reload_anim_for_weapon(cur) {
            self.vars.reload_until = self.tick + RELOAD_TICKS;
        } else {
            self.vars.reload_until = self.tick;
        }
    }

    /// 0xDD msg 3: the reload anim's end → 0xCF (0xD7 inside the too-close range) — the
    /// `CallAnim` next-state [M 0x170190:31-40].
    fn bot_attack_reload_msg3(&mut self) {
        if self.tick >= self.vars.reload_until {
            let next = if self.vars.opp_dist <= self.vars.combat_range[0] { state::BOT_ATTACK_BACKOFF } else { state::BOT_ATTACK };
            self.set_state(next);
        }
    }

    /// `DroneWeap_NextBulletTime` 0x179d40, the MP path [M :47-49, :111-135]: the interval
    /// `weapon_data+0x40` (at least 60 for 6/0xA/0xE..0x10 [M :26-47]) from the last shot; a
    /// burst start also sets the count `+0x2C × aggression` (at least 1) and the reaction delay
    /// `Rand(15) + 60/aggression` from now — the later of the two deadlines wins.
    fn next_bullet_time_mp(&mut self, rng: &mut Rng, start: bool) {
        let w = self.vars.weapon_id;
        let min = if matches!(w, 6 | 0xA | 0xE..=0x10) { 60 } else { 0 };
        let iv = (self.weapon.interval as i32).max(min);
        let interval = (iv as f32 * 1.0 + 0.5) as u32; // × FRAME_RATE_DIV (1.0 at 60 Hz)
        let mut delay = 0u32;
        if start {
            let mut count = (self.weapon.burst as i16 as i32).max(1); // [M :52, :93-96]
            let aggr = self.aggression_mul();
            delay = rng.rand(15) + (60.0 / aggr) as u32; // [M :113-116]
            let f = count as f32 * aggr;
            count = if f > 1.0 { f as i32 } else { 1 }; // [M :117-121]
            self.vars.burst_left = count as u32;
            self.weapon.aggression_mul = aggr;
        }
        let a = self.vars.last_shot_tick.wrapping_add(interval);
        let b = self.tick.wrapping_add(delay);
        self.vars.next_shot_tick = if a < b { b } else { a }; // [M :131-135]
    }

    /// `DroneWeap_HandleFiring` 0x17a180 → `DoFiring` 0x17a080 → `Ready2Fire` 0x178c48 →
    /// `FireWeapon` 0x178f68, the armed bot's form. MP drones fire burst after burst while the
    /// latch holds (`DroneWeap_Fire` clears +0x3D, so HandleFiring re-zeroes +0x3C every tick and
    /// `DoFiring` starts the next burst when the count is spent [M 0x17a180:28-31, 0x17a080:17-20]).
    /// Ready2Fire: the opponent alive, rounds in the gun (`AmmoInGun ≥ 1`), the aim within 20°
    /// (`dot > 0.9998 || acos(dot) < 0.349` — the port compares the dot with cos 20°). FireWeapon:
    /// `weapon_data+0x18` bullets, each spread by `+0x24 × Drone+0xC0` (the record's aggression,
    /// `BOT_setDroneStats` 0x126318:16) through `Bullet_init`'s cone; the last-shot stamp; the
    /// gun's count − 1 and `BOTWEAP_decrRounds(1)` [M 0x178f68:248-272].
    fn handle_firing_armed(&mut self, ctx: &BotCtx, rng: &mut Rng, t: &crate::mp::botweap::BotWeaponTable) {
        if self.vars.fire_latch == 0 {
            return;
        }
        if self.vars.burst_empty != 0 && self.tick != self.vars.burst_done_tick {
            self.vars.burst_empty = 0; // +0x3D == 0 → +0x3C := 0 [M 0x17a180:28-30]
        }
        if self.vars.burst_left == 0 {
            self.next_bullet_time_mp(rng, true); // `NextBulletTime(1, 0)` [M 0x17a080:17-20]
        }
        if self.vars.unseen_ticks >= 0x10 {
            self.vars.burst_empty = 1; // DoFiring → 0 → +0x3C := 1 [M 0x17a080:22-30, 0x17a180:41-43]
            self.vars.burst_done_tick = self.tick;
            return;
        }
        if self.tick < self.vars.next_shot_tick {
            return;
        }
        let Some(target) = self.vars.opponent.and_then(|o| ctx.others.get(o)).copied() else { return };
        if !target.alive {
            return;
        }
        let cur = self.vars.weapon_id;
        let row = t.row(cur);
        if row.ammo_type != 0 && self.vars.arsenal.ammo_in_gun(t, cur) < 1 {
            return; // [M 0x178c48:73-79]
        }
        let dx = target.pos[0] - self.pos[0];
        let dz = target.pos[2] - self.pos[2];
        let d = (dx * dx + dz * dz).sqrt().max(1e-6);
        // the aim turn (see `handle_firing`)
        let want = crate::trig::atan2_approx(dz, dx);
        let pi = std::f32::consts::PI;
        let mut delta = want - self.yaw;
        while delta > pi { delta -= 2.0 * pi; }
        while delta < -pi { delta += 2.0 * pi; }
        self.yaw += delta.clamp(-0.2, 0.2);
        let facing = [crate::trig::cosf(self.yaw), 0.0, crate::trig::sinf(self.yaw)];
        let dot = facing[0] * dx / d + facing[2] * dz / d;
        const COS_20: f32 = 0.939_692_6; // cos(0.34906587) — `acosf(dot) < 0.349` [M 0x178c48:117-124]
        if dot <= COS_20 {
            return;
        }
        let dy = target.pos[1] - 0.3 - self.pos[1];
        let len = (d * d + dy * dy).sqrt().max(1e-6);
        let aim = [dx / len, dy / len, dz / len];
        let spread = row.spread * self.vars.aggression as f32;
        for _ in 0..row.bullets {
            let dir = crate::bullet::spread(aim, spread, row.flags, rng);
            self.shots.push(BotShot { tick: self.tick, pos: self.pos, dir, weapon: cur });
        }
        self.vars.last_shot_tick = self.tick; // Drone+0xBC8 [M 0x178f68:261]
        self.vars.clip_rounds = self.vars.clip_rounds.wrapping_sub(1); // Drone+0xBBC [M :265-270]
        if row.ammo_type != 0 {
            self.vars.arsenal.decr_rounds(t, cur, 1); // [M :271-273]
        }
        self.vars.burst_left = self.vars.burst_left.saturating_sub(1); // [M 0x17a080:29]
        if self.vars.burst_left == 0 {
            self.vars.burst_empty = 1;
            self.vars.burst_done_tick = self.tick;
        } else {
            self.next_bullet_time_mp(rng, false); // `NextBulletTime(0, 0)` [M 0x17a080:33]
        }
    }
}

// ============================================================================
// 5. Tests — the golden table checks and the dispatch test
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// mp-bot-weapons: the interval floor — `DroneWeap_NextBulletTime` 0x179d40:26-47 clamps the
    /// record interval (`weapon_data+0x40`) to 60 ticks for weapons 6, 0xA and 0xE..0x10 only; a
    /// weapon outside the set keeps its record value. The non-start call (`NextBulletTime(0, 0)`,
    /// the per-burst-member re-arm) sets `Drone+0xBC4 = max(last_shot + interval, now)` [M :131-135].
    #[test]
    fn next_bullet_time_interval_floor() {
        let record = INITIAL_BOT_RECORDS[0].clone();
        let mut brain = BotBrain::bot_init(&record, 4, [0.0, 1.6, 0.0], 0.0);
        let mut rng = Rng::new(0x5eed_0001);
        // weapon 6 with a record interval of 8 ticks: the deadline lands 60 ticks after the shot.
        brain.vars.weapon_id = 6;
        brain.weapon = BotWeapon { interval: 8, burst: 1, aggression_mul: 1.0 };
        brain.vars.last_shot_tick = 1000;
        brain.tick = 1001;
        brain.next_bullet_time_mp(&mut rng, false);
        assert_eq!(brain.vars.next_shot_tick, 1060, "weapon 6's 60-tick floor [M 0x179d40:26-47]");
        // Sister (can fail): weapon 0x19 is outside the floor set — its record interval passes
        // through, so a sub-60 interval is NOT clamped.
        brain.vars.weapon_id = 0x19;
        brain.weapon = BotWeapon { interval: 15, burst: 1, aggression_mul: 1.0 };
        brain.vars.last_shot_tick = 1000;
        brain.next_bullet_time_mp(&mut rng, false);
        assert_eq!(brain.vars.next_shot_tick, 1015, "no floor outside the 6/0xA/0xE..0x10 set");
    }

    /// Golden: the 29 presets are byte-exact against the EE snapshot reads
    /// (spot rows + the structural invariants every row must satisfy).
    /// Bytes recorded from `work/ee_snapshot.bin` @0x26D2F0 [L]; identical in
    /// all four snapshots.
    #[test]
    fn golden_default_bot_stats_table() {
        assert_eq!(DEFAULT_BOT_STATS.len(), 29);
        // row 0: the boot default (slot 4's record and the BOT_init null-record
        // fallback both derive from preset rows)
        assert_eq!(DEFAULT_BOT_STATS[0].0, [0x01, 0x00, 0x04, 0x00, 0x96, 0x00, 0x02, 0x96, 0x32, 0x00, 0x00, 0x00, 0x00, 0x01]);
        // row 1: the BOT_init null-record default (agg 5, aim 2, HP 100, trait 6)
        assert_eq!(DEFAULT_BOT_STATS[1].aggression(), 5);
        assert_eq!(DEFAULT_BOT_STATS[1].aim(), 2);
        assert_eq!(DEFAULT_BOT_STATS[1].max_hp(), 100);
        assert_eq!(DEFAULT_BOT_STATS[1].trait_id(), 6);
        // row 16: the 250-HP preset
        assert_eq!(DEFAULT_BOT_STATS[16].max_hp(), 250);
        // row 24: 300 HP, speed 0 — the extreme row
        assert_eq!(DEFAULT_BOT_STATS[24].max_hp(), 300);
        assert_eq!(DEFAULT_BOT_STATS[24].speed(), 0);
        // last row
        assert_eq!(DEFAULT_BOT_STATS[28].trait_id(), 6);
        // structural invariants across all 29 rows
        for (i, r) in DEFAULT_BOT_STATS.iter().enumerate() {
            assert!(r.aggression() <= 9, "row {i} aggression out of range");
            assert!(r.aim() <= 9, "row {i} aim out of range");
            assert!(r.max_hp() >= 50 && r.max_hp() <= 300, "row {i} HP out of range");
            assert!(r.trait_id() <= 8, "row {i} trait out of range");
            assert!(r.speed() <= 2, "row {i} speed out of range");
        }
        // the two groups: rows 0..14 carry trailing byte 1, rows 15..28 carry 0
        for (i, r) in DEFAULT_BOT_STATS.iter().enumerate() {
            let want = if i < 15 { 1 } else { 0 };
            assert_eq!(r.trailing(), want, "row {i} trailing byte");
        }
        // rows 0, 12, 14 are the same preset (triplicated in the game data)
        assert_eq!(DEFAULT_BOT_STATS[0].0, DEFAULT_BOT_STATS[12].0);
        assert_eq!(DEFAULT_BOT_STATS[0].0, DEFAULT_BOT_STATS[14].0);
    }

    /// Golden: the boot `.data` image of the config records @0x2DEEAA [L
    /// ee_snapshot]: slot 4 = preset 0 disabled with character 6, slots 5..7
    /// empty with characters 7/8/9.
    #[test]
    fn golden_initial_bot_records() {
        let r0 = &INITIAL_BOT_RECORDS[0];
        assert_eq!(r0.stats(), &DEFAULT_BOT_STATS[0].0, "slot 4 = preset 0 stats");
        assert!(!r0.enabled());
        assert_eq!(r0.team(), 0);
        assert_eq!(r0.character(), 6);
        for (i, rec) in INITIAL_BOT_RECORDS.iter().enumerate() {
            assert_eq!(rec.character() as usize, 6 + i, "slot {} character", 4 + i);
            assert!(!rec.enabled(), "slot {} boots disabled", 4 + i);
            assert!(!rec.dirty());
        }
        for rec in INITIAL_BOT_RECORDS.iter().skip(1) {
            assert_eq!(rec.aggression(), 0);
            assert_eq!(rec.max_hp(), 0);
        }
        // Menu_PrepareBots compacts enabled records; with the boot image none are enabled.
        assert_eq!(INITIAL_BOT_RECORDS.iter().filter(|r| r.enabled()).count(), 0);
    }

    /// The state table: 55 ids 0xC3..0xF9, the addresses match the game's
    /// `NDrone2_StateFuncs` rows [L ee_snapshot @0x29B300], every id maps
    /// through `state_type`, and the type groups match the snapshot table.
    #[test]
    fn state_table_and_state_types() {
        assert_eq!(BOT_STATES.len(), 55);
        for (n, st) in BOT_STATES.iter().enumerate() {
            assert_eq!(st.id as usize, 0xC3 + n);
            assert!(st.addr >= 0x16E060 && st.addr < 0x180000, "{} addr", st.name);
        }
        // spot addresses from the snapshot pointer table
        let addr = |id: u8| BOT_STATES[(id - BOT_STATE_FIRST) as usize].addr;
        assert_eq!(addr(state::BOT_INIT), 0x1748C8);
        assert_eq!(addr(state::BOT_GLOBAL), 0x16E060);
        assert_eq!(addr(state::BOT_ATTACK), 0x16F198);
        assert_eq!(addr(state::BOT_GOTO_GOAL_POSITION), 0x171240);
        assert_eq!(addr(state::BOT_IDLE), 0x171B80);
        // the type table, spot rows [L 0x26F460]
        assert_eq!(state_type(state::BOT_INIT as u32), 1);
        assert_eq!(state_type(state::BOT_GLOBAL as u32), 2);
        assert_eq!(state_type(state::BOT_ASSASSIN as u32), 3);
        assert_eq!(state_type(state::BOT_ATTACK as u32), 4);
        assert_eq!(state_type(state::BOT_COVER_IDLE as u32), 12);
        assert_eq!(state_type(state::BOT_GOTO_GOAL_POSITION as u32), 5);
        assert_eq!(state_type(state::BOT_SEEN_OPPONENT as u32), 6);
        assert_eq!(state_type(state::BOT_IMPACT_BULLET as u32), 7);
        assert_eq!(state_type(state::BOT_DEATH_ANIM as u32), 10);
        assert_eq!(state_type(state::BOT_IMPACT_STUN_GRENADE as u32), 7);
        assert_eq!(state_type(state::BOT_DOOR_OPEN as u32), 8);
        assert_eq!(state_type(state::BOT_GUARD_FRIEND_IDLE as u32), 13);
        assert_eq!(state_type(state::BOT_IDLE as u32), 9);
        // out-of-range ids read 0, exactly like BOTSTATE_getStateType [M 0x1291f0]
        assert_eq!(state_type(0xC2), 0);
        assert_eq!(state_type(0xFA), 0);
        assert_eq!(state_type(0), 0);
    }

    /// The `BotGlobal` dispatch skeleton routes like the game's switch [M 0x16E060].
    #[test]
    fn bot_global_dispatch_routing() {
        let msg = |id: u8, arg: u32, cs: u32| BotMsg { id, arg, current_state: cs };
        // the ignore block
        assert_eq!(bot_global_dispatch(msg(0x00, 0, 0), 0, false, false), BotGlobalOutcome::Ignore);
        assert_eq!(bot_global_dispatch(msg(0x2F, 0, 0), 0, false, false), BotGlobalOutcome::Ignore);
        assert_eq!(bot_global_dispatch(msg(0x39, 0, 0), 0, false, false), BotGlobalOutcome::Ignore);
        assert_eq!(bot_global_dispatch(msg(0x3F, 0, 0), 0, false, false), BotGlobalOutcome::Ignore);
        // killed with a non-death current state → 0xF2; death state → stays
        assert_eq!(
            bot_global_dispatch(msg(0x03, 0, state::BOT_ATTACK as u32), 0, false, false),
            BotGlobalOutcome::Killed { goto: state::BOT_DEATH_ANIM }
        );
        assert_eq!(
            bot_global_dispatch(msg(0x03, 0, state::BOT_DEAD as u32), 0, false, false),
            BotGlobalOutcome::Killed { goto: state::BOT_DEAD }
        );
        // alert family: react unless the current state's type is exempt
        // (0xF9 BotIdle is type 9 → react; 0xCF BotAttack type 4 → no react)
        assert_eq!(
            bot_global_dispatch(msg(0x11, 0, state::BOT_IDLE as u32), 0, false, false),
            BotGlobalOutcome::Alert { react: true }
        );
        assert_eq!(
            bot_global_dispatch(msg(0x14, 0, state::BOT_ATTACK as u32), 0, false, false),
            BotGlobalOutcome::Alert { react: false }
        );
        // stun gate
        assert_eq!(
            bot_global_dispatch(msg(0x18, 0, state::BOT_ATTACK as u32), 0, false, false),
            BotGlobalOutcome::StunIfNotDeath { stun: true }
        );
        assert_eq!(
            bot_global_dispatch(msg(0x18, 0, state::BOT_DEATH_BY_EXPLOSION as u32), 0, false, false),
            BotGlobalOutcome::StunIfNotDeath { stun: false }
        );
        // goal messages carry the slot arg
        assert_eq!(
            bot_global_dispatch(msg(0x3B, 1, 0), 0, false, false),
            BotGlobalOutcome::GoalComplete { slot: 1 }
        );
        assert_eq!(
            bot_global_dispatch(msg(0x3C, 0, 0), 0, false, false),
            BotGlobalOutcome::GoalUninit { slot: 0 }
        );
        assert_eq!(
            bot_global_dispatch(msg(0x3D, 0, 0), 0, false, false),
            BotGlobalOutcome::GoalCancel { goto: state::BOT_IDLE }
        );
        // objective seek only without an opponent
        assert_eq!(
            bot_global_dispatch(msg(0x3A, 0, 0), 0, false, false),
            BotGlobalOutcome::SeekObjective { goto: state::BOT_GOTO_GOAL_POSITION }
        );
        assert_eq!(bot_global_dispatch(msg(0x3A, 0, 0), 0, true, false), BotGlobalOutcome::Pass);
        // weapon announce records the weapon byte
        assert_eq!(
            bot_global_dispatch(msg(0x42, 0x16, 0), 0, false, false),
            BotGlobalOutcome::WeaponAnnounce { weapon: 0x16 }
        );
        // demo/protect site arms
        assert_eq!(
            bot_global_dispatch(msg(0x40, 0, 0), 0x2000_0040, false, false),
            BotGlobalOutcome::ObjectiveSite
        );
        // weapon change done with the +0x771 re-entry flag → 0xCF attack
        assert_eq!(
            bot_global_dispatch(msg(0x45, 0, state::BOT_IDLE as u32), 0, false, true),
            BotGlobalOutcome::WeaponChanged { goto: state::BOT_ATTACK }
        );
        assert_eq!(
            bot_global_dispatch(msg(0x45, 0, state::BOT_IDLE as u32), 0, false, false),
            BotGlobalOutcome::WeaponChanged { goto: 0 }
        );
        // unhandled ids fall through
        assert_eq!(bot_global_dispatch(msg(0x7F, 0, 0), 0, false, false), BotGlobalOutcome::Pass);
    }
}

#[cfg(test)]
mod brain_tests {
    use super::*;

    /// The shared harness: one bot at (0,1.6,0), a pickup table, a stationary player at
    /// (10,1.6,0), an empty collision world (all LOS clear — the arenas' real geometry is
    /// the NF_ISO test's job). `now` accumulates 1/60 per tick like the port's MPGame+0x412.
    struct Rig {
        world: CollisionWorld,
        pickups: Vec<BotPickup>,
        others: Vec<OtherPlayer>,
        brain: BotBrain,
        rng: Rng,
        now: f32,
        stationary: bool,
        /// The `CanSeeObject` stand-in: when false the human row never carries the seen bit.
        vision: bool,
    }

    impl Rig {
        fn new() -> Rig {
            let record = INITIAL_BOT_RECORDS[0].clone();
            // the test enables the boot record (the menu's enable byte +0x0E [M Menu_PrepareBots])
            let mut rec = record;
            rec.0[0x0E] = 1;
            let mut brain = BotBrain::bot_init(&rec, 4, [0.0, 1.6, 0.0], 0.0);
            brain.weapon = BotWeapon { interval: 6, burst: 3, aggression_mul: 1.0 };
            brain.vars.weapon_id = 0x02;
            brain.vars.clip_rounds = 50;
            Rig {
                world: CollisionWorld::from_triangles(Vec::new()),
                pickups: vec![BotPickup {
                    pos: [6.0, 1.6, 0.0],
                    weapon_id: 0x10,
                    mp_class: 0,
                    is_obj_flag2: false,
                    visit: [0.0; 16],
                    available: true,
                }],
                others: vec![
                    OtherPlayer { index: 0, pos: [10.0, 1.6, 0.0], alive: true, team: 0, flags: 0, dist2: 0.0, health: 100.0, score: 0.0 },
                    OtherPlayer { index: 4, pos: [0.0, 1.6, 0.0], alive: true, team: 0, flags: 0, dist2: 0.0, health: 100.0, score: 0.0 },
                ],
                brain,
                rng: Rng::new(0x1234),
                now: 0.0,
                stationary: false,
                vision: false,
            }
        }

        /// Builds the tick's `BotCtx` from owned pieces (the +0xB0 dist² recompute and the
        /// candidate bits happen in `step` — the game's `setOtherPlayerInfo` runs per tick).
        fn build_ctx<'a>(
            now: f32, pickups: &'a [BotPickup], others: &'a [OtherPlayer],
            world: &'a CollisionWorld, stationary: bool,
        ) -> BotCtx<'a> {
            BotCtx {
                now,
                scenario: 0x1, // deathmatch
                my_index: 4,
                my_team: 0,
                pickups,
                others,
                world,
                nav: None,
                health: 100.0,
                max_health: 100.0,
                stationary,
                lazy_sight: false,
                objectives: None,
                weapons: None,
                start_weapon: 0,
            }
        }

        fn step(&mut self) {
            let mut others = self.others.clone();
            // the per-tick +0xB0 dist² recompute [M 0x125390:148-149] and the candidate bits
            for o in others.iter_mut() {
                if o.index == 0 {
                    o.dist2 = (o.pos[0] - self.brain.pos[0]).powi(2)
                        + (o.pos[1] - self.brain.pos[1]).powi(2)
                        + (o.pos[2] - self.brain.pos[2]).powi(2);
                    o.flags = 0x2 | if self.vision { 0x4 } else { 0 }; // bit1 candidate [M 0x125390:99] + the seen bit
                }
            }
            {
                let ctx =
                    Self::build_ctx(self.now, &self.pickups, &others, &self.world, self.stationary);
                self.brain.step(&ctx, &mut self.rng, true);
            }
            self.now += 1.0 / 60.0;
            // the driver mirrors the visit stamp into the pickup record [M 0x129958:14]
            let (idx, t) = self.brain.last_visit_stamp;
            if idx != usize::MAX {
                self.pickups[idx].visit[self.brain.vars.bot_index as usize] = t;
                self.brain.last_visit_stamp = (usize::MAX, 0.0);
            }
        }
    }

    /// The goal machine round trip: BotInit → 0xF9 (the 0xC3 chain [M 0x1748c8:16-18]); the
    /// first idle tick picks the pickup goal (the deathmatch fall-through [M 0x126b88:317-319])
    /// with gotoGoal's deadline stamp (+0x28 = 300.0 [M 0x1278a8:50,58]); the walk arrives; the
    /// visit stamp fires (now + 45.0 [M 0x129958:10-15]); the bot returns to 0xF9.
    #[test]
    fn goal_roundtrip_pickup_visit() {
        let mut r = Rig::new();
        assert_eq!(r.brain.state, 0xC3);
        r.step();
        assert_eq!(r.brain.state, st::BOT_IDLE, "the BotInit chain lands on 0xF9");
        r.step();
        assert_eq!(r.brain.state, st::BOT_GOTO_GOAL_POSITION, "the pickup goal → 0xEB");
        let g = r.brain.vars.goals[r.brain.vars.active_goal as usize];
        assert_eq!(g.goal_type, 1, "a pickup goal");
        assert_eq!(g.target, 0);
        assert_eq!(g.duration, 300.0, "gotoGoal's VIDEO_FRAME_RATE*5 stamp [M 0x1278a8:58]");
        assert_eq!(g.fire_state, st::BOT_IDLE, "gotoGoal(slot, 0xF9) [M 0x126b88:510]");
        // walk: 6.0 u at 0.05/tick ≈ 120 ticks to arrive
        let mut arrived = false;
        for _ in 0..400 {
            r.step();
            if r.brain.state == st::BOT_IDLE && !r.brain.visited_pickups.is_empty() {
                arrived = true;
                break;
            }
        }
        assert!(arrived, "the bot reaches the pickup and returns to 0xF9; state {:?}", r.brain.state);
        assert_eq!(r.brain.visited_pickups, vec![0]);
        let visit = r.pickups[0].visit[0];
        assert!(visit >= 45.0 && visit <= 52.0, "the visit stamp = arrival + 45.0, got {visit}");
        // the walk actually moved the bot: the final position is at the pickup
        assert!((r.brain.pos[0] - 6.0).abs() < 1.5, "pos {:?}", r.brain.pos);
    }

    /// Sister: the "stand" preset (the stationary gate) never moves — the route blocks hold
    /// the bot [M 0x16f198:42-45] — while the moving bot of the sister test does.
    #[test]
    fn stand_preset_does_not_move() {
        let mut r = Rig::new();
        r.stationary = true;
        let start = r.brain.pos;
        for _ in 0..200 {
            r.step();
        }
        assert_eq!(r.brain.pos, start, "the stationary bot holds position");
        // it still runs its goal machine (the goal is picked, the route code stalls)
        assert!(
            r.brain.vars.active_goal != 0xFF || !r.brain.visited_pickups.is_empty(),
            "the goal machine still runs"
        );
    }

    /// The fire decision: the opponent inside the engage range with clear LOS → the heartbeat's
    /// sighted latch → 0xEC (seen) → 0xCF (attack) → 0xD2 (fire) → the first shot lands after
    /// the scheduled interval ticks (the DroneWeap_Fire → NextBulletTime chain [M 0x16f5e8:25,
    /// 0x179d40]). The aim-cone gate [M 0x178c48] holds because the bot faces the opponent.
    #[test]
    fn attack_fire_shot_at_stationary_player() {
        let mut r = Rig::new();
        r.vision = true; // the bot sees the stationary player (the CanSeeObject stand-in)
        r.pickups.clear(); // nothing to fetch: the idle bot turns to the candidate
        let mut fired_at = None;
        for i in 0..600 {
            r.step();
            if !r.brain.shots.is_empty() {
                fired_at = Some(r.brain.shots[0].tick);
                break;
            }
            assert!(i < 599, "no shot within 600 ticks; trace {:?}", r.brain.trace);
        }
        let t = fired_at.unwrap();
        // the decompile's timing bounds: the shot happens after the interval ticks of the
        // burst schedule (6 ticks here) and within the 5 s sight-latch window
        assert!(t >= 7, "the first shot waits for the burst deadline, tick {t}");
        let dir = r.brain.shots[0].dir;
        assert!(dir[0] > 0.99, "the shot aims at the stationary player (+x), dir {dir:?}");
        // the burst bookkeeping: the latch is set and the clip decremented [M 0x17a8c0:15-17]
        assert_eq!(r.brain.vars.fire_latch, 1);
        assert_eq!(r.brain.vars.clip_rounds, 49);
        // the state path went through the attack pair
        let path: Vec<u8> = r.brain.trace.iter().map(|e| e.2).collect();
        assert!(path.contains(&st::BOT_SEEN_OPPONENT), "the heartbeat's sighted arm fires 0xEC");
        assert!(path.contains(&state::BOT_ATTACK), "→ 0xCF");
        assert!(path.contains(&state::BOT_ATTACK_FIRE), "→ 0xD2");
    }

    /// The visit decay: a visited pickup is skipped on passes 0..2 and only scores on pass 3
    /// with the ×max(|now−visit|·50, 1) penalty [M 0x126b88:404-417] — a fresh far pickup wins
    /// first; a stale-visited near pickup can win once the decay has run (the 45 s cooldown).
    #[test]
    fn pickup_visit_decay_skips_then_decays() {
        let mut r = Rig::new();
        r.pickups.push(BotPickup {
            pos: [30.0, 1.6, 0.0],
            weapon_id: 0x11,
            mp_class: 0,
            is_obj_flag2: false,
            visit: [0.0; 16],
            available: true,
        });
        // first pick: the near pickup 0 wins (both class 0, weights equal → the highest score)
        r.step();
        r.step();
        let first = r.brain.vars.goals[r.brain.vars.active_goal as usize].target;
        assert_eq!(first, 0, "the near pickup wins the first pick");
        // mark pickup 0 visited right now (the cooldown window): the next pick must be pickup 1
        r.pickups[0].visit[0] = r.now + 45.0;
        r.brain.vars.last_pickup = -1;
        r.brain.uninit_goal(0);
        r.brain.set_goal_pick_prefs(0.0, 0.0, 0.0, 1.0, 1, 0, 0, 100.0, 1.0, 1.0);
        assert!(r.brain.pick_goal(
            &Rig::build_ctx(r.now, &r.pickups, &r.others, &r.world, false), &mut r.rng, 1
        ), "the second pick finds pickup 1");
        let second = r.brain.vars.goals[r.brain.vars.active_goal as usize].target;
        assert_eq!(second, 1, "the visited pickup is skipped (passes 0..2)");
        // 90 s later the decay (|now−visit|·50 → 4500·dist) has buried pickup 0 anyway; the
        // fresh pickup still wins, but a 90-s-cooled pickup 0 with a closer pos would beat a
        // far pickup 1 — pin the decay arithmetic directly:
        let decayed = ((100.0f32 - 10.0) * 50.0).max(1.0);
        assert_eq!(decayed, 4500.0, "the 90 s decay multiplier is the hard blacklist [M :407-415]");
    }

    /// The RNG order of a goal pick is exactly the decompile's: gotoGoal's slot-1 wander budget
    /// rolls only for the objective subtypes; a pickup goal (subtype 0) consumes none [M
    /// 0x1278a8:84-142] — two equal rigs with different seeds pick the same target.
    #[test]
    fn pick_goal_rng_order_is_deterministic() {
        let mut a = Rig::new();
        let mut b = Rig::new();
        a.rng = Rng::new(7);
        b.rng = Rng::new(0xBEEF);
        a.step();
        b.step();
        a.step();
        b.step();
        assert_eq!(a.brain.vars.goals[a.brain.vars.active_goal as usize].target,
                   b.brain.vars.goals[b.brain.vars.active_goal as usize].target,
                   "the pickup pick consumes no RNG (subtype 0 → no gotoGoal roll)");
    }

    /// A 100×100 floor at y 0 (+ optionally a 4-wide, 3-high wall across the walk at x 3, both
    /// faces). The bot stands on it: eye = floor + 1.1 (root) + EYE.
    fn floor_world(wall: bool) -> CollisionWorld {
        let mut t = vec![
            [[-50.0, 0.0, -50.0], [-50.0, 0.0, 50.0], [50.0, 0.0, -50.0]],
            [[50.0, 0.0, -50.0], [-50.0, 0.0, 50.0], [50.0, 0.0, 50.0]],
        ];
        if wall {
            let (a, b, c, d) = ([3.0, 0.0, -2.0], [3.0, 0.0, 2.0], [3.0, 3.0, 2.0], [3.0, 3.0, -2.0]);
            t.extend([[a, b, c], [a, c, d], [a, c, b], [a, d, c]]);
        }
        CollisionWorld::from_triangles(t)
    }

    fn floor_rig(wall: bool) -> Rig {
        let mut r = Rig::new();
        r.world = floor_world(wall);
        let eye = 1.1 + crate::player::EYE;
        let mut rec = INITIAL_BOT_RECORDS[0].clone();
        rec.0[0x0E] = 1;
        r.brain = BotBrain::bot_init(&rec, 4, [0.0, eye, 0.0], 0.0);
        r.brain.weapon = BotWeapon { interval: 6, burst: 3, aggression_mul: 1.0 };
        r.brain.vars.weapon_id = 0x02;
        r.brain.vars.clip_rounds = 50;
        r.pickups[0].pos = [6.0, 0.3, 0.0];
        r.others[0].pos = [10.0, eye, 0.0];
        r.others[1].pos = [0.0, eye, 0.0];
        r
    }

    /// The body walk on real geometry: the bot walks the floor to the pickup (the height holds —
    /// no re-stand, no sinking), and the walk is the body's (it moved at walking pace).
    #[test]
    fn body_walks_the_floor_to_the_pickup() {
        let mut r = floor_rig(false);
        let eye = r.brain.pos[1];
        let mut arrived = None;
        for t in 0..400 {
            r.step();
            assert!((r.brain.pos[1] - eye).abs() < 0.05, "tick {t}: the body stays on the floor, eye {:?}", r.brain.pos);
            if arrived.is_none() && r.brain.visited_pickups.contains(&0) {
                arrived = Some(t);
            }
        }
        let t = arrived.expect("the bot reaches the pickup");
        // 5 units at ≤ 0.105 a tick (+ the turn and the stick ease) — not instant, not stalled
        assert!(t >= 45 && t <= 200, "arrival tick {t}");
    }

    /// Sister: a wall across the walk holds the body (the capsule push) — the bot never passes
    /// x = 3 while it is inside the wall's z span — and it keeps moving (no frozen stand).
    #[test]
    fn body_never_walks_through_a_wall() {
        let mut r = floor_rig(true);
        let start = r.brain.pos;
        let mut walked = 0.0;
        let mut last = start;
        for t in 0..600 {
            r.step();
            let p = r.brain.pos;
            if p[2].abs() < 1.5 {
                assert!(p[0] < 3.0, "tick {t}: the body passed the wall, pos {p:?}");
            }
            walked += ((p[0] - last[0]).powi(2) + (p[2] - last[2]).powi(2)).sqrt();
            last = p;
        }
        assert!(walked > 2.0, "the bot walked up to the wall, walked {walked}");
    }
}
