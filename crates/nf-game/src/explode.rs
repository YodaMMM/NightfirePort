//! The explosion entity: `Explode_Create` 0x17cbd0, `Explode_Update` 0x17c350, `Explode_Delete` 0x17cdc0,
//! `Explode_CollisionHandler` 0x17cde0 and `Explode_Propagate` 0x17c518.
//!
//! The game's explosion is a real object (control type 0x40, `obj+0xFF = 0xF`, collision `obj+0xF0 = 0x30`)
//! carrying an EXPLODEINFO block at `obj+0xE0`:
//! - `+0x20` cel (`*(owner_obj+0x20)` — the collision world the sphere test runs against),
//! - `+0x24` attacker obj (the thrower; `Control_ObjectReferenced(attacker, 1)` keeps it alive),
//! - `+0x28` damage (float, `Explode_Propagate`'s amount),
//! - `+0x2C` radius (float, the falloff divisor and the camera-shake amount),
//! - `+0x30` lifetime ticks (script? `*(u16*)(SCRIPTINFO+0xAB0)` : 15),
//! - `+0x34` SCRIPTINFO (the per-source effect script: 0x6000052 from the fuse/contact callers,
//!   0x60007c4 from the Samurai bolt's 0x100000/0x200000 arms, the destroyable's type script
//!   0x6000027/0x600004f/0x6000052 from `Destroy_Smash` — fix-blast-scripts census, from each
//!   site's disasm),
//! - `+0x38` u16 debris kind (`Explode_Update`'s switch: 5/6 metal, 8/10/0xB stone on 0x7000008 only),
//! - `+0x3A` u8 "propagate ran" latch (`Explode_CollisionHandler` fires `Explode_Propagate` once).
//!
//! Object fields: `obj+0x30` = the blast position, `obj+0xE8` = radius · 0.06666667 (the script scale),
//! `obj+0xF0` = 0x30, `obj+0xFF` = 0xF; `SCRIPTINFO+0x94` = f12 (the radius again at the fuse call site).
//!
//! [M] every field above is read off the ACTION.ELF disassembly (mipsdis 0x17cbd0/0x17c350/0x17cde0);
//! the decompiler drops the float args (f12/f13/f14 = script float/radius/damage), so the call sites at
//! 0x12cd98 (the fuse: radius = weapon+0x08, damage = weapon+0x0C, script 0x6000052, attacker = BU+0x30)
//! and 0x12ce80:194-206 (impact explode, `weapon+0x70 & 4`) were disassembled to pin them. The `addiu
//! $t2, $zero, 0x64` (100) at those sites is the 10th argument — an u8 `Explode_Create` never reads —
//! NOT the blast damage: the damage is `f14 = *(float*)(weapon_data+0x0C)` [L the records, named by the
//! disc labels (Txt_BindLabel of +0x38, resolved against USATxt.dat): 0x34 'Fragmentation Grenade' =
//! radius 8 / damage 100 — the real frag kill; 0x35 'Stun Grenade' = 10/0 and 0x36 'Smoke Grenade' = 0/0
//! never reach Explode_Create (the stun takes `Bullet_DoTrails` 0x12d890's `+0x70 & 0x2000` death branch,
//! the smoke its `& 0x20000` branch); 0x37-0x39 'Remote Mine' and 0x3a 'Laser Tripbomb' = 8/100;
//! 0x3b-0x40 'Satchel Charge' = 8/2000].

use crate::world::V3;

/// One explosion (`Explode_Create` 0x17cbd0's object).
#[derive(Clone, Debug)]
pub struct Explosion {
    /// `obj+0x30`: the blast position.
    pub pos: V3,
    /// `EXPLODEINFO+0x2C`: the radius — the falloff divisor and the camera-shake amount.
    pub radius: f32,
    /// `EXPLODEINFO+0x28`: the damage (`Explode_Propagate`'s amount).
    pub damage: f32,
    /// `EXPLODEINFO+0x24`: the attacker (the port's drone index, `usize::MAX` = Bond's throw).
    pub attacker: usize,
    /// `EXPLODEINFO+0x30`: lifetime ticks (the script's `+0xAB0` u16, else 15 [M 0x17cd38]).
    pub lifetime: u16,
    /// `EXPLODEINFO+0x34 != 0`: a script was loaded (the port's script player carries the visual).
    pub script: bool,
    /// `EXPLODEINFO+0x38`: the debris kind (`Explode_Update`'s switch; the port has no debris system —
    /// the game's own debris only spawns for kinds 5/6 (metal) and 8/10/0xB (stone, 0x7000008 only)
    /// after a random gate [M 0x17c350:31-67]).
    pub debris: u16,
    /// `EXPLODEINFO+0x3A`: the propagate latch (`Explode_CollisionHandler`).
    pub propagated: bool,
    /// `obj+0xEC` vs `GameState+0x52`: the age in ticks (the port counts from creation).
    pub age: u32,
    /// The weapon type that caused it (informational here — the death-branch routing lives in
    /// `drone.rs`'s tick on the weapon's own `+0x70` flags: & 0x2000 flash / & 0x20000 smoke [M
    /// 0x12dd60, 0x12e104]).
    pub weapon_type: u16,
    /// `SCRIPTINFO+0x98`: the loaded script's id — `Explode_Update`'s debris arm plays only for
    /// 0x0600052 [M 0x17c350:28]. 0 = no script loaded (the arm's script gate then passes — the
    /// game reads `inst+0x34 != 0 &&` before the hash compare).
    pub script_hash: u32,
    /// `instance+0x00..0x0f`: the debris direction — `Mat_GetUp` reads the aligned script matrix's
    /// up row into the block [M 0x17cbd0:96]. The port stores the aligned basis' up row [S: the
    /// game's matrix is the SCRIPT's, aligned by `Mat_Align2Dir`; the port's `align2dir` builds
    /// the same shape from the to-player vector].
    pub debris_dir: V3,
    /// `instance+0x10..0x1f`: the debris jitter — basisA + basisB of `Vec_MakeOrthonormalBasis`
    /// over the normalised to-closest-player vector [M 0x17c930:118-131, 0x17cbd0].
    pub debris_jitter: V3,
}

impl Explosion {
    /// `Explode_Create` (0x17cbd0): `script_hash = 0` gives lifetime 15 [M 0x17cd38 `addiu $v0, $zero, 0xf`];
    /// a script's lifetime is its `SCRIPTINFO+0xAB0` u16 [M 0x17cd18]. The camera shake is the caller's
    /// (`Camera_Shake(pos, radius)` only when radius ≥ 0.5 [M 0x17cc28-0x17cc44]); the port's Sim exposes
    /// it through `Drones::explosions` for the view.
    pub fn create(pos: V3, radius: f32, damage: f32, attacker: usize, script_lifetime: Option<u16>, debris: u16, weapon_type: u16) -> Explosion {
        Explosion {
            pos, radius, damage, attacker,
            lifetime: script_lifetime.unwrap_or(15),
            script: script_lifetime.is_some(),
            debris,
            propagated: false,
            age: 0,
            weapon_type,
            script_hash: 0,
            debris_dir: [0.0, 1.0, 0.0],
            debris_jitter: [0.0; 3],
        }
    }

    /// `Explode_Update` (0x17c350): the script player runs first (`Script_Update`); when it stops the
    /// object dies (`obj+0xFE |= 1` [M 0x17c394-0x17c3a0]). Without a script the lifetime ticks out.
    /// The debris spawns (kinds 5/6/8/10/0xB) are not ported — the port has no debris system; the
    /// gate was `lifetime>>2 < age` then `age % (Rand(5)+10) == 0` [M 0x17c3d8-0x17c460].
    /// Returns false when the explosion is done.
    pub fn update(&mut self) -> bool {
        self.age += 1;
        if self.script { return self.age <= self.lifetime as u32; }
        self.age < self.lifetime as u32
    }
}

/// One debris particle the port carries — `Debris_CreateEx` 0x1ac808's per-particle record
/// (the draw side renders the gfx hash; the motion integrator is `Debris_Update` 0x1aac00's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DebrisSpawn {
    /// instance+0x10..0x18: the initial velocity [M 0x1ac808:35-39].
    pub vel: V3,
    /// instance+0x20..0x2b: the `Rand_FRandHalf_Vec(0.24414063)` spin [M 0x1ac808:40 — 0x3e71463b].
    pub spin: V3,
    /// instance s16+0x30: the life in ticks, `Rand(count/2) + count/2` [M 0x1ac808:29-31].
    pub life: u16,
    /// obj+0xE8: the scale, `Rand_FRand(0.1) + 0.4` [M 0x1ac808:25-28].
    pub scale: f32,
    /// The gfx hash picked from the caller's array, `gfx[Rand(gfx_count−1)]` [M 0x1ac808:41-43].
    pub gfx: u32,
    /// obj+0xFF = 0x10 (class `Debris_Update`) [M 0x1ac808:24].
    pub class: u8,
}

/// `Debris_CreateEx` 0x1ac808 over the port's RNG. `speed` is param 1 (the blast's speed scale),
/// `dir` param 3 (the blast direction), `jitter` param 4 (the spread direction), `life_base`
/// param 5 (the life is `Rand(life_base/2) + life_base/2`), `count` param 6 (how many particles),
/// `gfx`/`gfx` param 8/9 (the mesh-hash array to pick from — the game picks
/// `gfx[Rand(gfx_count−1)]` per particle [M 0x1ac808:41-43]). The velocity [M 0x1ac808:32-39]:
/// `mag = speed · (Rand_FRand(1.5) + 0.1) · 0.5`; per axis
/// `vel[k] = (mag·dir[k] + jitter[k]·Rand_FRandHalf(speed)) · REC_FRAME_RATE` (REC_FRAME_RATE =
/// `1.0 / FRAME_RATE` = 1/60 at the shipped 60 Hz [M GS_SetRefreshRate 0x17d2b8: `REC_FRAME_RATE =
/// 1.0 / FRAME_RATE`] — a per-tick velocity from a per-second one). Empty `gfx` or `count == 0` spawns
/// nothing (the game's own guards [M 0x1ac808:18]).
pub fn create_debris(
    rng: &mut crate::drone::Rng,
    speed: f32,
    dir: V3,
    jitter: V3,
    life_base: u32,
    count: u32,
    gfx: &[u32],
) -> Vec<DebrisSpawn> {
    if gfx.is_empty() || count == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let scale = rng.frand(0.1) + 0.4;
        let life = rng.rand(life_base >> 1) as u16 + (life_base >> 1) as u16;
        let mag = speed * (rng.frand(1.5) + 0.1) * 0.5;
        let mut vel = [0f32; 3];
        let mut spin = [0f32; 3];
        for k in 0..3 {
            vel[k] = (mag * dir[k] + jitter[k] * rng.frand_half(speed)) * (1.0 / 60.0); // REC_FRAME_RATE [M 0x1ac808:33-38]
        }
        // `Rand_FRandHalf_Vec(0x3e71463b, …)` [M 0x1ac808:40] — three half-range draws. The
        // constant is 0x3E71463B = 0.2356194704771042 (exact f32) [L ee_start_05.bin].
        for s in spin.iter_mut() {
            *s = rng.frand_half(0.235_619_47);
        }
        let g = gfx[rng.rand(gfx.len() as u32 - 1) as usize];
        out.push(DebrisSpawn { vel, spin, life, scale, gfx: g, class: 0x10 });
    }
    out
}

/// `MetalHash_132` @0x2A3718 — the metal debris mesh hashes `Explode_Update` hands to
/// `Debris_CreateEx` for kinds 5/6 [L ee_start_05.bin @0x2a3718, four u32s; symbol
/// work/action-symtab.tsv `002a3718 MetalHash.132`].
pub const METAL_HASH_132: [u32; 4] = [0x0200_0414, 0x0200_0415, 0x0200_0416, 0x0200_041b];

/// `StoneHash_133` @0x2A3728 — the stone debris meshes, kinds 8/10/0xB, gated on level 0x0700008
/// with switch channel 0x53 on [L ee_start_05.bin @0x2a3728; the 4th entry repeats the 1st].
pub const STONE_HASH_133: [u32; 4] = [0x0200_0664, 0x0200_0665, 0x0200_0666, 0x0200_0664];

/// The blast script 0x0600052's `SCRIPTINFO+0xAB0` u16 — the lifetime `Explode_Create` stores for
/// a loaded script [M 0x17cd18], the window the debris arm may spawn in (first quarter: age ≤
/// 41 [M 0x17c350:31]).
pub const BLAST_SCRIPT_0x0600052: u32 = 0x0600_0052;
pub const BLAST_SCRIPT_LIFETIME: u16 = 165;

/// `Explode_Update` 0x17c350's debris arm, over the port's RNG. The gates in the game's order
/// [M 0x17c350:21-67]: MPSettings == 1 → none (the port's sim is single-player, no MP branch to
/// gate); debris kind == 0 → none WITHOUT a draw [M :25]; a loaded script whose +0x98 !=
/// 0x0600052 → none WITHOUT a draw [M :28]; `lifetime >> 2 < age` → none WITHOUT a draw [M :31];
/// then ONE `Rand_Rand(5)` draw, and `GameState_time % (r + 10) == 0` gates the spawn [M :34-38];
/// kinds 5/6 → `MetalHash_132`, 8/10/0xB → `StoneHash_133` only on level 0x0700008 with switch
/// channel 0x53 on, 7/9 and everything else → nothing [M :41-62]. A passing tick calls
/// `Debris_CreateEx(2·radius, obj+0xC0, up, jitter, 180, 3, cel, hash, 4)` — 3 particles from the
/// 180-tick life base [M :63-67, 0x1ac808]. `game_time` is the port's frame counter standing in
/// for GameState+0x52 [S: the alignment of the two counters is a live check]. Returns the spawned
/// particles (empty = nothing this tick); the draws happen exactly as in the game — consumed
/// even when the modulo gate then fails.
pub fn debris_arm(
    e: &Explosion,
    rng: &mut crate::drone::Rng,
    game_time: u32,
    level: u32,
    stone_channel_on: bool,
) -> Vec<DebrisSpawn> {
    if e.debris == 0 {
        return Vec::new();
    }
    if e.script_hash != 0 && e.script_hash != BLAST_SCRIPT_0x0600052 {
        return Vec::new();
    }
    if (e.lifetime as u32) >> 2 < e.age {
        return Vec::new();
    }
    let r = rng.rand(5);
    if game_time % (r + 10) != 0 {
        return Vec::new();
    }
    let gfx: &[u32] = match e.debris {
        5 | 6 => &METAL_HASH_132,
        8 | 10 | 0xb => {
            if level != 0x0700_0008 || !stone_channel_on {
                return Vec::new();
            }
            &STONE_HASH_133
        }
        _ => return Vec::new(),
    };
    create_debris(rng, e.radius + e.radius, e.debris_dir, e.debris_jitter,
        (60.0f32 * 3.0) as u32, 3, gfx)
}

/// Who an explosion damaged, in the port's terms (`Explode_Propagate`'s per-case effects).
#[derive(Clone, Debug, Default)]
pub struct PropagateHits {
    /// Case 2 (drones): `(drone index, damage)` — the game sends msg 9 with a
    /// `DroneImpact{+0 damage, +4 radius, +0x10 pos, +0x20 attacker}` [M 0x17c6e0-0x17c6fc].
    pub drones: Vec<(usize, f32)>,
    /// Case 3 (Bond): the `Player_Hurt` amount (pre-difficulty; the caller applies the level factor).
    pub player: Option<f32>,
    /// Case 0x20 (breakables): `(object index, damage)` — `hitdata+8 = damage`, `Break_ApplyDamage`.
    pub breaks: Vec<(usize, f32)>,
    /// Case 0x21 (destroyables): `Destroy_Smash(obj, attacker, 0x20)` when damage ≥ 1 and
    /// `*(u16*)(obj->data+0x14) & 0x20 == 0` [M 0x17c6a8-0x17c6c0].
    pub destroys: Vec<usize>,
    /// Case 0 (the copter): `(copter index, summed damage)` — the blast sweeps the copter's 9
    /// part slots; every part inside the sphere adds its falloff amount to rec+0x64 AND rec+0x6c
    /// DIRECTLY — no Bits scaling, no weapon filter, no HITDATA chain [M Explode_Propagate
    /// 0x17c518 case 0, the `CopterList` walk over `rec+0x30..0x54`].
    pub copters: Vec<(usize, f32)>,
}

/// The targets one `Explode_Propagate` sweep can hit, as the port's Sim sees them. The game gets them
/// from `Collide_SphereIntersect(pos, radius, cel, …, 0xc0)`; the port passes its own lists.
pub struct Targets<'a> {
    /// Case 2 — live drones `(index, pos, hit radius)` (`obj+0x80` position, `obj+0x8C` sphere [M]).
    pub drones: &'a [(usize, V3, f32)],
    /// Case 3 — Bond's position.
    pub player: V3,
    /// Case 0x20 — the breakables `(index, pos, hit radius)`.
    pub breaks: &'a [(usize, V3, f32)],
    /// Case 0 — the copter parts `(copter index, [(pos, radius); ≤9])`, the port's read of the
    /// game's `Collide_SphereIntersect` sweep over the script-entity parts (the parts are class-0
    /// objects; the copter itself is 0x3b and never matches case 0 [M Copter_Create 0x17d340
    /// sets obj+0xFF = 0x3b, the parts are the SP's class-0 entities]).
    pub copters: &'a [(usize, Vec<(V3, f32)>)],
    /// Case 0x21 — the destroyables `(index, pos, hit radius, smashed already, tag+0x14 mask)`.
    pub destroys: &'a [(usize, V3, f32, bool, u16)],
}

/// `Explode_Propagate` (0x17c518): a sphere of `radius` from `pos`; for every hit object
/// `amount = damage · (1 − (int)(dist − obj_radius) / radius)` — the game truncates the
/// distance-minus-sphere term to an INTEGER [M 0x17c620 `cvt.w.s` / `mtc1` pair], clamped at ≥ 0,
/// and anything below 0.0002 takes nothing [M 0x17c62c]. Case dispatch on `obj+0xFF`:
/// 0 copter (accumulate), 2 drone, 3 player, 5 destructible models {0x2B, 0x34–0x37, 0x3A},
/// 0x20 break, 0x21 destroy, 0x28 vehicle, 0x35 MP, 0x36 GT (+ disable), 0x37 sensor, 0x38 monitor,
/// 0x3D sub (class 'J'), 0x4B hit-list entry — 12 damaging cases plus the default skip.
/// `radius < 0.0002` (the flashbang's 0) hits NOTHING [M 0x17c550 early return].
pub fn propagate(pos: V3, radius: f32, damage: f32, attacker: usize, t: &Targets) -> PropagateHits {
    let mut out = PropagateHits::default();
    if !(damage >= 0.0002) || !(radius >= 0.0002) { return out; }
    let mut sweep = |px: V3, obj_radius: f32, hit: &mut dyn FnMut(f32)| {
        let dx = px[0] - pos[0];
        let dy = px[1] - pos[1];
        let dz = px[2] - pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt() - obj_radius;
        // `fVar10 = param_2 * (1 - (int)fVar10 / param_1)` — the (int) cast truncates toward zero
        // [M 0x17c61c-0x17c628]; negative distances (inside the object's sphere) give full damage.
        let truncated = if dist > 0.0 { (dist as i32) as f32 } else if dist < 0.0 { -((-dist) as i32) as f32 } else { 0.0 };
        let amount = damage * (1.0 - truncated / radius);
        if amount >= 0.0002 { hit(amount); }
    };
    for &(i, p, r) in t.drones {
        let mut d = 0.0f32;
        sweep(p, r, &mut |a| d = a);
        if d > 0.0 { out.drones.push((i, d)); }
    }
    {
        let mut p = 0.0f32;
        sweep(t.player, 0.0, &mut |a| p = a);
        if p > 0.0 { out.player = Some(p); }
    }
    for &(i, p, r) in t.breaks {
        let mut d = 0.0f32;
        sweep(p, r, &mut |a| d = a);
        if d > 0.0 { out.breaks.push((i, d)); }
    }
    for &(i, p, r, smashed, mask14) in t.destroys {
        if smashed { continue; }
        let mut d = 0.0f32;
        sweep(p, r, &mut |a| d = a);
        // Case 0x21's own gate: the smash needs at least 1.0 [M 0x17c6a8], and the tag's
        // bullet mask must open the 0x20 bit [M 0x17c518:170 — `*(u16*)(data+0x14) & 0x20 ==
        // 0`]: the C5 doors' 0xFFEF keeps it set, so NO blast ever smashes them — the drain
        // through `Destroy_CollisionHandler` is the only death.
        if d >= 1.0 && mask14 & 0x20 == 0 { out.destroys.push(i); }
    }
    // Case 0 [M 0x17c518:86-105]: the class-0 part sweep — per matching part the amount adds to
    // both accumulators, so the copter's total is the SUM over its hit parts (the game adds per
    // slot match with no dedup).
    for (ci, parts) in t.copters {
        let mut total = 0.0f32;
        for &(p, r) in parts {
            let mut d = 0.0f32;
            sweep(p, r, &mut |a| d = a);
            total += d;
        }
        if total > 0.0 { out.copters.push((*ci, total)); }
    }
    let _ = attacker;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explosion_without_script_lives_15_ticks() {
        // [M 0x17cd38] `Explode_Create` with script == 0 stores lifetime 15.
        let mut e = Explosion::create([0.0; 3], 10.0, 100.0, usize::MAX, None, 0, 0x35);
        for _ in 0..14 { assert!(e.update(), "alive at age {}", e.age); }
        assert!(!e.update(), "dead at age 15");
        // Sister: with a script the lifetime is the script's +0xAB0 (165 for the blast script's END 165).
        let mut e = Explosion::create([0.0; 3], 10.0, 0.0, usize::MAX, Some(165), 0, 0x35);
        for _ in 0..165 { assert!(e.update()); }
        assert!(!e.update(), "the scripted explosion dies at 166");
    }

    #[test]
    fn propagate_falloff_truncates_the_distance() {
        // Radius 10, damage 100: a drone at distance 5.7 − radius 1 = 4.7 → (int) 4 → 100·(1−4/10) = 60.
        // A smooth falloff would give 57; the game's integer truncation gives 60 [M 0x17c61c-0x17c628].
        let t = Targets { drones: &[(0, [5.7, 0.0, 0.0], 1.0)], player: [1000.0; 3], breaks: &[], destroys: &[], copters: &[] };
        let h = propagate([0.0; 3], 10.0, 100.0, usize::MAX, &t);
        assert_eq!(h.drones.len(), 1);
        assert!((h.drones[0].1 - 60.0).abs() < 1e-4, "truncated falloff, got {}", h.drones[0].1);
        // Inside the object's own sphere (dist − radius < 0): full damage, not negative.
        let t = Targets { drones: &[(0, [0.5, 0.0, 0.0], 1.0)], player: [1000.0; 3], breaks: &[], destroys: &[], copters: &[] };
        let h = propagate([0.0; 3], 10.0, 100.0, usize::MAX, &t);
        assert!((h.drones[0].1 - 100.0).abs() < 1e-4, "inside the sphere: full, got {}", h.drones[0].1);
        // Below the 0.0002 cutoff: nothing.
        let t = Targets { drones: &[(0, [9.999, 0.0, 0.0], 0.0)], player: [1000.0; 3], breaks: &[], destroys: &[], copters: &[] };
        let h = propagate([0.0; 3], 10.0, 100.0, usize::MAX, &t);
        assert!(h.drones.is_empty() || h.drones[0].1 >= 0.0002);
    }

    #[test]
    fn zero_radius_hits_nothing() {
        // The smoke grenade's record is radius 0 / damage 0 [L weapon 0x36 +0x08/+0x0C 'Smoke Grenade']:
        // the early return [M 0x17c550] means the smoke damages NOBODY through Propagate (and the stun
        // 0x35 never reaches Propagate at all — the `& 0x2000` flash branch).
        let t = Targets { drones: &[(0, [0.1, 0.0, 0.0], 0.0)], player: [0.1, 0.0, 0.0], breaks: &[], destroys: &[], copters: &[] };
        let h = propagate([0.0; 3], 0.0, 100.0, usize::MAX, &t);
        assert!(h.drones.is_empty() && h.player.is_none() && h.breaks.is_empty() && h.destroys.is_empty());
        let h = propagate([0.0; 3], 10.0, 0.0, usize::MAX, &t);
        assert!(h.drones.is_empty() && h.player.is_none());
    }

    #[test]
    fn destroy_case_needs_damage_at_least_one() {
        // Case 0x21's gate [M 0x17c6a8]: the smash fires only when the falloff reaches 1.0. With the
        // integer truncation the amounts jump in damage/radius steps: damage 5, radius 10 → a target at
        // 9.5 (truncated 9) takes 5·(1−9/10) = 0.5 → no smash; at 4.5 (truncated 4) it takes 3 → smash.
        let far = Targets { drones: &[], player: [1000.0; 3], breaks: &[],
            destroys: &[(0, [9.5, 0.0, 0.0], 0.0, false, 0)], copters: &[] };
        let h = propagate([0.0; 3], 10.0, 5.0, usize::MAX, &far);
        assert!(h.destroys.is_empty(), "0.5 damage must not smash");
        let near = Targets { drones: &[], player: [1000.0; 3], breaks: &[],
            destroys: &[(0, [4.5, 0.0, 0.0], 0.0, false, 0), (1, [4.5, 0.0, 0.0], 0.0, true, 0)], copters: &[] };
        let h = propagate([0.0; 3], 10.0, 5.0, usize::MAX, &near);
        assert_eq!(h.destroys, vec![0], "3 damage smashes the unsmashed one only");
    }

    #[test]
    fn destroy_case_respects_the_0x20_mask_bit() {
        // The second half of case 0x21's gate [M 0x17c518:170]: the smash needs
        // `*(u16*)(data+0x14) & 0x20 == 0`. The C5 doors' tag+0x14 is 0xFFEF [D #93/#96 p4
        // 0x10, M 0x12c318:47-52] — bit 0x20 set, so even a full-damage blast never smashes
        // them; a mask-0 tag (every p4-0 destroyable) takes the blast.
        let door = Targets { drones: &[], player: [1000.0; 3], breaks: &[],
            destroys: &[(0, [0.5, 0.0, 0.0], 0.0, false, 0xffef)], copters: &[] };
        let h = propagate([0.0; 3], 10.0, 100.0, usize::MAX, &door);
        assert!(h.destroys.is_empty(), "0xFFEF & 0x20 ≠ 0 — the gate blocks the smash [M 0x17c518:170-176]");
        let plain = Targets { drones: &[], player: [1000.0; 3], breaks: &[],
            destroys: &[(0, [0.5, 0.0, 0.0], 0.0, false, 0), (1, [0.5, 0.0, 0.0], 0.0, false, 0xffdf)], copters: &[] };
        let h = propagate([0.0; 3], 10.0, 100.0, usize::MAX, &plain);
        assert_eq!(h.destroys, vec![0, 1], "mask 0 and 0xFFDF (0x20 clear) both smash; the C5-shaped 0xFFEF does not");
    }

    #[test]
    fn debris_spawns_follow_the_create_math() {
        // [M 0x1ac808] the per-particle math over the port's bit-exact RNG.
        let mut rng = crate::drone::Rng::new(7);
        let gfx = [0x2000_001u32, 0x2000_002, 0x2000_003];
        let d = create_debris(&mut rng, 8.0, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 60, 5, &gfx);
        assert_eq!(d.len(), 5);
        for p in &d {
            assert_eq!(p.class, 0x10);
            assert!((0.4..0.5).contains(&p.scale), "scale = Rand(0.1) + 0.4, got {}", p.scale);
            assert!((30..60).contains(&p.life), "life = Rand(30) + 30, got {}", p.life);
            assert!(gfx.contains(&p.gfx));
            for s in p.spin {
                // [M 0x1ac808:40] the spin constant is 0x3E71463B = 0.23561947… [L ee_start_05.bin].
                assert!(s.abs() <= 0.235_619_47, "the spin is a half-range draw, got {s}");
            }
            // The velocity's y rides the blast direction (jitter only perturbs it around mag·dir).
            assert!(p.vel[1] > 0.0, "the direction term dominates on the y axis: {p:?}");
        }
        // Sister (can fail): the same seed reproduces bit-exact.
        let mut rng = crate::drone::Rng::new(7);
        let e = create_debris(&mut rng, 8.0, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], 60, 5, &gfx);
        assert_eq!(d, e);
    }

    #[test]
    fn debris_guards_match_the_game() {
        // [M 0x1ac808:18] `param_8 != 0 && param_6 != 0` — no mesh array or zero count spawns nothing.
        let mut rng = crate::drone::Rng::new(1);
        assert!(create_debris(&mut rng, 8.0, [0.0; 3], [0.0; 3], 60, 4, &[]).is_empty());
        assert!(create_debris(&mut rng, 8.0, [0.0; 3], [0.0; 3], 60, 0, &[0x2000_001]).is_empty());
        // Sister (can fail): a one-entry array is legal (Rand(0) → 0).
        let d = create_debris(&mut rng, 8.0, [0.0; 3], [0.0; 3], 2, 3, &[0x2000_009]);
        assert_eq!(d.len(), 3);
        assert!(d.iter().all(|p| p.gfx == 0x2000_009));
    }

    #[test]
    fn the_debris_arm_spin_constant_is_the_games_word() {
        // [M 0x1ac808:40 `Rand_FRandHalf_Vec(0x3e71463b, …)`; L ee_start_05.bin] the port's
        // constant must be bit-equal to the game's word — the old 0.24414063 (0x3E7A0000) was not.
        assert_eq!(0.235_619_47f32.to_bits(), 0x3e71_463b);
    }

    #[test]
    fn the_hash_arrays_match_the_snapshot() {
        // [L ee_start_05.bin @0x2a3718/@0x2a3728; symbols MetalHash.132/StoneHash.133].
        assert_eq!(METAL_HASH_132, [0x0200_0414, 0x0200_0415, 0x0200_0416, 0x0200_041b]);
        assert_eq!(STONE_HASH_133, [0x0200_0664, 0x0200_0665, 0x0200_0666, 0x0200_0664]);
    }

    fn blast(kind: u16, script: u32) -> Explosion {
        Explosion { pos: [0.0; 3], radius: 10.0, damage: 80.0, attacker: usize::MAX,
            lifetime: 165, script: script != 0, debris: kind, propagated: true, age: 1,
            weapon_type: 0x34, script_hash: script, debris_dir: [0.0, 1.0, 0.0],
            debris_jitter: [0.0; 3] }
    }

    #[test]
    fn kind_zero_blasts_never_draw_or_spawn_debris() {
        // The mine's inline Explode_Create stores kind 0 [M 0x17c930:128], Destroy_Smash's stack
        // slot is an explicit 0 [M 0x12bcf8 sw $zero,($sp)], the fuse arm's too [M 0x12ce14] — and
        // `Explode_Update` returns BEFORE the Rand_Rand(5) draw on kind == 0 [M 0x17c350:25-27].
        // 07000015's mines and oil barrels blast through these calls: no debris, no draws, ever.
        for kind in [0u16] {
            let mut rng = crate::drone::Rng::new(3);
            for age in 1..=41u32 {
                let mut e = blast(kind, 0x0600_0052);
                e.age = age;
                let before = rng.draws;
                assert!(debris_arm(&e, &mut rng, 1000 + age, 0x0700_0015, false).is_empty());
                assert_eq!(rng.draws - before, 0, "kind 0 returns before the draw [M 0x17c350:25]");
            }
        }
        // Sister (can fail): kind 6 DOES reach the draw.
        let mut rng = crate::drone::Rng::new(3);
        let e = blast(6, 0x0600_0052);
        let before = rng.draws;
        let _ = debris_arm(&e, &mut rng, 1000, 0x0700_0015, false);
        assert_eq!(rng.draws - before, 1, "one Rand_Rand(5) draw per gated tick [M 0x17c350:34]");
    }

    #[test]
    fn non_blast_script_blasts_never_draw_or_spawn_debris() {
        // `inst+0x34 != 0 && SCRIPTINFO+0x98 != 0x6000052 → return` before the draw [M 0x17c350:28].
        // The monitor blast (0x0600027) and the Samurai bolt's (0x06007c4) never spawn debris.
        for script in [0x0600_0027u32, 0x0600_004f, 0x0600_07c4] {
            let mut rng = crate::drone::Rng::new(5);
            let e = blast(6, script);
            let before = rng.draws;
            assert!(debris_arm(&e, &mut rng, 1000, 0x0700_0015, false).is_empty());
            assert_eq!(rng.draws - before, 0);
        }
        // Sister (can fail): the standard blast script passes the gate.
        let mut rng = crate::drone::Rng::new(5);
        let e = blast(6, 0x0600_0052);
        let before = rng.draws;
        let _ = debris_arm(&e, &mut rng, 1000, 0x0700_0015, false);
        assert_eq!(rng.draws - before, 1);
    }

    #[test]
    fn the_first_quarter_gate_and_the_modulo_gate() {
        // lifetime>>2 < age → return before the draw [M 0x17c350:31]; with lifetime 165 the arm
        // is dead from age 42 on [M 0x17cd18 the +0xAB0 u16]. The draw then happens EVERY gated
        // tick — consumed even when the modulo fails [M 0x17c350:34-38] — and the passing tick
        // spawns 3 particles whose 10 draws each ride the shared stream [M 0x1ac808 draws].
        let mut rng = crate::drone::Rng::new(9);
        for age in 42..=50u32 {
            let mut e = blast(6, 0x0600_0052);
            e.age = age;
            let before = rng.draws;
            assert!(debris_arm(&e, &mut rng, 7, 0x0700_0015, false).is_empty());
            assert_eq!(rng.draws - before, 0, "age {} is past the first quarter", age);
        }
        // Find a passing game_time with this seed (fresh stream per probe), then pin the counts.
        let mut passing = None;
        for t in 1..200u32 {
            let mut rng = crate::drone::Rng::new(11);
            let e = blast(6, 0x0600_0052);
            if !debris_arm(&e, &mut rng, t, 0x0700_0015, false).is_empty() { passing = Some(t); break; }
        }
        let t = passing.expect("a kind-6 blast must pass the modulo gate within 200 ticks");
        let mut rng = crate::drone::Rng::new(11);
        let e = blast(6, 0x0600_0052);
        let d = debris_arm(&e, &mut rng, t, 0x0700_0015, false);
        assert_eq!(d.len(), 3, "Debris_CreateEx count = 3 [M 0x17c350:66]");
        assert_eq!(rng.draws, 1 + 3 * 10, "the gate draw + 10 draws per particle [M 0x1ac808:25-41]");
        assert!(d.iter().all(|p| METAL_HASH_132.contains(&p.gfx)), "metal meshes for kind 6");
        assert!(d.iter().all(|p| (90..180).contains(&p.life)), "life = Rand(90) + 90 [M 0x1ac808:29-31]");
        // Sister (can fail): a different seed at the same tick — the draw stream differs, so the
        // replay is not vacuous (the port's rand is the ledger-replayed primitive,
        // tests/diff_calls.rs `Rand_Rand` 40 calls bit-exact).
        let mut rng = crate::drone::Rng::new(12);
        let e = blast(6, 0x0600_0052);
        let before = rng.draws;
        let d2 = debris_arm(&e, &mut rng, t, 0x0700_0015, false);
        assert!(rng.draws - before >= 1);
        let _ = d2;
    }

    #[test]
    fn stone_kinds_are_gated_on_level_and_channel() {
        // Kinds 8/10/0xB need GameState level 0x0700008 AND switch_channels[0x53] on [M 0x17c350:49-58];
        // kinds 7/9 and the unlisted kinds fall through the switch to nothing (after the draw) [M :46-48].
        for kind in [8u16, 10, 0xb] {
            let mut rng = crate::drone::Rng::new(7);
            let e = blast(kind, 0x0600_0052);
            let before = rng.draws;
            assert!(debris_arm(&e, &mut rng, 5, 0x0700_0015, true).is_empty(), "wrong level");
            assert_eq!(rng.draws - before, 1, "the draw still happens (the level gate is post-draw)");
            assert!(debris_arm(&e, &mut rng, 5, 0x0700_0008, false).is_empty(), "channel off");
            // The stone meshes on the one level where they can fire.
            let mut found = None;
            for t in 1..200u32 {
                let mut rng = crate::drone::Rng::new(21);
                if !debris_arm(&e, &mut rng, t, 0x0700_0008, true).is_empty() { found = Some(t); break; }
            }
            let t = found.expect("stone debris must be reachable on 0x0700008 with ch 0x53");
            let mut rng = crate::drone::Rng::new(21);
            let d = debris_arm(&e, &mut rng, t, 0x0700_0008, true);
            assert_eq!(d.len(), 3);
            assert!(d.iter().all(|p| STONE_HASH_133.contains(&p.gfx)));
        }
        for kind in [7u16, 9, 1, 12] {
            let mut rng = crate::drone::Rng::new(7);
            let e = blast(kind, 0x0600_0052);
            let before = rng.draws;
            assert!(debris_arm(&e, &mut rng, 5, 0x0700_0008, true).is_empty());
            assert_eq!(rng.draws - before, 1, "kind {} draws then falls through the switch", kind);
        }
    }
}
