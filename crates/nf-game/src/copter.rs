//! The helicopter (`Copter_Create` 0x17d340, `Copter_Update` 0x17dbb8, `Copter_Move` 0x17d628;
//! placement type 0x2F — fix-objects2). 07000008's `Helicopter-Body` (#45) is the mission's last
//! writer: gated on ch 5 (which the ride's end cinematic 0x06000078 writes), it flies its script
//! and its arrival writes ch 0x4B — "Escape with Zoe in gondola" completes and mux #276 fires
//! ch 100, ending the level into 0x0730008 [L mission record 3; M Mission_Update 0x182c60:85-116].
//!
//! The game's model [M]:
//! - `Copter_Create` (0x17d340 disasm 0x17d49c-0x17d4d0): record+0x70 = param 6, +0x72 = param 7,
//!   +0x74 = param 8, +0x76 = param 9; +0x48/+0x54 hold its scripts (params 4/5 — 0x06000051 the
//!   parts assembly, 0x06000050 the flight [D #45; the level bin's type-7 entries]), loaded through
//!   a hidden `SP_Create` (0x1dc7e0) whose script entities register as copter paths
//!   (`Copter_Register` 0x17e530).
//! - `Copter_Update` (0x17dbb8 head): a non-zero gate channel that is off returns at once — the
//!   copter does nothing until the gate opens; then the +0x76 countdown pulses +0x74 (:16-22).
//! - The move states: `Copter_Move` (0x17d628) advances the path playhead (record+0x5C) and, when
//!   the frames run out, the state machine advances; cases 4/5/6 write
//!   `switch_channels[record+0x72] = 1` (:249-268) — the arrival channel.
//! - The combat states (case 2: `Music_Event(2,1)`, `Copter_FireMissile`/`FireGun`, the cone-sphere
//!   player detection) are the attack helicopter's — not ported; on 07000008 the copter is the
//!   extraction and its gate only opens at the ride's end. `Copter_FireGun` 0x17d828 (452 B, sole
//!   caller `Copter_Update` 0x17dbb8:188-198 case 2's odd-attackmode arm) is ported below as
//!   `fire_gun_tick` — the cadence, the muzzle and the weapon id are exact; the gun part positions
//!   arrive per call because the game's part slots (`rec+0x30..0x54`, `Copter_Register` 0x17e530)
//!   are SP script entities the port does not capture as ids.
//!
//! The game's list plumbing, disposed:
//! - `Copter_Init` 0x17e4f0 (60 B, `ResetMap_GameInit` 0x1d5080:33): `LList_Init(0x2a3830, 0x7c,0)`
//!   + `LList_Init(0x316fe0, 0x1c,0)` — the copter + path list resets. The port's per-map
//!   `Copter::create` rebuild (`self.copters = Copter::create(...)` in the scene build) is the
//!   same observable: fresh empty lists each load.
//! - `Copter_Delete` 0x17e568 (72 B, the class-0x3B delete dispatch slot at 0x26fb5c [D], invoked
//!   by the delete pass after the unported death path): `SP_Delete(*(ext+0x2c))` +
//!   `LList_Remove(0x2a3830, ext)` + 0x7c zeroing, unconditional [M 0x17e568:8-12]. Ported below
//!   as `delete` (removal from the port's copter vec); it becomes live when the hp/destruction
//!   walk (`Copter_Update` 0x17dbb8:277-371) lands.
//! - `Copter_GetBody` 0x17e4b8 (56 B; callers `Bullet_heatseekable` 0x12f248:33-38, `GT_Track`
//!   0x17f9c8:173-183, `HUD_UpdateRedeemerPane` 0x19e178:144-171): the body accessor with the
//!   hide-gate folded in — record+0x70 gate set and its channel off → the SP_HideObj'd body reads
//!   as absent; else `*(rec+0x38)` (part slot 2). Ported below as `body`.
//!
//! Port status: the channel machine is exact (gate, the countdown pulse, the arrival write); the
//! flight is timed by the flight script's Nis length (0x06000050, 299 ticks [D]) — `Copter_Move`'s
//! frame counter reaches the same end [S: the exact per-path advance is not modelled]. The visuals
//! ride the port's script-player machinery: a synthetic player (mode 1, plays once) animates the
//! flight script's entities (the body 0x02000427 type 0x2F and its parented parts [D script
//! streams]) from the placement's matrix, and they join the collision world through the normal
//! script-entity path.

use crate::triggers::Channels;
use crate::world::V3;
use nf_formats::placement::{object_matrix, Placement};
use nf_formats::anim::Mat4;

#[derive(Clone, Debug)]
pub struct Copter {
    pub placement: usize,
    /// The placement's world matrix (the hidden SP's frame — the script's keys are local to it).
    pub matrix: Mat4,
    /// record+0x70 = param 6: the gate channel [M 0x17d340:0x17d4ac, 0x17dbb8 head].
    pub gate: u16,
    /// record+0x72 = param 7: written on the move's completion [M 0x17dbb8:249-268].
    pub out: u16,
    /// record+0x74/+0x76 = params 8/9: the countdown pulse written when the countdown expires
    /// [M 0x17dbb8:16-22]. Both 0 on 07000008 [D #45] — inert there, implemented anyway.
    pub arrival: u16,
    pub countdown: u16,
    /// The scripts (params 4/5): [assembly, flight] [D #45 = 0x06000051 / 0x06000050].
    pub scripts: [u32; 2],
    /// Idle until the gate's rising edge (the game's early return).
    pub started: bool,
    /// Ticks since the flight began; `out` fires at the flight script's length.
    pub flight_t: u32,
    pub flight_len: u32,
    pub done: bool,
    /// `obj+0x68` = 299.0 x the difficulty multiplier [M `Copter_Create` 0x17d340:76-94]:
    /// d1 x1.0 (the decomp's fall-through default), d2 x1.5, d3 x2.0, d4 x4.0.
    pub hp: f32,
    /// `*(rec+0x38)` — part slot 2, the body entity id (`Copter_Register` 0x17e530:7-8 fills the
    /// slots from the SP script entities; the port captures the body when the assembly script
    /// spawns it). None until captured.
    pub body_obj: Option<u32>,
    /// The gun parts' fire counters — the game keeps `*(short*)(gun+0xf4)` per gun obj
    /// (slots 5/6) [M `Copter_FireGun` 0x17d828:63-65, 93-96]. [0] = gun A (rec+0x44),
    /// [1] = gun B (rec+0x48).
    pub gun_counters: [u16; 2],
}

/// The per-difficulty health multiplier (`Copter_Create` 0x17d340:76-93): 2 -> 1.5, 3 -> 2.0,
/// 4 -> 4.0, everything else (d1) -> 1.0 [M].
pub fn copter_hp_mult(difficulty: u32) -> f32 {
    match difficulty {
        2 => 1.5,
        3 => 2.0,
        4 => 4.0,
        _ => 1.0,
    }
}

impl Copter {
    /// `Copter_Create` over the main map's 0x2F placements. `difficulty` is the selected
    /// difficulty (`GameState+0x28`, 0x2a3790) — the HP scales with it [M 0x17d340:76-94].
    pub fn create(placements: &[Placement], difficulty: u32) -> Vec<Copter> {
        let mut out = Vec::new();
        for (i, p) in placements.iter().enumerate() {
            if p.flags & 0xff != 0x2f { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            out.push(Copter {
                placement: i,
                matrix: object_matrix(p.rotation, p.translation),
                gate: param(6) as u16,
                out: param(7) as u16,
                arrival: param(8) as u16,
                countdown: param(9) as u16,
                scripts: [param(4), param(5)],
                started: false,
                flight_t: 0,
                flight_len: 0,
                done: false,
                hp: copter_hp_mult(difficulty) * 299.0,
                body_obj: None,
                gun_counters: [0; 2],
            });
        }
        out
    }

    /// `Copter_Update`'s channel machine. `flight_len` = the flight script's Nis length (the game's
    /// path frame count). Returns `(started_flight, arrived)`:
    /// - `started_flight` — the gate just rose; the caller starts the flight script's player;
    /// - `arrived` — the move completed this tick; `out` has been written [M cases 4/5/6].
    pub fn tick(&mut self, ch: &mut Channels, flight_len: u32, frame: u32) -> (bool, bool) {
        // The gate: a non-zero gate channel that is off stops everything [M 0x17dbb8 head].
        if self.gate != 0 && !ch.on(self.gate as u32) { return (false, false); }
        // The countdown pulse [M 0x17dbb8:16-22].
        let mut arrived = false;
        if self.countdown != 0 {
            self.countdown -= 1;
            if self.countdown == 0 && self.arrival != 0 && !self.done { ch.set(self.arrival as u32, 1, frame); }
        }
        if self.done { return (false, false); }
        if !self.started {
            self.started = true;
            self.flight_len = flight_len;
            self.flight_t = 0;
            if self.flight_len == 0 {
                // No flight script: the move is already complete [M Copter_Move's +0x54 == 0 → 1].
                self.done = true;
                if self.out != 0 { ch.set(self.out as u32, 1, frame); }
                return (false, true);
            }
            return (true, false);
        }
        // The flight: the playhead advances per tick; at the end the state machine writes `out`.
        if self.flight_t < self.flight_len { self.flight_t += 1; }
        if self.flight_t >= self.flight_len {
            self.done = true;
            if self.out != 0 { ch.set(self.out as u32, 1, frame); }
            arrived = true;
        }
        (false, arrived)
    }

    /// The copter's world position (the matrix row 3) — the viewer's draw/cull helpers.
    pub fn pos(&self) -> V3 { [self.matrix[3][0], self.matrix[3][1], self.matrix[3][2]] }

    /// `Copter_GetBody` 0x17e4b8:6-9 — the body entity, with the hide-gate folded in: a set gate
    /// whose channel is off means the body is `SP_HideObj`'d (Create 0x17d340:73-74 hides it,
    /// Update 0x17dbb8:95 unhides) and the accessor reads as absent. `gate_channel_on` is the
    /// caller's `switch_channels[gate] != 0` read.
    pub fn body(&self, gate_channel_on: bool) -> Option<u32> {
        if self.gate != 0 && !gate_channel_on {
            return None; // [M 0x17e4b8:6-8]
        }
        self.body_obj // [M 0x17e4b8:9 — *(rec+0x38)]
    }

    /// `Copter_FireGun` 0x17d828 — one attack tick. Only gun A (`rec+0x44`, slot 5) counts and
    /// fires [M :58-65]; gun B (`rec+0x48`) is only re-primed to 2 after a shot (with its +0xf0
    /// bit 0x10 cleared) when present [M :94-96]. `gun` is gun A's
    /// world position (the game reads it from the gun obj's +0xc0), `radius` the gun's cel-glist
    /// sphere radius (`*(glist)+0x38` [M :66-84]), `player` Bond's +0x30 with y+0.5 [M :55-59],
    /// `level_is_07000008` selects the weapon id (0x61 on 0700008, else 0x71 [M :86-89]).
    /// Returns Some((muzzle, dir, weapon_id)) on the tick the gun fires, else None.
    ///
    /// [M] the cadence: gun A's counter pre-increments and fires when the OLD value > 4 — every
    /// 6th call (:63-65). [M] after a fire gun A's counter resets to 0 and gun B's (when
    /// `gun_b_present`) is set to 2 (:93-96). No RNG here (the gun/missile pick is the caller's
    /// `Rand_Rand(10)` at 0x17dbb8:184).
    pub fn fire_gun_tick(
        &mut self,
        gun_b_present: bool,
        gun: V3,
        radius: f32,
        player: V3,
        level_is_07000008: bool,
    ) -> Option<(V3, V3, u32)> {
        let old = self.gun_counters[0] as i16; // signed short [M :63 `short sVar1`]
        self.gun_counters[0] = (old as u16).wrapping_add(1); // [M :63-65 pre-increment]
        if old <= 4 {
            return None; // [M :65 — fires only when the old value > 4]
        }
        let target = [player[0], player[1] + 0.5, player[2]]; // [M :55-59]
        let mut dir = [target[0] - gun[0], target[1] - gun[1], target[2] - gun[2]];
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if len > 0.0 {
            dir = [dir[0] / len, dir[1] / len, dir[2] / len]; // [M :66-72 normalise]
        }
        let muzzle = [gun[0] + dir[0] * radius, gun[1] + dir[1] * radius, gun[2] + dir[2] * radius];
        // [M :66-84 muzzle = gun pos + dir * cel-glist radius]
        let weapon = if level_is_07000008 { 0x61 } else { 0x71 }; // [M :86-89]
        self.gun_counters[0] = 0; // [M :93]
        if gun_b_present {
            self.gun_counters[1] = 2; // [M :94-95]
        }
        Some((muzzle, dir, weapon))
    }

    /// `Copter_Delete` 0x17e568:8-12 — remove the copter from the port's list (the game's
    /// `LList_Remove(0x2a3830, ext)` + the 0x7c zeroing; the SP part delete is the script
    /// machinery's). Returns true when a copter actually left. Live once the death path
    /// (Copter_Update 0x17dbb8:277-371) pokes the delete dispatch.
    pub fn delete(copters: &mut Vec<Copter>, index: usize) -> bool {
        if index < copters.len() {
            copters.remove(index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copter() -> Copter {
        Copter {
            placement: 0,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            gate: 0,
            out: 0,
            arrival: 0,
            countdown: 0,
            scripts: [0; 2],
            started: false,
            flight_t: 0,
            flight_len: 0,
            done: false,
            hp: 299.0,
            body_obj: Some(0x1234),
            gun_counters: [0; 2],
        }
    }

    // [M 0x17e4b8:6-9] the hide-gate: gate set + channel off → absent; channel on or gate 0 → body.
    #[test]
    fn body_respects_the_hide_gate() {
        let mut c = copter();
        c.gate = 5;
        assert_eq!(c.body(false), None, "gated and off → SP_HideObj'd, absent");
        assert_eq!(c.body(true), Some(0x1234), "gated and on → the slot-2 body");
        c.gate = 0;
        assert_eq!(c.body(false), Some(0x1234), "no gate → always visible");
    }

    // [M 0x17d828:63-65] the 6-tick cadence: fires when the OLD counter value > 4.
    #[test]
    fn fire_gun_fires_every_sixth_call() {
        let mut c = copter();
        let player = [100.0, 10.0, 0.0];
        let gun = [0.0, 0.0, 0.0];
        for i in 0..5 {
            assert!(c.fire_gun_tick(true, gun, 2.0, player, false).is_none(), "call {i} must not fire");
        }
        let shot = c.fire_gun_tick(true, gun, 2.0, player, false);
        assert!(shot.is_some(), "call 6 fires [M :65]");
    }

    // [M 0x17d828:86-89] the weapon id: 0x71 normally, 0x61 on 0700008.
    #[test]
    fn fire_gun_weapon_id_per_level() {
        let mut c = copter();
        for _ in 0..5 { c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false); }
        assert_eq!(c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false).unwrap().2, 0x71);
        let mut c2 = copter();
        for _ in 0..5 { c2.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], true); }
        assert_eq!(c2.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], true).unwrap().2, 0x61);
    }

    // [M 0x17d828:93-96] after a fire: firing counter resets to 0, the sister's becomes 2.
    #[test]
    fn fire_gun_swaps_the_counters() {
        let mut c = copter();
        for _ in 0..5 { c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false); }
        c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false);
        assert_eq!(c.gun_counters[0], 0);
        assert_eq!(c.gun_counters[1], 2);
        // gun B never fires itself: only gun A's counter moves on later calls [M :58-65].
        c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false);
        assert_eq!(c.gun_counters, [1, 2]);
        // sister: no gun B → its counter is left alone [M :94 the null test]
        let mut d = copter();
        for _ in 0..6 { d.fire_gun_tick(false, [0.0; 3], 1.0, [0.0; 3], false); }
        assert_eq!(d.gun_counters, [0, 0]);
    }

    // [M 0x17d828:66-84] the muzzle sits radius units along the aim direction from the gun.
    #[test]
    fn muzzle_is_radius_along_the_aim() {
        let mut c = copter();
        for _ in 0..5 { c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false); }
        let (muzzle, dir, _) = c.fire_gun_tick(true, [0.0, 0.0, 0.0], 3.0, [4.0, 3.0, 0.0], false).unwrap();
        // dir = normalise(4, 3.5, 0); muzzle = dir * 3.
        let len = (4.0f32 * 4.0 + 3.5 * 3.5).sqrt();
        let want = [4.0 / len * 3.0, 3.5 / len * 3.0, 0.0];
        assert!((dir[0] - 4.0 / len).abs() < 1e-5 && (dir[1] - 3.5 / len).abs() < 1e-5);
        assert!((muzzle[0] - want[0]).abs() < 1e-5 && (muzzle[1] - want[1]).abs() < 1e-5);
        // [M :55-59] the target's y carries the +0.5.
        assert!(dir[1] > 0.0);
    }

    // [M 0x17e568:8-12] the delete removes the copter from the list.
    #[test]
    fn delete_removes_from_the_list() {
        let mut v = vec![copter(), copter()];
        v[1].placement = 7;
        assert!(Copter::delete(&mut v, 1));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].placement, 0);
        assert!(!Copter::delete(&mut v, 5), "out-of-range delete is a no-op");
    }
}
