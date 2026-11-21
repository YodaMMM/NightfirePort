//! The driving missions' HUD draw logic (DRIVING.ELF), headless: per frame it turns the game state
//! into a list of HUD draws (sprite / text / panel, with position, colour and text). **Nothing here
//! renders**; the draw side resolves fonts and textures. Data: the `hudNTSC.gal` gallery
//! (`nf_formats::gal`, docs/formats/hud-gal.md). System doc: docs/systems/driving-hud.md.
//!
//! Evidence (all [M] DRIVING.ELF unless noted; EE addresses, decompile `work/decomp/DRIVING.ELF/`,
//! disassembly checked where the decompile drops arguments):
//! - `FUN_0023ae30` builds the HUD: loads `hudNTSC.gal`, finds the animations `hud_base`,
//!   `hud_msg`, `pauseN`, `paus_err` and the elements by name into fixed slots, then moves the
//!   element keys in place: y −10 for weapon, ammo, iris1..7, gadget0..5, a007 + arm1..7 and
//!   healthBorder (0x23c004-0x23c0b4); with the renderer's mode word +0x58 ∈ {1, 2} (set to 2 by
//!   `FUN_001abee0` exactly when the widescreen flag +0x5c is on) x +78 for weapon, ammo, stylin
//!   and the gadgets, x −78 for iris, a007/arm and healthBorder (0x23c2c0-0x23c398). The third
//!   case (+0x5c on, +0x58 ∉ {1, 2}: x ±110, y +60) is not produced by `FUN_001abee0` and is not
//!   ported.
//! - `FUN_0023cf88` is the HUD frame. With HUD state +0x2b0 == 1 it calls, in order:
//!   `FUN_00240c98` weapon + ammo, `FUN_0023f328` crosshairs/targets (**not ported**),
//!   `FUN_0023da30` health, `FUN_002469b8` message line, `FUN_002470f0` (empty), `FUN_00245ab8`
//!   pickup line (**not ported**), `FUN_00244f88` gadget pop-up, `FUN_0023e580` timer. Before
//!   them `FUN_00246c48` (mission status banner) and `FUN_002472b0` (stunt award) run — **not
//!   ported**. `DriveHudDraw::frame` emits the ported passes in that order.
//! - Colours are the GS words the game writes to the draw state (R in the low byte, 0x80 = full).
//! - `FUN_0019f9c8` (the sprite primitive) puts (x, y) at the top-left corner, (x+w, y+h) at the
//!   bottom-right.

use nf_formats::gal::{Gal, Key};

/// One bitmap element of the layout, after `FUN_0023ae30`'s moves.
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    pub name: String,
    pub pos: [f32; 2],
    /// width/height × the element's own scale (`FUN_002386c8` sets gal+0x80/0x84 from key
    /// +0x54/+0x58; `FUN_00238d40` multiplies) [M].
    pub size: [f32; 2],
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
        Sprite { name: name.to_owned(), pos: [x, y], size: [w * sx, h * sy], uv: k.uv(), sheet: k.sheet(), mirror: k.mirror(), repeats: k.repeats() }
    }
    fn draw(&self, colour: u32) -> Vec<HudDraw> {
        // FUN_002391e8 → FUN_00238d40(gal, (int)x, (int)y, key, 0) [M].
        self.draw_at(self.pos[0] as i32, self.pos[1] as i32, colour)
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

/// A text element: only its position is read by the ported passes (the font binding
/// `FUN_002377d8(gal, element)` is not decoded — the draw side resolves the font by name).
#[derive(Clone, Debug, PartialEq)]
pub struct TextAnchor { pub name: String, pub pos: [f32; 2] }

/// The slots `FUN_0023ae30` fills, with their in-place moves applied.
#[derive(Clone, Debug, PartialEq)]
pub struct HudLayout {
    /// hud+0x14 `weapon`, hud+0x18 `ammo` (text, `hud_base`).
    pub weapon: TextAnchor,
    pub ammo: TextAnchor,
    /// hud+0x74 `stylin` (bitmap; its pass is not ported, kept for the layout).
    pub stylin: Sprite,
    /// hud+0x1c.. `iris1..iris7`.
    pub iris: Vec<Sprite>,
    /// hud+0x38 `a007`, hud+0x3c.. `arm1..arm7` (the armour row starts with the 007 logo).
    pub arms: Vec<Sprite>,
    /// hud+0x58 `healthBorder`.
    pub health_border: Sprite,
    /// hud+0x78.. `gadget0..gadget5`.
    pub gadgets: Vec<Sprite>,
    /// hud+0x25c `txt` (`hud_msg`) — the message line's font element.
    pub msg_txt: TextAnchor,
    /// hud+0x1f0 `scratchTxt` (`pauseN`) — the timer's font element.
    pub timer_font: TextAnchor,
    /// Screen centre (hud+0x454 / +0x458 = renderer width / height × 0.5; 640 × 448 NTSC,
    /// 640 × 512 for the other video mode [M `FUN_001abee0`, `FUN_0023c858`]).
    pub centre: [f32; 2],
}

impl HudLayout {
    /// Build the layout from the gallery like `FUN_0023ae30` [M]. `widescreen` = the renderer's
    /// +0x5c flag (which sets +0x58 = 2 in `FUN_001abee0`); `screen_h` = 448 or 512.
    pub fn from_gal(g: &Gal, widescreen: bool, screen_h: f32) -> Result<HudLayout, String> {
        let base = g.anim("hud_base").ok_or("no hud_base")?;
        let key = |a: &nf_formats::gal::Anim, n: &str| -> Result<Key, String> {
            a.element(n).and_then(|e| e.hud_key()).cloned().ok_or_else(|| format!("no element {n}"))
        };
        let sprite = |n: &str| -> Result<Sprite, String> { Ok(Sprite::of(n, &key(base, n)?)) };
        let text = |a: &nf_formats::gal::Anim, n: &str| -> Result<TextAnchor, String> {
            let [x, y, _] = key(a, n)?.pos();
            Ok(TextAnchor { name: n.to_owned(), pos: [x, y] })
        };
        let mut l = HudLayout {
            weapon: text(base, "weapon")?,
            ammo: text(base, "ammo")?,
            stylin: sprite("stylin")?,
            iris: (1..=7).map(|i| sprite(&format!("iris{i}"))).collect::<Result<_, _>>()?,
            arms: std::iter::once("a007".to_owned()).chain((1..=7).map(|i| format!("arm{i}")))
                .map(|n| sprite(&n)).collect::<Result<_, _>>()?,
            health_border: sprite("healthBorder")?,
            gadgets: (0..6).map(|i| sprite(&format!("gadget{i}"))).collect::<Result<_, _>>()?,
            msg_txt: text(g.anim("hud_msg").ok_or("no hud_msg")?, "txt")?,
            timer_font: text(g.anim("pauseN").ok_or("no pauseN")?, "scratchTxt")?,
            centre: [320.0, screen_h * 0.5],
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

/// A text source: a literal, or a label id the game resolves through `FUN_0024b1a0` (the
/// localized string table — not ported here).
#[derive(Clone, Debug, PartialEq)]
pub enum HudText { Literal(String), Label(i32) }

/// How a text's x is placed. `width(s)` = the font width the game measures with `FUN_00237ca8`
/// at the draw's scale (font metrics are the draw side's).
#[derive(Clone, Debug, PartialEq)]
pub enum TextX {
    /// x = at − width(text) (right-aligned).
    Right(f32),
    /// x = at − width(text) / 2.
    Centre(f32),
    /// x = at − width(measure) / 2 — the timer centres on the width of `"00:00"`, not its text.
    CentreOn(f32, String),
}

/// One HUD draw.
#[derive(Clone, Debug, PartialEq)]
pub enum HudDraw {
    /// A gallery bitmap quad: top-left (x, y), w × h; `uv` = (u, v) at the top-left corner then
    /// (u, v) at the bottom-right (a flipped copy swaps them); `tx` sheet; GS colour.
    Sprite { element: String, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], sheet: u32, colour: u32 },
    /// Text with a drop shadow: the shadow at (x+1, y+1) in `shadow`, then the text at (x, y)
    /// in `colour` [M `FUN_00244398`]. `font` = the element whose font binds.
    Text { font: String, text: HudText, x: TextX, y: f32, scale: f32, colour: u32, shadow: u32 },
    /// A filled box `FUN_00238408(x, y, w, h, colour)` sized from a measured string:
    /// x = centre − width(measure)/2 + dx, w = width(measure) + dw,
    /// h = height(measure) × h_mul + h_add (`FUN_00237ce0` height).
    Panel { font: String, measure: HudText, scale: f32, centre: f32, dx: f32, y: f32, dw: f32, h_mul: f32, h_add: f32, colour: u32 },
}

/// The per-frame game state the ported passes read.
#[derive(Clone, Debug, PartialEq)]
pub struct HudInput {
    /// hud+0x368: frame seconds = ticks since the last update × seconds per tick [M `FUN_0023cbc0`].
    pub dt: f32,
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
}

impl Default for DriveHudDraw { fn default() -> Self { Self::new() } }

/// `(int)(v ± 0.5)` — the game's rounding before every colour byte (`cvt.w.s` truncates) [M].
fn rnd(v: f32) -> i32 { (v + if v < 0.0 { -0.5 } else { 0.5 }) as i32 }

pub const TEXT_COLOUR: u32 = 0x7fb3_dbfa;
pub const TEXT_SHADOW: u32 = 0x4600_0000;
/// The health iris colours by ratio (> 0.5, > 0.25, else) and their dim twins [M 0x23da30].
pub const IRIS_COLOURS: [(u32, u32); 3] = [(0x468b_a852, 0x138b_a852), (0x464e_98c6, 0x134e_98c6), (0x4625_2cb0, 0x1325_2cb0)];
const SCALE_1_2: f32 = f32::from_bits(0x3f99_999a);
const SCALE_1_4: f32 = f32::from_bits(0x3fb3_3333);

impl DriveHudDraw {
    pub fn new() -> DriveHudDraw {
        DriveHudDraw {
            health_show: 3.0, health_fade: 0.0, health_blink: 0.0, last_health: 1.0, last_armour: 0.0, grunt_cycle: 0,
            msg_label: -1, msg_hold: 0.0, msg_fade: 0.0, msg_long: false,
            gadget_slot: 0, gadget_time: 0.0, gadget_slide: 0.0, gadget_blink: 0.0, gadget_blinks: 0,
            timer_beep_last: 10,
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

    /// One HUD frame of the ported passes, in `FUN_0023cf88`'s order.
    pub fn frame(&mut self, l: &HudLayout, i: &HudInput) -> HudFrame {
        let mut f = HudFrame::default();
        let gated = i.suppressed || self.msg_long;
        if !gated { Self::weapon_ammo(l, i, &mut f); }
        if !gated { self.health(l, i, &mut f); }
        self.message(l, i, &mut f);
        self.gadget(l, i, &mut f);
        self.timer(l, i, &mut f);
        f
    }

    /// `FUN_00240c98` [M, disassembly 0x240c98-0x240dfc].
    fn weapon_ammo(l: &HudLayout, i: &HudInput, f: &mut HudFrame) {
        let Some((id, ammo)) = i.weapon else { return };
        // Ammo is drawn unless the weapon id is 0x64b with ammo != 0; negative ammo prints 0.
        if !(id == 0x64b && ammo != 0) {
            f.draws.push(HudDraw::Text {
                font: l.ammo.name.clone(), text: HudText::Literal(format!("{}", ammo.max(0))),
                x: TextX::Right(l.ammo.pos[0]), y: l.ammo.pos[1], scale: 1.0, colour: TEXT_COLOUR, shadow: TEXT_SHADOW,
            });
        }
        f.draws.push(HudDraw::Text {
            font: l.weapon.name.clone(), text: HudText::Label(id),
            x: TextX::Right(l.weapon.pos[0]), y: l.weapon.pos[1], scale: 1.0, colour: TEXT_COLOUR, shadow: TEXT_SHADOW,
        });
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

    /// `FUN_002469b8` — the message line [M, disassembly 0x246a3c-0x246b98].
    fn message(&mut self, l: &HudLayout, i: &HudInput, out: &mut HudFrame) {
        let mut f = 1.0f32;
        if self.msg_hold <= 0.0 {
            if self.msg_fade > 0.0 { f = self.msg_fade * 2.0 } else { self.msg_long = false; f = 0.0 }
        }
        if f > 0.0 {
            let text = HudText::Label(self.msg_label);
            out.draws.push(HudDraw::Panel {
                font: l.msg_txt.name.clone(), measure: text.clone(), scale: SCALE_1_2, centre: l.centre[0],
                dx: -4.0, y: 376.0, dw: 12.0, h_mul: 1.0, h_add: 18.0, colour: 0x2000_0000,
            });
            out.draws.push(HudDraw::Text {
                font: l.msg_txt.name.clone(), text, x: TextX::Centre(l.centre[0]), y: 380.0, scale: SCALE_1_2,
                colour: (rnd(f * 127.0) as u32) << 24 | 0xb3_dbfa, shadow: (rnd(f * 32.0) as u32) << 24,
            });
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

    /// `FUN_0023e580` — the mission clock [M, disassembly 0x23e580-0x23e7b0].
    fn timer(&mut self, l: &HudLayout, i: &HudInput, out: &mut HudFrame) {
        let Some((secs, countdown)) = i.timer else { return };
        if secs <= 0 { return; }
        let font = l.timer_font.name.clone();
        let zero = "00:00".to_owned();
        out.draws.push(HudDraw::Panel {
            font: font.clone(), measure: HudText::Literal(zero.clone()), scale: SCALE_1_4, centre: l.centre[0],
            dx: 0.0, y: 33.0, dw: 0.0, h_mul: 2.0, h_add: -2.0, colour: 0x4605_050f,
        });
        let (m, s) = (secs / 60, secs % 60);
        let red = countdown && m == 0 && s < 10;
        out.draws.push(HudDraw::Text {
            font, text: HudText::Literal(format!("{m:2}:{s:02}")), x: TextX::CentreOn(l.centre[0], zero), y: 35.0,
            scale: SCALE_1_4, colour: if red { 0x7f05_05c8 } else { TEXT_COLOUR }, shadow: TEXT_SHADOW,
        });
        if red && s == self.timer_beep_last - 1 {
            out.sounds.push("SFX_timerbeep");
            self.timer_beep_last = s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp(name: &str, x: f32, y: f32) -> Sprite { Sprite { name: name.into(), pos: [x, y], size: [10.0, 10.0], uv: [0.0; 4], sheet: 0, mirror: 0, repeats: [0; 4] } }
    fn layout() -> HudLayout {
        HudLayout {
            weapon: TextAnchor { name: "weapon".into(), pos: [605.0, 410.0] },
            ammo: TextAnchor { name: "ammo".into(), pos: [605.0, 390.0] },
            stylin: sp("stylin", 510.0, 40.0),
            iris: (1..=7).map(|i| sp(&format!("iris{i}"), i as f32, 0.0)).collect(),
            arms: (0..8).map(|i| sp(&format!("arm{i}"), i as f32, 1.0)).collect(),
            health_border: sp("healthBorder", 68.0, 394.0),
            gadgets: (0..6).map(|i| sp(&format!("gadget{i}"), 540.0, 340.0)).collect(),
            msg_txt: TextAnchor { name: "txt".into(), pos: [260.0, 54.0] },
            timer_font: TextAnchor { name: "scratchTxt".into(), pos: [0.0, 0.0] },
            centre: [320.0, 224.0],
        }
    }
    fn input(h: f32, a: f32) -> HudInput {
        HudInput { dt: 1.0 / 60.0, health: h, health_max: 1.0, armour: a, armour_max: 1.0, weapon: None, timer: None,
            health_always: false, grunt_on_hit: false, suppressed: false }
    }
    fn sprites(f: &HudFrame) -> Vec<(String, u32)> {
        f.draws.iter().filter_map(|d| match d { HudDraw::Sprite { element, colour, .. } => Some((element.clone(), *colour)), _ => None }).collect()
    }

    /// Health 0.6 / armour 0.3 [M 0x23da30]: 7 dim iris (0x13 alpha), 4 full (0x46, > 0.5
    /// colour), one partial at alpha round(0x46 × (4.2 − 4)) = 14, 2 full arms (a007, arm1) at
    /// 0x7f, the partial arm at round(round(0.4 × 127) × 1) = 51, then the border in full colour.
    #[test]
    fn health_meter_segments_follow_fun_0023da30() {
        let l = layout();
        let mut d = DriveHudDraw::new();
        let f = d.frame(&l, &input(0.6, 0.3));
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
        let f = DriveHudDraw::new().frame(&l, &input(0.5, 1.0));
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
        for _ in 0..12 { assert!(!sprites(&d.frame(&l, &i)).is_empty()); }
        // The 12th frame ran show to 0 and armed fade 0.5 — and the fade block right after it
        // already took this frame's dt (0.25 left) [M: the two blocks run in that order].
        assert_eq!((d.health_show, d.health_fade), (0.0, 0.25));
        let f = d.frame(&l, &i);
        assert_eq!(sprites(&f)[0].1 >> 24, 10, "f = 2 × 0.25: dim alpha round(0x13 × 0.5) = 10");
        assert!(sprites(&d.frame(&l, &i)).is_empty(), "hidden once the fade ran out");
        i.health = 0.9;
        d.frame(&l, &i);
        assert_eq!(d.health_show, 3.0, "a change re-arms the show timer");
        // Sister: health_always keeps it lit with show = 0 and fade = 0.
        let mut d2 = DriveHudDraw { health_show: 0.0, ..DriveHudDraw::new() };
        let mut i2 = input(1.0, 0.0);
        i2.health_always = true;
        assert!(!sprites(&d2.frame(&l, &i2)).is_empty());
    }

    /// The clock [M 0x23e580]: "%2d:%02d", red + beep in the last 10 s of a countdown, hidden at 0.
    #[test]
    fn timer_face_follows_fun_0023e580() {
        let l = layout();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 0.0);
        i.suppressed = true;
        i.timer = Some((75, true));
        let f = d.frame(&l, &i);
        let t: Vec<_> = f.draws.iter().filter_map(|x| match x { HudDraw::Text { text: HudText::Literal(s), colour, y, .. } => Some((s.clone(), *colour, *y)), _ => None }).collect();
        assert_eq!(t, vec![(" 1:15".to_owned(), TEXT_COLOUR, 35.0)]);
        i.timer = Some((9, true));
        let f = d.frame(&l, &i);
        assert!(f.draws.iter().any(|x| matches!(x, HudDraw::Text { colour: 0x7f05_05c8, .. })));
        assert_eq!(f.sounds, vec!["SFX_timerbeep"], "9 = 10 − 1 beeps");
        assert!(d.frame(&l, &i).sounds.is_empty(), "the same second does not beep twice");
        i.timer = Some((9, false));
        assert!(!d.frame(&l, &i).draws.iter().any(|x| matches!(x, HudDraw::Text { colour: 0x7f05_05c8, .. })), "a count-up never turns red");
        i.timer = Some((0, true));
        assert!(d.frame(&l, &i).draws.is_empty(), "0 s draws nothing");
    }

    /// Weapon + ammo [M 0x240c98]: right-aligned at the element x, ammo floored at 0, the
    /// 0x64b id hides a non-zero ammo count.
    #[test]
    fn weapon_and_ammo_lines() {
        let l = layout();
        let mut i = input(1.0, 1.0);
        i.weapon = Some((7, -3));
        let f = DriveHudDraw::new().frame(&l, &i);
        assert!(f.draws.contains(&HudDraw::Text { font: "ammo".into(), text: HudText::Literal("0".into()), x: TextX::Right(605.0), y: 390.0, scale: 1.0, colour: TEXT_COLOUR, shadow: TEXT_SHADOW }));
        assert!(f.draws.contains(&HudDraw::Text { font: "weapon".into(), text: HudText::Label(7), x: TextX::Right(605.0), y: 410.0, scale: 1.0, colour: TEXT_COLOUR, shadow: TEXT_SHADOW }));
        i.weapon = Some((0x64b, 5));
        let f = DriveHudDraw::new().frame(&l, &i);
        assert!(!f.draws.iter().any(|d| matches!(d, HudDraw::Text { font, .. } if font == "ammo")));
        i.weapon = Some((0x64b, 0));
        let f = DriveHudDraw::new().frame(&l, &i);
        assert!(f.draws.iter().any(|d| matches!(d, HudDraw::Text { font, .. } if font == "ammo")), "0x64b with 0 ammo still prints 0");
    }

    /// A long message hides weapon/ammo and health until its fade ends [M 0x245890, 0x2469b8].
    #[test]
    fn a_long_message_hides_the_meters_until_it_fades() {
        let l = layout();
        let mut d = DriveHudDraw::new();
        let mut i = input(1.0, 1.0);
        i.weapon = Some((1, 10));
        i.dt = 0.25;
        d.message_show(0.5, 42, 33, false);
        let f = d.frame(&l, &i);
        assert!(sprites(&f).is_empty() && f.draws.iter().all(|x| !matches!(x, HudDraw::Text { font, .. } if font == "ammo")));
        assert!(f.draws.contains(&HudDraw::Text { font: "txt".into(), text: HudText::Label(42), x: TextX::Centre(320.0), y: 380.0, scale: SCALE_1_2, colour: 0x7fb3_dbfa, shadow: 0x2000_0000 }));
        d.frame(&l, &i); // hold → 0, fade 0.5
        d.frame(&l, &i); // f = 1.0, fade 0.25
        let f = d.frame(&l, &i); // f = 0.5, fade → 0, long cleared at the end
        assert!(f.draws.iter().any(|x| matches!(x, HudDraw::Text { colour, .. } if *colour == (64u32 << 24 | 0xb3_dbfa))), "f = 0.5 → alpha round(63.5) = 64");
        assert!(!d.msg_long);
        assert!(!sprites(&d.frame(&l, &i)).is_empty(), "meters back");
        // Sister: a 32-byte message is not long.
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
        assert_eq!(pos(&d.frame(&l, &i)), (315, 214, 127), "centre − (w/2, h) at full alpha");
        for _ in 0..3 { d.frame(&l, &i); }
        assert_eq!(d.gadget_blinks, 0, "two 0.5 s blinks");
        assert_eq!(pos(&d.frame(&l, &i)).0, 315, "still centred: the slide starts after the blinks");
        d.frame(&l, &i);
        assert_eq!(pos(&d.frame(&l, &i)), (540, 340, 127), "home once the slide ran out");
    }

    /// `FUN_00238d40`'s mirror and repeat expansion [M]: mirror 3 = four quads around the key,
    /// mirror 1 = only the x-flipped quad, repeats step by the size.
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
        assert_eq!(xy(s.draw(0)).iter().map(|&(x, y, _)| (x, y)).collect::<Vec<_>>(), vec![(90.0, 50.0)], "bit 0 alone draws only the flipped copy");
        s.mirror = 0;
        s.repeats = [0, 3, 0, 0];
        assert_eq!(xy(s.draw(0)).iter().map(|&(x, _, _)| x).collect::<Vec<_>>(), vec![100.0, 110.0, 120.0]);
        s.repeats = [0, 0, -1, 0];
        assert!(s.draw(0).is_empty(), "a negative count sum draws nothing");
    }
}
