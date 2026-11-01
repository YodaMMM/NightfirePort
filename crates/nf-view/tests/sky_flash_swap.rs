//! The sky's lightning swap on the draw side (`scene::sky_variant_drawn`, used by the renderer's draw
//! filter): `View_DrawSky` 0x1e5ad0 loads a slot's normal pair (+0x10 glist / +0x18 object) [M :97-98]
//! and replaces it with the flash pair (+0x14 / +0x1c) on a flash tick [M :122-157]; exactly one variant
//! draws per tick, and a flash-only slot draws nothing between flashes. Pure test: no NF_ISO needed.

use nf_view::scene::sky_variant_drawn;

#[test]
fn one_variant_per_slot_per_tick() {
    // Untagged draws (slots without a flash variant, every non-sky draw) always pass.
    assert!(sky_variant_drawn(None, 0));
    assert!(sky_variant_drawn(None, u32::MAX));
    // Slot 3 dark: its normal variant draws, its flash variant does not.
    assert!(sky_variant_drawn(Some((3, false)), 0));
    assert!(!sky_variant_drawn(Some((3, true)), 0));
    // Slot 3 flashing: the swap.
    assert!(sky_variant_drawn(Some((3, true)), 1 << 3));
    assert!(!sky_variant_drawn(Some((3, false)), 1 << 3));
    // Sister: another slot's flash leaves slot 3 alone (the bit is per slot).
    assert!(sky_variant_drawn(Some((3, false)), 1 << 4));
    assert!(!sky_variant_drawn(Some((3, true)), 1 << 4));
    // The game's 0x15 slots fit the mask; an out-of-range slot never reads a flash bit.
    assert!(sky_variant_drawn(Some((20, true)), 1 << 20));
    assert!(sky_variant_drawn(Some((40, false)), u32::MAX) && !sky_variant_drawn(Some((40, true)), u32::MAX));
}
