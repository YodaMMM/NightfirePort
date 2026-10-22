# Sound lifecycle and the frame timer: `Sound_Init`, `Sound_Kill`, `Sound_SetLogSfxFile`, `UpdateT0Count`

Four never-mentioned ACTION.ELF functions around the load path and the frame clock.

## `Sound_Init` 0x1ddf88 (68 bytes) — reached [M]

Body [M 0x1ddf88]: `DList_Init(0x349b00, 0x60, 0x100)` (a 0x100-node × 0x60-byte free list — the
sound system's node pool), `SFXInitialise(0)` (boot the IOP SFX system), `memset(&EventList, 0, 0x100)`
(256 bytes = the 64-u32 music event board).

Callers [M]: `ResetMap_Load` 0x1d6360:24 (warm path) and :131 (cold path) — every level load and the
boot, right after `Sound_Kill` + `Mem_Init` + `hashtable_initialise`. So the game tears down and
re-boots the whole sound system on every load.

Port replacement: the port rebuilds the audio world per load at the same point of the load sequence
— `nf-view/src/lib.rs` `build_sound` (fresh `SfxDriver`, banks, streams, `nf_audio::Output`) and
`wire_music` (fresh `Music::new` — a zeroed 64-u32 event board = the `memset(&EventList,0,0x100)`,
zeroed machine state). Cited there (`Sound_Init` 0x1ddf88).

## `Sound_Kill` 0x1ddfd0 (40 bytes) — reached [M]

Body [M 0x1ddfd0]: `if (SoundInit != 0) SFXReset(); SoundInit = 0;`. `SoundInit` is set to 1 by
`Sound_Ready` 0x1dceb8:107, so from the second load on, every load resets the SFX system first
(`SFXReset`), then `Sound_Init` re-boots it.

Callers [M]: `ResetMap_Load` 0x1d6360:24 and :131 — the same two sites as `Sound_Init`, immediately
before it.

Port replacement: same sites as above — the per-load engine recreation (dropping the previous
`Music` engine and driver state) is the port's `SFXReset` + `SoundInit = 0`. Cited in
`nf-view/src/lib.rs` (`Sound_Kill` 0x1ddfd0).

## `Sound_SetLogSfxFile` 0x1de368 (8 bytes) — reached, and EMPTY [M]

Body [M 0x1de368]: a bare `ret` (the shipped binary ships it empty — whatever logging hook it once
had was compiled out).

Caller [M]: `ResetMap_GameInit` 0x1d5080:43 — called on every game init (after
`Init_SwitchChannels`, before `Mission_Init`).

Port: the call is preserved as nothing — cited at the port's level-init sequence
(`nf-view/src/lib.rs`, the `ResetMap_GameInit`-equivalent build path): `Sound_SetLogSfxFile`
0x1de368 ships as a bare `ret`; there is nothing to port.

## `UpdateT0Count` 0x109468 (40 bytes) — reached every frame [M]

Body [M 0x109468]: read the hardware timer-0 count twice, add the delta to `RunningPS2Count`, write
the delta back (resetting the counter), return the accumulator.

Callers [M] — it is the game's profiling clock between named points:
- `VBlankInt` 0x124088:76: `PS2DrawFrameTime = UpdateT0Count()` — once per vblank;
- `psiPreDraw` 0x121590:15 / `psiPostDraw` 0x121320:11: `PS2DrawCodeTime` brackets the draw;
- `psiPreGame_Run` 0x109520:17-19 / `psiPostGame_Run` 0x109580:11-12: `RunningPS2Count` /
  `PS2GameCodeTime` / `PS2GameCodeTimeW` bracket the game step;
- `PS2_SFX_SendBatchCommandsToIOP` 0x1155d0:12,22, `FlagDMAChainDone` 0x124950:9,
  `UpdateDrops` 0x18c580:91,294: more probes.

Port replacement: the wall-clock role is taken by the port's fixed 60 Hz step — the Sim is
deterministic and runs no wall-clock timing (driver rule: no wall-clock time in Sim), so the
profiling counters (`PS2DrawFrameTime`, `PS2GameCodeTime`, …) have no port counterpart. Cited at the
fixed-step entry (`nf-view/src/scene.rs`, `step_bond`): `UpdateT0Count` 0x109468.
