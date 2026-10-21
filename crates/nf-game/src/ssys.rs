//! The security-subsystem alarm state machine — `SSys_Init` 0x1b01b0, `SSys_Monitor` 0x1ad998,
//! `SSys_Msg` 0x1ad878 and `SSys_RamLoad` 0x1b0208 (the four Phoenix Building security subsystems:
//! laser 1 / laser 2 / camera 1 / camera 2). Reached on every on-foot level [M callers]:
//! `ResetMap_GameInit` 0x1d5080 calls `SSys_Init`, `Game_Run` 0x1c9790 calls `SSys_Monitor` after
//! the `GS_IsPaused` gate (docs/atlas/object.md "Update order"), and `Player_RamLoad` 0x1957a8
//! calls `SSys_RamLoad`.
//!
//! State [M]: `SSysItems` 0x2befd0 is 5 records of 8 bytes `{ f32 timer; u8 channel; u8 state; }`
//! — item 0 unused, items 1..=4 are the subsystems. `SSysDelayTimer` 0x30cd94 is the laser-2
//! arming delay. Thresholds [L ee_snapshot @ 0x30cd84..0x30cd90]: `MIN3` = 180.0, `MIN2` = 120.0,
//! `MIN1` = 60.0, `MIN01` = 6.0.
//!
//! `SSys_Monitor` 0x1ad998 flow per item (only while `ScriptCam == 0`, and only when not paused —
//! the Game_Run gate):
//! - `channel == 0xff` → skip (the item is disarmed).
//! - the item's channel is off: if `state != 0` → `state = 0`, `Msg(i, 0)`, `timer = -1.0`.
//! - the item's channel is on: if `timer <= -1.0` the item arms — item 2 (laser 2) first waits out
//!   the shared 8 s `SSysDelayTimer` and on expiry plays `Msg(1, 0)`, forces item 1's channel on
//!   and disarms it (`channel = 0xff`), then re-arms through the shared path next tick; every other
//!   item (and item 2 once item 1's channel is 0xff) takes the shared arm: `timer = 240.0`
//!   (0x43700000), `state = 6`, `Msg(i, 6)`. Otherwise `timer -= dt` (REC_FRAME_RATE; 1.0 at the
//!   fixed 60 Hz).
//! - the countdown switch: state 6 → 5 at `timer < 180` (`Msg(i,5)`), 5 → 4 at 120, 4 → 3 at 60,
//!   3 → 2 at 6, and state 2 at `timer <= 0` → `Msg(i,1)`, the item's channel switched **off**,
//!   `state = 0`, `timer = -1.0` (the subsystem fires and shuts itself down).
//!
//! `SSys_Msg` 0x1ad878 plays `table[msg_id].sound` (the u32 at `table + id·8 + 4`, 100.0f volume)
//! from one of the four sound tables, with level gates [M]: with `GameState == 0x0700000b` and
//! `switch_channels[0x76] == 0` only laser 2 (type 2) sounds play; on `0x0700000a` camera 1
//! (type 3) never plays. Tables [L ee_start_05 @ 0x2bef60 / 0x2bef98 / 0x2beef0 / 0x2bef28
//! (`Laser1.134` / `Laser2.135` / `Cam1.132` / `Cam2.133`)]: the sound column of ids 0..=6.
//!
//! `SSys_Init` 0x1b01b0 sets items 1..=5 to `{-1.0, 0xff, 0}` unless `IsWarmReset`.
//! `SSys_RamLoad` 0x1b0208 copies the SSYS save block over items 1..=4 (skipping item 2 on
//! 0x0700000a) and, on 0x070000b, forces item 1 back to `{-1.0, 0xff, 0}`; `SSysDelayTimer = 0`.

/// One `SSysItems` record [M 0x1ad998: `SSysItems + i·8`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SsysItem {
    /// The countdown timer (f32; −1.0 = armed-and-waiting, 240.0 on arm).
    pub timer: f32,
    /// The switch channel that keeps the subsystem live (0xff = disarmed).
    pub channel: u8,
    /// The alarm state (0 disarmed; 6 armed → 5 → 4 → 3 → 2 → 1 fired).
    pub state: u8,
}

/// The countdown thresholds [L ee_snapshot @ 0x30cd84..0x30cd94 — `MIN3`/`MIN2`/`MIN1`/`MIN01`].
pub const MIN3: f32 = 180.0;
pub const MIN2: f32 = 120.0;
pub const MIN1: f32 = 60.0;
pub const MIN01: f32 = 6.0;
/// The arm time written at (re)arm [M 0x1ad998: `0x43700000` = 240.0f].
pub const ARM_TIME: f32 = 240.0;
/// The laser-2 arming delay [M 0x1ad998: `SSysDelayTimer = 8.0`].
pub const LASER2_DELAY: f32 = 8.0;

/// The sound column of the four tables, ids 0..=6 [L ee_start_05; `SSys_Msg` reads +4 of each
/// 8-byte row]. Laser 1 `Laser1.134` 0x2bef60, laser 2 `Laser2.135` 0x2bef98, camera 1
/// `Cam1.132` 0x2beef0, camera 2 `Cam2.133` 0x2bef28.
pub const MSG_SOUNDS: [[u32; 7]; 4] = [
    [0x2a4, 0x2a3, 0x298, 0x297, 0x296, 0x295, 0x2a2], // laser 1
    [0x431, 0x43d, 0x298, 0x297, 0x296, 0x295, 0x438], // laser 2
    [0x2a1, 0x2a0, 0x298, 0x297, 0x296, 0x295, 0x29b], // camera 1
    [0x2a1, 0x2a0, 0x298, 0x297, 0x296, 0x295, 0x29b], // camera 2
];

/// The whole security system.
#[derive(Clone, Debug, PartialEq)]
pub struct Ssys {
    /// `SSysItems` — index 0 is the game's unused header record.
    pub items: [SsysItem; 5],
    /// `SSysDelayTimer`.
    pub delay_timer: f32,
}

impl Default for Ssys {
    fn default() -> Self {
        Self::new()
    }
}

impl Ssys {
    /// `SSys_Init` 0x1b01b0: items 1..=5 = `{-1.0, 0xff, 0}` unless warm reset.
    pub fn new() -> Ssys {
        let mut items = [SsysItem { timer: 0.0, channel: 0, state: 0 }; 5];
        for it in items.iter_mut().skip(1) {
            *it = SsysItem { timer: -1.0, channel: 0xff, state: 0 };
        }
        Ssys { items, delay_timer: 0.0 }
    }

    /// `SSys_RamLoad` 0x1b0208. `block[i-1]` is the saved record i (the game walks `param_1`
    /// quads over items 1..=4). `level` is `GameState+0xc`.
    pub fn ram_load(&mut self, block: &[SsysItem; 4], level: u32) {
        for (k, it) in block.iter().enumerate() {
            let i = k + 1;
            if level == 0x0700_000a && i == 2 {
                continue; // the game skips item 2 on 0x0700000a [M 0x1b0208:13]
            }
            self.items[i] = *it;
        }
        if level == 0x0700_000b {
            self.items[1] = SsysItem { timer: -1.0, channel: 0xff, state: 0 };
        }
        self.delay_timer = 0.0;
    }

    /// `SSys_Msg` 0x1ad878: the sound id for `(item, msg)` under the level gates, or `None` when
    /// the game stays silent. `level` = `GameState+0xc`; `ch_0x76_off` = `switch_channels[0x76] == 0`.
    pub fn msg_sound(&self, item: usize, msg: u8, level: u32, ch_0x76_off: bool) -> Option<u32> {
        let phoenix_b = level == 0x0700_000b;
        let gate_b = phoenix_b && ch_0x76_off; // the decomp's `bVar1`
        match item {
            2 => {
                if phoenix_b && !ch_0x76_off {
                    return None; // laser 2 is silent on 0x070000b while channel 0x76 is set
                }
            }
            1 | 3 | 4 => {
                if gate_b {
                    return None;
                }
                if item == 3 && level == 0x0700_000a {
                    return None; // camera 1 never plays on 0x0700000a [M 0x1ad878:26]
                }
            }
            _ => return None,
        }
        if msg > 6 {
            return None;
        }
        Some(MSG_SOUNDS[item - 1][msg as usize])
    }

    /// `SSys_Monitor` 0x1ad998 for one tick. `channel_on(c)` reads `switch_channels[c]`;
    /// `set_channel(c, on)` writes it (the state-2 expiry and the laser-2 delay path both flip
    /// channels). Returns the `(item, msg_id)` pairs to play this tick, in the game's play order.
    /// The caller owns the `ScriptCam == 0` and not-paused gates [M 0x1ad998:10, Game_Run].
    pub fn monitor(
        &mut self,
        mut channel_on: impl FnMut(u8) -> bool,
        mut set_channel: impl FnMut(u8, bool),
    ) -> Vec<(usize, u8)> {
        let mut out = Vec::new();
        for i in 1..5usize {
            let ch = self.items[i].channel;
            if ch == 0xff {
                continue;
            }
            if !channel_on(ch) {
                if self.items[i].state != 0 {
                    self.items[i].state = 0;
                    self.items[i].timer = -1.0;
                    out.push((i, 0));
                }
                continue;
            }
            if self.items[i].timer <= -1.0 {
                if i == 2 {
                    // the laser-2 arming delay [M 0x1ad998:25-38]
                    if self.delay_timer > 0.0 {
                        self.delay_timer -= 1.0;
                        continue;
                    }
                    if self.items[1].channel != 0xff {
                        out.push((1, 0));
                        self.items[1].timer = -1.0;
                        set_channel(self.items[1].channel, true);
                        self.items[1].channel = 0xff;
                        self.delay_timer = LASER2_DELAY;
                        self.items[1].state = 0;
                        continue;
                    }
                }
                self.items[i].timer = ARM_TIME;
                self.items[i].state = 6;
                out.push((i, 6));
                continue;
            }
            self.items[i].timer -= 1.0;
            match self.items[i].state {
                6 => {
                    if self.items[i].timer < MIN3 {
                        self.items[i].state = 5;
                        out.push((i, 5));
                    }
                }
                5 => {
                    if self.items[i].timer < MIN2 {
                        self.items[i].state = 4;
                        out.push((i, 4));
                    }
                }
                4 => {
                    if self.items[i].timer < MIN1 {
                        self.items[i].state = 3;
                        out.push((i, 3));
                    }
                }
                3 => {
                    if self.items[i].timer < MIN01 {
                        self.items[i].state = 2;
                        out.push((i, 2));
                    }
                }
                2 => {
                    if self.items[i].timer <= 0.0 {
                        out.push((i, 1));
                        if self.items[i].channel != 0 {
                            set_channel(self.items[i].channel, false);
                        }
                        self.items[i].state = 0;
                        self.items[i].timer = -1.0;
                    }
                }
                _ => {}
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_and_arm_time_are_the_disc_values() {
        // [L ee_snapshot @ 0x30cd84..0x30cd94] and [M 0x1ad998 0x43700000].
        assert_eq!(MIN3, 180.0);
        assert_eq!(MIN2, 120.0);
        assert_eq!(MIN1, 60.0);
        assert_eq!(MIN01, 6.0);
        assert_eq!(ARM_TIME, 240.0);
        assert_eq!(ARM_TIME.to_bits(), 0x4370_0000);
        // Sister (can fail): the arm time is not the 8 s delay.
        assert_ne!(ARM_TIME, LASER2_DELAY);
    }

    #[test]
    fn msg_sound_tables_match_the_ee_snapshot() {
        // [L ee_start_05 @ 0x2bef60/0x2bef98/0x2beef0/0x2bef28 — the +4 column of each 8-byte row].
        assert_eq!(MSG_SOUNDS[0], [0x2a4, 0x2a3, 0x298, 0x297, 0x296, 0x295, 0x2a2]);
        assert_eq!(MSG_SOUNDS[1], [0x431, 0x43d, 0x298, 0x297, 0x296, 0x295, 0x438]);
        assert_eq!(MSG_SOUNDS[2][0], 0x2a1);
        assert_eq!(MSG_SOUNDS[3][6], 0x29b);
        // Sister (can fail): the tables differ between laser 1 and laser 2 at ids 0/1/6.
        assert_ne!(MSG_SOUNDS[0][0], MSG_SOUNDS[1][0]);
    }

    #[test]
    fn msg_gates_follow_the_decompile() {
        let s = Ssys::new();
        // [M 0x1ad878] on 0x0700000b with channel 0x76 off only laser 2 sounds.
        assert_eq!(s.msg_sound(1, 1, 0x0700_000b, true), None);
        assert_eq!(s.msg_sound(2, 1, 0x0700_000b, true), Some(0x43d));
        // On 0x070000b with 0x76 SET, laser 2 is gated and the rest play.
        assert_eq!(s.msg_sound(2, 1, 0x0700_000b, false), None);
        assert_eq!(s.msg_sound(1, 1, 0x0700_000b, false), Some(0x2a3));
        // [M 0x1ad878:26] camera 1 never plays on 0x0700000a.
        assert_eq!(s.msg_sound(3, 1, 0x0700_000a, false), None);
        assert_eq!(s.msg_sound(4, 1, 0x0700_000a, false), Some(0x2a0));
        // Sister (can fail): camera 2 is NOT gated on 0x0700000b+0x76-off… it is (types 1/3/4 share
        // bVar1) — assert the gate catches it and laser 2 alone survives.
        assert_eq!(s.msg_sound(4, 1, 0x0700_000b, true), None);
    }

    #[test]
    fn monitor_runs_the_full_countdown_and_shuts_the_channel_off() {
        let mut s = Ssys::new();
        s.items[1] = SsysItem { timer: -1.0, channel: 0x52, state: 0 };
        let chans: std::cell::RefCell<std::collections::BTreeMap<u8, bool>> =
            std::cell::RefCell::new(std::collections::BTreeMap::from([(0x52u8, true)]));
        let on = |c: u8| chans.borrow().get(&c).copied().unwrap_or(false);
        let mut set = |c: u8, v: bool| { chans.borrow_mut().insert(c, v); };
        // Arm tick: 240.0 + state 6 + msg 6.
        let msgs = s.monitor(on, &mut set);
        assert_eq!(msgs, vec![(1, 6)]);
        assert_eq!(s.items[1].timer, ARM_TIME);
        assert_eq!(s.items[1].state, 6);
        // Drive the timer to each threshold.
        let mut seen: Vec<(usize, u8)> = Vec::new();
        for _ in 0..300 {
            seen.extend(s.monitor(on, &mut set));
        }
        let got: Vec<u8> = seen.iter().filter(|(i, _)| *i == 1).map(|(_, m)| *m).collect();
        assert_eq!(got, vec![5, 4, 3, 2, 1], "the countdown order 6→5→4→3→2→1");
        // State 2's expiry switched the channel off and disarmed.
        assert_eq!(chans.borrow().get(&0x52).copied(), Some(false));
        assert_eq!(s.items[1].state, 0);
        assert_eq!(s.items[1].timer, -1.0);
        // Sister (can fail): with the channel off, a disarmed item does NOT re-arm.
        let msgs = s.monitor(on, &mut set);
        assert!(msgs.is_empty());
        // A re-set channel re-arms through msg 6 again.
        chans.borrow_mut().insert(0x52, true);
        let msgs = s.monitor(on, &mut set);
        assert_eq!(msgs, vec![(1, 6)]);
    }

    #[test]
    fn laser2_arms_through_the_8_second_delay_while_laser1_is_live() {
        let mut s = Ssys::new();
        s.items[1] = SsysItem { timer: -1.0, channel: 0x53, state: 0 };
        s.items[2] = SsysItem { timer: -1.0, channel: 0x54, state: 0 };
        s.delay_timer = LASER2_DELAY;
        let chans: std::cell::RefCell<std::collections::BTreeMap<u8, bool>> =
            std::cell::RefCell::new(std::collections::BTreeMap::from([(0x53u8, true), (0x54u8, true)]));
        let on = |c: u8| chans.borrow().get(&c).copied().unwrap_or(false);
        let forced: std::cell::RefCell<Vec<(u8, bool)>> = std::cell::RefCell::new(Vec::new());
        let mut set = |c: u8, v: bool| { chans.borrow_mut().insert(c, v); forced.borrow_mut().push((c, v)); };
        // Laser 1 arms at once; laser 2 waits behind the delay.
        assert_eq!(s.monitor(on, &mut set), vec![(1, 6)]);
        let mut msgs;
        let mut waited = 0;
        let mut laser1_msg_at = None;
        loop {
            msgs = s.monitor(on, &mut set);
            if msgs.contains(&(1usize, 0u8)) {
                laser1_msg_at = Some(waited);
            }
            if msgs.iter().any(|&(i, _)| i == 2) {
                break;
            }
            waited += 1;
            assert!(waited < 20, "laser 2 never armed");
        }
        // The game's laser-2 arming [M 0x1ad998:25-38]: 8 delay ticks → the (1, 0) laser-1
        // message + the delay rewritten to 8.0 → 8 more delay ticks → the shared arm. The count
        // here is 16 because the pre-loop arm call consumed one delay tick already.
        assert_eq!(waited, 16, "7 + 1 + 8 ticks of delay after the pre-loop arm tick");
        assert_eq!(laser1_msg_at, Some(7), "the (1, 0) laser-1 message fired at the first delay expiry");
        assert_eq!(msgs, vec![(2usize, 6u8)]);
        assert_eq!(forced.borrow().clone(), vec![(0x53, true)]);
        assert_eq!(s.items[1].channel, 0xff);
        // The 8.0 rewrite happens only on the laser-1 branch; at the arm the delay is spent.
        assert_eq!(s.delay_timer, 0.0);
        // The break tick WAS the arm: item 2 carries the shared 240 s arm.
        assert_eq!(s.items[2].timer, ARM_TIME);
        assert_eq!(s.items[2].state, 6);
        // Sister (can fail): item 1 stayed disarmed (0xff) — the game does not re-live it here.
        assert_ne!(s.items[1].channel, 0x53);
    }

    #[test]
    fn ram_load_skips_item_2_on_0a_and_resets_item_1_on_0b() {
        let block = [
            SsysItem { timer: 10.0, channel: 0x51, state: 3 },
            SsysItem { timer: 20.0, channel: 0x52, state: 6 },
            SsysItem { timer: 30.0, channel: 0x53, state: 5 },
            SsysItem { timer: 40.0, channel: 0x54, state: 4 },
        ];
        // [M 0x1b0208:13] item 2 is not loaded on 0x0700000a.
        let mut s = Ssys::new();
        s.ram_load(&block, 0x0700_000a);
        assert_eq!(s.items[2], SsysItem { timer: -1.0, channel: 0xff, state: 0 });
        assert_eq!(s.items[1].timer, 10.0);
        assert_eq!(s.items[3].timer, 30.0);
        assert_eq!(s.delay_timer, 0.0);
        // [M 0x1b0208:19] on 0x070000b item 1 is forced back to {-1, 0xff, 0}.
        let mut s = Ssys::new();
        s.ram_load(&block, 0x0700_000b);
        assert_eq!(s.items[1], SsysItem { timer: -1.0, channel: 0xff, state: 0 });
        assert_eq!(s.items[2].timer, 20.0);
        // Sister (can fail): on any other level everything loads.
        let mut s = Ssys::new();
        s.ram_load(&block, 0x0700_000c);
        assert_eq!(s.items[1].timer, 10.0);
        assert_eq!(s.items[2].timer, 20.0);
        assert_eq!(s.items[4].timer, 40.0);
    }

    #[test]
    fn init_matches_the_game_layout() {
        // [M 0x1b01b0] items 1..=5 = {-1.0, 0xff, 0}; [L ee_start_05 @ 0x2befd0 — items 1..4 are
        // (-1.0, 255), item 0's header bytes stay 0].
        let s = Ssys::new();
        assert_eq!(s.items[0], SsysItem { timer: 0.0, channel: 0, state: 0 });
        for it in &s.items[1..] {
            assert_eq!(*it, SsysItem { timer: -1.0, channel: 0xff, state: 0 });
        }
        // Sister (can fail): the armed timer bit pattern is not the init pattern.
        assert_ne!(SsysItem { timer: 240.0, channel: 0xff, state: 0 }, s.items[1]);
    }
}
