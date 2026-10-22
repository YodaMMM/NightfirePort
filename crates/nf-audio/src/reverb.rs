//! The SPU2 reverb, register-exact (GAPS G-032): the per-mode effect preset tables the game's
//! `PS2_SetupReverb` writes, the room environment depth chain (`SFXSetEnvironment` /
//! `SFXUpdateEnvironment` / `psiSetReverb`), and the reverb DSP those registers run.
//!
//! Evidence:
//! - `PS2_SetupReverb` (SFX.IRX 0xc944) calls `sceSdSetEffectAttr(core, {mode: 4, depth 0/0, delay 30,
//!   feedback 200})` on both cores, `sceSdSetCoreAttr(core | SD_CORE_EFFECT_ENABLE, 1)` and
//!   `sceSdSetAddr(core | SD_A_EEA_HI, EEA−1)` with EEA 0x200000 (core 0) / 0x1E0000 (core 1)
//!   [M SFX.IRX decomp]. libsd ignores delay/feedback for non-ECHO/DELAY modes (resets them to 0),
//!   so mode 4 runs the plain preset [M ps2sdk libsd effect.c].
//! - `psiSetReverb` (0xc840) writes the scaled depth to EVOLL/EVOLR of both cores: entries
//!   0xb80/0xc80 (core 0) and 0xb81/0xc81 (core 1) — libsd `ParamRegList[0xb/0xc]` = `SD_P_EVOLL/EVOLR`
//!   [M ps2sdk libsd freesd.c]. The per-voice reverb sends are the VMIXEL/VMIXER wet-mix enables
//!   (switch entries 0x1900/0x1b00 + core bit) written from the `PS2_VoiceSetReverb` (0xebcc) bitmask
//!   in `PS2_SendKeyONKeyOff` (0xd544) [M].
//! - `SFXSetEnvironment` (0xdb4) / `SFXUpdateEnvironment` (0xe60) / `SFXResetEnvironment` (0xd68):
//!   the gp-relative globals SFXNewReverb/SFXReverb (depth target/current) and
//!   SFXNewIndoorsVolume/SFXIndoorsVolume (wet-percent target/current), ramped +16/−4 and ±4 per call
//!   [M sound.md §5.5]. Depth value −26472 (−0x6768) is the underwater kill: flag + depth 0.
//! - Register map: the 17-bit delay registers live in the core's R block at 0x2E0 + core*0x400
//!   (SAME_L_DST 0x2EC … APF2_R_DST 0x338, ESA 0x2E0, EEA 0x33C), volumes in the P block at
//!   0x760 + core*40 (IIR_VOL 0x774 … APF2_VOL 0x782, IN_COEF_L/R 0x784/6) [M PCSX2 SPU2 regs.h;
//!   ps2sdk spu2regs.h]. Sony libsd stores each 17-bit delay as two halfwords (val>>14, val<<2);
//!   PCSX2's V_Reverb reassembles them into u32 fields (V_Reverb in defs.h) — the DSP's sample
//!   offsets are the preset value × 4. The preset u16s themselves are what this module stores.
//! - Preset tables + the `sceSdSetEffectAttr` write sequence: ps2sdk libsd `effect.c` (TyRaNiD /
//!   Lukasz Bruun), a register-compatible reimplementation of Sony's closed libsd whose per-mode
//!   tables are the Sony presets; cross-checked against PCSX2 SPU2 (`Reverb.cpp` DoReverb +
//!   `defs.h` V_Reverb + `RegTable.cpp`), whose equations the DSP below follows directly.
//! - **Register signedness**: every volume register is SIGNED s16 — PCSX2 `defs.h` `struct V_Reverb`
//!   declares `s16 IN_COEF_L/R`, `s16 APF1_VOL/APF2_VOL`, `s16 IIR_VOL/WALL_VOL`,
//!   `s16 COMB1_VOL..COMB4_VOL`, and `DoReverb` multiplies them through `#define MUL(x, y)
//!   ((x) * (y) >> 15)` on s32 (arithmetic shift, floors). STUDIO_3's comb2 0xBCE0, comb4 0xBEF0,
//!   wall 0xA680 and IN_COEF 0x8000 are therefore NEGATIVE gains (−0.525/−0.508/−0.699/−1.0); the
//!   preset halfwords stay raw u16 and are sign-reinterpreted at use (`s16()` below).
//! - **Resampling**: PCSX2 `ReverbResample.cpp` — `ReverbDownsample`/`ReverbUpsample` are 39-tap
//!   FIRs over `RevbDownBuf[2][64]` / `RevbUpBuf[2][64]` rings (`filter_down_coefs`, up = 2× down
//!   clamped to s16 so the centre 16384→32767). `DoReverb` runs once per SPU output sample
//!   (`spu2sys.cpp` TimeUpdate: `Cycles++` … `spu2Mix()` → `Mixer.cpp` `MixCore` → `DoReverb`),
//!   `bool R = Cycles & 1` picks the side, the wet `out` lands at `RevbUpBuf[R][pos]` with
//!   `RevbUpBuf[!R][pos] = 0` (zero-stuffed), and `return ReverbUpsample(*this)` FIRs BOTH side
//!   rings every call — the up FIR's odd-index centre tap (2×16384→32767) reads the opposite
//!   parity of the zero-stuffed ring, i.e. the other side's 19-ticks-ago wet sample, so both
//!   channels return wet every tick (no zero gaps). Write-backs and `out` go through
//!   `clamp_mix` (s16 clamp) [PCSX2 Reverb.cpp L109-120].
//! - Mode names disagree between ps2sdk sources: `libsd-common.h` calls mode 9 PIPE, `effect.c`'s
//!   comment calls it CLEAR. Both call mode 4 STUDIO_3 (what the game uses). Recorded, not resolved.

/// The effect-area sizes per mode (`EffectSizes` in ps2sdk effect.c), in 16-sample units.
pub const EFFECT_SIZES: [u16; 10] = [0x1, 0x26C, 0x1F4, 0x484, 0x6FE, 0xADE, 0xF6C, 0x1804, 0x1804, 0x3C0];

/// The per-mode preset tables (`EffectParams[10][16]` in ps2sdk effect.c), u32 rows that unpack to
/// the 32 register halfwords little-endian (the IRX memcpy's them onto the stack as u16s, MIPS LE).
pub const EFFECT_PRESETS: [[u32; 16]; 10] = [
    // OFF
    [0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00010001, 0x00000000, 0x00000000,
     0x00000000, 0x00010001, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000],
    // ROOM
    [0x005B007D, 0x54B86D80, 0x0000BED0, 0xBA800000, 0x53005800, 0x033304D6, 0x022703F0, 0x01EF0374,
     0x01B70336, 0x01B60335, 0x01B50334, 0x01B50334, 0x01B50334, 0x013601B4, 0x005C00B8, 0x80008000],
    // STUDIO_1
    [0x00250033, 0x4FA870F0, 0x4410BCE0, 0x9C00C0F0, 0x4EC05280, 0x031B03E4, 0x02AF03A4, 0x02660372,
     0x025D031C, 0x018E025C, 0x0135022F, 0x00B701D2, 0x00B5018F, 0x008000B4, 0x0026004C, 0x80008000],
    // STUDIO_2
    [0x007F00B1, 0x4FA870F0, 0x4510BCE0, 0xB4C0BEF0, 0x4EC05280, 0x076B0904, 0x065F0824, 0x061607A2,
     0x05ED076C, 0x042E05EC, 0x0305050F, 0x02B70462, 0x0265042F, 0x01B20264, 0x00800100, 0x80008000],
    // STUDIO_3 — the mode the game writes [M PS2_SetupReverb]
    [0x00A900E3, 0x4FA86F60, 0x4510BCE0, 0xA680BEF0, 0x52C05680, 0x0B580DFB, 0x0A3C0D09, 0x09730BD9,
     0x08DA0B59, 0x05E908D9, 0x04B007EC, 0x03D206EF, 0x031D05EA, 0x0238031C, 0x00AA0154, 0x80008000],
    // HALL
    [0x013901A5, 0x50006000, 0xB8004C00, 0xC000BC00, 0x5C006000, 0x11BB15BA, 0x10BD14C2, 0x0DC111BC,
     0x0DC311C0, 0x09C10DC0, 0x07C10BC4, 0x06CD0A00, 0x05C109C2, 0x041A05C0, 0x013A0274, 0x80008000],
    // SPACE
    [0x0231033D, 0x50007E00, 0xB000B400, 0xB0004C00, 0x54006000, 0x1A311ED6, 0x183B1D14, 0x16B21BC2,
     0x15EF1A32, 0x105515EE, 0x0F2D1334, 0x0C5D11F6, 0x0AE11056, 0x07A20AE0, 0x02320464, 0x80008000],
    // ECHO
    [0x00030003, 0x7FFF7FFF, 0x00000000, 0x81000000, 0x00000000, 0x0FFD1FFD, 0x00091009, 0x00000000,
     0x00091009, 0x1FFF1FFF, 0x1FFE1FFE, 0x1FFE1FFE, 0x1FFE1FFE, 0x10041008, 0x00040008, 0x80008000],
    // DELAY
    [0x00030003, 0x7FFF7FFF, 0x00000000, 0x00000000, 0x00000000, 0x0FFD1FFD, 0x00091009, 0x00000000,
     0x00091009, 0x1FFF1FFF, 0x1FFE1FFE, 0x1FFE1FFE, 0x1FFE1FFE, 0x10041008, 0x00040008, 0x80008000],
    // PIPE (effect.c's comment calls this row CLEAR)
    [0x00130017, 0x4FA870F0, 0x4510BCE0, 0x8500BEF0, 0x54C05F80, 0x02AF0371, 0x01DF02E5, 0x01D702B0,
     0x026A0358, 0x011E01D6, 0x00B1012D, 0x0059011F, 0x00E301A0, 0x00400058, 0x00140028, 0x80008000],
];

/// The 32 reverb registers one core runs: the 12 volume/size halfwords (u16, 1.15 fixed point) and
/// the 20 delay registers (u16 in the preset's own units; the DSP scales them ×4 — the hardware's
/// 17-bit form stores val<<2). Named per PCSX2's V_Reverb; ordered by hardware address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReverbRegs {
    pub apf1_size: u16,
    pub apf2_size: u16,
    pub iir_vol: u16,
    pub comb1_vol: u16,
    pub comb2_vol: u16,
    pub comb3_vol: u16,
    pub comb4_vol: u16,
    pub wall_vol: u16,
    pub apf1_vol: u16,
    pub apf2_vol: u16,
    pub same_l_dst: u16,
    pub same_r_dst: u16,
    pub comb1_l_src: u16,
    pub comb1_r_src: u16,
    pub comb2_l_src: u16,
    pub comb2_r_src: u16,
    pub same_l_src: u16,
    pub same_r_src: u16,
    pub diff_l_dst: u16,
    pub diff_r_dst: u16,
    pub comb3_l_src: u16,
    pub comb3_r_src: u16,
    pub comb4_l_src: u16,
    pub comb4_r_src: u16,
    pub diff_l_src: u16,
    pub diff_r_src: u16,
    pub apf1_l_dst: u16,
    pub apf1_r_dst: u16,
    pub apf2_l_dst: u16,
    pub apf2_r_dst: u16,
    pub in_coef_l: u16,
    pub in_coef_r: u16,
}

impl ReverbRegs {
    /// Unpacks `EFFECT_PRESETS[mode]` into the register set. ps2sdk memcpy's the 64 bytes as u16s
    /// (each u32 row splits little-endian into two registers), then `SetEffectData` writes them in
    /// hardware order: volumes raw, delays through `SetEffectRegister`'s 17-bit (val>>14, val<<2)
    /// halfword pairs. This returns the preset's own u16 values — the documented register set.
    pub fn from_mode(mode: usize) -> ReverbRegs {
        let rows = EFFECT_PRESETS[mode];
        let mut w = [0u16; 32];
        for (i, row) in rows.iter().enumerate() {
            w[i * 2] = (*row & 0xFFFF) as u16;
            w[i * 2 + 1] = (*row >> 16) as u16;
        }
        ReverbRegs {
            apf1_size: w[0], apf2_size: w[1], iir_vol: w[2], comb1_vol: w[3], comb2_vol: w[4],
            comb3_vol: w[5], comb4_vol: w[6], wall_vol: w[7], apf1_vol: w[8], apf2_vol: w[9],
            // The 20 delays, in ps2sdk's write order = hardware address order 0x2EC..0x338:
            same_l_dst: w[10], same_r_dst: w[11],
            comb1_l_src: w[12], comb1_r_src: w[13], comb2_l_src: w[14], comb2_r_src: w[15],
            same_l_src: w[16], same_r_src: w[17],
            diff_l_dst: w[18], diff_r_dst: w[19],
            comb3_l_src: w[20], comb3_r_src: w[21], comb4_l_src: w[22], comb4_r_src: w[23],
            diff_l_src: w[24], diff_r_src: w[25],
            apf1_l_dst: w[26], apf1_r_dst: w[27], apf2_l_dst: w[28], apf2_r_dst: w[29],
            in_coef_l: w[30], in_coef_r: w[31],
        }
    }
}

/// What `PS2_SetupReverb` writes per core [M SFX.IRX 0xc944; ps2sdk effect.c].
pub const GAME_EFFECT_MODE: usize = 4; // STUDIO_3
/// `sceSdSetAddr(core | 0x1d00, EEA−1)`: (0x800 − core*0x80) * 0x400 − 1 [M SFX.IRX 0xc944].
pub const GAME_EEA: [u32; 2] = [0x1FFFFF, 0x1DFFFF];
/// Effects-enable bit in the core attr register (`SD_ENABLE_EFFECTS`; libsd's
/// `sceSdSetCoreAttr(core | SD_CORE_EFFECT_ENABLE, 1)` sets it — CoreAttrShifts[0] = 7).
pub const SD_ENABLE_EFFECTS: u16 = 1 << 7;
/// The wet-mix enable switches (`sceSdSetSwitch` in `PS2_SendKeyONKeyOff`): entry = core | switch.
pub const SD_SWITCH_VMIXEL: [u16; 2] = [0x1900, 0x1901];
pub const SD_SWITCH_VMIXER: [u16; 2] = [0x1B00, 0x1B01];

/// The effect start address libsd computes for a mode: EEA − (size*16 − 2) [M ps2sdk effect.c].
pub fn effect_esa(core: usize, mode: usize) -> u32 {
    GAME_EEA[core] - (((EFFECT_SIZES[mode] as u32) << 4) - 2)
}

/// The reverb resampler's FIR taps (PCSX2 `ReverbResample.cpp`: `NUM_TAPS = 39`,
/// `filter_down_coefs`; `filter_up_coefs[i] = clamp(filter_down_coefs[i] * 2, s16)` — only the
/// centre 16384×2 = 32768 clamps to 32767). Odd indices are 0 except the centre; the FIRs run over
/// 64-entry per-side rings with wraparound.
pub const REVERB_FIR_DOWN: [i32; 39] = [
    -1, 0, 2, 0, -10, 0, 35, 0, -103, 0, 266, 0, -616, 0, 1332, 0, -2960, 0, 10246, 16384, 10246,
    0, -2960, 0, 1332, 0, -616, 0, 266, 0, -103, 0, 35, 0, -10, 0, 2, 0, -1,
];
/// `make_up_coefs()` in ReverbResample.cpp: down × 2, clamped to i16 (16384 → 32767).
pub const REVERB_FIR_UP: [i32; 39] = [
    -2, 0, 4, 0, -20, 0, 70, 0, -206, 0, 532, 0, -1232, 0, 2664, 0, -5920, 0, 20492, 32767, 20492,
    0, -5920, 0, 2664, 0, -1232, 0, 532, 0, -206, 0, 70, 0, -20, 0, 4, 0, -2,
];

/// The volume registers are signed s16 in hardware (PCSX2 `defs.h` `struct V_Reverb`; the preset
/// halfwords are raw u16 and get sign-reinterpreted at every use in the DSP).
#[inline]
pub fn s16(v: u16) -> i32 {
    v as i16 as i32
}

/// `clamp_mix` (PCSX2 SPU2): clamp an s32 intermediate into the s16 range.
#[inline]
fn clamp16(x: i32) -> i32 {
    x.clamp(i16::MIN as i32, i16::MAX as i32)
}

/// The effect-area size in samples for the game's setup (the DSP delay-RAM length).
pub fn game_area(core: usize) -> usize {
    (GAME_EEA[core] - effect_esa(core, GAME_EFFECT_MODE) + 1) as usize
}

/// `psiSetReverb` (SFX.IRX 0xc840): the EVOL value written to all four EVOL registers for a room
/// depth 0..100 — the exact 32-bit integer chain `(((depth * 0x50) / 100) * 0x3fff) / 100`.
pub fn psi_set_reverb_evol(depth: i32) -> u16 {
    ((((depth * 0x50) / 100) * 0x3FFF) / 100) as u16
}

/// The environment state (the driver's gp-relative globals): the room depth target/current pair and
/// the indoors wet-percent target/current pair, set by `SFXSetEnvironment` (0xdb4) and ramped by
/// `SFXUpdateEnvironment` (0xe60) each `SFXUpdate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SfxEnvironment {
    /// SFXNewReverb (gp−0x7fa8) / SFXReverb (gp−0x7fa4).
    pub depth_target: i32,
    pub depth: i32,
    /// SFXNewIndoorsVolume (gp−0x7fa0) / SFXIndoorsVolume (gp−0x7f9c): 100 outdoors, 50 indoors.
    pub wet_target: i32,
    pub wet: i32,
    /// SFXUnderWater (gp−0x7f98): set by the magic depth −26472; `ES_CombineVolumes` (0x1300) halves
    /// the dry volume of non-underwater samples while set.
    pub underwater: bool,
}

impl Default for SfxEnvironment {
    /// `SFXResetEnvironment` (0xd68): depth 0, wet 100.
    fn default() -> SfxEnvironment {
        SfxEnvironment { depth_target: 0, depth: 0, wet_target: 100, wet: 100, underwater: false }
    }
}

impl SfxEnvironment {
    /// `SFXSetEnvironment` (0xdb4): `depth` clamps 0..100 (or the −26472 underwater kill),
    /// `indoors` picks the wet-percent target (50 when true, 100 otherwise).
    pub fn set(&mut self, depth: i32, indoors: bool) {
        if depth == -0x6768 {
            self.underwater = true;
            self.depth_target = 0;
        } else {
            self.underwater = false;
            self.depth_target = depth.clamp(0, 100);
        }
        self.wet_target = if indoors { 0x32 } else { 100 };
    }

    /// `SFXUpdateEnvironment` (0xe60): SFXReverb moves toward its target by at most +16/−4 (then
    /// `psiSetReverb` writes it), SFXIndoorsVolume by at most ±4. Returns the EVOL value written.
    pub fn update(&mut self) -> u16 {
        let mut d = self.depth_target - self.depth;
        if d > 0x10 { d = 0x10; }
        if d < -4 { d = -4; }
        self.depth += d;
        let mut w = self.wet_target - self.wet;
        if w > 4 { w = 4; }
        if w < -4 { w = -4; }
        self.wet += w;
        psi_set_reverb_evol(self.depth)
    }
}

/// The reverb DSP: the equations SPU2's reverb runs each tick (PCSX2 `V_Core::DoReverb`, "pretty
/// much directly ripped from" drhell's PS1 SPU page), plus the core's own resampling
/// (`ReverbResample.cpp`): one shared delay RAM per core (the effect area); the left and right
/// paths alternate per tick (`bool R = Cycles & 1`), reading/writing the same RAM through per-side
/// offsets. All volume registers apply as s16 (`s16()`). MUL is `x * y >> 15` on s32 (1.15 fixed
/// point, arithmetic shift). The dry stereo input goes through the 39-tap downsample FIR into the
/// DSP, and the wet output is the 39-tap upsample FIR over the zero-stuffed per-side ring — BOTH
/// channels every tick, PCSX2-exact.
pub struct ReverbCore {
    regs: ReverbRegs,
    /// The effect-area ring buffer (the SPU2's delay RAM in the ESA..EEA window).
    mem: Vec<i16>,
    /// The write position (the "current" reverb address, `Cycles >> 1` in PCSX2).
    pos: usize,
    tick: u32,
    /// `RevbDownBuf[2][64]`: the last 64 dry stereo inputs, per side (Reverb.cpp L50-53).
    down: [[i16; 64]; 2],
    /// `RevbUpBuf[2][64]`: the zero-stuffed wet outputs, per side (Reverb.cpp L115-118).
    up: [[i16; 64]; 2],
    /// `RevbSampleBufPos` (Reverb.cpp L119).
    buf_pos: usize,
}

impl ReverbCore {
    /// Builds a core for the given registers and effect-area size in samples (`game_area`).
    pub fn new(regs: ReverbRegs, area: usize) -> ReverbCore {
        ReverbCore {
            regs,
            mem: vec![0; area.max(1)],
            pos: 0,
            tick: 0,
            down: [[0; 64]; 2],
            up: [[0; 64]; 2],
            buf_pos: 0,
        }
    }

    /// `ReverbDownsample_reference` (ReverbResample.cpp): the 39-tap FIR over this side's input
    /// ring, `>> 15`, clamped. The window covers `buf_pos−39 .. buf_pos−1` — the input stored this
    /// tick enters the window on the NEXT tick, exactly like PCSX2 (the store happens before the
    /// FIR, the buffer position increments after it).
    fn downsample(&self, right: bool) -> i32 {
        let idx = (self.buf_pos + 64 - 39) & 63;
        let side = right as usize;
        let mut out: i32 = 0;
        for (i, c) in REVERB_FIR_DOWN.iter().enumerate() {
            out += self.down[side][(idx + i) & 63] as i32 * c;
        }
        clamp16(out >> 15)
    }

    /// `ReverbUpsample_reference` (ReverbResample.cpp): the 39-tap FIR over BOTH side rings (the
    /// up coefs are the down coefs × 2). Called with `buf_pos` already advanced, so the window is
    /// `buf_pos−39 .. buf_pos−1` of the NEW position = the just-written slot included; the even
    /// taps read one parity (the fresh side's history) and the odd centre tap reads the opposite
    /// parity (the other side's wet from 19 ticks ago) — both channels return a value every call.
    fn upsample(&self) -> [i16; 2] {
        let idx = (self.buf_pos + 64 - 39) & 63;
        let (mut l, mut r) = (0i32, 0i32);
        for (i, c) in REVERB_FIR_UP.iter().enumerate() {
            l += self.up[0][(idx + i) & 63] as i32 * c;
            r += self.up[1][(idx + i) & 63] as i32 * c;
        }
        [clamp16(l >> 15) as i16, clamp16(r >> 15) as i16]
    }

    /// One reverb tick: `input` is the stereo dry signal of the reverb-routed voices this tick.
    /// Returns the wet stereo output — BOTH sides, every tick (PCSX2's `DoReverb` →
    /// `ReverbUpsample`).
    pub fn tick(&mut self, input: [i16; 2]) -> [i16; 2] {
        let r = self.regs.clone();
        let right = self.tick & 1 == 1; // PCSX2's `R = Cycles & 1`
        // MUL is plain C s32 math in PCSX2 (`#define MUL(x, y) ((x) * (y) >> 15)`, Reverb.cpp
        // L97): the extreme-corner product wraps on x86 like C's imul, so use wrapping_mul —
        // identical to the C semantics in every non-corner case.
        let mul = |x: i32, y: i32| x.wrapping_mul(y) >> 15;
        let n = self.mem.len();
        let at = |off: usize| self.mem[(self.pos + off) % n] as i32;

        // Store the dry input both sides (Reverb.cpp L50-53), before the FIR.
        let bp = self.buf_pos;
        self.down[0][bp] = input[0];
        self.down[1][bp] = input[1];

        // Sample offsets are the register value × 4 (the hardware's stored 17-bit form).
        let x4 = |v: u16| (v as usize) * 4;
        let (same_src, same_dst) = if right { (x4(r.same_r_src), x4(r.same_r_dst)) } else { (x4(r.same_l_src), x4(r.same_l_dst)) };
        let (diff_src, diff_dst) = if right { (x4(r.diff_l_src), x4(r.diff_r_dst)) } else { (x4(r.diff_r_src), x4(r.diff_l_dst)) };
        let (comb1, comb2, comb3, comb4) = if right {
            (x4(r.comb1_r_src), x4(r.comb2_r_src), x4(r.comb3_r_src), x4(r.comb4_r_src))
        } else {
            (x4(r.comb1_l_src), x4(r.comb2_l_src), x4(r.comb3_l_src), x4(r.comb4_l_src))
        };
        let (a1_dst, a2_dst, a1_size, a2_size) = if right {
            (x4(r.apf1_r_dst), x4(r.apf2_r_dst), x4(r.apf1_size), x4(r.apf2_size))
        } else {
            (x4(r.apf1_l_dst), x4(r.apf2_l_dst), x4(r.apf1_size), x4(r.apf2_size))
        };

        // `in = MUL(IN_COEF_R/L, ReverbDownsample(*this, R))` (Reverb.cpp L98): the input is the
        // 39-tap FIR of this side's dry ring, and IN_COEF is s16 (0x8000 = −1.0).
        let in_coef = s16(if right { r.in_coef_r } else { r.in_coef_l });
        let input = mul(in_coef, self.downsample(right));

        // All volumes s16 (V_Reverb fields are s16 in PCSX2 defs.h): wall 0xA680 and the combs
        // 0xBCE0/0xBEF0 of STUDIO_3 are negative gains, not the +1.3/+1.5 of a raw u16 read.
        let wall = s16(r.wall_vol);
        let iir = s16(r.iir_vol);
        let same = mul(iir, input + mul(wall, at(same_src)) - at(same_dst.wrapping_sub(1) % n)) + at(same_dst.wrapping_sub(1) % n);
        let diff = mul(iir, input + mul(wall, at(diff_src)) - at(diff_dst.wrapping_sub(1) % n)) + at(diff_dst.wrapping_sub(1) % n);

        let mut out = mul(s16(r.comb1_vol), at(comb1))
            + mul(s16(r.comb2_vol), at(comb2))
            + mul(s16(r.comb3_vol), at(comb3))
            + mul(s16(r.comb4_vol), at(comb4));

        let a1_src = a1_dst - a1_size;
        let apf1 = out - mul(s16(r.apf1_vol), at(a1_src));
        out = at(a1_src) + mul(s16(r.apf1_vol), apf1);
        let a2_src = a2_dst - a2_size;
        let apf2 = out - mul(s16(r.apf2_vol), at(a2_src));
        out = at(a2_src) + mul(s16(r.apf2_vol), apf2);

        let set = |mem: &mut Vec<i16>, i: usize, v: i32| {
            let j = i % mem.len();
            mem[j] = clamp16(v) as i16;
        };
        set(&mut self.mem, self.pos + same_dst, same);
        set(&mut self.mem, self.pos + diff_dst, diff);
        set(&mut self.mem, self.pos + a1_dst, apf1);
        set(&mut self.mem, self.pos + a2_dst, apf2);

        // Zero-stuffed wet ring + advance (Reverb.cpp L114-119): the computed side's `out` lands
        // at buf_pos, the other side's slot is zeroed, then the position moves.
        let out = clamp16(out) as i16;
        self.up[right as usize][bp] = out;
        self.up[!right as usize][bp] = 0;
        self.buf_pos = (bp + 1) & 63;

        // The reverb address is `(Cycles >> 1) + offset` (PCSX2 `RevbGetIndexer`): the left and
        // right ticks of a pair read/write the same position; it advances once per pair.
        if right {
            self.pos = (self.pos + 1) % n;
        }
        self.tick += 1;
        self.upsample()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mode 4 (STUDIO_3 — what `PS2_SetupReverb` writes) decodes to the exact register set [M
    /// ps2sdk effect.c EffectParams row 4; PCSX2 regs.h naming]. Numbers only.
    #[test]
    fn studio3_preset_decodes_to_the_register_set() {
        let r = ReverbRegs::from_mode(GAME_EFFECT_MODE);
        // Volumes/sizes are plain halfwords: row 0x00A900E3 → {0xE3, 0xA9}, 0x4FA86F60 → {0x6F60, 0x4FA8}, …
        assert_eq!(r.apf1_size, 0xE3);
        assert_eq!(r.apf2_size, 0xA9);
        assert_eq!(r.iir_vol, 0x6F60);
        assert_eq!(r.comb1_vol, 0x4FA8);
        assert_eq!(r.comb2_vol, 0xBCE0);
        assert_eq!(r.comb3_vol, 0x4510);
        assert_eq!(r.comb4_vol, 0xBEF0);
        assert_eq!(r.wall_vol, 0xA680);
        assert_eq!(r.apf1_vol, 0x5680);
        assert_eq!(r.apf2_vol, 0x52C0);
        // The 20 delay registers, in hardware order (ps2sdk SetEffectData write order):
        assert_eq!(r.same_l_dst, 0x0DFB);
        assert_eq!(r.same_r_dst, 0x0B58);
        assert_eq!(r.comb1_l_src, 0x0D09);
        assert_eq!(r.comb1_r_src, 0x0A3C);
        assert_eq!(r.comb2_l_src, 0x0BD9);
        assert_eq!(r.comb2_r_src, 0x0973);
        assert_eq!(r.same_l_src, 0x0B59);
        assert_eq!(r.same_r_src, 0x08DA);
        assert_eq!(r.diff_l_dst, 0x08D9);
        assert_eq!(r.diff_r_dst, 0x05E9);
        assert_eq!(r.comb3_l_src, 0x07EC);
        assert_eq!(r.comb3_r_src, 0x04B0);
        assert_eq!(r.comb4_l_src, 0x06EF);
        assert_eq!(r.comb4_r_src, 0x03D2);
        assert_eq!(r.diff_l_src, 0x05EA);
        assert_eq!(r.diff_r_src, 0x031D);
        assert_eq!(r.apf1_l_dst, 0x031C);
        assert_eq!(r.apf1_r_dst, 0x0238);
        assert_eq!(r.apf2_l_dst, 0x0154);
        assert_eq!(r.apf2_r_dst, 0x00AA);
        // Input coefficients: +1.0 on both sides.
        assert_eq!(r.in_coef_l, 0x8000);
        assert_eq!(r.in_coef_r, 0x8000);
    }

    /// ROOM (mode 1) spot-checks and the all-preset invariants: input coefficients (±1.0 except
    /// OFF, which mutes the input), and every delay register small enough for the DSP's ×4 scaling
    /// inside every preset's own area.
    #[test]
    fn presets_decode_consistently() {
        let room = ReverbRegs::from_mode(1);
        assert_eq!(room.apf1_size, 0x7D);
        assert_eq!(room.same_l_dst, 0x04D6);
        for mode in 0..EFFECT_PRESETS.len() {
            let r = ReverbRegs::from_mode(mode);
            let want = if mode == 0 { (0, 0) } else { (0x8000, 0x8000) };
            assert_eq!((r.in_coef_l, r.in_coef_r), want, "mode {mode}");
            let area = ((EFFECT_SIZES[mode] as usize) << 4) - 1;
            let big = [r.same_l_dst, r.same_r_dst, r.diff_l_dst, r.diff_r_dst, r.apf1_l_dst, r.apf1_r_dst, r.apf2_l_dst, r.apf2_r_dst];
            for v in big { assert!((v as usize) * 4 < area, "mode {mode} delay {v:#x}"); }
        }
    }

    /// `psiSetReverb`'s EVOL chain: exact integer math [M SFX.IRX 0xc840].
    #[test]
    fn evol_depth_numbers() {
        assert_eq!(psi_set_reverb_evol(0), 0x0000);
        assert_eq!(psi_set_reverb_evol(100), 0x3332); // 80 * 0x3FFF / 100
        assert_eq!(psi_set_reverb_evol(50), 0x1999);  // 40 * 0x3FFF / 100
        assert_eq!(psi_set_reverb_evol(20), 0x0A3D);  // 16 * 0x3FFF / 100 = 2621
        assert_eq!(psi_set_reverb_evol(25), 0x0CCC);  // 20 * 0x3FFF / 100 = 3276
        for d in [0, 7, 20, 55, 99, 100] {
            assert_eq!(psi_set_reverb_evol(d) as i32, (((d * 0x50) / 100) * 0x3FFF) / 100);
        }
    }

    /// `PS2_SetupReverb`'s fixed setup: mode 4, EEA per core, ESA from the effect size, effects bit.
    #[test]
    fn game_setup_registers() {
        assert_eq!(GAME_EFFECT_MODE, 4);
        assert_eq!(GAME_EEA, [0x1FFFFF, 0x1DFFFF]);
        // EffectSizes[4] = 0x6FE (matches the 0x6fe0 the decompile keeps beside the attr [M]).
        assert_eq!(EFFECT_SIZES[GAME_EFFECT_MODE], 0x6FE);
        // ESA = EEA − (0x6FE*16 − 2).
        assert_eq!(effect_esa(0, GAME_EFFECT_MODE), 0x1FFFFF - 0x6FDE);
        assert_eq!(effect_esa(1, GAME_EFFECT_MODE), 0x1DFFFF - 0x6FDE);
        assert_eq!(game_area(0), 0x6FDF);
        assert_eq!(SD_ENABLE_EFFECTS, 0x80);
        assert_eq!(SD_SWITCH_VMIXEL, [0x1900, 0x1901]);
        assert_eq!(SD_SWITCH_VMIXER, [0x1B00, 0x1B01]);
    }

    /// The environment chain [M SFX.IRX 0xdb4/0xe60/0xd68]: clamps, the −26472 underwater magic,
    /// the +16/−4 and ±4 ramps, and the exact EVOL values it writes on the way up.
    #[test]
    fn environment_ramp_and_underwater() {
        let mut e = SfxEnvironment::default();
        assert_eq!((e.depth, e.wet, e.underwater), (0, 100, false));
        e.set(100, false);
        assert_eq!((e.depth_target, e.wet_target), (100, 100));
        // Depth ramps +16 per update; 7 updates reach 100 (16,32,48,64,80,96,100).
        let mut evols = Vec::new();
        for _ in 0..7 { evols.push(e.update()); }
        assert_eq!(e.depth, 100);
        assert_eq!(e.update(), 0x3332);
        assert_eq!(evols, [0x07AD, 0x0FFF, 0x1851, 0x20A3, 0x28F5, 0x30A3, 0x3332]);
        assert_eq!((e.wet, e.wet_target), (100, 100));
        // Indoors: wet target 50, ramps down by 4 per update; depth follows at −4 (the clamp) until
        // the last 2-step. Converge both, then assert.
        e.set(30, true);
        assert_eq!(e.wet_target, 0x32);
        for _ in 0..64 { if e.depth == e.depth_target && e.wet == e.wet_target { break; } e.update(); }
        assert_eq!((e.depth, e.wet), (30, 50));
        // A drop from 100 to 30 passes through the −4 clamp first (96 → 92 …), not −4 total:
        let mut e2 = SfxEnvironment { depth_target: 100, depth: 100, wet_target: 100, wet: 100, underwater: false };
        e2.depth_target = 30;
        e2.update();
        assert_eq!(e2.depth, 96); // the −4 clamp, not −16
        // Underwater magic −0x6768 (−26472): flag + depth 0; ordinary clamps otherwise.
        let mut e3 = SfxEnvironment::default();
        e3.set(-0x6768, false);
        assert_eq!((e3.underwater, e3.depth_target), (true, 0));
        e3.set(-5, false);
        assert_eq!((e3.underwater, e3.depth_target), (false, 0));
        e3.set(150, false);
        assert_eq!(e3.depth_target, 100);
        e3.set(0, false);
        assert_eq!(e3.update(), 0);
    }

    /// The DSP: silence stays silence; a one-sample impulse comes back as a bounded, deterministic
    /// tail on BOTH sides (the FIR upsample returns both channels every tick). The pinned first
    /// returns are a regression pin for the equation transcription (same fixed-point chain,
    /// computed twice). NOTE: the pre-signedness version of this test pinned a left-only tail —
    /// V_Reverb's volumes are s16 (PCSX2 defs.h) and DoReverb returns both sides via
    /// ReverbUpsample, so the old pins encoded the wrong math and were recomputed.
    #[test]
    fn reverb_core_silence_impulse_and_tail() {
        let regs = ReverbRegs::from_mode(GAME_EFFECT_MODE);
        let area = game_area(0);
        let mut core = ReverbCore::new(regs, area);
        for _ in 0..100 { assert_eq!(core.tick([0, 0]), [0, 0]); }
        // The impulse reaches the DSP through the 39-tap downsample FIR and IN_COEF −1.0 (s16
        // 0x8000), lands in the delay RAM at same/diff_dst, and returns through the combs and the
        // FIR upsample on both channels.
        assert_eq!(core.tick([12345, 0]), [0, 0], "tick 0: the FIR window is empty, combs read a zeroed buffer");
        let mut tail = Vec::new();
        for i in 0..9000 { let w = core.tick([0, 0]); if w != [0, 0] { tail.push((i + 1, w)); } }
        assert!(!tail.is_empty(), "the impulse must come back");
        assert!(tail.iter().all(|(_, w)| w[0].abs() <= 12345 && w[1].abs() <= 12345), "bounded");
        // Determinism: a fresh core repeats the exact same tail.
        let mut core2 = ReverbCore::new(ReverbRegs::from_mode(GAME_EFFECT_MODE), area);
        core2.tick([12345, 0]);
        let mut tail2 = Vec::new();
        for i in 0..9000 { let w = core2.tick([0, 0]); if w != [0, 0] { tail2.push((i + 1, w)); } }
        assert_eq!(tail, tail2);
        // Re-measured 2026-10-22 after the address fix (it advances once per L/R pair, PCSX2
        // `RevbGetIndexer` `Cycles >> 1` — the first pins, 970/3859, were taken with it advancing
        // every tick, i.e. every delay halved).
        // Pin the first ten returns (regression pins of the transcription; measured on this code —
        // the pre-signedness branch pinned a left-only tail with different numbers, which encoded
        // the u16 volume reads that PCSX2's s16 V_Reverb disproves). The left side returns first
        // (same/diff write-backs read back through the comb chain ~970 ticks in), the right side
        // only through the shared-RAM diff cross-feed (DIFF_L writes read at DIFF_R_SRC, 3859).
        assert_eq!(tail[..10], [
            (1918, [-1, 0]), (1922, [-3, 0]), (1924, [7, 0]), (1926, [-20, 0]), (1928, [45, 0]),
            (1930, [-98, 0]), (1932, [216, 0]), (1934, [-765, 0]), (1935, [-1267, 0]), (1936, [-892, 0]),
        ]);
        assert_eq!(tail.iter().find(|(_, w)| w[1] != 0), Some(&(8533, [-2, -1i16])));
        assert_eq!(tail.iter().map(|(_, w)| w[0].abs().max(w[1].abs())).max(), Some(1462));
    }

    /// (a) A sustained FULL-SCALE noise input for 10 s of samples stays bounded and does not
    /// saturate, and the tail decays once the input stops. This is the signedness teeth: with the
    /// registers applied as raw u16 (wall +1.30, iir 0.87 → loop gain 1.13 > 1) the same/diff IIR
    /// loops blow up and ride the s16 clamp; with the signed gains (wall −0.699) the DSP is
    /// stable. A one-impulse input cannot see this. Deterministic xorshift noise, no draws from
    /// the port's shared RNG.
    #[test]
    fn reverb_core_full_scale_noise_stays_bounded() {
        let mut core = ReverbCore::new(ReverbRegs::from_mode(GAME_EFFECT_MODE), game_area(0));
        let mut rng: u32 = 0x1234_5678;
        let mut noise = move || {
            rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5;
            (((rng >> 8) & 0xFFFF) as i32 - 0x8000) as i16 // full-scale i16 range, deterministic
        };
        let (mut peak, mut clamped) = (0i32, 0u32);
        for _ in 0..480_000 {
            let w = core.tick([noise(), noise()]);
            for x in w {
                let v = x as i32;
                peak = peak.max(v.abs());
                if v.abs() >= 32_767 { clamped += 1; }
            }
        }
        let samples = 960_000u32;
        assert!(peak > 0, "full-scale noise must produce wet output");
        // Bounded: every sample inside i16 (the DSP clamps, like PCSX2's clamp_mix), and the clip
        // duty is TINY — only the loud peaks touch the rails (measured 0.73 % here). The u16
        // volume reads blow the same/diff loop up (measured 17.2 %) and pin the tail at the rails
        // forever; the signed DSP leaves them.
        assert!(peak <= 32_768, "bounded: peak {peak}");
        assert!((clamped as u64) * 20 < samples as u64, "no sustained saturation: {clamped}/{samples}");
        // Stability: after the input stops the tail DECAYS out of saturation and toward silence
        // (measured: rails for ~10k ticks, then 6972 by 48k and falling). The u16 blow-up is a
        // permanent full-scale limit cycle — every 4800-tick window stays at 32768.
        let mut last_window = 0i32;
        for half in 0..2 {
            let mut wp = 0i32;
            for _ in 0..48_000 { let w = core.tick([0, 0]); for x in w { wp = wp.max((x as i32).abs()); } }
            if half == 1 { last_window = wp; }
        }
        assert!(last_window < 32_767, "tail leaves the rails: last window peak {last_window}");
        assert!(last_window * 4 < peak, "tail decays: last window {last_window} vs driven peak {peak}");
    }

    /// (c) The register signs as applied: STUDIO_3's comb2/comb4/wall volumes and IN_COEFs are
    /// NEGATIVE s16 gains. PCSX2 `defs.h` `struct V_Reverb` declares them `s16`, and `DoReverb`
    /// multiplies them with `MUL(x, y) = (x*y) >> 15` on s32 — the raw halfwords are
    /// sign-reinterpreted at use (`s16()` in this module, which `tick` calls for every volume).
    #[test]
    fn studio3_volume_registers_apply_as_negative_s16() {
        let r = ReverbRegs::from_mode(GAME_EFFECT_MODE);
        assert_eq!(s16(r.comb2_vol), -0x4320); // 0xBCE0 raw: -0.525, not +1.476
        assert_eq!(s16(r.comb4_vol), -0x4110); // 0xBEF0 raw: -0.508, not +1.492
        assert_eq!(s16(r.wall_vol), -0x5980);  // 0xA680 raw: -0.699, not +1.301
        assert_eq!(s16(r.in_coef_l), -0x8000); // -1.0
        assert_eq!(s16(r.in_coef_r), -0x8000);
        // The positive ones stay positive (sign-interpreted, not negated wholesale).
        assert!(s16(r.iir_vol) > 0 && s16(r.comb1_vol) > 0 && s16(r.comb3_vol) > 0);
        assert!(s16(r.apf1_vol) > 0 && s16(r.apf2_vol) > 0);
    }

    /// PCSX2 `RevbGetIndexer`: the address is `(Cycles >> 1) + offset` — both sides of a tick
    /// pair share one position, so it advances once every two ticks.
    #[test]
    fn the_reverb_address_advances_once_per_left_right_pair() {
        let mut c = ReverbCore::new(ReverbRegs::from_mode(GAME_EFFECT_MODE), game_area(0));
        c.tick([0, 0]);
        assert_eq!(c.pos, 0, "after the left tick the pair's address is unchanged");
        c.tick([0, 0]);
        assert_eq!(c.pos, 1, "after the right tick it advances by one");
        for _ in 0..8 { c.tick([0, 0]); }
        assert_eq!(c.pos, 5, "10 ticks = 5 pairs");
    }
}
