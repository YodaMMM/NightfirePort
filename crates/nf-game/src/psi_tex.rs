//! The psi texture-space pipeline helpers (ACTION.ELF): the pure maths the game runs when it
//! packs loaded map textures into PS2 texture space and builds sprite/GS command packets.
//!
//! The PS2-specific hardware around them has no port equivalent and is cited at its owners:
//!
//! - `psiPreGameRun` 0x11e2e8 → `Reduce` 0x11daf0 (520 B): the VRAM texture-space packer. Walks
//!   every loaded texture (`NumPS2TexturesLoaded`, `TexSize`), skips ones already placed
//!   (record `+0x4 == -1`), claims a free slot in the texture-space table (0x25caf0, 0x400
//!   entries, stride 0x1c, used flag at +9) for the first unplaced texture, then calls
//!   `CheckAndAddTextureToSpace` 0x11db58 for every other unplaced texture of the same height.
//!   The port's textures live in normal GPU memory (`nf_view::scene`), no VRAM packing.
//! - `psiCreateMapTextures` 0x11d200 → `FillTextureDMA` 0x11d110 (240 B): allocates 0x40 bytes
//!   per texture (`Mem_Malloc(0x40, 0x1180, 0)`) and fills it with a 16-word GS/DMA packet
//!   template (words [2]=0x10000000, [3]=0x50000003, [4]=0x8002, [6]=0xe, [10]=6, [0xe]=8).
//!   The port renders decoded `nf_formats::texture::Texture` pixels directly; no DMA chains.
//! - `CreateSpaceAndPackTextures` 0x11dcf8 → `Conv4to32` 0x11e8d8 → `PageConv4to32` 0x11e640
//!   → `BlockConv4to32` 0x11e530 (268 B): converts the disc's 4-bpp swizzled blocks to 32-bit
//!   words through `lut4` before VRAM upload. The port's equivalent is the decode in
//!   `nf_formats::texture` (4 bpp, left pixel in the low nibble, CSM1 palettes) done per pixel.
//! - `psiDrawSprites` 0x120e68 → `FillSpriteData` 0x120cf8 (364 B): fills a 52-word GS packet
//!   template per sprite (GIF tags 0x6000000c/0x5000000c, TEX0 from the texture-space entry,
//!   two vertex blocks coloured by the `SpriteColourR/G/B/A` globals 0x2469d0..). The port
//!   draws sprites through `nf_view::hud` instead; the colour/UV data that packet carried is
//!   what `HudFrame` sprites carry.
//!
//! What IS portable (and tested below) are the four pure helpers the pipeline is built on.

/// `GetBitPos(n)` (ACTION.ELF 0x11f808, 348 B): bit position used to size 64-texel strips.
///
/// [M 0x11f808 decomp + disassembly] Exact-match fast path for powers of two 1..0x400
/// (1→0 … 0x400→10), then a range chain over the rest: 0/negatives→0, 2→1, 3–4→2, 5–8→3,
/// 9–16→4, 17–32→5 (0x20 returns 5 through the chain's tail), 33–64→6, 65–128→7,
/// 129–256→8, 257–512→9, 513–1024→10, and **n > 0x400 → 0** (verified in the disassembly:
/// `movz v0, zero, v1` at 0x11f958 in the `jr ra` delay slot — the C's `if (0x400 < n) → 0`).
/// Net: for 1 ≤ n ≤ 0x400 the result is the bit length of n, except exact powers of two
/// return log2(n); anything else is 0.
pub fn get_bit_pos(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    let n = n as u32;
    if n > 0x400 {
        return 0;
    }
    if n.is_power_of_two() {
        n.trailing_zeros() as i32
    } else {
        (32 - n.leading_zeros()) as i32
    }
}

/// `GetMinWidth(n)` (ACTION.ELF 0x1205c8, 72 B): texture-space width → 64-texel strip count.
///
/// [M 0x1205c8] `n < 0x41 → 1`, else `(1 << GetBitPos(n)) >> 6`. The decomp's
/// `w + 0x3f` before the shift is dead code: the disassembly (`movn v0, v1, a0` at 0x1205f4,
/// with `slt a0, -1, v1` at 0x1205ec) selects `w` over `w + 63` whenever `w ≥ 0`, and
/// `w = 1 << GetBitPos(n)` is always ≥ 0 — so the real formula is `w >> 6`, and
/// **n > 1024 → GetBitPos 0 → 0** (not 1). Values: 1 for n ≤ 64, 2 for 65–128, 4 for
/// 129–256, 8 for 257–512, 16 for 513–1024, 0 above.
pub fn get_min_width(n: i32) -> i32 {
    if n < 0x41 {
        return 1;
    }
    (1i32 << (get_bit_pos(n) as u32 & 0x1f)) >> 6
}

/// `GetMinWidth2(n)` (ACTION.ELF 0x120610, 72 B): byte-identical to `GetMinWidth` except the
/// small-input arm is `n < 0x81 → 2` — pointer/blob sprites always get at least two strips
/// (≥ 128 texels) [M 0x120610].
///
/// Callers [M grep ACTION.ELF decomp]: `CreatePointerSpriteListColour` 0x119910,
/// `CreatePointerSpriteList` 0x11a160, `CreateLoadingBlob` 0x11bf70, `CacheTexture` 0x11cd58,
/// `psiDrawSprites` 0x120e68 — front-end, loading and in-game sprite paths;
/// `GetMinWidth` is called by `psiCreateMapTextures` 0x11d200 (level texture setup).
pub fn get_min_width2(n: i32) -> i32 {
    if n < 0x81 {
        return 2;
    }
    (1i32 << (get_bit_pos(n) as u32 & 0x1f)) >> 6
}

/// `ByteCompare(a, b, n)` (ACTION.ELF 0x1206c0, 64 B): 1 when the first n bytes match.
///
/// [M 0x1206c0 decomp] `if (n <= 0) return 1;` then walks byte by byte and returns 0 on the
/// FIRST mismatch (it does not scan the rest), 1 when all n match. Unlike `memcmp` it only
/// reports equality — there is no ordering — and n ≤ 0 counts as equal. Caller [M grep]:
/// `psiCreateMapTextures` 0x11d200 (comparing texture records while building the map's
/// texture list).
///
/// Port deviation [S]: the game trusts `n ≤ a.len(), b.len()` (raw pointers); with slices we
/// return false instead of reading out of bounds when either is shorter than n. The game's
/// callers always pass exact lengths.
pub fn byte_compare(a: &[u8], b: &[u8], n: usize) -> bool {
    if a.len() < n || b.len() < n {
        return false;
    }
    if n == 0 {
        return true;
    }
    a[..n] == b[..n]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [M 0x11f808] every exact-match arm of the fast path.
    #[test]
    fn get_bit_pos_exact_powers() {
        for (n, want) in [
            (1, 0),
            (2, 1),
            (4, 2),
            (8, 3),
            (0x10, 4),
            (0x20, 5),
            (0x40, 6),
            (0x80, 7),
            (0x100, 8),
            (0x200, 9),
            (0x400, 10),
        ] {
            assert_eq!(get_bit_pos(n), want, "GetBitPos(0x{n:x})");
        }
    }

    /// Sister: the range chain — non-powers return the bit length, including 17..32 → 5
    /// (the chain's tail fall-through), and the boundaries between arms.
    #[test]
    fn get_bit_pos_range_chain() {
        for (n, want) in [
            (0, 0),
            (-5, 0),
            (3, 2),
            (5, 3),
            (7, 3),
            (9, 4),
            (16, 4),
            (17, 5),
            (31, 5),
            (33, 6),
            (63, 6),
            (65, 7),
            (127, 7),
            (129, 8),
            (255, 8),
            (257, 9),
            (511, 9),
            (513, 10),
            (1023, 10),
        ] {
            assert_eq!(get_bit_pos(n), want, "GetBitPos({n})");
        }
    }

    /// Sister: the >0x400 quirk returns 0 (the `movz` at 0x11f958), which is NOT the bit
    /// length — a wrong port (floor log2 / bit length) would return 10 or 11 here.
    #[test]
    fn get_bit_pos_above_0x400_is_zero() {
        assert_eq!(get_bit_pos(0x401), 0);
        assert_eq!(get_bit_pos(0x4000), 0);
        assert_eq!(get_bit_pos(i32::MAX), 0);
    }

    /// [M 0x1205c8] the strip-count table.
    #[test]
    fn get_min_width_table() {
        for (n, want) in [
            (0, 1),
            (64, 1),
            (65, 2),
            (128, 2),
            (129, 4),
            (256, 4),
            (257, 8),
            (512, 8),
            (513, 16),
            (1024, 16),
        ] {
            assert_eq!(get_min_width(n), want, "GetMinWidth({n})");
        }
    }

    /// Sister: the dead `+0x3f` — a port that computed `(w + 63) >> 6` would answer 1 for
    /// n > 1024 (GetBitPos 0 → (1+63)>>6), the real path answers 0.
    #[test]
    fn get_min_width_above_0x400_is_zero() {
        assert_eq!(get_min_width(1025), 0);
        assert_eq!(get_min_width(0x10000), 0);
    }

    /// [M 0x120610] the wider small-input arm: minimum 2, threshold 129.
    #[test]
    fn get_min_width2_table() {
        for (n, want) in [
            (0, 2),
            (64, 2),
            (128, 2),
            (129, 4),
            (256, 4),
            (257, 8),
            (1024, 16),
            (1025, 0),
        ] {
            assert_eq!(get_min_width2(n), want, "GetMinWidth2({n})");
        }
    }

    /// Sister: GetMinWidth2 differs from GetMinWidth only below 129 — a port that used the
    /// wrong threshold answers 1 at n = 128.
    #[test]
    fn get_min_width2_below_129_is_two() {
        assert_eq!(get_min_width2(65), 2);
        assert_eq!(get_min_width(65), 2);
        assert_eq!(get_min_width2(0x40), 2);
    }

    /// [M 0x1206c0] first-mismatch semantics: equal → true, first differing byte → false
    /// (later bytes never inspected), n = 0 → true.
    #[test]
    fn byte_compare_first_mismatch() {
        assert!(byte_compare(b"abcdef", b"abcdef", 6));
        assert!(!byte_compare(b"abcdef", b"abcxef", 6));
        // The first mismatch decides even when the tails differ wildly.
        assert!(!byte_compare(b"ab", b"ax", 2));
        assert!(byte_compare(b"ab", b"ax", 1));
        assert!(byte_compare(b"ab", b"ax", 0));
    }

    /// Sister: a port that fell back to full-slice equality would wrongly answer false for
    /// a zero-length n with different tails.
    #[test]
    fn byte_compare_n_zero_ignores_contents() {
        assert!(byte_compare(b"one", b"two", 0));
        // And the port-safety guard: n beyond a slice is false, not a panic.
        assert!(!byte_compare(b"ab", b"abc", 3));
    }
}
