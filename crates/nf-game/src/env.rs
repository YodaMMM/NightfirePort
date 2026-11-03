//! Weather: snow / rain drops (`Env_Create` 0x17b968, `InitDrops` 0x18bf68, `Env_Update` 0x17bc88 → `UpdateDrops`
//! 0x18c580, `AddDrop` 0x18c278, `RainBox_Clip` 0x18d0d0). Spec: docs/systems/render.md "Snow".
//!
//! The load-time reset — `Env_Reset` 0x17c1b0, called by `ResetMap_GameInit` 0x1d5080 [M] —
//! clears the effect globals (`memset(GfxList+0x10, 0, 0x540)`, `Vec_Zero(0x2d88a0)`,
//! `GlobalEnv = 0`), calls `InitDrops(0xff)` and `InitRainBox` 0x18d838 (`RainBoxRoot = 0`, the
//! head of the `RainBox_Create` 0x18d840 box list). The port does the same job by rebuilding its
//! weather state per level: `Drops::new` (this file) takes a fresh box list from the level's 0x32
//! placements each load — there is no persistent `RainBoxRoot` list to clear, and `GlobalEnv`'s
//! sun vector is the `Env_Create` record's own field (world_fx.rs).
//!
//! The drops use the shared `Rand_*` generator every tick (the wind, each respawn), and `Env_Update` draws one
//! `Rand_Rand(20000)` first; `Game_Run` calls it before the object handler, so `Drones::tick` runs it first.

use crate::drone::Rng;
use crate::world::V3;

/// A snow / rain volume (placement type 0x32 `Effect_Snow_*`, `RainBox_Create` 0x18d840): the glist bounds plus the
/// object position.
#[derive(Clone, Copy, Debug)]
pub struct RainBox { pub min: V3, pub max: V3 }

#[derive(Clone, Debug)]
pub struct Drops {
    /// `DropType`: 0 snow, 1 rain, 2 rising.
    pub kind: u8,
    /// `RandTable1`: 32 velocities per tick.
    pub vel: [V3; 32],
    /// `DropVector` (x, y, z, kill height w) and `DropState` (1 = falling).
    pub drops: Vec<[f32; 4]>,
    pub alive: Vec<bool>,
    /// `WindVector` (x, z).
    pub wind: [f32; 2],
    /// `DAT_003194f4`: the step's extra y, set by `InitDrops` to −0.5 for rain (kind 1) and 0.0 for
    /// snow / rising [M 0x18bf68: `DAT_003194f4 = 0xbf000000` in the `DropType == 1` arm, 0 in the
    /// else]. For rain it is the WHOLE y step: the rain branch adds the wind quad
    /// `{WindVector, DAT_003194f4, DAT_003194f8} · FRAME_RATE_MUL` uniformly to every drop and
    /// nothing else [M 0x18c580 rain branch: `_lqc2(_WindVector2)` + `_vadd` into each position —
    /// disasm 0x18c6f8 `beq DropType,1 → 0x18c7e0`; the per-slot `RandTable2` is computed but read
    /// only by the snow/rising scratchpad walk]. So rain falls at exactly 0.5/tick (30 units/s)
    /// with NO per-slot slant — a drop keeps the x/z its `AddDrop` draws gave it for its whole
    /// fall (the wind stays 0 for rain: `InitDrops` zeroes it and the kind-1 path never runs the
    /// two wind draws [M 0x18c580:139]). Snow/rising keep the table's 0.025–0.1/tick plus slant.
    pub fall: f32,
    pub boxes: Vec<RainBox>,
    /// Clipped boxes of this tick and whether each is active.
    clipped: Vec<(RainBox, bool)>,
    /// `InitBodge` (07000005–06): the first update fills the cube.
    pub init_bodge: bool,
    /// `CubeCentre`.
    pub centre: V3,
}

impl Drops {
    /// `InitDrops(kind)`: the velocity table (x, z ∈ [−0.02, 0.02], y ∈ [−0.1, −0.025]; ±0.002 on 07000007–08;
    /// ×0.25 and upward for kind 2), 4096 drops for snow (2048 otherwise, rain 128 / 64). `rng`: the generator the
    /// table comes from — the game's runs before the level-start snapshot the port starts from, so the port takes it
    /// from a separate one (*deviation*: the table's values are not the game's).
    pub fn new(kind: u8, boxes: Vec<RainBox>, level: u32, rng: &mut Rng) -> Drops {
        let narrow = matches!(level, 0x0700_0007 | 0x0700_0008);
        let mut vel = [[0f32; 3]; 32];
        for v in &mut vel {
            let (x, y, z) = if narrow {
                (rng.frand(0.004) - 0.002, -(rng.frand(0.075) + 0.025), rng.frand(0.004) - 0.002)
            } else {
                (rng.frand(0.04) - 0.02, -(rng.frand(0.075) + 0.025), rng.frand(0.04) - 0.02)
            };
            *v = if kind == 2 { [x * 0.25, y * -0.25, z * 0.25] } else { [x, y, z] };
        }
        let max = match kind { 0 => 0x1000, 1 => if matches!(level.wrapping_sub(0x0700_0009), 0..=2) { 0x80 } else { 0x40 }, _ => 0x800 };
        Drops { kind, vel, drops: vec![[0.0, 0.0, -1024.0, 0.0]; max], alive: vec![false; max], wind: [0.0; 2],
            fall: if kind == 1 { -0.5 } else { 0.0 }, boxes, clipped: Vec::new(),
            init_bodge: matches!(level, 0x0700_0005 | 0x0700_0006), centre: [0.0; 3] }
    }

    /// `UpdateDrops` for the viewer at `eye` with the player's `yaw` (the cube centre: the eye 7.5 ahead).
    /// `fill_rng`: the generator for the first-frame fill (*deviation*: the game's fill precedes the snapshot).
    pub fn update(&mut self, rng: &mut Rng, fill_rng: &mut Rng, eye: V3, yaw: f32) {
        if self.init_bodge {
            // The whole cube at once, kill height eye − 7.5.
            self.init_bodge = false;
            for i in 0..self.drops.len() {
                self.alive[i] = true;
                let x = eye[0] + (fill_rng.frand(30.0) - 15.0);
                let z = eye[2] + (fill_rng.frand(30.0) - 15.0);
                let y = eye[1] + (fill_rng.frand(30.0) - 15.0);
                self.drops[i] = [x, y, z, eye[1] - 7.5];
            }
            return;
        }
        let mut added = 0;
        if self.kind != 1 {
            self.wind[0] = (self.wind[0] + (rng.frand(0.0004) - 0.0002)).clamp(-0.01, 0.01);
            self.wind[1] = (self.wind[1] + (rng.frand(0.0004) - 0.0002)).clamp(-0.01, 0.01);
        }
        self.centre = [eye[0] + crate::trig::sinf(yaw) * 7.5, eye[1], eye[2] + crate::trig::cosf(yaw) * 7.5];
        // `RainBox_Clip`: each box clipped to the cube (± 15); active when its x and z ranges are not empty.
        let c = self.centre;
        self.clipped = self.boxes.iter().map(|b| {
            let lo = [b.min[0].max(c[0] - 15.0), b.min[1].max(c[1] - 15.0), b.min[2].max(c[2] - 15.0)];
            let hi = [b.max[0].min(c[0] + 15.0), b.max[1].min(c[1] + 15.0), b.max[2].min(c[2] + 15.0)];
            (RainBox { min: lo, max: hi }, hi[0] > lo[0] && hi[2] > lo[2])
        }).collect();
        if !self.clipped.iter().any(|b| b.1) { return; }
        // Rain steps by the wind quad ALONE — the same vector for every drop [M 0x18c580 rain
        // branch: `auVar21 = _lqc2(_WindVector2); auVar20 = _vadd(auVar21, auVar20)` per drop; the
        // per-slot `RandTable2` computed just above is read ONLY by the snow/rising branch's
        // scratchpad walk `((uint)lVar2 & 0x1f) * 0x10 + iStack_f8` (iStack_f8 = RandTable2
        // 0x3192f0)]. Snow/rising: `RandTable2[i] = RandTable1[i]·FRM + quad·FRM` [M 0x18c580:185-202].
        let step: Vec<V3> = self.vel.iter().map(|v| [v[0] + self.wind[0], v[1] + self.fall, v[2] + self.wind[1]]).collect();
        let rain_step = [self.wind[0], self.fall, self.wind[1]];
        for i in 0..self.drops.len() {
            if self.alive[i] {
                let s = if self.kind == 1 { rain_step } else { step[i & 31] };
                let d = &mut self.drops[i];
                for k in 0..3 { d[k] += s[k]; }
                let dead = if self.kind == 2 { d[3] < d[1] } else { d[1] < d[3] };
                if dead { self.alive[i] = false; d[1] -= 1024.0; }
            } else if added < 0x15 {
                added += 1;
                self.add(rng, i);
            }
        }
    }

    /// `AddDrop`: x, z within the cube, y above the centre (7.5 + rand 7.5; rising drops anywhere in ±15); accepted only
    /// inside an active box (above its floor), the kill height max(box floor, centre − 7.5); else it stays dead.
    fn add(&mut self, rng: &mut Rng, i: usize) {
        let c = self.centre;
        let mut p = [c[0], c[1], c[2], 0.0];
        p[0] += rng.frand(30.0) - 15.0;
        p[2] += rng.frand(30.0) - 15.0;
        p[1] += if self.kind == 2 { rng.frand(30.0) - 15.0 } else { rng.frand(7.5) + 7.5 };
        for (b, active) in &self.clipped {
            if !*active || !(b.min[0] < p[0] && p[0] < b.max[0] && b.min[2] < p[2] && p[2] < b.max[2]) { continue; }
            if self.kind == 2 {
                p[1] = p[1].max(b.min[1]);
                p[3] = b.max[1].min(c[1] + 7.5);
            } else {
                if !(b.min[1] < p[1]) { continue; }
                p[3] = b.min[1].max(c[1] - 7.5);
            }
            self.drops[i] = p;
            self.alive[i] = true;
            return;
        }
        p[1] -= 1054.0;
        self.drops[i] = p;
    }
}


/// `PS2SnowSize` per level: `psiPreGameRun` 0x11e2e8 writes 0x46e27c00 (28990) on 07000005, 0x46486800
/// (12826) on 07000006 and 0x4694da00 (19053) on 07000008 [M 0x11e2e8 cases]; no other level writes it,
/// so they keep the `.sdata` boot value 10000.0 [D ACTION.ELF .sdata 0x30c974] (after a 05/06/08 visit the
/// game would keep that level's value — the port takes the boot value, the first-load case).
pub fn ps2_snow_size(level: u32) -> f32 {
    match level {
        0x0700_0005 => f32::from_bits(0x46e2_7c00),
        0x0700_0006 => f32::from_bits(0x4648_6800),
        0x0700_0008 => f32::from_bits(0x4694_da00),
        _ => 10000.0,
    }
}

/// `Ps2Far1` per level (`psiPreGameRun` 0x11e2e8; the `default` arm 4194240 [M]).
pub fn ps2_far1(level: u32) -> f32 {
    match level {
        0x0700_0002 => 9_987_520.0,
        0x0700_0004 => 9_379_776.0,
        0x0700_0005 => 19_118_528.0,
        0x0700_0006 => 8_059_232.0,
        0x0700_0008 => 13_191_350.0,
        0x0700_000c => 20_475_840.0,
        0x0700_000d => 19_707_840.0,
        _ => 4_194_240.0,
    }
}

/// The eye distance inside which the GS's 24-bit depth wraps: `PS2WorldViewVU1Kick2` 0x10b2b0 maps a
/// vertex at view depth d to zbuf(d) = N1 + (F1 − N1)·n·(f/d − 1)/(f − n) with N1 = `Ps2Near1` 8.0,
/// F1 = `Ps2Far1`, n = `Ps2Near2` 0.1, f = `Ps2Far2` 2^24 [M; L ee_start_05.bin], and the leaves
/// write ZBUF_1 PSM 1 (Z24) [D ps2gfx]. zbuf reaches 2^24 at this distance: nearer vertices (the
/// near-plane clip points at d = n included, zbuf = F1) land past the 24-bit range. Only the levels
/// whose F1 exceeds 2^24 (05, 0C, 0D) put it beyond the 0.1 near plane; elsewhere it is < n.
pub fn zbuf_wrap_distance(level: u32) -> f32 {
    const N1: f64 = 8.0;
    const NEAR: f64 = 0.1;
    const FAR2: f64 = 16_777_216.0;
    let f1 = ps2_far1(level) as f64;
    (FAR2 / (1.0 + (FAR2 - N1) * (FAR2 - NEAR) / ((f1 - N1) * NEAR))) as f32
}

/// The world-space half-extent of a weather flake (snow / rising drops, `DrawDrops` 0x18cb90 →
/// `psiDrawParticleList` → `CreatePointerSpriteList` 0x11a160). The sprite's VU1 scale word is
/// `1/(PS2SnowSize·PS2SnowSizeMult)` [M 0x11a160:81]; an emitter sprite of def size `s` gets
/// `4·s/(PS2ParticleSize·PS2SnowSizeMult)` with `PS2ParticleSize = Ps2Far1·0.002386635` [M 0x119910:81,
/// 0x11e2e8:67], so a flake is the emitter sprite of size `Ps2Far1·0.002386635/(4·PS2SnowSize)` — the
/// ∝ Far1 factor cancels the depth scale the same way for both. The three levels that set a size land on
/// nearly the same world size (Far1/SnowSize = 659 / 628 / 692 on 05 / 06 / 08), which is what a size
/// tuned against Far1 looks like. The emitter `s` is a FULL width here (half-extent `s/2`): measured on
/// the 05 f198 and 06 f206 PS2 captures, where the near flakes are ~2× the old 0.1 stand-in (a half-
/// extent `s` read draws them ~4×). 05 → 0.197, 06 → 0.187, 08 → 0.207, boot value (07, 14) → 0.125.
pub fn flake_half_extent(level: u32) -> f32 {
    ps2_far1(level) * 0.002386635 / (4.0 * ps2_snow_size(level)) * 0.5
}

/// The breath-vapor puff's mesh (`Gas_Init` arg 7 in `Env_Breath` 0x17be00: `ori $t0, $t0, 0x21d`
/// with `lui $t0, 0x200` at 0x17bf14/0x17bfb8 [M disasm]).
pub const BREATH_MESH: u32 = 0x0200_021d;

/// Both breath variants' `Gas_Init` rgba end: `lui $t3, 0xffff` + `ori $t3, $t3, 0xff01` = 0xffffff01
/// [M disasm 0x17bf20/0x17bf68 and 0x17bfc8/0x17c020] — white, alpha 1.
pub const BREATH_RGBA_END: u32 = 0xffff_ff01;

/// `Env_WaterRipple` 0x17c118 (148 B): the hit-driven water ring. `material` is the HITDATA+0x50
/// surface byte, `hit_point` the HITDATA+0x40 contact. The gate [M disasm 0x17c120-0x17c138]:
/// `hit != 0` (the caller passes nothing on a null HITDATA) and `(material & 0x3f) - 0xd < 2`
/// unsigned — surface materials 0x0d/0x0e are the water pair. The attach-cel resolution
/// (HITDATA+0x54, else HITDATA+0x58 -> +0x20, else abort [M 0x17c140-0x17c158]) is the caller's:
/// the port's `Gas` record carries no room/cel link (the game's `build_LinkToRoom` /
/// `control_link_object_to_cel` cull attach — `Gas_Init` 0x1aacc8 tail), so a null cel here is
/// only knowable upstream [S — the port's hit paths do not model cel attach yet].
///
/// The spawn [M disasm 0x17c160-0x17c19c]: `Gas_Init(0.5, 1.01, cel, hit+0x40, quat 0, vel 0,
/// 0x0200_0129, kind 1, rgba 0x7f7f7f7f, rgba_end 2, life 63)` — `sw $v0(0x3f), ($sp)` at
/// 0x17c180 is the life the decompiler drops; kind 1 with a null velocity keeps `Gas_Init`'s
/// (0, REC, 0) default [M 0x1aacc8 tail; gas.rs `init_ex` `_` arm]. White ring, alpha 0x7f -> 2,
/// growing 0.5 -> 1.01 over 63 ticks.
pub fn env_water_ripple(gas: &mut crate::gas::GasPool, rng: &mut Rng, material: u8, hit_point: V3) {
    if material.wrapping_sub(0x0d) & 0x3f >= 2 {
        return;
    }
    gas.init_ex(rng, crate::world_fx::RIPPLE_MESH, hit_point, 0.0, [0.0; 3], 0.5, 1.01, 1,
        [0x7f, 0x7f, 0x7f, 0x7f], 2, 63.0);
}

/// `Env_Breath` 0x17be00 (584 B): the cold-breath vapor puffs. Args map to the game's
/// `(VECTOR* pos, VECTOR* dir, cel_tag* cel, uint phase)` [M 0x17be00 prologue] plus the fields
/// its gates read: `zone` = `cel+0xa0` (the per-cel breath zone byte, signed, [M 0x17be24]),
/// `cel_env_bit` = `cel+0x3c & 0x10000000` [M 0x17beec-0x17bef4], `cam_mode` =
/// `glb_viewer+0x236` (the viewer's cam word [M 0x17bf7c-0x17bf84]), and `env_gate` = the
/// `GlobalEnv` pair — `Some(cond)` is the game's `GlobalEnv == 0 || GlobalEnv+0x4b != 0`
/// disjunct result, `None` the unported-`GlobalEnv` case, which takes the `GlobalEnv == 0` arm
/// (true) [S — the port models no GlobalEnv; the two arms differ only in variant A vs B].
/// `frame` = `GameState+0x34` (the decompile's `_52_4_`, decimal 52) [M 0x17be3c-0x17be48; the same counter `WorldFx::tick` takes].
///
/// Gates [M disasm 0x17be1c-0x17bec8]: zone != 0; `(frame - phase) % (FRAME_RATE_INT/10) == 0`
/// (the game divides the global by 10 — 60/10 = 6 at the shipped refresh, folded to the literal
/// 10-divide at 0x17be30-0x17be40); and `(float)((frame - phase) % (FRAME_RATE_INT*2))` <=
/// `(FRAME_RATE+FRAME_RATE) * 0.33333334` = 40.0 — a 41-of-120 sub-window.
///
/// The spawn, `pos + dir*0.1` (0x3dcccccd = 0.1, stored as the broadcast scalar at sp+0x20 / sp+0x30
/// for the COP2 `_vmulbc/_vadd` [M disasm 0x17bf00-0x17bf18, 0x17bfa4-0x17bfbc]):
/// - variant A (env gate true AND cam_mode == 2 [M 0x17bed0-0x17bf84]): `Gas_Init(0.05, 0.2,
///   cel, pos, quat {0, 0, Rand_FRand(6.2831855), ..}, dir, 0x0200_021d, kind 0, rgba
///   0xffffffc8, rgba_end 0xffffff01, life 40)` [M 0x17bfe4-0x17c020: f12 = 0x3d4ccccd, f13 =
///   0x3e4ccccd, t2 = `lui 0xffff` + `ori 0xffc8`, t3 = `lui 0xffff` + `ori 0xff01`, the
///   `sw $v0(0x28), ($sp)` life at 0x17c010], then the gas object's `+0xf0 |= 0x400`
///   [M 0x17c024-0x17c02c] — the x-ray-only draw bit (`View_AddObjects` 0x1e62d8:38-47 submits it
///   only while the viewer's +0x236 is 2), carried as `Gas::xray_only`. The velocity is the `dir`
///   argument times REC_FRAME_RATE (Gas_Init's non-kind-4 `param_6 != 0` arm [M 0x1aacc8 tail]) —
///   the puff drifts with the breath.
/// - variant B (env gate false — any cam mode): `Gas_Init(0.05, 0.1, cel, pos, quat NULL, dir,
///   0x0200_021d, kind 0, rgba 0xffffff80, rgba_end 0xffffff01, life 40)` [M 0x17bf00-0x17bf68:
///   f12 = 0x3d4ccccd (0.05), f13 = 0x3dcccccd (0.1), t2 = 0xffffff80, t3 = 0xffffff01, the
///   `sw 0x28` life at 0x17bf48; a2 = 0 is the null quaternion] — the plain puff, no random yaw,
///   no +0xf0 flag, growing 0.05 -> 0.1 over its 40 ticks.
///
/// Both ends are 0xffffff01: the puff stays white and only its alpha runs out (128 -> 1 or
/// 200 -> 1), in `Gas_Init`'s big-endian (r,g,b,a) packing at obj+0x103..0x106 [M 0x1aacc8].
/// (The decompile prints the end as the sign-extended 0xffffffffffffff01; an earlier reading
/// truncated it to 0xff01, a fade to blue that darkened the puff.) The variant-A yaw draw and
/// `Gas_Init`'s own spin/sign draws now come out of the shared stream in the game's order
/// [M 0x17bf8c-0x17bfa0, 0x1aacc8:72-80] — the old counter-hash stand-in shifted every later
/// shared draw (AI, bullets) on every breathing guard.
/// The breath origin a guard hands `Env_Breath` [M `NDrone2_ControlSTANDARD` 0x1490f0:172-198]:
/// `AnimGetBoneWorldTrans(obj, bone 5, …)` gives the head bone's world matrix; the direction is
/// its z row (the COP2 `_vmulabc/_vmaddabc/_vmaddbc` of the rows with (0, 0, 1)), and the position
/// is the bone origin plus that direction · 0.12 (0x3df5c28f). `head` uses the row-vector
/// convention (rows = axes, row 3 = translation).
pub fn guard_breath_origin(head: &[[f32; 4]; 4]) -> (V3, V3) {
    let dir = [head[2][0], head[2][1], head[2][2]];
    let k = f32::from_bits(0x3df5_c28f);
    ([head[3][0] + dir[0] * k, head[3][1] + dir[1] * k, head[3][2] + dir[2] * k], dir)
}

/// The `Env_Breath` env-gate pair `GlobalEnv == 0 || GlobalEnv+0x4b != 0` [M 0x17bed0-0x17bee4]:
/// `GlobalEnv` is the level's `Env_Create` 0x17b968 record and `+0x4b` its drop type
/// (`pauVar8[4][0xb] = level_tag+0x2c`, the `InitDrops` argument [M 0x17b968:66-68,112-114]).
/// `drop_type` = the winning 0xDF placement's param 0, `None` without one. Drop type 0 is SNOW
/// (`Drops::kind`), so the gate is false exactly on the snow levels: there the plain puff
/// (variant B) shows in every mode, and everywhere else breath spawns only in x-ray (variant A).
pub fn breath_env_gate(drop_type: Option<u8>) -> bool {
    match drop_type { None => true, Some(k) => k != 0 }
}

pub fn env_breath(
    gas: &mut crate::gas::GasPool, rng: &mut Rng, frame: u32, phase: u32, pos: V3, dir: V3,
    zone: i8, cel_env_bit: bool, cam_mode: i16, env_gate: Option<bool>,
) {
    if zone == 0 {
        return;
    }
    // Both remainders are `divu` (unsigned) on the u32 difference [M disasm 0x17be40-0x17be64].
    let delta = frame.wrapping_sub(phase);
    if delta % 6 != 0 {
        return;
    }
    if (delta % 120) as f32 > 40.0 {
        return;
    }
    // pos + dir*0.1 [M 0x3dcccccd at 0x17bf00 / 0x17bfa4, both variants]
    let at = [pos[0] + dir[0] * 0.1, pos[1] + dir[1] * 0.1, pos[2] + dir[2] * 0.1];
    // The env gate: `GlobalEnv == 0 || GlobalEnv+0x4b != 0 || cel+0x3c & 0x10000000`
    // [M 0x17bed0-0x17bef8]. `None` = no GlobalEnv modelled -> the `GlobalEnv == 0` arm.
    let gate = env_gate.unwrap_or(true) || cel_env_bit;
    if gate {
        if cam_mode != 2 {
            return;
        }
        // Variant A: the shared-stream yaw draw `Rand_FRand(2π)` stored as the quaternion z
        // [M 0x17bf8c-0x17bfa0: `lui $at, 0x40c9; ori $at, $at, 0xfdb; jal Rand_FRand` — the
        // result lands in the quat quad's z slot at sp+0x68 before the `jal Gas_Init` at
        // 0x17bfe4], kind 0, dir*REC velocity. The draw comes BEFORE Gas_Init's own spin/sign
        // (init_ex's), the game's order.
        let yaw = rng.frand(6.2831854820251465);
        gas.init_ex(rng, BREATH_MESH, at, yaw, dir, 0.05, 0.2, 0, [255, 255, 255, 200], BREATH_RGBA_END, 40.0);
        if let Some(g) = gas.puffs.last_mut() {
            // Gas_Init's non-kind-4 velocity arm: arg * REC_FRAME_RATE [M 0x1aacc8 tail].
            let rec = 1.0 / 60.0;
            g.vel = [dir[0] * rec, dir[1] * rec, dir[2] * rec];
        }
        // +0xf0 |= 0x400 [M 0x17c024-0x17c02c]: the puff draws only in x-ray (`Gas::xray_only`).
        if let Some(g) = gas.puffs.last_mut() { g.xray_only = true; }
    } else {
        // Variant B: null quaternion, NO yaw draw [M 0x17bf00-0x17bf68: a2 = 0 is the null
        // quaternion; no Rand call on this arm], 0.05 -> 0.1 [f12 0x3d4ccccd, f13 0x3dcccccd].
        gas.init_ex(rng, BREATH_MESH, at, 0.0, dir, 0.05, 0.1, 0, [255, 255, 255, 128], BREATH_RGBA_END, 40.0);
        if let Some(g) = gas.puffs.last_mut() {
            let rec = 1.0 / 60.0;
            g.vel = [dir[0] * rec, dir[1] * rec, dir[2] * rec];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snow_falls_inside_its_box_and_respawns_above_the_viewer() {
        let mut rng = Rng::new(1);
        let mut fill = Rng::new(2);
        let boxes = vec![RainBox { min: [-100.0, -20.0, -100.0], max: [100.0, 80.0, 100.0] }];
        let mut d = Drops::new(0, boxes, 0x0700_0005, &mut rng);
        assert_eq!(d.drops.len(), 4096);
        let eye = [0.0, 10.0, 0.0];
        d.update(&mut rng, &mut fill, eye, 0.0);
        assert!(d.alive.iter().all(|&a| a), "first frame: the cube is full");
        for _ in 0..600 { d.update(&mut rng, &mut fill, eye, 0.0); }
        // Every live flake is near the cube around the centre (eye + 7.5 ahead; flakes drift up to 0.02 + wind per tick
        // and are only killed by height) and above its kill height.
        for (p, &a) in d.drops.iter().zip(&d.alive) {
            if !a { continue; }
            assert!((p[0] - d.centre[0]).abs() < 40.0 && (p[2] - d.centre[2]).abs() < 40.0, "{p:?}");
            assert!(p[1] >= p[3] - 0.2, "{p:?}");
        }
        assert!(d.alive.iter().filter(|&&a| a).count() > 1000);
        // Sister: no box → nothing moves or respawns.
        let mut e = Drops::new(0, Vec::new(), 0x0700_0001, &mut rng);
        e.update(&mut rng, &mut fill, eye, 0.0);
        assert!(e.alive.iter().all(|&a| !a));
    }

/// Rain falls at the game's speed AND straight: `DAT_003194f4 = −0.5` is the whole y step and the
/// x/z stay where `AddDrop` put them. [M 0x18bf68 sets DAT_003194f4 = −0.5 for rain; M 0x18c580
/// rain branch: each drop's step is `_lqc2(_WindVector2)` + `_vadd` — the uniform wind quad
/// `{WindVector, DAT_003194f4, DAT_003194f8}·FRM`, NOT the per-slot `RandTable2` (the table loop
/// runs but is read only by the snow/rising scratchpad walk); disasm 0x18c6f8 `beq DropType,1 →
/// 0x18c7e0`]. So every rain drop falls at exactly 0.5/tick (30 units/s) with no slant — a
/// 15–22.5 m fall (spawn eye+7.5..15 [M 0x18c278:38-39], kill max(box floor, eye−7.5)
/// [M 0x18c278:106-108]) takes 30–45 ticks, and with WindVector 0 for rain [M 0x18bf68 zeroes it;
/// 0x18c580:139 the kind-1 path skips the two wind draws] the drift is exactly 0 — rain stays in
/// its RainBox (the office-lounge symptom: the slow fall + slant drifted it up to ~6 m into rooms
/// the boxes don't cover). Snow keeps the slow per-slot fall
/// (`DAT_003194f4 = 0` [M 0x18bf68 else arm]) and drifts wide — the sister pins that.
#[test]
fn rain_falls_at_the_game_speed_and_stays_in_its_box() {
    let mut rng = Rng::new(7);
    let mut fill = Rng::new(8);
    // A tall open-air box: the clipped floor stays below the kill height, so every drop falls the
    // full spawn-to-kill 15..22.5 m.
    let boxes = vec![RainBox { min: [-30.0, -100.0, -30.0], max: [30.0, 60.0, 30.0] }];
    let mut d = Drops::new(1, boxes, 0x0700_000a, &mut rng);
    assert_eq!(d.fall, -0.5, "InitDrops sets DAT_003194f4 = -0.5 for rain [M 0x18bf68]");
    assert_eq!(d.drops.len(), 0x80, "rain on 07000009..0b runs 128 drops [M 0x18bf68]");
    let eye = [0.0, 10.0, 0.0];
    // Track one drop's full fall: y steps exactly 0.5/tick; x/z NEVER change [M 0x18c580 rain
    // branch — the step is the uniform quad with WindVector 0; no per-slot slant exists].
    let mut tracked: Option<[f32; 4]> = None;
    let mut fell = 0.0f32;
    let mut steps = 0u32;
    for tick in 0..600 {
        d.update(&mut rng, &mut fill, eye, 0.0);
        let mut matched = false;
        for (p, &a) in d.drops.iter().zip(&d.alive) {
            if !a { continue; }
            assert!(p[0].abs() <= 30.0 && p[2].abs() <= 30.0,
                "tick {tick}: rain drop {p:?} outside its box — rain has no slant to drift with");
            if let Some(t) = tracked {
                if p[0] == t[0] && p[2] == t[2] && p[1] < t[1] {
                    let dy = t[1] - p[1];
                    assert_eq!(dy, 0.5, "rain y step must be exactly 0.5 [M 0x18c580 rain branch], got {dy}");
                    fell += dy; steps += 1; matched = true;
                    tracked = Some(*p);
                }
            } else if p[1] > eye[1] + 7.0 && p[1] < eye[1] + 15.5 {
                tracked = Some(*p);
                matched = true;
            }
        }
        if !matched && steps < 10 { tracked = None; } // re-acquire after a death
        if steps >= 10 { break; }
    }
    assert!(steps >= 10, "no drop survived 10 ticks to measure the fall (fell {fell})");
}

/// Sister: snow keeps `DAT_003194f4 = 0` [M 0x18bf68 else arm] — its y step stays in the table's
/// −0.025..−0.1, so its long fall drifts drops well past 1 m of the box (the game's snow clouds
/// spread; this pins that the rain fix did not slow or confine the snow).
#[test]
fn snow_keeps_the_slow_fall_and_drifts() {
    let mut rng = Rng::new(9);
    let mut fill = Rng::new(10);
    let boxes = vec![RainBox { min: [-30.0, -100.0, -30.0], max: [30.0, 60.0, 30.0] }];
    let mut d = Drops::new(0, boxes, 0x0700_0005, &mut rng);
    assert_eq!(d.fall, 0.0, "snow's DAT_003194f4 is 0 [M 0x18bf68]");
    let eye = [0.0, 10.0, 0.0];
    let mut max_out = 0.0f32;
    for _ in 0..600 { d.update(&mut rng, &mut fill, eye, 0.0); }
    for (p, &a) in d.drops.iter().zip(&d.alive) {
        if !a { continue; }
        max_out = max_out.max((p[0].abs() - 30.0).max(p[2].abs() - 30.0));
    }
    assert!(max_out > 1.0, "snow drifts past 1 m of its box (max out {max_out}) — the slow fall is intact");
    // And its y step stays in the table's band (no −0.5).
    for i in 0..d.drops.len() {
        if !d.alive[i] { continue; }
        let before = d.drops[i][1];
        d.update(&mut rng, &mut fill, eye, 0.0);
        if !d.alive[i] { continue; }
        let dy = before - d.drops[i][1];
        assert!((0.02..=0.11).contains(&dy), "snow y step {dy} must stay in the table band");
        break;
    }
}

#[test]
fn breath_gate_fires_on_the_6th_frame_window() {
    let mut gas = crate::gas::GasPool::default();
    let mut rng = Rng::new(0x5e4c04);
    // (frame - phase) % 6 == 0 and % 120 <= 40 fires: frame 42, phase 0 -> 42%6=0, 42<=40? NO.
    env_breath(&mut gas, &mut rng, 42, 0, [0.0; 3], [0.0, 0.0, 1.0], 1, false, 2, None);
    assert!(gas.puffs.is_empty(), "42 is inside the 6-gate but outside the 40-window");
    assert_eq!(rng.draws, 0, "a gated-out breath consumes nothing [M 0x17be1c-0x17bec8]");
    // frame 36: 36%6=0, 36<=40 -> fires. Variant A (cam 2, no GlobalEnv): 0.05 -> 0.2, alpha 200.
    env_breath(&mut gas, &mut rng, 36, 0, [1.0, 2.0, 3.0], [0.0, 0.0, 1.0], 1, false, 2, None);
    assert_eq!(gas.puffs.len(), 1);
    let g = &gas.puffs[0];
    assert_eq!(g.mesh, BREATH_MESH);
    assert!((g.pos[2] - 3.1).abs() < 1e-6 && g.pos[0] == 1.0 && g.pos[1] == 2.0); // pos + dir*0.1 [M 0x3dcccccd at 0x17bfa4]
    assert!((g.scale - 0.05).abs() < 1e-6);
    assert!((g.scale_rate - (0.2 - 0.05) / 40.0).abs() < 1e-6); // (p2-p1)/life [M 0x1aacc8]
    assert_eq!(g.rgba[3], 200.0); // 0xffffffc8 alpha [M 0x17c018]
    assert_eq!(g.rgba_rate[3], (1.0 - 200.0) / 40.0); // end 0xffffff01 alpha 1 [M 0x17bfc8/0x17c020]
    assert_eq!(&g.rgba_rate[..3], &[0.0; 3]); // white stays white: end rgb 0xffffff [M lui 0xffff]
    assert!((g.life - 40.0).abs() < 1e-6); // sw 0x28 ($sp) [M 0x17c010]
    assert!((g.vel[2] - 1.0 / 60.0).abs() < 1e-6); // dir*REC [M 0x1aacc8 tail]
    // The draws: variant A = the yaw Rand_FRand(2π) [M 0x17bf8c-0x17bfa0] + Gas_Init's spin/sign
    // [M 0x1aacc8:72-80] = 3, replayed bit for bit on a twin stream.
    let mut r = Rng::new(0x5e4c04);
    let yaw = r.frand(6.2831854820251465);
    let spin = r.frand(std::f32::consts::FRAC_PI_2 * crate::gas::REC_FRAME_RATE);
    let sign = r.rand(0x100);
    let expect_spin = if sign >= 0x80 { -spin } else { spin };
    assert_eq!(g.spin.to_bits(), expect_spin.to_bits(), "variant A: the yaw draw then Gas_Init's spin/sign, in order");
    assert_eq!(r.draws, 3, "variant A consumed exactly the game's 3 draws");
    assert_eq!(rng.draws, 3);
}

#[test]
fn breath_gate_sisters_zone_and_window_edges() {
    // Sister: the zone byte clears -> nothing, whatever the frame [M 0x17be24-0x17be28].
    let mut gas = crate::gas::GasPool::default();
    let mut rng = Rng::new(0x5e4c04);
    env_breath(&mut gas, &mut rng, 36, 0, [0.0; 3], [0.0; 3], 0, false, 2, None);
    assert!(gas.puffs.is_empty());
    // Sister: frame 41 (41%6 != 0) -> nothing [M 0x17be58].
    env_breath(&mut gas, &mut rng, 41, 0, [0.0; 3], [0.0; 3], 1, false, 2, None);
    assert!(gas.puffs.is_empty());
    // Sister: variant B (env gate Some(false)) fires WITHOUT the cam-2 gate, alpha 128, 0.05 -> 0.1.
    env_breath(&mut gas, &mut rng, 36, 0, [0.0; 3], [0.0; 3], 1, false, 0, Some(false));
    assert_eq!(gas.puffs.len(), 1);
    assert_eq!(gas.puffs[0].rgba[3], 128.0); // 0xffffff80 [M 0x17bf60]
    assert!((gas.puffs[0].scale - 0.05).abs() < 1e-6); // f12 0x3d4ccccd [M 0x17bf24-0x17bf2c]
    assert!((gas.puffs[0].scale_rate - (0.1 - 0.05) / 40.0).abs() < 1e-7); // f13 0x3dcccccd [M 0x17bf00-0x17bf08]
    assert_eq!(&gas.puffs[0].rgba_rate[..3], &[0.0; 3]); // end 0xffffff01: no fade to blue [M 0x17bf20/0x17bf68]
    assert_eq!(gas.puffs[0].rgba_rate[3], (1.0 - 128.0) / 40.0);
    // Variant B makes NO yaw draw (null quaternion [M 0x17bf00-0x17bf68]) — Gas_Init's spin/sign
    // only [M 0x1aacc8:72-80]. The twin stream: 2 draws, then identical.
    assert_eq!(rng.draws, 2, "variant B consumed exactly Gas_Init's 2 draws");
    let mut r = Rng::new(0x5e4c04);
    let spin = r.frand(std::f32::consts::FRAC_PI_2 * crate::gas::REC_FRAME_RATE);
    let sign = r.rand(0x100);
    let expect = if sign >= 0x80 { -spin } else { spin };
    assert_eq!(gas.puffs[0].spin.to_bits(), expect.to_bits());
}

/// The two variants' draw bit: the mode-2 puff (env gate true) carries +0xF0 |= 0x400 — x-ray
/// only [M 0x17c024-0x17c02c]; the plain puff does not (sister). Gate true outside mode 2 spawns
/// nothing (sister). The env gate is false exactly for a snow env (drop type 0).
#[test]
fn breath_xray_variant_carries_the_xray_only_bit() {
    let mut gas = crate::gas::GasPool::default();
    let mut rng = Rng::new(0x5e4c04);
    env_breath(&mut gas, &mut rng, 36, 0, [0.0; 3], [0.0, 0.0, 1.0], 1, false, 2, Some(breath_env_gate(None)));
    assert_eq!(gas.puffs.len(), 1);
    assert!(gas.puffs[0].xray_only);
    assert_eq!(gas.puffs[0].rgba[3], 200.0); // 0xffffffc8 [M 0x17c018]
    // Sister: the same gate in mode 1 (night sight) spawns nothing.
    env_breath(&mut gas, &mut rng, 42, 0, [0.0; 3], [0.0, 0.0, 1.0], 1, false, 1, Some(breath_env_gate(Some(1))));
    assert_eq!(gas.puffs.len(), 1);
    // Sister: the snow env (drop type 0) takes the plain puff in mode 0 — no x-ray bit.
    env_breath(&mut gas, &mut rng, 30, 0, [0.0; 3], [0.0, 0.0, 1.0], 1, false, 0, Some(breath_env_gate(Some(0))));
    assert_eq!(gas.puffs.len(), 2);
    assert!(!gas.puffs[1].xray_only);
    // The underwater cel bit (cel+0x3c & 0x10000000) forces the mode-2 variant even under snow.
    env_breath(&mut gas, &mut rng, 24, 0, [0.0; 3], [0.0, 0.0, 1.0], 1, true, 0, Some(breath_env_gate(Some(0))));
    assert_eq!(gas.puffs.len(), 2);
    assert!(breath_env_gate(None) && breath_env_gate(Some(1)) && breath_env_gate(Some(0xff)) && !breath_env_gate(Some(0)));
}

/// The guard's breath origin: head bone z row as the direction, origin + dir·0.12
/// [M 0x1490f0:172-198]; a sister with a turned head moves both.
#[test]
fn guard_breath_origin_is_the_head_z_row() {
    let m = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [2.0, 1.5, -3.0, 1.0]];
    let (p, d) = guard_breath_origin(&m);
    assert_eq!(d, [0.0, 0.0, 1.0]);
    assert!((p[2] - (-3.0 + 0.12)).abs() < 1e-6 && p[0] == 2.0 && p[1] == 1.5);
    let turned = [[0.0, 0.0, -1.0, 0.0], [0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0], [2.0, 1.5, -3.0, 1.0]];
    let (p2, d2) = guard_breath_origin(&turned);
    assert_eq!(d2, [1.0, 0.0, 0.0]);
    assert_ne!(p2, p);
}

#[test]
fn water_ripple_fires_on_the_water_materials() {
    let mut gas = crate::gas::GasPool::default();
    let mut rng = Rng::new(0x5e4c04);
    env_water_ripple(&mut gas, &mut rng, 0x0d, [5.0, 0.5, 6.0]);
    assert_eq!(gas.puffs.len(), 1);
    let g = &gas.puffs[0];
    assert_eq!(g.mesh, crate::world_fx::RIPPLE_MESH); // 0x0200_0129 [M 0x17c18c]
    assert_eq!(g.pos, [5.0, 0.5, 6.0]); // HITDATA+0x40 [M 0x17c17c]
    assert!((g.scale - 0.5).abs() < 1e-6 && (g.scale_rate - (1.01 - 0.5) / 63.0).abs() < 1e-6);
    assert_eq!(g.rgba, [127.0, 127.0, 127.0, 127.0]); // 0x7f7f7f7f [M 0x17c194]
    assert_eq!(g.rgba_rate[3], (2.0 - 127.0) / 63.0); // end 2 [M 0x17c19c], life 63 [M 0x17c180]
    assert_eq!(g.kind, 1); // t1 [M 0x17c190]
    // The draws: Gas_Init's spin/sign only — kind 1 with a NULL velocity makes no velocity draws
    // [M 0x1aacc8:72-80, 105-129].
    let mut r = Rng::new(0x5e4c04);
    let spin = r.frand(std::f32::consts::FRAC_PI_2 * crate::gas::REC_FRAME_RATE);
    let sign = r.rand(0x100);
    let expect = if sign >= 0x80 { -spin } else { spin };
    assert_eq!(g.spin.to_bits(), expect.to_bits());
    assert_eq!(r.draws, 2, "the ripple consumed exactly Gas_Init's 2 draws");
    // Sister: material 0x0c (just below the water pair) -> nothing [M 0x17c130-0x17c138 sltiu 2].
    let mut gas2 = crate::gas::GasPool::default();
    env_water_ripple(&mut gas2, &mut rng, 0x0c, [0.0; 3]);
    assert!(gas2.puffs.is_empty());
    // Sister: 0x0f (just above) -> nothing.
    env_water_ripple(&mut gas2, &mut rng, 0x0f, [0.0; 3]);
    assert!(gas2.puffs.is_empty());
    assert_eq!(rng.draws, 2, "a gated-out ripple consumes nothing [M 0x17c130-0x17c138]");
}
}

#[test]
fn snow_size_is_the_game_table_and_the_flake_follows_it() {
    // [M 0x11e2e8] the three level writes, as the float words the game stores.
    assert_eq!(ps2_snow_size(0x0700_0005), 28990.0);
    assert_eq!(ps2_snow_size(0x0700_0006), 12826.0);
    assert_eq!(ps2_snow_size(0x0700_0008), 19053.0);
    // [D .sdata 0x30c974] every other level keeps the boot value.
    assert_eq!(ps2_snow_size(0x0700_0007), 10000.0);
    assert_eq!(ps2_snow_size(0x0700_0014), 10000.0);
    let h = |l| flake_half_extent(l);
    assert!((h(0x0700_0005) - 0.19674).abs() < 1e-4);
    assert!((h(0x0700_0006) - 0.18746).abs() < 1e-4);
    assert!((h(0x0700_0008) - 0.20655).abs() < 1e-4);
    assert!((h(0x0700_0007) - 0.12513).abs() < 1e-4, "07: Far1 4194240 default / boot size");
    // Sister: the size really depends on the level (05 vs 06 differ; neither is the old fixed 0.1).
    assert!((h(0x0700_0005) - h(0x0700_0006)).abs() > 0.005);
    assert!((h(0x0700_0005) - 0.1).abs() > 0.05);
}

#[test]
fn zbuf_wrap_distance_follows_far1() {
    // 0C (Far1 20475840): zbuf passes 2^24 at d ≈ 0.122 — past the 0.1 near plane.
    let c = zbuf_wrap_distance(0x0700_000c);
    assert!((c - 0.12205).abs() < 2e-4, "0C wrap distance {c}");
    // The formula at that distance is 2^24 (the definition), and at the near plane it is Far1 itself.
    let zb = |d: f64, f1: f64| 8.0 + (f1 - 8.0) * 0.1 * (16_777_216.0 / d - 1.0) / (16_777_216.0 - 0.1);
    assert!((zb(c as f64, 20_475_840.0) - 16_777_216.0).abs() < 1e3);
    assert!((zb(0.1, 20_475_840.0) - 20_475_840.0).abs() < 1.0);
    // 05 / 0D also wrap beyond the near plane; 12 (the other Ronin level) and 07 keep the default
    // Far1 4194240 → the wrap sits well inside the near plane.
    assert!(zbuf_wrap_distance(0x0700_0005) > 0.1 && zbuf_wrap_distance(0x0700_000d) > 0.1);
    assert!(zbuf_wrap_distance(0x0700_0012) < 0.03, "{}", zbuf_wrap_distance(0x0700_0012));
    // Sister: 08 (Far1 13191350 < 2^24) never wraps past the near plane.
    assert!(zbuf_wrap_distance(0x0700_0008) < 0.1);
}
