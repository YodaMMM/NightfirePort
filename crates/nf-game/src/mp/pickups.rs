//! mp-pickups-respawn-2: the MP registered-pickup layer and the enviro toggles — re-landed from
//! the never-merged `s/mp-pickups-respawn` (task 648) onto the current master shapes. The weapon
//! set table, the random re-roll and the spawn loadout live in [`crate::mp::equip`] (mp-lobby-
//! settings landed them); this module carries what master still lacked:
//!
//! - the 64 `MPpickups` records (@0x2a4b50, 0xa0 bytes each) with `MP_RegisterPickup` 0x189d80's
//!   free-slot search / lifetime eviction, `MP_UnregisterPickup` 0x18a058 and
//!   `MP_Pickup_Process` 0x18a168's visit-stamp expiry;
//! - the kind→weapon gate of `Pickup_CreateFromSet` 0x18f1b0;
//! - the enviro toggles `GunImp_Create` 0x131f20 and `Car_Create` 0x1305b0.
//!
//! Evidence tags: [M] the decompile (`work/decomp/ACTION.ELF/<addr>_<name>.c`, `:NN` = line),
//! [D] ACTION.ELF bytes, [S] port choice. Ghidra names offsets in DECIMAL (`MPSettings._472_2_`
//! = the u16 at hex +0x1D8); this file cites hex offsets.
//!
//! Corrections over the old branch (found by re-reading the decompile in this task):
//! - the registered count is `MPSettings+0x1D8` u16 and `MP_Pickup_Process`/the eviction scan
//!   walk rows until they have SEEN that many NON-NULL rows [M 0x18a168:9-14, 0x189d80:52-72] —
//!   the old port walked the first `count` ROWS regardless;
//! - the eviction goes through `MP_UnregisterPickup` [M 0x189d80:76], so the count is net
//!   UNCHANGED on a full-ring register (−1 then +1) and `PickupLastDeletedIndex` ends up set to
//!   the reused slot [M 0x18a058:24];
//! - `PickupNextAddIndex` wraps to 0 only when the slot was > 0x3e [M 0x189d80:103-105];
//! - the process clock is `MPGame+0x19C` [M 0x18a168:17 (`MPGame._412_4_`)].

use crate::drone::Rng;

/// One `MPpickups` record (0xa0 bytes @0x2a4b50, 64 entries) — the fields the game's MP code
/// reads: the registered pickup object (the game stores the `obj_tag*` at record+0 [M
/// 0x189d80:79]), the placement position (obj+0x30 copied to record+0x10 [M :84-88]), the cel
/// (record+0x20 [M :89]) and the four per-slot visit stamps (record+0x80, `float[4]` —
/// `MP_Pickup_Process` 0x18a168:18-26 walks exactly 4).
#[derive(Clone, Copy, Debug, Default)]
pub struct MpPickupRecord {
    /// The registered pickup (the port's pickup index; `None` = the game's null pointer).
    pub obj: Option<usize>,
    /// record+0x10: the pickup's position.
    pub pos: [f32; 3],
    /// record+0x20: the cel record.
    pub cel: u32,
    /// record+0x80: the four per-slot "visited until" stamps (`MPGame+0x19C` clock floats).
    pub visit: [f32; 4],
    /// `PICKUPINFO+0x2E` (the eviction key): the pickup's lifetime ticks [M 0x189d80:56-71].
    pub lifetime: u16,
}

/// The MP registered-pickup store: the 64 `MPpickups` records plus the ring cursors
/// `PickupNextAddIndex` / `PickupLastDeletedIndex` (@0x30d77c/@0x30d778; `MP_Init` 0x183438
/// clears the records and inits the cursors [M 0x183438:37-43]). The registered count itself is
/// `MpSettings::registered_pickups` (the u16 at +0x1D8) — the caller's, like the game's [M
/// 0x189d80:107]. The weapon-set matrix is NOT modelled here: master keeps it in
/// [`crate::mp::equip`] and derives the live row per match (`equip::weapon_row`).
#[derive(Clone, Debug)]
pub struct MpPickups {
    /// The 64 records (`memset 0x2800` at `MP_Init` 0x183438:37 — 0x2800 = 64 × 0xa0).
    pub records: Vec<MpPickupRecord>,
    /// `PickupNextAddIndex` (init 0 [M 0x183438:40,43]).
    pub next_add: usize,
    /// `PickupLastDeletedIndex` (init 0xffffffff [M 0x183438:40]; `None` = −1).
    pub last_deleted: Option<usize>,
}

impl MpPickups {
    /// The fresh store: the record array cleared, the cursors at the `MP_Init` values.
    pub fn new() -> MpPickups {
        MpPickups { records: vec![MpPickupRecord::default(); 64], next_add: 0, last_deleted: None }
    }

    /// `MP_RegisterPickup` 0x189d80's slot search + store. `count` is the caller's registered
    /// count (`MPSettings+0x1D8`) — the game's eviction budget [M :52-73]. Returns
    /// `(slot, evicted)`: `slot == usize::MAX` is the game's −1 (nothing free and nothing
    /// evictable), `evicted` says the chosen slot was taken from a registered row (the game
    /// calls `MP_UnregisterPickup` for it [M :76], so the caller's count ends net unchanged;
    /// without an eviction it bumps by one [M :107]).
    pub fn register(&mut self, obj: usize, pos: [f32; 3], cel: u32, lifetime: u16, count: u16) -> (usize, bool) {
        let mut slot: Option<usize> = None;
        let mut evicted = false;
        // `PickupLastDeletedIndex` first, while it still points at a free row [M :29-31 + :79].
        if let Some(d) = self.last_deleted {
            if self.records[d].obj.is_none() {
                slot = Some(d);
                self.last_deleted = None;
            }
        }
        // Else the first free row from `PickupNextAddIndex` up to 0x40 [M :33-39].
        if slot.is_none() {
            let mut i = self.next_add;
            while i < 64 && self.records[i].obj.is_some() {
                i += 1;
            }
            if i < 64 {
                slot = Some(i);
            }
        }
        // Else the wrap scan 1..PickupNextAddIndex (only when the cursor has moved past 1) [M
        // :42-51 — `if (0 < PickupNextAddIndex)`].
        if slot.is_none() && self.next_add > 0 {
            let mut i = 1;
            while i < self.next_add && self.records[i].obj.is_some() {
                i += 1;
            }
            if i < self.next_add {
                slot = Some(i);
            }
        }
        // Full: evict the registered pickup with the smallest non-zero `PICKUPINFO+0x2E`
        // lifetime among the first `count` NON-NULL rows — the scan walks rows until it has
        // seen `count` non-null ones, exactly like `MP_Pickup_Process` [M :52-73].
        let slot = match slot {
            Some(s) => s,
            None => {
                let mut best: Option<usize> = None;
                let mut best_life = 0xffffu16;
                let mut seen = 0u16;
                for i in 0..64 {
                    if seen >= count {
                        break;
                    }
                    if self.records[i].obj.is_some() {
                        seen += 1;
                        if self.records[i].lifetime != 0 && self.records[i].lifetime < best_life {
                            best_life = self.records[i].lifetime;
                            best = Some(i);
                        }
                    }
                }
                match best {
                    Some(b) => {
                        evicted = true;
                        // `MP_UnregisterPickup` on the victim [M :76]: the row clears,
                        // `PickupLastDeletedIndex` records the slot [M 0x18a058:24] — and the
                        // caller's count drops by one before the +1 below nets it out.
                        self.records[b] = MpPickupRecord::default();
                        self.last_deleted = Some(b);
                        b
                    }
                    None => return (usize::MAX, false),
                }
            }
        };
        self.records[slot] = MpPickupRecord { obj: Some(obj), pos, cel, visit: [0.0; 4], lifetime };
        self.next_add = slot + 1;
        if slot > 0x3e {
            self.next_add = 0;
        }
        (slot, evicted)
    }

    /// `MP_UnregisterPickup` 0x18a058's record side: the game gates on MP on, the obj present,
    /// its kind 0x2f and the obj's own slot field matching, then zeroes the 0xa0 row, records
    /// `PickupLastDeletedIndex` and drops the count [M :5-31]. The port's caller passes the
    /// row's slot and owns the count.
    pub fn unregister(&mut self, slot: usize) {
        self.records[slot] = MpPickupRecord::default();
        self.last_deleted = Some(slot);
    }

    /// `MP_Pickup_Process` 0x18a168: walk the rows from 0, and for each NON-NULL row zero the
    /// four visit stamps that are set and behind the `MPGame+0x19C` clock — until `count`
    /// non-null rows have been visited [M :9-27].
    pub fn process(&mut self, clock: f32, count: u16) {
        let mut seen = 0u16;
        for rec in self.records.iter_mut() {
            if seen >= count {
                break;
            }
            if rec.obj.is_none() {
                continue;
            }
            seen += 1;
            for v in rec.visit.iter_mut() {
                if *v != 0.0 && *v < clock {
                    *v = 0.0;
                }
            }
        }
    }

    /// `BOTSTATE_setPickupVisitTime` 0x129958: the bot's stamp on the record becomes
    /// `MPGame+0x19C` + 45.0, +0.1 when that would be 0.0 [M :9-15]. The stamp slot is the bot's
    /// roster id; the record carries 4 floats (the process manages exactly 4 [M 0x18a168:18-26]
    /// and the game's bot count clamps at 4 [M 0x183a50:133-135]), so ids ≥ 4 do not fit — the
    /// port's own >4-bot extension lives in the mp-bot-weapons `BotPickup` view, not here [S].
    pub fn set_visit_time(&mut self, record: usize, bot: usize, clock: f32) {
        if record >= 64 || bot >= 4 {
            return;
        }
        let mut t = clock + 45.0;
        if t == 0.0 {
            t += 0.1;
        }
        self.records[record].visit[bot] = t;
    }
}

impl Default for MpPickups {
    fn default() -> MpPickups {
        MpPickups::new()
    }
}

/// `Pickup_CreateFromSet` 0x18f1b0:56's read, as a pure gate: the weapon is the u16 at
/// `PickupMatrix + set·10 + (kind−7)·2` (byte offsets — `Pickup_MakeRandomWeaponSet` 0x18f098
/// writes row 10 at byte offsets 100..108), `set` = the u16 at `MPSettings+0x1B4` [M]. `row` is
/// the set's row; kinds outside 7..=11 never reach the table (`Pickup_Create` 0x18dda0:104's
/// `uVar7 - 7 < 5` intercept).
pub fn weapon_for_kind(row: &[u16; 5], kind: u32) -> Option<u16> {
    if !(7..=11).contains(&kind) {
        return None;
    }
    Some(row[(kind - 7) as usize])
}

/// `GunImp_Create` 0x131f20:50-62's weapon class: in SP (`master == 0`, `MPSettings+0x180`) class
/// 0xb; in MP the toggle `MPSettings+0x1B8` 0 creates no emplacement (`None`), the Random set
/// (10 — the u16 at +0x1B4) draws `Rand(10)`, otherwise the class is the weapon set id itself.
pub fn gun_imp_class(master: u32, enabled: u32, set: u32, rng: &mut Rng) -> Option<i32> {
    if master == 0 {
        return Some(0xb);
    }
    if enabled == 0 {
        return None;
    }
    if set == 10 {
        Some(rng.rand(10) as i32)
    } else {
        Some(set as i32)
    }
}

/// `Car_Create` 0x1305b0's vehicle kind: `None` = no vehicle (MP with `MPSettings+0x1CC` == 0 [M
/// :13-15], or the Ravine arena 0x700004b [M :17-19]). Otherwise the game's obj+0x112 decision
/// tree [M :42-52]: 2 → helicopter; < 3 → helicopter unless 1 (tank); ≥ 3 and != 3 → helicopter;
/// 3 → `Rand(100) < 0x33` (51 %) → helicopter else tank. The returned byte is the game's
/// `obj+0x112` (0 tank, 1 helicopter); the 8-vehicle cap (`7 < NumTanks` [M :21]) stays with the
/// caller. SP reaches the same tree with the boot 0 (+0x1CC) → helicopter [M 0x1c7e78:42].
pub fn car_kind(master: u32, vehicles: u32, level: u32, rng: &mut Rng) -> Option<u8> {
    if master != 0 && vehicles == 0 {
        return None;
    }
    if level == 0x7000_04b {
        return None;
    }
    let heli = if vehicles == 2 {
        true
    } else if vehicles < 3 {
        vehicles != 1
    } else if vehicles != 3 {
        true
    } else {
        rng.rand(100) < 0x33
    };
    Some(if heli { 1 } else { 0 })
}
