# misc-9: the kernel-import / libc seam — the 25 never-mentioned functions

Whole-ELF scan (2026-10-21). All 25 sit in the 0x23bdc0+
region: fifteen 16-byte PS2 kernel import stubs (`li $v1, <syscall#>; syscall; jr $ra; nop`
— disassembled from the ELF, [M]), two real vertical-sync routines, and the newlib libc
syscall hooks. These are the seams where the game talks to the PS2 OS; the port talks to
the host OS instead. Per-function: the ELF-verified shape, the full caller map (capstone
jal **and tail-`j`** scan over the LOAD segment 0x107100..0x30d670 — mcDelayThread reaches
`SleepThread` by a tail jump, which a jal-only scan misses), and the port's site.

Evidence tags: **[M]** code (function + address), **[D]** disc, **[L]** EE snapshot, **[S]**
suggested.

## 1. The frame gate — VSync

`main` 0x109080 (called by `_start`) → `CreateDB` 0x10b000:52 → `sceGsSyncV` 0x225930 →
**VSync 0x23c058** and **VSync2 0x23c0e8**; `psiPreGame_Run` 0x109520 does the same at the
pre-game. `VSync` disables interrupts, clears the INTC vsync bit (4) and busy-waits the
next vsync interrupt; `VSync2` additionally registers the kernel flag through
**SetVSyncFlag 0x23bf90** (stub, syscall 0x73) and waits on flag-or-vsync [M decomp]. This
is the game's 60 Hz frame gate — the port's fixed-tick scheduler is the same job:
`ticks_for_frame` (`crates/nf-view/src/lib.rs`) advances the 60 Hz clock per rendered
frame and runs whole ticks (determinism rule: no wall-clock in Sim). Cited there.

## 2. The file-I/O + synchronisation seam

The game's file reads run the IOP CD service through semaphores, DMA transport and cache
flushes. The port loads levels synchronously inside a worker thread with coherent host
memory — no IOP, no semaphores, no caches — so these are the seams the worker-thread load
replaces. All cited in the `crates/nf-view/src/async_load.rs` module doc:

| function | shape | reached how [M] |
|---|---|---|
| **FlushCache 0x23bf20** | stub, syscall 0x64 | **unconditionally** per file read — `psiFileRead` 0x10bdf0:18 `FlushCache(0)` before the `sceCdRead` loop; also `_start`, `CreateDB` 0x10b000, `PS2WipeVram`, `PS2EndCalcPacket` 0x124510, `psiAllocateLoadingBlobs` 0x11f270, `sendToIOP` 0x1077e0 (14 call sites) |
| **iFlushCache 0x23bf40** | stub, syscall −0x68 | `RunNextModule` 0x120008 (the module switch — every level load / DRIVING hand-off) |
| **SyncDCache 0x23c410** | 124 B real code (range flush wrapper) | `amemcpy` 0x11f5c0 — the game's memcpy, called before every DMA staging copy — plus `_send_to_iop`, `scePadGetDmaStr` |
| **DeleteSema 0x23be30** | stub, syscall 0x41 | 26 call sites: `RunNextModule`, `cdvd_exit`, `sceCdDelayThread`, `sceClose`, `sceIoctl`, `sceOpen`, `sceSifBindRpc`, … |
| **SignalSema 0x23be40** | stub, syscall 0x42 | 53 call sites across the CD/file layer (`_sceCd_*_prechk`, `get_iob`, `sceCdMmode`, `sceMcOpen`, …) |
| **iSignalSema 0x23be50** | stub, syscall −0x43 | interrupt-context signallers: `_sceCd_cd_callback`, `_sceFs_Rcv_Intr`, `_request_end`, `CB_DelayTh`, `iWakeupThread` |
| **WaitSema 0x23be60** | stub, syscall 0x44 | 18 call sites (`_sceFsWaitS`, `get_iob`, `new_iob`, `sceCdDelayThread`, `sceClose`, …) |
| **ReferThreadStatus 0x23bdc0** | stub, syscall 0x30 | `RunNextModule`, `_sceCd_ncmd_prechk`, `_sceCd_scmd_prechk`, `sceCdSearchFile` |
| **ReferSemaStatus 0x23be90** | stub, syscall 0x47 | `RunNextModule` |
| **SleepThread 0x23bdd0** | stub, syscall 0x32 | `mcDelayThread` 0x2285a0 **tail-jumps** it (`j 0x23bdd0` at 0x2285dc) — the mcman delay thread behind every memory-card op |
| **isceSifSetDma 0x23bfc0** | stub, syscall −0x77 | `_sceSifSendCmd` (every SIF RPC packet — CD commands) |
| **isceSifSetDChain 0x23bfe0** | stub, syscall −0x78 | `_sceSifCmdIntrHdlr` (the SIF cmd interrupt) |

The sema/CD layer is reached on every level load (on-foot, front end, MP alike) and every
card access. The port's equivalents: the async `Job` worker thread (async_load.rs) and the
synchronous host file reads in `nf-formats::disc` — one thread needs no semaphore, coherent
host memory needs no flush.

## 3. The launch and boot prep

**DisableIntc 0x23c4b0** (104 B: masks an INTC channel — real code, not a stub) is called
by `psiLaunchDriving` 0x120af0 (the DRIVING exec jump) and `sceGsResetGraph` [M]. The port's
launch site is the PTP handover — `crates/nf-game/src/ptp.rs` (psiLaunchDriving's copy +
jump documented there); a host process exec needs no interrupt masking. Cited there.

## 4. The heap and console seams (newlib)

`psiMem_Init`'s three `malloc`s at boot grow the newlib heap through
`malloc_extend_top`/`_malloc_trim_r` → `_sbrk_r` → **sbrk 0x23c298** (172 B), which bounds
itself with **EndOfHeap 0x23be10** (stub, syscall 0x3e) [M caller scan + sbrk decomp]. The
later stdio buffers (`__smakebuf` → `_calloc_r` → `malloc`) do the same. The port
allocates with the host allocator — no heap emulation to port.

The console hooks: **write 0x23c190** (fd 1/2 → `sceTtyInit`/`sceTtyWrite`), **isatty
0x23c348** (always 1) and **fstat 0x23c350** (character-device bit 0x2000) are reached
through the C standard streams when a printf-family call fires: `printf` has 29 call sites
(the SFXUpdate 0x1156c0:13 "BATCH_BUFFER_SIZE exceeded!" overflow warning, `_ErrMessage`'s
error path, `_send_to_iop`, the sceGs debug prints) → `vfprintf` → `_vfprintf_r` →
`__swsetup` → `__smakebuf` (fstat + isatty) → `__sfvwrite` → the stream's write slot →
`_write_r` → `write` [M caller scan]. Whether any fires in a normal session is not
statically provable (the sites are error paths) — the port's equivalent is the host's
stderr/stdout logging (`eprintln!` at the load-failure and dev paths). **Deci2Call
0x23c020** (stub, syscall 0x7c) is the PS2 debug console: `PS2FreshBoot` 0x120ab0 (from
`main`) calls `scePrintf` **unconditionally** at every fresh boot [M 0x120ab0:6] →
`deci2Putchar` → `kputs` → `Deci2Call`; the CD-layer scePrintfs are additionally guarded
by `SCE_CD_debug` (init 0 [D .word at 0x2e4f90]). Cited with the OS-seam block in
`crates/nf-game/src/lib.rs`'s module doc.

## 5. Never reached

See `docs/port-map/unreached.md` for the three:

- **GetMemorySize 0x23c030** (stub, syscall 0x7f) — its only path is `Exit` →
  `TerminateLibrary` → `InitTLB` → `GetMemorySize`, and `main` 0x109080 never returns (the
  frame loop ends in `goto LAB_001092a8` [M 0x109080 tail]); the console is reset, never
  exited [M caller scan].
- **read 0x23c210** — stdin-only (fd 0 → `sceTtyRead`); no read-side stdio user exists in
  the whole decompile (no `scanf`/`fread`/`fgets`/`getchar` callers [M grep]; `__sread` is
  wired into the std FILE slot by `std` 0x239f30 ← `__sinit` but never exercised).
- **lseek 0x23c290** (8 B: `li $v0, −1; jr`) — its callers `_lseek_r` ← `__sseek`/`__swrite`
  serve seekable/append streams, of which the game has none [M caller scan + no fopen/
  fseek users].

## Test coverage

No behaviour was ported — the 25 are OS-seam functions whose jobs the host facilities
already do (frame gate, worker thread, coherent memory, host allocator, host console).
Everything above is citation + evidence; the pass/fail suite that guards the cited sites
already exists. The reachability evidence itself (the caller maps) is the capstone scan
documented here and is not assertable from Rust.
