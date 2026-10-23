# Launcher (`NightfirePort.exe`)

The player-facing entry point: the only thing a player supplies is the disc image.

- **First run:** "Select disc image" (the system open dialog) or drop the `.iso` on the window. The image is checked
  by `nf_formats::identity` (volume id `NIGHTFIRE`, `SYSTEM.CNF` boots `SLUS_205.79` with `VMODE = NTSC`, the boot
  ELF and `ACTION.ELF` SHA-1s from build-identity.md, not truncated) — a few MB read, ~7 ms on the retail disc
  (*measured*, `tests/identity_disc.rs`). A wrong image gets a plain-language reason (another game, PAL release,
  incomplete copy, patched executable).
- **Remembered** in `launcher.toml` beside `settings.toml` (`%APPDATA%\NightfirePort`; `NF_CONFIG_DIR` overrides).
  Not a section of `settings.toml`: that parser treats any non-integer value as corruption and `save_opts` rewrites
  the file.
- **Later runs:** the disc card (file name, serial, region, version, size, READY) and PLAY. Enter = PLAY,
  O = change disc, Esc = quit. `NightfirePort <iso>` (or dragging an image onto the exe) checks and remembers it.
- **PLAY** starts `nf-view.exe <iso>` from the launcher's folder with `CREATE_NO_WINDOW` (no console), stdout/stderr
  to `nf-view.log` in the config dir, and closes the launcher when the game's window appears (60 s cap). If the game
  exits first, the launcher stays open with the log's last line.

Why a separate binary: `nf-view` is a console program (CLI tools, logs), so double-clicking it always opens a
console; the launcher is GUI-subsystem. winit allows one event loop per process, so the game runs as a fresh process.

Drawing: CPU canvas (`crates/nf-view/src/launcher/canvas.rs`, `ab_glyph` + the system font — Bahnschrift on
Windows 10+, then Segoe UI / Arial / DejaVu Sans) blitted through one wgpu texture. Nothing from the disc is drawn;
the gun-barrel art is procedural. Headless render of any state:

```text
NightfirePort --shot out.png [--state first|ready|error|drop|starting|hover] [--iso <path>] [--scale S] [--t SEC]
```
