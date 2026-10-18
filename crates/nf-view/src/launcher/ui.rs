//! The launcher screen: its state, layout (logical pixels, 960×560) and painting. Pure functions of the state and
//! the time, so the window and the headless `--shot` path draw the same frame.

use std::path::{Path, PathBuf};

use nf_formats::identity::{self, DiscInfo};

use super::canvas::{ascent, ellipsize_middle, measure, rgb, Canvas, Fonts, Rect, Rgb};

/// The window's logical size.
pub const W: f32 = 960.0;
pub const H: f32 = 560.0;

const BG_TOP: Rgb = rgb(0x0f0e0c);
const BG_BOTTOM: Rgb = rgb(0x050505);
const GOLD: Rgb = rgb(0xc9a14a);
const GOLD_HI: Rgb = rgb(0xf1d58c);
const GOLD_LO: Rgb = rgb(0xa47e31);
const GLOW: Rgb = rgb(0xffc76a);
const INK: Rgb = rgb(0x16120a);
const TEXT: Rgb = rgb(0xece7dc);
const DIM: Rgb = rgb(0x8c867a);
const FAINT: Rgb = rgb(0x575249);
const CARD: Rgb = rgb(0x15140f);
const CARD_EDGE: Rgb = rgb(0x2e2a21);
const ALERT: Rgb = rgb(0xe0705e);

/// The right-hand column.
const COL_X: f32 = 468.0;
const COL_W: f32 = 440.0;
const BARREL: (f32, f32, f32) = (232.0, 270.0, 162.0);
const CARD_R: Rect = Rect::new(COL_X, 270.0, COL_W, 100.0);
const PRIMARY_R: Rect = Rect::new(COL_X, 388.0, COL_W, 60.0);
const SECOND_Y: f32 = 476.0;

/// A disc image that passed the identity check.
#[derive(Clone, Debug, PartialEq)]
pub struct Disc {
    pub path: PathBuf,
    pub bytes: u64,
    pub info: DiscInfo,
}

impl Disc {
    /// Validates `path` (nf_formats::identity) into a [`Disc`]; the error is the friendly sentence.
    pub fn check(path: &Path) -> Result<Disc, String> {
        let info = identity::check_path(path, &identity::NIGHTFIRE_NTSC_U).map_err(|e| e.to_string())?;
        let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        Ok(Disc { path: path.to_path_buf(), bytes, info })
    }
    pub fn file_name(&self) -> String { self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() }
    /// `SLUS-20579  ·  NTSC-U  ·  v1.00  ·  2.27 GB`
    pub fn details(&self) -> String {
        let region = if self.info.vmode.eq_ignore_ascii_case("NTSC") && self.info.serial.starts_with("SLUS") { "NTSC-U".to_string() } else { self.info.vmode.clone() };
        let mut s = format!("{}  \u{b7}  {}", self.info.serial, region);
        if !self.info.version.is_empty() { s.push_str(&format!("  \u{b7}  v{}", self.info.version)); }
        s.push_str(&format!("  \u{b7}  {:.2} GB", self.bytes as f64 / (1u64 << 30) as f64));
        s
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Btn { Select, Play, Change, Quit }

#[derive(Clone, Debug, Default)]
pub struct Ui {
    pub disc: Option<Disc>,
    /// The last problem (bad image, launch failure), shown under the buttons.
    pub error: Option<String>,
    pub hover: Option<Btn>,
    pub pressed: Option<Btn>,
    /// A file is being dragged over the window.
    pub drop_hover: bool,
    /// PLAY was pressed and the game is starting.
    pub launching: bool,
    /// Seconds since the launcher opened (animation clock).
    pub t: f32,
}

impl Ui {
    /// The clickable buttons in logical pixels.
    pub fn buttons(&self) -> Vec<(Btn, Rect)> {
        let mut v = vec![(if self.disc.is_some() { Btn::Play } else { Btn::Select }, PRIMARY_R)];
        if self.disc.is_some() { v.push((Btn::Change, Rect::new(COL_X - 6.0, SECOND_Y - 18.0, 120.0, 28.0))); }
        v.push((Btn::Quit, Rect::new(COL_X + COL_W - 60.0, SECOND_Y - 18.0, 66.0, 28.0)));
        v
    }
    pub fn hit(&self, lx: f32, ly: f32) -> Option<Btn> {
        if self.launching { return None; }
        self.buttons().into_iter().find(|(_, r)| r.contains(lx, ly)).map(|(b, _)| b)
    }
    /// The action Enter / Space takes.
    pub fn default_action(&self) -> Btn { if self.disc.is_some() { Btn::Play } else { Btn::Select } }
}

/// Splits `text` into lines no wider than `max_w`.
pub fn wrap(font: &ab_glyph::FontVec, size: f32, text: &str, max_w: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let cand = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if measure(font, size, &cand, 0.0) <= max_w || cur.is_empty() { cur = cand; } else { lines.push(std::mem::replace(&mut cur, word.to_string())); }
    }
    if !cur.is_empty() { lines.push(cur); }
    lines
}

/// The static backdrop (gradient, warm light behind the barrel, vignette, a faint grain) at `scale`.
pub fn paint_background(c: &mut Canvas, scale: f32) {
    let (w, h) = (c.w as f32, c.h as f32);
    let (bx, by) = (BARREL.0 * scale, BARREL.1 * scale);
    c.shade(|x, y| {
        let t = y / h;
        let mut col = [0.0; 3];
        for k in 0..3 { col[k] = BG_TOP[k] + (BG_BOTTOM[k] - BG_TOP[k]) * t; }
        // Warm light pooling around the barrel.
        let d = (((x - bx) / scale).powi(2) + ((y - by) / scale).powi(2)).sqrt();
        let pool = (1.0 - d / 520.0).max(0.0).powi(2) * 0.05;
        for k in 0..3 { col[k] += GOLD[k] * pool; }
        // Vignette.
        let (nx, ny) = (x / w - 0.5, y / h - 0.5);
        let v = 1.0 - 0.55 * (nx * nx * 1.2 + ny * ny * 1.6);
        // Grain: a fixed hash per pixel, ±0.6%.
        let n = ((x as u32).wrapping_mul(73_856_093) ^ (y as u32).wrapping_mul(19_349_663)).wrapping_mul(2_654_435_761) >> 24;
        let g = (n as f32 / 255.0 - 0.5) * 0.012;
        [col[0] * v + g, col[1] * v + g, col[2] * v + g]
    });
}

/// Paints the whole frame. `bg` is [`paint_background`]'s output at the canvas size (copied, not redrawn).
pub fn paint(c: &mut Canvas, bg: &Canvas, fonts: &Fonts, ui: &Ui, scale: f32) {
    c.copy_from(bg);
    let s = scale;
    let t = ui.t;

    // ── The barrel ──────────────────────────────────────────────────────────────────────────────────────
    c.barrel(BARREL.0 * s, BARREL.1 * s, BARREL.2 * s, t, GOLD, GLOW);

    // ── Title block ─────────────────────────────────────────────────────────────────────────────────────
    let x = COL_X * s;
    c.text(&fonts.bold, 12.5 * s, x, 92.0 * s, "NIGHTFIREPORT", GOLD, 1.0, 4.2 * s);
    c.text(&fonts.display, 34.0 * s, x - 1.0 * s, 146.0 * s, "007", GOLD, 1.0, 2.0 * s);
    let title_size = 78.0 * s;
    let title_w = measure(&fonts.display, title_size, "NIGHTFIRE", 3.0 * s);
    let title_size = if title_w > COL_W * s { title_size * COL_W * s / title_w } else { title_size };
    c.text(&fonts.display, title_size, x - 3.0 * s, 222.0 * s, "NIGHTFIRE", TEXT, 1.0, 3.0 * s);
    c.hline_fade(x, 242.0 * s, COL_W * s, 1.2 * s, GOLD, 0.85);

    // ── Disc card ───────────────────────────────────────────────────────────────────────────────────────
    let card = CARD_R.scaled(s);
    let rad = 6.0 * s;
    c.fill_rrect(card, rad, CARD, 0.92);
    if ui.drop_hover {
        c.glow_rrect(card, rad, 18.0 * s, GOLD, 0.16);
        c.stroke_rrect(card, rad, 1.6 * s, GOLD, 1.0);
    } else {
        c.stroke_rrect(card, rad, 1.0 * s, CARD_EDGE, 1.0);
    }
    // A small disc glyph at the card's left.
    let (dcx, dcy, dr) = (card.x + 46.0 * s, card.y + card.h * 0.5, 24.0 * s);
    let ready = ui.disc.is_some() && !ui.drop_hover;
    let disc_col = if ready { GOLD } else { FAINT };
    c.stroke_rrect(Rect::new(dcx - dr, dcy - dr, dr * 2.0, dr * 2.0), dr, 1.4 * s, disc_col, 1.0);
    c.stroke_rrect(Rect::new(dcx - dr * 0.62, dcy - dr * 0.62, dr * 1.24, dr * 1.24), dr, 0.8 * s, disc_col, 0.35);
    c.fill_rrect(Rect::new(dcx - 5.0 * s, dcy - 5.0 * s, 10.0 * s, 10.0 * s), 5.0 * s, disc_col, 0.9);
    let tx = card.x + 88.0 * s;
    let tw = card.x + card.w - tx - 18.0 * s;
    if ui.drop_hover {
        c.text(&fonts.bold, 17.0 * s, tx, card.y + 46.0 * s, "Release to use this disc image", GOLD_HI, 1.0, 0.0);
        c.text(&fonts.regular, 13.5 * s, tx, card.y + 70.0 * s, "It will be checked before it is remembered.", DIM, 1.0, 0.0);
    } else if let Some(d) = &ui.disc {
        // READY tag, top right.
        let tag = "READY";
        let tag_w = measure(&fonts.bold, 11.0 * s, tag, 2.4 * s);
        let tag_x = card.x + card.w - 18.0 * s - tag_w;
        c.text(&fonts.bold, 11.0 * s, tag_x, card.y + 30.0 * s, tag, GOLD, 1.0, 2.4 * s);
        c.text(&fonts.bold, 11.0 * s, tx, card.y + 30.0 * s, "DISC IMAGE", FAINT, 1.0, 2.4 * s);
        let name = ellipsize_middle(&fonts.bold, 18.0 * s, &d.file_name(), tw);
        c.text(&fonts.bold, 18.0 * s, tx, card.y + 58.0 * s, &name, TEXT, 1.0, 0.0);
        c.text(&fonts.regular, 13.0 * s, tx, card.y + 81.0 * s, &d.details(), DIM, 1.0, 0.3 * s);
    } else {
        c.text(&fonts.bold, 17.0 * s, tx, card.y + 40.0 * s, "No disc image selected", TEXT, 1.0, 0.0);
        let body = "Choose your 007: Nightfire (USA, SLUS-20579) .iso \u{2014} or drop it onto this window.";
        for (i, line) in wrap(&fonts.regular, 13.5 * s, body, tw).iter().take(2).enumerate() {
            c.text(&fonts.regular, 13.5 * s, tx, card.y + (64.0 + 19.0 * i as f32) * s, line, DIM, 1.0, 0.0);
        }
    }

    // ── Primary button ──────────────────────────────────────────────────────────────────────────────────
    let b = PRIMARY_R.scaled(s);
    let primary = ui.default_action();
    let hot = ui.hover == Some(primary) && !ui.launching;
    let down = ui.pressed == Some(primary) && hot;
    if hot { c.glow_rrect(b, rad, 22.0 * s, GLOW, 0.13); }
    let (top, bot) = if down { (GOLD_LO, GOLD) } else if hot { (GOLD_HI, GOLD) } else { (rgb(0xe2bf68), GOLD_LO) };
    c.fill_rrect_vgrad(b, rad, top, bot, 1.0);
    c.stroke_rrect(b, rad, 1.0 * s, rgb(0xf6e2a8), 0.35);
    // A sheen that crosses the button every few seconds (and continuously while starting).
    let period = if ui.launching { 1.1 } else { 4.5 };
    let ph = (t % period) / period;
    let sx = b.x - 0.3 * b.w + ph * b.w * 1.9;
    sheen(c, b, rad, sx, 70.0 * s, if ui.launching { 0.28 } else { 0.16 });
    let label = if ui.launching { "STARTING\u{2026}" } else if primary == Btn::Play { "PLAY" } else { "SELECT DISC IMAGE" };
    let ls = if primary == Btn::Play && !ui.launching { 23.0 * s } else { 18.0 * s };
    let track = if primary == Btn::Play && !ui.launching { 9.0 * s } else { 4.0 * s };
    let base = b.y + b.h * 0.5 + ascent(&fonts.bold, ls) * 0.36;
    c.text_centered(&fonts.bold, ls, b.x + b.w * 0.5, base, label, INK, 1.0, track);

    // ── Secondary row ───────────────────────────────────────────────────────────────────────────────────
    for (btn, r) in ui.buttons() {
        let label = match btn { Btn::Change => "Change disc", Btn::Quit => "Quit", _ => continue };
        let hot = ui.hover == Some(btn) && !ui.launching;
        let col = if hot { TEXT } else { DIM };
        let size = 14.5 * s;
        let lw = measure(&fonts.regular, size, label, 0.4 * s);
        let lx = if btn == Btn::Quit { (r.x + r.w) * s - lw - 6.0 * s } else { (r.x + 6.0) * s };
        c.text(&fonts.regular, size, lx, SECOND_Y * s, label, col, 1.0, 0.4 * s);
        if hot { c.fill_rrect(Rect::new(lx, (SECOND_Y + 5.0) * s, lw, 1.2 * s), 0.0, GOLD, 0.9); }
    }

    // ── Error ───────────────────────────────────────────────────────────────────────────────────────────
    if let Some(e) = &ui.error {
        let lines = wrap(&fonts.regular, 13.0 * s, e, (COL_W - 22.0) * s);
        let n = lines.len().min(2);
        let y0 = (SECOND_Y + 30.0) * s - (n.saturating_sub(1)) as f32 * 8.0 * s;
        c.fill_rrect(Rect::new(x, y0 - 10.0 * s, 3.0 * s, (n as f32 * 17.0 - 2.0) * s), 1.5 * s, ALERT, 1.0);
        for (i, line) in lines.iter().take(2).enumerate() {
            c.text(&fonts.regular, 13.0 * s, x + 12.0 * s, y0 + i as f32 * 17.0 * s, line, ALERT, 1.0, 0.0);
        }
    }

    // ── Footer ──────────────────────────────────────────────────────────────────────────────────────────
    let fy = (H - 18.0) * s;
    let ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    c.text(&fonts.regular, 11.5 * s, 24.0 * s, fy, &ver, FAINT, 1.0, 0.3 * s);
}

/// A soft diagonal light band across a rounded rect (additive).
fn sheen(c: &mut Canvas, r: Rect, rad: f32, cx: f32, width: f32, a: f32) {
    let (x0, y0) = (r.x.max(0.0) as i32, r.y.max(0.0) as i32);
    let (x1, y1) = (((r.x + r.w) as i32).min(c.w as i32), ((r.y + r.h) as i32).min(c.h as i32));
    for y in y0..y1 {
        for x in x0..x1 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            // Inside test with the same rounded corners.
            let inside = Rect::new(r.x + rad, r.y, r.w - 2.0 * rad, r.h).contains(px, py)
                || Rect::new(r.x, r.y + rad, r.w, r.h - 2.0 * rad).contains(px, py)
                || [(r.x + rad, r.y + rad), (r.x + r.w - rad, r.y + rad), (r.x + rad, r.y + r.h - rad), (r.x + r.w - rad, r.y + r.h - rad)]
                    .iter().any(|&(qx, qy)| (px - qx).powi(2) + (py - qy).powi(2) <= rad * rad);
            if !inside { continue; }
            let d = (px - cx + (py - r.y) * 0.45).abs() / width;
            if d < 1.0 { c.add(x, y, [1.0, 0.97, 0.88], a * (1.0 - d) * (1.0 - d)); }
        }
    }
}

/// A 64×64 RGBA window icon: the barrel motif on black.
pub fn icon_rgba() -> (Vec<u8>, u32) {
    let n = 64;
    let mut c = Canvas::new(n, n);
    c.shade(|_, _| rgb(0x070706));
    c.barrel(32.0, 32.0, 25.0, 0.0, GOLD, GLOW);
    let mut out = Vec::new();
    c.to_rgba(&mut out);
    (out, n)
}
