//! The game gates the keypad mini-game on the pause-menu manager's existence —
//! `MenuManager_Exists` 0x21de90 (`manager[(id&0xff)·0x1d8 + 0x1D1]`, the in-use byte of the
//! 0x1D8-byte manager slots [M]: `Lock_Activate` 0x1aed00:12 refuses while pause manager 1
//! exists; `Lock_Update` 0x1ae3d8:272-291 enables/deletes/resets around it) — the port has no
//! menu-manager pool (the front-end is `nf-view`'s `FrontEnd`/`Page` state machine), so the
//! gate is structural: the lock cannot be activated while the sim is paused [S].
//!
//! The locks (`Lock_Create` 0x1ae1f8, `Lock_Update` 0x1ae3d8, `Lock_Activate` 0x1aed00,
//! `Lock_Unlock` 0x1b0650; placement type 0x2E, `parsemap_create_dynamic_objects` case 0x2E —
//! fix-objects2): the burnable locks the Q-Laser / laser watch burns through — 07000008's
//! `Object_SafeBolt_1` ×2 (the safe's two bolts, out channels 134/135 [D #207/#208]; mux #278
//! ORs them into ch 136, which opens the `Object_SafeDoor` SimpleScript and plays
//! `ScriptPlayer_SafePickUps`).
//!
//! The game's model [M]:
//! - `Lock_Create` (0x1ae1f8): `control_create_object(0x20, …)`, class 0x20; the record's u16 words
//!   at +4/+6 from placement params 3/4 (0xffff here — the take/key channels), the burn timer base
//!   `u16[+8] = (tag+2 == 0 ? 0 : tag+2 − 1)` scaled by `FRAME_RATE / u16[tag+4]`, flags2 |= 0x44,
//!   and the framelist pose (Spline_Interp) when the key count is non-zero. The difficulty gate
//!   (`0x1ae1f8:67-69`, GameState+0x40 DECIMAL = +0x28 hex = 0x2a3790): `level_tag+0x44` (param 6)
//!   non-zero AND the selected difficulty == 1 (Operative) -> `Lock_Unlock(obj, 1)` at create,
//!   which sets the lock's state word +0xf4 = 1 (unlocked) [M Lock_Unlock 0x1b0650:28-31] — the
//!   tagged locks start open on Operative only (docs/systems/progress.md "doors auto-unlock").
//! - `Lock_Update` (0x1ae3d8) case 0 reads the ACTIVATING weapon off the hit: 0x4E/0x4F (the
//!   Q-Decryptor, :57) or the laser family — the `− 0x56 < 2` band at :162 (0x56/0x57; the watch's
//!   scoped laser 0x54 arrives through the same hit path) — and advances the burn; the open writes
//!   `switch_channels[record u16[+8]] = 1` (:194-195) — placement param 5 [D #207: 0x86 = 134].
//! - `Lock_Activate` (0x1aed00): the E-interaction plays the locked/burnt sounds only; the burn is
//!   the laser's doing, not the use key.
//!
//! Port status (fix-party-mission): FIXED in play — the Lazer Watch 0x4E/0x4F's bullets burn the
//! lock through the bullet pass's segment-vs-sphere contact (`Lock_Update` case 0's cell walk, the
//! weapon id already on the port's BU), and the burn is instant (the create-time base is 0 on this
//! level's data [M 0x1ae1f8 — 07000008's #207/#208 param 2 = 0]); the chain test drives both bolts
//! through the real path (watch select, aim, fire) to ch 136. The watch's fp view-model beam and
//! the bolt meshes are still render-side work (0x2E is not in the port's dynamic draw list). The
//! gadget itself was previously not ported (docs/port-map/gadgets.md — all of 0x4A..0x5E are out of
//! selection), so the burn was a method the (future) `Player_LaserPointer` 0x1a65d8 calls per tick
//! while its ray hits the lock's world sphere, and tests could drive it directly. The framelist
//! open-animation is not posed — render-side, noted in the report.
//! ported (docs/port-map/gadgets.md — all of 0x4A..0x5E are out of selection), so the burn is a
//! method the (future) `Player_LaserPointer` 0x1a65d8 calls per tick while its ray hits the lock's
//! world sphere, and tests can drive it directly. The bolt meshes are not drawn yet (the 0x2E
//! placements are not in the port's dynamic draw list) and the framelist open-animation is not
//! posed — both render-side, noted in the report.

use crate::triggers::Channels;
use crate::world::V3;
use nf_formats::placement::Placement;

#[derive(Clone, Debug)]
pub struct Lock {
    pub placement: usize,
    /// The object's world sphere (the create's `build_LinkToRoom` position + the glist bounds).
    pub pos: V3,
    pub radius: f32,
    /// Record u16[+8] = placement param 5: the channel written on burn-through [M 0x1ae3d8:194].
    pub out: u16,
    /// Placement params 3/4 (the record's +4/+6 key/take words, 0xffff on the safe bolts [D]).
    pub key_channels: [u16; 2],
    /// Ticks of laser burn applied (`Lock_Update`'s timer). The create-time base is 0 on this
    /// level's data [M 0x1ae1f8: the tag+2 word is 0], so the first tick of burn completes it.
    pub burn: u32,
    pub burned: bool,
    /// level_tag+0x44 (param 6): the "starts unlocked on Operative" tag [M 0x1ae1f8:67-69].
    pub unlock_on_operative: bool,
    /// The create-time unlock state (the game's lock state word +0xf4 = 1 via `Lock_Unlock`
    /// 0x1b0650:28-31): true iff `unlock_on_operative` and the selected difficulty is 1 [M].
    pub unlocked: bool,
    /// Record u16[+0x00] = placement param 0 — the lock's kind: **1 = the laser-burn lock**
    /// (`Lock_Update` case 0's 0x4e/0x4f burn arm [M 0x1ae3d8:57-65]), **0 or 2 = the decryptor
    /// lock** (the case-0 decryptor arm and the use gate's `== 0 || == 2` test [M
    /// Player_WeaponFiring 0x1a5460:562]).
    pub kind: u16,
    /// Record u16[+0x02] = placement param 1 — the KeyCodes index (the code copied to the display
    /// buffer at scan start [M 0x1ae3d8:180], the key `Lock_Unlock` equips [M 0x1b0650:19]).
    pub code: u16,
    /// The decrypt scan's progress, the game's lock `obj+0xf6` (state 0 accumulates contacts,
    /// state 5 the +1/+3 per tick; the decay floor is 0 and the cap 3000 [M 0x1ae3d8:327-344]).
    pub scan_progress: u32,
    /// The solved digit count, `DAT_002a37e3` (0..4; 4 = the unlock [M 0x1aebec-0x1aebf4]).
    pub scan_solved: u8,
    /// The game's lock state word is 5 (scanning) while a scan runs [M 0x1ae3d8:188].
    pub scanning: bool,
}

impl Lock {
    /// `Lock_Create` over the main map's 0x2E placements. `difficulty` is the selected difficulty
    /// (`GameState+0x28`, 0x2a3790) — the param-6 tag auto-unlocks on Operative [M 0x1ae1f8:67-69].
    pub fn create(placements: &[Placement], difficulty: u32, object: impl Fn(&Placement) -> Option<(V3, f32)>) -> Vec<Lock> {
        let mut out = Vec::new();
        for (i, p) in placements.iter().enumerate() {
            if p.flags & 0xff != 0x2e { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let (centre, radius) = object(p).unwrap_or((p.translation, 1.0));
            out.push(Lock {
                placement: i,
                pos: [p.translation[0] + centre[0], p.translation[1] + centre[1], p.translation[2] + centre[2]],
                radius,
                out: param(5) as u16,
                key_channels: [param(3) as u16, param(4) as u16],
                burn: 0,
                burned: false,
                unlock_on_operative: param(6) != 0,
                unlocked: param(6) != 0 && difficulty == 1,
                kind: param(0) as u16,
                code: param(1) as u16,
                scan_progress: 0,
                scan_solved: 0,
                scanning: false,
            });
        }
        out
    }

    /// One tick of the laser's burn (`Player_LaserPointer` 0x1a65d8's hook — the gadget is not
    /// ported; see the module doc). `Lock_Update` writes the out channel when the burn completes
    /// [M 0x1ae3d8:194-195]. Returns whether it completed this tick.
    pub fn burn(&mut self, ch: &mut Channels, frame: u32) -> bool {
        if self.burned || self.unlocked { return false; }
        self.burned = true; // the create-time burn base is 0 [M 0x1ae1f8]
        if self.out != 0 { ch.set(self.out as u32, 1, frame); }
        true
    }

    /// `Lock_ReLock` 0x1b0750 — the re-arming of a burnt-open lock. The game's version:
    /// `if (player != 0 && lock != 0 && player->obj+0xff == 3 /* a player */) { lock->state
    /// (+0xf4) = 0; Player_UnEquipKey(player->BLData, lock->instance u16[+2]);
    /// Control_NextLOD(lock); }` — the lock goes back to state 0 (armed) and the key the player
    /// burned with is unequipped. Returns the key id to strip from the player (the port's
    /// `code` — record u16[+2], the same word `Lock_Unlock` 0x1b0650 equips [M]).
    pub fn re_lock(&mut self) -> Option<u16> {
        self.burned = false;
        self.unlocked = false;
        Some(self.code)
    }

    /// `Lock_Update` 0x1ae3d8 **case 2** (the burnt-open tick): the decryptor locks (kind 0)
    /// re-arm the moment their burn-through channel drops [M 0x1ae3d8 case 2: `if
    /// (instance u16[0] == 0 && u16[+8] != 0 && switch_channels[u16[+8]] == 0)
    /// Lock_ReLock(glb_players, lock)`]. `channel_on` reads that switch. Returns the key id to
    /// unequip when the re-lock fired. (Kind 1 = the laser-burn locks never re-arm here [M —
    /// the guard is `== 0`].)
    pub fn update_burnt_open(&mut self, channel_on: bool) -> Option<u16> {
        if self.kind == 0 && self.out != 0 && !channel_on {
            return self.re_lock();
        }
        None
    }

    /// The lock of a placement index (the render's per-instance lookup, when the meshes land).
    pub fn of_placement(list: &[Lock], placement: usize) -> Option<usize> {
        list.iter().position(|l| l.placement == placement)
    }

    /// One tick of the decrypt scan (`Lock_Update` 0x1ae3d8 case 0's decryptor arm + case 5).
    /// `beams` = the count of decryptor shots in the lock's contact list this tick (the port's
    /// stand-in: 1 while the held-fire gate passes — see gadget::pick_decryptor_lock), `upgraded`
    /// = the 0x57 (3/tick) vs the 0x56 (1/tick) [M 0x1ae3d8:305-315]. No beam decays −4/tick to a
    /// full reset (state 5 → 0, progress and digits lost) [M :327-344]. A non-decryptor lock
    /// (record[0] == 1) never scans — the use gate already refused it [M 0x1a5460:562]. Returns
    /// whether the unlock completed THIS tick (all four digits solved: the disasm's
    /// `bne v1,v0` 0x1aebec-0x1aebf4 — the decompile's "+1" is an artifact); the unlock writes the
    /// lock's out channel exactly as `Lock_Unlock` → state 1 → case 1 does [M 0x1b0650:28-31,
    /// 0x1ae3d8:193-196] (the port's one-tick fold, as the laser burn's).
    /// The digit thresholds are `progress > (4 − digit) · 0x28` (160/120/80/40) with the digit
    /// index capped at 3 [M :347-364], one solve per tick [M the single case-5 check], and a solve
    /// resets the progress to 5 [M :367].
    pub fn decrypt_tick(&mut self, beams: u32, upgraded: bool, ch: &mut Channels, frame: u32) -> bool {
        if self.burned || self.unlocked { return false; }
        if self.kind != 0 && self.kind != 2 { return false; }
        if beams == 0 {
            if !self.scanning { return false; }
            self.scan_progress = self.scan_progress.saturating_sub(4);
            if self.scan_progress == 0 {
                // The full reset [M 0x1ae3d8:340-344: memset GlobalVars, the flag cleared, state 0].
                self.scanning = false;
                self.scan_solved = 0;
            }
            return false;
        }
        self.scanning = true;
        self.scan_progress = (self.scan_progress + if upgraded { 3 } else { 1 })
            .min(3000); // the game's progress cap [M 0x1ae3d8:333-337]
        let digit = self.scan_solved.min(3) as u32;
        if (4 - digit) * 40 < self.scan_progress {
            self.scan_solved += 1;
            self.scan_progress = 5; // the solve resets the progress [M :367]
            if self.scan_solved >= 4 {
                self.unlocked = true; // Lock_Unlock 0x1b0650:28-31 → state 1 (leaves the scan state 5)
                self.scanning = false;
                if self.out != 0 { ch.set(self.out as u32, 1, frame); } // case 1 [M :193-196]
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock(kind: u16, out: u16, code: u16) -> Lock {
        Lock {
            placement: 0, pos: [0.0; 3], radius: 1.0, out, key_channels: [0xffff; 2],
            burn: 0, burned: false, unlock_on_operative: false, unlocked: false,
            kind, code, scan_progress: 0, scan_solved: 0, scanning: false,
        }
    }

    // [M 0x1ae3d8 case 2 + 0x1b0750] a kind-0 lock re-arms when its channel drops; the key comes back.
    #[test]
    fn burnt_open_decryptor_locks_rearm_when_the_channel_drops() {
        let mut l = lock(0, 0x88, 0x1234);
        l.burned = true;
        assert_eq!(l.update_burnt_open(true), None, "channel still on — no re-lock");
        assert!(l.burned);
        let key = l.update_burnt_open(false);
        assert_eq!(key, Some(0x1234), "the unequipped key is the lock's record u16[+2]");
        assert!(!l.burned && !l.unlocked, "state word back to 0");
        // Sister (can fail): a kind-1 (laser-burn) lock never re-arms here — the guard is `== 0`.
        let mut laser = lock(1, 0x88, 0x1234);
        laser.burned = true;
        assert_eq!(laser.update_burnt_open(false), None);
        assert!(laser.burned);
        // A kind-0 lock with no channel never re-arms (the game's `u16[+8] != 0` guard).
        let mut mute = lock(0, 0, 0x7);
        mute.burned = true;
        assert_eq!(mute.update_burnt_open(false), None);
    }
}
