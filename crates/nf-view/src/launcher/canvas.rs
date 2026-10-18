//! A small CPU rasterizer for the launcher: an RGBA8 frame, anti-aliased shapes (signed-distance coverage) and text
//! through `ab_glyph` with a system font. The launcher is one screen of flat shapes and a few lines of text, so
//! drawing it on the CPU keeps it identical between the window and the headless `--shot` path.

use ab_glyph::{Font, FontVec, PxScale, ScaleFont, VariableFont};

pub type Rgb = [f32; 3];

/// `0xRRGGBB` → linear-ish 0..1 floats (the surface is non-sRGB, so these are the bytes shown).
pub const fn rgb(hex: u32) -> Rgb {
    [((hex >> 16) & 0xff) as f32 / 255.0, ((hex >> 8) & 0xff) as f32 / 255.0, (hex & 0xff) as f32 / 255.0]
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Rect { x, y, w, h } }
    pub fn contains(&self, px: f32, py: f32) -> bool { px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h }
    pub fn scaled(&self, s: f32) -> Rect { Rect { x: self.x * s, y: self.y * s, w: self.w * s, h: self.h * s } }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub struct Canvas {
    pub w: u32,
    pub h: u32,
    px: Vec<Rgb>,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self { Canvas { w, h, px: vec![[0.0; 3]; (w * h) as usize] } }

    pub fn resize(&mut self, w: u32, h: u32) {
        if (w, h) != (self.w, self.h) { *self = Canvas::new(w, h); }
    }

    /// Copies another canvas of the same size (the cached backdrop).
    pub fn copy_from(&mut self, other: &Canvas) {
        if other.px.len() == self.px.len() { self.px.copy_from_slice(&other.px); }
    }

    /// The frame as RGBA8 (alpha 255), with a 1-LSB ordered dither so the dark gradients don't band.
    pub fn to_rgba(&self, out: &mut Vec<u8>) {
        const BAYER: [f32; 16] = [0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0];
        out.clear();
        out.reserve(self.px.len() * 4);
        for (i, p) in self.px.iter().enumerate() {
            let (x, y) = (i as u32 % self.w, i as u32 / self.w);
            let d = BAYER[((y & 3) * 4 + (x & 3)) as usize] / 16.0;
            for c in p { out.push((c.clamp(0.0, 1.0) * 255.0 + d).min(255.0) as u8); }
            out.push(255);
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Rgb, a: f32) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 || a <= 0.0 { return; }
        let p = &mut self.px[(y as u32 * self.w + x as u32) as usize];
        let a = a.min(1.0);
        for k in 0..3 { p[k] += (c[k] - p[k]) * a; }
    }

    #[inline]
    pub fn add(&mut self, x: i32, y: i32, c: Rgb, a: f32) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 || a <= 0.0 { return; }
        let p = &mut self.px[(y as u32 * self.w + x as u32) as usize];
        for k in 0..3 { p[k] += c[k] * a; }
    }

    /// Pixel bounds of a rect expanded by `pad`, clipped to the canvas.
    fn bounds(&self, r: Rect, pad: f32) -> (i32, i32, i32, i32) {
        let x0 = ((r.x - pad).floor() as i32).max(0);
        let y0 = ((r.y - pad).floor() as i32).max(0);
        let x1 = ((r.x + r.w + pad).ceil() as i32).min(self.w as i32);
        let y1 = ((r.y + r.h + pad).ceil() as i32).min(self.h as i32);
        (x0, y0, x1, y1)
    }

    /// Fills with a per-pixel function of (x, y) → colour.
    pub fn shade(&mut self, f: impl Fn(f32, f32) -> Rgb) {
        for y in 0..self.h {
            for x in 0..self.w {
                self.px[(y * self.w + x) as usize] = f(x as f32 + 0.5, y as f32 + 0.5);
            }
        }
    }

    /// Signed distance from a pixel centre to a rounded rect (negative inside).
    fn rr_sdf(r: Rect, radius: f32, px: f32, py: f32) -> f32 {
        let (hw, hh) = (r.w * 0.5, r.h * 0.5);
        let (cx, cy) = (r.x + hw, r.y + hh);
        let rad = radius.min(hw).min(hh);
        let qx = (px - cx).abs() - (hw - rad);
        let qy = (py - cy).abs() - (hh - rad);
        let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
        outside + qx.max(qy).min(0.0) - rad
    }

    pub fn fill_rrect(&mut self, r: Rect, radius: f32, c: Rgb, a: f32) {
        let (x0, y0, x1, y1) = self.bounds(r, 1.0);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = Self::rr_sdf(r, radius, x as f32 + 0.5, y as f32 + 0.5);
                self.blend(x, y, c, a * (0.5 - d).clamp(0.0, 1.0));
            }
        }
    }

    /// A vertical gradient fill (top colour → bottom colour) inside a rounded rect.
    pub fn fill_rrect_vgrad(&mut self, r: Rect, radius: f32, top: Rgb, bottom: Rgb, a: f32) {
        let (x0, y0, x1, y1) = self.bounds(r, 1.0);
        for y in y0..y1 {
            let t = ((y as f32 + 0.5 - r.y) / r.h.max(1.0)).clamp(0.0, 1.0);
            let c = [top[0] + (bottom[0] - top[0]) * t, top[1] + (bottom[1] - top[1]) * t, top[2] + (bottom[2] - top[2]) * t];
            for x in x0..x1 {
                let d = Self::rr_sdf(r, radius, x as f32 + 0.5, y as f32 + 0.5);
                self.blend(x, y, c, a * (0.5 - d).clamp(0.0, 1.0));
            }
        }
    }

    pub fn stroke_rrect(&mut self, r: Rect, radius: f32, width: f32, c: Rgb, a: f32) {
        let (x0, y0, x1, y1) = self.bounds(r, width + 1.0);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = Self::rr_sdf(r, radius, x as f32 + 0.5, y as f32 + 0.5);
                self.blend(x, y, c, a * (width * 0.5 + 0.5 - (d + width * 0.5).abs()).clamp(0.0, 1.0));
            }
        }
    }

    /// A soft additive glow around a rounded rect (`spread` px falloff).
    pub fn glow_rrect(&mut self, r: Rect, radius: f32, spread: f32, c: Rgb, a: f32) {
        let (x0, y0, x1, y1) = self.bounds(r, spread);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = Self::rr_sdf(r, radius, x as f32 + 0.5, y as f32 + 0.5);
                if d > 0.0 { let k = 1.0 - (d / spread).min(1.0); self.add(x, y, c, a * k * k); }
            }
        }
    }

    /// A horizontal line fading out at both ends.
    pub fn hline_fade(&mut self, x: f32, y: f32, w: f32, thick: f32, c: Rgb, a: f32) {
        let (x0, y0, x1, y1) = self.bounds(Rect::new(x, y, w, thick), 1.0);
        for py in y0..y1 {
            let cov_y = ((py as f32 + 1.0).min(y + thick) - (py as f32).max(y)).clamp(0.0, 1.0);
            for px in x0..x1 {
                let t = (px as f32 + 0.5 - x) / w;
                if !(0.0..=1.0).contains(&t) { continue; }
                let fade = smoothstep(0.0, 0.18, t) * smoothstep(1.0, 0.82, t);
                self.blend(px, py, c, a * cov_y * fade);
            }
        }
    }

    /// The gun-barrel motif: looking down a rifled bore. Original procedural art. A wall point at depth `z` (bore
    /// radii) projects to `u = 1/z` of `radius`; the rifling turns slowly with depth, the light from the muzzle
    /// falls off with it, and a warm light sits at the far end. `t` (seconds) turns the rifling and breathes the light.
    pub fn barrel(&mut self, cx: f32, cy: f32, radius: f32, t: f32, gold: Rgb, glow: Rgb) {
        let lands = 8.0f32;
        let twist = 0.16f32; // radians of turn per bore radius of depth
        let spin = t * 0.12;
        let pulse = 0.88 + 0.12 * (t * 1.1).sin();
        let face = 0.07f32; // muzzle face width (fraction of radius)
        let r = Rect::new(cx - radius * 1.4, cy - radius * 1.4, radius * 2.8, radius * 2.8);
        let (x0, y0, x1, y1) = self.bounds(r, 2.0);
        let px_u = 1.0 / radius; // one pixel in u
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let d = (dx * dx + dy * dy).sqrt();
                let u = d / radius;
                let theta = dy.atan2(dx);
                if u > 1.0 + face + 0.5 * px_u {
                    // Faint gold halo outside the muzzle.
                    let k = (1.0 - (u - 1.0 - face) / 0.4).clamp(0.0, 1.0);
                    self.add(x, y, gold, 0.05 * k * k * k * pulse);
                    continue;
                }
                if u >= 1.0 {
                    // The muzzle face: metal lit from the upper left, gold-toned, bright outer and inner edges.
                    let lightdir = ((theta + 2.36).cos() * 0.5 + 0.5).powf(1.5); // 2.36 rad ≈ upper left
                    let f = ((u - 1.0) / face).clamp(0.0, 1.0); // 0 inner lip → 1 outer edge
                    let base = 0.22 + 0.5 * lightdir;
                    let edge = (1.0 - (f - 0.5).abs() * 2.0).max(0.0).powf(0.4);
                    let mut c = [0.0; 3];
                    for k in 0..3 { c[k] = gold[k] * base * (0.55 + 0.45 * edge); }
                    let rim_hi = ((1.0 - f) * face / px_u).clamp(0.0, 1.0).min(((f * face) / px_u).clamp(0.0, 1.0));
                    let outer_aa = ((1.0 + face - u) / px_u + 0.5).clamp(0.0, 1.0);
                    self.blend(x, y, c, outer_aa * (0.6 + 0.4 * rim_hi));
                    continue;
                }
                // Inside the bore.
                let uu = u.max(1e-3);
                let z = 1.0 / uu; // depth in bore radii (1 = muzzle)
                let light = (1.0 / z).powf(1.35); // muzzle light falling off down the bore
                let phase = (theta + twist * (z - 1.0) + spin) * lands / std::f32::consts::TAU;
                let f = phase - phase.floor();
                // Each land: a lit flank rising to a sharp bright edge, then a darker groove.
                let flank = smoothstep(0.05, 0.42, f) * smoothstep(0.5, 0.44, f);
                let edge = smoothstep(0.40, 0.47, f) * smoothstep(0.53, 0.47, f);
                let groove = smoothstep(0.55, 0.62, f) * smoothstep(1.0, 0.92, f);
                // Grooves crowd toward the throat; fade them out where they'd alias.
                let detail = smoothstep(0.05, 0.22, u);
                let wall = 0.10 + detail * (0.12 * flank + 0.26 * edge - 0.05 * groove);
                // Machining rings at whole bore-radius depths, sharpest near the muzzle.
                let ring = ((z * std::f32::consts::PI).sin().abs()).powf(30.0) * smoothstep(0.35, 0.8, u);
                let lum = (wall + ring * 0.08) * light;
                let mut c = [lum * 1.0, lum * 0.86, lum * 0.62];
                // Lit from the upper left like the face.
                let side = 0.75 + 0.25 * (theta + 2.36).cos();
                for k in 0..3 { c[k] *= side; }
                // The far opening: warm light.
                let g = (1.0 - u / 0.3).clamp(0.0, 1.0);
                let g = g * g * g * pulse;
                for k in 0..3 { c[k] += glow[k] * g * 0.75; }
                let core = (1.0 - u / 0.07).clamp(0.0, 1.0);
                for k in 0..3 { c[k] += core * core * 0.75 * pulse; }
                // Shadow just inside the lip.
                let lip = smoothstep(0.95, 1.0, u);
                for k in 0..3 { c[k] *= 1.0 - 0.45 * lip; }
                self.blend(x, y, c, 1.0);
            }
        }
    }
}

/// The launcher's three type styles (one family where possible).
pub struct Fonts {
    pub regular: FontVec,
    pub bold: FontVec,
    /// Heavy, tight: the title.
    pub display: FontVec,
}

/// Candidate system fonts, best first: (regular, bold, is variable). Bahnschrift (Windows 10+) is a DIN-style
/// variable face — the clean engineered look of spy-film titling; the others are plain fallbacks.
fn candidates() -> Vec<(std::path::PathBuf, std::path::PathBuf, bool)> {
    let mut v = Vec::new();
    if cfg!(windows) {
        let dir = std::path::PathBuf::from(std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into())).join("Fonts");
        v.push((dir.join("bahnschrift.ttf"), dir.join("bahnschrift.ttf"), true));
        v.push((dir.join("segoeui.ttf"), dir.join("segoeuib.ttf"), false));
        v.push((dir.join("arial.ttf"), dir.join("arialbd.ttf"), false));
    } else {
        for d in ["/usr/share/fonts/truetype/dejavu", "/usr/share/fonts/dejavu", "/usr/share/fonts/TTF"] {
            let d = std::path::PathBuf::from(d);
            v.push((d.join("DejaVuSans.ttf"), d.join("DejaVuSans-Bold.ttf"), false));
        }
        v.push(("/System/Library/Fonts/Supplemental/Arial.ttf".into(), "/System/Library/Fonts/Supplemental/Arial Bold.ttf".into(), false));
    }
    v
}

impl Fonts {
    pub fn load() -> Option<Fonts> {
        for (reg, bold, variable) in candidates() {
            let Ok(rb) = std::fs::read(&reg) else { continue };
            let bb = std::fs::read(&bold).unwrap_or_else(|_| rb.clone());
            let (Ok(regular), Ok(mut b), Ok(mut display)) = (FontVec::try_from_vec(rb.clone()), FontVec::try_from_vec(bb), FontVec::try_from_vec(rb)) else { continue };
            let mut regular = regular;
            if variable {
                regular.set_variation(b"wght", 350.0);
                b.set_variation(b"wght", 600.0);
                display.set_variation(b"wght", 700.0);
                display.set_variation(b"wdth", 88.0);
            } else {
                display = FontVec::try_from_vec(std::fs::read(&bold).ok()?).ok()?;
            }
            return Some(Fonts { regular, bold: b, display });
        }
        None
    }
}

/// Text width in px at `size` with `tracking` px added after each character but the last.
pub fn measure(font: &FontVec, size: f32, s: &str, tracking: f32) -> f32 {
    let sf = font.as_scaled(PxScale::from(size));
    let mut w = 0.0;
    let mut prev = None;
    let n = s.chars().count();
    for (i, ch) in s.chars().enumerate() {
        let id = sf.glyph_id(ch);
        if let Some(p) = prev { w += sf.kern(p, id); }
        w += sf.h_advance(id);
        if i + 1 < n { w += tracking; }
        prev = Some(id);
    }
    w
}

/// Cap height-ish: the ascent used to place text by its top.
pub fn ascent(font: &FontVec, size: f32) -> f32 { font.as_scaled(PxScale::from(size)).ascent() }

impl Canvas {
    /// Draws `s` with its baseline at `y`; returns the advance.
    pub fn text(&mut self, font: &FontVec, size: f32, x: f32, y: f32, s: &str, c: Rgb, a: f32, tracking: f32) -> f32 {
        let scale = PxScale::from(size);
        let sf = font.as_scaled(scale);
        let mut pen = x;
        let mut prev = None;
        for ch in s.chars() {
            let id = sf.glyph_id(ch);
            if let Some(p) = prev { pen += sf.kern(p, id); }
            let g = id.with_scale_and_position(scale, ab_glyph::point(pen, y));
            if let Some(og) = font.outline_glyph(g) {
                let b = og.px_bounds();
                let (bx, by) = (b.min.x as i32, b.min.y as i32);
                og.draw(|gx, gy, cov| self.blend(bx + gx as i32, by + gy as i32, c, cov * a));
            }
            pen += sf.h_advance(id) + tracking;
            prev = Some(id);
        }
        pen - x - tracking
    }

    /// Text centred on `cx`.
    pub fn text_centered(&mut self, font: &FontVec, size: f32, cx: f32, y: f32, s: &str, c: Rgb, a: f32, tracking: f32) {
        let w = measure(font, size, s, tracking);
        self.text(font, size, cx - w * 0.5, y, s, c, a, tracking);
    }
}

/// Shortens `s` with a middle ellipsis until it fits `max_w` (keeps the file name's end, where the region is).
pub fn ellipsize_middle(font: &FontVec, size: f32, s: &str, max_w: f32) -> String {
    if measure(font, size, s, 0.0) <= max_w { return s.to_string(); }
    let chars: Vec<char> = s.chars().collect();
    let mut keep = chars.len();
    while keep > 4 {
        keep -= 1;
        let head = keep / 2;
        let tail = keep - head;
        let t: String = chars[..head].iter().chain(['…'].iter()).chain(chars[chars.len() - tail..].iter()).collect();
        if measure(font, size, &t, 0.0) <= max_w { return t; }
    }
    "…".into()
}
