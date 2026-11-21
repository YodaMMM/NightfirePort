//! The driving missions' HUD draw logic (DRIVING.ELF), headless: per frame it turns the game state
//! into a list of HUD draws (sprite / text / panel, with position, colour and text). **Nothing here
//! renders.** Data: the `hudNTSC.gal` gallery (`nf_formats::gal`, docs/formats/hud-gal.md), the
//! HUD font `data\loading\loading.sfn` (`nf_formats::sfn`, docs/formats/sfn.md) and the string
//! table `IDL_*.loc` (`nf_formats::loc`, docs/formats/loc.md). System doc:
//! docs/systems/driving-hud.md.
//!
//! Evidence (all [M] DRIVING.ELF unless noted; EE addresses, decompile `work/decomp/DRIVING.ELF/`,
//! disassembly checked where the decompile drops arguments):
//! - `FUN_0023ae30` builds the HUD: loads `hudNTSC.gal`, binds `loading.sfn` as its one font
//!   (`FUN_002378b8` → gal+0xa8), finds the animations `hud_base`, `hud_msg`, `pauseN`, `paus_err`
//!   and the elements by name into fixed slots, then moves the element keys in place: y −10 for
//!   weapon, ammo, iris1..7, gadget0..5, a007 + arm1..7 and healthBorder (0x23c004-0x23c0b4); with
//!   the renderer's mode word +0x58 ∈ {1, 2} (set to 2 by `FUN_001abee0` exactly when the
//!   widescreen flag +0x5c is on) x +78 for weapon, ammo, stylin and the gadgets, x −78 for iris,
//!   a007/arm and healthBorder (0x23c2c0-0x23c398). The third case (+0x5c on, +0x58 ∉ {1, 2}: x
//!   ±110, y +60) is not produced by `FUN_001abee0` and is not ported.
//! - `FUN_0023cf88` is the HUD frame. While HUD +0x314 == 0 it runs `FUN_00246c48` (objective
//!   banner) and `FUN_002472b0` (stunt award), then with HUD state +0x2b0 == 1: `FUN_00240c98`
//!   weapon + ammo, `FUN_0023f328` crosshairs/targets (**not ported**), `FUN_0023da30` health,
//!   `FUN_002469b8` message line, `FUN_002470f0` (empty), `FUN_00245ab8` button prompt line,
//!   `FUN_00244f88` gadget pop-up, `FUN_0023e580` timer. `DriveHudDraw::frame` emits the ported
//!   passes in that order (the +0x314 / +0x2b0 gates are the caller's).
//! - Text: one font per gallery, so every text element shares one scale state (font +0x38/+0x3c).
//!   `FUN_002377d8(gal, key)` binds an element (scale = key +0x38 / +0x3c ÷ 1000, colour +0x30),
//!   `FUN_002378a0(sx, sy)` sets the scale, `FUN_00237ca8` / `FUN_00237ce0` measure at the scale
//!   the font holds, and `FUN_00237b80` (the draw) rewrites that scale for the screen and leaves
//!   it so (`draw_transform`). `Pen` tracks that state so every measure uses the scale the game's
//!   measure sees.
//! - Colours are the GS words the game writes to the draw state (R in the low byte, 0x80 = full).
//! - `FUN_0019f9c8` (the sprite primitive) puts (x, y) at the top-left corner, (x+w, y+h) at the
//!   bottom-right.

use nf_formats::gal::{Gal, Key};
use nf_formats::loc::Loc;
use nf_formats::ps2float::{add_noguard, mul};
use nf_formats::sfn::{Font, GlyphQuad};

#[inline]
fn add(a: f32, b: f32) -> f32 { add_noguard(a, b, true) }
#[inline]
fn sub(a: f32, b: f32) -> f32 { add_noguard(a, -b, true) }

/// One bitmap element of the layout, after `FUN_0023ae30`'s moves.
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    pub name: String,
    pub pos: [f32; 2],
    /// width/height × the element's own scale (`FUN_002386c8` sets gal+0x80/0x84 from key
    /// +0x54/+0x58; `FUN_00238d40` multiplies) [M].
    pub size: [f32; 2],
    /// width/height as stored (key +0x40 / +0x44) — what passes that override the scale read.
    pub key_size: [f32; 2],
    /// u0, u1, v0, v1 as stored (key +0x30..0x40).
    pub uv: [f32; 4],
    pub sheet: u32,
    /// key+0x5c mirror bits and the signed repeat counts (left, right, up, down) [M 0x238d40].
    pub mirror: u32,
    pub repeats: [i8; 4],
}

impl Sprite {
    fn of(name: &str, k: &Key) -> Sprite {
        let [x, y, _] = k.pos();
        let [w, h] = k.size();
        let [sx, sy] = k.scale();
        Sprite { name: name.to_owned(), pos: [x, y], size: [w * sx, h * sy], key_size: [w, h], uv: k.uv(), sheet: k.sheet(), mirror: k.mirror(), repeats: k.repeats() }
    }
    fn draw(&self, colour: u32) -> Vec<HudDraw> {
        // FUN_002391e8 → FUN_00238d40(gal, (int)x, (int)y, key, 0) [M].
        self.draw_at(self.pos[0] as i32, self.pos[1] as i32, colour)
    }

    /// The same element with the draw state's scale overridden (gal+0x80 / +0x84 written after
    /// `FUN_002386c8`, as the stunt award does) [M 0x2473a8].
    fn rescaled(&self, s: [f32; 2]) -> Sprite {
        Sprite { size: [self.key_size[0] * s[0], self.key_size[1] * s[1]], ..self.clone() }
    }

    /// `FUN_00238d40(gal, x, y, key, 0)` [M]: the quads one element call draws. Repeats: the step
    /// is −w / +w (left / right count > 0) and −h / +h (up / down), the call count is the sum of the
    /// four signed counts (0 → 1, negative → none). Mirror bits (key+0x5c): 0 → the quad at (x, y);
    /// 3 → four quads around (x, y) (normal, both-flipped at (x−w, y−h), x-flipped at (x−w, y),
    /// y-flipped at (x, y−h)); 1 → only the x-flipped quad at (x−w, y); 2 → only the y-flipped
    /// quad at (x, y−h). UVs are inset by 1/512 (DAT_0033aae0 = 0x3b000000 [D], no writer [M]).
    fn draw_at(&self, mut x: i32, mut y: i32, colour: u32) -> Vec<HudDraw> {
        const D: f32 = 0.001953125;
        let [w, h] = self.size;
        let [u0, u1, v0, v1] = self.uv;
        let [l, r, up, dn] = self.repeats;
        let mut dx = 0i32;
        let mut dy = 0i32;
        if l > 0 { dx = -w as i32; }
        if r > 0 { dx = w as i32; }
        if up > 0 { dy = -h as i32; }
        if dn > 0 { dy = h as i32; }
        let mut n = l as i32 + r as i32 + up as i32 + dn as i32;
        if n == 0 { n = 1; }
        let mut out = Vec::new();
        let mut quad = |x: f32, y: f32, uv: [f32; 4]| out.push(HudDraw::Sprite {
            element: self.name.clone(), x, y, w, h, uv, sheet: self.sheet, colour,
        });
        let both = self.mirror & 3 == 3;
        let mx = self.mirror & 1 != 0;
        let my = self.mirror & 2 != 0;
        while n > 0 {
            let (fx, fy) = (x as f32, y as f32);
            if both {
                quad(fx, fy, [u0 + D, v0 + D, u1 - D, v1 - D]);
                quad(fx - w, fy - h, [u1 - D, v1 - D, u0 + D, v0 + D]);
            }
            if mx { quad(fx - w, fy, [u1 - D, v0 + D, u0 + D, v1 - D]); }
            if my { quad(fx, fy - h, [u0 + D, v1 - D, u1 - D, v0 + D]); }
            if !mx && !my && !both { quad(fx, fy, [u0 + D, v0 + D, u1 - D, v1 - D]); }
            y += dy;
            n -= 1;
            x += dx;
        }
        out
    }
}

/// A text element: its key position, the scale `FUN_002377d8` binds (key +0x38 / +0x3c as
/// integer thousandths, `div.s` by 1000.0 [M 0x237874-0x23788c]) and its own label id (+0x34,
/// drawn when a pass hands `FUN_00244398` no text [M]).
#[derive(Clone, Debug, PartialEq)]
pub struct TextAnchor { pub name: String, pub pos: [f32; 2], pub scale: [f32; 2], pub label: i32 }

impl TextAnchor {
    fn of(name: &str, k: &Key) -> TextAnchor {
        let [x, y, _] = k.pos();
        let i = |o: usize| i32::from_le_bytes(k.raw[o..o + 4].try_into().unwrap());
        TextAnchor { name: name.to_owned(), pos: [x, y], scale: [i(0x38) as f32 / 1000.0, i(0x3c) as f32 / 1000.0], label: i(0x34) }
    }
}

/// The slots `FUN_0023ae30` fills, with their in-place moves applied.
#[derive(Clone, Debug, PartialEq)]
pub struct HudLayout {
    /// hud+0x14 `weapon`, hud+0x18 `ammo` (text, `hud_base`).
    pub weapon: TextAnchor,
    pub ammo: TextAnchor,
    /// hud+0x74 `stylin` (the stunt award bitmap).
    pub stylin: Sprite,
    /// hud+0x1c.. `iris1..iris7`.
    pub iris: Vec<Sprite>,
    /// hud+0x38 `a007`, hud+0x3c.. `arm1..arm7` (the armour row starts with the 007 logo).
    pub arms: Vec<Sprite>,
    /// hud+0x58 `healthBorder`.
    pub health_border: Sprite,
    /// hud+0x78.. `gadget0..gadget5`.
    pub gadgets: Vec<Sprite>,
    /// hud+0x25c `txt` (`hud_msg`) — the message line's element.
    pub msg_txt: TextAnchor,
    /// hud+0x1f0 `scratchTxt` (`pauseN`) — the timer's element.
    pub timer_font: TextAnchor,
    /// hud+0x268 `objobjtxt` / hud+0x26c `objbody` (`hud_msg`) — the objective banner's lines.
    pub obj_head: TextAnchor,
    pub obj_body: TextAnchor,
    /// hud+0xc0 `buttonTxt` (`hud_msg`) — the button prompt line.
    pub button_txt: TextAnchor,
    /// The prompt's button icons by token `*A*`, `*G*`, `*P*`, `*D*` (0x3aae00..): hud+0xd0
    /// `PS2X`, hud+0xcc `PS2L1`, hud+0xc4 `PS2R1`, hud+0xc8 `PS2R1` [M 0x245c30-0x245c78].
    pub buttons: [Sprite; 4],
    /// Screen centre (hud+0x454 / +0x458 = renderer width / height × 0.5; 640 × 448 NTSC,
    /// 640 × 512 for the other video mode [M `FUN_001abee0`, `FUN_0023c858`]).
    pub centre: [f32; 2],
    /// The renderer's +0x5c flag (widescreen) and +0x54 height, read by every text draw
    /// (`FUN_00237b80`).
    pub widescreen: bool,
    pub screen_h: f32,
    /// hud+0x4f0: 25 / 30 / 60 by `FUN_002d6b80` (2 / other / 3) [M `FUN_0023ae30` tail]; the word
    /// comes from hand-over word 0x100a1c (`FUN_00173c60`: 2 or 5 → 2, 3 → 3, else 1). 30 here —
    /// which word an NTSC boot hands over is not traced [S].
    pub hud_rate: f32,
}

impl HudLayout {
    /// Build the layout from the gallery like `FUN_0023ae30` [M]. `widescreen` = the renderer's
    /// +0x5c flag (which sets +0x58 = 2 in `FUN_001abee0`); `screen_h` = 448 or 512.
    pub fn from_gal(g: &Gal, widescreen: bool, screen_h: f32) -> Result<HudLayout, String> {
        let base = g.anim("hud_base").ok_or("no hud_base")?;
        let msg = g.anim("hud_msg").ok_or("no hud_msg")?;
        let key = |a: &nf_formats::gal::Anim, n: &str| -> Result<Key, String> {
            a.element(n).and_then(|e| e.hud_key()).cloned().ok_or_else(|| format!("no element {n}"))
        };
        let sprite = |a: &nf_formats::gal::Anim, n: &str| -> Result<Sprite, String> { Ok(Sprite::of(n, &key(a, n)?)) };
        let text = |a: &nf_formats::gal::Anim, n: &str| -> Result<TextAnchor, String> { Ok(TextAnchor::of(n, &key(a, n)?)) };
        let mut l = HudLayout {
            weapon: text(base, "weapon")?,
            ammo: text(base, "ammo")?,
            stylin: sprite(base, "stylin")?,
            iris: (1..=7).map(|i| sprite(base, &format!("iris{i}"))).collect::<Result<_, _>>()?,
            arms: std::iter::once("a007".to_owned()).chain((1..=7).map(|i| format!("arm{i}")))
                .map(|n| sprite(base, &n)).collect::<Result<_, _>>()?,
            health_border: sprite(base, "healthBorder")?,
            gadgets: (0..6).map(|i| sprite(base, &format!("gadget{i}"))).collect::<Result<_, _>>()?,
            msg_txt: text(msg, "txt")?,
            timer_font: text(g.anim("pauseN").ok_or("no pauseN")?, "scratchTxt")?,
            obj_head: text(msg, "objobjtxt")?,
            obj_body: text(msg, "objbody")?,
            button_txt: text(msg, "buttonTxt")?,
            buttons: [sprite(msg, "PS2X")?, sprite(msg, "PS2L1")?, sprite(msg, "PS2R1")?, sprite(msg, "PS2R1")?],
            centre: [320.0, screen_h * 0.5],
            widescreen,
            screen_h,
            hud_rate: 30.0,
        };
        // y − 10 [M 0x23c004-0x23c0b4] (stylin is not in the list).
        l.weapon.pos[1] -= 10.0;
        l.ammo.pos[1] -= 10.0;
        for s in l.iris.iter_mut().chain(l.gadgets.iter_mut()).chain(l.arms.iter_mut()) { s.pos[1] -= 10.0; }
        l.health_border.pos[1] -= 10.0;
        if widescreen {
            // x ± 78 [M 0x23c2c0-0x23c398].
            l.weapon.pos[0] += 78.0;
            l.ammo.pos[0] += 78.0;
            l.stylin.pos[0] += 78.0;
            for s in &mut l.iris { s.pos[0] -= 78.0; }
            for s in &mut l.arms { s.pos[0] -= 78.0; }
            for s in &mut l.gadgets { s.pos[0] += 78.0; }
            l.health_border.pos[0] -= 78.0;
        }
        Ok(l)
    }
}

/// The font and the string table the HUD text goes through.
#[derive(Clone, Copy)]
pub struct HudStrings<'a> { pub font: &'a Font, pub loc: &'a Loc }

/// One HUD draw.
#[derive(Clone, Debug, PartialEq)]
pub enum HudDraw {
    /// A gallery bitmap quad: top-left (x, y), w × h; `uv` = (u, v) at the top-left corner then
    /// (u, v) at the bottom-right (a flipped copy swaps them); `tx` sheet; GS colour.
    Sprite { element: String, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], sheet: u32, colour: u32 },
    /// `FUN_00244398(x, y, scale, hud, element, shadow, colour, text)`: the shadow at (x+1, y+1),
    /// then the text at (x, y), both at (scale, scale) through `FUN_00237b80` — `text_glyphs`
    /// turns it into glyph sprites. `x` is final (the pass already placed it by measured width).
    Text { element: String, text: Vec<u8>, x: f32, y: f32, scale: f32, colour: u32, shadow: u32 },
    /// A filled box `FUN_00238408(x, y, w, h, colour)` (screen units, no draw-time scaling).
    Panel { x: f32, y: f32, w: f32, h: f32, colour: u32 },
}

/// `FUN_00237b80`'s screen adjustment of one text draw [M, disassembly 0x237bb8-0x237c64]:
/// k = (float)height / 480; y scale × k and y × k; widescreen: x scale × 0.75 and, with mode word
/// +0x58 == 2 (always so with widescreen, `FUN_001abee0`), x pulled toward 320 by
/// ((320 − x) / 320) × 80 (or − ((x − 320) / 320) × 80 right of centre); a widescreen mode ≠ 2
/// would scale y by 0.75 instead (not produced). Returns ((x, y), scale) as the font then holds
/// them — the scale stays in the font afterwards.
pub fn draw_transform(x: f32, y: f32, scale: [f32; 2], widescreen: bool, screen_h: f32) -> ([f32; 2], [f32; 2]) {
    let k = (screen_h as i32) as f32 / 480.0;
    let y = mul(y, k);
    let mut sy = mul(scale[1], k);
    let mut sx = scale[0];
    let mut x = x;
    if widescreen {
        sx = mul(sx, 0.75);
        let mode2 = true;
        if mode2 {
            if x < 320.0 { x = add(x, mul(sub(320.0, x) / 320.0, 80.0)); } else { x = sub(x, mul(sub(x, 320.0) / 320.0, 80.0)); }
        } else {
            sy = mul(sy, 0.75);
        }
    }
    ([x, y], [sx, sy])
}

/// The glyph sprites of a `HudDraw::Text` (shadow first, then the text) [M `FUN_00244398` →
/// `FUN_00237b80` → `FUN_00259040` → `FUN_00258100` / `FUN_002a1e50`]. `FUN_00259040` first
/// runs the text through vsprintf as a format with no arguments; strings with `%` are passed
/// through unchanged here [S — 3 English strings hold a `%`, none of them a HUD line ported here].
pub fn text_glyphs(d: &HudDraw, font: &Font, widescreen: bool, screen_h: f32) -> (Vec<GlyphQuad>, Vec<GlyphQuad>) {
    let HudDraw::Text { text, x, y, scale, .. } = d else { return (vec![], vec![]) };
    let (p, s) = draw_transform(x + 1.0, y + 1.0, [*scale; 2], widescreen, screen_h);
    let shadow = font.layout(p[0], p[1], text, s);
    let (p, s) = draw_transform(*x, *y, [*scale; 2], widescreen, screen_h);
    (shadow, font.layout(p[0], p[1], text, s))
}

/// The gallery's one font state while a frame runs: its scale (font +0x38 / +0x3c).
struct Pen<'a> { s: HudStrings<'a>, scale: [f32; 2], widescreen: bool, screen_h: f32 }

impl<'a> Pen<'a> {
    /// `FUN_002377d8(gal, key)`: the element's scale.
    fn bind(&mut self, a: &TextAnchor) { self.scale = a.scale; }
    /// `FUN_002378a0(s, s, gal)`.
    fn set(&mut self, s: f32) { self.scale = [s, s]; }
    fn width(&self, t: &[u8]) -> f32 { self.s.font.text_width(t, self.scale) }
    fn height(&self, t: &[u8]) -> f32 { self.s.font.text_height(t, self.scale) }
    fn label(&self, id: i32) -> Vec<u8> { self.s.loc.label(id).to_vec() }
    /// `FUN_00241ea8(s, hud, element, text)`: x = centre − width × 0.5 after binding the element
    /// and setting scale s [M 0x241ea8-0x241f28].
    fn centred(&mut self, s: f32, a: &TextAnchor, t: &[u8], centre: f32) -> f32 {
        self.bind(a);
        self.set(s);
        sub(centre, mul(self.width(t), 0.5))
    }
    /// `FUN_00244398`: text `None` = the element's own label; the font keeps the scale the last
    /// `FUN_00237b80` left.
    fn text(&mut self, out: &mut HudFrame, a: &TextAnchor, x: f32, y: f32, s: f32, shadow: u32, colour: u32, t: Option<Vec<u8>>) {
        self.bind(a);
        let text = t.unwrap_or_else(|| self.label(a.label));
        self.scale = draw_transform(x, y, [s, s], self.widescreen, self.screen_h).1;
        out.draws.push(HudDraw::Text { element: a.name.clone(), text, x, y, scale: s, colour, shadow });
    }
}

/// The per-frame game state the ported passes read.
#[derive(Clone, Debug, PartialEq)]
pub struct HudInput {
    /// hud+0x368: frame seconds = ticks since the last update × seconds per tick [M `FUN_0023cbc0`].
    pub dt: f32,
    /// hud+0x364: the tick count itself (the stunt award's fade subtracts it) [M 0x24754c].
    pub ticks: u32,
    /// world+0x378 / +0x37c (health, max) and +0x380 / +0x384 (armour, max) [M `FUN_0023da30`].
    pub health: f32,
    pub health_max: f32,
    pub armour: f32,
    pub armour_max: f32,
    /// The current weapon-list entry (DAT_0033a530: +0x10 base, +4 index, stride 0x54; +8 id,
    /// +0x18 ammo) [M `FUN_00240c98`].
    pub weapon: Option<(i32, i32)>,
    /// The timer slot hud+0x464 points at, as (integer seconds = value / ticks-per-second,
    /// countdown = rate < 0 [M `FUN_00209d48`]); `None` while hud+0x464 < 0.
    pub timer: Option<(i32, bool)>,
    /// hud+0x434 (`iRam00100998 != 0` at init) — keeps the health meter lit [M]; what sets the
    /// source word is not traced.
    pub health_always: bool,
    /// hud+0x2a8 — enables the body-hit grunt [M `FUN_00241040` writes it]; meaning not traced.
    pub grunt_on_hit: bool,
    /// hud+0x460 != 0 or `FUN_0024a1e0(FUN_0024a1d0()) != 0` — weapon/ammo and health skip
    /// [M]; what those two states are is not traced.
    pub suppressed: bool,
    /// hud+0x2a0 — the pause menu is up (`FUN_0023cf88` draws it while set); the objective
    /// banner does nothing then [M 0x246c68].
    pub paused: bool,
}

/// One frame's output: the draws in game order, plus the sounds the passes start.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudFrame { pub draws: Vec<HudDraw>, pub sounds: Vec<&'static str> }

/// The ported passes' private state (HUD object fields, init values from `FUN_0023c858` [M]).
#[derive(Clone, Debug, PartialEq)]
pub struct DriveHudDraw {
    /// +0x2cc health show timer (3.0), +0x2d0 health fade (0), +0x2d4 low-health blink (0).
    pub health_show: f32,
    pub health_fade: f32,
    pub health_blink: f32,
    /// +0x36c / +0x370 last health / armour ratios. +0x36c = 1.0 at init; +0x370 is not written
    /// by `FUN_0023c858` — 0.0 here is [S].
    pub last_health: f32,
    pub last_armour: f32,
    /// DAT_0033b1c8 — the grunt cycle (0..3) [M]; its initial value is not traced (0 here).
    pub grunt_cycle: usize,
    /// +0x404 message label (−1), +0x408 hold, +0x40c fade, +0x410 long-message flag.
    pub msg_label: i32,
    pub msg_hold: f32,
    pub msg_fade: f32,
    pub msg_long: bool,
    /// +0x41c gadget slot, +0x420 time left, +0x424 slide, +0x428 blink, +0x42c blinks left.
    pub gadget_slot: u32,
    pub gadget_time: f32,
    pub gadget_slide: f32,
    pub gadget_blink: f32,
    pub gadget_blinks: i32,
    /// +0x2c8 the last timer second that beeped (10 at init).
    pub timer_beep_last: i32,
    /// Objective banner: +0x3a8 header blinks done, +0x3ac blink clock, +0x3b0 time left,
    /// +0x3b4 body label, +0x3bc kind (1 new, 2 complete, 3 failed; −1 none), +0x3b8 / +0x3c0
    /// the queued body label / kind (−1 none), +0x308 the setter's 4th argument (cleared when
    /// the banner ends; its reader is not traced).
    pub banner_blinks: i32,
    pub banner_blink: f32,
    pub banner_time: f32,
    pub banner_label: i32,
    pub banner_kind: i32,
    pub banner_next_label: i32,
    pub banner_next_kind: i32,
    pub banner_flag: u32,
    /// Stunt award: +0x2ec state (0 off, 1 zoom, 2 hold), +0x2e8 time, +0x2f0 scale, +0x2f4 fade.
    pub stunt_state: i32,
    pub stunt_time: f32,
    pub stunt_scale: f32,
    pub stunt_fade: f32,
    /// Button prompt: +0x2f8 text, +0x2fc label (used when > 0), +0x300 time, +0x304 pulse.
    pub prompt_text: Option<Vec<u8>>,
    pub prompt_label: i32,
    pub prompt_time: f32,
    pub prompt_pulse: f32,
}

impl Default for DriveHudDraw { fn default() -> Self { Self::new() } }

/// `(int)(v ± 0.5)` — the game's rounding before every colour byte (`cvt.w.s` truncates) [M].
fn rnd(v: f32) -> i32 { (v + if v < 0.0 { -0.5 } else { 0.5 }) as i32 }

pub const TEXT_COLOUR: u32 = 0x7fb3_dbfa;
pub const TEXT_SHADOW: u32 = 0x4600_0000;
/// The health iris colours by ratio (> 0.5, > 0.25, else) and their dim twins [M 0x23da30].
pub const IRIS_COLOURS: [(u32, u32); 3] = [(0x468b_a852, 0x138b_a852), (0x464e_98c6, 0x134e_98c6), (0x4625_2cb0, 0x1325_2cb0)];
pub const SCALE_1_1: f32 = f32::from_bits(0x3f8c_cccd);
pub const SCALE_1_2: f32 = f32::from_bits(0x3f99_999a);
pub const SCALE_1_4: f32 = f32::from_bits(0x3fb3_3333);
/// Banner header label ids by kind: 1 → 99998 (0x1869e), 2 → 99999, 3 → 99736 (0x18598), any
/// other → 99998 [M 0x246d10-0x246d4c].
pub fn banner_header(kind: i32) -> i32 { match kind { 2 => 99999, 3 => 0x18598, _ => 0x1869e } }
/// The prompt's button tokens, in `FUN_002459f8` search order (0x3aae00, 4-byte stride) [M, D].
pub const BUTTON_TOKENS: [&[u8]; 4] = [b"*A*", b"*G*", b"*P*", b"*D*"];

/// `FUN_002459f8(hud, text, before, after)` [M]: the first token (in table order) found in the
/// text (strstr) splits it into the bytes before it and the bytes after its 3 characters.
pub fn split_button(text: &[u8]) -> Option<(usize, Vec<u8>, Vec<u8>)> {
    for (i, t) in BUTTON_TOKENS.iter().enumerate() {
        if let Some(p) = text.windows(3).position(|w| w == *t) {
            return Some((i, text[..p].to_vec(), text[p + 3..].to_vec()));
        }
    }
    None
}

impl DriveHudDraw {
    pub fn new() -> DriveHudDraw {
        DriveHudDraw {
            health_show: 3.0, health_fade: 0.0, health_blink: 0.0, last_health: 1.0, last_armour: 0.0, grunt_cycle: 0,
            msg_label: -1, msg_hold: 0.0, msg_fade: 0.0, msg_long: false,
            gadget_slot: 0, gadget_time: 0.0, gadget_slide: 0.0, gadget_blink: 0.0, gadget_blinks: 0,
            timer_beep_last: 10,
            banner_blinks: 0, banner_blink: 0.0, banner_time: 0.0, banner_label: -1, banner_kind: -1,
            banner_next_label: -1, banner_next_kind: -1, banner_flag: 0,
            stunt_state: 0, stunt_time: 0.0, stunt_scale: 0.0, stunt_fade: 0.0,
            prompt_text: None, prompt_label: 0, prompt_time: 0.0, prompt_pulse: 0.0,
        }
    }

    /// `FUN_00245890(duration, hud, label)` [M]: (only while HUD state +0x314 == 0 — the caller's
    /// check) hold = duration, label set, long = text longer than 32 bytes in 4:3. The fade timer
    /// is NOT reset.
    pub fn message_show(&mut self, duration: f32, label: i32, text_len: usize, widescreen: bool) {
        self.msg_hold = duration;
        self.msg_label = label;
        self.msg_long = !widescreen && text_len > 0x20;
    }

    /// `FUN_00244f48(duration, hud, slot)` [M]: slide 0.5, blink 0.5, two blinks.
    pub fn gadget_show(&mut self, duration: f32, slot: u32) {
        self.gadget_slot = slot;
        self.gadget_slide = 0.5;
        self.gadget_time = duration;
        self.gadget_blinks = 2;
        self.gadget_blink = 0.5;
    }

    /// `FUN_00245910(hud, kind, label, flag)` [M] (the caller's +0x314 == 0 check included):
    /// with no banner up it starts one (5 s, `SFX_misstatus`); with one up it queues the new one
    /// (replacing any queued one) — unless it is the same label and kind, which only cancels the
    /// prompt line. Every accepted call cancels the button prompt (+0x300 = 0).
    pub fn banner_show(&mut self, kind: i32, label: i32, flag: u32) -> Vec<&'static str> {
        let mut sounds = vec![];
        if self.banner_time > 0.0 {
            if self.banner_label == label && self.banner_kind == kind {
                self.prompt_time = 0.0;
                return sounds;
            }
            self.banner_next_label = label;
            self.banner_next_kind = kind;
        } else {
            self.banner_blinks = 0;
            self.banner_next_kind = -1;
            self.banner_label = label;
            self.banner_kind = kind;
            self.banner_blink = 0.0;
            self.banner_time = 5.0;
            self.banner_flag = flag;
            self.banner_next_label = -1;
            sounds.push("SFX_misstatus");
        }
        self.prompt_time = 0.0;
        sounds
    }

    /// `FUN_002459d0(duration, hud, text, label)` [M]: the prompt line for `duration` s (pulse
    /// 0.5); it also ends any objective banner (+0x3b0, +0x3a8, +0x3ac = 0).
    pub fn prompt_show(&mut self, duration: f32, text: Option<Vec<u8>>, label: i32) {
        self.prompt_text = text;
        self.prompt_label = label;
        self.prompt_time = duration;
        self.prompt_pulse = 0.5;
        self.banner_time = 0.0;
        self.banner_blinks = 0;
        self.banner_blink = 0.0;
    }

    /// `FUN_0023c820(hud)` [M]: start the stunt award (state 1, time 200 / rate, fade 0).
    pub fn stunt_show(&mut self, l: &HudLayout) {
        self.stunt_state = 1;
        self.stunt_fade = 0.0;
        self.stunt_time = 200.0 / l.hud_rate;
    }

    /// One HUD frame of the ported passes, in `FUN_0023cf88`'s order.
    pub fn frame(&mut self, l: &HudLayout, i: &HudInput, s: HudStrings) -> HudFrame {
        let mut f = HudFrame::default();
        let mut pen = Pen { s, scale: s.font.file_scale, widescreen: l.widescreen, screen_h: l.screen_h };
        self.banner(l, i, &mut pen, &mut f);
        self.stunt(l, i, &mut f);
        let gated = i.suppressed || self.msg_long;
        if !gated { Self::weapon_ammo(l, i, &mut pen, &mut f); }
        if !gated { self.health(l, i, &mut f); }
        self.message(l, i, &mut pen, &mut f);
        self.prompt(l, i, &mut pen, &mut f);
        self.gadget(l, i, &mut f);
        self.timer(l, i, &mut pen, &mut f);
        f
    }

    /// `FUN_00246c48` — the objective banner [M, disassembly 0x246c48-0x2470ec]: a box sized to
    /// the wider of header and body, the header (`OBJECTIVE COMPLETE` / `NEW OBJECTIVE` /
    /// `OBJECTIVE FAILED`) blinking in twice over 0.5 s each, then a sound and the body line (a
    /// complete / failed banner drops the body and shortens to 1 s). A queued banner starts 2 s
    /// after the current one ends.
    fn banner(&mut self, l: &HudLayout, i: &HudInput, pen: &mut Pen, out: &mut HudFrame) {
        if i.paused { return; }
        let q = self.banner_next_label;
        if q != -1 && self.banner_kind == -1 {
            self.banner_time = sub(self.banner_time, i.dt);
            if self.banner_time > 0.0 { return; }
            self.banner_blinks = 0;
            self.banner_label = q;
            self.banner_kind = self.banner_next_kind;
            self.banner_blink = 0.0;
            self.banner_time = 5.0;
            out.sounds.push("SFX_misstatus");
            self.banner_next_kind = -1;
            self.banner_next_label = -1;
            return;
        }
        if self.banner_time > 0.0 {
            let kind = self.banner_kind;
            let hs = pen.label(banner_header(kind));
            let bs = pen.label(self.banner_label);
            pen.bind(&l.obj_head);
            pen.set(SCALE_1_1);
            let (hw, hh) = (pen.width(&hs), pen.height(&hs));
            let (bw, bh) = (pen.width(&bs), pen.height(&bs));
            let m = if hw <= bw { bw } else { hw };
            let y0 = sub(l.obj_head.pos[1], 5.0);
            let w = add(m, 20.0);
            let bottom = add(add(add(l.obj_body.pos[1], 5.0), bh), hh);
            out.draws.push(HudDraw::Panel { x: sub(l.centre[0], mul(w, 0.5)), y: y0, w, h: sub(bottom, y0), colour: 0x2000_0000 });
            let mut f = 1.0f32;
            if (self.banner_blinks as f32) < 2.0 {
                let p = self.banner_blink;
                if p < 0.5 {
                    f = add(p, p);
                    self.banner_blink = add(p, i.dt);
                } else {
                    self.banner_blink = 0.0;
                    self.banner_blinks += 1;
                }
            }
            let x = pen.centred(SCALE_1_1, &l.obj_head, &hs, l.centre[0]);
            pen.text(out, &l.obj_head, x, l.obj_head.pos[1], SCALE_1_1, (rnd(mul(f, 70.0)) as u32) << 24,
                (rnd(mul(f, 127.0)) as u32) << 24 | 0x64_ccfe, Some(hs));
            let mut c = self.banner_blinks as f32;
            if c == 2.0 {
                out.sounds.push(if kind == 2 { "SFX_objcomp" } else { "SFX_objective" });
                self.banner_blinks += 1;
                if (kind as u32).wrapping_sub(2) < 2 { self.banner_time = 1.0; }
                c = self.banner_blinks as f32;
            }
            if 2.0 <= c {
                if (kind as u32).wrapping_sub(2) >= 2 {
                    let x = pen.centred(SCALE_1_1, &l.obj_body, &bs, l.centre[0]);
                    pen.text(out, &l.obj_body, x, add(l.obj_body.pos[1], 5.0), SCALE_1_1, TEXT_SHADOW, TEXT_COLOUR, Some(bs));
                }
                self.banner_time = sub(self.banner_time, i.dt);
            }
            if self.banner_time <= 0.0 {
                self.banner_time = 0.0;
                self.banner_flag = 0;
            }
        } else if q != -1 {
            self.banner_kind = -1;
            self.banner_time = 2.0;
        }
    }

    /// `FUN_002472b0` — the stunt award (`stylin`) [M, disassembly 0x2472b0-0x247610]: centred on
    /// (centre x, 95). State 1, while time > d = 100 / rate: scale x = (d − t′) / d, y = x ×
    /// cos(2π · frac(2t′ / d)) (`FUN_002cbeb0`), t′ = time − d; colour 0x7f676767. Then 3 s at the
    /// last scale (state 2), alpha from the fade only once its time is out.
    fn stunt(&mut self, l: &HudLayout, i: &HudInput, out: &mut HudFrame) {
        if self.stunt_state == 0 { return; }
        let d = 100.0 / l.hud_rate;
        let cx = l.centre[0];
        let [kw, kh] = l.stylin.key_size;
        if self.stunt_state == 1 {
            let t = self.stunt_time;
            if 0.0 < t && d < t {
                let tp = sub(t, d);
                let mut a = add(tp, tp) / d;
                let sx = sub(d, tp) / d;
                while 1.0 < a { a = sub(a, 1.0); }
                let sy = mul(sx, crate::driving::sub_physics::cos_turns(a));
                self.stunt_scale = sx;
                let x = sub(cx, mul(mul(kw, 0.5), sx)) as i32;
                let y = sub(95.0, mul(mul(sy, 0.5), kh)) as i32;
                out.draws.extend(l.stylin.rescaled([sx, sy]).draw_at(x, y, 0x7f67_6767));
                self.stunt_time = sub(self.stunt_time, i.dt);
            } else {
                self.stunt_time = 3.0;
                self.stunt_state = 2;
            }
        }
        if self.stunt_state != 2 { return; }
        let mut f = 1.0f32;
        if self.stunt_time <= 0.0 { f = if 0.0 < self.stunt_fade { add(self.stunt_fade, self.stunt_fade) } else { 0.0 }; }
        let colour = (rnd(mul(f, 127.0)) as u32) << 24 | 0x67_6767;
        let h = mul(self.stunt_scale, 0.5);
        let x = sub(cx, mul(h, kw)) as i32;
        let y = sub(95.0, mul(h, kh)) as i32;
        out.draws.extend(l.stylin.rescaled([self.stunt_scale; 2]).draw_at(x, y, colour));
        if 0.0 < self.stunt_fade {
            // The fade drops by the tick count, not by seconds [M 0x24754c-0x24758c].
            self.stunt_fade = sub(self.stunt_fade, i.ticks as f32);
            if self.stunt_fade <= 0.0 { self.stunt_fade = 0.0; }
        }
        if 0.0 < self.stunt_time {
            self.stunt_time = sub(self.stunt_time, i.dt);
            if self.stunt_time <= 0.0 {
                self.stunt_time = 0.0;
                self.stunt_fade = 0.5;
                self.stunt_state = 0;
            }
        }
    }

    /// `FUN_00240c98` [M, disassembly 0x240c98-0x240dfc]: ammo measured at scale 1.0
    /// (`FUN_002378a0(1, 1)`), the weapon label measured at the `weapon` element's bound scale;
    /// both right-aligned on their element x and drawn at 1.0.
    fn weapon_ammo(l: &HudLayout, i: &HudInput, pen: &mut Pen, f: &mut HudFrame) {
        let Some((id, ammo)) = i.weapon else { return };
        // Ammo is drawn unless the weapon id is 0x64b with ammo != 0; negative ammo prints 0.
        if !(id == 0x64b && ammo != 0) {
            let t = format!("{}", ammo.max(0)).into_bytes();
            pen.set(1.0);
            let w = pen.width(&t);
            pen.text(f, &l.ammo, sub(l.ammo.pos[0], w), l.ammo.pos[1], 1.0, TEXT_SHADOW, TEXT_COLOUR, Some(t));
        }
        let t = pen.label(id);
        pen.bind(&l.weapon);
        let w = pen.width(&t);
        pen.text(f, &l.weapon, sub(l.weapon.pos[0], w), l.weapon.pos[1], 1.0, TEXT_SHADOW, TEXT_COLOUR, Some(t));
    }

    /// `FUN_0023da30` — the 007 iris (health) and armour meter [M].
    fn health(&mut self, l: &HudLayout, i: &HudInput, out: &mut HudFrame) {
        let h = i.health / i.health_max;
        let a = i.armour / i.armour_max;
        let mut f = 1.0f32;
        let mut t = self.health_blink;
        if !i.health_always && self.health_show <= 0.0 {
            f = if self.health_fade > 0.0 { self.health_fade * 2.0 } else { 0.0 };
            t = self.health_blink;
        }
        if t > 0.0 { f = t * 2.0; }
        let hf = (h * 7.0) as i32;
        let af = (a * 8.0) as i32;
        let (mut n_iris, mut n_arm) = (hf, af);
        if h <= 0.0 { f = 1.0; n_iris = 0; n_arm = 0; }
        if f > 0.0 {
            let (full, dim) = if h > 0.5 { IRIS_COLOURS[0] } else if h > 0.25 { IRIS_COLOURS[1] } else { IRIS_COLOURS[2] };
            let full_a = rnd((full >> 24) as f32 * f) as u32;
            let full_c = full & 0xff_ffff | full_a << 24;
            let dim_c = dim & 0xff_ffff | (rnd((dim >> 24) as f32 * f) as u32) << 24;
            for s in &l.iris { out.draws.extend(s.draw(dim_c)); }
            for s in l.iris.iter().take(n_iris.max(0) as usize) { out.draws.extend(s.draw(full_c)); }
            if h < 1.0 {
                let pa = rnd((full_a & 0xff) as f32 * (h * 7.0 - hf as f32)) as u32;
                if let Some(s) = l.iris.get(n_iris.max(0) as usize) { out.draws.extend(s.draw(full & 0xff_ffff | pa << 24)); }
            }
            let arm_c = (rnd(f * 127.0) as u32) << 24 | 0xff_ffff;
            for s in l.arms.iter().take(n_arm.max(0) as usize) { out.draws.extend(s.draw(arm_c)); }
            if a < 1.0 {
                let pa = (rnd((a * 8.0 - af as f32) * 127.0) & 0xff) as f32 * f;
                if let Some(s) = l.arms.get(n_arm.max(0) as usize) { out.draws.extend(s.draw((rnd(pa) as u32) << 24 | 0xff_ffff)); }
            }
            out.draws.extend(l.health_border.draw(full_c));
        }
        if i.grunt_on_hit && ((h * 32.0) as i32) < ((self.last_health * 32.0) as i32) {
            const GRUNTS: [&str; 3] = ["SFX_Bond_body_hit", "SFX_Bond_body_hit2", "SFX_Bond_body_hit3"];
            out.sounds.push(GRUNTS[self.grunt_cycle % 3]);
            self.grunt_cycle = (self.grunt_cycle + 1) % 3;
        }
        if self.health_show > 0.0 {
            self.health_show -= i.dt;
            if self.health_show <= 0.0 { self.health_show = 0.0; self.health_fade = 0.5; }
        }
        if self.health_fade > 0.0 {
            self.health_fade -= i.dt;
            if self.health_fade <= 0.0 { self.health_fade = 0.0; }
        }
        if h <= 0.25 {
            self.health_blink -= i.dt;
            if self.health_blink <= 0.0 { self.health_blink = 0.5; }
        } else {
            self.health_blink = 0.0;
        }
        if h != self.last_health || a != self.last_armour {
            self.last_health = h;
            self.health_show = 3.0;
            self.last_armour = a;
        }
    }

    /// `FUN_002469b8` — the message line [M, disassembly 0x246a3c-0x246b98]: x = centre − width
    /// × 0.5 at scale 1.2; a box at (x − 4, 376), width + 12 wide, (height + 10) + 8 high.
    fn message(&mut self, l: &HudLayout, i: &HudInput, pen: &mut Pen, out: &mut HudFrame) {
        let mut f = 1.0f32;
        if self.msg_hold <= 0.0 {
            if self.msg_fade > 0.0 { f = self.msg_fade * 2.0 } else { self.msg_long = false; f = 0.0 }
        }
        if f > 0.0 {
            let t = pen.label(self.msg_label);
            let x = pen.centred(SCALE_1_2, &l.msg_txt, &t, l.centre[0]);
            pen.bind(&l.msg_txt);
            pen.set(SCALE_1_2);
            let (w, h) = (pen.width(&t), pen.height(&t));
            out.draws.push(HudDraw::Panel { x: sub(x, 4.0), y: 376.0, w: add(w, 12.0), h: add(add(h, 10.0), 8.0), colour: 0x2000_0000 });
            pen.text(out, &l.msg_txt, x, 380.0, SCALE_1_2, (rnd(f * 32.0) as u32) << 24, (rnd(f * 127.0) as u32) << 24 | 0xb3_dbfa, Some(t));
        }
        if self.msg_fade > 0.0 {
            self.msg_fade -= i.dt;
            if self.msg_fade <= 0.0 { self.msg_fade = 0.0; self.msg_long = false; }
        }
        if self.msg_hold > 0.0 {
            self.msg_hold -= i.dt;
            if self.msg_hold <= 0.0 { self.msg_hold = 0.0; self.msg_fade = 0.5; }
        }
    }

    /// `FUN_00245ab8` — the button prompt line [M, disassembly 0x245ab8-0x245eac]: centred at
    /// y 100, scale 1.4, alpha pulsing from 2 × pulse (0.5 → 0, re-armed). A token (`*A*` …)
    /// splits it: the part before, a button icon at (x + 5, 100 − 0.2 × icon height), the part
    /// after — each step measured at the scale the previous draw left (× 1.3333333 for the part
    /// before in widescreen) plus the width of `"s"` as a gap. Without a token the game passes no
    /// text, so `FUN_00244398` draws the `buttonTxt` element's own label (−1 → `"MISSING"`) [M].
    fn prompt(&mut self, l: &HudLayout, i: &HudInput, pen: &mut Pen, out: &mut HudFrame) {
        if self.prompt_time <= 0.0 { return; }
        let f = add(self.prompt_pulse, self.prompt_pulse);
        let colour = (rnd(mul(f, 127.0)) as u32) << 24 | 0xb3_dbfa;
        let shadow = (rnd(mul(f, 70.0)) as u32) << 24;
        let text = if self.prompt_label > 0 { Some(pen.label(self.prompt_label)) } else { self.prompt_text.clone() };
        let Some(text) = text else {
            // "Could not draw button text because the string is NULL!" → FUN_00245eb0 clears it.
            self.prompt_pulse = 0.0;
            self.prompt_text = None;
            self.prompt_time = 0.0;
            return;
        };
        let mut x = pen.centred(SCALE_1_4, &l.button_txt, &text, l.centre[0]);
        let a = &l.button_txt;
        match split_button(&text) {
            None => pen.text(out, a, x, 100.0, SCALE_1_4, shadow, colour, None),
            Some((k, before, after)) => {
                let icon_c = (rnd(mul(f, 127.0)) as u32) << 24 | 0xff_ffff;
                pen.text(out, a, x, 100.0, SCALE_1_4, shadow, colour, Some(before.clone()));
                let mut w = pen.width(&before);
                if l.widescreen { w = mul(w, f32::from_bits(0x3faa_aaaa)); }
                x = add(x, w);
                x = add(x, pen.width(b"s"));
                let icon = &l.buttons[k];
                out.draws.extend(icon.draw_at(add(x, 5.0) as i32, sub(100.0, mul(icon.key_size[1], f32::from_bits(0x3e4c_cccd))) as i32, icon_c));
                x = add(x, icon.key_size[0]);
                let x2 = add(x, pen.width(b"s"));
                pen.text(out, a, x2, 100.0, SCALE_1_4, shadow, colour, Some(after));
            }
        }
        self.prompt_pulse = sub(self.prompt_pulse, i.dt);
        if self.prompt_pulse <= 0.0 { self.prompt_pulse = 0.5; }
        self.prompt_time = sub(self.prompt_time, i.dt);
        if self.prompt_time <= 0.0 { self.prompt_time = 0.0; }
    }

    /// `FUN_00244f88` — the gadget pop-up: shown at the screen centre, blinks twice, then slides
    /// to its corner with a smoothstep [M].
    fn gadget(&mut self, l: &HudLayout, i: &HudInput, out: &mut HudFrame) {
        if self.gadget_time <= 0.0 { return; }
        // slot < 6 → gadget[slot]; else hud+0x84 = gadget3 [M].
        let s = l.gadgets.get(self.gadget_slot as usize).filter(|_| self.gadget_slot < 6).unwrap_or(&l.gadgets[3]);
        let alpha = rnd(self.gadget_blink * 127.0 * 2.0) as u32;
        let u = self.gadget_slide * 2.0;
        let t = (3.0 - (u + u)) * u * u;
        let x = t * (l.centre[0] - s.size[0] * 0.5) + (1.0 - t) * s.pos[0];
        let y = t * (l.centre[1] - s.size[1]) + (1.0 - t) * s.pos[1];
        out.draws.extend(s.draw_at(x as i32, y as i32, alpha << 24 | 0x7f_7f7f));
        self.gadget_time -= i.dt;
        if self.gadget_time <= 0.0 { self.gadget_time = 0.0; }
        if self.gadget_blinks < 1 { self.gadget_slide -= i.dt; }
        if self.gadget_slide <= 0.0 { self.gadget_slide = 0.0; }
        if self.gadget_blinks > 0 { self.gadget_blink -= i.dt; }
        if self.gadget_blink <= 0.0 { self.gadget_blink = 0.5; self.gadget_blinks -= 1; }
    }

    /// `FUN_0023e580` — the mission clock [M, disassembly 0x23e580-0x23e7b0]: `"00:00"` measured
    /// at 1.4 after binding `scratchTxt`; box (x, 33, width, 2 × height − 2) with x = hud+0x44c ×
    /// 0.5 − width × 0.5; the `"%2d:%02d"` text at (x, 35).
    fn timer(&mut self, l: &HudLayout, i: &HudInput, pen: &mut Pen, out: &mut HudFrame) {
        let Some((secs, countdown)) = i.timer else { return };
        if secs <= 0 { return; }
        let zero = b"00:00";
        pen.bind(&l.timer_font);
        pen.set(SCALE_1_4);
        let (w, h) = (pen.width(zero), pen.height(zero));
        let x = sub(l.centre[0], mul(w, 0.5));
        out.draws.push(HudDraw::Panel { x, y: 33.0, w, h: sub(add(h, h), 2.0), colour: 0x4605_050f });
        let (m, s) = (secs / 60, secs % 60);
        let red = countdown && m == 0 && s < 10;
        pen.text(out, &l.timer_font, x, 35.0, SCALE_1_4, TEXT_SHADOW, if red { 0x7f05_05c8 } else { TEXT_COLOUR },
            Some(format!("{m:2}:{s:02}").into_bytes()));
        if red && s == self.timer_beep_last - 1 {
            out.sounds.push("SFX_timerbeep");
            self.timer_beep_last = s;
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A font with fixed-width glyphs for every printable byte: w = 8, advance 10, x offset 1,
    /// y offset 2, h 12; 'W' is 16 wide (advance 18); one kern pair (A then V: −3); line 18 + 5.
    pub(crate) fn font() -> Font {
        let n = 0x7f - 0x20 + 1;
        let go = 0x80usize;
        let ko = go + n * 12;
        let so = (ko + 8 + 15) & !15;
        let mut f = vec![0u8; so + 0x20 + 16 + 1024];
        f[..4].copy_from_slice(b"FNTS");
        f[0xa..0xc].copy_from_slice(&(n as u16).to_le_bytes());
        f[0x12] = 18;
        f[0x13] = 5;
        f[0x14..0x18].copy_from_slice(&(go as u32).to_le_bytes());
        f[0x18..0x1c].copy_from_slice(&(ko as u32).to_le_bytes());
        f[0x1c..0x20].copy_from_slice(&(so as u32).to_le_bytes());
        f[0x30..0x34].copy_from_slice(&0.5f32.to_le_bytes());
        f[0x34..0x38].copy_from_slice(&0.5f32.to_le_bytes());
        for i in 0..n {
            let c = 0x20 + i as u8;
            let (w, adv) = if c == b'W' { (16u8, 18u8) } else if c == b' ' { (0, 6) } else { (8, 10) };
            let kerned = (c == b'V') as u8;
            f[go + i * 12..go + i * 12 + 12].copy_from_slice(&[c, 0, w, 12, i as u8, 0, 0, 0, adv, 1, 2, kerned]);
        }
        f[ko..ko + 4].copy_from_slice(&1u32.to_le_bytes());
        f[ko + 4..ko + 8].copy_from_slice(&[b'A', 0, (-3i8) as u8, b'V']);
        f[so..so + 4].copy_from_slice(&(2u32 | 0x20 << 8).to_le_bytes());
        f[so + 4..so + 8].copy_from_slice(&[4, 0, 4, 0]);
        f[so + 0x20..so + 0x24].copy_from_slice(&0x21u32.to_le_bytes());
        Font::parse(&f).unwrap()
    }

    /// A string table: id 7 "Machine Gun", 42 "Hello", 99999 "OBJECTIVE COMPLETE", 99998
    /// "NEW OBJECTIVE", 99736 "OBJECTIVE FAILED", 500 "Find the car", 600 "Press *P* to Fire",
    /// 601 "Press *X* now".
    pub(crate) fn loc() -> Loc {
        let strings: [(u16, &[u8]); 8] = [(7, b"Machine Gun"), (42, b"Hello"), (500, b"Find the car"), (600, b"Press *P* to Fire"),
            (601, b"Press *X* now"), (0x8598, b"OBJECTIVE FAILED"), (0x869e, b"NEW OBJECTIVE"), (0x869f, b"OBJECTIVE COMPLETE")];
        let n = strings.len();
        let mut f = vec![0u8; 0x14];
        f[..4].copy_from_slice(b"LOCH");
        f[4..8].copy_from_slice(&0x14u32.to_le_bytes());
        f[8] = 1;
        f[0xc..0xe].copy_from_slice(&1u16.to_le_bytes());
        let mut t = vec![0u8; 0x10];
        t[..4].copy_from_slice(b"LOCI");
        t[8..12].copy_from_slice(&(n as u32).to_le_bytes());
        for (i, (id, _)) in strings.iter().enumerate() { t.extend_from_slice(&id.to_le_bytes()); t.extend_from_slice(&(i as u16).to_le_bytes()); }
        f.extend_from_slice(&t);
        let s0 = f.len();
        f[0x10..0x14].copy_from_slice(&(s0 as u32).to_le_bytes());
        let mut s = vec![0u8; 0x10 + n * 4];
        s[..4].copy_from_slice(b"LOCL");
        s[0xc..0x10].copy_from_slice(&(n as u32).to_le_bytes());
        for (i, (_, b)) in strings.iter().enumerate() {
            let o = s.len() as u32;
            s[0x10 + i * 4..0x14 + i * 4].copy_from_slice(&o.to_le_bytes());
            s.extend_from_slice(b);
            s.push(0);
        }
        f.extend_from_slice(&s);
        Loc::parse(&f).unwrap()
    }

    fn sp(name: &str, x: f32, y: f32) -> Sprite { Sprite { name: name.into(), pos: [x, y], size: [10.0, 10.0], key_size: [10.0, 10.0], uv: [0.0; 4], sheet: 0, mirror: 0, repeats: [0; 4] } }
    fn ta(name: &str, x: f32, y: f32) -> TextAnchor { TextAnchor { name: name.into(), pos: [x, y], scale: [1.0, 1.0], label: -1 } }
    pub(crate) fn layout() -> HudLayout {
        HudLayout {
            weapon: ta("weapon", 605.0, 410.0),
            ammo: ta("ammo", 605.0, 390.0),
            stylin: Sprite { size: [122.0 * 1.3, 41.0 * 1.3], key_size: [122.0, 41.0], ..sp("stylin", 510.0, 40.0) },
            iris: (1..=7).map(|i| sp(&format!("iris{i}"), i as f32, 0.0)).collect(),
            arms: (0..8).map(|i| sp(&format!("arm{i}"), i as f32, 1.0)).collect(),
            health_border: sp("healthBorder", 68.0, 394.0),
            gadgets: (0..6).map(|i| sp(&format!("gadget{i}"), 540.0, 340.0)).collect(),
            msg_txt: ta("txt", 260.0, 54.0),
            timer_font: ta("scratchTxt", 2.0, 0.0),
            obj_head: ta("objobjtxt", 180.0, 108.0),
            obj_body: ta("objbody", 180.0, 126.0),
            button_txt: TextAnchor { scale: [1.25, 1.25], ..ta("buttonTxt", 228.0, 152.0) },
            buttons: [Sprite { key_size: [50.0, 51.0], size: [50.0, 51.0], ..sp("PS2X", 0.0, 0.0) }, sp("PS2L1", 0.0, 0.0),
                Sprite { key_size: [55.0, 34.0], size: [55.0, 34.0], ..sp("PS2R1", 0.0, 0.0) }, sp("PS2R1", 0.0, 0.0)],
            centre: [320.0, 224.0],
            widescreen: false,
            screen_h: 448.0,
            hud_rate: 30.0,
        }
    }
    fn input(h: f32, a: f32) -> HudInput {
        HudInput { dt: 1.0 / 60.0, ticks: 1, health: h, health_max: 1.0, armour: a, armour_max: 1.0, weapon: None, timer: None,
            health_always: false, grunt_on_hit: false, suppressed: false, paused: false }
    }
    fn sprites(f: &HudFrame) -> Vec<(String, u32)> {
        f.draws.iter().filter_map(|d| match d { HudDraw::Sprite { element, colour, .. } => Some((element.clone(), *colour)), _ => None }).collect()
    }
    fn texts(f: &HudFrame) -> Vec<(String, String, f32, f32, u32)> {
        f.draws.iter().filter_map(|d| match d {
            HudDraw::Text { element, text, x, y, colour, .. } => Some((element.clone(), String::from_utf8_lossy(text).into_owned(), *x, *y, *colour)),
            _ => None,
        }).collect()
    }
    fn run(d: &mut DriveHudDraw, l: &HudLayout, i: &HudInput) -> HudFrame {
        let (f, s) = (font(), loc());
        d.frame(l, i, HudStrings { font: &f, loc: &s })
    }

    /// Health 0.6 / armour 0.3 [M 0x23da30]: 7 dim iris (0x13 alpha), 4 full (0x46, > 0.5
    /// colour), one partial at alpha round(0x46 × (4.2 − 4)) = 14, 2 full arms (a007, arm1) at
    /// 0x7f, the partial arm at round(round(0.4 × 127) × 1) = 51, then the border in full colour.
    #[test]
    fn health_meter_segments_follow_fun_0023da30() {
        let l = layout();
        let mut d = DriveHudDraw::new();
        let f = run(&mut d, &l, &input(0.6, 0.3));
        let s = sprites(&f);
        assert_eq!(s.len(), 7 + 4 + 1 + 2 + 1 + 1);
        assert!(s[..7].iter().all(|(_, c)| *c == 0x138b_a852));
        assert!(s[7..11].iter().all(|(_, c)| *c == 0x468b_a852));
        assert_eq!(s[11], ("iris5".into(), 0x008b_a852 | (rnd(70.0 * (0.6f32 * 7.0 - 4.0)) as u32) << 24));
        assert_eq!(s[12], ("arm0".into(), 0x7fff_ffff));
        assert_eq!(s[13], ("arm1".into(), 0x7fff_ffff));
        assert_eq!(s[14].0, "arm2");
        assert_eq!(s[14].1 >> 24, rnd((0.3f32 * 8.0 - 2.0) * 127.0) as u32);
        assert_eq!(s[15], ("healthBorder".into(), 0x468b_a852));
        // Sister: the colour band at 0.25 < h <= 0.5 is the second pair.
        let f = run(&mut DriveHudDraw::new(), &l, &input(0.5, 1.0));
        assert_eq!(sprites(&f)[0].1, 0x134e_98c6);
    }

    /// After 3 s without change the meter fades over 0.5 s and then is hidden; a change re-arms
    /// the 3 s show [M 0x23da30 tail].
    #[test]
    fn health_meter_hides_after_three_seconds_and_a_change_shows_it() {
        let l = layout();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 0.0);
        d.last_armour = 0.0;
        i.dt = 0.25;
        for _ in 0..12 { assert!(!sprites(&run(&mut d, &l, &i)).is_empty()); }
        assert_eq!((d.health_show, d.health_fade), (0.0, 0.25));
        let f = run(&mut d, &l, &i);
        assert_eq!(sprites(&f)[0].1 >> 24, 10, "f = 2 × 0.25: dim alpha round(0x13 × 0.5) = 10");
        assert!(sprites(&run(&mut d, &l, &i)).is_empty(), "hidden once the fade ran out");
        i.health = 0.9;
        run(&mut d, &l, &i);
        assert_eq!(d.health_show, 3.0, "a change re-arms the show timer");
        let mut d2 = DriveHudDraw { health_show: 0.0, ..DriveHudDraw::new() };
        let mut i2 = input(1.0, 0.0);
        i2.health_always = true;
        assert!(!sprites(&run(&mut d2, &l, &i2)).is_empty());
    }

    /// The clock [M 0x23e580]: "%2d:%02d" placed by the width of "00:00" at 1.4, red + beep in
    /// the last 10 s of a countdown, hidden at 0.
    #[test]
    fn timer_face_follows_fun_0023e580() {
        let l = layout();
        let fnt = font();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 0.0);
        i.suppressed = true;
        i.timer = Some((75, true));
        let f = run(&mut d, &l, &i);
        let w = fnt.text_width(b"00:00", [SCALE_1_4; 2]);
        let x = sub(320.0, mul(w, 0.5));
        assert_eq!(texts(&f), vec![("scratchTxt".to_owned(), " 1:15".to_owned(), x, 35.0, TEXT_COLOUR)]);
        let h = fnt.text_height(b"00:00", [SCALE_1_4; 2]);
        assert!(f.draws.contains(&HudDraw::Panel { x, y: 33.0, w, h: sub(add(h, h), 2.0), colour: 0x4605_050f }));
        i.timer = Some((9, true));
        let f = run(&mut d, &l, &i);
        assert!(f.draws.iter().any(|x| matches!(x, HudDraw::Text { colour: 0x7f05_05c8, .. })));
        assert_eq!(f.sounds, vec!["SFX_timerbeep"], "9 = 10 − 1 beeps");
        assert!(run(&mut d, &l, &i).sounds.is_empty(), "the same second does not beep twice");
        i.timer = Some((9, false));
        assert!(!run(&mut d, &l, &i).draws.iter().any(|x| matches!(x, HudDraw::Text { colour: 0x7f05_05c8, .. })), "a count-up never turns red");
        i.timer = Some((0, true));
        assert!(run(&mut d, &l, &i).draws.is_empty(), "0 s draws nothing");
    }

    /// Weapon + ammo [M 0x240c98]: right-aligned at the element x by the measured width, ammo
    /// floored at 0, the 0x64b id hides a non-zero ammo count, the label from the string table.
    #[test]
    fn weapon_and_ammo_lines() {
        let l = layout();
        let fnt = font();
        let mut i = input(1.0, 1.0);
        i.weapon = Some((7, -3));
        let f = run(&mut DriveHudDraw::new(), &l, &i);
        let t = texts(&f);
        assert_eq!(t[0], ("ammo".into(), "0".into(), 605.0 - 8.0, 390.0, TEXT_COLOUR), "\"0\": box x 1..9 → width 8");
        let w = fnt.text_width(b"Machine Gun", [1.0, 1.0]);
        assert_eq!(t[1], ("weapon".into(), "Machine Gun".into(), 605.0 - w, 410.0, TEXT_COLOUR));
        assert_eq!(w, 104.0, "box 1..105: ten 10-advances and one 6-wide space before the last n's 1 + 8");
        i.weapon = Some((0x64b, 5));
        let f = run(&mut DriveHudDraw::new(), &l, &i);
        assert!(!texts(&f).iter().any(|t| t.0 == "ammo"));
        assert_eq!(texts(&f)[0].1, "MISSING", "an id the table lacks prints MISSING");
        i.weapon = Some((0x64b, 0));
        let f = run(&mut DriveHudDraw::new(), &l, &i);
        assert!(texts(&f).iter().any(|t| t.0 == "ammo"), "0x64b with 0 ammo still prints 0");
    }

    /// A long message hides weapon/ammo and health until its fade ends [M 0x245890, 0x2469b8];
    /// the box is (x − 4, 376, w + 12, (h + 10) + 8) around the centred text.
    #[test]
    fn a_long_message_hides_the_meters_until_it_fades() {
        let l = layout();
        let fnt = font();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 1.0);
        i.weapon = Some((1, 10));
        i.dt = 0.25;
        d.message_show(0.5, 42, 33, false);
        let f = run(&mut d, &l, &i);
        assert!(sprites(&f).is_empty() && texts(&f).iter().all(|t| t.0 != "ammo"));
        let s = [SCALE_1_2; 2];
        let (w, h) = (fnt.text_width(b"Hello", s), fnt.text_height(b"Hello", s));
        let x = sub(320.0, mul(w, 0.5));
        assert_eq!(f.draws[0], HudDraw::Panel { x: sub(x, 4.0), y: 376.0, w: add(w, 12.0), h: add(add(h, 10.0), 8.0), colour: 0x2000_0000 });
        assert_eq!(f.draws[1], HudDraw::Text { element: "txt".into(), text: b"Hello".to_vec(), x, y: 380.0, scale: SCALE_1_2, colour: 0x7fb3_dbfa, shadow: 0x2000_0000 });
        run(&mut d, &l, &i);
        run(&mut d, &l, &i);
        let f = run(&mut d, &l, &i);
        assert!(texts(&f).iter().any(|t| t.4 == (64u32 << 24 | 0xb3_dbfa)), "f = 0.5 → alpha round(63.5) = 64");
        assert!(!d.msg_long);
        assert!(!sprites(&run(&mut d, &l, &i)).is_empty(), "meters back");
        let mut d = DriveHudDraw::new();
        d.message_show(0.5, 1, 32, false);
        assert!(!d.msg_long);
    }

    /// The gadget pop-up starts centred (t = 1), blinks twice, then slides home [M 0x244f88].
    #[test]
    fn gadget_pop_up_centres_blinks_then_slides_home() {
        let l = layout();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 1.0);
        i.suppressed = true;
        i.dt = 0.25;
        d.gadget_show(5.0, 2);
        let pos = |f: &HudFrame| f.draws.iter().find_map(|x| match x { HudDraw::Sprite { element, x, y, colour, .. } if element == "gadget2" => Some((*x as i32, *y as i32, *colour >> 24)), _ => None }).unwrap();
        assert_eq!(pos(&run(&mut d, &l, &i)), (315, 214, 127), "centre − (w/2, h) at full alpha");
        for _ in 0..3 { run(&mut d, &l, &i); }
        assert_eq!(d.gadget_blinks, 0, "two 0.5 s blinks");
        assert_eq!(pos(&run(&mut d, &l, &i)).0, 315, "still centred: the slide starts after the blinks");
        run(&mut d, &l, &i);
        assert_eq!(pos(&run(&mut d, &l, &i)), (540, 340, 127), "home once the slide ran out");
    }

    /// `FUN_00238d40`'s mirror and repeat expansion [M].
    #[test]
    fn element_quads_follow_fun_00238d40() {
        let mut s = sp("b", 100.0, 50.0);
        s.uv = [0.25, 0.5, 0.0, 1.0];
        let xy = |v: Vec<HudDraw>| v.into_iter().map(|d| match d { HudDraw::Sprite { x, y, uv, .. } => (x, y, uv), _ => unreachable!() }).collect::<Vec<_>>();
        const D: f32 = 1.0 / 512.0;
        assert_eq!(xy(s.draw(0)), vec![(100.0, 50.0, [0.25 + D, 0.0 + D, 0.5 - D, 1.0 - D])]);
        s.mirror = 3;
        let q = xy(s.draw(0));
        assert_eq!(q.iter().map(|&(x, y, _)| (x, y)).collect::<Vec<_>>(), vec![(100.0, 50.0), (90.0, 40.0), (90.0, 50.0), (100.0, 40.0)]);
        assert_eq!(q[2].2, [0.5 - D, 0.0 + D, 0.25 + D, 1.0 - D], "x-flipped: u swapped");
        s.mirror = 1;
        assert_eq!(xy(s.draw(0)).iter().map(|&(x, y, _)| (x, y)).collect::<Vec<_>>(), vec![(90.0, 50.0)]);
        s.mirror = 0;
        s.repeats = [0, 3, 0, 0];
        assert_eq!(xy(s.draw(0)).iter().map(|&(x, _, _)| x).collect::<Vec<_>>(), vec![100.0, 110.0, 120.0]);
        s.repeats = [0, 0, -1, 0];
        assert!(s.draw(0).is_empty());
    }

    /// The objective banner [M 0x246c48]: box sized to the wider line, the header blinking in
    /// twice (alpha 2 × clock), then `SFX_objective` and the body; a second banner queues and
    /// starts 2 s after the first ends.
    #[test]
    fn objective_banner_blinks_then_shows_the_body_and_queues() {
        let l = layout();
        let fnt = font();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 1.0);
        i.suppressed = true;
        i.dt = 0.25;
        assert_eq!(d.banner_show(1, 500, 0), vec!["SFX_misstatus"]);
        let f = run(&mut d, &l, &i);
        let s = [SCALE_1_1; 2];
        let m = fnt.text_width(b"NEW OBJECTIVE", s).max(fnt.text_width(b"Find the car", s));
        let w = add(m, 20.0);
        let HudDraw::Panel { x, y, w: pw, h, .. } = f.draws[0] else { panic!("box first") };
        assert_eq!((x, y, pw), (sub(320.0, mul(w, 0.5)), 103.0, w));
        assert_eq!(h, sub(add(add(131.0, fnt.text_height(b"Find the car", s)), fnt.text_height(b"NEW OBJECTIVE", s)), 103.0));
        let t = texts(&f);
        assert_eq!(t.len(), 1);
        assert_eq!((t[0].1.as_str(), t[0].4), ("NEW OBJECTIVE", 0x0064_ccfe), "blink clock 0 → alpha 0");
        let f = run(&mut d, &l, &i);
        assert_eq!(texts(&f)[0].4 >> 24, 64, "clock 0.25 → f 0.5");
        run(&mut d, &l, &i); // clock 0.5 → blink 1 done
        run(&mut d, &l, &i);
        run(&mut d, &l, &i);
        let f = run(&mut d, &l, &i); // blink 2 done → this frame: sound + body
        assert_eq!(f.sounds, vec!["SFX_objective"]);
        assert_eq!(texts(&f).iter().map(|t| t.1.as_str()).collect::<Vec<_>>(), vec!["NEW OBJECTIVE", "Find the car"]);
        assert_eq!(texts(&f)[1].3, 131.0, "body at objbody y + 5");
        // Queue a completion; it waits for this one (5 s) to end, then 2 s more.
        assert!(d.banner_show(2, 42, 0).is_empty());
        let mut frames = 0;
        while d.banner_kind != 2 { run(&mut d, &l, &i); frames += 1; assert!(frames < 100); }
        assert_eq!(d.banner_label, 42);
        let f = run(&mut d, &l, &i);
        assert_eq!(texts(&f)[0].1, "OBJECTIVE COMPLETE");
        // Sister: paused, nothing moves.
        let before = d.clone();
        i.paused = true;
        assert!(run(&mut d, &l, &i).draws.is_empty());
        assert_eq!(d, before);
    }

    /// The prompt line [M 0x245ab8]: a token splits it around a button icon; without a token
    /// the element's own label is drawn.
    #[test]
    fn button_prompt_splits_around_the_icon() {
        let l = layout();
        let fnt = font();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 1.0);
        i.suppressed = true;
        d.prompt_show(2.0, None, 600);
        let f = run(&mut d, &l, &i);
        let s = [SCALE_1_4; 2];
        let x = sub(320.0, mul(fnt.text_width(b"Press *P* to Fire", s), 0.5));
        let t = texts(&f);
        assert_eq!(t.iter().map(|t| t.1.as_str()).collect::<Vec<_>>(), vec!["Press ", " to Fire"]);
        assert_eq!((t[0].2, t[0].3, t[0].4), (x, 100.0, 0x7fb3_dbfa), "pulse 0.5 → f 1");
        // After the first draw the font holds (1.4, 1.4 × 448/480); widths are measured there.
        let held = draw_transform(x, 100.0, s, false, 448.0).1;
        let x1 = add(add(x, fnt.text_width(b"Press ", held)), fnt.text_width(b"s", held));
        let icon = f.draws.iter().find_map(|d| match d { HudDraw::Sprite { element, x, y, .. } => Some((element.clone(), *x, *y)), _ => None }).unwrap();
        assert_eq!(icon, ("PS2R1".into(), add(x1, 5.0) as i32 as f32, sub(100.0, mul(34.0, f32::from_bits(0x3e4c_cccd))) as i32 as f32));
        assert_eq!(t[1].2, add(add(x1, 55.0), fnt.text_width(b"s", held)));
        // No token: the buttonTxt label (−1) → MISSING.
        let mut d = DriveHudDraw::new();
        d.prompt_show(2.0, None, 601);
        assert_eq!(texts(&run(&mut d, &l, &i)).iter().map(|t| t.1.clone()).collect::<Vec<_>>(), vec!["MISSING".to_owned()]);
        // Widescreen: the before-part width is scaled by 1.3333333 after the 0.75 draw scale.
        let mut lw = layout();
        lw.widescreen = true;
        let mut d = DriveHudDraw::new();
        d.prompt_show(2.0, None, 600);
        let f = run(&mut d, &lw, &i);
        let held = draw_transform(x, 100.0, s, true, 448.0).1;
        assert_eq!(held[0], mul(SCALE_1_4, 0.75));
        let x1 = add(add(x, mul(fnt.text_width(b"Press ", held), f32::from_bits(0x3faa_aaaa))), fnt.text_width(b"s", held));
        assert!(f.draws.iter().any(|d| matches!(d, HudDraw::Sprite { x, .. } if *x == add(x1, 5.0) as i32 as f32)));
        // A banner cancels the prompt.
        let mut d = DriveHudDraw::new();
        d.prompt_show(2.0, None, 600);
        d.banner_show(1, 500, 0);
        assert_eq!(d.prompt_time, 0.0);
    }

    /// The stunt award [M 0x2472b0]: 200 / rate s total; zooms for 100 / rate s (scale x grows
    /// 0 → 1, y = x × cos), then holds 3 s at the last scale, centred on (320, 95).
    #[test]
    fn stunt_award_zooms_then_holds() {
        let l = layout();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 1.0);
        i.suppressed = true;
        i.dt = 0.5;
        d.stunt_show(&l);
        assert_eq!(d.stunt_time, 200.0 / 30.0);
        let f = run(&mut d, &l, &i);
        // t′ = 20/3 − 10/3 = 10/3 → x scale 0, the bitmap collapses to 0 × 0 at the centre.
        let HudDraw::Sprite { x, y, w, colour, .. } = f.draws[0] else { panic!() };
        assert_eq!((x, y, w, colour), (320.0, 95.0, 0.0, 0x7f67_6767));
        let mut n = 1;
        while d.stunt_state == 1 { run(&mut d, &l, &i); n += 1; }
        assert_eq!(n, 8, "zoom frames while time > 10/3 (6.67 → 3.33 by 0.5), then the switch frame");
        assert_eq!(d.stunt_time, 3.0 - 0.5, "state 2 drew once in the switch frame");
        let f = run(&mut d, &l, &i);
        let HudDraw::Sprite { w, .. } = f.draws[0] else { panic!() };
        assert_eq!(w, 122.0 * d.stunt_scale);
        while d.stunt_state != 0 { run(&mut d, &l, &i); }
        assert!(run(&mut d, &l, &i).draws.is_empty());
    }

    #[test]
    fn button_tokens_follow_fun_002459f8() {
        assert_eq!(split_button(b"Press *D* to continue"), Some((3, b"Press ".to_vec(), b" to continue".to_vec())));
        assert_eq!(split_button(b"*G* *A*"), Some((0, b"*G* ".to_vec(), b"".to_vec())), "table order, not text order");
        assert_eq!(split_button(b"Press *X* now"), None);
    }

    /// `FUN_00237b80` [M]: y × 448/480, widescreen x scale × 0.75 and x pulled toward 320.
    #[test]
    fn draw_transform_follows_fun_00237b80() {
        let k = 448.0f32 / 480.0;
        assert_eq!(draw_transform(100.0, 35.0, [1.4, 1.4], false, 448.0), ([100.0, mul(35.0, k)], [1.4, mul(1.4, k)]));
        let ([x, _], [sx, _]) = draw_transform(0.0, 0.0, [1.0, 1.0], true, 448.0);
        assert_eq!((x, sx), (80.0, 0.75));
        let ([x, _], _) = draw_transform(640.0, 0.0, [1.0, 1.0], true, 448.0);
        assert_eq!(x, 560.0);
        let d = HudDraw::Text { element: "t".into(), text: b"A".to_vec(), x: 10.0, y: 20.0, scale: 1.0, colour: 0, shadow: 0 };
        let (sh, main) = text_glyphs(&d, &font(), false, 448.0);
        assert_eq!(main[0].xy[0], 11, "x 10 + x offset 1");
        assert_eq!(sh[0].xy[0], 12, "the shadow one pixel right");
    }
}
