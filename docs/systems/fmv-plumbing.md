# FMV plumbing: the sceMpeg callback family, the IOP PCM transport and the GS upload helpers

This is the machinery behind docs/systems/movies.md (which covers what plays where, skipping and
volume). The whole-ELF scan (2026-10-20) flagged 22 of its functions as never mentioned in
`crates/*/src`; unported-misc-1 (2026-10-21) traced every one to its caller and cites each as
**replaced** by the port's whole-stream movie player (`crates/nf-view/src/movie.rs`), texture
upload (`crates/nf-view/src/gpu.rs`) and PSS demux/decode (`crates/nf-formats/src/pss.rs`).
Nothing in this family needed porting: the port deliberately has no IOP, no SIF DMA, no IPU and
no GIF tags — see docs/systems/movies.md "Port status" and movie.rs's module doc.

Evidence tags: [M] decompile (function + address), [L] EE snapshot, [I] inference marked as such.

## Registration: one site wires the whole family

`StartUpBGFMV` 0x1083a8 — called by `psiStartBackgroundMovie` 0x1081a8 and by the loop-restart
`psiDecodeBackgroundMovieFrame` 0x108060 — is the **only** game caller of `sceMpegCreate`,
`sceMpegAddStrCallback` and `sceMpegAddCallback` in the whole decompile [M grep]. It registers
exactly five callbacks [M 0x1083a8:12-17]:

| call | callback | address | invoked by |
|---|---|---|---|
| `sceMpegAddStrCallback(mp,0,0,…)`, stream 0 | `videoCallback` | 0x108848 | demuxer per video PES (libmpeg `sceMpegDemuxPssRing` 0x2295c0) |
| `sceMpegAddStrCallback(mp,2,0,…)`, stream 2 | `audioCallback` | 0x108988 | demuxer per audio PES |
| `sceMpegAddCallback(mp,1,…)`, event 1 | `nodataCallback` | 0x108a60 | `sceMpegGetPicture` 0x22a3f8 idle path [I: call site in the unrecovered 0x22a730 jumptable] |
| `sceMpegAddCallback(mp,4,…)`, event 4 | `backgroundCallback` | 0x108b80 | same idle path [I] |
| `sceMpegAddCallback(mp,0,…)`, event 0 | `errorCallback` | 0x108c00 | same, error path [I] |

Reachability of the whole family follows from `psiStartBackgroundMovie`'s callers [M, all
documented in movies.md]: boot logos / attract / extras / trailer / win-game page handlers
(P_INTRO 0x209e88, P_ATTRACT 0x20a1f0, P_ESTHERO 0x20a060, P_FMVPLAYER 0x20ad30, P_TRAILER
0x214f10, P_WINGAME 0x215100), the mission flow (ResetMap_Load 0x1d6360 briefing, Mission_Update
0x182c60 case 6 outros via GameFlow state 3/6) and the menu background (`Menu_PlayMovie` 0x215990
← Manager_SendMessage 0x21c858). So every function below is **reached** on shipped paths; the
only question was who does its job in the port.

## Per-function notes and dispositions

### The demux pump and the stream callbacks

- `fillBuff` 0x108c08 [M]: the game-side demux pump. Reads the next 0x4000 bytes (8 CD sectors)
  through the `fileRead` pointer (0x30d684 = `strFileRead` 0x108fe0, `sceCdSt*` streaming) into
  `muxBuff` 0x30de40, then `sceMpegDemuxPss(mp 0x30dda0, …)`; leftovers sit at the tail.
  Callers: `StartUpBGFMV` priming loop, `SwapBGFMVFrame`, and both event callbacks. **Replaced**
  by `pss::demux` (whole-file PES demux, `crates/nf-formats/src/pss.rs`), cited at movie.rs
  `Movie::load` and pss.rs `demux`.
- `videoCallback` 0x108848 [M]: copies the video PES payload (+8 ptr / +0xC len) into the video
  ES ring (`demuxBuff` 0x30d694, size 0x30d698, cursor 0x30d69c, uncached-mirror OR 0x20000000),
  full/empty disambiguated via `demuxBuffPresent` 0x30d6a0 / `demuxBuffFuture` 0x30d6a4.
  **Replaced** by the demuxed whole-stream buffer + `MovieStream` decode-ahead worker
  (movie.rs, cited at `MovieStream::start`).
- `audioCallback` 0x108988 [M]: reserves PCM ring space with `audioDecBeginPut` 0x107480, skips
  the 4-byte `FF A0` sub-stream header, memcpys the payload across the two ring segments and
  commits with `audioDecEndPut` 0x107530. **Replaced** by `Audio::parse/decode`
  (`pss.rs`, header skip = `AUDIO_HEADER_BYTES`, PES sub-stream skip documented there) — cited
  at movie.rs `Player::open` (the `Vec<i16>` PCM build) and pss.rs.

### The IOP PCM transport (AudioDec 0x30dd40)

All EE→IOP traffic is `sceSdRemote(1, 0x80d0/0x80e0/0x8100, …)` RPCs to the game's SFX.IRX-side
decoder plus synchronous SIF DMA. Struct map from the four functions' accesses [M]:

| off | meaning |
|---|---|
| +0x00 | state 0 priming / 1 ring armed / 2 playing / 3 resetting |
| +0x04.. | 0x28-byte SSHd/SSbd header staged while priming (+0x2c counts to 0x28) |
| +0x30/+0x3c | EE ring base / size; +0x34 put, +0x38 pending, +0x40 cumulative |
| +0x44/+0x48 | IOP ring base / size; +0x4c put, +0x54 cumulative DMAed |
| +0x50 | IOP playback offset (queried via `sceSdRemote 0x80e0`) |
| +0x58/+0x5c | IOP work area (0x800 bytes, zeroed at create) |

- `audioDecCreate` 0x1072b8 [M]: zeroes the state, stores ring geometry, memsets + DMAs the
  0x800 work area (`sendToIOP`), sets master volume 0x3fff. Sole caller `StartUpBGFMV` 0x1083a8.
  **Replaced** by `Player::open` (whole-track PCM allocation; volume via `movie_input_volume`,
  movies.md §Volume).
- `audioDecEndPut` 0x107530 [M]: commits bytes written into `audioDecBeginPut`'s areas; while
  priming, the first 0x28 bytes land in the staged header. Sole caller `audioCallback`. **Replaced**
  by the whole-track PCM buffer (no ring; `pss.rs` handles the header).
- `audioDecIsPreset` 0x1075b0 [M]: `return dec[+0x48] <= dec[+0x54]` — IOP ring filled once =
  audio primed. Sole caller `DecodeBGFMVFrame` 0x108550:34, gating `audioDecStart` 0x107360.
  **Replaced** by opening the cpal device on the first picture (`open_pcm_out` call in
  `present`, movie.rs) — same observable behaviour.
- `iopGetArea` 0x1075c8 [M]: up to two contiguous writable IOP-ring segments 1 KB behind the
  playback address. Sole caller `audioDecSendToIOP` 0x107100 (state 2). **Replaced** by the
  device callback pulling from the PCM buffer (`open_pcm_out`, movie.rs).
- `sendToIOP2area` 0x107698 [M]: fills two IOP ranges from two EE segments via 2–3
  `sendToIOP` transfers (wrap handling). Sole caller `audioDecSendToIOP`. **Replaced** by the
  cpal stream (no IOP bus; the transport is the device callback).
- `sendToIOP` 0x1077e0 [M]: synchronous EE→IOP SIF DMA (`SifDmaTransfer` on stack,
  `FlushCache(0)`, `sceSifSetDma`, busy-wait `sceSifDmaStat`). Callers: `audioDecCreate`,
  `sendToIOP2area`. **Replaced** as above.

### The PATH3 GS upload helpers (classic Sony loadImage tag builders)

Only user in the whole binary: the BG FMV frame upload [M exhaustive grep — no font/texture
loader touches this family]:

- `DecodeBGFMVFrame` 0x108550 calls `setLoadImageTags` 0x107b90 twice per frame (chains into
  `path3tag` 0x30d690 and +0x15100) [M 0x108550:27-30]; `setLoadImageTags` is a pure wrapper
  that adds the 0x400 (one 16×16 CT32 block) source stride and calls `setLoadImageTagsYX`.
- `setLoadImageTagsYX` 0x107910 [M]: builds the whole PATH3 chain into a caller buffer — per
  16×16 block: CNT DMA tag (`setDMAscTag` 0x107bd0, QWC=4) → A+D GIFtag NLOOP=2
  (`setGIFtag` 0x107c20, REGS 0x0E) → `setBITBLTBUF` 0x107c70 (offset 0x50; DBP from
  `CreateDB` 0x10b000's DrawBufBase, DBW 8, DPSM 0 CT32) → `setTRXREG` 0x107cd0 (0x52; 16×16)
  → `setTRXPOS` 0x107ca0 (0x51; dest YX) → `setTRXDIR` 0x107d08 (0x53; 0 = host→GS) → IMAGE
  GIFtag (FLG 2, NLOOP 0x40, EOP on last) → REF DMA tag (QWC 0x40 = 1 KB = one block), END tag.
  Destination = the DBP VRAM slot; the frame is consumed as column-major 16×16 CT32 tiles [M
  arithmetic; tile-order producer outside these files — I].
- `loadImage` 0x107d30 [M]: kicks PATH3 — `D2_TADR = addr`, `D2_QWC = 0`, `D2_CHCR = 0x105`.
  Sole caller `PS2UploadBGMImage` 0x108740 (gates on `BGFMVPlaying && ImageToUpload`, busy-waits
  D2_CHCR/GIF_STAT/GS_CSR), which runs from the INTC handler `VIFDMA__Fi` 0x123e90 (registered
  by `main` 0x109080:23) on the frame's first DMA interrupt.
- Oddity [I, flagged]: chain B (`path3tag+0x15100`, dest Y+0x1c0) is rebuilt every frame but
  nothing kicks it; plausibly a second VRAM slot for the display flip, possibly dead.

**All replaced** by `Renderer`'s wgpu texture path: the decoded 512×448 RGBA frame is written
into `movie_tex`/`movie_bg` (`gpu.rs render()`, `queue.write_texture` + full-screen plane quad).
Cited at gpu.rs, movie-plane block.

### Per-tick flip and pacing

- `SwapBGFMVFrame` 0x1087c8 [M]: despite the name it flips no pointers — it tops the demux
  buffer (`fillBuff` non-blocking while T1 < 450 ticks) and returns `drawid` 0x30c6b0 [I: no
  nonzero writer in the decompiled set; likely an interrupt-side write]. Sole caller
  `psiSwapBGFMVFrame` 0x1082f8 (gated on `BGFMVPlaying`) from `main` per tick. **Replaced** by
  `Player::present`'s per-tick frame hand-out.
- `nodataCallback` 0x108a60 [M]: re-syncs the ES ring, blocking-`fillBuff`s until ≥ 0x1000
  demuxed bytes, then programs the IPU input DMA (`REG_DMAC_4_IPU_TO_QWC`, CHCR 0x101).
  **Replaced** by the bounded decode-ahead channels (`MovieStream::start`, STREAM_AHEAD).
- `backgroundCallback` 0x108b80 [M]: the frame pacer — busy-waits `bgDuration` T1 ticks
  (0x30d688 = w·h·0.000151 ≈ 34.6 at 512×448, set by DecodeBGFMVFrame) while topping the demux
  buffer; `sceMpegGetPicture` blocks inside it. **Replaced** by the audio-master/wall clocks in
  `present_on_audio`/`present`.
- `errorCallback` 0x108c00 [M]: 8 bytes, `return 1` — acknowledge and keep going. **Replaced**
  by the per-GOP `unwrap_or_default` in the decode worker (a bad GOP yields no frames, playback
  continues) and `StartError` for hard failures.

## Summary table (22 functions, all REACHED-CITE-AS-REPLACED)

| address | function | port location |
|---|---|---|
| 0x1072b8 | audioDecCreate | movie.rs `Player::open` |
| 0x107530 | audioDecEndPut | movie.rs `Player::open` PCM build / pss.rs `Audio` |
| 0x1075b0 | audioDecIsPreset | movie.rs `present` → `open_pcm_out` on first picture |
| 0x1075c8 | iopGetArea | movie.rs `open_pcm_out` |
| 0x107698 | sendToIOP2area | movie.rs `open_pcm_out` |
| 0x1077e0 | sendToIOP | movie.rs `open_pcm_out` |
| 0x107910 | setLoadImageTagsYX | gpu.rs movie-plane block (wgpu texture upload) |
| 0x107b90 | setLoadImageTags | gpu.rs movie-plane block |
| 0x107bd0 | setDMAscTag | gpu.rs movie-plane block |
| 0x107c20 | setGIFtag | gpu.rs movie-plane block |
| 0x107c70 | setBITBLTBUF | gpu.rs movie-plane block |
| 0x107ca0 | setTRXPOS | gpu.rs movie-plane block |
| 0x107cd0 | setTRXREG | gpu.rs movie-plane block |
| 0x107d08 | setTRXDIR | gpu.rs movie-plane block |
| 0x107d30 | loadImage | gpu.rs movie-plane block |
| 0x1087c8 | SwapBGFMVFrame | movie.rs `Player::present` |
| 0x108848 | videoCallback | movie.rs `MovieStream::start` / `Movie::load` demux |
| 0x108988 | audioCallback | movie.rs `Player::open` PCM / pss.rs |
| 0x108a60 | nodataCallback | movie.rs `MovieStream` bounded channels |
| 0x108b80 | backgroundCallback | movie.rs `present_on_audio` / `present` clocks |
| 0x108c00 | errorCallback | movie.rs decode worker `unwrap_or_default` |
| 0x108c08 | fillBuff | pss.rs `demux` |

(The three boot/FS/cache functions FS_Init 0x10bc48, initcache 0x109688, allocache 0x109700 are
documented in docs/systems/boot-fs-cache.md.)
