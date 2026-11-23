//! How the driving engine turns a car's damage zones into the `DamageZones` the car microcode
//! deforms the body with (docs/formats/car-model.md §10.5; the vertex side is
//! `nf_formats::carmesh::deformed`). Logic only, nothing is drawn.
//!
//! Sources (DRIVING.ELF, decompile under work/decomp/DRIVING.ELF/) [M]:
//! - The zones themselves are the `PBondCar` words +0x2f0 + 8·i, (a, b) per zone
//!   (`damage::CarDamage::zones`): `FUN_00189598` (the damage virtual, `damage::inflict`) adds to
//!   them, `FUN_001876c8` (vtable +0x1a4, `ESetVisualDamage`) sets them at random inside a range,
//!   `FUN_00187578` (vtable +0x19c, the repair) clears them. Nothing decays them: those three and
//!   the constructor are their only writers [M: every decompiled store to +0x2f0/+0x2f4 of a
//!   `PBondCar`].
//! - `FUN_0018aa78` (vtable 0x379ef0 + 0x24, also `PTank`'s 0x37a9f8 + 0x24) hands out
//!   `car + 0x2f0` and the count 16.
//! - `FUN_001d3138`, run by the vehicle draw `FUN_001d35d8` every frame before the items are
//!   drawn, fills the render object's `EAGL::DamageZones` block (+0xe0, 16 quadwords;
//!   `FUN_001d2588` answers that name with +0xe0) from that array: `(a, b, 3·b, a > 0.75 ? 127 :
//!   0)`, the last as an integer word; all zero when the render object has no owner (+0x24 null).
//! - `FUN_001adea8` (per drawn object) then runs `FUN_001d2b38`, which copies those 0x100 bytes
//!   to `DAT_00424620` because the vehicle constructor `FUN_001d2338` sets bit 28 of +0x18
//!   (`FUN_001d2a50`, the other branch, zeroes them). `FUN_001ada20` registers `DAT_00424620`
//!   with the count 0x10 in the `MatCarPerCar` parameter object, whose template unpacks it to
//!   VU1 0x56 (§10.4).
//! - The level 0x332090 (0.75) is read-only in the ELF [M: every reference is a load].

use nf_formats::carmesh::{self, CarStrip, DamageZone, DeformedStrip, DAMAGE_ZONES};

use super::damage::{CarDamage, MODEL10_LEVEL, MODEL9_LEVEL, STAGES};
use super::effects::DriveRng;

/// The glass-break level, 0x332090 = 0.75: a zone whose `a` is above it uploads `.w` = 127
/// [M `FUN_001d3138`; the same word gates "model call 9" in `FUN_00189598` / `FUN_001876c8`].
pub const GLASS_BREAK_LEVEL: f32 = MODEL9_LEVEL;
/// The `.w` word of a broken zone: `0x7f` stored as an integer (the decompile prints it as the
/// float 1.77965e-43) [M `FUN_001d3138`].
pub const GLASS_BROKEN_ALPHA: u32 = 0x7f;
/// `.z` = this × `b` [M `FUN_001d3138`: `pfVar5[2] = fVar8 * 3.0`].
pub const SPECULAR_DAMAGE_SCALE: f32 = 3.0;

/// `FUN_001d3138` + `FUN_001d2b38`: the 16 `DamageZones` quadwords the car microcode reads for
/// a car whose zones are `zones` (`None` = a render object with no owner: all zero) [M].
pub fn damage_zone_upload(zones: Option<&[[f32; 2]; DAMAGE_ZONES]>) -> [DamageZone; DAMAGE_ZONES] {
    let mut out = [DamageZone::default(); DAMAGE_ZONES];
    if let Some(z) = zones {
        for (o, &[a, b]) in out.iter_mut().zip(z) {
            *o = DamageZone { x: a, y: b, z: b * SPECULAR_DAMAGE_SCALE, w: if GLASS_BREAK_LEVEL < a { GLASS_BROKEN_ALPHA } else { 0 } };
        }
    }
    out
}

/// The zones whose glass shows broken (`.w` = 127), as a 16-bit mask.
pub fn broken_glass_mask(zones: &[[f32; 2]; DAMAGE_ZONES]) -> u16 {
    zones.iter().enumerate().filter(|(_, z)| GLASS_BREAK_LEVEL < z[0]).fold(0, |m, (i, _)| m | 1 << i)
}

/// One strip of `car`'s body as the microcode draws it this frame.
pub fn deform_strip(strip: &CarStrip, car: &CarDamage) -> Option<DeformedStrip> {
    carmesh::deformed(strip, &damage_zone_upload(Some(&car.zones)))
}

/// What `FUN_001876c8` asked the model for.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisualDamage {
    /// `FUN_001a2b50(model, event, mask, now, 2)` calls, in the game's order (0x16 first): event
    /// 0x13 + k with every zone whose max(a, b) ≥ `STAGES[k]`; only non-empty masks.
    pub events: Vec<(u32, u32)>,
    /// The model vtable +0x84 masks with argument 10 (max > 0.35) and 9 (max > 0.75), sent when
    /// non-zero (see [`RenderDamage::model_call`]).
    pub model10: u32,
    pub model9: u32,
}

/// `FUN_001876c8(a_lo, a_hi, b_lo, b_hi, car)`, `PBondCar`/`PTank` vtable +0x1a4 [M]: with
/// `0 ≤ a_hi − a_lo`, `0 ≤ b_hi − b_lo`, `a_hi ≤ 1`, `b_hi ≤ 1` (else nothing, `None`), every
/// zone gets `a = a_lo + r1·(a_hi − a_lo)/65536`, `b = b_lo + r2·(b_hi − b_lo)/65536` with two
/// draws of the driving RNG per zone (zone 0 first). The zones are replaced, not added to.
pub fn set_visual_damage(car: &mut CarDamage, a: (f32, f32), b: (f32, f32), rng: &mut DriveRng) -> Option<VisualDamage> {
    let da = a.1 - a.0;
    let db = b.1 - b.0;
    if !(0.0 <= da && 0.0 <= db && da + a.0 <= 1.0 && db + b.0 <= 1.0) {
        return None;
    }
    let mut stage = [0u32; 4];
    let mut r = VisualDamage::default();
    for i in 0..DAMAGE_ZONES {
        let r1 = rng.bits();
        let r2 = rng.bits();
        let z = &mut car.zones[i];
        z[0] = a.0 + r1 as f32 * da * 1.525_878_9e-5;
        z[1] = b.0 + r2 as f32 * db * 1.525_878_9e-5;
        let m = if z[0] < z[1] { z[1] } else { z[0] };
        let bit = 1u32 << i;
        for (k, s) in stage.iter_mut().enumerate() {
            if STAGES[k] <= m {
                *s |= bit;
            }
        }
        if MODEL10_LEVEL < m {
            r.model10 |= bit;
        }
        if GLASS_BREAK_LEVEL < m {
            r.model9 |= bit;
        }
    }
    for k in (0..4).rev() {
        if stage[k] != 0 {
            r.events.push((0x13 + k as u32, stage[k]));
        }
    }
    Some(r)
}

/// `ESetVisualDamage`'s handler `FUN_00159660`: the four integer parameters are percent
/// (`× 0.01`) and go to [`set_visual_damage`] as (a_lo, a_hi, b_lo, b_hi) [M].
pub fn set_visual_damage_percent(car: &mut CarDamage, p: [i32; 4], rng: &mut DriveRng) -> Option<VisualDamage> {
    let f = |v: i32| v as f32 * 0.01;
    set_visual_damage(car, (f(p[0]), f(p[1])), (f(p[2]), f(p[3])), rng)
}

/// The vehicle render object's damage masks [M `FUN_001caba8`, the generic model call that the
/// vehicle's `FUN_001d3c28` (vtable 0x3885d8 + 0x84) passes 9 and 10 on to].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderDamage {
    /// +0x1c: the last call-10 mask.
    pub level10: u16,
    /// +0x1e: every zone ever sent with call 9 (only grows).
    pub level9: u16,
}

/// What a model call made the render object do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderDamageAction {
    /// Call 10 with zones not in +0x1c: `FUN_001a2b50(model, 0x12, new, now, 2)`.
    Event12(u16),
    /// Call 9 with new zones: `FUN_001ca7a0(obj, +0x1e | mask)`, the item swap of those zones
    /// (not ported here).
    SwapItems(u16),
}

impl RenderDamage {
    /// `FUN_001caba8(obj, call, mask)` for call 10 / 9 [M]. Call 10: `new = mask & !+0x1c`; when
    /// non-zero the event fires and +0x1c = mask (replaced, not or-ed). Call 9: when `mask` has a
    /// bit not in +0x1e, the swap runs with the union and +0x1e |= mask.
    pub fn model_call(&mut self, call: u32, mask: u32) -> Option<RenderDamageAction> {
        let mask = mask as u16;
        match call {
            10 => {
                let new = !self.level10 & mask;
                (new != 0).then(|| {
                    self.level10 = mask;
                    RenderDamageAction::Event12(new)
                })
            }
            9 => (!self.level9 & mask != 0).then(|| {
                self.level9 |= mask;
                RenderDamageAction::SwapItems(self.level9)
            }),
            _ => None,
        }
    }

    /// The repair `FUN_00187578` rebuilds the model through `FUN_001c7f10` → `FUN_001c7658` →
    /// `FUN_001c7940`, which clears +0x1c and +0x1e (and the +0x20 timer) [M]; that last step
    /// runs when the model has an `in  ` item list, which every car file has [D carmesh_disc].
    pub fn repair(&mut self) { *self = RenderDamage::default(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driving::damage::VehicleDamageAttrs;

    #[test]
    fn the_upload_is_a_b_three_b_and_the_glass_word() {
        let mut z = [[0.0f32; 2]; 16];
        z[0] = [0.5, 0.2];
        z[3] = [0.75, 1.0]; // exactly the level: not broken (strict >)
        z[7] = [0.76, 0.0];
        let u = damage_zone_upload(Some(&z));
        assert_eq!(u[0], DamageZone { x: 0.5, y: 0.2, z: 0.2 * 3.0, w: 0 });
        assert_eq!(u[3], DamageZone { x: 0.75, y: 1.0, z: 3.0, w: 0 });
        assert_eq!(u[7], DamageZone { x: 0.76, y: 0.0, z: 0.0, w: 0x7f });
        assert_eq!(broken_glass_mask(&z), 1 << 7);
        // b does not break the glass, only a does.
        z[9] = [0.1, 2.0];
        assert_eq!(damage_zone_upload(Some(&z))[9].w, 0);
        assert!(damage_zone_upload(None).iter().all(|q| *q == DamageZone::default()));
    }

    #[test]
    fn visual_damage_draws_two_per_zone_and_replaces_the_zones() {
        let mut car = CarDamage::new(2, &VehicleDamageAttrs::default());
        car.zones = [[0.9, 0.9]; 16];
        let mut rng = DriveRng { state: 0x1234 };
        let mut want = DriveRng { state: 0x1234 };
        let v = set_visual_damage(&mut car, (0.5, 1.0), (0.0, 0.0), &mut rng).unwrap();
        for i in 0..16 {
            let r1 = want.bits();
            let _r2 = want.bits();
            assert_eq!(car.zones[i], [0.5 + r1 as f32 * 0.5 * 1.525_878_9e-5, 0.0], "zone {i}");
        }
        assert_eq!(rng, want, "32 draws");
        // Every zone is ≥ 0.5 → stages 0.15 and 0.35 fire with all 16; model10 = all.
        assert_eq!(v.model10, 0xffff);
        assert!(v.events.contains(&(0x13, 0xffff)) && v.events.contains(&(0x14, 0xffff)));
        let hi: u32 = (0..16).filter(|&i| car.zones[i][0] > 0.75).fold(0, |m, i| m | 1 << i);
        assert_eq!(v.model9, hi);
        assert_eq!(v.events.first().map(|e| e.0), Some(if (0..16).any(|i| car.zones[i][0] >= 0.85) { 0x16 } else { 0x15 }));
        // A range past 1 or upside down does nothing and draws nothing.
        let before = (car.zones, rng);
        assert_eq!(set_visual_damage(&mut car, (0.5, 1.1), (0.0, 0.0), &mut rng), None);
        assert_eq!(set_visual_damage(&mut car, (0.6, 0.5), (0.0, 0.0), &mut rng), None);
        assert_eq!((car.zones, rng), before);
        // Percent form: (100, 100, 20, 20) → a = 1, b = 0.2 exactly on every zone.
        set_visual_damage_percent(&mut car, [100, 100, 20, 20], &mut rng).unwrap();
        assert!(car.zones.iter().all(|z| *z == [1.0, 20.0 * 0.01]));
        assert_eq!(broken_glass_mask(&car.zones), 0xffff);
    }

    #[test]
    fn model_calls_keep_the_traced_masks() {
        let mut r = RenderDamage::default();
        assert_eq!(r.model_call(10, 0b0110), Some(RenderDamageAction::Event12(0b0110)));
        // Call 10 replaces the mask: a subset fires nothing and keeps 0b0110.
        assert_eq!(r.model_call(10, 0b0010), None);
        assert_eq!(r.level10, 0b0110);
        // A new bit fires with only the new bit, and the mask becomes the call's mask.
        assert_eq!(r.model_call(10, 0b1000), Some(RenderDamageAction::Event12(0b1000)));
        assert_eq!(r.level10, 0b1000);
        assert_eq!(r.model_call(9, 0b0001), Some(RenderDamageAction::SwapItems(0b0001)));
        assert_eq!(r.model_call(9, 0b0001), None);
        assert_eq!(r.model_call(9, 0b0100), Some(RenderDamageAction::SwapItems(0b0101)));
        r.repair();
        assert_eq!(r, RenderDamage::default());
    }
}
