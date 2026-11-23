//! What the driving engine draws for a projectile in flight, its explosion and the gun's bullet
//! streaks: the model (a copy of a shooter car item), the body matrix, the smoke ribbon, the
//! point lights, the explosion's Giotto effect and light, and the streak quads.
//! docs/systems/driving-projectile-gfx.md is the write-up; the data is `nf_formats::driveproj`.
//!
//! Sources (DRIVING.ELF, [M] decompile/disassembly):
//! - `FUN_00181960`, the `Missile` draw: the EMP light (kind 0xe), the glow request
//!   (`FUN_001afba8`), the ribbon feed (`FUN_001e4640` / `FUN_001e43e0`) and draw
//!   (`FUN_001e4258`).
//! - `FUN_001afd50` (run each frame by `FUN_001aede8`): the explosion and glow lights through
//!   `FUN_001af520`, which keeps at most 4 point lights a frame.
//! - `FUN_001afa88`: the explosion light class and slot; `FUN_0015a098`: the explosion event
//!   (Giotto effect, radius damage, scorch decal, light, camera shake).
//! - `FUN_0019a9b8` / `FUN_00199358` / `FUN_001993e8` / `FUN_001d7d78`: the bullet-streak list,
//!   its 60-tick purge, the per-frame advance and the quad build.
//! - `FUN_00210540`: the launch orientation (rows x = up × f, y = f × x, z = f).
//!
//! Logic only; nothing here draws.

use nf_formats::drivefx::BulletStreak;
use nf_formats::driveproj::{
    ActionItem, BlastTune, GlowTune, ProjectileLighting, STREAK_BUCKET_LIMIT, STREAK_LIST_LIMIT, STREAK_UV, TRAIL_DRAW_VERTS, TRAIL_JITTER,
    TRAIL_MAX_VERTS, TRAIL_WIDTH,
};

use super::effects::DriveRng;
use super::weapons::{Projectile, FOD_ROCKET, MISSILE, Q_PULSE, REMOTE_TORPEDO, ROCKET, SNOWMOBILE_ROCKET, TORPEDO, WEAPONS};

type V3 = [f32; 3];
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn scale(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn norm(a: V3) -> V3 { let l = dot(a, a).sqrt(); if l > 0.0 { scale(a, 1.0 / l) } else { a } }

// ---------------------------------------------------------------- model and matrix

/// The body matrix the launch builds (`FUN_00210540`): rows x = norm(up × f), y = f × x, z = f,
/// then the position. The projectile's model is drawn with an identity local matrix
/// (`FUN_001c7b60` copies 0x355bb0 into the item) [M]. The game then turns a quaternion; the
/// port carries the forward axis only (weapons.rs), so the roll is not modelled.
pub fn body_matrix(pos: V3, fwd: V3) -> [[f32; 4]; 4] {
    let f = norm(fwd);
    let mut x = cross([0.0, 1.0, 0.0], f);
    if dot(x, x) < 1e-12 {
        x = [1.0, 0.0, 0.0]; // straight up/down: the game's rsqrt of 0 is undefined; not traced
    }
    let x = norm(x);
    let y = cross(f, x);
    [[x[0], x[1], x[2], 0.0], [y[0], y[1], y[2], 0.0], [f[0], f[1], f[2], 0.0], [pos[0], pos[1], pos[2], 1.0]]
}

/// One barrel's projectile model on a car.
#[derive(Clone, Debug, PartialEq)]
pub struct BarrelModel {
    /// The vehicle action (barrel id) the weapon plays.
    pub action: u8,
    /// The car item whose action byte is the barrel and whose effect list carries the launch entry.
    pub item: usize,
    /// The item copied as the projectile model (the entry's `+0x15`).
    pub model_item: usize,
    /// The launch entry's Giotto name, if it names one (`GIOT::Wp_Rock`, …).
    pub launch_fx: Option<String>,
}

/// The projectile models of `weapon` on a car: for each barrel action, the items carrying that
/// action whose effect list has a type-6 / type-2 entry, the model being that entry's `+0x15`
/// item [M `FUN_0014ca60` + `FUN_001c9328` + `FUN_0020e678`]. Which entry the fire event carries is
/// set by the event dispatcher (`DAT_0032e710..`, not traced): this takes each such entry.
pub fn barrel_models(items: &[ActionItem], weapon: usize) -> Vec<BarrelModel> {
    let mut out = Vec::new();
    for &a in WEAPONS[weapon].barrels {
        for it in items.iter().filter(|i| i.action == a) {
            for fx in &it.effects {
                let k = fx.entry.kind;
                if k == nf_formats::trackfx::kind::GIOTTO || k == nf_formats::trackfx::kind::GLARE {
                    let m = BarrelModel { action: a, item: it.index, model_item: it.model_item(&fx.entry), launch_fx: fx.name.clone() };
                    if !out.contains(&m) {
                        out.push(m);
                    }
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- the smoke ribbon

/// The ribbon type (`Missile` +0xa8): 1 for the Snowmobile Rocket, 2 for both torpedoes, else 0
/// [M `FUN_00180bc8`].
pub fn trail_type(kind: usize) -> usize {
    match kind {
        SNOWMOBILE_ROCKET => 1,
        TORPEDO | REMOTE_TORPEDO => 2,
        _ => 0,
    }
}

/// The ribbon is drawn for every kind but the Missile and the FOD Rocket, and the Q-Pulse never
/// reaches the ribbon code [M `FUN_00181960`: the kind-0xe branch returns first; `!= 2 && != 0x1e`].
pub fn trail_drawn(kind: usize) -> bool { kind != MISSILE && kind != FOD_ROCKET && kind != Q_PULSE }

/// One ribbon vertex as the draw hands it on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailVertex {
    pub pos: V3,
    pub rgba: [u8; 4],
    pub uv: [f32; 2],
}

/// The ribbon record at `Missile + 0xa0` (0x650 bytes) [M `FUN_001e4640`, `FUN_001e43e0`,
/// `FUN_001e4258`].
#[derive(Clone, Debug, PartialEq)]
pub struct Trail {
    /// +0x08: the type (half-width and jitter index).
    pub ty: usize,
    /// +0x00: vertices stored (two per sample, at most 0x60).
    pub n: usize,
    /// +0x04: calls so far (feed or spent).
    pub ticks: u32,
    /// +0x10: the vertices (stride 0x10).
    pub verts: Vec<V3>,
}

impl Trail {
    pub fn new(kind: usize) -> Trail { Trail { ty: trail_type(kind), n: 0, ticks: 0, verts: vec![[0.0; 3]; TRAIL_MAX_VERTS] } }

    /// `FUN_001e4640(trail, pos)`: on even calls a new sample takes two slots; the newest pair
    /// is then (re)written: centre = pos + 3 jitter draws (`-j/2 + bits·j/65536` per axis, x y z),
    /// vertex 0 = centre + right·w, vertex 1 = centre - right·w. `right` is the first row of the
    /// view matrix (`**(DAT_00333760 + 4)`; which camera axis that row is was not traced).
    pub fn feed(&mut self, pos: V3, right: V3, rng: &mut DriveRng) {
        if self.n < TRAIL_MAX_VERTS {
            if self.ticks & 1 == 0 {
                self.n += 2;
            }
            let j = TRAIL_JITTER[self.ty];
            let w = TRAIL_WIDTH[self.ty];
            let mut c = pos;
            for v in c.iter_mut() {
                *v = *v + -(j * 0.5) + rng.bits() as f32 * j * 1.525_878_9e-5;
            }
            self.verts[self.n - 2] = add(c, scale(right, w));
            self.verts[self.n - 1] = sub(c, scale(right, w));
        }
        self.ticks += 1;
    }

    /// `FUN_001e43e0`: the spent projectile only counts, so the ribbon's tail runs out.
    pub fn tick_spent(&mut self) { self.ticks += 1; }

    /// `FUN_001e4258`'s window: end = min(n, round_up_even(ticks)), start = that rounded value
    /// - 0x28 when >= 1, else 0; drawn when n > 3 and end - start > 3.
    pub fn window(&self) -> Option<(usize, usize)> {
        if self.n <= 3 {
            return None;
        }
        let a0 = ((self.ticks as i32 + 1) / 2) * 2;
        let end = if (self.n as i32) <= a0 { self.n as i32 } else { a0 };
        let st = a0 - TRAIL_DRAW_VERTS as i32;
        let start = if st < 1 { 0 } else { st };
        (end - start > 3).then_some((start as usize, end as usize))
    }

    /// The drawn vertices (a strip): colour `trail_vertex_colour(k)` from the window start, uv
    /// (s, t) = (1 for even / 0 for odd vertices, bit 1 of the vertex index) [M `FUN_001e3e28`'s
    /// tables: 0x3de1b0, or 0x3de1d0 = two entries on when `start & 2`].
    pub fn draw(&self) -> Vec<TrailVertex> {
        let Some((s, e)) = self.window() else { return Vec::new() };
        (s..e)
            .enumerate()
            .map(|(k, j)| TrailVertex {
                pos: self.verts[j],
                rgba: nf_formats::driveproj::trail_vertex_colour(k),
                uv: [if j & 1 == 0 { 1.0 } else { 0.0 }, ((j >> 1) & 1) as f32],
            })
            .collect()
    }
}

// ---------------------------------------------------------------- lights

/// A point light as `FUN_001af520` stores it (at most [`MAX_LIGHTS`] a frame).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointLight {
    pub pos: V3,
    pub rgb: [f32; 3],
    pub size: f32,
}

/// `FUN_001af520` drops lights past the fourth of a frame [M `iVar2 < 4`].
pub const MAX_LIGHTS: usize = 4;

impl PointLight {
    /// The global light mode 1 (`DAT_003349e0 + 0x30 == 1`, writer not traced): red 0, blue
    /// halved [M `FUN_001af520`].
    pub fn mode1(mut self) -> PointLight {
        self.rgb[0] = 0.0;
        self.rgb[2] *= 0.5;
        self
    }
}

/// The glow type the draw requests: Rocket 1, Snowmobile Rocket 2, every other kind 0
/// (`Missile`); none for the torpedo, the Q-Pulse (it has its own light) or a spent body
/// [M `FUN_00181960`]. Remote Torpedo (0x16) is not excluded by the code.
pub fn glow_type(kind: usize, spent: bool) -> Option<usize> {
    if spent || kind == TORPEDO || kind == Q_PULSE {
        return None;
    }
    Some(match kind {
        ROCKET => 1,
        SNOWMOBILE_ROCKET => 2,
        _ => 0,
    })
}

/// The glow measure the draw passes: for the first fps/4 ticks `(2·(fps/4 - age))²`, after
/// that `age`, times 1/fps [M `FUN_00181960`].
pub fn glow_measure(age: u32, fps: i32) -> f32 {
    let q = (fps as u32) >> 2;
    let v = if age < q {
        let u = (q - age) as f32;
        (u + u) * (u + u)
    } else {
        age as f32
    };
    v * (1.0 / fps as f32)
}

/// The two glow slots (`RLightManager + 0x2e0 + 0xe4..`, reset each frame) [M `FUN_001afba8`]:
/// while fewer than two are held the request takes the next slot; otherwise it replaces the slot
/// with the largest measure above 0 (none when both are <= 0).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GlowSlots {
    pub n: usize,
    pub slot: [(V3, usize, f32); 2],
}

impl GlowSlots {
    pub fn request(&mut self, pos: V3, ty: usize, measure: f32) {
        let i = if self.n < 2 {
            self.n += 1;
            self.n - 1
        } else {
            let (mut best, mut pick) = (0.0f32, 2usize);
            for i in 0..2 {
                if self.slot[i].2 > best {
                    best = self.slot[i].2;
                    pick = i;
                }
            }
            pick
        };
        if i < 2 {
            self.slot[i] = (pos, ty, measure);
        }
    }

    /// The lights `FUN_001afd50` makes of them: colour = the type's Colour, size = Size ·
    /// max(measure, 1).
    pub fn lights(&self, tune: &[GlowTune; 5]) -> Vec<PointLight> {
        self.slot[..self.n]
            .iter()
            .map(|&(pos, ty, m)| {
                let t = tune[ty];
                PointLight { pos, rgb: [t.colour[0], t.colour[1], t.colour[2]], size: t.size * m.max(1.0) }
            })
            .collect()
    }
}

/// The Q-Pulse's own light, added straight to the list once `age > homing delay`: colour
/// (0.3, 0.3, 1.0), size 4; spent: size 1 and the colour × linger / fps [M `FUN_00181960`,
/// DAT_00378660 = (0.3, 0.3, 1.0, 0) ELF data].
pub fn emp_light(pos: V3, age: i32, homing_delay: i32, spent: bool, linger: i32, fps: i32) -> Option<PointLight> {
    if age <= homing_delay {
        return None;
    }
    let mut l = PointLight { pos, rgb: [0.3, 0.3, 1.0], size: 4.0 };
    if spent {
        let k = linger as f32 * (1.0 / fps as f32);
        l.size = 1.0;
        l.rgb = scale(l.rgb, k);
    }
    Some(l)
}

/// The explosion light class by damage: > 800 Huge, > 400 Large, > 200 Medium, > 100 Small,
/// else Tiny [M `FUN_001afa88`].
pub fn blast_class(damage: f32) -> usize {
    if damage > 800.0 {
        4
    } else if damage > 400.0 {
        3
    } else if damage > 200.0 {
        2
    } else if damage > 100.0 {
        1
    } else {
        0
    }
}

/// The four explosion light slots [M `FUN_001afa88`, `FUN_001afd50`]: start tick (0 = free),
/// class, position.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BlastLights {
    pub start: [u32; 4],
    pub class: [usize; 4],
    pub pos: [V3; 4],
}

impl BlastLights {
    /// A new explosion takes the slot with the smallest start tick (ties keep the earlier slot,
    /// the scan stops at a free one) and starts at `now` (DAT_00334010).
    pub fn add(&mut self, damage: f32, pos: V3, now: u32) {
        let (mut best, mut idx) = (u32::MAX, 0usize);
        for i in 0..4 {
            if self.start[i] < best {
                best = self.start[i];
                idx = i;
            }
            if best == 0 {
                break;
            }
        }
        if best != u32::MAX {
            self.pos[idx] = pos;
            self.class[idx] = blast_class(damage);
            self.start[idx] = now;
        }
    }

    /// This frame's lights: dt = now - start; f = 1/(dt·0.5) - 0.02 for dt > 0, else 0; a slot
    /// with f <= 0 is freed; colour = Colour · min(Decay·f, 16), size = Size. So a light lasts
    /// 99 ticks.
    pub fn frame(&mut self, now: u32, tune: &[BlastTune; 5]) -> Vec<PointLight> {
        let mut out = Vec::new();
        for i in 0..4 {
            if self.start[i] == 0 {
                continue;
            }
            let dt = now.wrapping_sub(self.start[i]) as i32;
            let f = if dt > 0 { 1.0 / (dt as f32 * 0.5) - 0.02 } else { 0.0 };
            if f <= 0.0 {
                self.start[i] = 0;
                continue;
            }
            let t = tune[self.class[i]];
            let k = (t.decay * f).min(16.0);
            out.push(PointLight { pos: self.pos[i], rgb: [t.colour[0] * k, t.colour[1] * k, t.colour[2] * k], size: t.size });
        }
        out
    }
}

/// One frame's light list in `FUN_001afd50`'s order (explosions, then glows), cut at 4. The
/// draw-time Q-Pulse lights go in when the projectile draws; their order against this pass is
/// not traced, so the caller places them.
pub fn frame_lights(blasts: &mut BlastLights, glows: &GlowSlots, now: u32, tune: &ProjectileLighting) -> Vec<PointLight> {
    let mut v = blasts.frame(now, &tune.blast);
    v.extend(glows.lights(&tune.glow));
    v.truncate(MAX_LIGHTS);
    v
}

// ---------------------------------------------------------------- explosions

/// What `FUN_0015a098` reads to pick the explosion's Giotto effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlastPick {
    /// [0xe]: the projectile kind.
    pub kind: usize,
    /// [0x10]: the world hit's surface byte (0 for an object hit).
    pub surface: u8,
    /// The player vehicle's object (vtable +0x1d4) has `+0xbc` set — what that flag means was
    /// not traced (the names suggest underwater).
    pub player_flag_bc: bool,
    /// [0xc]: the object hit is the player's vehicle.
    pub hit_player: bool,
    pub damage: f32,
    /// DAT_00339f54.
    pub now: u32,
}

/// The effect name [M `FUN_0015a098`, the no-name path]; `cycle` is DAT_0032bbdc (starts 0).
pub fn explosion_effect(p: &BlastPick, cycle: &mut usize) -> &'static str {
    if p.kind == FOD_ROCKET {
        return "FODHitEffect";
    }
    match p.surface {
        9 => return "snowspecificexplosion",
        6 => return "rocketsplash",
        _ => {}
    }
    if p.player_flag_bc {
        return if p.damage > 600.0 { "Un_ExplL" } else if p.damage > 200.0 { "Un_ExplM" } else { "Un_ExplS" };
    }
    if matches!(p.surface, 2..=5 | 10 | 0x50 | 0x60) {
        return ["Ex_SmallD", "Ex_SmallD2", "Ex_SmallD3"][(p.now % 3) as usize];
    }
    let name = if p.hit_player { ["Ex_SmB00", "Ex_SmB01", "Ex_SmB02"][*cycle] } else { ["Ex_Small", "Ex_Small2", "Ex_Small3"][*cycle] };
    *cycle = (*cycle + 1) % 3;
    name
}

/// The scorch the explosion leaves: `FUN_001eb078(8.0, …, pos, 5)`, decal.dat record 5 [M
/// `FUN_0015a098`; the meaning of the 8.0 argument is not traced].
pub const SCORCH_DECAL: (usize, f32) = (5, 8.0);

/// The explosion shakes the camera when the player is within 20 units, by min(10/dist, 1) (only
/// when the player object's +0x50 is set) [M `FUN_0015a098` → `FUN_00165fb0`].
pub fn blast_shake(dist: f32, player_50: bool) -> Option<f32> {
    let d = if dist < 0.0001 { 0.0001 } else { dist };
    (d <= 20.0 && player_50).then(|| (10.0 / d).min(1.0))
}

// ---------------------------------------------------------------- per-projectile state

/// The draw-side state one projectile carries and what it yields each tick.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileFx {
    pub trail: Trail,
}

/// One tick's draw description of a projectile.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileDraw {
    /// Draw the copied car item with this matrix (identity local).
    pub matrix: [[f32; 4]; 4],
    /// The glow request for [`GlowSlots`]: (type, measure).
    pub glow: Option<(usize, f32)>,
    /// The Q-Pulse light, straight to the list.
    pub emp: Option<PointLight>,
    /// The ribbon strip (empty when not drawn).
    pub trail: Vec<TrailVertex>,
}

impl ProjectileFx {
    pub fn new(kind: usize) -> ProjectileFx { ProjectileFx { trail: Trail::new(kind) } }

    /// `FUN_00181960` for one tick (the `param_2 != 0` path). `spent` = the hit flag (+0x710),
    /// `linger` = +0x8c, `mode3` = DAT_00339f44 == 3 (the ribbon is neither fed nor counted).
    pub fn tick(&mut self, p: &Projectile, now: i32, fps: i32, spent: bool, linger: i32, right: V3, mode3: bool, rng: &mut DriveRng) -> ProjectileDraw {
        let age = now - p.born;
        let matrix = body_matrix(p.pos, p.fwd);
        if p.kind == Q_PULSE {
            return ProjectileDraw { matrix, glow: None, emp: emp_light(p.pos, age, p.p.homing_delay, spent, linger, fps), trail: Vec::new() };
        }
        let glow = glow_type(p.kind, spent).map(|t| (t, glow_measure(age as u32, fps)));
        if !mode3 {
            if spent {
                self.trail.tick_spent();
            } else {
                self.trail.feed(p.pos, right, rng);
            }
        }
        let trail = if trail_drawn(p.kind) { self.trail.draw() } else { Vec::new() };
        ProjectileDraw { matrix, glow, emp: None, trail }
    }
}

// ---------------------------------------------------------------- bullet streaks

/// One live streak (`0x3320c0 + 0x20·i`) [M `FUN_0019a9b8`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Streak {
    pub start: V3,
    pub born: u32,
    pub end: V3,
    /// BULLET_STREAK_TYPE.
    pub ty: usize,
    /// Fired from a barrel the start follows, with the POV flag (`DAT_00327670 + 0x164`) set:
    /// the distance runs with t² instead of t.
    pub follows_pov: bool,
}

/// One streak quad, two triangles: (h+s, h-s, t-s) and (t-s, t+s, h+s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreakQuad {
    /// `BulletStreakTexture` (0..3, `driveproj::STREAK_TEXTURES`).
    pub texture: u32,
    pub verts: [V3; 6],
    pub uv: [[f32; 2]; 6],
    /// Each byte of the tun colour halved [M `>> 1 & 0x7f7f7f7f`], in the tun's order.
    pub rgba: [u8; 4],
}

/// The streak list (at most 0x40).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Streaks {
    pub list: Vec<Streak>,
}

impl Streaks {
    /// `FUN_0019a9b8`: type < 0 (the shot that does not streak) adds nothing.
    pub fn spawn(&mut self, s: Streak) {
        if self.list.len() < STREAK_LIST_LIMIT {
            self.list.push(s);
        }
    }

    /// `FUN_001993e8` + `FUN_001d7d78` for one frame: purge those born before now - 60, then per
    /// streak t = age / fps (squared when following a barrel), head = start + dir·(t·Speed +
    /// Length); a head past the end removes it, else one quad. `tune[ty]` is the FX tun entry of
    /// that type; `bucket` counts vertices per texture (a quad only while < 0x46; when the game
    /// resets the counters was not traced: per frame here).
    pub fn frame(&mut self, now: u32, fps: i32, tune: &[BulletStreak], cam: V3) -> Vec<StreakQuad> {
        let cut = now.wrapping_sub(60);
        self.list.retain(|s| s.born >= cut);
        let mut bucket = [0usize; 4];
        let mut out = Vec::new();
        let dt = 1.0 / fps as f32;
        self.list.retain(|s| {
            let mut t = now.wrapping_sub(s.born) as f32 * dt;
            if s.follows_pov {
                t *= t;
            }
            let Some(b) = tune.get(s.ty) else { return true };
            let to = sub(s.end, s.start);
            let dir = norm(to);
            let head = add(s.start, scale(dir, t * b.speed + b.length));
            let back = sub(s.start, head);
            if dot(to, to) < dot(back, back) {
                return false;
            }
            let tex = (b.texture as usize).min(3);
            if bucket[tex] < STREAK_BUCKET_LIMIT {
                let v = norm(sub(cam, head));
                let side = scale(norm(cross(v, dir)), b.width);
                let tail = sub(head, scale(dir, b.length));
                out.push(StreakQuad {
                    texture: b.texture,
                    verts: [add(head, side), sub(head, side), sub(tail, side), sub(tail, side), add(tail, side), add(head, side)],
                    uv: STREAK_UV,
                    rgba: b.colour.map(|c| c >> 1),
                });
                bucket[tex] += 6;
            }
            true
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driving::weapons::{launch_params, Launcher};

    #[test]
    fn body_matrix_rows_follow_the_launch_construction() {
        let m = body_matrix([1.0, 2.0, 3.0], [0.0, 0.0, 2.0]);
        assert_eq!(m[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(m[1], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(m[2], [0.0, 0.0, 1.0, 0.0]);
        assert_eq!(m[3], [1.0, 2.0, 3.0, 1.0]);
        // flying along +x: x row = up × f = (0,0,-1)
        let m = body_matrix([0.0; 3], [1.0, 0.0, 0.0]);
        assert_eq!([m[0][0], m[0][1], m[0][2]], [0.0, 0.0, -1.0]);
        assert_eq!([m[1][0], m[1][1], m[1][2]], [0.0, 1.0, 0.0]);
    }

    #[test]
    fn trail_types_and_who_draws_one() {
        assert_eq!((trail_type(ROCKET), trail_type(SNOWMOBILE_ROCKET), trail_type(TORPEDO), trail_type(REMOTE_TORPEDO), trail_type(MISSILE)), (0, 1, 2, 2, 0));
        assert!(trail_drawn(ROCKET) && trail_drawn(TORPEDO) && trail_drawn(SNOWMOBILE_ROCKET));
        assert!(!trail_drawn(MISSILE) && !trail_drawn(FOD_ROCKET) && !trail_drawn(Q_PULSE));
    }

    #[test]
    fn trail_adds_a_pair_every_other_feed_and_rewrites_the_newest() {
        let mut t = Trail::new(SNOWMOBILE_ROCKET); // jitter 0, width 0.1
        let mut rng = DriveRng::seeded(0);
        t.feed([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], &mut rng);
        assert_eq!((t.n, t.ticks), (2, 1));
        assert_eq!(t.verts[0], [0.1, 0.0, 0.0]);
        assert_eq!(t.verts[1], [-0.1, 0.0, 0.0]);
        t.feed([0.0, 0.0, 5.0], [1.0, 0.0, 0.0], &mut rng);
        assert_eq!((t.n, t.ticks), (2, 2)); // odd call: same pair, moved
        assert_eq!(t.verts[0], [0.1, 0.0, 5.0]);
        t.feed([0.0, 0.0, 6.0], [1.0, 0.0, 0.0], &mut rng);
        assert_eq!(t.n, 4);
    }

    #[test]
    fn trail_jitter_uses_three_draws_of_the_shared_generator() {
        let mut t = Trail::new(ROCKET); // jitter 0.2, width 0.3575
        let mut rng = DriveRng::seeded(0);
        let mut r2 = rng;
        t.feed([0.0; 3], [0.0, 1.0, 0.0], &mut rng);
        let j: Vec<f32> = (0..3).map(|_| -0.1 + r2.bits() as f32 * 0.2 * 1.525_878_9e-5).collect();
        assert_eq!(t.verts[0], [j[0], j[1] + 0.3575, j[2]]);
        assert_eq!(t.verts[1], [j[0], j[1] - 0.3575, j[2]]);
        assert_eq!(rng, r2);
    }

    #[test]
    fn trail_window_caps_at_40_and_runs_out_after_the_hit() {
        let mut t = Trail::new(ROCKET);
        let mut rng = DriveRng::seeded(0);
        for i in 0..3 {
            t.feed([0.0, 0.0, i as f32], [1.0, 0.0, 0.0], &mut rng);
        }
        assert_eq!(t.window(), Some((0, 4)));
        for i in 3..200 {
            t.feed([0.0, 0.0, i as f32], [1.0, 0.0, 0.0], &mut rng);
        }
        assert_eq!(t.n, 0x60);
        // 200 calls: a0 = 200, end = min(96, 200) = 96, start = 160 → nothing (end < start)
        assert_eq!(t.window(), None);
        let mut t = Trail::new(ROCKET);
        for i in 0..60 {
            t.feed([0.0, 0.0, i as f32], [1.0, 0.0, 0.0], &mut rng);
        }
        assert_eq!((t.n, t.window()), (60, Some((20, 60))));
        for _ in 0..34 {
            t.tick_spent();
        }
        // 94 calls: start 54, end 60 → 6 vertices; two more → 4 left; then none
        assert_eq!(t.window(), Some((54, 60)));
        t.tick_spent();
        t.tick_spent();
        assert_eq!(t.window(), Some((56, 60)));
        t.tick_spent();
        t.tick_spent();
        assert_eq!(t.window(), None);
        let d = Trail { ty: 0, n: 60, ticks: 60, verts: t.verts.clone() }.draw();
        assert_eq!(d.len(), 40);
        assert_eq!((d[0].rgba[3], d[39].rgba[3]), (0, 30));
        assert_eq!((d[0].uv, d[1].uv, d[2].uv, d[3].uv), ([1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]));
    }

    #[test]
    fn glow_type_measure_and_size() {
        assert_eq!((glow_type(ROCKET, false), glow_type(SNOWMOBILE_ROCKET, false), glow_type(MISSILE, false)), (Some(1), Some(2), Some(0)));
        assert_eq!((glow_type(TORPEDO, false), glow_type(Q_PULSE, false), glow_type(ROCKET, true)), (None, None, None));
        assert_eq!(glow_type(REMOTE_TORPEDO, false), Some(0));
        // fps 60: age 0 → 30² / 60 = 15; age 14 → 2² / 60; age 15 → 15/60
        assert_eq!(glow_measure(0, 60), 900.0 * (1.0 / 60.0));
        assert_eq!(glow_measure(14, 60), 4.0 * (1.0 / 60.0));
        assert_eq!(glow_measure(15, 60), 15.0 * (1.0 / 60.0));
        assert_eq!(glow_measure(120, 60), 120.0 * (1.0 / 60.0));
        let mut g = GlowSlots::default();
        g.request([0.0; 3], 1, 0.25);
        let tune = nf_formats::driveproj::ELF_DEFAULTS.glow;
        assert_eq!(g.lights(&tune)[0].size, 5.0); // max(0.25, 1) · 5
        g.request([1.0; 3], 0, 3.0);
        g.request([2.0; 3], 2, 0.5); // replaces the largest measure (slot 1)
        assert_eq!(g.slot[1], ([2.0; 3], 2, 0.5));
        assert_eq!(g.n, 2);
    }

    #[test]
    fn emp_light_waits_for_the_homing_delay_and_fades_when_spent() {
        assert_eq!(emp_light([0.0; 3], 19, 19, false, 60, 60), None);
        assert_eq!(emp_light([0.0; 3], 20, 19, false, 60, 60).unwrap(), PointLight { pos: [0.0; 3], rgb: [0.3, 0.3, 1.0], size: 4.0 });
        let l = emp_light([0.0; 3], 90, 19, true, 30, 60).unwrap();
        assert_eq!(l.size, 1.0);
        assert_eq!(l.rgb, [0.3 * 0.5, 0.3 * 0.5, 0.5]);
    }

    #[test]
    fn blast_light_class_slot_and_decay() {
        assert_eq!([blast_class(100.0), blast_class(100.5), blast_class(250.0), blast_class(500.0), blast_class(800.0), blast_class(801.0)], [0, 1, 2, 3, 3, 4]);
        let mut b = BlastLights::default();
        b.add(250.0, [1.0, 0.0, 0.0], 100);
        assert_eq!((b.start[0], b.class[0]), (100, 2));
        let tune = nf_formats::driveproj::ELF_DEFAULTS.blast;
        assert!(b.frame(100, &tune).is_empty()); // dt 0 frees the slot
        b.add(250.0, [1.0, 0.0, 0.0], 100);
        let l = b.frame(101, &tune); // f = 2 - 0.02 = 1.98; 120·1.98 > 16 → 16
        assert_eq!(l[0].rgb, [0.04 * 16.0, 0.03 * 16.0, 0.02 * 16.0]);
        assert_eq!(l[0].size, 0.4);
        let l = b.frame(150, &tune); // f = 1/25 - 0.02 = 0.02
        let k = 120.0 * (1.0f32 / 25.0 - 0.02);
        assert_eq!(l[0].rgb, [0.04 * k, 0.03 * k, 0.02 * k]);
        assert!(!b.frame(199, &tune).is_empty());
        assert!(b.frame(200, &tune).is_empty());
        // full slots: the oldest start is replaced
        let mut b = BlastLights { start: [5, 3, 9, 3], ..Default::default() };
        b.add(50.0, [0.0; 3], 20);
        assert_eq!(b.start, [5, 20, 9, 3]);
    }

    #[test]
    fn explosion_effect_order() {
        let mut c = 0;
        let p = BlastPick { kind: ROCKET, surface: 0, player_flag_bc: false, hit_player: false, damage: 250.0, now: 7 };
        assert_eq!(explosion_effect(&BlastPick { kind: FOD_ROCKET, surface: 9, ..p }, &mut c), "FODHitEffect");
        assert_eq!(explosion_effect(&BlastPick { surface: 9, player_flag_bc: true, ..p }, &mut c), "snowspecificexplosion");
        assert_eq!(explosion_effect(&BlastPick { surface: 6, ..p }, &mut c), "rocketsplash");
        assert_eq!(explosion_effect(&BlastPick { player_flag_bc: true, damage: 601.0, ..p }, &mut c), "Un_ExplL");
        assert_eq!(explosion_effect(&BlastPick { player_flag_bc: true, damage: 600.0, ..p }, &mut c), "Un_ExplM");
        assert_eq!(explosion_effect(&BlastPick { player_flag_bc: true, damage: 200.0, ..p }, &mut c), "Un_ExplS");
        assert_eq!(explosion_effect(&BlastPick { surface: 3, ..p }, &mut c), "Ex_SmallD2"); // 7 % 3
        assert_eq!(c, 0);
        assert_eq!(explosion_effect(&p, &mut c), "Ex_Small");
        assert_eq!(explosion_effect(&BlastPick { hit_player: true, ..p }, &mut c), "Ex_SmB01");
        assert_eq!(explosion_effect(&p, &mut c), "Ex_Small3");
        assert_eq!(c, 0);
    }

    #[test]
    fn projectile_tick_routes_q_pulse_rocket_and_missile() {
        let mut rng = DriveRng::seeded(0);
        let q = Projectile::launch(Q_PULSE, launch_params(Q_PULSE, 0.0, 50.0, 60, Launcher::Player), [0.0; 3], [0.0, 0.0, 1.0], 0);
        let mut fx = ProjectileFx::new(Q_PULSE);
        let d = fx.tick(&q, 30, 60, false, 60, [1.0, 0.0, 0.0], false, &mut rng);
        assert!(d.emp.is_some() && d.glow.is_none() && d.trail.is_empty());
        assert_eq!(fx.trail.ticks, 0);
        let r = Projectile::launch(ROCKET, launch_params(ROCKET, 0.0, 250.0, 60, Launcher::Player), [0.0; 3], [0.0, 0.0, 1.0], 0);
        let mut fx = ProjectileFx::new(ROCKET);
        let mut last = None;
        for now in 0..4 {
            last = Some(fx.tick(&r, now, 60, false, 120, [1.0, 0.0, 0.0], false, &mut rng));
        }
        let d = last.clone().unwrap();
        assert_eq!(d.glow.map(|g| g.0), Some(1));
        assert_eq!(d.trail.len(), 4);
        let m = Projectile::launch(MISSILE, launch_params(MISSILE, 0.0, 250.0, 60, Launcher::Player), [0.0; 3], [0.0, 0.0, 1.0], 0);
        let mut fx = ProjectileFx::new(MISSILE);
        for now in 0..4 {
            last = Some(fx.tick(&m, now, 60, false, 120, [1.0, 0.0, 0.0], false, &mut rng));
        }
        assert!(last.unwrap().trail.is_empty()); // fed, never drawn
        assert_eq!(fx.trail.n, 4);
    }

    #[test]
    fn streaks_advance_purge_and_build_a_camera_facing_quad() {
        let tune = vec![BulletStreak { class: "Henchmen".into(), colour: [255, 210, 95, 35], texture: 1, width: 0.02, length: 1.9, speed: 70.9, count: 2 }];
        let mut s = Streaks::default();
        s.spawn(Streak { start: [0.0; 3], born: 100, end: [0.0, 0.0, 100.0], ty: 0, follows_pov: false });
        let q = s.frame(130, 60, &tune, [0.0, 10.0, 0.0]);
        assert_eq!(q.len(), 1);
        let h = 0.5 * 70.9 + 1.9; // t = 30/60
        let hz = 30.0f32 * (1.0 / 60.0) * 70.9 + 1.9;
        assert!((hz - h).abs() < 1e-4);
        // camera above: v = (0,1,-h)/|..|, side = norm(v × z)·w points along +x
        assert!(q[0].verts[0][0] > 0.0 && (q[0].verts[0][2] - hz).abs() < 1e-5);
        assert_eq!(q[0].verts[2][2], hz - 1.9);
        assert_eq!(q[0].rgba, [127, 105, 47, 17]);
        assert_eq!(q[0].uv, STREAK_UV);
        // past the end it is removed (age 50: head 61 > 50), before the 60-tick purge
        let mut s2 = Streaks::default();
        s2.spawn(Streak { start: [0.0; 3], born: 100, end: [0.0, 0.0, 50.0], ty: 0, follows_pov: false });
        assert_eq!(s2.frame(140, 60, &tune, [0.0, 10.0, 0.0]).len(), 1);
        assert!(s2.frame(150, 60, &tune, [0.0, 10.0, 0.0]).is_empty());
        assert!(s2.list.is_empty());
        // older than 60 ticks: purged first
        let mut s3 = Streaks::default();
        s3.spawn(Streak { start: [0.0; 3], born: 100, end: [0.0, 0.0, 1.0e6], ty: 0, follows_pov: false });
        assert_eq!(s3.frame(160, 60, &tune, [0.0, 10.0, 0.0]).len(), 1);
        assert!(s3.frame(161, 60, &tune, [0.0, 10.0, 0.0]).is_empty());
    }

    #[test]
    fn blast_shake_range() {
        assert_eq!(blast_shake(5.0, true), Some(1.0));
        assert_eq!(blast_shake(20.0, true), Some(0.5));
        assert_eq!(blast_shake(20.5, true), None);
        assert_eq!(blast_shake(5.0, false), None);
    }
}
