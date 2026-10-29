//! `Player_UnEquipKey` 0x1a97b8 (16 B): `BL[0x70C + (u16)id] = 0` — the key-ring slot clear
//! [M 0x1a97b8:6]. Sole caller `Lock_ReLock` 0x1b0750:8 (guarded player-class-3), reached from
//! `Lock_Update` 0x1ae3d8 case 2 when the take channel drops: the single-use key re-locks the
//! door and removes Bond's key. The port's lock side does not consult or clear the key ring yet
//! (bldata.md:95's recorded gap — `Inventory::keys` pickup.rs:376 is the pickup side); the clear
//! itself is one line when `Lock_ReLock` lands.
//!
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
//! open-animation is not posed (render-side).
//! ported (docs/port-map/gadgets.md — all of 0x4A..0x5E are out of selection), so the burn is a
//! method the (future) `Player_LaserPointer` 0x1a65d8 calls per tick while its ray hits the lock's
//! world sphere, and tests can drive it directly. The bolt meshes are not drawn yet (the 0x2E
//! placements are not in the port's dynamic draw list) and the framelist open-animation is not
//! posed (both render-side).

use crate::drone::Rng;
use crate::triggers::Channels;
use crate::world::V3;
use nf_formats::placement::{view_rot_trans_matrix, Placement};

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
    /// The burn bytes, the game's lock `obj+0x104`/`obj+0x105` (u8 each). `control_init_object`
    /// 0x134630 sets both 0xff for every created object and `Lock_Create` 0x1ae1f8 writes neither,
    /// so a fresh lock holds 0xff/0xff [M]; the contact arm burns the laser's amount off BOTH
    /// (clamped at 0) and the unlock fires when `+0x105` drops under 4 [M 0x1ae3d8:82-101]; the
    /// idle arm refills both +1/tick to the 0xff cap [M :113-121].
    pub burn_a: u8,
    pub burn_b: u8,
    /// The idle/refill grace, the game's lock short `obj+0xf6` in case 0 kind 1: a contact tick
    /// stores 2, every no-contact tick decrements it, and the +1/tick refill runs only when it
    /// reads 0 [M 0x1ae3d8:79, 107-121]. (The decryptor arm reuses +0xf6 as the scan counter —
    /// the two arms never share a lock: kind 1 burns, kinds 0/2 scan [M :47, :62].)
    pub grace: u8,
    /// The decrypt scan's progress, the game's lock `obj+0xf6` (state 0 accumulates contacts,
    /// state 5 the +1/+3 per tick; the decay floor is 0 and the cap 3000 [M 0x1ae3d8:327-344]).
    pub scan_progress: u32,
    /// The solved digit count, `DAT_002a37e3` (0..4; 4 = the unlock [M 0x1aebec-0x1aebf4]).
    pub scan_solved: u8,
    /// fix-decryptor-sounds: the sounds this tick's scan earned, drained by the caller and played at
    /// the lock's position ([M `Lock_Update` 0x1ae3d8:368 `Sound_Play(100.0, 0x5db, 0, 0)` per solved
    /// digit, :374 `Sound_Play(100.0, 0x5da, 0, 0)` on the unlock). The game's `Sound_Play` is the
    /// non-positional family; the port's only path is the 3-D one, so the caller plays it at `pos`.
    pub pending_sounds: Vec<u32>,
    /// The game's lock state word is 5 (scanning) while a scan runs [M 0x1ae3d8:188].
    pub scanning: bool,
    /// `Control_NextLOD` 0x1349b0 has run: `obj+0xD8` is the glist's LOD variant (the keypad's
    /// "Lock - KeypadUnlocked" [D block 0x27]). The player unlock swaps at once [M Lock_Unlock
    /// 0x1b0650:23] and case 1 swaps again when the open framelist ends [M 0x1ae3d8 case 1]; the
    /// chain ends at the variant (its +0x40 is null), so a re-lock's NextLOD [M 0x1b0750] is a no-op
    /// and the variant stays. The draw shows the variant from the first unlock on.
    pub lod_next: bool,
    /// `Mat_GetDir(obj+0x90)` — the lock's facing (the placement rotation's matrix row 2), the
    /// use-pick facing gate's direction: `Player_Activate` 0x1a5170's class-0x3A arm refuses the
    /// activation when `dir · (user+0xC0 − lock+0xC0) < 0` (behind the panel) [M 0x1a5170:158-166].
    pub facing: V3,
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
            // The placement's object matrix (translation unused for the direction) [M 0x1a5170:158].
            let m = view_rot_trans_matrix(p.rotation, [0.0; 3]);
            out.push(Lock {
                placement: i,
                pos: [p.translation[0] + centre[0], p.translation[1] + centre[1], p.translation[2] + centre[2]],
                radius,
                out: param(5) as u16,
                key_channels: [param(3) as u16, param(4) as u16],
                burn: 0,
                burned: false,
                burn_a: 0xff,
                burn_b: 0xff,
                grace: 0,
                unlock_on_operative: param(6) != 0,
                unlocked: param(6) != 0 && difficulty == 1,
                kind: param(0) as u16,
                code: param(1) as u16,
                scan_progress: 0,
                scan_solved: 0,
                pending_sounds: Vec::new(),
                scanning: false,
                lod_next: false,
                facing: [m[2][0], m[2][1], m[2][2]],
            });
        }
        out
    }

    /// One tick of LASER CONTACT (`Lock_Update` 0x1ae3d8 case 0, kind 1, per contact entry):
    /// `amount` is the contact's per-tick burn — the game reads the contact object's weapon id
    /// off its `link+0x44` record (**0x4E → 3, 0x4F → 10** [M :57-66]) and subtracts it from BOTH
    /// burn bytes, clamped at 0 [M :82-95]; a contact tick stores the grace 2 [M :79]; when
    /// `+0x105` drops under 4 the lock completes: sound 0x249 at the lock [M :96-99] then
    /// `Lock_Unlock`, which writes `switch_channels[record+8]` [M :194-195, 0x1b0650]. Returns
    /// whether it completed THIS tick.
    pub fn burn(&mut self, amount: u8, ch: &mut Channels, frame: u32) -> bool {
        if self.burned || self.unlocked { return false; }
        self.grace = 2; // *(lock+0xf6) = 2 on the contact tick [M 0x1ae3d8:79]
        self.burn_a = self.burn_a.saturating_sub(amount);
        self.burn_b = self.burn_b.saturating_sub(amount);
        if self.burn_b >= 4 { return false; }
        self.burned = true;
        if self.out != 0 { ch.set(self.out as u32, 1, frame); }
        true
    }

    /// One tick with NO contact (`Lock_Update` case 0 kind 1's tail): while the grace is up it
    /// counts down, and at 0 BOTH burn bytes refill +1 to the 0xff cap [M 0x1ae3d8:107-121 —
    /// the partial burn heals, ~1 tick per unit]. The decryptor arms reuse +0xf6 for the scan
    /// count, so this runs for kind-1 locks only (the port drives it from `Drone::tick`'s
    /// per-tick pass over the locks).
    pub fn idle_tick(&mut self) {
        if self.burned || self.unlocked { return; }
        if self.kind != 1 { return; }
        if self.grace > 0 { self.grace -= 1; return; }
        self.burn_a = self.burn_a.saturating_add(1);
        self.burn_b = self.burn_b.saturating_add(1);
    }

    /// THE TEST-PREMISE SHORTCUT (not the game path): completes the burn in one call — the state
    /// a level arrives with when a premise test needs the lock already gone. The REAL path is the
    /// ~84 contact ticks of `burn(3)` (0x4E from 0xff [M control_init_object 0x134630 sets both
    /// bytes 0xff; 0x1ae3d8:82-101]) driven by the bullet pass; only premise tests (the mux/SS
    /// sides) use this.
    pub fn burn_through(&mut self, ch: &mut Channels, frame: u32) -> bool {
        if self.burned || self.unlocked { return false; }
        self.burned = true;
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
        // Only the case-2 (burnt-open) tick runs this [M 0x1ae3d8:220-232]; the port's `burned`
        // is that state [S mapping]. Any other state never re-locks.
        if !self.burned {
            return None;
        }
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
    /// resets the progress to 5 [M :367]. The FIRST contact after a reset only flips state 0 → 5:
    /// the bullet's `+0xf6 += 1` makes the progress non-zero and the `sVar4 != 0` arm then moves
    /// the lock to state 5 and resets the progress to 0 — nothing banks on the flip tick
    /// [M 0x1ae3d8:163-190].
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
        let was_scanning = self.scanning;
        self.scanning = true;
        if !was_scanning {
            // The state-0 entry [M 0x1ae3d8:179-189]: the first contact only enters state 5 and
            // RESETS the progress to 0 — the transition consumes the contact, it does not decode.
            self.scan_progress = 0;
            return false;
        }
        self.scan_progress = (self.scan_progress + if upgraded { 3 } else { 1 })
            .min(3000); // the game's progress cap [M 0x1ae3d8:333-337]
        let digit = self.scan_solved.min(3) as u32;
        if (4 - digit) * 40 < self.scan_progress {
            self.scan_solved += 1;
            self.scan_progress = 5; // the solve resets the progress [M :367]
            // fix-decryptor-sounds: one chime per solved digit [M 0x1ae3d8:368].
            self.pending_sounds.push(0x5db);
            if self.scan_solved >= 4 {
                self.unlocked = true; // Lock_Unlock 0x1b0650:28-31 → state 1 (leaves the scan state 5)
                self.scanning = false;
                // fix-decryptor-sounds: the unlock chime [M 0x1ae3d8:374].
                self.pending_sounds.push(0x5da);
                if self.out != 0 { ch.set(self.out as u32, 1, frame); } // case 1 [M :193-196]
                return true;
            }
        }
        false
    }
}

/// The keypad code table: `Locks_Init` 0x1b0548's `KeyCodes` 0x2a37a8 (51 × 5 bytes — four digit
/// chars + a display flag). Boot warms the RNG with `GameState+0x34 · 10 & 0x1ff` draws and fills
/// each entry as `(Rand_Random() % 9) + '0'` — digits '0'..'8' only — then `KeyCodes[i*5+4] = 0`
/// [M 001b0548_Locks_Init__Fv.c:18-33]. The Q-Worm burn sets the entry's flag byte [M
/// 001aee20_PCQWorm_Update__FP7obj_tag.c:31], and `Lock_Update` case 3 shows the code on the
/// keypad page when it is set [M 0x1ae3d8:249-258]. The port generates the table with a LOCAL
/// `Rng` ([S] the real stream position comes out of the boot path the port's level-start snapshot
/// pre-consumes; the count of draws, the digit domain and the entry shape are [M]).
pub struct KeyCodes(pub [[u8; 5]; 0x33]);

impl KeyCodes {
    /// `Locks_Init` 0x1b0548 over a private stream. `warmup` is the game's
    /// `GameState+0x34 · 10 & 0x1ff` value ([M :17] — the port passes it in; the shared-stream
    /// draws it stands for are pre-consumed by the snapshot).
    pub fn init(rng: &mut Rng, warmup: u32) -> KeyCodes {
        for _ in 0..(warmup.wrapping_mul(10) & 0x1ff) {
            rng.random();
        }
        let mut out = [[0u8; 5]; 0x33];
        for (i, e) in out.iter_mut().enumerate() {
            for d in e.iter_mut().take(4) {
                *d = (rng.random() % 9) as u8 + b'0';
            }
            let _ = i;
            e[4] = 0;
        }
        KeyCodes(out)
    }

    /// The 4-digit code for lock placement param 1 (the record's u16[+2] indexes the table
    /// [M 0x1ae3d8:181 `KeyCodes + puVar1[1]*5`]).
    pub fn code(&self, index: u16) -> [u8; 4] {
        let e = &self.0[(index as usize) % self.0.len()];
        [e[0], e[1], e[2], e[3]]
    }

    /// The entry's display flag — the Q-Worm reveal [M 0x1aee20:31].
    pub fn shown(&self, index: u16) -> bool {
        self.0[(index as usize) % self.0.len()][4] != 0
    }
}

/// The in-game keypad mini-game — page 0x40000021 of the per-level menu def (kind-8 entry
/// 08000001, page file-hash 0x80000003 [D census: present with control 0x100000ba in
/// 07000003/05/08/09/0b/0c/0d/11/12/13/14/15/16/1b/23/25/26/27/28.bin]), created by
/// `Lock_Update` case 3 (`MenuManager_Create(0x80000003, 0x40000021, …)` [M 0x1ae3d8:242-243]).
///
/// Game model [M]:
/// - arm (case 3, 0x1ae3d8:176-190): the 5-byte entry `KeyCodes[param1]` is memcpy'd to
///   0x2a37d8; the type cap 0x2a37e2 = 4; the count 0x2a37e3 = 0; the typed buffer
///   0x2a37dd..e0 = '*' (0x2a) — the display masks untyped positions; lock +0xf4 = 5.
/// - a digit button (control 0x100000ba → `C_KEYPAD_Handler` 0x216810, msg 0x4b): if the count
///   reached the cap it wraps to 0 (:8-10); `typed[count] = '0' + control+0x20`; the display
///   string 0x30d8c8 = typed[0..=count] NUL-terminated (:13-21); msg 0x18 (set text) to the
///   display control 0x10000075; count++.
/// - msg 0x51 (the page's refresh): rebinds the display and sends 0x62 (the beep) [M :24-29].
/// - match (case 4, 0x1ae3d8:262-296): typed[i] == code[i] for all i < cap AND count == cap →
///   `Lock_Unlock` (state 1, the out channel), the keypad manager deleted; if the page was
///   closed without a match → state 0 (re-armable). The per-frame `MenuManager_Enable(1, slot)`
///   keep-alive is the manager-pool housekeeping the port folds into `open` staying true.
///
/// The view draws: the keypad page's 24 controls as laid out in the level def —
/// the digit grid, and the display label 0x10000075 bound to [`Self::display`]; when
/// `code_shown` the hint line shows the code (`Lock_Update` case 3 binds the sprintf'd
/// "%d%d%d%d" through Txt_BindLabel 0x2000014 + Text_AddMsg 0xb4 [M 0x1ae3d8:249-258]).
#[derive(Clone, Debug)]
pub struct Keypad {
    /// The code being typed against (0x2a37d8..dc).
    pub code: [u8; 4],
    /// KeyCodes[i*5+4]: the Q-Worm reveal (the code prints on the page) [M 0x1ae3d8:249-258].
    pub code_shown: bool,
    /// The typed buffer 0x2a37dd..e0 — starts as '*' masks [M 0x1ae3d8:185-187].
    pub typed: [u8; 4],
    /// The type cap 0x2a37e2 = 4 [M 0x1ae3d8:182].
    pub cap: u32,
    /// The type count 0x2a37e3 [M 0x216810].
    pub count: u32,
    /// The keypad page exists (the game's manager slot in use; `MenuManager_Exists`).
    pub open: bool,
    /// The match fired (`Lock_Unlock` ran) — the caller writes the lock's out channel + state 1.
    pub solved: bool,
}

impl Keypad {
    /// `Lock_Update` case 3's arming [M 0x1ae3d8:176-190].
    pub fn arm(code: [u8; 4], code_shown: bool) -> Keypad {
        Keypad { code, code_shown, typed: [b'*'; 4], cap: 4, count: 0, open: true, solved: false }
    }

    /// `C_KEYPAD_Handler` 0x216810 msg 0x4b — a digit button press. `d` is the button's
    /// control+0x20 value (the digit) [M :13 `* (param_2 + 0x20) + '0'`].
    pub fn digit(&mut self, d: u32) {
        if !self.open { return; }
        if self.cap <= self.count {
            self.count = 0; // the wrap [M 0x216810:8-10]
        }
        self.typed[self.count as usize] = d as u8 + b'0';
        self.count += 1;
    }

    /// The display string for the label control 0x10000075 (msg 0x18 set-text target
    /// [M 0x216810:13-22]): the game copies typed[0..=count_before] INCLUSIVE — the just-typed
    /// digit included, i.e. `typed[..count]` with the post-increment count — then NULs the tail.
    pub fn display(&self) -> String {
        let n = (self.count as usize).min(4);
        String::from_utf8_lossy(&self.typed[..n]).into_owned()
    }

    /// The Q-Worm hint line — `sprintf("%d%d%d%d", …)` bound through Txt_BindLabel 0x2000014
    /// [M 0x1ae3d8:249-258]. Empty unless the code is revealed.
    pub fn hint(&self) -> String {
        if !self.code_shown { return String::new(); }
        self.code.iter().map(|&c| (c as char).to_string()).collect()
    }

    /// `Lock_Update` case 4's match arm [M 0x1ae3d8:262-296]. One tick of the open page: the
    /// full buffer matching the code with the count at the cap unlocks (`Lock_Unlock` → the
    /// caller plays 0x42c80000/0xbf800000 unlock sound and closes the manager). Returns whether
    /// it solved this tick.
    pub fn tick(&mut self) -> bool {
        if !self.open || self.solved { return false; }
        let full = self.count >= self.cap;
        let same = self.typed.iter().zip(self.code.iter()).all(|(t, c)| t == c);
        if full && same {
            self.solved = true; // Lock_Unlock 0x1b0650 (state 1 + the out channel) [M :283-289]
            self.open = false; // MenuManager_Delete(slot) [M :288]
            return true;
        }
        false
    }

    /// The page closed without a match (the game's Exists==0 arm: state back to 0, re-armable
    /// [M 0x1ae3d8:292-295]).
    pub fn close(&mut self) {
        self.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock(kind: u16, out: u16, code: u16) -> Lock {
        Lock {
            placement: 0, pos: [0.0; 3], radius: 1.0, out, key_channels: [0xffff; 2],
            burn: 0, burned: false, unlock_on_operative: false, unlocked: false,
            kind, code, scan_progress: 0, scan_solved: 0, pending_sounds: Vec::new(), scanning: false, lod_next: false,
            burn_a: 0xff, burn_b: 0xff, grace: 0,
            facing: [0.0, 0.0, 1.0],
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
        // A kind-0 lock that is not burnt open (state != 2) never re-locks [M 0x1ae3d8 case 2 only].
        let mut idle = lock(0, 0x88, 0x1234);
        assert_eq!(idle.update_burnt_open(false), None);
    }
}
