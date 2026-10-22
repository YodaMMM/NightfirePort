# PS2 SDK runtime glue in ACTION.ELF (libkernel/libc/debug) — unported-misc-10

The 24 never-mentioned functions in misc bucket 10/10 are all PS2 SDK runtime glue: INTC/DMAC
enable-disable, the SIF RPC request handlers, the DECI2 debug tty, the libc stdio FILE table,
the exit/TLB teardown and the kernel-syscall registration tables. This doc maps each to the
evidence chain that decides whether the shipped game executes it, and to the port site that
does the same job. Method: name-grep over `work/decomp/ACTION.ELF/` plus a machine-level scan
of `work/elf/ACTION.ELF` (every `j`/`jal` opcode resolved, every 4-byte data word, every
`lui`+`addiu`/`ori` address-building pair — the same method as unported-misc-7).

## The boot chain (what actually runs, in order)

1. `zerobss` 0x2242a8 (crt0, disposed in unported-misc-7) zeroes BSS, `syscall` 60/61, calls
   `main` (jal 0x22433c [M scan]) and then `Exit(uVar3)` (jal 0x224344 [M scan]).
2. `main` 0x109080 [M decompile] — full hardware init. It calls, in order: `sceSifInitRpc(0)`
   (jal 0x1090e8 [M scan]), `sceCdInit`, `FS_Init`, `AddIntcHandler(5, 0x123e90)` /
   `AddIntcHandler(2, 0x124088)` (VIF1 + VBlank), `PS2FreshBoot` (jal 0x109144),
   `InitaliseMemoryCard`, `CreateDB(0)`, `system_input_init`,
   `EnableIntc(2)` / `EnableIntc(5)` (jals 0x10927c, 0x109284 [M scan]), then the infinite
   frame loop `GameFlow_Main` (main never returns [M decompile 0x1092a8..0x109414]).
3. `sceSifInitRpc` 0x23dcc8 [M decompile]: `DIntr()` … `EIntr()` (unconditional, first and
   mid-body), `sceSifInitCmd()` → `EnableDmac` (jal 0x23d7c0 [M scan]), and registers the four
   SIF request handlers:
   `sceSifAddCmdHandler(0x…08, _request_end 0x23df68)`, `(0x…09, _request_bind 0x23e0d0)`,
   `(0x…0a, _request_call 0x23e2c0)`, `(0x…0c, _request_rdata 0x23e020)` [M decompile 0x23ddxx].
4. `PS2FreshBoot` 0x120ab0 [M decompile head]: `scePrintf("NFSV = 0x%\n" @0x2f3af0 [D ELF
   data], iRam00100960)` unconditionally, returns `iRam00100960 != 1`; its real body also
   `jal RunNextModule` at 0x120b60 [M scan; next decompiled function 0x120b88].
5. `RunNextModule` 0x120008 [M decompile]: `sceOpen`/`sceRead`/`sceLseek`/`sceClose` — the
   stdio-style module loader (checks the `\x7fELF` magic at 0x1e20000).
6. `scePrintf` 0x23d570 [M decompile]: saves `_putchar` (OBJECT 0x2e7a58 [M symtab], initial
   value = `serialPutchar` 0x23cd40 [D ELF data word at 0x2e7a58]), sets `_putchar =
   deci2Putchar` (imm pair at 0x23d574 [M scan]), runs `_printf`, restores.
7. `_printf` 0x23cf70 [M decompile] emits every literal char through `(*(code *)_putchar)()`
   — so the boot banner's 9 literal chars "NFSV = 0x" execute `deci2Putchar` 9×. The trailing
   `%\n` hits _printf's unknown-conversion default branch, which skips 2 chars WITHOUT
   emitting [M decompile 0x23d508 default path] — no newline, so `kputs` does not flush.
8. `system_input_init` 0x10c418 [M decompile]: `sceMtapInit()` unconditionally first; its 5
   `exit(-1)` jals (0x226bd8..0x226d78 [M scan]) are IOP bind-failure branches that do not
   fire on retail hardware (the game runs).
9. `psiPreGame_Run` 0x109520 [M decompile]: on `PS2FirstGameRun != 0` calls `CreateDB(1)` →
   `sceGsSyncV(0)` (jal 0x10b278 in CreateDB [M scan]) → `VSync`/`VSync2` → more `DIntr`/
   `EIntr`. `PS2FirstGameRun` is written only by psiPreGame_Run (to 0) [M grep] — whether it
   is non-zero on first run is set in .bss/.data, not provable statically [S unverified].

## Dispositions — all 24

| Address | Name | Outcome | Evidence |
|---|---|---|---|
| 0x23c518 | EnableIntc | cited-as-replaced — INTC mask enable for the VBlank/VIF1-driven frame loop; the port's fixed 60 Hz tick replaces interrupt-driven timing | jal 0x10927c/0x109284 in `main` [M scan+decompile] |
| 0x23c580 | DisableDmac | cited-as-replaced — DMAC channel mask; the port has no DMA channels (direct uploads + direct SFX batch execution) | jal 0x23d8b0 in `sceSifExitCmd` ← `RunNextModule` 0x1200fc [M scan]; also `_doCSC2`/`_csc_storeRefImage` (FMV, misc-7) |
| 0x23c5e8 | EnableDmac | cited-as-replaced — same | jal 0x23d7c0 in `sceSifInitCmd` ← `sceSifInitRpc` ← `main` [M decompile chain] |
| 0x23c650 | kCopy | unreached — registered into `SysEntry` 0x2e7a10 (syscall 0x5a entry, data word 0x2e7a14) and never invoked | data word 0x2e7a14 [M scan+symtab]; no jal; no `syscall 0x5a` anywhere in ACTION.ELF [M syscall scan] |
| 0x23c698 | iWakeupThread | cited-as-replaced — IOP alarm + SIF RPC thread wakeups; the port's polling card-flow machine and direct batch execution have no threads to wake | jal 0x228580 in `mcHearAlarm` (misc-7 site) + 0x23e33c in `_request_call`, registered by `sceSifInitRpc` [M scan+decompile] |
| 0x23c738 | QueueInit | unreached — DECI2 tty queue init; only via `sceTtyInit` ← `write()` (fd 1/2), whose only game-side trigger is stdio printf glue; on retail `sceDeci2Open(0x210,…)` has no DECI2 host | jal 0x23cc30 in `sceTtyInit` [M scan+decompile]; `write` 0x23c190 jal'd only from libc blob 0x23b978 ← `__swrite` (0 refs) [M scan] |
| 0x23c760 | QueuePeekWriteDone | unreached — same tty queue, advance in the DECI2 handler `sceTtyHandler` (registered by `sceTtyInit`, needs a DECI2 host) | jal 0x23c8c8 in `sceTtyHandler` [M scan] |
| 0x23c7a0 | QueuePeekReadDone | unreached — same, read side (`sceTtyRead` ← `read()`, no game caller) | jal 0x23cb40 in `sceTtyRead` [M scan] |
| 0x23cc58 | kputchar | unreached — SIO serial putchar; only `serialPutchar` calls it, and `serialPutchar` is the never-used default of `_putchar` | jals 0x23cd50/0x23cd60/0x23cd6c in `serialPutchar` [M scan]; `_putchar` 0x2e7a58 init = serialPutchar [D ELF data]; `_printf` runs only after scePrintf swaps it |
| 0x23cc90 | deci2Putchar | cited-as-reached+replaced — the printf char sink; executes at boot (9 banner chars) | chain in § boot; port site: `crates/nf-view/src/lib.rs` diagnostics (`eprintln!`) |
| 0x23cd40 | serialPutchar | unreached — default `_putchar` value, never invoked (every _printf path swaps to deci2Putchar first) | data word 0x2e7a58 [M symtab+D]; 0 jals to it [M scan] |
| 0x23cd78 | ftoi | unreached — %f-only print path (`printfloat` 0x23ce08 jal 0x23cf14); the %f gate is dead on retail (menus-6, misc-7) | [M scan]; misc-7 report § dp-family |
| 0x23d538 | kprintf | unreached — debug printf via tty handler + printfloat + InitTLB32MB error branches — all dead on retail | jals 0x23c844..0x23c950 (sceTtyHandler), 0x23cf20..0x23cf68 (printfloat), 0x23fe80.. (TLB error) [M scan] |
| 0x23dab0 | isceSifSendCmd | cited-as-replaced — EE→IOP SIF command send inside the RPC handlers; the port calls its IOP-side systems directly | jals 0x23e078 in `_request_rdata`, 0x23e178 in `_request_bind`; both registered by `sceSifInitRpc` [M scan+decompile] |
| 0x23e5f8 | new_iob | cited-as-reached+replaced — FILE-table slot alloc for `sceOpen`; runs at boot (module load) | jal 0x23ee68 in `sceOpen` ← `RunNextModule` 0x120030 ← `PS2FreshBoot` 0x120b60 ← `main` [M scan chains]; also `strFileOpen` (FMV) |
| 0x23e680 | get_iob | cited-as-reached+replaced — FILE lookup for sceClose/sceLseek/sceRead/sceIoctl; same boot chain | jals 0x23f078/0x23f208/0x23f448/0x23f6b8 [M scan] |
| 0x23fbb8 | DIntr | cited-as-reached+replaced — interrupt disable; runs at boot (sceSifInitRpc head) and every VSync/FMV SDK entry | jal 0x23c060/0x23c0b4 in `VSync` ← `sceGsSyncV` ← `CreateDB(1)` [M]; 36 sites, all SDK libs [M scan]; `sceSifInitRpc` body [M decompile] |
| 0x23fc00 | EIntr | cited-as-reached+replaced — same | 41 sites, same chains [M scan] |
| 0x23fc18 | TerminateLibrary | unreached-by-replacement — `j InitTLB` (8 bytes); only via `Exit`/`_exit`, which never run on retail | jal 0x23fc34 in `Exit`; 0 refs to Exit on live paths [M scan] |
| 0x23fc28 | Exit | unreached-by-replacement — post-main teardown (crt0 after the infinite loop) + error branches | jal 0x224344 in `zerobss` (main never returns [M decompile]), 0x224350 in `_exit` ← `exit` ← `sceMtapInit` bind-failure branches, 0x23feac/0x23ff14/0x23ff90 in InitTLB32MB overflow checks [M scan] |
| 0x23fd78 | kputs | unreached — DECI2 line flush; needs a `\n` through _printf, and every live-adjacent \n site is a non-firing error/debug branch | jal 0x23ccc8/0x23cd10 in `deci2Putchar` [M scan+decompile]; banner `%\n` swallowed [M 0x23d508 default]; "ReadCount…" sites are error branches; `SCE_CD_debug` = 0 [D ELF 0x2e4f90] |
| 0x23fda8 | kCopy | unreached — second copy, same SysEntry registration (0x5a, data word 0x2e7e74) | [M scan+symtab]; same syscall-usage scan |
| 0x23fdf8 | InitTLB | unreached-by-replacement — boot/exit TLB setup; only via `TerminateLibrary` (which is only via Exit) | jal 0x23fc18 [M scan] |
| 0x23fe38 | InitTLB32MB | unreached-by-replacement — the 32 MB TLB map (kernelTLB/defaultTLB tables 0x2e7eb0/0x2e7f80); its own `Exit(1)` branches are TLB-count overflow checks | jal 0x23fe14 in `InitTLB` [M scan]; kprintf error branches [M decompile] |

Where cited, the citation carries the function name + address in `crates/*/src` so the
whole-ELF scan sees it: EnableIntc/DIntr/EIntr/EnableDmac/DisableDmac (`nf-formats/ps2gfx.rs`
module doc), iWakeupThread + isceSifSendCmd (`nf-view/menu.rs` memcard-save-flow,
`nf-audio/sfx.rs` batch), new_iob/get_iob (`nf-formats/iso9660.rs`), deci2Putchar
(`nf-view/lib.rs` diagnostics). No code behaviour changed (comments + docs only).
