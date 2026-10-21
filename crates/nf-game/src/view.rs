//! The view system's shared state and ops — the never-mentioned render-side batch
//! `unported-render-side-1` (whole-ELF scan, master 20874c31). Spec:
//! `docs/systems/view-display.md` (addresses + evidence). No drawing here: these are the
//! data, state and flow the game's gameplay code sets and the draw code consumes.
//!
//! Objects the game masks live at `obj+0xF8` (the u16 display-mask bits, init 0x1F by
//! `control_init_object` 0x134630); the per-frame consumer is `View_CaptureScene` 0x1e6df0
//! (`object_display_mask` 0x30d87c = `1 << viewer+0x28`) filtering through `View_AddObjects`
//! 0x1e62d8:30 and `View_AddForcedObjects` 0x1e5fb8:79.

use nf_formats::ps2float::{add_noguard, mul};

use crate::trig::sinf;

// ---------------------------------------------------------------------------
// The display mask (obj+0xF8)
// ---------------------------------------------------------------------------

/// obj+0xF8 at create: `control_init_object` 0x134630 inits the u16 display mask to 0x1F
/// (all five viewers) [M docs/fields/obj-tag.md +0xF8].
pub const MASK_INIT: u16 = 0x1f;

/// flags2 (obj+0xFA) bit 0x8 — the object has an AnimObject link, so the mask setters
/// propagate to the skin's attachment objects [M disasm 0x1e7ad0-0x1e7ae0].
pub const FLAGS2_ANIM: u16 = 0x8;

/// The mask-carrying view of one object — exactly the state `View_SetViewMask` 0x1e7ac8 /
/// `View_DrawInThisViewOnly` 0x1e7b30 read and write.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjMask {
    /// obj+0xF8, the display-mask bits.
    pub display_mask: u16,
    /// (obj+0xFA & 0x8) != 0 and obj+0xDC non-0: the AnimObject link the propagation rides on.
    pub has_anim_link: bool,
    /// The AnimObject's attachment objects (anim+0xC0 array, count = skin+0x2D) [M
    /// docs/fields/animobject.md +0x94/+0xC0]. The game writes their obj+0xF8 directly —
    /// no recursion through the attachments' own links [M disasm 0x1e7af8-0x1e7b24].
    pub attachments: Vec<ObjMask>,
}

impl ObjMask {
    /// A plain object: mask only, no AnimObject, no attachments.
    pub fn plain(display_mask: u16) -> ObjMask {
        ObjMask { display_mask, has_anim_link: false, attachments: Vec::new() }
    }
}

/// `View_SetViewMask` 0x1e7ac8 [M disasm 0x1e7ac8..0x1e7b24]: `obj+0xF8 = mask & 0xffff`,
/// then — only when the object carries the flags2 0x8 AnimObject link — the same mask to
/// every attachment (`skin+0x2D` count over the `anim+0xC0` array). No recursion.
pub fn set_view_mask(obj: &mut ObjMask, mask: u32) {
    let mask = (mask & 0xffff) as u16;
    obj.display_mask = mask;
    if obj.has_anim_link {
        for a in &mut obj.attachments {
            a.display_mask = mask;
        }
    }
}

/// `View_DrawInThisViewOnly` 0x1e7b30 [M disasm 0x1e7b30..0x1e7b64]: the mask is
/// `1 << (view & 0x1f)` (sllv, then `& 0xffff`), everything else identical to 0x1e7ac8.
pub fn draw_in_this_view_only(obj: &mut ObjMask, view: u32) {
    set_view_mask(obj, 1u32 << (view & 0x1f));
}

/// `View_CaptureScene` 0x1e6df0: the per-viewer draw bit — the global `object_display_mask`
/// (0x30d87c) = `1 << (viewer+0x28 & 0x1f)` [M].
pub fn viewer_display_mask(viewer_index: u32) -> u32 {
    1 << (viewer_index & 0x1f)
}

/// The draw-list filter: `View_AddObjects` 0x1e62d8:30 and `View_AddForcedObjects`
/// 0x1e5fb8:79 submit the object only when `(obj+0xF8 & object_display_mask) != 0` [M].
pub fn passes_display_mask(obj_mask: u16, viewer_bit: u32) -> bool {
    (obj_mask as u32 & viewer_bit) != 0
}

// ---------------------------------------------------------------------------
// View_RotTransScaleMatrix 0x1e4ff0
// ---------------------------------------------------------------------------

/// HALF_PI as the game loads it (`lui 0x3fc9 / ori 0xfdb`, disasm 0x1e501c-0x1e5020) [M].
const HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);

/// The game's EE add (chopped, no guard bit) — the π/2 offsets and the row sums of 0x1e4ff0.
fn ee_add(a: f32, b: f32) -> f32 { add_noguard(a, b, true) }

/// The game's EE subtract (disasm `sub.s` 0x1e50c4/0x1e50ec).
fn ee_sub(a: f32, b: f32) -> f32 { add_noguard(a, -b, true) }

/// `View_RotTransScaleMatrix(angles, trans, scale) -> MATRIX` [M disasm 0x1e4ff0..0x1e5164].
///
/// The decompiler dropped the argument registers (`s0` = angles, `s1` = out, `s2` = scale,
/// `s3` = trans); the disassembly pins every entry (spec table in
/// `docs/systems/view-display.md`). Returns the game's 16-float MATRIX storage: rotation
/// rows at 0..2 / 4..6 / 8..10, the translation row at 12..14; the pad entries 3/7/11/15
/// are NEVER written by the game (it issues exactly 12 `swc1`s — disasm) and the port
/// returns 0 there. The sines are the game's `PS2Sinf3` = `PS2Sinf` per lane (`trig::sinf`,
/// bit-exact [WORKLOG 0026]); every product is an EE-chopped multiply, the π/2 offsets and
/// the row sums are EE adds, in the disassembly's operation order.
///
/// Not `RotMatrix` 0x1e2368 / `RotTransMatrix` 0x1e2478 (the sky's Euler order, no scale —
/// `sky::rot_matrix` ports those): a different composition, verified entry-by-entry against
/// both decompiles.
pub fn rot_trans_scale_matrix(angles: [f32; 3], trans: [f32; 3], scale: [f32; 3]) -> [f32; 16] {
    // PS2Sinf3 call 1 (disasm 0x1e502c-0x1e5044): (a0+π/2, a0, a1+π/2) → sp+0, sp+4, sp+8.
    let c_a0 = sinf(ee_add(angles[0], HALF_PI)); // sp+0x00
    let s_a0 = sinf(angles[0]); // sp+0x04
    let c_a1 = sinf(ee_add(angles[1], HALF_PI)); // sp+0x08
    // PS2Sinf3 call 2 (disasm 0x1e5048-0x1e5060): (a2+π/2, a2, a1) → sp+0xc, sp+0x10, sp+0x14.
    let c_a2 = sinf(ee_add(angles[2], HALF_PI)); // sp+0x0c
    let s_a2 = sinf(angles[2]); // sp+0x10
    let s_a1 = sinf(angles[1]); // sp+0x14

    let (sx, sy, sz) = (scale[0], scale[1], scale[2]);
    let mut m = [0.0f32; 16];

    // disasm 0x1e5064-0x1e5098: f4=c_a2 f3=s_a1 f6=sx f2=s_a2 f0=c_a1 f5=c_a0 f8=s_a0;
    // f1 = −c_a2·s_a1, f10 = c_a1·c_a2, f9 = s_a0·s_a1, f1 = f1·sx.
    let t_c2s1 = mul(c_a2, s_a1);
    let out2 = mul(mul(-t_c2s1, sx), 1.0); // neg.s 0x1e5080 then mul.s f1,f1,f6 0x1e5098
    let f10 = mul(c_a1, c_a2); // 0x1e5088
    let f9 = mul(s_a0, s_a1); // 0x1e5090
    let f11 = mul(c_a0, c_a1); // 0x1e50a0
    let f3 = mul(c_a0, s_a1); // 0x1e50a4
    m[2] = out2; // swc1 0x1e50a8

    let f0 = mul(c_a1, s_a0); // 0x1e50ac
    let f7 = mul(f11, s_a2); // 0x1e50b0
    let f6 = mul(f3, s_a2); // 0x1e50b8
    let f5 = mul(c_a0, c_a2); // 0x1e50bc
    let f10 = mul(f10, sx); // 0x1e50c0
    let f7 = ee_sub(f9, f7); // 0x1e50c4
    let f6 = ee_add(f0, f6); // 0x1e50c8
    m[0] = f10; // swc1 0x1e50cc

    let f0 = mul(f0, s_a2); // 0x1e50d0
    let f9 = mul(f9, s_a2); // 0x1e50d4
    let f4 = mul(c_a2, s_a0); // 0x1e50dc
    let f3 = ee_add(f3, f0); // 0x1e50e0
    let f2 = mul(s_a2, sx); // 0x1e50e4
    let f4 = -f4; // neg.s 0x1e50e8
    let f11 = ee_sub(f11, f9); // 0x1e50ec
    m[1] = f2; // swc1 0x1e50f0

    m[4] = mul(f7, sy); // 0x1e50f8 / swc1 0x1e50fc
    m[6] = mul(f6, sy); // 0x1e5104 / swc1 0x1e5108
    m[5] = mul(f5, sy); // 0x1e5110 / swc1 0x1e5114

    m[8] = mul(f3, sz); // 0x1e511c / swc1 0x1e5120
    m[9] = mul(f4, sz); // 0x1e5128 / swc1 0x1e512c
    m[10] = mul(f11, sz); // 0x1e5138 / swc1 0x1e513c

    // Translation row (disasm 0x1e5140-0x1e5158): out[12..14] = trans[0..2].
    m[12] = trans[0];
    m[13] = trans[1];
    m[14] = trans[2];
    m
}

// ---------------------------------------------------------------------------
// View_DrawGlist 0x1e77f8
// ---------------------------------------------------------------------------

/// The draw record `View_DrawGlist` 0x1e77f8 hands to `psiDrawObjectMatrix` 0x124708: the
/// glist, the matrix built by 0x1e4ff0, and the `param` word (the callers pass obj+0xEC,
/// the frame/wobble word [M docs/fields/obj-tag.md +0xEC]). The renderer consumes this.
#[derive(Clone, Debug, PartialEq)]
pub struct GlistDraw {
    /// The `celglist_tag` to draw (the game passes the pointer; the port the id/hash slot).
    pub glist: u32,
    /// The matrix 0x1e4ff0 built.
    pub matrix: [f32; 16],
    /// The `psiDrawObjectMatrix` param word.
    pub param: u32,
}

/// `View_DrawGlist(glist, pos, angles, scale, param)` [M 001e77f8]: build the matrix, then
/// hand the record to the renderer. Shipped caller: `AnimObjectDraw` 0x1c4490:316 — the
/// laser-sight ray (glist 0x2000131/0x2000133, `scale` = {1, 1, |dir|}, `param` = obj+0xEC).
pub fn draw_glist(glist: u32, pos: [f32; 3], angles: [f32; 3], scale: [f32; 3], param: u32) -> GlistDraw {
    GlistDraw { glist, matrix: rot_trans_scale_matrix(angles, pos, scale), param }
}

// ---------------------------------------------------------------------------
// View_AddSprite 0x1e7848 — the runtime sprite queue
// ---------------------------------------------------------------------------

/// One queue slot: the 0x30-byte runtime sprite record `Font_DrawText` 0x1ee598 stages at
/// 0x34c7a0 and `amemcpy`s in [M 001ee598:199-200].
pub const SPRITE_SLOT_BYTES: usize = 0x30;
/// The flush watermark: `View_AddSprite` flushes when `0x3f − announce <= SprCnt` (u32
/// compare, `sltu`; disasm 0x1e7850-0x1e7868), so at most 0x3f sprites sit queued [M disasm].
pub const SPRITE_QUEUE_MAX: u32 = 0x3f;

/// The sprite queue state of `View_AddSprite` 0x1e7848: the 0x30-byte slots at
/// `SprBuffList` 0x34a790 and the `_SprCnt` counter (gp−0x75c0) [M disasm]. Flush-then-
/// allocate: on overflow the queued sprites draw whole (`psiDrawSprites` 0x120e68 with the
/// count BEFORE the reset) and the new sprite lands in slot 0. The draw side supplies the
/// flush closure; nothing here touches pixels.
pub struct SpriteQueue {
    /// `SprBuffList` 0x34a790: the queued sprites, index = slot.
    pub slots: Vec<[u8; SPRITE_SLOT_BYTES]>,
    /// `_SprCnt`: the number of queued (undrawn) sprites.
    pub count: u32,
}

impl Default for SpriteQueue {
    fn default() -> Self {
        SpriteQueue { slots: Vec::new(), count: 0 }
    }
}

impl SpriteQueue {
    /// `View_AddSprite(announce)` [M disasm 0x1e7848..0x1e78a8]: `announce & 0xffff` sprites
    /// will follow this call. Returns the slot index to fill (the game returns the pointer
    /// into `SprBuffList`). `flush` receives the queue and the count drawn.
    pub fn add(&mut self, announce: u16, flush: &mut dyn FnMut(&SpriteQueue, u32)) -> usize {
        // if 0x3f − announce <= SprCnt (u32/sltu: wraps when announce > 0x3f → always flush)
        if SPRITE_QUEUE_MAX.wrapping_sub(announce as u32) <= self.count {
            flush(self, self.count);
            self.count = 0;
        }
        if (self.count as usize) >= self.slots.len() {
            self.slots.push([0; SPRITE_SLOT_BYTES]);
        }
        let slot = self.count as usize;
        self.count += 1;
        slot
    }
}

// ---------------------------------------------------------------------------
// View_Init 0x1e77f0
// ---------------------------------------------------------------------------

/// `View_Init` 0x1e77f0: 8 bytes, `jr $ra; nop` [M disasm 0x1e77f0]. `ResetMap_GameInit`
/// 0x1d5080:26 calls it between `control_init_object_lists` and `Sprite_InitLists` [M]; the
/// port records the step as this no-op (the port's reset flow keeps its own ordering and
/// does not call it — a no-op call would be dead code).
pub fn view_init() {}

// ---------------------------------------------------------------------------
// Tests — numbers/ids only, each with a sister that can fail.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // [M 0x134630] obj+0xF8 inits to 0x1F; [M 0x1e62d8:30] the filter passes bit 0 of it.
    #[test]
    fn mask_init_passes_viewer_zero() {
        assert_eq!(MASK_INIT, 0x1f);
        assert!(passes_display_mask(MASK_INIT, viewer_display_mask(0)));
        // Sister (can fail): a masked-out viewer bit must NOT pass — 0x1f has only bits 0-4.
        assert!(!passes_display_mask(MASK_INIT, 0x20));
    }

    // [M disasm 0x1e7ac8] set_view_mask: the mask (andi 0xffff) lands on the object and on
    // every attachment when the flags2 0x8 link is set; without the link, only the object.
    #[test]
    fn set_view_mask_writes_object_and_attachments() {
        let mut attached = ObjMask {
            display_mask: MASK_INIT,
            has_anim_link: true,
            attachments: vec![ObjMask::plain(MASK_INIT), ObjMask::plain(MASK_INIT), ObjMask::plain(MASK_INIT)],
        };
        set_view_mask(&mut attached, 0x2);
        assert_eq!(attached.display_mask, 0x2);
        for a in &attached.attachments {
            assert_eq!(a.display_mask, 0x2);
        }
        // Sister (can fail): a 4th attachment would mean the count came from somewhere other
        // than the record itself — the game writes exactly the skin+0x2D array entries.
        assert_eq!(attached.attachments.len(), 3);

        let mut plain = ObjMask::plain(MASK_INIT);
        set_view_mask(&mut plain, 0x2);
        assert_eq!(plain.display_mask, 0x2);
        // Sister: without the anim link the propagation cannot run (the game needs
        // obj+0xFA & 8 AND obj+0xDC != 0 — disasm 0x1e7ad0/0x1e7ae0).
        assert!(!plain.has_anim_link);
    }

    // [M disasm 0x1e7acc + the 0x1e7adc delay slot] the mask argument is andi'd to 0xffff —
    // high garbage never reaches obj+0xF8.
    #[test]
    fn set_view_mask_truncates_to_u16() {
        let mut o = ObjMask::plain(MASK_INIT);
        set_view_mask(&mut o, 0x1_0002);
        assert_eq!(o.display_mask, 0x2);
        // Sister (can fail): 0xffff stays 0xffff.
        set_view_mask(&mut o, 0xffff);
        assert_eq!(o.display_mask, 0xffff);
    }

    // [M disasm 0x1e7b30-0x1e7b3c] draw_in_this_view_only: mask = 1 << (view & 0x1f).
    #[test]
    fn draw_in_this_view_only_shifts_one() {
        let mut o = ObjMask::plain(MASK_INIT);
        draw_in_this_view_only(&mut o, 2);
        assert_eq!(o.display_mask, 0x4);
        // Sister (can fail): view 0 must give bit 0 (not bit 1) — the sllv base is 1.
        draw_in_this_view_only(&mut o, 0);
        assert_eq!(o.display_mask, 0x1);
    }

    // [M disasm 0x1e7b34] the shift input is masked to 5 bits by the hardware — view 32 == view 0.
    #[test]
    fn draw_in_this_view_only_wraps_view_index() {
        let mut o = ObjMask::plain(MASK_INIT);
        draw_in_this_view_only(&mut o, 32);
        assert_eq!(o.display_mask, 0x1);
        // Sister (can fail): view 31 → 1<<31 = 0x80000000 in the sllv, then the disasm's
        // `andi t0, v0, 0xffff` (0x1e7b3c) truncates it to 0 — the object draws in NO view.
        // The port's (mask & 0xffff) models exactly that; a port that kept the full 32-bit
        // shift (or pre-masked the index differently) would assert 0x8000 here and fail.
        let mut o2 = ObjMask::plain(MASK_INIT);
        draw_in_this_view_only(&mut o2, 31);
        assert_eq!(o2.display_mask, 0x0);
        // And the boundary that DOES fit: view 15 → 0x8000.
        let mut o3 = ObjMask::plain(MASK_INIT);
        draw_in_this_view_only(&mut o3, 15);
        assert_eq!(o3.display_mask, 0x8000);
    }

    // [M disasm 0x1e4ff0] zero angles, unit scale: the cos lanes are sinf(0+π/2) and the
    // sin lanes sinf(0) = 0.0 exactly (the poly accumulates +x onto zero products). The
    // structure pins WHICH lane lands WHERE; the cos lanes compare against the same
    // bit-exact sinf call the game's PS2Sinf3 performs.
    #[test]
    fn zero_angles_build_the_scaled_identity() {
        let m = rot_trans_scale_matrix([0.0; 3], [0.0; 3], [1.0, 1.0, 1.0]);
        let c = sinf(ee_add(0.0, HALF_PI));
        assert_eq!(m[0], mul(c, c));
        assert_eq!(m[1], 0.0); // s_a2·sx, sinf(0) = 0 exactly
        assert_eq!(m[2], 0.0); // −c_a2·s_a1·sx, s_a1 = 0
        assert_eq!(m[4], 0.0); // s_a0·s_a1 − c_a0·c_a1·s_a2, all sin lanes 0
        assert_eq!(m[5], mul(c, c));
        assert_eq!(m[6], 0.0); // c_a1·s_a0 + c_a0·s_a1·s_a2
        assert_eq!(m[8], 0.0); // c_a0·s_a1 + c_a1·s_a0·s_a2
        assert_eq!(m[9], 0.0); // −c_a2·s_a0
        assert_eq!(m[10], mul(c, c));
        // Sister (can fail): the off-diagonal entries must be EXACT zeros — a wrong Euler
        // order or a swapped sinf lane leaves a polynomial residue in one of them.
        assert_ne!(c, 0.5);
    }

    // [M disasm 0x1e4ff0] the scale multiplies the rows (the stored transposed form: the
    // input axes) and the translation lands at 12..14.
    #[test]
    fn scale_multiplies_rows_and_translation_lands_in_row3() {
        let m = rot_trans_scale_matrix([0.0; 3], [7.0, 8.0, 9.0], [2.0, 3.0, 4.0]);
        let c = sinf(ee_add(0.0, HALF_PI));
        assert_eq!(m[0], mul(mul(c, c), 2.0));
        assert_eq!(m[5], mul(mul(c, c), 3.0));
        assert_eq!(m[10], mul(mul(c, c), 4.0));
        assert_eq!(m[12], 7.0);
        assert_eq!(m[13], 8.0);
        assert_eq!(m[14], 9.0);
        // Sister (can fail): scale never leaks between rows — row 1's translation-index
        // entries stay 0 and m[1]/m[2] are exact zeros.
        assert_eq!(m[1], 0.0);
        assert_eq!(m[2], 0.0);
    }

    // [M disasm 0x1e4ff0: exactly 12 swc1s] the pad entries 3/7/11/15 are never written by
    // the game; the port returns 0 there.
    #[test]
    fn the_pad_entries_are_never_written() {
        let m = rot_trans_scale_matrix([0.37, -1.2, 2.9], [1.0, 2.0, 3.0], [1.0, 1.0, 1.0]);
        assert_eq!(m[3], 0.0);
        assert_eq!(m[7], 0.0);
        assert_eq!(m[11], 0.0);
        assert_eq!(m[15], 0.0);
        // Sister (can fail): with non-trivial angles the written entries are NOT all zero —
        // a port that zeroed the whole matrix would pass the pads and fail here.
        assert_ne!(m[0], 0.0);
        assert_ne!(m[5], 0.0);
        assert_ne!(m[10], 0.0);
    }

    // [M disasm 0x1e50c4/0x1e50ec] the row-1/row-2 mixed terms at a quarter turn: a0 = π/2
    // puts the s_a0 lanes where the c_a0 lanes were — pins the Euler order against
    // transposition swaps (RotMatrix 0x1e2368 is the OTHER order and would fail this).
    #[test]
    fn quarter_turn_matches_the_game_order() {
        let (a, b) = (HALF_PI, 0.0f32);
        let m = rot_trans_scale_matrix([a, b, b], [0.0; 3], [1.0, 1.0, 1.0]);
        // c_a0 = sinf(π/2+π/2) = sinf(π) lane, s_a0 = sinf(π/2) lane, c_a1 = sinf(π/2),
        // c_a2 = sinf(π/2), s_a1 = s_a2 = 0.
        let c0 = sinf(ee_add(HALF_PI, HALF_PI));
        let s0 = sinf(HALF_PI);
        let c = sinf(ee_add(0.0, HALF_PI));
        assert_eq!(m[0], mul(c, c)); // c_a1·c_a2
        assert_eq!(m[2], 0.0);
        assert_eq!(m[4], mul(s0, 0.0)); // s_a0·s_a1 − c_a0·c_a1·s_a2 = s_a0·0 − 0
        assert_eq!(m[5], mul(c0, c)); // c_a0·c_a2
        assert_eq!(m[6], mul(c, s0)); // c_a1·s_a0 + c_a0·s_a1·s_a2
        assert_eq!(m[9], mul(-c, s0)); // −c_a2·s_a0
        // Sister (can fail): m[10] = c_a0·c_a1 − s_a0·s_a1·s_a2 = c0·c, not c·c — a
        // transposed composition swaps rows 1/2's roles and breaks this entry.
        assert_eq!(m[10], mul(c0, c));
    }

    // [M 0x1e77f8 + 0x1c4490:316] draw_glist produces the record the game hands to
    // psiDrawObjectMatrix: glist and param pass through, the matrix comes from 0x1e4ff0.
    #[test]
    fn draw_glist_passthrough_and_matrix() {
        let d = draw_glist(0x2000131, [7.0, 8.0, 9.0], [0.0; 3], [1.0, 1.0, 1.0], 0x1234);
        assert_eq!(d.glist, 0x2000131);
        assert_eq!(d.param, 0x1234);
        // Sister (can fail): the matrix is 0x1e4ff0's, not an identity stand-in — the
        // translation row must carry pos and the diagonal the cos·cos term.
        assert_eq!(d.matrix[12], 7.0);
        assert_eq!(d.matrix[0], rot_trans_scale_matrix([0.0; 3], [0.0; 3], [1.0; 3])[0]);
    }

    // [M disasm 0x1e7848] flush-then-allocate: the first 0x3f adds never flush; the next
    // add flushes the 0x3f queued sprites, resets, and hands back slot 0.
    #[test]
    fn sprite_queue_flushes_at_the_watermark() {
        let mut q = SpriteQueue::default();
        let mut flushed: Vec<u32> = Vec::new();
        for i in 0..0x3f {
            let slot = q.add(0, &mut |_q, n| flushed.push(n));
            assert_eq!(slot, i as usize);
        }
        assert!(flushed.is_empty());
        assert_eq!(q.count, 0x3f);
        // The 0x40th add: flush (count 0x3f drawn), reset, slot 0.
        let slot = q.add(0, &mut |_q, n| flushed.push(n));
        assert_eq!(flushed, vec![0x3f]);
        assert_eq!(slot, 0);
        assert_eq!(q.count, 1);
        // Sister (can fail): after the flush the count is 1, not 0x40 — the reset precedes
        // the increment (disasm 0x1e787c sw zero, then 0x1e789c sw v1).
        assert_ne!(q.count, 0x40);
    }

    // [M disasm 0x1e784c + 0x1e7854] the announce word is masked to 0xffff and subtracted
    // from 0x3f BEFORE the compare — announce 0x3f flushes even with an empty queue
    // (0x3f − 0x3f = 0 <= 0), announce > 0x3f wraps and always flushes.
    #[test]
    fn sprite_queue_announce_drives_the_flush() {
        let mut flushes = 0;
        let mut q = SpriteQueue::default();
        q.add(0x3f, &mut |_, _| flushes += 1);
        assert_eq!(flushes, 1);
        assert_eq!(q.count, 1);
        // Sister (can fail): announce 0x3e on an empty queue must NOT flush (0x3f−0x3e=1 > 0).
        let mut q2 = SpriteQueue::default();
        q2.add(0x3e, &mut |_, _| flushes += 1);
        assert_eq!(flushes, 1);
        assert_eq!(q2.count, 1);
        // [M disasm 0x1e785c-0x1e7868] announce 0xffff: `subu v0, 0x3f, 0xffff` wraps to
        // 0xFFFF0040 and `sltu SprCnt, v0` is TRUE — the branch SKIPS the flush. A large
        // announce therefore never flushes (the game's callers always pass 0).
        let mut q3 = SpriteQueue::default();
        let mut f3 = 0;
        q3.add(0xffff, &mut |_, _| f3 += 1);
        assert_eq!(f3, 0);
        assert_eq!(q3.count, 1);
        // Sister (can fail): the boundary at count 1 — announce 0x3e now flushes
        // (0x3f−0x3e = 1 <= 1), unlike the same announce on the empty queue above.
        q3.add(0x3e, &mut |_, _| f3 += 1);
        assert_eq!(f3, 1);
        assert_eq!(q3.count, 1);
    }

    // [M disasm 0x1e77f0] View_Init is an 8-byte no-op (`jr $ra; nop` — nothing else).
    #[test]
    fn view_init_is_a_no_op() {
        // The call must compile and return; there is no state to observe.
        view_init();
        // Sister (can fail): the mask constants the module carries stay the game's — an
        // accidental redefinition would break the first assert.
        assert_eq!(MASK_INIT, 0x1f);
        assert_eq!(SPRITE_QUEUE_MAX, 0x3f);
    }
}
