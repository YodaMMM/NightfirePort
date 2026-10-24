# cite-libsd-reverb-tables

The reverb preset values in `crates/nf-audio/src/reverb.rs` (STUDIO_3/mode 4 at :416, the ROOM
spot-checks and invariants at :459, `EFFECT_SIZES[4] == 0x6FE` at :492) were cited to ps2sdk's
`effect.c` — a reimplementation. I found both tables inside the disc's own LIBSD module, cited the
bytes, diffed every value the port uses against them, and fixed the one place the port disagreed
with the disc.

## What the disc says

The module ([D]): `/MODULES/LIBSD.IRX` on SLUS-20579 — LBA 768, 25,397 bytes, sha1
`7217b6c78b233d9ad72dbd379f148ad334311b8a` (matches `crates/nf-formats/golden/disc_files.tsv`).
Extracted with dd from the ISO (byte offset 768 × 2048 = 0x180000). It is an IOP ELF
(e_type 0xFF80, MIPS LE); its single load segment maps **file offset 0xb0 to va 0**, so
va = file − 0xb0 below. This is the module SFX.IRX imports: its `sceSd*` symbols at 0x14814+
are 8-byte import stubs, and the disc's own `sceSdSetEffectAttr` body lives in LIBSD.IRX.

**The effect size table** ([D]): 10 u32 words at **file offset 0x4208 (va 0x4158)**:

```
0x4208: 00000002 000004d8 000003e8 00000908 00000dfc
0x421c: 000015bc 00001ed8 00003008 00003008 00000780
```

Each word is exactly **2× the 16-sample-unit size** ps2sdk's `EffectSizes` holds, and the code
consumes it that way: `lw $v1, 0x4158(mode*4)` at **0x9dc**, then `sll $v1, $v1, 3` at **0x9e4**
(samples = word × 8 = units × 16). Mode 4 reads **0xDFC** → 0xDFC << 3 = **0x6fe0** samples.

**The effect parameter table** ([D]): 10 rows of **0x44 bytes** at **file offset 0x4230
(va 0x4180)**. Each row is a reserved zero u32 followed by the 16 param u32s (the last one is the
`IN_COEF 0x8000,0x8000` pair). The indexing is visible in the consumer: at **0xa08..0xa24** libsd
computes `src = 0x4180 + mode*0x44` (`sll 4 / addu / sll 2` = mode × 68) and memcpy's **0x44
bytes** to its stack (memcpy helper at 0x880; a second copy at 0xae4..0xb00). It then consumes
**row words 1..16** as the register set (the ECHO/DELAY rewrite path touches the same slots, e.g.
`sh` to stack +0x28/+0x2a = row words 6/7 = same_l/r_dst at 0xb40/0xb50).

**The attr struct** ([D] SFX.IRX 0xc944 exact disassembly): the game builds 24 bytes at $sp+0x14
and passes that as attr — `sw 4,0x18($fp)` (mode 4 at **attr+4**), `sh 0,0x1c` / `sh 0,0x1e`
(depth 0/0 at attr+8/+a), `sw 0x1e,0x20` (delay at attr+0xc), `sw 0xc8,0x24` (feedback 200 at
attr+0x10), `sw 0x6fe0,0x28` (**0x6fe0 at attr+0x14**). The first attr word ($fp+0x14) is never
stored. Disc libsd reads only attr+4 (mode, 0x960), attr+8/+a (depths → P-block regs 0x764/0x766 +
core*40, 0xcac..0xcc4) and, for ECHO/DELAY modes only, attr+0xc/0x10 (delay/feedback, 0xb18/0xbfc).
**attr+0x14 is never read.** ps2sdk's 10-byte `sceSdEffectAttr` (mode at +0) is not the disc's
layout; the port's doc now says so.

**The ESA formula** ([D]): `sceSdGetAddr`-style helper at 0xdb8 reads the EEA reg (per-core base +
0x33c) and returns `(reg << 17) | 0x1FFFF`. SetEffectAttr then computes
`ESA = EEA − (samples − 1)` at **0x9e8..0x9ec** and stores it to a shadow (0x4a38 + core*20),
written to the hardware as HI reg 0x2e0 = esa>>17 and LO reg 0x2e2 = esa>>1 at **0xccc..0xd0c**.
So the effect area is **exactly samples = size × 16** — 0x6FE0 samples for mode 4.

## Comparison: port vs disc

I diffed all 10 size words and all 10 × 16 param words against `EFFECT_SIZES` /
`EFFECT_PRESETS` (throwaway script, `scripts/libsd_diff.py` outside the repo): **every value the
port uses matches the disc word-for-word, in all ten modes.** ps2sdk's reimplementation is
byte-faithful for the table contents. The tests' numbers (STUDIO_3 registers, ROOM spot-checks,
`EFFECT_SIZES[4] == 0x6FE`) therefore stand — I re-cited them from the disc bytes instead of
ps2sdk, and added the sister test `disc_libsd_tables_are_the_port_tables`, which pins the raw
disc words (the 10 size u32s and the mode-4 row's 16 u32s) plus the derived quantities, so any
future drift from the disc fails by name.

**The one real difference — the ESA/area off-by-one.** The port computed
`ESA = EEA − (size*16 − 2)` (area 0x6FDF) from ps2sdk's `effect.c`. The disc computes
`ESA = EEA − (samples − 1)` → **area 0x6FE0**. Disc wins: `effect_esa` now subtracts
`size*16 − 1`, `game_area(0)` is 0x6FE0, and `game_setup_registers` pins
`0x1FFFFF − 0x6FDF` / `0x1DFFFF − 0x6FDF` / area 0x6FE0. The DSP tail pins in
`reverb_core_silence_impulse_and_tail` were re-measured by the suite and are unchanged — every
delay offset (×4) sits far below the old boundary, so the extra sample changes no read or write
position. The module doc no longer claims libsd "resets" delay/feedback for non-ECHO/DELAY modes
(the disc simply never reads those attr fields outside ECHO/DELAY) and now documents the disc's
attr layout and the 17-bit delay form (`val<<2` at 0x139c/0x13a4).

## The 0x6fe0 stack word (PS2_SetupReverb 0xc944:23)

**Yes — the table agrees with it.** 0x6fe0 = 0xDFC << 3 = (0x6FE × 2) << 3 = 0x6FE × 0x10: it is
exactly the mode-4 effect-area size in samples that the disc's size table yields, in the exact
scaling (word << 3) the disc's own code applies at 0x9e4. It sits at attr+0x14 in the game's
24-byte attr block, and disc libsd never reads that field (its attr reads are +4/+8/+a, plus
+c/+0x10 for ECHO/DELAY only), so the word is inert as far as the call is concerned — but the
number the game stacks there is precisely what its own LIBSD table says the mode-4 area size is.

## What I ran

- `cargo test --offline --release -j2 -p nf-audio -- --test-threads=1`: all green — reverb unit
  tests 23 passed (incl. the new `disc_libsd_tables_are_the_port_tables`, the updated
  `game_setup_registers`, and the unchanged DSP tail pins), plus the crate's live tests.
- Full four-crate fast suite (`python3 tools/test/suite.py`) restarted on the final tree after
  the fix; total line quoted below.

## Live checks for the main agent

None needed — the evidence is disc bytes and the port's sim never disagreed with them except for
the ESA off-by-one, which the tests now pin.

## What's left

Nothing for this task. Two out-of-scope observations, marked [S] unverified: the P-block regs the
attr depths land on (0x764/0x766 + core*40) could be given their port register-map names; and the
ECHO/DELAY-only delay/feedback rewrite (row words rewritten from attr+0xc/0x10 at 0xb1c..0xb7c,
plus the mysterious row word 0 = 0x0C011C00 written at 0xb0c) is cited but not modelled — the
game only ever runs mode 4.

## Usage

- Input tokens / output tokens / context window used / cost: not visible to me (this sandbox
  exposes no token, credit or context figures to copy).
