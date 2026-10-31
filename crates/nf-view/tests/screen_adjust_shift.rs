//! Screen Adjust's offset reaches the rendered frame, in the game's direction and limits, and rides
//! the option file.
//!
//! Evidence (docs/systems/screen-adjust.md §3/§5): `VBlankInt` 0x124088:13-17 writes
//! `DISPLAY2 = (PS2ScreenXPos + 0x27c) | (PS2ScreenYPos + 0x32) << 12 | 0x1bf9ff02000000` [M] —
//! MAGH 5 × 512 = 2560 VCK per line (DW 0x9ff), 448 lines (DH 0x1bf); `psiAdjustScreenPos` 0x10b890
//! steps 12 / 4 and clamps ±0x1b0 / ±0x30 [M]; `Scroll_Update` 0x221b20 gives ◀ (action 0x1c) and
//! ▲ (0x1a) the 0x40 arm (value − 1) [M], which `C_SBSCREEN_Handler` 0x217b08 feeds to
//! psiAdjustScreenPos [M]; DISPLAY's DX/DY are the display area's position on the monitor, so a
//! smaller value moves the picture left / up [S GS manual]. `LS_MakeGlobalSettings` 0x1cd9d0 /
//! `LS_LoadGlobalSettings` 0x1cdbc8 save and restore X/Y with the AV values [M].

use nf_view::gpu::screen_adjust_viewport;
use nf_view::menu::{Action, FrontEnd, Page, OPT_SCREEN_X, OPT_SCREEN_Y};

const T: [f32; 2] = [1280.0, 720.0];

fn shift(fe: &FrontEnd) -> [f32; 4] {
    let (fx, fy) = fe.screen_offset_frac();
    screen_adjust_viewport(T, None, [fx, fy]).unwrap_or([0.0, 0.0, T[0], T[1]])
}

/// The press direction is the picture's direction: ◀ moves the frame left, ▲ up, ▶ right, ▼ down.
/// Sister: the zero offset keeps the pass's own viewport (None) — so a renderer that ignored the
/// offset, or flipped its sign, fails the signed asserts below.
#[test]
fn presses_move_the_frame_the_arrow_way() {
    assert_eq!(screen_adjust_viewport(T, None, [0.0, 0.0]), None, "no offset, no band: nothing set");
    let band = [160.0, 0.0, 960.0, 720.0];
    assert_eq!(screen_adjust_viewport(T, Some(band), [0.0, 0.0]), Some(band), "the cut-scene band stands unshifted");

    let mut fe = FrontEnd::new();
    for _ in 0..8 { fe.tick(); }
    fe.enter_page_for_test(Page::ScreenAdjust);
    fe.screen_adjust_open();
    assert!(fe.action(Action::Left));
    let v = shift(&fe);
    assert!(v[0] < 0.0 && v[1] == 0.0, "◀ = X − 12 = DX − 12: the frame moves left ({v:?})");
    assert!(fe.action(Action::Up));
    let v = shift(&fe);
    assert!(v[0] < 0.0 && v[1] < 0.0, "▲ = Y − 4 = DY − 4: the frame moves up ({v:?})");
    for _ in 0..2 { assert!(fe.action(Action::Right)); assert!(fe.action(Action::Down)); }
    let v = shift(&fe);
    assert!(v[0] > 0.0 && v[1] > 0.0, "▶ / ▼ move it right / down ({v:?})");
    assert_eq!((v[2], v[3]), (T[0], T[1]), "the shift never resizes the frame");
}

/// The amounts: X/2560 and Y/448 of the 4:3 picture at the target's height (the HUD's pixel shape).
/// One X step (12) on a 720-tall target = 12/2560 · 960 = 4.5 px; one Y step (4) = 4/448 · 720 px;
/// the limits ±432 / ±48 = ±162 px / ±77.14 px. Sister: the clamp — 40 presses past the limit
/// land on the same viewport as 36 (432/12) and 12 (48/4).
#[test]
fn the_shift_matches_display2_units_and_limits() {
    let mut fe = FrontEnd::new();
    fe.screen_adjust(1, 1);
    let v = shift(&fe);
    assert!((v[0] - 4.5).abs() < 1e-4, "one X step = 4.5 px at 720 tall ({})", v[0]);
    assert!((v[1] - 4.0 / 448.0 * 720.0).abs() < 1e-4, "one Y step ({})", v[1]);
    let mut a = FrontEnd::new();
    for _ in 0..36 { a.screen_adjust(1, 0); }
    for _ in 0..12 { a.screen_adjust(0, 1); }
    let mut b = FrontEnd::new();
    for _ in 0..40 { b.screen_adjust(1, 1); }
    assert_eq!((a.screen_x, a.screen_y), (0x1b0, 0x30));
    assert_eq!(shift(&a), shift(&b), "the clamp holds the frame at the limit");
    let v = shift(&a);
    assert!((v[0] - 162.0).abs() < 1e-3 && (v[1] - 48.0 / 448.0 * 720.0).abs() < 1e-3, "{v:?}");
    let mut c = FrontEnd::new();
    for _ in 0..40 { c.screen_adjust(-1, -1); }
    let v = shift(&c);
    assert!((v[0] + 162.0).abs() < 1e-3 && (v[1] + 48.0 / 448.0 * 720.0).abs() < 1e-3, "{v:?}");
    // The band (a cut-scene) rides the same shift.
    let band = [160.0, 0.0, 960.0, 720.0];
    let (fx, fy) = a.screen_offset_frac();
    let s = screen_adjust_viewport(T, Some(band), [fx, fy]).unwrap();
    assert!((s[0] - 322.0).abs() < 1e-3 && (s[1] - v[1].abs()).abs() < 1e-3 && s[2] == 960.0, "{s:?}");
}

/// The option file carries the offset: save → load restores it (the game's global settings blob
/// [M 0x1cd9d0 / 0x1cdbc8]); a hand-edited value past the limits clamps on load; a file without
/// the keys boots 0/0. Sister: a different saved offset loads differently.
#[test]
fn the_offset_saves_and_loads_with_the_options() {
    let dir = std::env::temp_dir().join(format!("nf-screen-adjust-{}", std::process::id()));
    let path = dir.join("settings.toml");
    let mut fe = FrontEnd::new();
    fe.screen_adjust(-3, 2);
    assert_eq!((fe.screen_x, fe.screen_y), (-36, 8));
    nf_view::settings::save_opts(&path, &fe.opts_with_screen()).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("screen_x = -36") && text.contains("screen_y = 8"), "{text}");
    let (opts, warn) = nf_view::settings::load_opts(&path);
    assert!(warn.is_none());
    let mut back = FrontEnd::new();
    back.set_opts(opts);
    assert_eq!((back.screen_x, back.screen_y), (-36, 8), "the load restores the offset");
    assert_eq!(back.opts.get(&0x1000_0135), fe.opts.get(&0x1000_0135), "the rows still load");

    let mut other = FrontEnd::new();
    other.screen_adjust(5, -1);
    nf_view::settings::save_opts(&path, &other.opts_with_screen()).unwrap();
    let (opts, _) = nf_view::settings::load_opts(&path);
    assert_eq!((opts.get(&OPT_SCREEN_X).copied(), opts.get(&OPT_SCREEN_Y).copied()), (Some(60), Some(-4)), "sister: another offset loads as itself");

    std::fs::write(&path, "[av]\nscreen_x = 9999\nscreen_y = -9999\n").unwrap();
    let (opts, _) = nf_view::settings::load_opts(&path);
    let mut c = FrontEnd::new();
    c.set_opts(opts);
    assert_eq!((c.screen_x, c.screen_y), (0x1b0, -0x30), "hand edits clamp to psiAdjustScreenPos' limits");

    std::fs::write(&path, "[av]\n0x0194 = 50\n").unwrap();
    let (opts, _) = nf_view::settings::load_opts(&path);
    let mut d = FrontEnd::new();
    d.screen_adjust(1, 1);
    d.set_opts(opts);
    assert_eq!((d.screen_x, d.screen_y), (0, 0), "no keys: the boot's 0/0");
    let _ = std::fs::remove_dir_all(&dir);
}
