//! The 0700001b space laser — `Create_SpaceLaser` 0x1aa8b8 (class 0x50 → `SpaceLaser_Update`
//! 0x1aa3f8, docs/atlas/object.md:164) [M 001aa8b8].
//!
//! The game's only caller is `parsemap_create_dynamic_objects` 0x1d04f0 **case 0x38** (line 227,
//! the dynamic-object placement walk). The disc ships exactly one kind-0x38 placement: 0700001b
//! PL#108 — the station's laser hazard, armed by switch channel 19, firing at the player every
//! 10 s, killed by channel 14 (Drake's death) [D script-census.md:144; docs/levels/0700001b.md:71].
//! The sibling kind-0x39 SpaceMissile is already ported (`crate::space_missile`, the same walk).
//!
//! Param map [M 001aa8b8:14-37] — the placement record's param words (rec+0x2c = param 0, the
//! same convention `Create_SpaceMissile` 0x1a9ef0 uses):
//! - ext+0x20/+0x24/+0x28 = the raw f32 bits of params 0/1/2,
//! - ext+0x30/+0x34 = `(float)(int)` of params 3/4 (signed int → float),
//! - ext+0x38 = 10.0f (the constant 0x41200000 — the 10 s fire period),
//! - ext+0x40/+0x42/+0x44 = the low u16 of params 5/6/7,
//! - obj+0xff = 0x50 (class), obj+0xf4 = 0 (the update state).
//!
//! `SpaceLaser_Update` 0x1aa3f8 (the arm/fire/damage flow) is the remaining gap — this module
//! ports the creator's data so the update port (and the draw code) has its state. Sound:
//! `sound-coverage.md:95` lists class 0x50 as MISSING.

/// One kind-0x38 placement's laser object (the ext block at obj+0xE0 plus obj+0xf4 [M 001aa8b8]).
#[derive(Clone, Debug, PartialEq)]
pub struct SpaceLaser {
    pub placement: usize,
    /// ext+0x20: param 0 as raw f32 bits [M].
    pub a: f32,
    /// ext+0x24: param 1 as raw f32 bits [M].
    pub b: f32,
    /// ext+0x28: param 2 as raw f32 bits [M].
    pub c: f32,
    /// ext+0x30: param 3 as an UNSIGNED u32 → float (the negative arm halves with the sticky bit,
    /// converts, doubles — disasm 0x1aa934-0x1aa968; review 2026-10-21, the branch read it signed).
    pub p3: f32,
    /// ext+0x34: param 4, the same unsigned conversion [disasm 0x1aa970-0x1aa9a4].
    pub p4: f32,
    /// ext+0x40: param 5 low u16 [M].
    pub ch5: u16,
    /// ext+0x42: param 6 low u16 [M].
    pub ch6: u16,
    /// ext+0x44: param 7 low u16 [M].
    pub ch7: u16,
    /// ext+0x38: the 10.0f constant [M 001aa8b8:35].
    pub period: f32,
    /// obj+0xf4: the update state, 0 at creation [M 001aa8b8:38].
    pub state: u16,
}

fn param(params: &[(u32, u32)], k: u32) -> u32 {
    params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

/// `Create_SpaceLaser` 0x1aa8b8's ext writes, from one kind-0x38 placement's params.
pub fn from_placement(placement: usize, params: &[(u32, u32)]) -> SpaceLaser {
    SpaceLaser {
        placement,
        a: f32::from_bits(param(params, 0)),
        b: f32::from_bits(param(params, 1)),
        c: f32::from_bits(param(params, 2)),
        p3: param(params, 3) as f32,
        p4: param(params, 4) as f32,
        ch5: param(params, 5) as u16,
        ch6: param(params, 6) as u16,
        ch7: param(params, 7) as u16,
        period: 10.0,
        state: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 001aa8b8:14-38] the full param map, synthetic placement.
    #[test]
    fn the_param_map_matches_the_creator() {
        let ps: Vec<(u32, u32)> = vec![
            (0, 1.5_f32.to_bits()),
            (1, (-2.0_f32).to_bits()),
            (2, 3.25_f32.to_bits()),
            (3, (-7_i32) as u32),
            (4, 90),
            (5, 19), // the arm channel [D docs/levels/0700001b.md:71]
            (6, 14), // the kill channel (Drake's death) [D]
            (7, 0x1234),
        ];
        let l = from_placement(4, &ps);
        assert_eq!(l.placement, 4);
        assert_eq!((l.a, l.b, l.c), (1.5, -2.0, 3.25));
        assert_eq!((l.p3, l.p4), ((-7_i32) as u32 as f32, 90.0));
        assert_eq!((l.ch5, l.ch6, l.ch7), (19, 14, 0x1234));
        assert_eq!(l.period, 10.0);
        assert_eq!(l.state, 0);
        // Sister (can fail): param 3 is an UNSIGNED u32→float — a signed read would give −7.
        assert!(l.p3 > 4.0e9, "ext+0x30 = (float)(u32)param [disasm 0x1aa950-0x1aa964: srl 1 | and 1, cvt.s.w, add.s ×2]");
        // Missing params read 0 (the game's record always carries them; the port defaults).
        let empty = from_placement(0, &[]);
        assert_eq!((empty.a, empty.p3, empty.ch5), (0.0, 0.0, 0));
    }
}
