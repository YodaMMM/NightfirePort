//! Bullets fired by drones: aiming (`DroneWeap_DoBulletAccuracy` 0x179740), spread (`Bullet_init` 0x12c5a0),
//! flight and collision (`Bullet_update` 0x12e7b0, `Collide_Update`, `Bullet_CollisionHandler` 0x12ce80), damage to
//! Bond (`Drone_ModBulletDamage` 0x139f50, `Player_HandlePain` 0x1902c8). Spec: docs/systems/drone-ai.md "Bullets".

use crate::world::{CollisionWorld, V3};

/// One projectile in flight.
#[derive(Clone, Debug)]
pub struct Bullet {
    pub pos: V3,
    /// Unit direction (`BU+0x00`).
    pub dir: V3,
    /// Units per tick (`BU+0xF8` = `weapon_data+0x20`).
    pub speed: f32,
    /// Distance travelled (`BU+0xF4`).
    pub travelled: f32,
    /// The flight death range: `weapon_data+0x1C` [M `Bullet_update` 0x12e7b0 — case 1 (state 1
    /// via the state jump table at 0x2f4300) kills the round when `weapon+0x1C < BU+0xF4`
    /// (0x12e95c-0x12e984); the step clamps to the +0x1C boundary (0x12f014-0x12f02c) and
    /// `BU+0xF4` keeps the overshoot, so the next tick kills it]. NOT `BU+0xFC` — that is the
    /// fuse/arm clock `Bullet_init` 0x12c5a0:103 writes (210.0, per-id overrides :199-228) and
    /// `Bullet_update` counts down only for `+0x70 & 0x80000` records (0x12ee08) and id 0x45
    /// (0x12efa4, 1.0/tick). laser-watch-0c: no id deviates any more — the Laser watch 0x4E/0x4F
    /// also flies its record's +0x1C = 4.0/8.0 [L weapon_data 0x2bf150]; the old 210.0 spawn-site
    /// stand-in is removed (the per-tick refire's state-0 segments cover [0, +0x1C] from the
    /// muzzle, which is what reaches the locks).
    pub range: f32,
    /// Damage after `Drone_ModBulletDamage` (applied when it hits Bond).
    pub damage: f32,
    pub owner: usize,
    /// State 0 (first tick: step (0.5 + FRand 0.5) · speed), 1 flying, 2 dead.
    pub state: u8,
    /// The weapon that fired it (`Drone+0x62` when the drone fired; the mesh `weapon_data[w]+0x5C` is the
    /// drawn projectile, `Bullet_update` disasm 0x12e8f0 [M]). 0 = none/unknown (hitscan).
    pub weapon: u16,
    /// `weapon_data+0x6C & 0x40000`: the weapon's bullets play a fly-by (whiz) sound in `Bullet_DoTrails` [M].
    pub flyby: bool,
    /// fix-impacts: the ricochet count (`Bullet_CollisionHandler` 0x12d798, killed at 3). Own line (lesson 4).
    pub bounces: u8,
    /// `BU+0xFC`: the fuse clock (`Bullet_init` 0x12c5a0:103 writes 210.0 for every round;
    /// :192-198 the `+0x70 & 0x10000` records get 300.0; :199-228 the satchel family
    /// `weapon+0x02 == 0x3b` gets `((kind − 0x3b)·5 + 5)·60` ticks — kind 0x6c → 900.0 — and
    /// its burn counter 100). Ticked in `Bullet_DoTrails` [M 0x12d890 disasm 0x12dcc4-0x12dd5c],
    /// NOT in `Bullet_update` (whose own fuse arms are the kind-0x3a designator 0x12ee08 and the
    /// kind-0x45 smoke 0x12efa4 only — the comment above already says so).
    pub fuse: f32,
    /// `weapon_data+0x70 & 0x200`: the record's fuse TICKS [M Bullet_DoTrails 0x12d890
    /// 0x12dcc4-0x12dd5c — every tick the round exists, flying (state 1, DoTrails at
    /// Bullet_update's tail 0x12efd8) or landed (state 3, DoTrails at 0x12efdc); family not
    /// 0x37/0x59/0x5b → `BU+0xFC −= FRAME_RATE_MUL` (1.0 [L ee_start_05.bin @0x30d0d8]); at
    /// `fuse < 0` → `obj+0xF4 = 2`, `obj+0xFE |= 1` (the delete, 0x12dd44-0x12dd58) and
    /// `Explode_Create(weapon+0x08 radius, weapon+0x0C damage, script 0x6000052, f12 scale =
    /// radius)` [M 0x12df3c region — the `+0x70 & 0x2000` gas arm and the `& 0x20000` smoke arm
    /// are both clear on the satchel's 0x300 [L ee_start_05.bin], so the plain blast runs].
    pub fuse_ticks: bool,
    /// `BU+0x102` (i16): the laser burn counter, init 100 for the 0x3b family [M Bullet_init
    /// 0x12c5a0:211 `*(undefined2 *)((int)pfVar3 + 0x102) = 100`]. `Bullet_DoTrails`' contact
    /// walk (0x12e2f0-0x12e390): each class-5 hitter whose weapon is the Laser watch 0x4E takes
    /// FRAME_RATE_MUL (1) off, its upgrade 0x4F takes 10 (the `FRAME_RATE_MUL * 10.0` arm).
    pub burn: i16,
    /// The burned-out charge. The game's burn-out arm re-randomizes the fuse EVERY tick to
    /// `60 + Rand_Rand(0x1518)` while the counter stays ≤ 0 [M 0x12e3cc-0x12e40c — the fuse never
    /// decrements again, the charge never explodes]. The port LATCHES instead of redrawing the
    /// shared bit-exact RNG per tick [*deviation*: the observable — no blast ever — is identical;
    /// the game's per-tick `Rand_Rand(5400)` would shift every later draw for every witness].
    pub defused: bool,
    /// `obj+0xF4 = 3`: the landed state [M Bullet_CollisionHandler 0x12ce80 @0x12d1fc — a world
    /// hit for a `+0x70 & 0x100` record with no struck object → state 3, the rest clank
    /// `Sound_Play3D(0x4a7)`; `Bullet_update` case 3 zeroes the speed and runs DoTrails only].
    /// A landed charge rides the mover it rests on (`Control_InheritVelocity` — the game calls it
    /// for every landed round at 0x12edd8; the port applies the mover's own per-tick delta the
    /// way `Player_FeetOnPoint` 0x194188 does for Bond [L the live run: the charge rides the
    /// descending cab — Bond at (1.799, −73.418, −3.453) with the charge on the cab roof]).
    pub landed: bool,
    /// The landing surface's normal (draw-side): the game's landed pose aligns the mesh to the
    /// hit normal (`Mat_Align2Dir(obj+0x90, −normal, …)` [M Bullet_CollisionHandler 0x12ce80:256-266,
    /// the thrown-stick arm's alignment — the port's landing stores the normal for the renderer]).
    /// None while flying.
    pub stick_normal: Option<V3>,
    /// `BU+0x100`: the quarter-second latch of the satchel countdown [M Bullet_DoTrails
    /// 0x12d890:435-441 — the beep fires when `qs != *(ushort*)(BU+0x100)`, then latches].
    pub last_qs: i16,
    /// The two countdown digits this tick (`SatchelDigits[sec/10]`, `SatchelDigits[sec%10]`,
    /// sec = fuse/60 [M 0x12d890:421-422, 485-553 — the digit objects spawn EVERY tick in every
    /// state < 2]). None until the first 0x3b DoTrails tick.
    pub digits: Option<(u8, u8)>,
}

impl Bullet {
    /// The tail defaults every construction site shares (the game's `Bullet_init` head:
    /// fuse 210.0 [M 0x12c5a0:103], no ticking, no burn, not landed). Fields on their own
    /// lines (lesson 4).
    pub const TAIL: Bullet = Bullet {
        pos: [0.0; 3], dir: [0.0; 3], speed: 0.0, travelled: 0.0, range: 0.0, damage: 0.0,
        owner: 0, state: 0, weapon: 0, flyby: false, bounces: 0,
        fuse: 210.0, fuse_ticks: false, burn: 0, defused: false, landed: false,
        stick_normal: None, last_qs: 0, digits: None,
    };

    /// `Bullet_init` 0x12c5a0's fuse block (the tail of the initializer, after the velocity
    /// setup): the base 210.0 [:103], the `+0x70 & 0x10000` → 300.0 arm [:192-198], the
    /// `weapon+0x02 == 0x3b` family arm [:199-211 — `((kind − 0x3b)·5 + 5)·0x3c` ticks, kind
    /// 0x6c → 900.0, the burn counter 100], and the DoTrails tick gate (`+0x70 & 0x200`
    /// [M 0x12d890 0x12dcc4]). `w` is the round's 0x10c-byte weapon record.
    pub fn init_fuse(&mut self, w: &[u8]) {
        let flags70 = u32::from_le_bytes(w[0x70..0x74].try_into().unwrap());
        self.fuse = 210.0; // [M 0x12c5a0:103]
        if flags70 & 0x1_0000 != 0 {
            self.fuse = 300.0; // [M 0x12c5a0:192-198]
        }
        let family = u16::from_le_bytes([w[2], w[3]]);
        let kind = u16::from_le_bytes([w[0], w[1]]);
        if family == 0x3b {
            // [M 0x12c5a0:199-211] `(((uint)*puVar4 - (uint)puVar4[1]) * 5 + 5) * 0x3c` —
            // kind 0x3b → 300, 0x3c → 600, 0x3d → 900 … 0x40 → 1800; kind 0x6c → 900.0.
            self.fuse = if kind == 0x6c { 900.0 } else { (((kind - 0x3b) as u32 * 5 + 5) * 0x3c) as f32 };
            self.burn = 100; // [M 0x12c5a0:211]
        }
        self.fuse_ticks = flags70 & 0x200 != 0; // [M Bullet_DoTrails 0x12dcc4]
    }

    /// The round's `weapon_data+0x02` family halfword (the satchel check's own read,
    /// `lhu v1, 2(v0)` over `BU+0x44`'s record [M Bullet_DoTrails 0x12e200]).
    pub fn family(&self, weapons: &nf_formats::weapon::WeaponData) -> u16 {
        weapons.records.get(self.weapon as usize).map_or(0, |r| u16::from_le_bytes([r[2], r[3]]))
    }
}

/// What a bullet did this tick.
#[derive(Clone, Debug, PartialEq)]
pub enum BulletEvent {
    World(V3),
    Player { damage: f32, at: V3 },
    /// `Bullet_DoTrails` fly-by (sound id 0x1dc): a bullet whizzed past the viewer. Played at the closest point on
    /// the bullet's line to the camera, only for weapons with `+0x6C & 0x40000`, in single player, when the bullet's
    /// owner is not the player, within 3 units, on a 1-in-5 `Rand_Rand(5)` roll [M `Bullet_DoTrails`@0x12d890].
    FlyBy(V3),
}

/// Bond's hit volume: an upright cylinder from his feet to above his eye (*deviation*: the game tests 16 boxes on
/// his skeleton, `HumanCollisionBones` 0x2c6be8 via `Collide_Jointy`; the viewer's Bond has no skeleton).
#[derive(Clone, Copy, Debug)]
pub struct PlayerVolume { pub feet: V3, pub height: f32, pub radius: f32 }

fn norm(v: V3) -> V3 { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(); if l > 0.0 { [v[0] / l, v[1] / l, v[2] / l] } else { v } }

/// `Bullet_init` direction: normalise(normalise(dir) + spherical→cartesian(r, FRand 2π, FRand π)) with
/// r = (FRand(2s) − s) · 0.0014, s = spread `+0x24` × (0 when `weapon_data+0x68 & 0x400000`, else `Drone+0xC0` = 1);
/// the offset's components are added to x, z, y in that order.
pub fn spread(dir: V3, spread: f32, flags68: u32, rng: &mut crate::drone::Rng) -> V3 {
    let s = if flags68 & 0x40_0000 != 0 { 0.0 } else { spread };
    let r = nf_formats::ps2float::mul(rng.frand(2.0 * s) - s, 0.0014);
    let (th, ph) = (rng.frand(crate::trig::TAU), rng.frand(crate::trig::PI));
    // G-046 ps2sinf-remaining: the game's cone is Vec_Spherical_2_Cartesian 0x1e3948 [M]: PS2Sinf2
    // (0x11f968) lane pairs — the port's old form had θ/φ's roles swapped and sin(ph)·sin(th) in z.
    // out = ((r·sinθ)·cosφ, r·sinφ, (r·cosθ)·cosφ), every product an EE mul.s [M disasm 0x1e39a4-0x1e39d4];
    // Bullet_init 0x12c5a0:171-180 then adds them x, z, y (add.s) and normalises twice.
    let (sth, cth) = (crate::trig::sinf(th), crate::trig::cosf(th));
    let (sph, cph) = (crate::trig::sinf(ph), crate::trig::cosf(ph));
    let off = [nf_formats::ps2float::mul(nf_formats::ps2float::mul(r, sth), cph), nf_formats::ps2float::mul(r, sph), nf_formats::ps2float::mul(nf_formats::ps2float::mul(r, cth), cph)];
    let d = norm(dir);
    norm([d[0] + off[0], d[1] + off[2], d[2] + off[1]])
}

/// First hit of the segment `a → b` with Bond's cylinder, as a fraction.
pub fn segment_cylinder(a: V3, b: V3, v: &PlayerVolume) -> Option<f32> {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    let (fx, fz) = (a[0] - v.feet[0], a[2] - v.feet[2]);
    let qa = dx * dx + dz * dz;
    let qb = 2.0 * (fx * dx + fz * dz);
    let qc = fx * fx + fz * fz - v.radius * v.radius;
    let t = if qa < 1e-12 { if qc <= 0.0 { 0.0 } else { return None } } else {
        let disc = qb * qb - 4.0 * qa * qc;
        if disc < 0.0 { return None; }
        let t0 = (-qb - disc.sqrt()) / (2.0 * qa);
        if qc <= 0.0 { 0.0 } else { t0 }
    };
    if !(0.0..=1.0).contains(&t) { return None; }
    let y = a[1] + (b[1] - a[1]) * t;
    (y >= v.feet[1] && y <= v.feet[1] + v.height).then_some(t)
}

impl Bullet {
    /// One tick (`Bullet_update` + collision): the hit test covers the segment from the current position along the
    /// direction for max(step, 1) units; the nearest of world and Bond wins; ranged-out bullets die.
    /// `viewer`: the camera eye (`glb_viewer + 0xc0`), for the `Bullet_DoTrails` fly-by test [M].
    pub fn update(&mut self, world: &CollisionWorld, player: &PlayerVolume, viewer: V3, rng: &mut crate::drone::Rng) -> Option<BulletEvent> {
        if self.state >= 2 { return None; }
        if self.travelled > self.range { self.state = 2; return None; }
        let mut step = if self.state == 0 { (rng.frand(0.5) + 0.5) * self.speed } else { self.speed };
        self.state = 1;
        self.travelled += step;
        if self.travelled > self.range { step -= self.travelled - self.range; }
        let l = step.max(1.0);
        let end = [self.pos[0] + self.dir[0] * l, self.pos[1] + self.dir[1] * l, self.pos[2] + self.dir[2] * l];
        // The flight trace drops the pass-flagged hits (segment_bullet) — the same rule the
        // player-bullet flight walks [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058 arms the
        // HITTEST pick every flying tick (+0x94 = 0x101/0x201, +0x9a = the surface mask), and
        // Collide_Update 0x1ea398:216-229 walks the sorted contact list past every pass-flagged
        // hit (glist node 0x40 / surface byte 0x40, Collide_Intersect disasm 0x1ebf78-0x1ebf80)
        // to the first solid one]. Drone-fired rounds run the same Bullet_update.
        // The candidate set is the game's cel-scoped pick, not the whole world: the bullet is
        // cel-linked at spawn (obj+0x20 = build_LinkToRoom 0x1c9680 [M Bullet_init disasm
        // 0x12c8d0-0x12c8d8]) and the flight query seeds Collide_RayIntersect with that cel,
        // re-fed from the live position every tick [M Bullet_update disasm 0x12ef04
        // `lw $a2,0x20($s5)` + 0x12f0cc-0x12f0d8] — geometry outside the straddle cells is never
        // a candidate [M Collide_Pick 0x1eb320:125 → Collide_StraddleCels 0x1eac98]. The port
        // re-derives the cel with build_FindCel 0x1c8cb0 each tick, exactly like the PlayerBullet
        // flight next door; the no-scope fallback is the flat ray.
        let tw = world.segment_bullet_scoped(world.find_cel(self.pos), self.pos, end);
        let tp = segment_cylinder(self.pos, end, player);
        let at = |t: f32| [self.pos[0] + (end[0] - self.pos[0]) * t, self.pos[1] + (end[1] - self.pos[1]) * t, self.pos[2] + (end[2] - self.pos[2]) * t];
        // CompHitData 0x1d33c0 [M] — QuickSort comparator id 2 (Collide_Update 0x1ea398 / Collide_RayIntersect 0x1ec690 / Collide_SphereIntersect 0x1ec940 / Collide_CylinderIntersect 0x1ecc00 / Collide_FilterBullets 0x1ece30 sort their hit lists by the f32 at HITDATA+0xc, ascending); the port keeps no hit list — the nearest-of comparison below replaces the sort.
        let ev = match (tw, tp) {
            (Some(w), Some(p)) if p < w => Some(BulletEvent::Player { damage: self.damage, at: at(p) }),
            (_, Some(p)) if tw.is_none() => Some(BulletEvent::Player { damage: self.damage, at: at(p) }),
            (Some(w), _) => Some(BulletEvent::World(at(w))),
            _ => None,
        };
        if ev.is_some() { self.state = 2; return ev; }
        let old = self.pos;
        self.pos = [self.pos[0] + self.dir[0] * step, self.pos[1] + self.dir[1] * step, self.pos[2] + self.dir[2] * step];
        // `Bullet_DoTrails` (0x12d890): the fly-by whiz. Only weapons with `+0x6C & 0x40000`, single player (the port is
        // single-player only), and not the player's own bullet (all Bullets are drone-fired here). The closest point on
        // the bullet's path this tick to the viewer; if within 3 units (SqDist < 9), a 1-in-5 `Rand_Rand(5)` roll plays
        // sound 0x1dc at that point [M]. RNG order: `Rand_FRand` (above) → `Rand_Rand(5)` (here), matching the game.
        if self.flyby {
            let cp = closest_point_on_segment(old, self.pos, viewer);
            let d2 = (cp[0] - viewer[0]).powi(2) + (cp[1] - viewer[1]).powi(2) + (cp[2] - viewer[2]).powi(2);
            if d2 < 9.0 && rng.rand(5) == 0 { return Some(BulletEvent::FlyBy(cp)); }
        }
        None
    }
}

/// `vecutil_calculate_closest_point_on_line` (0x1df7e0): the closest point on segment `a→b` to `p`, clamped to [0,1].
/// Returns the point (the game also returns the parametric t, but only the point is used by DoTrails).
fn closest_point_on_segment(a: V3, b: V3, p: V3) -> V3 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let len2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    if len2 < 1e-20 { return a; }
    let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1] + (p[2] - a[2]) * d[2]) / len2).clamp(0.0, 1.0);
    [a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t]
}

/// Bond's health and armour (`BL+0x894`, `+0x8B0`; single player starts at 100 / 0), the hit flash (`BL+0x968`)
/// and the hit-direction bits (`BL+0x967`).
#[derive(Clone, Copy, Debug)]
pub struct PlayerHealth { pub health: f32, pub armour: f32, pub flash: f32, /// `BL+0x967`: the damage-direction bits — 1/2 = the view's vertical axis (sprites 20/21), 4/8 = the horizontal (22/23) [M `HUD_UpdateHealthPane` 0x19b9c8:92-113].
pub dir_bits: u8,
/// The flashbang whiteout, 0..1: `Player_SetFlashBang(FRAME_RATE·f·10, BL, (u8)(f·255))`'s strength [M
/// Bullet_DoTrails 0x12d890:349] — set per blast by the dist (5→90 u) × facing × LOS falloff [M
/// 0x12e014-0x12e0b8], decayed 1/60·10·f per tick (the BL amount [*suggested*: the game's exact fade
/// is the view's `Camera_SetFade`, not modelled]). fix-explosions.
pub flashbang: f32,
/// fix-grenade-types: `BL+0x91C` — the flash's TOTAL time in ticks (`FRAME_RATE·f·10` = 600·f at the
/// 60 Hz FRAME_RATE [D docs/systems/input.md:126]), fixed at the blast. `Player_Update`'s fade gate
/// compares the remaining time's half against it [M 0x193600:340-352].
pub flash_total: f32,
/// fix-grenade-types: the flash's peak alpha as f/255 of `BL+0x963` (the byte `Player_SetFlashBang`
/// stores [M 0x196040]). The renderer paints the whiteout at this alpha until the half point, then
/// fades it with the remaining time [M 0x193600:328-352; GlobalBlur rides the same byte].
pub flash_alpha: f32,
/// `BL+0x910`: the melee hit-push — `Player_DealWithObjHit` 0x194458:239-253 writes ±0.1 per
/// melee hit (weapon 0x65 → −0.1, 0x66/99 → +0.1, gated on the record's u16 `+0x10 & 1`),
/// `Player_Update` decays it ×0.9/tick [M disasm 0x1937a4-0x1937d8] and adds it to the move vec's
/// Y before `obj+0x50 += move` [M disasm 0x193ccc-0x193cf0]. fix-bond-damage.
pub hit_push: f32 }

impl Default for PlayerHealth {
    fn default() -> Self {
        PlayerHealth {
            health: 100.0,
            armour: 0.0,
            flash: 0.0,
            dir_bits: 0,
            flashbang: 0.0,
            hit_push: 0.0,
            flash_total: 0.0,
            flash_alpha: 0.0,
        }
    }
}

impl PlayerHealth {
    /// `Player_HandlePain` (single player, damage type 0): × the level's damage factor for the difficulty
    /// (`Plr_DMod_*`, `ReadTuningVars`; 4 → ×2), armour absorbs first, health floors at 0 (below 1 → 0);
    /// flash += min(d · 21.333, 128), capped at 255. [L] 07000005 at difficulty 1: SG5 hits cost 1.65 = 2.75 × 0.6.
    /// Returns true when the game's 1-in-4 pain-grunt roll hits (the caller plays 0x88).
    pub fn pain(&mut self, rng: &mut crate::drone::Rng, damage: f32, difficulty: u32, level: u32) -> bool {
        self.apply(rng, damage, difficulty, level, 1.0)
    }

    /// The shared `Player_HandlePain` body. `armour_frac` is the pain type's armour fraction
    /// [M 0x1902c8:84-104]: the type switch sets 1.0 for the default/bullet arm and 0.0 for types
    /// 1..7 (the MP kill-feed writes in the 5/6/7 arms fall through to the same 0.0).
    /// Returns true when the pain grunt rolls. The grunt draw sits exactly where the game puts it:
    /// after the armour/health math, before the `+0x967`/`+0x968` writes [M 0x1902c8:110-119]. The
    /// game skips the roll only for pain type 7 with `CheatInfo != 0` (never in SP) — the port is
    /// single-player, so every non-early-outed pain draws.
    fn apply(&mut self, rng: &mut crate::drone::Rng, damage: f32, difficulty: u32, level: u32, armour_frac: f32) -> bool {
        if self.health <= 0.0 || damage <= 0.0 { return false; }
        let d = damage * damage_factor(level, difficulty);
        let absorbed = (d * armour_frac).min(self.armour);
        self.armour -= absorbed;
        self.health = (self.health - (d - absorbed)).max(0.0);
        let grunt = rng.rand(4) == 0; // Rand_Rand(4) [M 0x1902c8:111]
        self.flash = (self.flash + (d * 21.333).min(128.0)).min(255.0);
        grunt
    }

    /// `Player_HandlePain` 0x1902c8:124-130 also writes `BL+0x967`: the generic pain paths set 0xC (both
    /// sides — drowning/pain-type 7 use it too); pain type 6 sets 2. The directional path is
    /// `Player_DealWithObjHit` 0x194458:201-226, which the callers below mirror instead.
    ///
    /// The armour gate follows the game's pain-type switch [M 0x1902c8:84-104]: ONLY type 0 (the
    /// default/bullet arm) lets armour absorb; types 1..7 set armourFraction 0.0 — falls (6,
    /// `Player_CollisionHandler` 0x1949a8:120), drowning (7, `Player_MonitorAir` 0x191a70:142) and
    /// the hurt-volume types drain health alone. Returns the grunt roll (see `apply`).
    pub fn pain_generic(&mut self, rng: &mut crate::drone::Rng, damage: f32, difficulty: u32, level: u32, pain_type: u32) -> bool {
        self.dir_bits = if pain_type == 6 { 2 } else { 0xC };
        self.apply(rng, damage, difficulty, level, if (1..=7).contains(&pain_type) { 0.0 } else { 1.0 })
    }

    /// `Player_DealWithObjHit` 0x194458:201-226: the attacker position transformed into the viewer's frame,
    /// normalised, then `if 0.5 <= |x| or |y|`: `|y| < |x|` → `x < 0 ? 4 : 8`, else `y < 0 ? 1 : 2`.
    /// The game reads the matrix at `glb_viewer+0x160`; the port rebuilds the two axes from Bond's yaw
    /// (right = (cos yaw, 0, −sin yaw), forward = (sin yaw, 0, cos yaw)) and takes the view's y as the
    /// attacker's height over the eye — the same x rule exactly; the y rule is [S] (the world-up
    /// projection of a pitch-rotated frame). Returns the grunt roll (see `apply`).
    pub fn pain_from(&mut self, rng: &mut crate::drone::Rng, damage: f32, difficulty: u32, level: u32, attacker: [f32; 3], eye: [f32; 3], yaw: f32) -> bool {
        self.dir_bits = Self::hit_dir_bits(attacker, eye, yaw);
        self.pain(rng, damage, difficulty, level)
    }

    /// The `BL+0x967` direction bits of `Player_DealWithObjHit` 0x194458:201-226 alone (no pain,
    /// no RNG): `pain_from`'s rule, also used by the MP remote-hit feedback (mp-remote-fx).
    pub fn hit_dir_bits(attacker: [f32; 3], eye: [f32; 3], yaw: f32) -> u8 {
        let d = [attacker[0] - eye[0], attacker[1] - eye[1], attacker[2] - eye[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len <= 1e-4 { return 0; }
        let y = d[1] / len;
        // The port's object frame has +X = screen-left (see `Sim::crosshair`), so the view's x axis is
        // (−cos yaw, 0, sin yaw); x < 0 = the attacker on the screen's left → bit 4 (sprite 22).
        let (rx, rz) = (-yaw.cos(), yaw.sin());
        let vx = (d[0] / len) * rx + (d[2] / len) * rz;
        let vy = y;
        let (ax, ay) = (vx.abs(), vy.abs());
        if ax >= 0.5 || ay >= 0.5 {
            if ay < ax { if vx < 0.0 { 0x4 } else { 0x8 } } else if vy < 0.0 { 0x1 } else { 0x2 }
        } else {
            // Neither axis reaches 0.5 (a shot from straight ahead or behind, e.g. the 07000005 sniper):
            // both sides glow [M 0x194458:202 `bVar8 = 0xc`].
            0xC
        }
    }
}

/// `Plr_DMod_Easy/Normal/Hard` per level (`ReadTuningVars` 0x1d3a80 — the per-level jump-table groups
/// 0x1d3c44-0x1d4878; the ELF statics 0x30cc50/54/58 = 0.75/1.0/1.25 serve the levels no group covers):
/// 0x7000001–04 0.5/0.7/1.2 [0x1d3e00], 05–08 0.6/0.7/1.0 [0x1d3c44], 09–0B 0.6/0.8/1.0 [0x1d3f5c],
/// 0C–0D 0.4/0.7/1.2 [0x1d40f4], 11–13+4A 0.5/0.6/1.0 [0x1d42bc], 14–16 0.4/0.6/1.0 [0x1d4424],
/// 1B 0.6/0.7/1.0 [0x1d458c]; 21–29+4B+4C and every unlisted level keep the statics (their groups
/// write no Plr_DMod — 0x1d46ec/0x1d4878). Difficulty 4 doubles [M Player_HandlePain 0x190470].
pub fn damage_factor(level: u32, difficulty: u32) -> f32 {
    let (e, n, h) = match level & 0xff {
        0x01..=0x04 => (0.5, 0.7, 1.2),
        0x05..=0x08 => (0.6, 0.7, 1.0),
        0x09..=0x0b => (0.6, 0.8, 1.0),
        0x0c | 0x0d => (0.4, 0.7, 1.2),
        0x11..=0x13 | 0x4a => (0.5, 0.6, 1.0),
        0x14..=0x16 => (0.4, 0.6, 1.0),
        0x1b => (0.6, 0.7, 1.0),
        _ => (0.75, 1.0, 1.25),
    };
    // Difficulty 0/other picks Normal, not a no-op [M Player_HandlePain 0x190404-0x190470:
    // 1 → Easy, 2 and every unmatched value → Normal, 3 → Hard, 4 → the ×2 add.s f20,f20,f20].
    match difficulty { 1 => e, 3 => h, 4 => 2.0, _ => n }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// bullet-range-210: the flight death range is `weapon_data+0x1C` [M `Bullet_update` case 1,
    /// 0x12e95c-0x12e984 via the state jump table @0x2f4300], with the step clamped to the
    /// +0x1C boundary [M 0x12f014-0x12f02c] and the death on the NEXT tick (BU+0xF4 keeps the
    /// overshoot). A P2K round (`+0x1C` = 50.0 [D weapon_data 0x2bf150]) flying through open
    /// space stops at ~50 and dies; a round whose record flies 210-class ranges (`+0x1C` 1200,
    /// the sniper family [D]) is alive well past 150 — the same world, same tick count.
    #[test]
    fn p2k_round_stops_at_plus_1c_while_a_1200_record_passes_150() {
        let world = CollisionWorld::empty();
        let vol = PlayerVolume { feet: [1e4, 0.0, 1e4], height: 0.1, radius: 0.1 };
        // The P2K: range 50.0 (the disc's +0x1C), speed 5.0 (+0x20 [D census rec 2]).
        let mut p2k = Bullet { pos: [0.0, 1.0, 0.0], dir: [0.0, 0.0, 1.0], speed: 5.0, travelled: 0.0, range: 50.0,
            damage: 0.0, owner: 0, state: 0, weapon: 0x02, flyby: false, bounces: 0, ..Bullet::TAIL };
        let mut rng = crate::drone::Rng::new(3);
        for _ in 0..40 { if p2k.update(&world, &vol, [1e4, 0.0, 1e4], &mut rng).is_some() { break } }
        assert_eq!(p2k.state, 2, "the P2K round dies past its +0x1C = 50 [M 0x12e95c]");
        let along = p2k.pos[2];
        assert!((45.0..=50.5).contains(&along), "the final position sits at the clamped +0x1C boundary, got {along}");
        assert!(p2k.travelled > 50.0, "BU+0xF4 keeps the overshoot past the boundary [M 0x12f010]");
        // The sniper-family record flies 1200: alive and past 150 after the same 40 ticks.
        let mut sn = Bullet { pos: [0.0, 1.0, 0.0], dir: [0.0, 0.0, 1.0], speed: 5.0, travelled: 0.0, range: 1200.0,
            damage: 0.0, owner: 0, state: 0, weapon: 0x1e, flyby: false, bounces: 0, ..Bullet::TAIL };
        let mut rng2 = crate::drone::Rng::new(3);
        for _ in 0..40 { if sn.update(&world, &vol, [1e4, 0.0, 1e4], &mut rng2).is_some() { break } }
        assert_eq!(sn.state, 1, "the 1200 record's round is still flying");
        assert!(sn.pos[2] >= 150.0, "it passes 150 (got {}), the range the 831 note expected the P2K to fly", sn.pos[2]);
        // The clamp is exact: a 1.0-speed round with range 50 lands its FINAL position within one
        // step of 50.0 [M 0x12f020-0x12f02c clamps the step to the boundary].
        let mut slow = Bullet { pos: [0.0, 1.0, 0.0], dir: [0.0, 0.0, 1.0], speed: 1.0, travelled: 0.0, range: 50.0,
            damage: 0.0, owner: 0, state: 0, weapon: 0x02, flyby: false, bounces: 0, ..Bullet::TAIL };
        let mut rng3 = crate::drone::Rng::new(3);
        for _ in 0..80 { if slow.update(&world, &vol, [1e4, 0.0, 1e4], &mut rng3).is_some() { break } }
        assert!((slow.pos[2] - 50.0).abs() < 1.01, "the clamped boundary: {} vs 50.0", slow.pos[2]);
    }

    /// laser-watch-0c: the Laser watch's own +0x1C — 4.0 (0x4E) / 8.0 (0x4F) [L weapon_data
    /// 0x2bf150 + id·0x10C; the census pins them in nf-formats' bullet_range_census] — is the
    /// watch round's death range like any other id [M 0x12e95c-0x12e984; the 831-era 210.0
    /// spawn-site stand-in is removed]. The watch's speed is 33.33 (+0x20 [L]) and its refire
    /// spawns one round per held tick [L +0x40 = 1; M 0x1a5460:653-658], so the state-0 step
    /// (0.5 + FRand(0.5))·33.33 clamps onto the 4.0 boundary: the FIRST segment covers
    /// [0, 4.0] from the muzzle and the round is dead by the next tick's head check — no watch
    /// segment ever reaches past 4.0 (0x4E). A lock sphere at 4.5 along the ray can never be
    /// contacted; the repro hinge at 1.34 [D probe] is inside the first segment.
    #[test]
    fn the_watch_round_dies_at_plus_1c_after_the_first_segment() {
        let world = CollisionWorld::empty();
        let vol = PlayerVolume { feet: [1e4, 0.0, 1e4], height: 0.1, radius: 0.1 };
        // 0x4E: range 4.0, speed 33.33 [L weapon_data 0x2bf150 + 0x4E·0x10C: +0x1C, +0x20].
        let mut w = Bullet { pos: [0.0, 1.0, 0.0], dir: [0.0, 0.0, 1.0], speed: 33.33, travelled: 0.0, range: 4.0,
            damage: 0.0, owner: 0, state: 0, weapon: 0x4e, flyby: false, bounces: 0, ..Bullet::TAIL };
        let mut rng = crate::drone::Rng::new(3);
        // The first (state-0) tick: the step clamps onto the +0x1C boundary.
        w.update(&world, &vol, [1e4, 0.0, 1e4], &mut rng);
        assert!((w.pos[2] - 4.0).abs() < 1e-3, "the state-0 segment lands ON the 4.0 boundary [M 0x12f014-0x12f02c], got {}", w.pos[2]);
        assert!(w.travelled > 4.0, "BU+0xF4 keeps the overshoot [M 0x12f010], got {}", w.travelled);
        // The next tick's head check kills the round: no further segment.
        w.update(&world, &vol, [1e4, 0.0, 1e4], &mut rng);
        assert_eq!(w.state, 2, "the watch round dies at the +0x1C = 4.0 death [M 0x12e95c-0x12e984]");
        // 0x4F's 8.0: the same law, the boundary at 8.0.
        let mut w2 = Bullet { pos: [0.0, 1.0, 0.0], dir: [0.0, 0.0, 1.0], speed: 33.33, travelled: 0.0, range: 8.0,
            damage: 0.0, owner: 0, state: 0, weapon: 0x4f, flyby: false, bounces: 0, ..Bullet::TAIL };
        let mut rng2 = crate::drone::Rng::new(3);
        w2.update(&world, &vol, [1e4, 0.0, 1e4], &mut rng2);
        assert!((w2.pos[2] - 8.0).abs() < 1e-3, "the 0x4F boundary is 8.0 [L +0x1C], got {}", w2.pos[2]);
    }

    /// fields-player: the armour gate of `Player_HandlePain`'s pain-type switch
    /// [M 0x1902c8:84-104]: ONLY type 0 (the default/bullet arm) lets armour absorb;
    /// types 1..7 (falls 6, drowning 7, hurt volumes) set armourFraction 0.0.
    #[test]
    fn armour_bypass_by_pain_type() {
        // 07000005 at difficulty 1: damage_factor 0.6, so 30.0 -> 18.0 effective.
        let mut h = PlayerHealth { health: 100.0, armour: 50.0, flash: 0.0, dir_bits: 0, flashbang: 0.0, hit_push: 0.0, flash_total: 0.0, flash_alpha: 0.0 };
        let mut rng = crate::drone::Rng::new(0);
        h.pain_generic(&mut rng, 30.0, 1, 0x0700_0005, 6); // fall damage
        assert_eq!(h.armour, 50.0, "a fall must not drain armour [M 0x1902c8:84-104]");
        assert!((h.health - 82.0).abs() < 1e-4, "the fall still costs the full 18 health");
        assert_eq!(h.dir_bits, 2, "pain type 6 sets the vertical bits");
        h.health = 100.0; h.flash = 0.0;
        h.pain_generic(&mut rng, 30.0, 1, 0x0700_0005, 7); // drowning
        assert_eq!(h.armour, 50.0, "drowning must not drain armour either");
        assert_eq!(h.dir_bits, 0xC);
        // Sister: type 0 (bullets, `pain`) DOES absorb — armour first, then health. Armour 10 <
        // the effective 18: it empties and the remainder 8 reaches health.
        h.health = 100.0; h.flash = 0.0; h.dir_bits = 0; h.armour = 10.0;
        h.pain(&mut rng, 30.0, 1, 0x0700_0005);
        assert!((h.armour - 0.0).abs() < 1e-4, "bullets drain the armour first");
        assert!((h.health - 92.0).abs() < 1e-4, "the remainder 18 - 10 reaches health");
    }

    #[test]
    fn cylinder_and_pain() {
        let v = PlayerVolume { feet: [0.0, 0.0, 0.0], height: 1.8, radius: 0.3 };
        assert!(segment_cylinder([-5.0, 1.0, 0.0], [5.0, 1.0, 0.0], &v).is_some_and(|t| (t - 0.47).abs() < 1e-3));
        assert!(segment_cylinder([-5.0, 2.5, 0.0], [5.0, 2.5, 0.0], &v).is_none(), "over his head");
        assert!(segment_cylinder([-5.0, 1.0, 1.0], [5.0, 1.0, 1.0], &v).is_none(), "beside him");
        let mut h = PlayerHealth::default();
        let mut rng = crate::drone::Rng::new(0);
        h.pain(&mut rng, 2.75, 1, 0x0700_0005);
        assert!((h.health - (100.0 - 1.65)).abs() < 1e-4, "live: 1.65 per SG5 hit");
        assert!((h.flash - 1.65 * 21.333).abs() < 1e-3);
    }

    /// `Bullet_DoTrails` fly-by (0x12d890): a bullet passing within 3 units of the viewer on a 1-in-5 roll plays
    /// sound 0x1dc. Sister: a bullet with `flyby=false` never produces a FlyBy, and one too far never does.
    #[test]
    fn flyby_plays_within_three_units_on_one_in_five() {
        let world = CollisionWorld::empty();
        // A clean world (no collision) so the bullet flies past.
        let vol = PlayerVolume { feet: [100.0, 0.0, 100.0], height: 1.8, radius: 0.25 };
        // Viewer at the origin; bullet flies along +z, 2 units to the side (within 3).
        let mut b = Bullet { pos: [0.0, 1.0, -50.0], dir: [0.0, 0.0, 1.0], speed: 15.0, travelled: 0.0, range: 200.0,
            damage: 0.0, owner: 1, state: 0, weapon: 0x06, flyby: true, bounces: 0, ..Bullet::TAIL };
        let mut rng = crate::drone::Rng::new(0);
        // The bullet flies many ticks past the viewer; the 1-in-5 roll should eventually fire.
        let mut fired = false;
        for _ in 0..60 {
            if let Some(BulletEvent::FlyBy(_)) = b.update(&world, &vol, [0.0; 3], &mut rng) { fired = true; break; }
        }
        assert!(fired, "a fly-by bullet within 3 units should fire on the 1-in-5 roll within 60 ticks");
        // Sister: a bullet without the fly-by flag never fires.
        let mut b2 = Bullet { pos: [0.0, 1.0, -50.0], dir: [0.0, 0.0, 1.0], speed: 15.0, travelled: 0.0, range: 200.0,
            damage: 0.0, owner: 1, state: 0, weapon: 0x06, flyby: false, bounces: 0, ..Bullet::TAIL };
        let mut rng2 = crate::drone::Rng::new(0);
        let mut fired2 = false;
        for _ in 0..60 {
            if let Some(BulletEvent::FlyBy(_)) = b2.update(&world, &vol, [0.0; 3], &mut rng2) { fired2 = true; break; }
        }
        assert!(!fired2, "no fly-by flag → no FlyBy event");
        // Sister: a bullet 10 units away (SqDist 100 > 9) never fires even with the flag.
        let mut b3 = Bullet { pos: [10.0, 1.0, -50.0], dir: [0.0, 0.0, 1.0], speed: 15.0, travelled: 0.0, range: 200.0,
            damage: 0.0, owner: 1, state: 0, weapon: 0x06, flyby: true, bounces: 0, ..Bullet::TAIL };
        let mut rng3 = crate::drone::Rng::new(0);
        let mut fired3 = false;
        for _ in 0..60 {
            if let Some(BulletEvent::FlyBy(_)) = b3.update(&world, &vol, [0.0; 3], &mut rng3) { fired3 = true; break; }
        }
        assert!(!fired3, "beyond 3 units → no fly-by");
    }
}
