//! The C-library / PS2-SDK layer the game's ACTION.ELF ships (the whole-ELF scan's
//! never-mentioned block, task unported-misc-8). Every function here is REACHED in the shipped
//! game [M callers, jal census over the ELF unless noted] and every one is replaced on this port
//! by a host/rust-std equivalent — there is no game logic in this layer, so the port carries the
//! citations, not ports. `crates/nf-game/src/trig.rs` carries the libm half (fmodf, fabsf,
//! isnanf, finitef, scalbnf, floorf) next to the libm ports the same layer backs.
//!
//! # stdio / printf family
//!
//! - `sprintf` 0x2366c8 → `vfprintf` 0x237110 → `_vfprintf_r` 0x237188 (jal sites 0x236654 /
//!   0x23671c / 0x2370b4 [M]) — the game's HUD/label/score text formatting runs through it
//!   (71 decompiled `sprintf` callers, e.g. `HUD_UpdateHealthPane` 0x19b9c8, `MP_GetPlayerScore`
//!   0x189730, `Mission_MonitorObjectives` 0x1827d8 [M]). REACHED every frame with text on
//!   screen. The port's equivalent is Rust's `format!` at those sites (hud, MP score lines,
//!   mission banners) — bit-exact integer formatting is not needed: the formatted strings feed
//!   the draw side's textures.
//! - `std` 0x239f30: newlib's standard-streams initializer, called from `__sinit` 0x239fa0 at
//!   libc bring-up (3 jal sites 0x239fd8/0x239fec/0x23a000 [M]) — REACHED at boot. The port's
//!   stdout/stderr need no initialization.
//! - `fflush` 0x239e28: `__sbprintf` 0x237058's buffer flush (jal 0x2370c8 — sprintf's in-memory
//!   stream) plus the fclose family (0x23a4c0 gcc2 region) [M] — REACHED via every sprintf. The
//!   port's `format!` has no stream to flush.
//! - `localeconv` 0x23a950: `_vfprintf_r` 0x237188 reads the decimal point at its top — line 65
//!   of the decompile, BEFORE the format dispatch [M] — so every sprintf reaches it. The port's
//!   `format!` always writes '.'.
//!
//! # The never-executed float branch (recorded unreached — see docs/port-map/unreached.md)
//!
//! `_vfprintf_r`'s `%f/%e/%g` branch calls `isinf` 0x23b788 (jal 0x2375d0), `isnan` 0x23b7d0
//! (jal 0x237608) and `cvt` 0x2386a0 (jal 0x237644) → `_dtoa_r` 0x238c58 → `quorem` 0x238a40
//! [M chain]. The disc's entire .rodata carries exactly ONE float format string:
//! `"%.2f %.2f %.2f\nFPS %d\n"` — `HUD_UpdateHealthPane` 0x19b9c8:265-268 and
//! `HUD_UpdateMPHealthPane` 0x19b630 print it only while `switch_SHOW_COORDS != 0` [M], the
//! debug-coords byte only the never-opened per-level cheat hub writes (docs/port-map/
//! unreached.md, the C_CHCH* entries) [D census: no other float format]. So the branch never
//! executes on the shipped game. `isinf`/`isnan`/`quorem` (and cvt/_dtoa_r behind them) are
//! recorded there, not ported.
//!
//! # `exit` 0x2358b0 and `index` 0x235960
//!
//! - `exit` 0x2358b0: the SDK's failure paths — `sceMtapInit` 0x226b68 calls it ×5 (the network
//!   adapter init's error arms, jal sites 0x226bd8..0x226d78 [M]) and `_exit` 0x224350 plus
//!   `sceCdInit`/`cdvd_exit` mention it in the decompile [M]. Reached only when boot-time SDK
//!   init fails — a path the shipped discs never take (the game ships). The port: the host
//!   process's own `std::process::exit`; no site needs citing beyond this module.
//! - `index` 0x235960 = `strchr`: `strFileOpen` 0x108cc0 (jals 0x108ce4/0x108da0 — the game's
//!   filename/extension parsing on every file load [M]) and the sprite-list/credits string walks
//!   (`psiDrawSprites` 0x120e68, `CreatePointerSpriteList` 0x11a160, `P_CREDITS_Handler`
//!   0x2152b0) [M]. REACHED on every load. The port's file loading (nf-formats `disc`/`iso9660`)
//!   uses Rust `str::find`/`split` for the same job.
//!
//! # The PS2 kernel stubs (all 16-byte syscall wrappers [M ELF bytes])
//!
//! Every one is REACHED in the boot/SDK path and every one is the host's job on this port:
//!
//! | stub | caller [M jal census] | port equivalent |
//! |---|---|---|
//! | `SetGsCrt` 0x23bc50 | `sceGsResetGraph` 0x225430:30,41 (decomp; the video-mode set at boot) | the window/surface the viewer's shell opens |
//! | `AddIntcHandler` 0x23bc90 | `main` 0x109080 ×2 (0x10911c/0x109134 — the VBlank/interrupt hooks) | the fixed 60 Hz tick loop (no interrupts) |
//! | `RemoveIntcHandler` 0x23bca0 | `psiLaunchDriving` 0x120af0 ×2 + `sceGsResetGraph` 0x225430 | level transitions re-wire nothing on the port |
//! | `AddDmacHandler` 0x23bcb0 | `sceSifInitCmd` 0x23d628 (0x23d7b0 — the SIF DMA bring-up) | the host file reads |
//! | `AddDmacHandler2` 0x23bcc0 | `_doCSC2` 0x230430 + `_csc_storeRefImage` 0x2306e8 (the CD scratch DMA) | same |
//! | `RemoveDmacHandler` 0x23bcd0 | `_doCSC2`/`_csc_storeRefImage` teardown + `sceSifExitCmd` 0x23d8a8 | same |
//! | `SetAlarm` 0x23bd20 | `mcDelayThread` 0x2285a0 + `sceCdDelayThread` 0x230b98 (the memcard/CD delay threads) | the port's single-threaded sim has no alarm timers |
//! | `DeleteThread` 0x23bd50 | `RunNextModule` 0x120008 (0x120128 — the module loader teardown) | no runtime threads |
//! | `TerminateThread` 0x23bd70 | `RunNextModule` 0x120008 (0x120120) | no runtime threads |
//! | `GetThreadId` 0x23bdb0 | `mcDelayThread` 0x2285a0 + `sceCdInit` 0x2316a8 (0x2316f4) | single-threaded: the id is constant |
//!
//! `RunNextModule` 0x120008's thread teardown was also misc-2's subject (its port note lives in
//! the boot-flow docs); the stubs themselves stay here.

/// The one float format string on the disc's .rodata [D census — the test below re-scans the
/// disc image] — the debug-coords HUD line, gated on `switch_SHOW_COORDS` [M 0x19b9c8:265-268].
pub const ONLY_FLOAT_FORMAT: &str = "%.2f %.2f %.2f\nFPS %d\n";

#[cfg(test)]
mod tests {
    use super::*;

    // [D] the .rodata census the never-executed-float-branch verdict rests on: exactly one float
    // format string on the whole disc, the debug-coords one.
    #[test]
    fn the_disc_has_exactly_one_float_format_string() {
        let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
        let img = nf_formats::iso9660::IsoImage::open(&iso).unwrap();
        let elf = img.read_file(img.find("/ACTION.ELF").expect("ACTION.ELF on disc")).unwrap();
        // .rodata: vaddr 0x2e8180, file off 0x1e2080, size 0x244c0 [M section table].
        let ro = &elf[0x1e2080..0x1e2080 + 0x244c0];
        let mut floats: Vec<String> = Vec::new();
        for chunk in ro.split(|&b| b == 0) {
            if let Ok(s) = std::str::from_utf8(chunk) {
                if s.len() >= 4
                    && s.chars().all(|c| c.is_ascii_graphic() || c == ' ' || c == '\n' || c == '\t')
                    && s.contains(['f', 'e', 'E', 'g', 'G'])
                {
                    // a printf conversion spec scan
                    let bytes = s.as_bytes();
                    for (i, &b) in bytes.iter().enumerate() {
                        if b == b'%' {
                            let tail = &s[i + 1..];
                            let conv = tail.chars().find(|c| c.is_ascii_alphabetic());
                            if matches!(conv, Some('f' | 'e' | 'E' | 'g' | 'G')) {
                                floats.push(s.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
        floats.sort();
        floats.dedup();
        assert_eq!(floats, vec![ONLY_FLOAT_FORMAT.to_string()], "the disc's float formats");
        // Sister (can fail): the integer formats are everywhere — the census is not vacuous.
        assert!(ro.windows(2).any(|w| w == b"%d"), "the rodata carries %d formats too");
    }

    // [M] the stub layer's shape: 16 bytes each (one lw/jr pair) at their symtab addresses.
    #[test]
    fn the_kernel_stubs_are_sixteen_byte_syscall_wrappers() {
        let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
        let img = nf_formats::iso9660::IsoImage::open(&iso).unwrap();
        let elf = img.read_file(img.find("/ACTION.ELF").expect("ACTION.ELF on disc")).unwrap();
        // .text vaddr 0x107100 at file 0x1000 [M section table].
        let at = |va: u32| &elf[0x1000 + (va - 0x107100) as usize..][..16];
        let mut numbers = Vec::new();
        for (name, va) in [
            ("SetGsCrt", 0x23bc50u32), ("AddIntcHandler", 0x23bc90), ("RemoveIntcHandler", 0x23bca0),
            ("AddDmacHandler", 0x23bcb0), ("AddDmacHandler2", 0x23bcc0), ("RemoveDmacHandler", 0x23bcd0),
            ("SetAlarm", 0x23bd20), ("DeleteThread", 0x23bd50), ("TerminateThread", 0x23bd70),
            ("GetThreadId", 0x23bdb0),
        ] {
            let w = |k: usize| u32::from_le_bytes(at(va)[k * 4..k * 4 + 4].try_into().unwrap());
            // The stub shape [D ISO bytes]: `addiu $v1, $zero, N; syscall; jr $ra; nop` —
            // opcode 0x09 with rt = $v1, then 0x0000000c, then 0x3e00008, then 0.
            assert_eq!(w(0) >> 26, 0x09, "{name}: first word {:#x} is not addiu", w(0));
            assert_eq!((w(0) >> 16) & 0x1f, 3, "{name}: the target is not $v1");
            numbers.push(w(0) & 0xffff);
            assert_eq!(w(1), 0x0000_000c, "{name}: no syscall");
            assert_eq!(w(2), 0x03e0_0008, "{name}: no jr $ra");
            assert_eq!(w(3), 0, "{name}: delay slot not zero");
        }
        // Sister (can fail): the stubs are distinct syscalls, not ten copies of one number.
        assert_ne!(numbers[0], numbers[6], "SetGsCrt and SetAlarm load different numbers");
    }
}
