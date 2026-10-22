# Boot FS init and the double-buffered texture staging cache

Three functions the whole-ELF scan (2026-10-20) flagged as never mentioned in `crates/*/src`,
resolved by unported-misc-1 (2026-10-21). All three are **reached** on shipped paths (boot and
render loop) and all three are **replaced** by the port's direct file access and texture upload.
Evidence tags: [M] decompile, [D] disc bytes.

## FS_Init 0x10bc48 (168 bytes) — boot disc/FS init

[M 0x10bc48] Sets `DVDRmode` 0x30c7d8 = 0xff (+1/+2 bytes), `m_file` 0x30c7c4 = 0, then a
blocking retry loop: `sceCdSearchFile(&DVDfp 0x245560, "\FILES.BIN;1")` (the literal is at
0x2e81c0 [D ELF read]) until it returns 1 with `sceCdGetError() == 0`, re-issuing
`sceCdInit(0); sceCdMmode(2)` (DVD mode) on each failure. The resulting `DVDfp.lsn` is the base
sector of every `psiFileOpen/psiFileLoad/psiFileRead` [M 0x10bd08:31 — `offset = DVDfp_lsn +
(FileList[i].lba>>11)`; 0x10ba38:36,54]. Sole caller: `main` 0x109080:21, right after
`sceCdInit`/`sceCdMmode` and before `Mem_Init`/`PS2FreshBoot` — reached at every boot.

**Replaced by** `nf-formats::disc::files_bin_entries` (locates FILES.BIN in the ISO — FS_Init's
`\FILES.BIN;1` search) + `nf-formats::filelist` (`FileList` 0x2446a0, 118×32-byte entries:
name@+0, lba@+0x10, size@+0x14) + `nf-view/src/lib.rs load_impl`'s read closure
(`bin.lba*2048 + fe.offset` = psiFileLoad's sector math). Cited at disc.rs `files_bin_entries`.

## initcache 0x109688 / allocache 0x109700 — the VRAM staging cache

Despite the name, **not** a file preload cache. `initcache(p1, p2)` [M 0x109688] splits a
staging window into two halves: `TotalCacheStart` 0x30d6ac = p1, `TotalCacheSize` 0x30d6b0 =
p2, `CacheFrame` 0x30c738 = 1, `CacheStart` 0x30c73c = p1+p2/2, `CacheSize` 0x30c740 = p2/2,
`iLock` 0x30d6b4 = 0. Every caller passes `(TextureBufBase 0x30c7b0, 0x4000 - TextureBufBase)`
with `TextureBufBase = 0x2a00` set by `CreateDB` 0x10b000:35 [M] — a 0xb00-byte-per-half GS
VRAM staging window. `CacheFlipBuffers` 0x1095d0 [M] toggles the half each frame and ages out
the previous frame's slots by writing 0xff to their lock bytes (`pLock` 0x311e78, ring of 256).

`allocache(out, lockByte, size)` [M 0x109700] bump-allocates from the current half: writes
`CacheStart` to `*out` (the VRAM destination the caller's GS upload packet targets), records the
lock byte, advances. Sole caller: `CacheTexture__FiRiT1` 0x11cd58:50,62,66 [M] — allocates the
VRAM slot for each new texture patch (`(w>>6)*h >>3 or >>2` + 2 or 0x10 for the CLUT) and emits
`PS2TexturePatch2Ptr` upload packets.

**Reachability, twice over** [M]: `main` 0x109080:54 → `PS2RegionChange` 0x1094d8:11 →
`initcache` (boot); `ResetMap_Load` 0x1d6360:19,126 → `psiResetResources` 0x11f390:13 →
`initcache` (every level load). `allocache`: `GameFlow_Main` 0x1c98c8 → `Game_Draw` 0x1e6f50 →
`View_DrawObjList` 0x1e69a0 → `CacheAndFillNewGlistTextures` 0x11f180 → `CacheTexture` →
`allocache` (per-frame, world/object textures); font branch via `psiFont_SetTexture` 0x120880;
loading blobs via `CacheLoadingBlobs` 0x11f2e8 ← `psiPreDraw` 0x121590.

**Replaced by** the port's renderer: level/HUD textures are decoded once and uploaded directly
(`Renderer::new`'s texture creation, `crates/nf-view/src/gpu.rs`) — there is no per-frame VRAM
bump allocator to replace; wgpu owns residency. Cited at gpu.rs `Renderer::new` texture block.
