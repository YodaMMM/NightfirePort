//! mp-combat-weapons: the online match's shared combat — every round any MP agent fires, simulated
//! identically on every machine (docs/systems/mp-combat.md).
//!
//! [S port design — the PS2 game has no online play]: the shooter's own machine decides WHEN a
//! round leaves the weapon (its full fire pipeline: rate, burst, clip, the release frame) and
//! sends the shot description in its per-tick input (`nf_net::Input::shot*`: the fired record,
//! the aim). Every machine then runs the round here from that description, on the shared RNG
//! stream, against every MP body — the same `Bullet_init` / `Bullet_update` / `Explode_Propagate`
//! / `Player_HandlePain` arithmetic the SP port runs for Bond's own rounds:
//!
//! - **spawn** — `Bullet_init` 0x12c5a0: `+0x18` pellets, each with the three spread draws
//!   (`bullet::spread`), `+0x20` speed (× 0.25 for `+0x6C & 0x200` [M :188-190]), `+0x1C` range,
//!   `+0x0C` damage; the grenade family (`+0x6C & 8` gravity with a `+0x70` bounce/stick/contact
//!   bit) becomes a [`Grenade`] with `Bullet_init`'s fuse rules [M :103,196-229]; the remote-mine
//!   detonator (`+0x68 & 0x20000`) re-arms its owner's placed mines to −10.0 [M
//!   `Player_WeaponInitBullet` 0x1a4bf0 disasm 0x1a4c54-0x1a4c88].
//! - **flight** — `Bullet_update` 0x12e7b0: the first tick's (0.5 + FRand 0.5)·speed step, the
//!   case-1 gravity, the `max(step, 1)` hit segment, the range-out; the quarter-speed ramp
//!   (`Bullet_DoTrails` 0x12d890:134-138, speed += full·0.02 per tick); `Bullet_homing` 0x12f358's
//!   steer for `+0x6C & 0x20000`; the guided `+0x6C & 4` rocket steered by its owner's look input
//!   (`Bullet_update` :177-244, `drone::steer_missile`).
//! - **contact** — the nearest of the world (the bullet-family trace, foliage passes) and the
//!   bodies; the contact-explode bits `+0x70 & (4 | 0x100000 | 0x200000)` blast at the contact
//!   [M `Bullet_CollisionHandler` 0x12ce80:194-206].
//! - **blast** — `Explode_Propagate` 0x17c518's truncated falloff (`explode::propagate`) on every
//!   body, the owner included.
//! - **damage** — [`mp_pain`]: `Player_HandlePain` 0x1902c8's MP arm (= `BOT_handlePain`
//!   0x125ed8's): × `Plr_DMod_Multi` for a located hit, the Location Damage table (+0x1C8), ×3
//!   Professional Mode (+0x1BC), armour first.
//!
//! The bodies are hit as upright cylinders and the hit part comes from the contact's height and
//! side [S — the game tests the player's 16 skeletal boxes (`HumanCollisionBones` 0x2c6be8 via
//! `Collide_Jointy` 0x1ec180); the MP bodies are not posed in the shared sim].

use crate::drone::Rng;
use crate::grenade::{Grenade, GrenadeEvent, StuckGadget};
use crate::world::{CollisionWorld, V3};
use nf_formats::tuning::Tuning;
use nf_formats::weapon::WeaponData;

use super::MP_MAX_SLOTS;

/// The body's hit radius [S — the player's skeletal boxes stand-in].
pub const BODY_RADIUS: f32 = 0.35;
/// `Bond::pos` sits this far above the floor (`Bond::stand_at`: floor + 1.1).
pub const FEET_BELOW_POS: f32 = 1.1;
/// The head's top above the eye [S].
pub const HEAD_TOP: f32 = 0.15;

/// The hit parts the MP pain table reads (`HITDATA+0x52`, the anim coll-box row: 5 head, 4 torso,
/// 0x14 upper arm, 0x31 upper leg [M docs/fields/hitdata-spawner-ally.md +0x52]); −1 = no part
/// (`Player_Hurt` 0x196060's blast path).
pub mod part {
    pub const NONE: i16 = -1;
    pub const TORSO: i16 = 4;
    pub const HEAD: i16 = 5;
    pub const UPPER_ARM: i16 = 0x14;
    pub const UPPER_LEG: i16 = 0x31;
}

/// The sender's aim ray → the wire's two i16 angles (any machine-local precision: only the
/// decode must be identical everywhere). Yaw = atan2(x, z) (the port's `Bond::look` convention:
/// dir = (cos p · sin y, sin p, cos p · cos y)).
pub fn aim_from_dir(dir: V3) -> [i16; 2] {
    let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if !(l > 1e-9) { return [0, 0]; }
    let (x, y, z) = (dir[0] / l, dir[1] / l, dir[2] / l);
    let yaw = x.atan2(z);
    let pitch = y.clamp(-1.0, 1.0).asin();
    let yq = (yaw / std::f32::consts::TAU * 65536.0).round() as i32;
    let pq = (pitch / std::f32::consts::FRAC_PI_2 * 32767.0).round().clamp(-32767.0, 32767.0) as i32;
    [yq as i16, pq as i16]
}

/// The wire's aim → the unit ray, identically on every machine (the port's deterministic sine,
/// `trig::sinf` — no host libm).
pub fn dir_from_aim(aim: [i16; 2]) -> V3 {
    let yaw = aim[0] as f32 * (crate::trig::TAU / 65536.0);
    let pitch = aim[1] as f32 * (crate::trig::HALF_PI / 32767.0);
    let (sy, cy) = (crate::trig::sinf(yaw), crate::trig::cosf(yaw));
    let (sp, cp) = (crate::trig::sinf(pitch), crate::trig::cosf(pitch));
    let d = [cp * sy, sp, cp * cy];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9);
    [d[0] / l, d[1] / l, d[2] / l]
}

/// One MP body as the shared combat sees it (a human slot's bond or a bot).
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub slot: usize,
    /// `Bond::pos` (floor + 1.1).
    pub pos: V3,
    /// The eye's height (`Bond::head()[1]` — the crouch included).
    pub head_y: f32,
    pub yaw: f32,
    pub alive: bool,
}

/// The player object's world-sphere radius (`M_CONTROL+0x8C` = 1.0576 [L ee_start_05.bin
/// 0x16cb8a0+0x8C], `Control_BuildWorldSph` 0x133070) — `Explode_Propagate`'s per-object `+0x8C`.
pub const BODY_SPHERE_R: f32 = 1.0576;

/// `Explode_Propagate` 0x17c518's falloff for one object [M decomp :82-85]: `d = |obj+0x80 −
/// pos| − obj+0x8C`, `amount = damage · (1 − (d > 0 ? (int)d : 0) / radius)` — the distance
/// truncated to an integer, an object reaching into the blast centre takes the full damage; below
/// 0.0002 nothing. `radius`/`damage` below 0.0002 hit nothing (the early return).
pub fn blast_falloff(pos: V3, radius: f32, damage: f32, centre: V3, obj_r: f32) -> f32 {
    if !(radius >= 0.0002) || !(damage >= 0.0002) { return 0.0; }
    let dv = [centre[0] - pos[0], centre[1] - pos[1], centre[2] - pos[2]];
    let d = (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt() - obj_r;
    let ti = if d > 0.0 { (d as i32) as f32 } else { 0.0 };
    let a = damage * (1.0 - ti / radius);
    if a < 0.0002 { 0.0 } else { a }
}

impl Body {
    /// The body's world sphere centre (`obj+0x80`): `Bond::sphere_centre`'s yaw-rotated local
    /// offset (−0.1347, −0.1267, 0.2402) [L ee_start_05.bin M_CONTROL+0x80].
    pub fn sphere_centre(&self) -> V3 {
        const LOCAL: V3 = [-0.1347, -0.1267, 0.2402];
        let (s, c) = (crate::trig::sinf(self.yaw), crate::trig::cosf(self.yaw));
        [self.pos[0] + LOCAL[0] * c + LOCAL[2] * s, self.pos[1] + LOCAL[1], self.pos[2] - LOCAL[0] * s + LOCAL[2] * c]
    }

    /// What blast `bl` deals this body (`Explode_Propagate` → `Player_Hurt`, before the pain path).
    pub fn blast_amount(&self, bl: &Blast) -> f32 {
        blast_falloff(bl.pos, bl.radius, bl.damage, self.sphere_centre(), BODY_SPHERE_R)
    }

    fn feet_y(&self) -> f32 { self.pos[1] - FEET_BELOW_POS }
    fn top_y(&self) -> f32 { self.head_y + HEAD_TOP }

    /// The first contact of the segment `a → b` with the body's cylinder, as a fraction.
    pub fn segment_hit(&self, a: V3, b: V3) -> Option<f32> {
        let feet = [self.pos[0], self.feet_y(), self.pos[2]];
        let v = crate::bullet::PlayerVolume { feet, height: self.top_y() - feet[1], radius: BODY_RADIUS };
        crate::bullet::segment_cylinder(a, b, &v)
    }

    /// The hit part from the contact point [S]: the top 0.3 is the head, below the hip (0.95 over
    /// the feet) the legs, a torso-band contact more than 0.22 off the body's centre line
    /// sideways an arm, else the torso.
    pub fn part_at(&self, at: V3) -> i16 {
        if at[1] >= self.top_y() - 0.3 { return part::HEAD; }
        if at[1] < self.feet_y() + 0.95 { return part::UPPER_LEG; }
        // The body's right axis (the port's yaw convention: forward = (sin y, 0, cos y)).
        let (s, c) = (crate::trig::sinf(self.yaw), crate::trig::cosf(self.yaw));
        let side = (at[0] - self.pos[0]) * c - (at[2] - self.pos[2]) * s;
        if side.abs() > 0.22 { part::UPPER_ARM } else { part::TORSO }
    }
}

/// One flying MP round (`Bullet_init`'s object, the non-gravity family).
#[derive(Clone, Debug)]
pub struct Round {
    pub owner: usize,
    pub weapon: u16,
    pub pos: V3,
    pub dir: V3,
    /// `BU+0xF8`.
    pub speed: f32,
    /// `+0x20` — the quarter-speed ramp's ceiling (`+0x6C & 0x200`).
    pub full_speed: f32,
    pub ramp: bool,
    pub travelled: f32,
    pub range: f32,
    /// `+0x0C` — the hit's HITDATA+8.
    pub damage: f32,
    pub first: bool,
    pub gravity: bool,
    pub homing: bool,
    pub guided: bool,
    pub target: Option<usize>,
    /// `+0x70 & (4 | 0x100000 | 0x200000)`: blasts at the contact.
    pub contact_blast: bool,
    /// `+0x08` / `+0x0C`: the contact blast's radius and damage.
    pub radius: f32,
    pub dead: bool,
    /// The tick the round spawned (the ramp skips it, as `flush_world_impacts` does).
    pub born: u32,
}

/// One blast this tick (for the view; the damage is already applied through [`Hurt`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blast { pub pos: V3, pub radius: f32, pub damage: f32, pub owner: usize, pub weapon: u16 }

/// Damage the shared combat dealt this tick, in the game's two routes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hurt {
    /// A round's contact (`Player_DealWithObjHit` 0x194458 case 5 → `MP_RegisterBulletHit` →
    /// `Player_HandlePain(HITDATA+8, part)`; bots `BOT_handlePain`).
    Bullet { victim: usize, attacker: usize, amount: f32, part: i16, weapon: u16, dir: V3, at: V3 },
    /// A blast (`Explode_Propagate` → `Player_Hurt` 0x196060 → `Player_HandlePain(amount, −1)`).
    Blast { victim: usize, attacker: usize, amount: f32, weapon: u16, at: V3 },
    /// A round reached the Demolition/Protection site first (the site's hit damage [M
    /// 0x1870a0:81-91]; the model-sphere test is the port's stand-in [S]).
    Site { attacker: usize, amount: f32 },
}

/// The closest-approach fraction of the segment `a → b` to a sphere, when it passes inside.
fn segment_sphere(a: V3, b: V3, c: V3, r: f32) -> Option<f32> {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    if dd < 1e-9 { return None; }
    let t = (((c[0] - a[0]) * d[0] + (c[1] - a[1]) * d[1] + (c[2] - a[2]) * d[2]) / dd).clamp(0.0, 1.0);
    let p = [a[0] + d[0] * t - c[0], a[1] + d[1] * t - c[1], a[2] + d[2] * t - c[2]];
    (p[0] * p[0] + p[1] * p[1] + p[2] * p[2] <= r * r).then_some(t)
}

/// The guided-rocket ride per slot (`Player_ChangeSubState(owner, 10)` at the launch [M
/// `Bullet_init` 0x12c5a0:234-254]; the 15-tick restore after the rocket dies [M `Bullet_Delete`
/// 0x12e660:28 `+0x944 = 0xf`, `Player_Update` 0x193600:242-256]). While `riding || restore > 0`
/// the slot's body runs no arms — its movement input is zeroed on every machine.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ride { pub riding: bool, pub restore: u8 }

/// What one fired shot spawned (for the view: the remote muzzle/tracer effects).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fired { pub owner: usize, pub weapon: u16, pub origin: V3, pub dir: V3 }

/// The shared combat state (hashed by [`MpCombat::hash_into`]).
#[derive(Clone, Debug, Default)]
pub struct MpCombat {
    pub rounds: Vec<Round>,
    /// The grenade family in the world (thrown, bouncing, stuck, armed), owner = the slot.
    pub thrown: Vec<Grenade>,
    /// This tick's blasts / shots (cleared at the start of each [`MpCombat::step`] / fire batch).
    pub blasts: Vec<Blast>,
    pub fired: Vec<Fired>,
    pub ride: [Ride; MP_MAX_SLOTS],
}

fn f32_at(w: &[u8], o: usize) -> f32 { f32::from_le_bytes(w[o..o + 4].try_into().unwrap()) }
fn u32_at(w: &[u8], o: usize) -> u32 { u32::from_le_bytes(w[o..o + 4].try_into().unwrap()) }

/// `Bullet_init` 0x12c5a0's fuse (`BU+0xFC`, the port's `Grenade.range`) for a grenade-family
/// record fired by a PLAYER — the SP throw's rules (scene.rs `'shot` block): a no-fuse record
/// (`+0x70 & 0x200` clear) ranges out silently at `+0x1C / speed` ticks [M `Bullet_update`
/// 0x12e7b0:158-162]; else 210, 300 with `+0x70 & 0x10000` [M :103,196-229]; the satchel family
/// (base 0x3b) `((id − base)·5 + 5)·60`, 0x6c → 900 [M :199-208]; 0x69 → 5 [M :212-214]; the
/// player's remote mine 0x37 → 7777.0 (detonator-only) [M :214-223].
pub fn grenade_fuse(w: &[u8]) -> f32 {
    let speed = f32_at(w, 0x20);
    let flags70 = u32_at(w, 0x70);
    let fuse = if flags70 & 0x200 == 0 {
        (f32_at(w, 0x1c) / speed).max(1.0)
    } else if flags70 & 0x10000 != 0 { 300.0 } else { 210.0 };
    let wid = u16::from_le_bytes([w[0], w[1]]);
    let base = u16::from_le_bytes([w[2], w[3]]);
    let fuse = if base == 0x3b {
        if wid == 0x6c { 900.0 } else { (wid.wrapping_sub(base) as f32 * 5.0 + 5.0) * 60.0 }
    } else if base == 0x69 { 5.0 } else { fuse };
    if wid == 0x37 { 7777.0 } else { fuse }
}

/// The MP pain multipliers (`Plr_DMod_*` from the level's tuning, `ReadTuningVars` 0x1d3a80 —
/// the MP arenas' group: Multi 4.0, Head 4.0, Lower/UpperLimb 0.8 [M nf-game tuning.rs]) and the
/// Player Mods switches: +0x1C8 "Location Damage", +0x1BC "Professional Mode" [L USATxt.dat
/// labels 0x24e / 0x1ac, the rows P_MPPLAYERMODS 0x2104d0:14-15 bind].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PainRules { pub multi: f32, pub head: f32, pub upper: f32, pub lower: f32, pub location: bool, pub professional: bool }

impl PainRules {
    pub fn new(t: &Tuning, location: bool, professional: bool) -> PainRules {
        PainRules { multi: t.Plr_DMod_Multi, head: t.Plr_DMod_Head, upper: t.Plr_DMod_UpperLimb, lower: t.Plr_DMod_LowerLimb, location, professional }
    }
}

/// `Player_HandlePain` 0x1902c8's MP multiplier block [M :55-80], identical in `BOT_handlePain`
/// 0x125ed8 [M :49-74]: a located hit (`part != −1`) × `Plr_DMod_Multi`; with Location Damage
/// on, part 5 × Head, parts 0x14/0x15/0x20/0x23 × UpperLimb, 0x31..0x38 × LowerLimb (the switch
/// on `part − 5`: 0 / 0xf, 0x10, 0x1b, 0x1e / 0x2c..0x33); Professional Mode × 3.0.
pub fn mp_damage(r: &PainRules, amount: f32, part: i16) -> f32 {
    let mut d = amount;
    if part != part::NONE { d *= r.multi; }
    if r.location {
        match part.wrapping_sub(5) {
            0 => d *= r.head,
            0xf | 0x10 | 0x1b | 0x1e => d *= r.upper,
            0x2c..=0x33 => d *= r.lower,
            _ => {}
        }
    }
    if r.professional { d *= 3.0; }
    d
}

/// The armour + health half of `Player_HandlePain` [M 0x1902c8:82-134] (pain type 0 — bullets and
/// blasts both arrive as type 0, armourFraction 1.0): `absorbed = min(d, armour)`, `armour −=
/// absorbed`, `health = max(health − (d − absorbed), 0)`, below 1 → 0. Returns (health, armour).
pub fn apply_pain(health: f32, armour: f32, d: f32) -> (f32, f32) {
    let absorbed = if d < armour { d } else { armour };
    let armour = armour - absorbed;
    let h = health - (d - absorbed);
    let h = if h > 0.0 { h } else { 0.0 };
    (if h < 1.0 { 0.0 } else { h }, armour)
}

impl MpCombat {
    /// Whether `slot`'s body is held by a guided-rocket ride (its movement input is zeroed).
    pub fn riding(&self, slot: usize) -> bool {
        self.ride.get(slot).is_some_and(|r| r.riding || r.restore > 0)
    }

    /// The live guided rocket `slot` rides, if any.
    pub fn ridden_round(&self, slot: usize) -> Option<&Round> {
        self.rounds.iter().find(|r| r.owner == slot && r.guided && !r.dead)
    }

    /// One shot from `owner` (its head `origin`, the decoded aim `dir`) with record `weapon`:
    /// `Player_WeaponInitBullet` 0x1a4bf0 → `Bullet_init` 0x12c5a0. Draws the shared stream in the
    /// game's order (per pellet: the three spread draws). `frame` stamps the spawn.
    pub fn fire(&mut self, wd: &WeaponData, owner: usize, weapon: u8, origin: V3, dir: V3, rng: &mut Rng, frame: u32) {
        let Some(w) = wd.records.get(weapon as usize) else { return };
        let w = &w[..];
        let flags68 = u32_at(w, 0x68);
        let flags6c = u32_at(w, 0x6c);
        let flags70 = u32_at(w, 0x70);
        self.fired.push(Fired { owner, weapon: weapon as u16, origin, dir });
        // The detonator arm [M 0x1a4bf0 disasm 0x1a4c30-0x1a4cd4]: every bullet of the fired
        // record's base owned by the firing player takes BU+0xFC = −10.0 (the port's fuse model:
        // range = travelled − 10); no bullet spawns.
        if flags68 & 0x2_0000 != 0 {
            let base = wd.base(weapon as usize);
            for g in self.thrown.iter_mut() {
                if g.state < 2 && g.owner == owner && wd.base(g.weapon_type as usize) == base {
                    g.range = g.travelled - 10.0;
                }
            }
            return;
        }
        // The grenade family (the SP routing gate: gravity AND a bounce/stick/contact bit).
        if flags6c & 0x8 != 0 && flags70 & (0x4 | 0x80 | 0x100) != 0 {
            let sdir = crate::bullet::spread(dir, f32_at(w, 0x24), flags68, rng);
            let speed = f32_at(w, 0x20);
            self.thrown.push(Grenade {
                pos: origin,
                velocity: [sdir[0] * speed, sdir[1] * speed, sdir[2] * speed],
                speed, travelled: 0.0, range: grenade_fuse(w), damage: f32_at(w, 0x0c), radius: f32_at(w, 0x08),
                weapon_type: weapon as u16,
                mesh: wd.projectile_mesh(weapon as usize),
                owner, state: 0, bounces: 0, smoke_popped: false, resting: false,
                stuck: false, seek_target: None,
                phase: matches!(weapon as u16, 0x3a | 0x3b).then(|| {
                    Box::new(StuckGadget { normal: [0.0, 1.0, 0.0], arm: 0.0, armed: false, last_qs: 0, digits: None })
                }),
            });
            return;
        }
        // The bullet family: `+0x18` pellets [M 0x1a4bf0's per-pellet loop], each spread.
        let pellets = u32_at(w, 0x18).clamp(1, 16);
        let full = f32_at(w, 0x20);
        let quarter = flags6c & 0x200 != 0;
        for _ in 0..pellets {
            let sdir = crate::bullet::spread(dir, f32_at(w, 0x24), flags68, rng);
            self.rounds.push(Round {
                owner, weapon: weapon as u16, pos: origin, dir: sdir,
                speed: if quarter { full * 0.25 } else { full },
                full_speed: full, ramp: quarter,
                travelled: 0.0, range: f32_at(w, 0x1c), damage: f32_at(w, 0x0c),
                first: true,
                gravity: flags6c & 0x8 != 0,
                homing: flags6c & 0x2_0000 != 0,
                guided: flags6c & 0x4 != 0,
                target: None,
                contact_blast: flags70 & (0x4 | 0x10_0000 | 0x20_0000) != 0,
                radius: f32_at(w, 0x08),
                dead: false,
                born: frame,
            });
        }
        // `Bullet_init` 0x12c5a0:234-254: a `+0x6C & 4` round hands its owner's view to it.
        if flags6c & 0x4 != 0 && owner < MP_MAX_SLOTS {
            self.ride[owner] = Ride { riding: true, restore: 0 };
        }
    }

    /// One shared combat tick: every round and grenade moves, contacts resolve, blasts propagate.
    /// `steer[slot]` = the slot's look input (turn, look) this tick (the guided rocket's steering).
    /// Returns the damage dealt, in the order it happened.
    /// `site` = the scenario site's sphere (Demolition/Protection), a round target too.
    pub fn step(&mut self, world: &CollisionWorld, wd: &WeaponData, bodies: &[Body], site: Option<(V3, f32)>, steer: &[(f32, f32); MP_MAX_SLOTS], rng: &mut Rng, frame: u32) -> Vec<Hurt> {
        self.blasts.clear();
        let mut hurts = Vec::new();
        let mut blasts: Vec<Blast> = Vec::new();
        // 1. The rounds (`Bullet_update` 0x12e7b0), in spawn order.
        for r in self.rounds.iter_mut() {
            if r.dead { continue; }
            if r.travelled > r.range { r.dead = true; continue; }
            // The guided ride's steering [M 0x12e7b0:177-244] — the scene's own conversion (the
            // port's mouse `turn` radians / `look` π/2 units → the game's analog, capped).
            if r.guided && r.owner < MP_MAX_SLOTS {
                const K: f32 = 0.024_543_693;
                let (turn, look) = steer[r.owner];
                let yaw_in = (-turn / K).clamp(-1.0, 1.0);
                let pitch_in = (-look * crate::trig::HALF_PI / K).clamp(-1.0, 1.0);
                crate::drone::steer_missile(&mut r.dir, yaw_in, pitch_in);
            }
            let mut step = if r.first { (rng.frand(0.5) + 0.5) * r.speed } else { r.speed };
            let first = r.first;
            r.first = false;
            if r.gravity && !first {
                let (vx, vy, vz) = (r.dir[0] * r.speed, r.dir[1] * r.speed, r.dir[2] * r.speed);
                let vy = vy + crate::grenade::WLD_GRAVITY[1] * crate::grenade::REC_FRAME_RATE * crate::grenade::REC_FRAME_RATE;
                let len = (vx * vx + vy * vy + vz * vz).sqrt();
                if len > 1e-9 { r.dir = [vx / len, vy / len, vz / len]; r.speed = len; }
            }
            r.travelled += step;
            if r.travelled > r.range { step -= r.travelled - r.range; }
            let l = step.max(1.0);
            let end = [r.pos[0] + r.dir[0] * l, r.pos[1] + r.dir[1] * l, r.pos[2] + r.dir[2] * l];
            let tw = world.segment_surface_bullet(r.pos, end).map(|(t, _, _)| t);
            let mut tb: Option<(f32, usize)> = None;
            for (bi, b) in bodies.iter().enumerate() {
                if !b.alive || b.slot == r.owner { continue; }
                if let Some(t) = b.segment_hit(r.pos, end) {
                    if tb.is_none_or(|(bt, _)| t < bt) { tb = Some((t, bi)); }
                }
            }
            let at = |t: f32| [r.pos[0] + (end[0] - r.pos[0]) * t, r.pos[1] + (end[1] - r.pos[1]) * t, r.pos[2] + (end[2] - r.pos[2]) * t];
            // The site takes the round when it is the nearest thing on the segment.
            if let Some(ts) = site.and_then(|(c, rad)| segment_sphere(r.pos, end, c, rad)) {
                if tb.is_none_or(|(t, _)| ts < t) && tw.is_none_or(|w| ts <= w) {
                    hurts.push(Hurt::Site { attacker: r.owner, amount: r.damage });
                    if r.contact_blast { blasts.push(Blast { pos: at(ts), radius: r.radius, damage: r.damage, owner: r.owner, weapon: r.weapon }); }
                    r.dead = true;
                    continue;
                }
            }
            match (tb, tw) {
                (Some((t, bi)), w) if w.is_none_or(|w| t <= w) => {
                    let p = at(t);
                    let b = &bodies[bi];
                    hurts.push(Hurt::Bullet { victim: b.slot, attacker: r.owner, amount: r.damage, part: b.part_at(p), weapon: r.weapon, dir: r.dir, at: p });
                    if r.contact_blast { blasts.push(Blast { pos: p, radius: r.radius, damage: r.damage, owner: r.owner, weapon: r.weapon }); }
                    r.dead = true;
                }
                (_, Some(t)) => {
                    // The world contact: stops (the port's MP rounds neither pierce nor
                    // ricochet [S — the SP walk's penetration/ricochet draws are left out]).
                    if r.contact_blast { blasts.push(Blast { pos: at(t), radius: r.radius, damage: r.damage, owner: r.owner, weapon: r.weapon }); }
                    r.dead = true;
                }
                _ => r.pos = [r.pos[0] + r.dir[0] * step, r.pos[1] + r.dir[1] * step, r.pos[2] + r.dir[2] * step],
            }
            if r.dead { continue; }
            // `Bullet_homing` 0x12f358 (after the move): keep a live target, else the nearest
            // live body (not the owner) within 1000 inside the 0.66-dot cone with a clear
            // bullet-family LOS; steer each component by clamp((to − dir)·0.0333, ±0.01).
            if r.homing {
                let live = |s: usize| bodies.iter().find(|b| b.slot == s && b.alive);
                if r.target.is_some_and(|t| live(t).is_none()) { r.target = None; }
                if r.target.is_none() {
                    let mut best: Option<(usize, f32)> = None;
                    for b in bodies.iter().filter(|b| b.alive && b.slot != r.owner) {
                        let to = [b.pos[0] - r.pos[0], b.pos[1] - r.pos[1], b.pos[2] - r.pos[2]];
                        let dist = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
                        if dist <= 0.0 || dist > 1000.0 { continue; }
                        if best.is_some_and(|(_, bd)| dist >= bd) { continue; }
                        let dot = (to[0] * r.dir[0] + to[1] * r.dir[1] + to[2] * r.dir[2]) / dist;
                        if dot <= 0.66 { continue; }
                        if world.segment_bullet(r.pos, b.pos).is_some() { continue; }
                        best = Some((b.slot, dist));
                    }
                    r.target = best.map(|(s, _)| s);
                }
                if let Some(tp) = r.target.and_then(|t| live(t)).map(|b| b.pos) {
                    let to = [tp[0] - r.pos[0], tp[1] - r.pos[1], tp[2] - r.pos[2]];
                    let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
                    if l > 0.0 {
                        let s = |c: f32, d: f32| ((c - d) * 0.0333).clamp(-0.01, 0.01);
                        let d = [r.dir[0] + s(to[0] / l, r.dir[0]), r.dir[1] + s(to[1] / l, r.dir[1]), r.dir[2] + s(to[2] / l, r.dir[2])];
                        let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                        if m > 0.0 { r.dir = [d[0] / m, d[1] / m, d[2] / m]; }
                    }
                }
            }
            // The quarter-speed ramp [M `Bullet_DoTrails` 0x12d890:134-138], not on the spawn tick.
            if r.ramp && r.born != frame && r.speed < r.full_speed { r.speed += r.full_speed * 0.02; }
        }
        // The guided rides: a slot whose rocket died starts the 15-tick restore.
        for s in 0..MP_MAX_SLOTS {
            let alive = self.rounds.iter().any(|r| r.owner == s && r.guided && !r.dead);
            let ride = &mut self.ride[s];
            if ride.riding && !alive { ride.riding = false; ride.restore = 15; }
            else if !ride.riding && ride.restore > 0 { ride.restore -= 1; }
        }
        self.rounds.retain(|r| !r.dead);
        // 2. The grenade family (`Bullet_update` + `Bullet_CollisionHandler` + `Bullet_DoTrails`).
        for gi in 0..self.thrown.len() {
            let before = self.thrown[gi].pos;
            let flags70 = wd.records.get(self.thrown[gi].weapon_type as usize).map_or(0, |r| u32_at(&r[..], 0x70));
            let ev = self.thrown[gi].update(world, rng);
            let g = &mut self.thrown[gi];
            match ev {
                // The contact-explode records (the MGL 'Grenade' 0x2A, the AIMS launcher 0x1B)
                // blast at the world contact [M 0x12ce80:194-206]; the rest bounce.
                Some(GrenadeEvent::Bounce(p)) if flags70 & 4 != 0 => {
                    blasts.push(Blast { pos: p, radius: g.radius, damage: g.damage, owner: g.owner, weapon: g.weapon_type });
                    g.state = 2;
                }
                // The fuse expiry: only `+0x70 & 0x200` records blast [M `Bullet_DoTrails`
                // 0x12d890:262-281]; the no-fuse records range out silently [M 0x12e7b0:158-162].
                // The stun (0x2000) and smoke (0x20000) branches carry no Explode_Create — the
                // flash/cloud are the view's (agent 11); radius/damage 0 hits nobody anyway.
                Some(GrenadeEvent::Explode { pos, radius, damage, owner, weapon_type }) => {
                    if flags70 & 0x200 != 0 && flags70 & (0x2000 | 0x2_0000) == 0 {
                        blasts.push(Blast { pos, radius, damage, owner, weapon: weapon_type });
                    }
                }
                _ => {}
            }
            if g.state >= 2 || g.stuck { continue; }
            // A contact-explode grenade meeting a body on this tick's move blasts there [M
            // 0x12ce80:194-206 — the explode arm runs on object contacts too].
            if flags70 & 4 != 0 {
                let after = g.pos;
                let mut tb: Option<(f32, usize)> = None;
                for (bi, b) in bodies.iter().enumerate() {
                    if !b.alive || b.slot == g.owner { continue; }
                    if let Some(t) = b.segment_hit(before, after) {
                        if tb.is_none_or(|(bt, _)| t < bt) { tb = Some((t, bi)); }
                    }
                }
                if let Some((t, _)) = tb {
                    let p = [before[0] + (after[0] - before[0]) * t, before[1] + (after[1] - before[1]) * t, before[2] + (after[2] - before[2]) * t];
                    blasts.push(Blast { pos: p, radius: g.radius, damage: g.damage, owner: g.owner, weapon: g.weapon_type });
                    g.state = 2;
                }
            }
        }
        // The placed caps: at most 6 stuck mines / tripbombs per owner, the oldest goes [M
        // `Bullet_CollisionHandler` 0x12ce80:296-370 — `bVar4 = count < 6`, the oldest marked].
        for kind in [0x37u16, 0x3a] {
            for owner in 0..MP_MAX_SLOTS {
                let mut placed: Vec<usize> = (0..self.thrown.len()).filter(|&i| {
                    let g = &self.thrown[i];
                    g.state < 2 && g.stuck && g.owner == owner && wd.base(g.weapon_type as usize) == kind
                }).collect();
                while placed.len() > 6 { let i = placed.remove(0); self.thrown[i].state = 2; }
            }
        }
        // The armed tripbombs' beams [M `Bullet_update` case 3 0x12efb8-0x12efc8]: the ray from the
        // stuck position along +normal to the first wall (200 max); a live body within 1.0 of it
        // destructs the bomb → its 8/100 blast (the SP Drones::step arm's shape).
        for g in self.thrown.iter_mut() {
            if !(g.weapon_type == 0x3a && g.stuck && g.state < 2) { continue; }
            let Some(p) = g.phase.as_ref().filter(|p| p.armed) else { continue };
            let (pos, n) = (g.pos, p.normal);
            let far = [pos[0] + n[0] * 200.0, pos[1] + n[1] * 200.0, pos[2] + n[2] * 200.0];
            let t = world.segment(pos, far).unwrap_or(1.0).min(1.0);
            let ab = [(far[0] - pos[0]) * t, (far[1] - pos[1]) * t, (far[2] - pos[2]) * t];
            let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
            let near = |q: V3| {
                let u = if ab2 > 1e-9 { (((q[0] - pos[0]) * ab[0] + (q[1] - pos[1]) * ab[1] + (q[2] - pos[2]) * ab[2]) / ab2).clamp(0.0, 1.0) } else { 0.0 };
                let c = [pos[0] + ab[0] * u - q[0], pos[1] + ab[1] * u - q[1], pos[2] + ab[2] * u - q[2]];
                c[0] * c[0] + c[1] * c[1] + c[2] * c[2] < 1.0
            };
            if bodies.iter().any(|b| b.alive && near(b.pos)) {
                blasts.push(Blast { pos, radius: g.radius, damage: g.damage, owner: g.owner, weapon: 0x3a });
                g.state = 2;
            }
        }
        self.thrown.retain(|g| g.state < 2);
        // 3. The blasts (`Explode_Propagate` 0x17c518), each in order — a chained one appends:
        // - case 3 (players) / the bots: every live body in the sphere, the owner included
        //   (`Player_Hurt` 0x196060);
        // - case 0x35 (MP objects): the scenario site takes the falloff (`MP_ApplyDamage`);
        // - case 5 (bullets): the grenade-family records 0x2B, 0x34..0x37, 0x3A in the sphere are
        //   destructed (`Bullet_handle_object_destruction` [M :141-162]) — a frag, mine, tripbomb
        //   or MGL Delay round chains its own blast; the stun/smoke (their flash/cloud branches
        //   carry no Explode_Create) just go [S — their effects are the view's].
        let mut i = 0;
        while i < blasts.len() {
            let bl = blasts[i];
            i += 1;
            for b in bodies.iter().filter(|b| b.alive) {
                let a = b.blast_amount(&bl);
                if a > 0.0 { hurts.push(Hurt::Blast { victim: b.slot, attacker: bl.owner, amount: a, weapon: bl.weapon, at: bl.pos }); }
            }
            if let Some((c, rad)) = site {
                let a = blast_falloff(bl.pos, bl.radius, bl.damage, c, rad);
                if a > 0.0 { hurts.push(Hurt::Site { attacker: bl.owner, amount: a }); }
            }
            for g in self.thrown.iter_mut() {
                if g.state >= 2 || !matches!(g.weapon_type, 0x2b | 0x34..=0x37 | 0x3a) { continue; }
                if blast_falloff(bl.pos, bl.radius, bl.damage, g.pos, 0.0) <= 0.0 { continue; }
                g.state = 2;
                let f70 = wd.records.get(g.weapon_type as usize).map_or(0, |r| u32_at(&r[..], 0x70));
                if f70 & (0x2000 | 0x2_0000) == 0 {
                    blasts.push(Blast { pos: g.pos, radius: g.radius, damage: g.damage, owner: g.owner, weapon: g.weapon_type });
                }
            }
        }
        self.thrown.retain(|g| g.state < 2);
        self.blasts = blasts;
        hurts
    }

    /// Folds the whole shared combat state into the lockstep hash (fixed order).
    pub fn hash_into(&self, h: &mut impl FnMut(u32)) {
        let f = |v: f32| v.to_bits();
        h(self.rounds.len() as u32);
        for r in &self.rounds {
            h(r.owner as u32); h(r.weapon as u32);
            for k in 0..3 { h(f(r.pos[k])); h(f(r.dir[k])); }
            h(f(r.speed)); h(f(r.travelled)); h(r.first as u32); h(r.target.map_or(u32::MAX, |t| t as u32));
        }
        h(self.thrown.len() as u32);
        for g in &self.thrown {
            h(g.owner as u32); h(g.weapon_type as u32);
            for k in 0..3 { h(f(g.pos[k])); h(f(g.velocity[k])); }
            h(f(g.travelled)); h(f(g.range)); h(g.state as u32); h(g.bounces as u32);
            h(g.stuck as u32); h(g.resting as u32);
            h(g.phase.as_ref().map_or(0, |p| (p.armed as u32) | ((f(p.arm)) << 1)));
        }
        for r in &self.ride { h(r.riding as u32); h(r.restore as u32); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(location: bool, professional: bool) -> PainRules {
        // The MP arenas' tuning [M nf-game tuning.rs 0x07000021.. group]: Multi 4, Head 4, limbs 0.8.
        PainRules { multi: 4.0, head: 4.0, upper: f32::from_bits(0x3f4ccccd), lower: f32::from_bits(0x3f4ccccd), location, professional }
    }

    #[test]
    fn pain_table_per_part() {
        let r = rules(true, false);
        assert_eq!(mp_damage(&r, 3.5, part::TORSO), 14.0, "a torso hit × Multi");
        assert_eq!(mp_damage(&r, 3.5, part::HEAD), 56.0, "the head × Multi × Head");
        assert!((mp_damage(&r, 3.5, part::UPPER_LEG) - 3.5 * 4.0 * 0.8).abs() < 1e-5);
        assert!((mp_damage(&r, 3.5, part::UPPER_ARM) - 3.5 * 4.0 * 0.8).abs() < 1e-5);
        assert_eq!(mp_damage(&r, 60.0, part::NONE), 60.0, "a blast (no part) takes no multiplier");
        // Location Damage off: every located hit is just × Multi.
        let off = rules(false, false);
        assert_eq!(mp_damage(&off, 3.5, part::HEAD), 14.0);
        // Professional Mode × 3, blasts too.
        let pro = rules(false, true);
        assert_eq!(mp_damage(&pro, 10.0, part::NONE), 30.0);
        // Sister: the head row must differ from the torso row (the table can fail).
        assert_ne!(mp_damage(&r, 3.5, part::HEAD), mp_damage(&r, 3.5, part::TORSO));
    }

    #[test]
    fn armour_absorbs_first_and_health_floors() {
        assert_eq!(apply_pain(100.0, 25.0, 40.0), (85.0, 0.0));
        assert_eq!(apply_pain(100.0, 50.0, 40.0), (100.0, 10.0));
        assert_eq!(apply_pain(10.0, 0.0, 9.5), (0.0, 0.0), "below 1 → 0 [M 0x1902c8:130]");
        assert_eq!(apply_pain(10.0, 0.0, 50.0), (0.0, 0.0));
        // Sister: no armour → the full amount reaches health.
        assert_ne!(apply_pain(100.0, 0.0, 40.0), apply_pain(100.0, 25.0, 40.0));
    }

    #[test]
    fn aim_roundtrip_is_close_and_deterministic() {
        for d in [[0.0f32, 0.0, 1.0], [0.6, -0.2, -0.77], [-0.3, 0.9, 0.31], [1.0, 0.0, 0.0]] {
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let d = [d[0] / l, d[1] / l, d[2] / l];
            let back = dir_from_aim(aim_from_dir(d));
            let dot = d[0] * back[0] + d[1] * back[1] + d[2] * back[2];
            assert!(dot > 0.999_999, "{d:?} → {back:?}");
            assert_eq!(back, dir_from_aim(aim_from_dir(d)), "the decode is a pure function");
        }
        // Sister: a different aim decodes to a different ray.
        assert_ne!(dir_from_aim([100, 0]), dir_from_aim([0, 0]));
    }

    #[test]
    fn body_parts_by_height_and_side() {
        let b = Body { slot: 1, pos: [0.0, 1.1, 0.0], head_y: 1.8, yaw: 0.0, alive: true };
        assert_eq!(b.part_at([0.0, 1.85, 0.0]), part::HEAD);
        assert_eq!(b.part_at([0.0, 0.5, 0.0]), part::UPPER_LEG);
        assert_eq!(b.part_at([0.0, 1.3, 0.0]), part::TORSO);
        assert_eq!(b.part_at([0.3, 1.3, 0.0]), part::UPPER_ARM);
        // The cylinder: a ray through the chest hits, one over the head misses.
        assert!(b.segment_hit([-5.0, 1.3, 0.0], [5.0, 1.3, 0.0]).is_some());
        assert!(b.segment_hit([-5.0, 2.5, 0.0], [5.0, 2.5, 0.0]).is_none());
    }

    #[test]
    fn blast_falloff_truncates_and_saturates() {
        // Radius 8, damage 100: an object whose sphere edge is 2.7 away takes 100·(1 − 2/8) = 75.
        assert_eq!(blast_falloff([0.0; 3], 8.0, 100.0, [3.7, 0.0, 0.0], 1.0), 75.0);
        // Inside its own sphere: the full damage, never more [M 0x17c518:84 — the (d > 0) select].
        assert_eq!(blast_falloff([0.0; 3], 8.0, 100.0, [0.5, 0.0, 0.0], 1.0576), 100.0);
        // At the radius: nothing.
        assert_eq!(blast_falloff([0.0; 3], 8.0, 100.0, [9.1, 0.0, 0.0], 1.0), 0.0);
        // The zero-radius stun/smoke: nothing.
        assert_eq!(blast_falloff([0.0; 3], 0.0, 100.0, [0.1, 0.0, 0.0], 1.0), 0.0);
        // Sister: the port's SP `explode::propagate` overshoots deep inside a sphere (its negative
        // truncation), the game's select does not.
        let t = crate::explode::Targets { drones: &[(0, [-1.5, 0.0, 0.0], 3.0)], player: [1e6; 3], breaks: &[], destroys: &[] };
        let sp = crate::explode::propagate([0.0; 3], 8.0, 100.0, 0, &t).drones[0].1;
        assert_ne!(sp, blast_falloff([0.0; 3], 8.0, 100.0, [-1.5, 0.0, 0.0], 3.0));
    }

    #[test]
    fn fuse_rules() {
        let mut w = [0u8; nf_formats::weapon::RECORD];
        let set = |w: &mut [u8], o: usize, v: &[u8]| w[o..o + v.len()].copy_from_slice(v);
        // A frag 0x34: +0x70 = 0x280 → 210.
        set(&mut w, 0, &0x34u16.to_le_bytes()); set(&mut w, 2, &0x34u16.to_le_bytes());
        set(&mut w, 0x20, &0.2f32.to_le_bytes()); set(&mut w, 0x70, &0x280u32.to_le_bytes());
        assert_eq!(grenade_fuse(&w), 210.0);
        // The player's remote mine 0x37 → 7777.
        set(&mut w, 0, &0x37u16.to_le_bytes()); set(&mut w, 2, &0x37u16.to_le_bytes());
        assert_eq!(grenade_fuse(&w), 7777.0);
        // Satchel 0x3c (base 0x3b) → (1·5+5)·60 = 600.
        set(&mut w, 0, &0x3cu16.to_le_bytes()); set(&mut w, 2, &0x3bu16.to_le_bytes());
        assert_eq!(grenade_fuse(&w), 600.0);
        // A no-fuse impact record ranges out at +0x1C / speed.
        set(&mut w, 0, &0x2au16.to_le_bytes()); set(&mut w, 2, &0x2au16.to_le_bytes());
        set(&mut w, 0x70, &0x4u32.to_le_bytes()); set(&mut w, 0x1c, &100.0f32.to_le_bytes());
        assert_eq!(grenade_fuse(&w), 500.0);
    }
}
