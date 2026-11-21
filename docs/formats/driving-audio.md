# Driving missions: sound data (`BNKl` banks, `.asf` streams, EA-XA)

Reader: `crates/nf-formats/src/easnd.rs`. Disc test: `crates/nf-formats/tests/drive_audio_disc.rs`
(goldens `golden/driving_audio_banks.tsv`, `golden/driving_audio_streams.tsv`: per bank / stream the
counts and a SHA-1 of the decoded PCM). Logic side: `docs/systems/drive-audio.md`.

Tags: [M] DRIVING.ELF decompile / disassembly (EE addresses) or SNDDRV.IRX disassembly (the IOP
sound driver, `/MODULES/SNDDRV.IRX`, which keeps its `.symtab`); [D] measured on the disc by the
tests named here; [S] inference.

## Where the files are [D]

Each mission `.VIV` holds `data\audio\`: `g_common.bnk` + `.h`, `banks.ini`, the track's mix `.ini`
(`paris_mis01.ini` …; RACE carries only `default.ini`), and per-level banks with a `.h` id header
each (`pars\g1_paris.bnk` …, `cars\vanquish.bnk`). The `.MUS` (music) and `*EN.SPE` (speech)
archives are BIGF archives of `.asf` streams. Measured totals: 56 banks, 1,379 filled slots, 1,468
patches (1,219 VAG, 249 EA-XA), 30,331,875 bank samples; 236 streams (118 stereo, all EA-XA, all
48,000 Hz), 80,326 data blocks, 495,747,423 stream samples (summed over channels).

## PT header (the per-sound / per-stream header)

- Reader `FUN_00312300` [M]: skip `0xFC` bytes; `0xFF` ends; `0xFD`/`0xFE` are bare markers; any
  other byte is a tag followed by a size byte (`0xFF` = a 4-byte big-endian size follows) and the
  value bytes. `FUN_00312270` reads sizes 0–4 big-endian, **sign-extending** 1-, 2- and 3-byte values.
- Patch builder `FUN_0030b318` [M]: tags up to `0xFE` (another patch follows) or `0xFF`. Tags
  `0x00..=0x25` = patch block (kept raw; meanings not traced), `0x80..=0xA7` = audio block:

| tag | field | default |
|---|---|---|
| `0x80` | version | 0 |
| `0x82` | channels | 1 |
| `0x84` | sample rate (16-bit field) | 22050 |
| `0x85` | samples | 0 |
| `0x86` / `0x87` | loop start / end | -1 / -1 |
| `0x88` | data offset (banks: from the file start [D]) | 0 |
| `0x8C` | memory class (8 or 0x100 on the disc [D]) | 8 |
| `0xA0` | codec: 5 = SPU ADPCM (VAG), 0x0A = EA-XA [D] | 5 |
| `0xA1` | flags; ≠ 1 remaps the class from its bits | 1 |

Tag `0x14` entries are collected as a list (the `CNYS` blocks of the speech streams [D]).

## `BNKl` bank (`FUN_00308088` [M])

```
0x00 "BNKl"   0x04 u8 version (5)   0x06 u16 slots
0x08 u32 data start
0x0C u32 bytes of class-8 data      0x10 u32 bytes of class-0x100 data (version 5)
0x14 u32 × slots  offset of the slot's "PT" header from the slot word itself; 0 = empty
```

The class-0x100 block sits first at the data start, the class-8 block after it; the two tile the
file to its last byte in all 56 banks [D]. Every VAG patch is class 8 and every EA-XA patch class
0x100 [D] (consistent with VAG playing from SPU memory and EA-XA being decoded on the IOP [S]). An
EA-XA patch's data starts with a 16-byte header — byte 1 = 0x0A, then i32 LE loop start, loop end,
sample count, equal to the PT values in all 249 [D] — then the frames. Which code reads those 16
bytes is not traced.

`.h` headers: `#define%31s%d` lines; the key is `SFX_` + the text after the first `_` that follows
the prefix, cut at the first `.` (`SFX_par_SniperFire.ptc` → `SFX_SniperFire`), looked up
case-insensitively (`FUN_002f54e0`, `FUN_002a9198`) [M]. Every id in every `.h` names a filled slot [D].

## EA-XA (SNDDRV.IRX `decxa16c` 0xa390 [M])

15-byte frames of 28 samples: byte 0 = coefficient index (high nibble) and shift − 8 (low nibble),
then 14 bytes, high nibble first. Pairs (.data 0xb470): (0,0) (240,0) (460,-208) (392,-220).
`s = ((n << 28) >> shift + c1·h1 + c2·h2) >> 8`, 32-bit wrapping, history **not clamped**. The
history starts at 0 (`SFILTER_unpackxafinit`). No frame on the disc uses an index above 3 [D]. The
port saturates the output to i16; the IOP's own clamp/mix stage is not traced [S].

## `.asf` stream

Chunks `SCHl` (header: the PT block from +8), `SCCl`, `SCDl` × n, `SCEl`; each `id, u32 LE size`
[D, all 236]. `FUN_00308f30` dispatches `SCDl`/`SCHl` [M]; `FUN_00308d68` [M]: samples = u32 at
block+8 (bit 31 masked), channel `i` data at block + 0xC + 4·channels + u32[block + 0xC + 4i].
Each channel's data starts with 4 bytes before the frames [D: with no skip, 18,881 frame bytes of
the first blocks name a coefficient index > 3; with 4, none]. Those 4 bytes read as two LE i16 track
the decoded history at the block start closely but not exactly. Block joins (sum of |step| over all
joins): history carried 133,175,517; zeroed 866,677,287; loaded from the 4 bytes 137,454,415 [D].
The port carries the history. Which one the IOP does is not traced. The `SCDl` sample sum equals
tag `0x85` in every stream [D].

## Text files

- `banks.ini`: `[track]` sections of `Key=path`; the archive's own track section names exactly
  banks it carries [D]. Keys → slots in `docs/systems/drive-audio.md`.
- Mix `.ini` (`FUN_002f3058` [M]): `\r`-terminated lines, `sscanf("%[^=]=%f, %d")`, name trimmed
  of trailing spaces; missing file → `data/audio/default.ini`.
