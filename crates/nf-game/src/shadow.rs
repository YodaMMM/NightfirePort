//! `Player_InShadow` 0x193498 (360 bytes) — the player's body-shade accumulator [M].
//!
//! Every `Player_Update` tick ends with this call ([M `Player_Update` 0x193600:321, the last
//! statement]); the result lands in `BLData+0x914` (bldata.md:159), which has no reader inside
//! ACTION.ELF — the consumer is the render side. The port keeps the computation as a pure
//! function over the same inputs so the value and the shade rule stay pinned:
//!
//! 1. base = the average of the body colour bytes `obj+0x100..0x102` [M 0x193498:58-59].
//! 2. `Lights_CalcClosestLights(obj+0x80 world sphere, *(u16*)(obj+0xF0) mask)` → the light list
//!    [M :60-61] — the port: `light.rs::LightList::closest` (the 0x1cb2e0 port).
//! 3. per light: `acc = (acc + ((r+g+b)>>3) * (d / radius)) & 0xffff`, with r/g/b the light's
//!    colour bytes at light+0x44/0x45/0x46, d = `Vec_Dist3D(light+0x10, obj+0x80)` and radius the
//!    f32 at light+0x3C [M :62-74].
//! 4. `BL+0x914 = acc * (1/255)` [M :75]; **halved** when the per-class scratch `*(u16*)(obj+0xF6)`
//!    == 4 [M :76-78] (obj-tag.md:39; the shade-2 state).

use crate::light::ResolvedLights;

/// One candidate light row the accumulator folds in (the fields `Player_InShadow` reads).
#[derive(Clone, Copy, Debug)]
pub struct ShadowLight {
    /// The light's world position (light+0x10, the 3 floats before the pad).
    pub pos: [f32; 3],
    /// The light's colour bytes (light+0x44/0x45/0x46).
    pub rgb: [u8; 3],
    /// The f32 at light+0x3C — the distance divisor (the light's radius).
    pub radius: f32,
}

/// `Player_InShadow` 0x193498 over an explicit light set. `centre` = Bond's world-sphere centre
/// (obj+0x80, the point each light's distance is measured from [M :69]). Returns the BL+0x914
/// shade (acc/255, halved when `class_scratch == 4`).
///
/// `d / radius` and the `* (1/255)` are the game's single-precision ops; the accumulator itself is
/// integer arithmetic (u16 wrap), which is where the pinned behaviour lives.
pub fn in_shadow(colour: [u8; 3], centre: [f32; 3], lights: &[ShadowLight], class_scratch: u16) -> f32 {
    // base = (c0 + c1 + c2) / 3 [M 0x193498:58-59 — the three colour bytes summed then /3].
    let base = ((colour[0] as u32 + colour[1] as u32 + colour[2] as u32) / 3) as u64;
    let mut acc: u64 = base;
    for l in lights {
        let d = dist3([l.pos[0] - centre[0], l.pos[1] - centre[1], l.pos[2] - centre[2]]);
        // The game's `mean * (d / radius)` is an int·float product converted back to int on the
        // add (the decompile's mixed arithmetic at :62-74); the u16 wrap is exact.
        let add = ((l.rgb[0] as u32 + l.rgb[1] as u32 + l.rgb[2] as u32) >> 3) as f32
            * (d / l.radius);
        acc = (acc + add as u64) & 0xffff; // [M :74 — the u16 wrap]
    }
    let mut shade = (acc as f32) * 0.003921569; // [M :75 — acc * (1/255)]
    if class_scratch == 4 {
        shade *= 0.5; // [M :76-78 — the halve in shade state 4]
    }
    shade
}

/// `Vec_Dist3D` 0x1e2e48 — returns the float `SQRT(dx²+dy²+dz²)` [M 0x1e2e48]; no
/// quantisation (the caller's `(float)` cast in 0x193498:69 is the decompiler's artefact).
fn dist3(p: [f32; 3]) -> f32 {
    (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
}

/// Build the shadow-light rows from the port's light list: `indices` select the accepted lights
/// (what `Lights_CalcClosestLights` returns; the port's `ResolvedLights::closest` gives
/// (index, falloff) pairs over the resolved set — pass its indices here for the static lights).
/// The byte colours and the +0x3C radius are read raw, exactly what the game's loop reads.
pub fn in_shadow_from_list(
    colour: [u8; 3],
    centre: [f32; 3],
    list: &crate::light::LightList,
    indices: &[usize],
    class_scratch: u16,
) -> f32 {
    let rows: Vec<ShadowLight> = indices
        .iter()
        .map(|&i| ShadowLight {
            pos: list.lights[i].pos,
            rgb: list.lights[i].rgb,
            radius: list.lights[i].radius,
        })
        .collect();
    in_shadow(colour, centre, &rows, class_scratch)
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 0x193498:58-59] base = mean of the three colour bytes.
    #[test]
    fn base_is_the_mean_colour_byte() {
        // No lights: acc stays at the base → shade = base/255.
        let s = in_shadow([10, 20, 30], [0.0; 3], &[], 0);
        assert!((s - (20.0 * 0.003921569)).abs() < 1e-6, "s={s}");
    }

    // [M 0x193498:62-74] one light at d == radius adds ((r+g+b)>>3) * 1.
    #[test]
    fn light_at_radius_distance_adds_mean_quarter() {
        // rgb (200,100,50) → sum 350 >> 3 = 43; d == radius → the whole 43 lands in acc.
        let l = ShadowLight { pos: [30.0, 0.0, 0.0], rgb: [200, 100, 50], radius: 30.0 };
        let s = in_shadow([0, 0, 0], [0.0; 3], &[l], 0);
        let acc = 43u64 & 0xffff;
        assert!((s - (acc as f32 * 0.003921569)).abs() < 1e-4, "s={s}");
        // sister: the distance is measured from Bond's sphere centre [M :69 Vec_Dist3D(L+0x10,
        // obj+0x80)] — the same light at the centre adds nothing.
        assert_eq!(in_shadow([0, 0, 0], [30.0, 0.0, 0.0], &[l], 0), 0.0);
    }

    // [M 0x193498:74] the accumulator wraps at 16 bits: two additions of 0x8000 land back at 0.
    #[test]
    fn accumulator_wraps_at_16_bits() {
        // rgb (170,171,171) sums 512, >>3 = 64; d/radius = 512/1 → each light adds 64·512 = 0x8000.
        let l = ShadowLight { pos: [512.0, 0.0, 0.0], rgb: [170, 171, 171], radius: 1.0 };
        let s = in_shadow([0, 0, 0], [0.0; 3], &[l, l], 0);
        assert_eq!(s, 0.0, "two 0x8000 additions must wrap to acc 0");
    }

    // [M 0x193498:76-78] class_scratch == 4 halves the shade.
    #[test]
    fn scratch_state_4_halves_the_shade() {
        let a = in_shadow([10, 20, 30], [0.0; 3], &[], 0);
        let b = in_shadow([10, 20, 30], [0.0; 3], &[], 4);
        assert!((b - a * 0.5).abs() < 1e-6, "a={a} b={b}");
    }
}
