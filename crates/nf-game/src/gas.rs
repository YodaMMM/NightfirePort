//! The gas/smoke puffs: `Gas_Init` 0x1aacc8 (creator) and `gases_smoke` 0x1ab2d0 (control class 0xE's
//! update, docs/atlas/object.md:108). Every caller passes the same mesh hash 0x0200039 (`smoke_1_dust`,
//! a 4-vertex/2-triangle quad [D gfx-list]) via `hashtable_set_object_to_entity_gfx` [M 0x1aacc8:81], so
//! a puff is a growing, fading, spinning instance of one shared quad.
//!
//! Game model [M 0x1aacc8]:
//! - obj+0xE8 = scale start (param 1); data+0x34 = (param 2 − param 1) / life (the growth rate);
//!   data+0x38 = life (param 11, ticks — the update decrements it by FRAME_RATE_MUL = 1 per tick);
//!   data+0x3C byte = the behaviour type (param 8).
//! - data+0x20..0x2C = the current rgba as floats, started from the packed param 9 (e.g. the muzzle
//!   0x48403080 = r 0x48 g 0x40 b 0x30 a 0x80) and moved toward the packed param 10 (the game's callers
//!   pass an integer whose float bits are ~0, i.e. fade to zero) by data+0x10..0x1C rates each tick
//!   [M 0x1aacc8:56-71, 0x1ab2d0 tail]; obj+0x103..0x106 carry the byte form the draw reads.
//! - data+0x30 = the spin rate: `Rand_FRand(π/2 · REC_FRAME_RATE)`, negated when `Rand_Rand(0x100) > 0x7f`
//!   [M 0x1aacc8:72-80] — 2 shared-RNG draws per puff.
//! - Velocity by type [M 0x1aacc8:89-141]: type 4 = the given velocity · REC_FRAME_RATE with a +[0,0.05)
//!   x/z jitter (2 draws, 0x1ab21c/0x1ab248); type 7 with a NULL velocity = `Env_GetWindVel` + the
//!   `(rand−0.1, rand, rand−0.1)·0.2` spread (0x3e4ccccd = 0.2, 3 draws, 0x1ab160/0x1ab184/0x1ab1a4);
//!   the default = (0, REC_FRAME_RATE, 0)?? — the decomp's `pfVar1[1] = fVar10` with fVar10 = REC_FRAME_RATE.
//! - The decompile drops `Gas_Init`'s 11th argument (the life, a stack ushort) at EVERY call site — the
//!   disassembly passes it [M 0x1aad24 `lhu 0xc0(sp)`; the searchlight site 0x1cbce0/0x1cbcb0 = 0x80].
//! - Update (`gases_smoke` 0x1ab2d0) per type: 0 = vel damped ×0.95/frame + slight gravity, scale grows;
//!   4 = pos += vel (+ gravity·0.01), vel damped; 7 = pos += vel + WldGravity·(REC·−0.02) — the wind
//!   drift KEPT, a slight upward buoyancy [M 0x1ab2d0:47-65, rocket-trail-gas];
//!   9/10 = hidden emitters (not ported); the tail fades rgba and kills at life ≤ 0 (obj+0xFE |= 1).
//!
//! RNG discipline: the game's draw count per puff is 1 (the caller's rotation draw, if its site makes
//! one) + 2 (spin, sign) + the velocity branch — the searchlight's type-7/NULL-vel site consumes **6**
//! (the old "5" here counted only the internal ones). `init_ex`/`init` now consume the game's own
//! `Gas_Init` draws (spin `Rand_FRand(π/2·REC_FRAME_RATE)` 0x1ab060, sign `Rand_Rand(0x100)` 0x1ab08c
//! [M 0x1aacc8:72-80], then the kind-4 given-vel jitter `Rand_FRand(0.05)` ×2 [M 0x1ab21c/0x1ab248] or
//! the kind-7/NULL-vel wind `Rand_FRand(0.2)` ×3 [M 0x1ab160/0x1ab184/0x1ab1a4]) for every site the
//! game reaches through `Gas_Init`; the port stand-ins that are NOT game `Gas_Init` calls (the swoosh
//! ribbon puff, the GT destroy burst) stay on `init_hash`'s counter hash — the game consumes no draws
//! there, so the stream must not move either.

use crate::world::V3;

/// The one shared dust quad (`smoke_1_dust`), every `Gas_Init` caller's mesh [M 0x1aacc8:81 callers;
/// D gfx-list 07000029 entry 64 obj 0].
pub const SMOKE_QUAD: u32 = 0x0200_0039;

/// Every mesh hash the pool's own `Gas_Init` sites can carry, beyond the `EffectInfo` rows' gas
/// entries (those the renderer collects from `SurfaceSounds::effects` itself). flesh-hit-effects:
/// the renderer's draw-slot list must cover these or the puff sits in the pool with nothing to
/// draw it through — the missing red puffs were the blood puff 0x0200_08bc with no slot:
/// - `SMOKE_QUAD` — the `Gas_Init` default (muzzle, searchlight, laser burn, flashbang, exhaust,
///   ripples) [M 0x1aacc8:81];
/// - `0x0200_001f` — the projectile trail smoke [M `Bullet_trail_effect_smoke` 0x12cba8 disasm
///   0x12cc10-0x12cc74, gas.rs `init_trail_smoke`];
/// - `0x0200_08bc` — the FLESH hit's blood puff [M `NDrone2_DealWithObjHit` 0x13fff0's default arm,
///   disasm 0x140470 `ori $t0, $t0, 0x8bc` — sizes 3.0/6.0, kind 5, rgba 0xffffffff → 0xffffff00,
///   life 0x3c, and the gas obj's `+0xf0 |= 8` draw flag];
/// - `0x0200_026e` — the Phoenix Samurai bolt's flesh puff [M the 0x32/0x33 arm, disasm
///   0x140408 `ori $t0, $t0, 0x26e` — sizes 1.0/16.0];
/// - `0x0200_021d` — the breath puff [M `Env_Breath` 0x17be00, `lui $t0, 0x200` + `ori $t0, $t0,
///   0x21d` at 0x17bf14/0x17bfb8; env.rs `BREATH_MESH`].
pub const POOL_MESHES: &[u32] = &[SMOKE_QUAD, 0x0200_001f, 0x0200_08bc, 0x0200_026e, crate::env::BREATH_MESH];

/// REC_FRAME_RATE (0x30d0dc — GS_SetRefreshRate 0x17d2b8 sets it = 1/FRAME_RATE; 0x30d0cc is
/// FRAME_RATE_INT) = 1/60 [M symtab; grenade.rs:52].
pub(crate) const REC_FRAME_RATE: f32 = 1.0 / 60.0;
/// WldGravity [L grenade.rs:51; M the global the update's `_WldGravity` reads].
const WLD_GRAVITY: V3 = [0.0, -9.8, 0.0];

#[derive(Clone, Debug, PartialEq)]
pub struct Gas {
    pub pos: V3,
    pub vel: V3,
    /// The puff's mesh hash (`Gas_Init`'s hashtable_set_object_to_entity_gfx argument); the callers'
    /// common case is SMOKE_QUAD, the `Effect_Bullet` entries carry their own (0x02000124 hard-surface,
    /// 0x02000129 water [D EffectInfo@0x3037c0]). fix-impacts, additive field.
    pub mesh: u32,
    pub scale: f32,
    pub scale_rate: f32,
    /// The current rgba as floats (data+0x20), byte-clamped into the draw colour each tick.
    pub rgba: [f32; 4],
    pub rgba_rate: [f32; 4],
    /// data+0x30: the spin rate (radians/tick); the draw billboards the quad with it.
    pub spin: f32,
    /// data+0x38: ticks left; the update kills the object at ≤ 0.
    pub life: f32,
    /// data+0x3C byte: 0 = static damped, 4 = given velocity, 7 = buoyant drift.
    pub kind: u8,
    /// The gas obj's +0xF0 bit 3 (0x8) — set only by the blood-puff arm [M 0x13fff0:244-249; the
    /// bolt case :251-255 does not]: `View_DrawObjectGlist` 0x1e6780:43-66 rotates a bit-3 object
    /// from the VIEWER matrix (billboard) instead of its own +0x90. The draw side reads it.
    pub billboard: bool,
    /// The gas obj's +0xF0 bit 0x400 — set only by `Env_Breath`'s mode-2 puff [M 0x17c024-0x17c02c]:
    /// a 0x400 object is submitted to a view only while that viewer's goggle mode (+0x236) is 2
    /// [M View_AddObjects 0x1e62d8:38-47]. The draw side reads it.
    pub xray_only: bool,
    /// The spawn rotation (`Gas_Init`'s param-5 euler vector — the quaternion argument of
    /// `control_create_object` [M 0x1aacc8:31]; the game's gas objects are NOT billboards —
    /// `GAS_OBJ_FLAGS` carries no 0x8 — so `View_DrawObjectGlist` rotates them from their own
    /// +0x90). fix-scrape-and-impacts row 13: `Effect_Bullet`'s wall puffs spawn with
    /// `Vec_NormToRot(normal)`, floor puffs with the negated-normal euler or zero by the entry's
    /// +0x1b byte [M 0x1ce8a8:122-130]. The draw side reads it; every pre-existing site keeps
    /// [0,0,0] (the game's sites pass zero or the euler — zero except the impact puffs).
    pub rot: [f32; 3],
    /// The gas obj's +0xF0 bit 0x10 — the HIDDEN bit [M Gas_Init 0x1aacc8's LAB_001ab1e4
    /// `*(obj+0xf0) |= 0x10` for kinds 9/10; gases_smoke's cases 9/10 re-OR it every tick].
    /// A hidden puff never draws (fx-gas-and-blasts row 5): it is a stationary cloud EMITTER
    /// whose children are the visible puffs. The draw side reads it.
    pub hidden: bool,
    /// obj+0xec — the creation GameState, written by `control_init_object` 0x134630:15; the
    /// kind-9 emitter's warmup reads it (`GameState - obj[0xec] < 20` blocks the child spawns
    /// [M gases_smoke 0x1ab2d0 case 9]). Set from the pool's frame at init.
    pub born: u32,
    /// The kind-9/10 emitter's child-spawn record — the game stores Gas_Init's OWN args at the
    /// object's data table (+0x40..+0x58: rgba, end rgba, glist, p1, p2, life [M 0x1aacc8:46-52,
    /// the `8 < kind < 0xb` block]) and the update cases re-pass them to every child's Gas_Init
    /// [M 0x1ab2d0 case 9 / case 10], with the record's rgb re-combined with the parent's LIVE
    /// alpha byte each spawn (`(rec & 0xffffff00) | (u8)obj[0x106]`).
    pub child: Option<ChildSeed>,
}

/// The kind-9/10 emitter's stored Gas_Init args (see `Gas::child`).
#[derive(Clone, Debug, PartialEq)]
pub struct ChildSeed {
    pub p1: f32,
    pub p2: f32,
    pub mesh: u32,
    pub rgba: [u8; 4],
    pub rgba_end: u32,
    pub life: f32,
}

/// The obj+0xF0 bits `Gas_Init` ORs into every puff after `hashtable_set_object_to_entity_gfx` has
/// copied the glist flags in [M 0x1aacc8:83 `*(obj+0xf0) |= 0x20020120`]. It is the only writer of
/// bit 0x20000 in ACTION.ELF [M grep of every obj+0xF0 store], and no glist on the disc carries it in
/// its +0x44 flags [D nf-formats tests/night_quarter_alpha_census.rs].
pub const GAS_OBJ_FLAGS: u32 = 0x2002_0120;

/// The obj+0xF0 bit `View_SetupRenderModes_Tweak` 0x1e5888:12-17 tests: with it set and the viewer
/// in night sight (viewer+0x236 == 1), the alpha byte handed to `psiSetTweakARGB` is obj+0x106 >> 2.
pub const NIGHT_QUARTER_ALPHA: u32 = 0x2_0000;

impl Gas {
    /// obj+0x106 as `gases_smoke` 0x1ab2d0's tail writes it: `(int)a` clamped to 0..255.
    pub fn alpha_byte(&self) -> u8 {
        let a = self.rgba[3];
        if a >= 255.0 { 255 } else if a > 0.0 { a as u8 } else { 0 }
    }

    /// The draw's alpha factor (byte/255) for a viewer in goggle mode `mode`: the night-sight
    /// quarter [M View_SetupRenderModes_Tweak 0x1e5888:12-17, `bVar3 = obj+0x106 >> 2`], and the
    /// pool's live alpha otherwise (the unchanged mode-0/2 path).
    pub fn tweak_alpha(&self, mode: u16) -> f32 {
        if mode == 1 && GAS_OBJ_FLAGS & NIGHT_QUARTER_ALPHA != 0 {
            (self.alpha_byte() >> 2) as f32 / 255.0
        } else {
            self.rgba[3] / 255.0
        }
    }
}

#[derive(Default)]
pub struct GasPool {
    pub puffs: Vec<Gas>,
    /// The spawn counter feeding the deterministic jitter (the game's 5 RNG draws, un-consumed).
    counter: u32,
    /// The current GameState tick (`update` stores it; `init_ex` stamps it into `Gas::born` for
    /// the kind-9 warmup — the game reads obj+0xec, GameState at creation [M 0x134630:15]).
    pub frame: u32,
}

/// `Rand_Rand(0x100) > 0x7f`-quality sign/offset from the counter: a 32-bit bit-mix (xxhash-style), no
/// shared-stream involvement. The game's draws come from Rand_FRand — the port substitutes this [S].
fn jitter(counter: u32, salt: u32) -> f32 {
    let mut h = counter ^ salt;
    h = h.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    // 0..1
    (h & 0xffff) as f32 / 65536.0
}

impl GasPool {
    /// Slot cap: the muzzle puffs live 1 tick life... the game's searchlight break re-fires every 7th
    /// frame with the same parameters, so a few dozen live at once in play; 128 covers grenade chains.
    pub const CAP: usize = 128;

    /// `Gas_Init` 0x1aacc8 for the meshless callers (SMOKE_QUAD, rgba end 0): consumes the game's
    /// draws (see `init_ex`). `p1`/`p2` = the scale start/end, `rgba` = the packed param 9 (r, g, b, a),
    /// `yaw` = the quaternion z the callers pass (a drawn angle for the muzzle/breath A sites,
    /// unused by the port's billboard draw beyond the spin), `vel` = the given velocity vector.
    #[allow(clippy::too_many_arguments)]
    pub fn init(&mut self, rng: &mut crate::drone::Rng, pos: V3, yaw: f32, vel: V3, p1: f32, p2: f32, kind: u8, rgba: [u8; 4], life: f32) {
        self.init_ex(rng, crate::gas::SMOKE_QUAD, pos, yaw, vel, p1, p2, kind, rgba, 0, life);
    }

    /// The port stand-ins that are NOT game `Gas_Init` calls (the swoosh ribbon puff
    /// `Swoosh_Update` 0x18d200 has no gas/RNG path of its own [M 0x18d200:49-82]; the GT destroy
    /// burst is the port's presentation of `Emitter_Create` [M 0x180498:249-257]). The game consumes
    /// no shared draws for them, so neither does the port — the old counter-hash body, kept verbatim
    /// so the stream stays where the game's is.
    #[allow(clippy::too_many_arguments)]
    pub fn init_hash(&mut self, mesh: u32, pos: V3, yaw: f32, vel: V3, p1: f32, p2: f32, kind: u8, rgba: [u8; 4], rgba_end: u32, life: f32) {
        if life <= 0.0 || self.puffs.len() >= Self::CAP { return; }
        let n = self.counter.wrapping_add(0x9e37_79b9);
        self.counter = n;
        let spin = jitter(n, 1) * (std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE)
            * if jitter(n, 2) > 0.5 { 1.0 } else { -1.0 };
        let vel = match kind {
            4 => [vel[0] * REC_FRAME_RATE + (jitter(n, 3) - 0.025), vel[1] * REC_FRAME_RATE, vel[2] * REC_FRAME_RATE + (jitter(n, 4) - 0.025)],
            7 => [(jitter(n, 3) * 0.2 - 0.1) * REC_FRAME_RATE, jitter(n, 5) * 0.2 * REC_FRAME_RATE, (jitter(n, 4) * 0.2 - 0.1) * REC_FRAME_RATE],
            _ => [0.0, REC_FRAME_RATE, 0.0],
        };
        let _ = yaw;
        let end = [(rgba_end >> 24) as u8, (rgba_end >> 16) as u8, (rgba_end >> 8) as u8, rgba_end as u8];
        self.puffs.push(Gas {
            pos, vel,
            mesh,
            scale: p1,
            scale_rate: (p2 - p1) / life,
            rgba: rgba.map(f32::from),
            rgba_rate: [
                (f32::from(end[0]) - f32::from(rgba[0])) / life,
                (f32::from(end[1]) - f32::from(rgba[1])) / life,
                (f32::from(end[2]) - f32::from(rgba[2])) / life,
                (f32::from(end[3]) - f32::from(rgba[3])) / life,
            ],
            spin,
            life,
            kind,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
        if let Some(g) = self.puffs.last_mut() { g.mesh = mesh; }
    }

    /// `Gas_Init` for the broken-searchlight site (`Searchlight_Update` 0x1cb900:373-385), consuming the
    /// game's 6 shared-RNG draws in order [M disassembly]:
    /// 1. the caller's rotation draw `Rand_FRand(π)` (0x1cbca0, the rot-z euler — 0x40c90fdb);
    /// 2. the spin `Rand_FRand(π/2 · REC_FRAME_RATE)` (0x1ab060);
    /// 3. the spin sign `Rand_Rand(0x100)` — negate when ≥ 0x80 (0x1ab08c, decomp :77-80);
    /// 4-6. the type-7/NULL-vel wind spread `Rand_FRand(0.2)` ×3 (0x1ab160/0x1ab184/0x1ab1a4):
    ///     `vel = (wind.x + r − 0.1, wind.y + r, wind.z + r − 0.1) · REC_FRAME_RATE`, wind =
    ///     `Env_GetWindVel` 0x17c048 = 0 with no cel wind radiator (none ported — [S] the cel wind
    ///     system stays unported). Fixed args of the site: p1 0.05, p2 0.25, type 7, rgba 0x48403080,
    ///     end 1 (~0 float bits), life 128 ticks (the decomp-dropped 11th arg = 0x80 [M 0x1cbce0]).
    /// The draws are consumed BEFORE the pool-cap check — the game's RNG has no notion of the port's
    /// cap, and a skipped draw would shift the shared stream.
    pub fn init_searchlight_smoke(&mut self, rng: &mut crate::drone::Rng, pos: V3) {
        let yaw = rng.frand(std::f32::consts::PI);
        let spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        let (rx, ry, rz) = (rng.frand(0.2), rng.frand(0.2), rng.frand(0.2));
        if self.puffs.len() >= Self::CAP { return; }
        let vel = [rx - 0.1, ry, rz - 0.1].map(|j| mul_ee(j, REC_FRAME_RATE));
        self.puffs.push(Gas {
            pos,
            vel,
            mesh: SMOKE_QUAD,
            scale: 0.05,
            scale_rate: (0.25 - 0.05) / 128.0,
            rgba: [0x48 as f32, 0x40 as f32, 0x30 as f32, 0x80 as f32],
            rgba_rate: [-0x48 as f32 / 128.0, -0x40 as f32 / 128.0, -0x30 as f32 / 128.0, -0x80 as f32 / 128.0],
            spin,
            life: 128.0,
            kind: 7,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
        let _ = yaw; // the rot-z euler: stored to obj+0x50/0x60 in the game; the port's billboard draw reads only the spin
    }

    /// The GT turret's wreck smoke — `GT_Update` 0x180498 case 6 (DESTROYED), every 10th game
    /// frame while the wreck timer runs [M decompile L440-451: the `(int)GameState % 10 == 0`
    /// branch calls `Gas_Init(0.05, 0.25, cel, obj+0x30, rot-euler, 0, glist 0x2000039, 7,
    /// 0x48403080, 1)`]. The same 6-draw kind-7 shape as the searchlight site (the caller's
    /// `Rand_FRand(π)` at decompile L446 feeds the rot-z euler, then Gas_Init's own spin/sign/wind
    /// draws), the same fixed args and the sibling kind-7 life 128 (the decompile-dropped 11th
    /// arg; both other kind-7 sites read 0x80 — searchlight [M 0x1cbce0], player muzzle
    /// [M 0x1a43f8/0x1a442c]).
    pub fn init_gt_smoke(&mut self, rng: &mut crate::drone::Rng, pos: V3) {
        self.init_searchlight_smoke(rng, pos);
    }

    /// The GT turret's SHOT smoke — `GT_Update` 0x180498's two fire sites (case 4 @:356 = jal
    /// 0x180c38, case 7 @:530 = jal 0x181350), on every OTHER shot: the fire tail toggles the
    /// flash object's barrel byte `+0xf6 ^= 1` (and re-arms the flash `+0xf4 = 2`, unhides
    /// `+0xf0 &= ~0x10`) and calls `Gas_Init` only when the toggled value is 1 [M disasm
    /// 0x180bd0-0x180bf8]. Fixed args [M disasm 0x180bfc-0x180c3c]: scale 0.05 → 0.1 (the scale arg =
    /// 0x3d4ccccd, f13/f20 = 0x3dcccccd), cel = the GT's (obj+0x20), pos = the muzzle (the same
    /// vector `Bullet_init` spawned the round from), ROT vector z = the caller's draw, VEL =
    /// NULL → the kind-7 wind path, glist 0x2000039, kind 7, rgba 0x48403080, 11th arg 1, life
    /// 128 (stack `sw $v0(0x80), ($sp)` [M 0x180c00-0x180c1c]).
    /// Draws per smoke, in order [M]: 1. the caller's `Rand_FRand(2π)` (0x180bc0, 0x40c90fdb —
    /// stored to the ROT vector's z, sp+0x88; the puff's initial spin angle — the port's
    /// billboard reads only the spin rate, discarded here); 2. the spin `Rand_FRand(π/2·REC)`
    /// [M 0x1aacc8:72-80]; 3. the spin sign `Rand_Rand(0x100)` — negate at ≥ 0x80 [M 0x1ab08c];
    /// 4-6. the NULL-vel kind-7 wind spread `Rand_FRand(0.2)` ×3 over `Env_GetWindVel` (0 in the
    /// port's rooms) [M 0x1ab138-0x1ab1c8]. All consumed BEFORE the pool-cap check (the
    /// searchlight precedent).
    pub fn init_gt_shot_smoke(&mut self, rng: &mut crate::drone::Rng, pos: V3) {
        let _rot_z = rng.frand(6.2831853); // 0x40c90fdb [M 0x180bc0]
        let spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        let (rx, ry, rz) = (rng.frand(0.2), rng.frand(0.2), rng.frand(0.2));
        if self.puffs.len() >= Self::CAP { return; }
        let vel = [rx - 0.1, ry, rz - 0.1].map(|j| mul_ee(j, REC_FRAME_RATE));
        self.puffs.push(Gas {
            pos,
            vel,
            mesh: SMOKE_QUAD,
            scale: 0.05, // [M 0x180c0c-0x180c14 the scale arg = 0x3d4ccccd]
            scale_rate: (0.1 - 0.05) / 128.0, // [M f13/f20 = 0x3dcccccd = 0.1; life 128]
            rgba: [0x48 as f32, 0x40 as f32, 0x30 as f32, 0x80 as f32],
            rgba_rate: [-0x48 as f32 / 128.0, -0x40 as f32 / 128.0, -0x30 as f32 / 128.0, -0x80 as f32 / 128.0],
            spin,
            life: 128.0, // [M 0x180c00-0x180c1c stack arg 0x80]
            kind: 7,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
    }

    /// The mounted-gun OVERHEAT smoke — `GunImp_Update` 0x1320d8:218-247: while the hot byte
    /// (`ext+0x9c`, 200 heat) stands, every 6th game frame (`GameState % 6 == 0` [M 0x132480-8c
    /// divu/mfhi]) emits `Gas_Init(0.05, 0.25, cel, pos, rot, NULL-vel, 0x2000039, 7, 0x484030e4,
    /// 1)` with life 128 (stack `sw $v0(0x80), ($sp)` [M 0x132510-0x132540]). The position is the
    /// barrel-tip matrix product offset by `-(obj+0x8c) · U(0,1)` along the aim direction
    /// [M :224-241 — `fStack_100 = FRand(1.0)`, `fStack_100 = -(obj+0x8c)·fStack_100`, the VU0
    /// `matrix·vec + base` over `Mat_GetDir(obj+0x90)`]; obj+0x8c is the imp's world-sphere
    /// radius (the port's `GunImp.radius`). The ROT vector z = the caller's `Rand_FRand(2π)`
    /// [M :242-244]. VEL NULL → the kind-7 wind path (as `init_gt_shot_smoke`).
    /// Draws per emission, in order [M]: 1. `Rand_FRand(1.0)` (0x1324b4 — the offset fraction);
    /// 2. `Rand_FRand(2π)` (0x132504, 0x40c90fdb); 3. the spin; 4. the sign; 5-7. the wind ×3.
    pub fn init_gunimp_smoke(&mut self, rng: &mut crate::drone::Rng, pos: V3, back: V3, radius: f32) {
        let k = rng.frand(1.0); // [M 0x1324b4] — the offset fraction along the barrel axis
        let _rot_z = rng.frand(6.2831853); // [M 0x132504]
        let spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        let (rx, ry, rz) = (rng.frand(0.2), rng.frand(0.2), rng.frand(0.2));
        if self.puffs.len() >= Self::CAP { return; }
        let pos = [pos[0] + back[0] * radius * k, pos[1] + back[1] * radius * k, pos[2] + back[2] * radius * k];
        let vel = [rx - 0.1, ry, rz - 0.1].map(|j| mul_ee(j, REC_FRAME_RATE));
        self.puffs.push(Gas {
            pos,
            vel,
            mesh: SMOKE_QUAD,
            scale: 0.05, // [M 0x13251c-0x132524 the scale arg = 0x3d4ccccd]
            scale_rate: (0.25 - 0.05) / 128.0, // [M 0x13252c-0x132530 f13 = 0x3e800000 = 0.25]
            rgba: [0x48 as f32, 0x40 as f32, 0x30 as f32, 0xe4 as f32], // [M 0x132518/0x13254c t2 = 0x484030e4]
            rgba_rate: [-0x48 as f32 / 128.0, -0x40 as f32 / 128.0, -0x30 as f32 / 128.0, -0xe4 as f32 / 128.0],
            spin,
            life: 128.0, // [M stack arg 0x80]
            kind: 7,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
    }

    /// The bullet trail's BIG puff — `Bullet_trail_effect_smoke` 0x12cba8's `+0xf6 != 0` arm
    /// (decomp :29-53): the STUCK trail bullet (the hat is the only 0x20000 carrier that sets
    /// obj+0xf6 = 1 at its stick [M Bullet_DoTrails 0x12d890:397-403]), every 15th game tick
    /// (`(GameState + (u8)obj[0xec]) % 15 == 0` [M disasm 0x12cc7c-0x12cca0 divu/mfhi] — note NO
    /// MPSettings gate on this arm). Fixed args [M disasm 0x12cd30-0x12cd80]: scale 0.2 → 5.0
    /// (the scale arg = 0x3e4ccccd, f13 = 0x40a00000), cel = the bullet's (obj+0x20), pos = obj+0x30, VEL
    /// = the given vector (no wind draws [M 0x1ab26c-0x1ab28c]), glist 0x0200001f, kind 0xb,
    /// rgba 0x404040ff → END 0x404040ff (t3, NO fade), life 255 (stack `sw $v0(0xff), ($sp)`
    /// [M 0x12cd34-0x12cd6c]).
    /// Draws per puff, in order [M disasm 0x12cca8-0x12cd64]: 1-3. the velocity, each
    /// `Rand_FRand_MVar2(a, b) = a·U(0,1) − b` [M 0x1e38b0] then offset — x = MVar2(0.3, 0.15)
    /// + 0.6 ∈ [0.45, 0.75) (0x12ccc4 + 0x3f19999a), y = MVar2(0.1, 0.05) + 0.5 ∈ [0.45, 0.55)
    /// (0x12cd08, 0x3dcccccd/0x3d4ccccd), z = MVar2(0.3, 0.15) + 0.3 (0x12cce4); 4. the caller's
    /// `Rand_FRand(2π)` (0x12cd28 — the ROT vector z); 5. the spin `Rand_FRand(π/2·REC)` — kind
    /// 0xb SCALES it ×0.25 [M 0x1aacc8:72-88 `mul.s f0, f1, 0.25` on the kind==0xb arm];
    /// 6. the sign `Rand_Rand(0x100)`. The kind-0xb update moves the puff by the stored vel plus
    /// the shared buoyancy and caps the spin rate [M 0x1ab2d0 case 0xb].
    pub fn init_trail_big_puff(&mut self, rng: &mut crate::drone::Rng, pos: V3) {
        let vx = rng.frand(0.3) - 0.15 + 0.6; // MVar2(0.3, 0.15) + 0.6 [M 0x12ccc4+0x12cce0]
        let vz = rng.frand(0.3) - 0.15 + 0.3; // MVar2(0.3, 0.15) + 0.3 [M 0x12cce4, stored to sp+0x18 = z]
        let vy = rng.frand(0.1) - 0.05 + 0.5; // MVar2(0.1, 0.05) + 0.5 [M 0x12cd08, stored to sp+0x14 = y]
        let _rot_z = rng.frand(6.2831853); // [M 0x12cd28]
        let spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE) * 0.25; // kind 0xb ×0.25 [M 0x1ab084]
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        if self.puffs.len() >= Self::CAP { return; }
        self.puffs.push(Gas {
            pos,
            vel: [vx, vy, vz].map(|j| mul_ee(j, REC_FRAME_RATE)), // the given-vel store [M 0x1ab26c-0x1ab28c — the VU0 multiply]
            mesh: 0x0200_001f, // [M 0x12cd38 t0]
            scale: 0.2, // [M the scale arg = 0x3e4ccccd]
            scale_rate: (5.0 - 0.2) / 255.0, // [M f13 = 0x40a00000; life 255]
            rgba: [0x40 as f32, 0x40 as f32, 0x40 as f32, 0xff as f32],
            rgba_rate: [0.0; 4], // END 0x404040ff — no fade [M t3 = 0x404040ff]
            spin,
            life: 255.0, // [M stack arg 0xff]
            kind: 0xb,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
    }

    /// The projectile smoke-trail site (`Bullet_trail_effect_smoke` 0x12cba8 fast arm, called from
    /// `Bullet_DoTrails` 0x12d890 when the weapon record's `+0x6C` carries bit 0x2 [M decomp
    /// :144-146]), every 4th game tick (`(GameState tick + obj[0xec]) & 3 == 0` [M disasm
    /// 0x12cbf4-0x12cc08 — obj+0xec is the creation tick, so the sum is the current tick]) and
    /// behind the `MPSettings` trail-detail gate (word 384, 0 = trails on; .bss zero at boot [L
    /// MPSettings 0x2a47a0+384, `LS_LoadMPSettings` never sets it on this build]).
    /// Fixed args [M disasm 0x12cc10-0x12cc74]: scale 0.1 → 0.5, mesh 0x0200001f, kind 7,
    /// rgba 0x404040ff → 0x40404001, life 120 ticks (the decomp-dropped 11th arg = 0x78 [M
    /// 0x12cc60 `sw $v0, ($sp)` with v0 = 0x78]), velocity (0, 0, 0) — the game passes a non-NULL
    /// zero vector so `Gas_Init`'s given-velocity branch stores it with NO wind draws [M 0x1aacc8
    /// :106-111]. Draws [M]: the caller's `Rand_FRand(6.25)` (0x12cc18, 0x40c90fdb) + the 2 spin
    /// draws inside `Gas_Init` (0x1aacc8:72-80) = 3 per puff, consumed before the pool-cap check
    /// (the searchlight precedent). The kind-7 update is pure buoyancy — the puff hangs and fades.
    /// Records with bit 0x2 [L ee_start_05.bin @ 0x2bf150 sweep]: 0x1b (the AR's GL), 0x36 (the
    /// stun grenade), 0x69 — all gravity bullets (+0x6C & 8), so the port's Grenade path.
    pub fn init_trail_smoke(&mut self, rng: &mut crate::drone::Rng, pos: V3) {
        let yaw = rng.frand(6.25);
        let spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        if self.puffs.len() >= Self::CAP { return; }
        self.puffs.push(Gas {
            pos,
            vel: [0.0; 3],
            mesh: 0x0200_001f,
            scale: 0.1,
            scale_rate: (0.5 - 0.1) / 120.0,
            rgba: [0x40 as f32, 0x40 as f32, 0x40 as f32, 0xff as f32],
            rgba_rate: [0.0, 0.0, 0.0, (1.0 - 255.0) / 120.0],
            spin,
            life: 120.0,
            kind: 7,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
        let _ = yaw; // the rotation draw: stored to obj+0x50/0x60; the port's billboard reads only the spin
    }

    /// The rocket-exhaust site (`Bullet_DoTrails` 0x12d890 bit 0x20 arm, decomp :152-166), every 3rd
    /// game tick (`% 3 == 0` [M disasm 0x12d9f4-0x12da14]) behind the same MPSettings gate.
    /// Fixed args [M disasm 0x12da34-0x12da84]: scale 0.1 → 0.4, mesh SMOKE_QUAD 0x02000039, kind 0
    /// (the damped-velocity drift), rgba 0x7f7f7f7f → 0x80808000, life 64 ticks (v0 = 0x40 at
    /// 0x12da38), velocity (0, 0, `Rand_FRand(6.25)`) — the caller's draw is stored as the z of the
    /// velocity vector [M 0x12da2c jal Rand_FRand; 0x12da28/0x12da30 zero stores; 0x12da6c
    /// `swc1 $f0, 0x28($sp)`], so kind 0's `vel · REC_FRAME_RATE` gives a ≤0.104/tick drift.
    /// Draws: the caller's 1 + the 2 spin draws = 3 per puff. Records with bit 0x20 [L sweep]:
    /// 0x2a/0x2b (the Militek MGL twins).
    pub fn init_exhaust_puff(&mut self, rng: &mut crate::drone::Rng, pos: V3) {
        let vz = rng.frand(6.25);
        let spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        if self.puffs.len() >= Self::CAP { return; }
        self.puffs.push(Gas {
            pos,
            vel: [0.0, 0.0, vz * REC_FRAME_RATE],
            mesh: SMOKE_QUAD,
            scale: 0.1,
            scale_rate: (0.4 - 0.1) / 64.0,
            rgba: [0x7f as f32, 0x7f as f32, 0x7f as f32, 0x7f as f32],
            rgba_rate: [
                (0x80 as f32 - 0x7f as f32) / 64.0,
                (0x80 as f32 - 0x7f as f32) / 64.0,
                (0x80 as f32 - 0x7f as f32) / 64.0,
                (0x00 as f32 - 0x7f as f32) / 64.0,
            ],
            spin,
            life: 64.0,
            kind: 0,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
    }

    /// The copter's damage smoke (`Copter_Update` 0x17dbb8:321-353): past half hp a 1-in-10 roll
    /// spawns `Gas_Init` at the slot-3 emitter's obj+0x80. The caller has already consumed the
    /// walk's own draws (`Rand_Rand(10)`, `Rand_FRand(π)` 0x17e320, `Rand_FRand(2.0)` 0x17e338);
    /// this site consumes `Gas_Init`'s two kind-agnostic internal draws [M 0x1aacc8:72-80 — the
    /// spin `Rand_FRand(π/2 · REC_FRAME_RATE)` and the sign `Rand_Rand(0x100)`; kind 3 takes the
    /// default (0, REC, 0) velocity branch, no wind draws [M :89-141's else]]. Mesh 0x200001f and
    /// kind 3 are the call's own [M 0x17dbb8:347-349]; the colour word packs
    /// `amount = (1 − damage/hp)·127` as `amount<<24 | amount<<16 | amount<<8 | (0xbf−amount)`
    /// [M :339-350] — the port reads its bytes as [a, b, g, r] like `init_ex`'s packed end does.
    /// The decomp drops `Gas_Init`'s life argument here (the 11th, as at 0x1cbce0); the port
    /// keeps 64.0 — `// [S] unverified`, presentation only, no pass/fail assertion reads it.
    pub fn init_copter_smoke(&mut self, rng: &mut crate::drone::Rng, pos: V3, yaw: f32, size: f32, amount: u8) {
        let spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        let amount = amount as f32;
        let rgba = [amount, amount, amount, 191.0 - amount]; // 0xbf = 191
        if self.puffs.len() >= Self::CAP { return; }
        self.puffs.push(Gas {
            pos,
            vel: [0.0, REC_FRAME_RATE, 0.0], // [M 0x1aacc8:89-141's else]
            mesh: 0x0200_001f, // [M 0x17dbb8:348]
            scale: 2.0, // [M :347's f12 = 2.0]
            scale_rate: (size - 2.0) / 64.0,
            rgba,
            rgba_rate: [0.0; 4],
            spin,
            life: 64.0, // [S] unverified — the decompile drops the life arg
            kind: 3, // [M :349]
            billboard: false,
            xray_only: false,
            hidden: false,
            born: 0,
            child: None,
            rot: [0.0; 3],
        });
        let _ = yaw; // the rot-z euler (the searchlight's shape); the port's billboard reads only the spin
    }

    /// `Gas_Init` 0x1aacc8 — the game's own draws consumed in the game's order, for every site the
    /// game reaches through this function: the spin `Rand_FRand(π/2·REC_FRAME_RATE)` [M 0x1ab060;
    /// decompile 0x1aacc8:72-73], quartered for kind 0xb [M 0x1aacc8:74-76], the sign
    /// `Rand_Rand(0x100)` (negate when the draw is ≥ 0x80) [M 0x1ab08c; decompile :77-80], then the
    /// velocity branch — kind 4 with a GIVEN velocity: `vel·REC + Rand_FRand(0.05)` on x and z, no
    /// y draw [M 0x1ab21c/0x1ab248; decompile :91-98 — the draw is ADDED raw, not centred]; kind 7
    /// with a NULL velocity (every kind-7 caller here passes NULL — the muzzle 0x1a4428, the
    /// Effect_Bullet entries 0x1cea80): `Env_GetWindVel` (0 — no wind radiators in the port's rooms
    /// [S]) + the `(rand−0.1, rand, rand−0.1)` spread over 0.2 [M 0x1ab160/0x1ab184/0x1ab1a4;
    /// decompile :108-116]; every other kind: the (0, REC, 0) default, no draws [M decompile
    /// :105-129]. The draws land BEFORE the pool-cap check — the game's RNG has no notion of the
    /// port's cap, and a skipped draw would shift the shared stream. The life gate stays first:
    /// the game draws nothing when the 11th arg (the life ushort) is 0 [M 0x1aacc8:31].
    #[allow(clippy::too_many_arguments)]
    pub fn init_ex(&mut self, rng: &mut crate::drone::Rng, mesh: u32, pos: V3, yaw: f32, vel: V3, p1: f32, p2: f32, kind: u8, rgba: [u8; 4], rgba_end: u32, life: f32) {
        if life <= 0.0 { return; }
        // Draw 1: the spin [M 0x1ab060], quartered for kind 0xb [M 0x1aacc8:74-76].
        let mut spin_mag = rng.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        if kind == 0xb { spin_mag *= 0.25; }
        // Draw 2: the sign coin [M 0x1ab08c; decompile :77-80 — `0x7f < draw` negates].
        let spin = if rng.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        // Draws 3+: the velocity branch [M 0x1aacc8:89-141].
        let vel = match kind {
            4 => [
                mul_ee(vel[0], REC_FRAME_RATE) + rng.frand(0.05),
                mul_ee(vel[1], REC_FRAME_RATE),
                mul_ee(vel[2], REC_FRAME_RATE) + rng.frand(0.05),
            ],
            7 => {
                let (rx, ry, rz) = (rng.frand(0.2), rng.frand(0.2), rng.frand(0.2));
                [mul_ee(rx - 0.1, REC_FRAME_RATE), mul_ee(ry, REC_FRAME_RATE), mul_ee(rz - 0.1, REC_FRAME_RATE)]
            }
            // fx-gas-and-blasts row 5: kinds 9/10 are the HIDDEN cloud emitters — the game's
            // LAB_001ab1e4 vel store re-uses the caller's ROT vector as the velocity [M 0x1aacc8
            // `puVar9 = param_4 … *pfVar1 = *puVar9 …`], but the emitters never move (their update
            // cases have no motion), so the port parks them at (0,0,0).
            9 | 10 => [0.0; 3],
            _ => [0.0, REC_FRAME_RATE, 0.0],
        };
        if self.puffs.len() >= Self::CAP { return; }
        let _ = yaw;
        // fx-gas-and-blasts row 5: kind 10 halves the life at creation [M 0x1aacc8's kind-10 arm
        // `pfVar1[0xe] = pfVar1[0xe] * 0.5` — +0x38 IS the life] and both emitters take the hidden
        // bit + the child-spawn record (Gas_Init's own args, stored at the data table [M :46-52]).
        let life = if kind == 10 { life * 0.5 } else { life };
        let child = matches!(kind, 9 | 10)
            .then(|| ChildSeed { p1, p2, mesh, rgba, rgba_end, life });
        // The fade target: the entry's packed rgba end (the game's param 10 [M 0x1aacc8:56-71]).
        let end = [(rgba_end >> 24) as u8, (rgba_end >> 16) as u8, (rgba_end >> 8) as u8, rgba_end as u8];
        self.puffs.push(Gas {
            pos, vel,
            mesh,
            scale: p1,
            scale_rate: (p2 - p1) / life,
            rgba: rgba.map(f32::from),
            rgba_rate: [
                (f32::from(end[0]) - f32::from(rgba[0])) / life,
                (f32::from(end[1]) - f32::from(rgba[1])) / life,
                (f32::from(end[2]) - f32::from(rgba[2])) / life,
                (f32::from(end[3]) - f32::from(rgba[3])) / life,
            ],
            spin,
            life,
            kind,
            billboard: false,
            xray_only: false,
            hidden: matches!(kind, 9 | 10), // [M LAB_001ab1e4: `obj+0xf0 |= 0x10`]
            born: self.frame,
            child,
            rot: [0.0; 3],
        });
        // The mesh rides alongside for the draw (the game passes it to the gfx hashtable); the port's
        // billboard draw reads it from the puff via the pool index.
        if let Some(g) = self.puffs.last_mut() { g.mesh = mesh; }
    }

    /// `init_ex` with the spawn rotation (`Gas_Init`'s param-5 euler → the create-object quaternion
    /// [M 0x1aacc8:31]) — the impact puffs' orientation: `Effect_Bullet`'s wall puffs spawn with
    /// `Vec_NormToRot(normal)` [M 0x1ce8a8:129], floor puffs with the negated-normal euler or zero
    /// by the entry's +0x1b byte [M :122-127]. Everything else is `init_ex` exactly.
    #[allow(clippy::too_many_arguments)]
    pub fn init_ex_rot(&mut self, rng: &mut crate::drone::Rng, mesh: u32, pos: V3, yaw: f32, vel: V3, p1: f32, p2: f32, kind: u8, rgba: [u8; 4], rgba_end: u32, life: f32, rot: [f32; 3]) {
        self.init_ex(rng, mesh, pos, yaw, vel, p1, p2, kind, rgba, rgba_end, life);
        if let Some(g) = self.puffs.last_mut() { g.rot = rot; }
    }

    /// `gases_smoke` 0x1ab2d0 for the ported types (0, 4, 7, 9, 10): motion, growth, fade, the life
    /// countdown and the kill; the kind-9/10 hidden emitters spawn their children (fx-gas-and-blasts
    /// row 5). 0xb is ported on s/fx-muzzle-and-trail; no game caller in the port's scope creates
    /// kind-9/10 parents yet (the audit's census) — the mechanics land here for the callers that do.
    pub fn update(&mut self, frame: u32) {
        self.frame = frame; // obj+0xec-equivalent stamp for puffs created later this tick
        let mut children: Vec<(ChildSeed, V3, V3, u8, u8)> = Vec::new();
        self.puffs.retain_mut(|g| {
            match g.kind {
                // Case 0 [M 0x1ab2d0 case 0]: vel damped ×0.95, gravity scaled −0.02, pos += vel.
                0 => {
                    for k in 0..3 { g.vel[k] *= 0.95; }
                    for k in 0..3 { g.pos[k] += g.vel[k] + WLD_GRAVITY[k] * -0.02 * REC_FRAME_RATE; }
                }
                // Case 4 [M 0x1ab2d0 case 4]: pos += vel + gravity·0.01, vel damped.
                4 => {
                    for k in 0..3 { g.pos[k] += g.vel[k] + WLD_GRAVITY[k] * 0.01 * REC_FRAME_RATE; }
                    for k in 0..3 { g.vel[k] *= 0.95 - REC_FRAME_RATE; }
                }
                // Case 7 [M 0x1ab2d0 case 7 = the case-0 fallthrough body :47-65]: pos += vel +
                // WldGravity·(REC_FRAME_RATE·−0.02·FRAME_RATE_MUL) — the puff KEEPS its own wind
                // velocity and the negative gravity factor is an upward buoyancy (y +=
                // 9.8·0.02/60 ≈ +0.0033/tick). rocket-trail-gas: the old arm sank the puff at
                // gravity·0.05 and dropped vel entirely.
                7 => {
                    for k in 0..3 { g.pos[k] += g.vel[k]; }
                    g.pos[1] += WLD_GRAVITY[1] * -0.02 * REC_FRAME_RATE;
                }
                // fx-gas-and-blasts row 5 [M 0x1ab2d0 case 9 = :146-162]: the hidden cloud emitter
                // — no motion; after the 20-tick creation warmup (`GameState - obj[0xec] <
                // FRAME_RATE_MUL·20.0` blocks) it spawns a kind-8 child on every tick where
                // `GameState & 3 == 3`, with the parent's own creation args (the child record) and
                // the parent's rgb + LIVE alpha byte. The kind-8 NULL-vel store is (0, REC, 0)
                // [M 0x1aacc8's kind-8 arm].
                9 => {
                    if frame.wrapping_sub(g.born) >= 20 && frame & 3 == 3 {
                        if let Some(s) = g.child.clone() {
                            children.push((s, g.pos, [0.0, REC_FRAME_RATE, 0.0], 8, g.alpha_byte()));
                        }
                    }
                }
                // [M 0x1ab2d0 case 10 = :164-186]: the emitter — every tick where
                // `GameState & 7 == 7` it spawns a kind-7 child with the GIVEN velocity
                // (0, 2·REC, 0) through the generic given-vel store (·REC again [M 0x1aacc8:89-92]).
                10 => {
                    if frame & 7 == 7 {
                        if let Some(s) = g.child.clone() {
                            children.push((s, g.pos, [0.0, mul_ee(2.0 * REC_FRAME_RATE, REC_FRAME_RATE), 0.0], 7, g.alpha_byte()));
                        }
                    }
                }
                // Case 0xb [M 0x1ab2d0 case 0xb = 0x1ab44c, decomp :186-211]: the same motion —
                // pos += WldGravity·(REC·−0.02·FRM) then pos += the stored velocity — plus the
                // rot-z spin accumulation (`obj+0x58 += data+0`, the spin rate Gas_Init stored)
                // and the scale/colour ticks (the common tail). The game also clamps pos.y to
                // the host cel's +0x54 floor [M :201-203]; the port's puffs carry no cel host
                // and this puff only rises (vel y > 0.45·REC, buoyancy up), so the clamp is a
                // no-op for it [S].
                0xb => {
                    for k in 0..3 { g.pos[k] += g.vel[k]; }
                    g.pos[1] += WLD_GRAVITY[1] * -0.02 * REC_FRAME_RATE;
                }
                _ => {}
            }
            g.scale += g.scale_rate;
            for k in 0..4 { g.rgba[k] = (g.rgba[k] + g.rgba_rate[k]).clamp(0.0, 255.0); }
            g.life -= 1.0;
            g.life > 0.0
        });
        for (s, pos, vel, kind, alpha) in children {
            self.push_child(&s, pos, vel, kind, alpha);
        }
    }

    /// A kind-8/7 emitter child: the parent's Gas_Init re-call [M 0x1ab2d0 case 9/case 10] —
    /// the record's p1/p2/mesh/rgba-end/life, the rgb re-combined with the parent's live alpha
    /// byte, the spin+sign head (the counter-hash stand-in), no hidden bit, no child record.
    fn push_child(&mut self, s: &ChildSeed, pos: V3, vel: V3, kind: u8, alpha: u8) {
        if self.puffs.len() >= Self::CAP { return; }
        let n = self.counter.wrapping_add(0x9e37_79b9);
        self.counter = n;
        let spin = jitter(n, 1) * (std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE)
            * if jitter(n, 2) > 0.5 { 1.0 } else { -1.0 };
        let rgba = [s.rgba[0], s.rgba[1], s.rgba[2], alpha];
        let end = [(s.rgba_end >> 24) as u8, (s.rgba_end >> 16) as u8, (s.rgba_end >> 8) as u8, s.rgba_end as u8];
        self.puffs.push(Gas {
            pos, vel,
            mesh: s.mesh,
            scale: s.p1,
            scale_rate: (s.p2 - s.p1) / s.life,
            rgba: rgba.map(f32::from),
            rgba_rate: [
                (f32::from(end[0]) - f32::from(rgba[0])) / s.life,
                (f32::from(end[1]) - f32::from(rgba[1])) / s.life,
                (f32::from(end[2]) - f32::from(rgba[2])) / s.life,
                (f32::from(end[3]) - f32::from(rgba[3])) / s.life,
            ],
            spin,
            life: s.life,
            kind,
            billboard: false,
            xray_only: false,
            hidden: false,
            born: self.frame,
            child: None,
            rot: [0.0; 3],
        });
    }
}

/// The EE's `mul.s`: the port's ps2float::mul (the game's velocity scale runs on the EE FPU).
fn mul_ee(a: f32, b: f32) -> f32 { nf_formats::ps2float::mul(a, b) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_sight_quarters_the_alpha_byte() {
        // [M Gas_Init 0x1aacc8:83 |= 0x20020120; View_SetupRenderModes_Tweak 0x1e5888:12-17 >> 2 in mode 1;
        // gases_smoke 0x1ab2d0 tail: (int) clamp 0..255]
        assert_ne!(GAS_OBJ_FLAGS & NIGHT_QUARTER_ALPHA, 0);
        let mut rng = crate::drone::Rng::new(0x21d);
        let mut pool = GasPool::default();
        pool.init(&mut rng, [0.0; 3], 0.0, [0.0; 3], 0.5, 0.5, 0, [0xff, 0xff, 0xff, 0xc8], 60.0);
        let g = &mut pool.puffs[0];
        g.rgba[3] = 199.9;
        assert_eq!(g.alpha_byte(), 199);
        assert_eq!(g.tweak_alpha(1), (199u8 >> 2) as f32 / 255.0);
        // Sister (can fail): off and x-ray keep the live alpha.
        assert_eq!(g.tweak_alpha(0), 199.9 / 255.0);
        assert_eq!(g.tweak_alpha(2), 199.9 / 255.0);
        g.rgba[3] = 300.0;
        assert_eq!(g.alpha_byte(), 255);
        g.rgba[3] = -4.0;
        assert_eq!(g.alpha_byte(), 0);
    }

    #[test]
    fn searchlight_smoke_consumes_six_shared_draws_in_order() {
        // [M 0x1cbca0, 0x1ab060, 0x1ab08c, 0x1ab160, 0x1ab184, 0x1ab1a4] — the puff's fields must equal
        // the six draws replayed on a twin stream, bit for bit.
        let mut rng = crate::drone::Rng::new(0x5e4c04);
        let mut pool = GasPool::default();
        pool.init_searchlight_smoke(&mut rng, [1.0, 2.0, 3.0]);
        assert_eq!(pool.puffs.len(), 1);
        let g = &pool.puffs[0];
        assert_eq!(g.life, 128.0, "the decomp-dropped 11th arg = 0x80 [M 0x1cbce0]");
        assert_eq!(g.kind, 7);
        assert_eq!(g.mesh, SMOKE_QUAD);
        assert_eq!(g.scale, 0.05);
        assert_eq!(g.scale_rate, (0.25 - 0.05) / 128.0);
        assert_eq!(g.rgba, [0x48 as f32, 0x40 as f32, 0x30 as f32, 0x80 as f32]);
        // vel = (wind + (r−0.1), wind + r, wind + (r−0.1)) · REC with wind 0 — three draws 4..6,
        // replayed in order after the spin draws.
        let mut r = crate::drone::Rng::new(0x5e4c04);
        let _rot_z = r.frand(std::f32::consts::PI); // draw 1, the caller's euler
        let spin = r.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let sign = r.rand(0x100);
        let expect_spin = if sign >= 0x80 { -spin } else { spin };
        let (rx, ry, rz) = (r.frand(0.2), r.frand(0.2), r.frand(0.2));
        assert_eq!(g.spin.to_bits(), expect_spin.to_bits(), "the spin = draw 2 with the draw-3 sign");
        assert_eq!(g.vel[0].to_bits(), mul_ee(rx - 0.1, REC_FRAME_RATE).to_bits());
        assert_eq!(g.vel[1].to_bits(), mul_ee(ry, REC_FRAME_RATE).to_bits());
        assert_eq!(g.vel[2].to_bits(), mul_ee(rz - 0.1, REC_FRAME_RATE).to_bits());
        // The whole stream advanced by exactly 6 draws: the twin consumed the same six in the same
        // order, so a seventh replay continues both streams identically.
        let mut a = crate::drone::Rng::new(0x5e4c04);
        let mut pool2 = GasPool::default();
        pool2.init_searchlight_smoke(&mut a, [0.0; 3]);
        let mut b = crate::drone::Rng::new(0x5e4c04);
        for _ in 0..6 { b.frand(1.0); }
        let a7 = a.frand(1.0).to_bits();
        let b7 = b.frand(1.0).to_bits();
        assert_eq!(a7, b7, "the pool consumed exactly the game's 6 draws");
        assert_eq!(pool.puffs[0].pos, [1.0, 2.0, 3.0], "the puff sits at the head");
    }

    #[test]
    fn a_puff_grows_fades_and_dies() {
        let mut rng = crate::drone::Rng::new(0x21d);
        let mut pool = GasPool::default();
        pool.init(&mut rng, [1.0, 2.0, 3.0], 0.0, [0.0; 3], 0.05, 0.10, 7, [0x48, 0x40, 0x30, 0x80], 8.0);
        assert_eq!(pool.puffs.len(), 1);
        let g0 = pool.puffs[0].clone();
        pool.update(0);
        let g1 = &pool.puffs[0];
        assert!(g1.scale > g0.scale, "the puff grows");
        assert!(g1.rgba[3] < g0.rgba[3], "the puff fades");
        let mut n = 0;
        while !pool.puffs.is_empty() { pool.update(0); n += 1; assert!(n < 20, "the puff outlived its life"); }
        assert_eq!(n, 7, "life 8 = 8 ticks (1 spent before the count)");
    }

    #[test]
    fn the_pool_is_deterministic_and_capped() {
        let mut ra = crate::drone::Rng::new(0x21d);
        let mut rb = crate::drone::Rng::new(0x21d);
        let mut a = GasPool::default();
        let mut b = GasPool::default();
        for i in 0..64 { a.init(&mut ra, [i as f32, 0.0, 0.0], 0.0, [0.0; 3], 0.05, 0.1, 7, [72, 64, 48, 128], 4.0); }
        for i in 0..64 { b.init(&mut rb, [i as f32, 0.0, 0.0], 0.0, [0.0; 3], 0.05, 0.1, 7, [72, 64, 48, 128], 4.0); }
        assert_eq!(a.puffs, b.puffs, "same calls, same draws, same puffs (no wall-clock)");
        for _ in 0..200 { a.init(&mut ra, [0.0; 3], 0.0, [0.0; 3], 0.05, 0.1, 7, [72, 64, 48, 128], 4.0); }
        assert_eq!(a.puffs.len(), GasPool::CAP, "the pool caps");
    }

    #[test]
    fn init_ex_consumes_the_games_draws_in_order() {
        // [M 0x1ab060 spin, 0x1ab08c sign, 0x1ab160/0x1ab184/0x1ab1a4 kind-7 wind, 0x1ab21c/0x1ab248
        // kind-4 jitter; the kind-0xb quarter [M 0x1aacc8:74-76]] — each call's fields must equal the
        // draws replayed on a twin stream, bit for bit, and the stream must advance by exactly the
        // game's count (2 kind 0/1/0xb, 5 kind 7, 4 kind 4).
        // kind 0: spin + sign only, default (0, REC, 0) velocity.
        let mut rng = crate::drone::Rng::new(0x5e4c04);
        let mut pool = GasPool::default();
        pool.init_ex(&mut rng, SMOKE_QUAD, [0.0; 3], 0.0, [9.0, 9.0, 9.0], 0.05, 0.1, 0, [255, 255, 255, 200], 0, 40.0);
        let mut r = crate::drone::Rng::new(0x5e4c04);
        let spin = r.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let sign = r.rand(0x100);
        let g = &pool.puffs[0];
        let expect = if sign >= 0x80 { -spin } else { spin };
        assert_eq!(g.spin.to_bits(), expect.to_bits(), "kind 0: the spin draw with the sign coin");
        assert_eq!(g.vel, [0.0, REC_FRAME_RATE, 0.0], "kind 0: the default velocity, no draws");
        assert_eq!(r.draws, 2, "kind 0 consumed exactly the game's 2 draws");
        assert_eq!(rng.draws, r.draws, "the pool's stream matches the replay");
        // kind 7: spin + sign + the 3 wind draws; vel = (r−0.1, r, r−0.1)·REC [M 0x1aacc8:108-116].
        let mut pool7 = GasPool::default();
        pool7.init_ex(&mut rng, SMOKE_QUAD, [1.0, 2.0, 3.0], 0.0, [0.0; 3], 0.05, 0.1, 7, [0x48, 0x40, 0x30, 0x80], 1, 128.0);
        let spin7 = r.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let sign7 = r.rand(0x100);
        let (rx, ry, rz) = (r.frand(0.2), r.frand(0.2), r.frand(0.2));
        let g7 = &pool7.puffs[0];
        let expect7 = if sign7 >= 0x80 { -spin7 } else { spin7 };
        assert_eq!(g7.spin.to_bits(), expect7.to_bits(), "kind 7: the spin draw with the sign coin");
        assert_eq!(g7.vel[0].to_bits(), mul_ee(rx - 0.1, REC_FRAME_RATE).to_bits());
        assert_eq!(g7.vel[1].to_bits(), mul_ee(ry, REC_FRAME_RATE).to_bits());
        assert_eq!(g7.vel[2].to_bits(), mul_ee(rz - 0.1, REC_FRAME_RATE).to_bits());
        assert_eq!(r.draws, 7, "kind 7 consumed exactly the game's 5 more draws");
        // kind 4: spin + sign + 2 jitter draws; x/z = given·REC + the RAW draw [M 0x1aacc8:91-98].
        let mut pool4 = GasPool::default();
        pool4.init_ex(&mut rng, SMOKE_QUAD, [0.0; 3], 0.0, [1.0, 2.0, 3.0], 0.3, 1.5, 4, [255, 255, 255, 255], 1, 20.0);
        let spin4 = r.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let sign4 = r.rand(0x100);
        let (jx, jz) = (r.frand(0.05), r.frand(0.05));
        let g4 = &pool4.puffs[0];
        let expect4 = if sign4 >= 0x80 { -spin4 } else { spin4 };
        assert_eq!(g4.spin.to_bits(), expect4.to_bits(), "kind 4: the spin draw with the sign coin");
        assert_eq!(g4.vel[0].to_bits(), (mul_ee(1.0, REC_FRAME_RATE) + jx).to_bits(), "the draw is ADDED raw, not centred");
        assert_eq!(g4.vel[1].to_bits(), mul_ee(2.0, REC_FRAME_RATE).to_bits(), "no y draw [M 0x1aacc8:91-98]");
        assert_eq!(g4.vel[2].to_bits(), (mul_ee(3.0, REC_FRAME_RATE) + jz).to_bits());
        assert_eq!(r.draws, 11, "kind 4 consumed exactly the game's 4 more draws");
        // kind 0xb: the quarter spin [M 0x1aacc8:74-76], sign coin, default velocity.
        let mut poolb = GasPool::default();
        poolb.init_ex(&mut rng, 0x0200_001f, [0.0; 3], 0.0, [0.0; 3], 0.5, 1.68, 0xb, [255, 255, 255, 127], 0x8080_8000, 127.0);
        let spinb = r.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE) * 0.25;
        let signb = r.rand(0x100);
        let gb = &poolb.puffs[0];
        let expectb = if signb >= 0x80 { -spinb } else { spinb };
        assert_eq!(gb.spin.to_bits(), expectb.to_bits(), "kind 0xb: the quartered spin with the sign coin");
        assert_eq!(r.draws, 13, "kind 0xb consumed exactly the game's 2 more draws");
        // The stream keeps walking identically after all four puffs.
        let a = rng.frand(1.0).to_bits();
        let b = r.frand(1.0).to_bits();
        assert_eq!(a, b, "the pool consumed exactly the game's draws, nothing else");
    }

    /// fx-muzzle-and-trail row 6: the GT shot smoke consumes SIX shared draws in a fixed order —
    /// the caller's `FRand(2π)` rot-z [M 0x180bc0], the spin `FRand(π/2·REC)` [M 0x1aacc8:72-80],
    /// the sign `Rand(0x100)` [M 0x1ab08c], then the kind-7 NULL-vel wind spread `FRand(0.2)` ×3
    /// [M 0x1ab138-0x1ab1c8: x/z centred on the wind, y = raw +FRand(0.2) — the game's own
    /// asymmetry], all before the pool-cap check. The puff: scale 0.05 → 0.1 over life 128, rgba
    /// 0x48403080 fading to ~0 [M 0x180bfc-0x180c3c].
    #[test]
    fn gt_shot_smoke_draws_six_and_lands_the_values() {
        let mut pool = GasPool::default();
        let mut rng = crate::drone::Rng::new(11);
        pool.init_gt_shot_smoke(&mut rng, [3.0, 4.0, 5.0]);
        assert_eq!(pool.puffs.len(), 1);
        let g = &pool.puffs[0];
        assert_eq!(g.pos, [3.0, 4.0, 5.0]);
        assert_eq!(g.mesh, SMOKE_QUAD);
        assert_eq!(g.kind, 7);
        assert_eq!(g.scale, 0.05);
        assert_eq!(g.life, 128.0);
        assert_eq!(g.rgba, [0x48 as f32, 0x40 as f32, 0x30 as f32, 0x80 as f32]);
        // Replay the same six draws on a twin rng — the pool must sit EXACTLY there on the stream.
        let mut r2 = crate::drone::Rng::new(11);
        let _rot_z = r2.frand(6.2831853);
        let spin_mag = r2.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE);
        let spin = if r2.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        let (rx, ry, rz) = (r2.frand(0.2), r2.frand(0.2), r2.frand(0.2));
        assert_eq!(g.spin, spin, "draws 2-3 are the spin + sign");
        assert_eq!(g.vel, [(rx - 0.1), ry, (rz - 0.1)].map(|j| mul_ee(j, REC_FRAME_RATE)), "draws 4-6 are the wind spread");
        // Sister: a seventh draw before Gas_Init (e.g. a misplaced cap check) shifts the replay.
        let mut r3 = crate::drone::Rng::new(11);
        let mut pool3 = GasPool::default();
        let _shift = r3.frand(1.0);
        pool3.init_gt_shot_smoke(&mut r3, [3.0, 4.0, 5.0]);
        assert_ne!(pool3.puffs[0].spin, pool.puffs[0].spin);
    }

    /// fx-muzzle-and-trail row 12: the trail's BIG puff consumes SIX draws — vx `MVar2(0.3, 0.15)
    /// + 0.6` [M 0x12ccc4], vz `MVar2(0.3, 0.15) + 0.3` [M 0x12cce4], vy `MVar2(0.1, 0.05) + 0.5`
    /// [M 0x12cd08], the rot-z `FRand(2π)` [M 0x12cd28], the kind-0xb spin ×0.25 [M 0x1ab084], the
    /// sign [M 0x1ab08c] — then pushes scale 0.2 → 5.0 over life 255, rgba 0x404040ff with NO fade
    /// [M 0x12cd38 t0/t3, the scale arg = 0x3e4ccccd, f13 = 0x40a00000, stack 0xff].
    #[test]
    fn trail_big_puff_draws_six_and_lands_the_values() {
        let mut pool = GasPool::default();
        let mut rng = crate::drone::Rng::new(5);
        pool.init_trail_big_puff(&mut rng, [1.0, 1.0, 1.0]);
        assert_eq!(pool.puffs.len(), 1);
        let g = &pool.puffs[0];
        assert_eq!(g.mesh, 0x0200_001f);
        assert_eq!(g.kind, 0xb);
        assert_eq!(g.scale, 0.2);
        assert_eq!(g.life, 255.0);
        assert_eq!(g.rgba, [0x40 as f32, 0x40 as f32, 0x40 as f32, 0xff as f32]);
        assert_eq!(g.rgba_rate, [0.0; 4], "END = rgba: no fade");
        let mut r2 = crate::drone::Rng::new(5);
        let vx = r2.frand(0.3) - 0.15 + 0.6;
        let vz = r2.frand(0.3) - 0.15 + 0.3;
        let vy = r2.frand(0.1) - 0.05 + 0.5;
        let _rot_z = r2.frand(6.2831853);
        let spin_mag = r2.frand(std::f32::consts::FRAC_PI_2 * REC_FRAME_RATE) * 0.25;
        let spin = if r2.rand(0x100) >= 0x80 { -spin_mag } else { spin_mag };
        assert_eq!(g.spin, spin);
        assert_eq!(g.vel, [vx, vy, vz].map(|j| mul_ee(j, REC_FRAME_RATE)));
        // The buoyant rise: the kind-0xb motion is pos += vel then the shared gravity lift.
        let l0 = g.pos;
        let mut pool2 = GasPool::default();
        pool2.puffs.push(g.clone());
        pool2.update(0);
        assert!(pool2.puffs[0].pos[1] > l0[1], "the puff rises");
    }
}

#[cfg(test)]
mod emitter_tests {
    use super::*;

    /// fx-gas-and-blasts row 5 [M Gas_Init 0x1aacc8's kind-9/10 arms + gases_smoke 0x1ab2d0
    /// case 9 = :146-162 / case 10 = :164-186]: the hidden cloud emitters.
    #[test]
    fn kind9_emitter_warms_up_then_spawns_kind8_children_on_the_3_gate() {
        let mut pool = GasPool::default();
        let mut rng = crate::drone::Rng::new(0x5e4c04);
        pool.update(100); // the pool's frame stamp (GameState during tick 100)
        pool.init_ex(&mut rng, SMOKE_QUAD, [1.0, 2.0, 3.0], 0.0, [0.0; 3], 0.5, 1.0, 9, [0x40, 0x40, 0x40, 0xff], 0x40404001, 600.0);
        let p = &pool.puffs[0];
        assert!(p.hidden, "the kind-9 parent takes the +0x10 hidden bit [M LAB_001ab1e4]");
        assert_eq!(p.child.as_ref().unwrap().life, 600.0, "kind 9 keeps its life");
        // The 20-tick warmup: no child even on a &3 == 3 tick inside it.
        pool.update(101); // 101 - 100 = 1 < 20, and 101 & 3 == 1 — nothing
        pool.update(103); // & 3 == 3 but the warmup holds (103 - 100 = 3 < 20)
        assert!(pool.puffs.iter().all(|g| g.kind != 8), "the warmup blocks the children");
        // After the warmup: children only on the & 3 == 3 ticks. The parent's own rgba fades
        // each tick, so the child's alpha byte is the parent's LIVE one at the spawn tick.
        let parent_alpha = pool.puffs[0].alpha_byte();
        pool.update(123); // 123 - 100 = 23 ≥ 20, 123 & 3 == 3 → a child
        assert_eq!(pool.puffs.iter().filter(|g| g.kind == 8).count(), 1, "one child on the &3==3 tick");
        // The child carries the parent's record with the parent's LIVE alpha byte.
        let c = pool.puffs.iter().find(|g| g.kind == 8).unwrap();
        assert_eq!(c.pos, [1.0, 2.0, 3.0], "the child spawns at the parent");
        assert_eq!(c.mesh, SMOKE_QUAD);
        assert!((c.vel[1] - REC_FRAME_RATE).abs() < 1e-6, "the kind-8 NULL-vel store is (0, REC, 0) [M 0x1aacc8]");
        assert_eq!(c.rgba[3], f32::from(parent_alpha), "the parent's live alpha byte rides the child's rgba");
        assert!(!c.hidden && c.child.is_none(), "children are visible leaves");
        pool.update(124); // 124 & 3 == 0 → none
        assert_eq!(pool.puffs.iter().filter(|g| g.kind == 8).count(), 1);
        pool.update(127); // 127 & 3 == 3 → another
        assert_eq!(pool.puffs.iter().filter(|g| g.kind == 8).count(), 2);
        // Sister: a &3 == 3 tick BEFORE the warmup ends spawns nothing (covered above).
    }

    #[test]
    fn kind10_emitter_halves_its_life_and_spawns_kind7_children_on_the_7_gate() {
        let mut pool = GasPool::default();
        let mut rng = crate::drone::Rng::new(0x5e4c04);
        pool.update(0);
        pool.init_ex(&mut rng, SMOKE_QUAD, [0.0; 3], 0.0, [0.0; 3], 0.4, 2.0, 10, [0x80, 0x80, 0x80, 0xc8], 0x80808064, 100.0);
        let p = &pool.puffs[0];
        assert!(p.hidden);
        assert!((p.life - 50.0).abs() < 1e-4, "kind 10 halves the life at creation [M 0x1aacc8 `pfVar1[0xe] *= 0.5`]");
        // Children on the & 7 == 7 ticks only.
        pool.update(6); // & 7 == 6 → none
        assert!(pool.puffs.iter().all(|g| g.kind != 7));
        let parent_alpha = pool.puffs[0].alpha_byte();
        pool.update(7); // & 7 == 7 → a child
        let c = pool.puffs.iter().find(|g| g.kind == 7).unwrap();
        assert_eq!(c.pos, [0.0; 3]);
        assert!((c.vel[1] - mul_ee(2.0 * REC_FRAME_RATE, REC_FRAME_RATE)).abs() < 1e-9,
            "the given (0, 2·REC, 0) vel through the generic ·REC store [M 0x1aacc8:89-92]");
        assert_eq!(c.rgba[3], f32::from(parent_alpha), "the parent's live alpha byte");
        assert!((c.life - 50.0).abs() < 1e-4, "the child's life comes from the record");
        pool.update(15); // & 7 == 7 again
        assert_eq!(pool.puffs.iter().filter(|g| g.kind == 7).count(), 2);
    }
}
