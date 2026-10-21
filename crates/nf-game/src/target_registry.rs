//! The guards' shared target registry (NPCGlobals' 8 × 0x20-byte slots at DAT_00299110): the
//! last-seen-position pool the sight/talk/comms systems share. Lifted [M]-exact from
//! `DroneFunc_AllocateTargetID` 0x141f28 and `DroneFunc_TargetSeen` 0x14a008; the readers
//! (`Drone_GetOpponentInfo` 0x138f68's stamp gate, `Drone_IsCoverNodeUsable` 0x1386d8,
//! `Drone_InitComms` 0x139520) and the deviations are documented in docs/systems/dronefunc-rest.md.

/// One NPCGlobals slot: +0 flag word (bit 0 claimed, bit 1 seen-since-allocate — `TargetSeen`
/// sets it), +4 the target obj, +8 the last-seen frame, +0x10 the 16-byte last-seen position.
#[derive(Clone, Debug)]
pub struct TargetSlot {
    pub flags: u32,
    pub obj: usize,
    pub stamp: u32,
    pub pos: [f32; 3],
}

impl Default for TargetSlot {
    fn default() -> Self {
        TargetSlot { flags: 0, obj: usize::MAX, stamp: 0, pos: [0.0; 3] }
    }
}

/// The 8-slot pool. `GameState+52` (the game's frame counter) plays `stamp`.
#[derive(Clone, Debug, Default)]
pub struct TargetRegistry {
    pub slots: [TargetSlot; 8],
}

impl TargetRegistry {
    /// `DroneFunc_AllocateTargetID` 0x141f28 [M :11-40]: `eligible` is drone+0x44 == 1 (only full
    /// NPCs allocate; anything else gets 0xff). A slot already claiming `obj` returns its index;
    /// else the first slot with flag bit 0 clear is claimed (flags = 1, obj stored, stamp and the
    /// position zeroed — the game zeroes +0x8/+0x10/+0x14/+0x18) and its index returned; all 8
    /// held by others → 0xff.
    pub fn allocate(&mut self, eligible: bool, obj: usize) -> u8 {
        if !eligible {
            return 0xff;
        }
        for (i, s) in self.slots.iter().enumerate() {
            if s.flags & 1 != 0 && s.obj == obj {
                return i as u8;
            }
        }
        if let Some((i, s)) = self.slots.iter_mut().enumerate().find(|(_, s)| s.flags & 1 == 0) {
            *s = TargetSlot { flags: 1, obj, stamp: 0, pos: [0.0; 3] };
            return i as u8;
        }
        0xff
    }

    /// `DroneFunc_TargetSeen` 0x14a008 [M :12-24]: a valid id (not 0xff) stamps the frame, sets
    /// flag bit 1 and stores the 16-byte position at slot+0x10. The game's guard is the drone's
    /// own allocated id byte (+0x174); an unclaimed slot is left alone.
    pub fn target_seen(&mut self, id: u8, stamp: u32, pos: [f32; 3]) {
        if id == 0xff {
            return;
        }
        let s = &mut self.slots[id as usize];
        if s.flags & 1 == 0 {
            return;
        }
        s.stamp = stamp;
        s.flags |= 2;
        s.pos = pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [M 0x141f28:11-40] allocation: fresh NPC → slot 0; the same obj reuses its slot; the +0x44
    /// gate returns 0xff; the 8-slot exhaustion returns 0xff. Sisters flip each arm.
    #[test]
    fn allocate_follows_the_game_scan() {
        let mut r = TargetRegistry::default();
        assert_eq!(r.allocate(false, 7), 0xff, "+0x44 != 1 never allocates [M :11-13]");
        assert_eq!(r.allocate(true, 7), 0, "the first free slot [M :26-33]");
        assert_eq!(r.allocate(true, 7), 0, "the same obj reuses its slot [M :16-21]");
        assert_eq!(r.allocate(true, 8), 1);
        for i in 2..8 {
            assert_eq!(r.allocate(true, 100 + i as usize), i);
        }
        assert_eq!(r.allocate(true, 999), 0xff, "all 8 held by others → 0xff [M :36-38]");
        // Sister: freeing one (a fresh registry with one gap) lets the scan fill it.
        let mut r2 = TargetRegistry::default();
        for i in 0..8 {
            r2.allocate(true, 10 + i);
        }
        r2.slots[5].flags = 0; // the game's free slots are flag bit 0 clear [M :26]
        assert_eq!(r2.allocate(true, 999), 5, "the freed slot is the next allocation");
    }

    /// [M 0x14a008:12-24] seen: the stamp lands, flag bit 1 sets, the position stores; 0xff and
    /// unclaimed slots are no-ops.
    #[test]
    fn target_seen_stamps_the_slot() {
        let mut r = TargetRegistry::default();
        let id = r.allocate(true, 42);
        assert_eq!(id, 0);
        r.target_seen(0xff, 1234, [1.0, 2.0, 3.0]);
        assert_eq!(r.slots[0].stamp, 0, "id 0xff is a no-op [M :13]");
        r.target_seen(id, 1234, [1.0, 2.0, 3.0]);
        assert_eq!(r.slots[0].stamp, 1234, "GameState+52 lands at slot+8 [M :15]");
        assert_eq!(r.slots[0].flags & 2, 2, "flag bit 1 sets [M :16]");
        assert_eq!(r.slots[0].pos, [1.0, 2.0, 3.0], "the vector stores at slot+0x10 [M :20-23]");
        // Sister: an unclaimed slot is never written.
        let mut r2 = TargetRegistry::default();
        r2.target_seen(3, 7, [9.0; 3]);
        assert_eq!(r2.slots[3].stamp, 0);
        assert_eq!(r2.slots[3].flags, 0);
    }
}
