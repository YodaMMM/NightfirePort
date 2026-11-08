//! The GUNTURRET special object — `GT_Create` 0x17ffa0 (class 0x36 '6' → `GT_Update` 0x180498,
//! docs/atlas/object.md) and the mounted-minigun deploy `GT_DeployMiniGun` 0x181560 [M both
//! decompiles].
//!
//! `GT_Create`'s callers [M]:
//! - `parsemap_create_dynamic_objects` 0x1d04f0 **case 0x27** (line 115, spawn-flag gated) —
//!   41 placements disc-wide: 0700000a ×6 (sky-bridge turrets), 0700000b ×5 (terminal/exec
//!   floor), 2 per MP map [D script-census.md:141; docs/levels/0700000a.md:109,
//!   docs/levels/0700000b.md:160].
//! - `GT_DeployMiniGun` 0x181560 ← `DroneWeap_FireWeapon` 0x178f68:123 (the guard weapon-datum
//!   char 'R' deploy, see below) and `Player_WeaponInitBullet` 0x1a4bf0:282 (the 0x52 briefcase).
//!
//! Creator behaviour [M 0017ffa0:69-160]:
//! - gate: rec+0x34 (param 2) == 0 or rec+0x38 (param 3) == 0 → return 0 (no turret);
//! - `Control_CreateObjEx(1.0, 0x100, pos, rot, 0, parent, 0, 1, 4)`; memset ext 0x100;
//!   obj+0xff = 0x36; obj+0xf0 |= 0x810 (bit 0x10 = hidden until deployed — the draw gate
//!   `crates/nf-view/src/scene.rs` already honours);
//! - ext+0xec (s16) = param 0 (the facing mode: 0 → `Mat_GetDir`, else `Mat_GetUp`+negate);
//!   ext+0xcc = 100.0f; ext+0xee (u16) = param 1; ext+0xe8 (u16) = param 4; ext+0xea (u16) =
//!   param 5; ext+0xb8 = the parent object (param_6) or the new object itself;
//!   ext+0xf0 (u16) = 200 when param 0 == 0, else 0xffff (HP / invulnerable);
//!   ext+0xf2 (u8) = 0; ext+0xe4 = 0;
//! - `Script_Load(param 2 → ext+0xb0)` and `Script_Load(param 3 → ext+0xb4)` with the
//!   per-object callback `GT_Register` 0x181538; either load failing → obj+0xfe |= 1 (the
//!   object deletes itself) and return 0;
//! - `Script_Play`/`Script_Update` both; obj+0xf4 = 0; the script-1 object bbox's max.z × 1.5
//!   → ext+0xd0 (the activation radius);
//! - MP-only (MPSettings != 0, param 0 == 0): deletes sibling kind-'6' turrets with the same
//!   ext+0xb8 parent via `Explode_Create(5.0, 5.0, 50.0, …, 0x6000052, …)`.
//!
//! The port stores the placement-derived record; the script VM (`Script_Load/Play/Update` per
//! object) and the turret behaviours (`GT_Update` 0x180498, `GT_Track` 0x17f9c8,
//! `GT_ApplyDamage` 0x1816d8, `GT_Deploy`/`TakeControl`) are a known gap: the turret records carry
//! the data those ports (and the draw code) need.
//!
//! `GT_DeployMiniGun` 0x181560 [M 00181560 — Ghidra spills the stack args; the flow]: copy the
//! parent matrix, `Collide_PlaceObject(0.5, 0.6, pos, &mat, parent)` → 0 = no room, return 0;
//! else `Mat_GetDir`+`Vec_DirToRot` → rot vec, `Quat_MatToQuat(mat)`, the glist
//! `hashtable_hashcode_to_celglist(0x2000359)` (the 02000359 minigun — not a placement's), then
//! `GT_Create(pos, rotvec, quat, &mat, celglist, parent)`. Its guard-AI caller is
//! `DroneWeap_FireWeapon` 0x178f68:92-145: when the guard's weapon datum char
//! (BOT_vars+0x768) == 'R', `BOTSTATE_getStateType() == 4` (combat), the weapon id not
//! 0xdd/0xde, `AmmoInGun >= 1` and not `tooCloseForWeapon` (+0x170 gate), the game deploys at
//! **(x + sin(yaw), y, z + sin(yaw + 1.5707964))** [M 00178f68:129-137], yaw = obj+0x54; on
//! success `BOTWEAP_decrRounds(…, 1000)` + `BOTSTATE_changeWeapon(…, 0, 1, 0)` and the normal
//! bullet path never runs. The port does not yet model the weapon datum char (BOT_vars+0x768),
//! so the fire_weapon branch cannot select yet; the deploy position math is ported here and
//! unit-tested.

/// The minigun glist hash `GT_DeployMiniGun` fetches [M 00181560].
pub const MINIGUN_GLIST: u32 = 0x2000_359;
/// `GT_DeployMiniGun`'s stack record for `GT_Create` [M 00181560; the record base is sp+0x70 —
/// `auStack_120` — so +0x2c = uStack_f4 = 0 (facing), +0x30 = uStack_f0 = 2 (param 1),
/// +0x34 = uStack_ec = 0x06000045 (script A), +0x38 = uStack_e8 = 0x06000044 (script B),
/// +0x3c = uStack_e4 = 0 (enable channel); disasm 0x1815e0-0x181614 writes sp+156..172 and never
/// +0x40 (sp+176, the disable channel — stack residue, read only by the facing-1 gates)]. Script A carries the gun (0x200035a base, 0x2000361 /
/// 0x2000362 parented to it, the three `GT_Register` events f(2)/f(0)/f(1)); script B the
/// suitcase halves 0x2000358 ×2 and the 0x2000359 body [D 0700000c.bin nis dump].
pub const RONIN_SCRIPT_A: u32 = 0x0600_0045;
pub const RONIN_SCRIPT_B: u32 = 0x0600_0044;
/// The weapon records: 0x52 deploys (`Player_WeaponInitBullet` 0x1a4bf0:265-282 — the pellet loop
/// calls `GT_DeployMiniGun` instead of `Bullet_init` for id 0x52), 0x53 takes control of the
/// deployed turret (`Player_WeaponInitBullet` :113-133 — the first class-'6' object whose owner
/// is the firing player → `GT_TakeControl`) [M].
pub const RONIN_DEPLOY: u8 = 0x52;
pub const RONIN_CONTROL: u8 = 0x53;
/// The port's owner id for "the player" (`glb_players`) — the same id the turret candidate list
/// gives Bond (`TrackCand::obj == u32::MAX`).
pub const PLAYER_OWNER: u32 = u32::MAX;
/// `Player_PositionCamera` 0x18f5d8 case 0xf's eye offset in the registered[0] object's frame:
/// `ApplyMatrixLVI(reg0+0x90, (0.0, 0.15, -0.4))` then + reg0+0xC0 [M 0x18f5d8:277-299].
pub const RONIN_EYE: [f32; 3] = [0.0, 0.15, -0.4];

/// `GT_DeployMiniGun` 0x181560 + `GT_Create` 0x17ffa0 for the Ronin: the record the creator
/// writes for `GT_DeployMiniGun`'s stack args (facing 0 → HP 200), owned by the player
/// (`param_6` = the firing player → ext+0xB8 [M 0017ffa0:79-81]). `pos` is the
/// `Collide_PlaceObject`-seated position, `fire_dir` the create-time `Mat_GetDir(obj+0x90)`
/// (facing 0) [M 0017ffa0:141-143].
pub fn from_deploy(pos: [f32; 3], fire_dir: [f32; 3]) -> GTurret {
    GTurret {
        placement: usize::MAX,
        facing: 0,
        param1: 2,
        script_a: RONIN_SCRIPT_A,
        script_b: RONIN_SCRIPT_B,
        param4: 0,
        param5: 0,
        hp: 200,
        owner: PLAYER_OWNER,
        pos,
        fire_dir,
        scripts_done: false,
        ..GTurret::default()
    }
}

/// `Collide_PlaceObject(0.5, 0.6, pos, mat, parent)` 0x1ecfb8 — `GT_DeployMiniGun`'s seat test
/// [M 0x1ecfb8]: `pos.y += 0.05`; a 3.0-long ray down the matrix's up axis (flags 0x1d) must hit;
/// the position moves to the hit + normal·0.5; then the pick around it (radius 0.6, the sweep
/// down to `pos − normal·0.1`) rejects any plane nearer than 0.5 − 0.13 (`|d| − 0.5 < −0.13` →
/// return 0), grows the support box from the floor-like triangles (`|d| − 0.5 ≤ 0.2` and
/// `|n_tri·n| ≥ 0.96`), and the box must reach ≥ 0.6 (|Δ·side| + |Δ·dir|) at BOTH corners.
/// Returns the seated position (the object's obj+0x30) or None (no room — `GT_DeployMiniGun`
/// returns 0 and the caller refunds the round).
///
/// The pick volume (`Collide_Pick` with HITTEST type 0xd, radius 0.6) is approximated by the
/// triangles whose closest point lies within 0.6 of the sweep segment [S — the game's pick
/// shape is not separately measured]; the triangle cap is the game's 0xa6 [M].
pub fn place_object(world: &crate::world::CollisionWorld, pos: [f32; 3], dir: [f32; 3]) -> Option<[f32; 3]> {
    const R: f32 = 0.5; // param_1 0x3f000000
    const EXT: f32 = 0.6; // param_2 0x3f19999a
    let up = [0.0f32, 1.0, 0.0]; // Mat_GetUp of the player's (upright) matrix
    let p0 = [pos[0], pos[1] + 0.05, pos[2]];
    let down = [p0[0] - up[0] * 3.0, p0[1] - up[1] * 3.0, p0[2] - up[2] * 3.0];
    let (t, n) = world.segment_hit(p0, down)?;
    let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
    let n = [n[0] / nl, n[1] / nl, n[2] / nl];
    let hit = [p0[0] + (down[0] - p0[0]) * t, p0[1] + (down[1] - p0[1]) * t, p0[2] + (down[2] - p0[2]) * t];
    let low = [p0[0] - n[0] * 0.1, p0[1] - n[1] * 0.1, p0[2] - n[2] * 0.1];
    let seat = [hit[0] + n[0] * R, hit[1] + n[1] * R, hit[2] + n[2] * R];
    let lo = [seat[0].min(low[0]) - EXT, seat[1].min(low[1]) - EXT, seat[2].min(low[2]) - EXT];
    let hi = [seat[0].max(low[0]) + EXT, seat[1].max(low[1]) + EXT, seat[2].max(low[2]) + EXT];
    let mut bmin = seat;
    let mut bmax = seat;
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let mut count = 0usize;
    for tri in world.tris_near(lo, hi) {
        if count >= 0xa6 { break; }
        // in the pick: the triangle comes within EXT of the sweep segment (three samples)
        let near = [seat, [(seat[0] + low[0]) * 0.5, (seat[1] + low[1]) * 0.5, (seat[2] + low[2]) * 0.5], low]
            .iter().any(|&q| point_tri_dist2(q, tri) <= EXT * EXT);
        if !near { continue; }
        count += 1;
        let tn = cross(sub(tri[1], tri[0]), sub(tri[2], tri[0]));
        let l = dot(tn, tn).sqrt();
        if l < 1e-9 { continue; }
        let tn = [tn[0] / l, tn[1] / l, tn[2] / l];
        let d = dot(tn, sub(seat, tri[0])).abs();
        if d - R < -0.13 { return None; }
        if d - R <= 0.2 && dot(tn, n).abs() >= 0.96 {
            for v in tri {
                for k in 0..3 { bmin[k] = bmin[k].min(v[k]); bmax[k] = bmax[k].max(v[k]); }
            }
        }
    }
    // Mat_Align2Up(mat, n, dir) → the seated frame's dir / side axes.
    let dn = dot(dir, n);
    let mut f = [dir[0] - n[0] * dn, dir[1] - n[1] * dn, dir[2] - n[2] * dn];
    let fl = dot(f, f).sqrt();
    if fl < 1e-6 { f = [0.0, 0.0, 1.0]; } else { f = [f[0] / fl, f[1] / fl, f[2] / fl]; }
    let side = cross(n, f);
    let reach = |c: [f32; 3]| { let v = sub(c, seat); dot(v, side).abs() + dot(v, f).abs() };
    if reach(bmin) >= EXT && reach(bmax) >= EXT { Some(seat) } else { None }
}

/// The squared distance from `p` to the triangle `t` (the closest-point region test).
fn point_tri_dist2(p: [f32; 3], t: [[f32; 3]; 3]) -> f32 {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    let at = |q: [f32; 3]| { let v = sub(p, q); dot(v, v) };
    if d1 <= 0.0 && d2 <= 0.0 { return at(a); }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 { return at(b); }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return at([a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v]);
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 { return at(c); }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return at([a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w]);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return at([b[0] + (c[0] - b[0]) * w, b[1] + (c[1] - b[1]) * w, b[2] + (c[2] - b[2]) * w]);
    }
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    at([a[0] + ab[0] * v + ac[0] * w, a[1] + ab[1] * v + ac[1] * w, a[2] + ab[2] * v + ac[2] * w])
}
/// The `Collide_PlaceObject` radii [M 00181560].
pub const DEPLOY_RADIUS: (f32, f32) = (0.5, 0.6);
/// The deploy yaw offset: sin(yaw + π/2) for the z term [M 00178f68:137].
pub const HALF_PI: f32 = 1.570_796_4;

/// One kind-0x27 placement's turret record (the ext fields the creator writes [M 0017ffa0]).
#[derive(Clone, Debug, PartialEq)]
pub struct GTurret {
    pub placement: usize,
    /// ext+0xec (s16): param 0 — the facing mode (0 = Mat_GetDir, else Mat_GetUp + negate) and
    /// the HP selector (0 → HP 200, else 0xffff) [M 0017ffa0:83-96,141-145].
    pub facing: i16,
    /// ext+0xee (u16): param 1 [M 0017ffa0:85-87].
    pub param1: u16,
    /// ext+0xb0: `Script_Load(param 2)` — zero means the load failed and the game deletes the
    /// object [M 0017ffa0:100-106].
    pub script_a: u32,
    /// ext+0xb4: `Script_Load(param 3)` [M 0017ffa0:103-105].
    pub script_b: u32,
    /// ext+0xe8 (u16): param 4 [M 0017ffa0:88].
    pub param4: u16,
    /// ext+0xea (u16): param 5 [M 0017ffa0:89-91].
    pub param5: u16,
    /// ext+0xf0 (u16): 200 when param 0 == 0, else 0xffff [M 0017ffa0:92-96]. `GT_Update`
    /// 0x180498 decrements it per shot (case 4: `if (0 < *(short *)(ext+0xf0)) --`) and its
    /// death gate destroys the turret when it reaches 0 — the shot counter, not a hit meter.
    pub hp: u16,
    /// obj+0xf4: the update state, 0 at creation [M 0017ffa0:117].
    pub state: u16,
    /// ext+0xcc (f32): health, 100.0 at create [M 0017ffa0:84]; `GT_ApplyDamage` 0x1816d8
    /// subtracts; `GT_Update`'s death gate fires at < 0.0 [M 00180498:236-244].
    pub health: f32,
    /// The damage on the scripts' objects' `obj+0xD0` chains, [script A, script B]: the bullet pass
    /// sums this tick's contacts (`Collide_GetDamageNObjects` 0x1ee220 with no output list) and
    /// the next `GT_Update` reads them [M 00180498:128-148 script A every tick, :152-176 script B
    /// only while the state is 4]; the game's Collide runs after the walk [M 0x1336f8:238].
    pub contact: [f32; 2],
    /// obj+0xf6 (s16): the previous state — `GT_Disable` 0x1816a8 stores 4 there and state 5
    /// (`GT_Update` case 5) returns to it after 3.0 s [M both].
    pub prev_state: u16,
    /// ext+0xdc (f32): the state timer (1.2 s restore, 0.5 s holster, 3.0 s sweep, 10 s wreck)
    /// [M 00180498 cases 2/4/5/6].
    pub timer: f32,
    /// ext+0xe0 (f32): the fire-interval timer, 0.1 s per shot [M 00180498 case 4].
    pub fire_timer: f32,
    /// ext+0xbc: the current target object id (`GT_Track` 0x17f9c8's return, stored in case 4)
    /// [M 00180498 case 4].
    pub target: u32,
    /// ext+0xb8: the owner object (`GT_Create`: placement parent 6 or the turret itself)
    /// [M 0017ffa0:90-92]. `GT_PlayerHasDied` 0x181808 matches it against the dead player;
    /// `GT_Track`'s MP branch skips it [M 0x17f9c8 MP branch].
    pub owner: u32,
    /// ext+0xc0/c4/c8: the ≤3 `GT_Register` 0x181538 script-object slots (0xc4 = the barrel
    /// whose matrix spins) [M 00181538, 00180498 case 4 fire].
    pub registered: [u32; 3],
    /// registered[0]'s obj+0xF4 (u16): the muzzle-flash countdown — every shot writes 2, the
    /// update head counts it down [M 00180498:110-122, case 4 / case 7 fire].
    pub flash: u16,
    /// registered[0]'s obj+0xF6 (u8 as bool): the barrel alternator — every shot toggles it and
    /// the shot smoke fires when the toggled value is 1 [M disasm 0x180bd8-0x180bf8]. Fresh
    /// objects read 0, so the FIRST shot smokes.
    pub barrel_alt: bool,
    /// registered[0]'s obj+0xF0 bit 0x10 (hidden): set by the update head when the countdown is
    /// (or reaches) 0, cleared by every shot [M 00180498:117-120, case 4 / case 7 fire]. Live
    /// [L 0700000c deployed Ronin under remote control, no fire: reg[0] (0x2000361) obj+0xF0 =
    /// 0x231, the other script-A/B parts 0x1080/0x10c0].
    pub flash_hidden: bool,
    /// ext+0xf2 (u8): mounted — set by `GT_TakeControl` 0x181740, cleared by
    /// `GT_LoseControl` 0x181408 [M both].
    pub controlled: u8,
    /// obj+0xfe bit 0: the self-delete request (case 3 facing-1 drop, MP case-6 wreck,
    /// `GT_Create`'s own failure path) [M 0017ffa0:118, 00180498].
    pub delete_flag: bool,
    /// ext+0x90 (3×f32): the mounted aim vector — ext+0x90 pitch clamped ±0.5, ext+0x94 yaw
    /// clamped ±1.5707964 [M 00180498 case 7].
    pub aim: [f32; 3],
    /// ext+0xd4/d8: the barrel spin angle and velocity [M 00180498 case 4/7].
    pub spin: f32,
    pub spin_vel: f32,
    /// The angle this tick's spin block rotated registered[1] (the barrel) by — `RotMatrixZ(wrapped
    /// ext+0xD4)` concatenated onto the barrel's script pose, obj+0x90 = RotZ · pose [M 00180498:365-382,
    /// 538-556; L PCSX2 audits/render-1101-roninview: barrel row0 (−0.921, −0.388) = RotZ(−2.741) of the
    /// script-A pose, ext+0xD4 −2.741]. None on a tick the block did not run: the barrel shows its
    /// script pose (the angle is absolute, re-applied every tick, not accumulated in the matrix).
    pub spin_draw: Option<f32>,
    /// ext+0xd0: the muzzle offset along the barrel (GT_Create's script bbox max.z × 1.5 —
    /// the port's script VM is the tracked gap, so it stays a settable 0.0 from placements)
    /// [M 0017ffa0:155-158].
    pub muzzle_offset: f32,
    /// The two scripts' per-object done flag (obj+0x9c on each `Script_GetObj` record): the
    /// game's script VM sets it; the port's script VM is the tracked gap (gt.rs module doc),
    /// so the state machine carries it here — true = both scripts played through [S] the
    /// default keeps the port's turrets arming (the shipped scripts do finish).
    pub scripts_done: bool,
    /// obj+0x30: the turret position — the tick copies the placement translation in
    /// (`GT_Track`'s distances and the muzzle point read obj+0x30 [M 0x17f9c8:99-118]).
    pub pos: [f32; 3],
    /// The port's script-player indices for the two `Script_Load`s (the game's SCRIPTINFO
    /// pointers ext+0xb0/b4; the port runs them as `Scripts::players` entries). `usize::MAX` =
    /// not mounted. `GT_Update`'s every-tick `Script_Update` pair and its +0x9c
    /// reads/clears go through these [M 00180498:111-112, 223-228].
    pub script_player_a: usize,
    pub script_player_b: usize,
    /// ext+0x80: the create-time fire direction (facing 0 → `Mat_GetDir(obj+0x90)`, else
    /// `Mat_GetUp` negated) — `GT_Track`'s front-cone dot test reads it [M 0017ffa0:141-147,
    /// 0x17f9c8:252-257]. The port keys it from the placement rotation.
    pub fire_dir: [f32; 3],
    /// The drawn gun pose (yaw, pitch) script A is posed at — `ease_pose`; None = the create pose.
    pub pose: Option<(f32, f32)>,
    /// The two script players' `placement` key (the WireRegister/event-0xF hand-off matches it):
    /// the placement index for the placed turrets, a unique runtime key for a deployed Ronin.
    pub sp_key: usize,
}

fn param(params: &[(u32, u32)], k: u32) -> u32 {
    params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

/// `GT_Create` 0x17ffa0 from one kind-0x27 placement: None when the gate fails (param 2/3 == 0
/// → the game returns 0 before creating anything [M 0017ffa0:69]).
pub fn from_placement(placement: usize, params: &[(u32, u32)]) -> Option<GTurret> {
    let script_a = param(params, 2);
    let script_b = param(params, 3);
    if script_a == 0 || script_b == 0 {
        return None;
    }
    let facing = param(params, 0) as u16 as i16;
    Some(GTurret {
        placement,
        facing,
        param1: param(params, 1) as u16,
        script_a,
        script_b,
        param4: param(params, 4) as u16,
        param5: param(params, 5) as u16,
        hp: if facing == 0 { 200 } else { 0xffff },
        state: 0,
        health: 100.0,
        contact: [0.0; 2],
        prev_state: 0,
        timer: 0.0,
        fire_timer: 0.0,
        target: 0,
        owner: 0,
        registered: [0; 3],
        flash: 0,
        barrel_alt: false,
        flash_hidden: false,
        controlled: 0,
        delete_flag: false,
        aim: [0.0; 3],
        spin: 0.0,
        spin_vel: 0.0,
        spin_draw: None,
        muzzle_offset: 0.0,
        scripts_done: true,
        pos: [0.0; 3],
        script_player_a: usize::MAX,
        script_player_b: usize::MAX,
        fire_dir: [0.0, 0.0, 1.0],
        pose: None,
        sp_key: placement,
    })
}

/// `GT_DeployMiniGun` 0x181560's deploy position, as `DroneWeap_FireWeapon` 0x178f68:129-137
/// computes it before the call: (x + sin(yaw), y, z + sin(yaw + 1.5707964)), yaw = obj+0x54.
pub fn deploy_position(pos: [f32; 3], yaw: f32) -> [f32; 3] {
    [
        pos[0] + crate::trig::sinf(yaw),
        pos[1],
        pos[2] + crate::trig::sinf(yaw + HALF_PI),
    ]
}

/// The state ids (obj+0xf4) [M 00180498 switch].
pub mod gt_state {
    pub const DORMANT: u16 = 0;
    pub const ARMING: u16 = 1;
    pub const RESTORING: u16 = 2;
    pub const FINISHED: u16 = 3;
    pub const COMBAT: u16 = 4;
    pub const DISABLED_SWEEP: u16 = 5;
    pub const DESTROYED: u16 = 6;
    pub const MOUNTED: u16 = 7;
}

/// The state-3 facing-1 drop: `Pickup_Create(…, hashtable_hashcode_to_celglist(0x2000366), …)`
/// [M 00180498 case 3].
pub const PICKUP_GLIST: u32 = 0x2000_366;
/// The smoke glist both the fire-path muzzle puff and the wreck smoke pass to `Gas_Init`
/// [M 00180498 case 4 fire / case 6].
pub const SMOKE_GLIST: u32 = 0x2000_039;
/// The turret's `Bullet_init` weapon-definition pointer 0x2c55d0 as a `weapon_data` record
/// index: the table sits at EE 0x2bf150 in 0x10c-byte records [M
/// docs/systems/drone-weapon-drop.md:10], (0x2c55d0 − 0x2bf150) / 0x10c = 96 exactly.
pub const TURRET_WEAPON_RECORD: usize = 96;
/// The mounted aim scale — the literal 0.024543693 (0x3cc90fdb) at case 7
/// (`-input * 0.024543693 * FRAME_RATE_MUL`) [M 00180498 case 7].
pub const AIM_INPUT_SCALE: f32 = f32::from_bits(0x3cc9_0fdb);
/// The muzzle `Sound_Play3D` ids: 0x4e7 for facing-1 turrets, 0x10a otherwise [M 00180498 case 4].
pub const SOUND_FIRE_WALL: u32 = 0x4e7;
pub const SOUND_FIRE: u32 = 0x10a;

/// One `GT_Track` candidate. The caller (the tick's object walk) applies the game's
/// alive/type/health filters first — the decompile reads guard ext+0x4f8 flags (& 0x600
/// clear, & 0x100 set), ext+0xac health > 0 and obj+0xff ≠ 0x11 [M 0x17f9c8:118-136], and
/// the player's health/flag pair [M 0x17f9c8:99-110].
#[derive(Clone, Copy, Debug)]
pub struct TrackCand {
    /// The object id (the `TargetList_144` entry).
    pub obj: u32,
    /// The candidate position (obj+0x30).
    pub pos: [f32; 3],
    /// A copter: its distance is weighted ×0.25 before the range compare [M 0x17f9c8:158-166].
    pub copter: bool,
    /// `Collide_LineOfSight(muzzle, cand+0x80, level, obj, cand, 0x72f) != 0`
    /// [M 0x17f9c8:271-278] — the tick's LOS walk.
    pub los_ok: bool,
}

/// Everything one `GT_Update` tick reads beyond the record itself.
pub struct GTCtx<'a> {
    pub channels: &'a crate::triggers::Channels,
    /// `MPSettings != 0` [M 00180498 case 6 / GT_Track 0x17f9c8:92].
    pub mp: bool,
    /// `GameState+0x52` — the %10 smoke gate in state 6 [M 00180498 case 6].
    pub frame: u32,
    /// The callers' filtered target candidates (see `TrackCand`).
    pub cands: &'a [TrackCand],
    /// The mounted inputs (case 7): (yaw axis, pitch axis, fire held, exit pressed).
    pub mount: Option<(f32, f32, bool, bool)>,
}

/// What one `GT_Update` tick produced — the data/flow half of the draw/sound work (the view
/// side draws and plays; these are the exact game calls [M 00180498]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GTEvent {
    /// The top-of-update hide gate: registered[0]'s countdown (obj+0xF4) at or reaching 0 → that object's
    /// obj+0xf0 gets bit 0x10 (hidden) [M 00180498:110-122].
    Registered0Hidden,
    /// The death-gate burst: `Emitter_Create(10.0, 0, -2.0, base, up)` [M 00180498:249-257].
    DestroyedBurst { pos: [f32; 3] },
    /// `Music_Event(2, 1)` while a target exists [M 00180498 case 4].
    CombatSting,
    /// One shot: `Bullet_init(…, weapon 0x2c55d0, muzzle, dir, 0xffff)` +
    /// `Sound_Play3D(100.0, -1.0, -1.0, 0x4e7|0x10a, muzzle)` [M 00180498 case 4 / case 7].
    Fire { muzzle: [f32; 3], dir: [f32; 3], sound: u32 },
    /// The shot smoke: the fire tail's barrel byte toggle (`+0xf6 ^= 1` on the flash object,
    /// re-arming `+0xf4 = 2` / unhiding `+0xf0 &= ~0x10` at the same point) calls
    /// `Gas_Init(0.05, 0.1, cel, muzzle, rot, NULL-vel, 0x2000039, 7, 0x48403080, 1)` on every
    /// OTHER shot — when the toggled byte is 1 [M disasm 0x180bc8-0x180c3c / 0x181304-0x181350].
    /// The muzzle is the same vector the round spawned from.
    ShotSmoke { muzzle: [f32; 3] },
    /// The wreck smoke: `Gas_Init(0.05, 0.25, …, 0x2000039, 7, 0x48403080, 1)` on the %10
    /// gate [M 00180498 case 6].
    Smoke { pos: [f32; 3] },
    /// `Pickup_Create(…, 0x2000366, …)` + self-delete (state 3, facing 1) [M case 3].
    DropPickup { pos: [f32; 3] },
    /// The MP wreck: `Explode_Create(5.0, 5.0, 50.0, …, 0x6000052, …, 100)` + self-delete
    /// [M 00180498 case 6].
    Exploded { pos: [f32; 3] },
    /// `GT_RandMove`'s two shared-RNG draws [M 00180348:44-45]; the sweep direction itself
    /// is a port stand-in ([S] unverified — the game rotates the aim matrix by
    /// `RotMatrix(rand)` before the look-at; the draw side owns that matrix).
    Sweep { yaw: f32, pitch: f32 },
    /// The state machine's two `SCRIPTINFO+0x9c = 0` writes — the DORMANT tail (case 0:
    /// both scripts' delays cleared so the looping deploy anim replays) and the RESTORING
    /// expiry (case 2's 1.2 s: both cleared for the fold) [M 00180498:223-227, 256-261].
    /// The caller zeroes the mounted script players' `Script::delay` (the +0x9c model).
    RewindScripts,
}

impl GTurret {
    /// `GT_ApplyDamage` 0x1816d8: `ext+0xcc -= dmg`.
    pub fn apply_damage(&mut self, dmg: f32) {
        self.health -= dmg;
    }

    /// `GT_Disable` 0x1816a8: state 4 only — store 4 in obj+0xf6, state 5, ext+0xdc = 0.
    pub fn disable(&mut self) {
        if self.state == gt_state::COMBAT {
            self.prev_state = self.state;
            self.state = gt_state::DISABLED_SWEEP;
            self.timer = 0.0;
        }
    }

    /// `GT_Activate` 0x1816f0: states 0/1/2/3/6 ignore; the rest with facing 0 re-holster
    /// (ext+0xdc = 0, state 2).
    pub fn activate(&mut self) {
        match self.state {
            gt_state::DORMANT
            | gt_state::ARMING
            | gt_state::RESTORING
            | gt_state::FINISHED
            | gt_state::DESTROYED => {}
            _ => {
                if self.facing == 0 {
                    self.timer = 0.0;
                    self.state = gt_state::RESTORING;
                }
            }
        }
    }

    /// `GT_TakeControl` 0x181740: state 4 only — state 7, ext+0xf2 = 1. Returns whether the
    /// mount happened (the caller does the player's substate/cam/weapon-none plumbing).
    pub fn take_control(&mut self) -> bool {
        if self.state == gt_state::COMBAT {
            self.state = gt_state::MOUNTED;
            self.controlled = 1;
            true
        } else {
            false
        }
    }

    /// `GT_LoseControl` 0x181408: ext+0xf2 = 0; state 7 → 4 [M 00181408:66-73].
    pub fn lose_control(&mut self) {
        self.controlled = 0;
        if self.state == gt_state::MOUNTED {
            self.state = gt_state::COMBAT;
        }
    }

    /// `GT_Delete` 0x181678: `Script_Free` both scripts — the port drops the record handles.
    pub fn delete(&mut self) {
        self.script_a = 0;
        self.script_b = 0;
    }

    /// `GT_Register` 0x181538: `base+0xc0 + (idx & 0xffff)*4 = obj` for idx < 3 and obj != 0.
    /// Returns whether the slot took the object.
    pub fn register(&mut self, idx: u16, obj: u32) -> bool {
        if obj != 0 && (idx & 0xffff) < 3 {
            self.registered[(idx & 0xffff) as usize] = obj;
            true
        } else {
            false
        }
    }

    /// fx-gas-and-blasts row 10 [M GT_Create 0x17ffa0:150-166]: after a turret enters the list,
    /// every OTHER class-6 object whose ext+0xb8 equals the new turret's owner is self-deleted
    /// (`obj+0xfe |= 1`) with the `Explode_Create(5.0, 5.0, 50.0, …, 0x6000052, …, 100)` blast at
    /// its own +0x30. Returns the blown turrets' positions (the caller queues the blasts).
    pub fn create_chain(turrets: &mut [GTurret], new_idx: usize) -> Vec<[f32; 3]> {
        let Some(new) = turrets.get(new_idx) else { return Vec::new() };
        let owner = new.owner;
        let mut blown = Vec::new();
        for (i, t) in turrets.iter_mut().enumerate() {
            if i != new_idx && !t.delete_flag && t.owner == owner {
                t.delete_flag = true;
                blown.push(t.pos);
            }
        }
        blown
    }

    /// `GT_PlayerHasDied` 0x181808: every state-7 turret whose owner is the dead object
    /// loses control [M 00181808 — the object-list walk collapses to the turret list].
    pub fn player_has_died(turrets: &mut [GTurret], dead_owner: u32) {
        for t in turrets.iter_mut() {
            if t.state == gt_state::MOUNTED && t.owner == dead_owner {
                t.lose_control();
            }
        }
    }

    /// `GT_Track` 0x17f9c8's selection core over the caller-filtered candidates.
    ///
    /// Range: the game compares the SQUARED distance (`Vec_SqDist3D`) against 2500.0 SP / 500.0 MP
    /// [M 0x17f9c8:92-99,178] — 50 units in SP (review 2026-10-21: the branch squared the thresholds
    /// to 6250000 / 250000, a 2500-unit reach). Drones are candidates only for facing-0 turrets
    /// [M :101 `sVar1 == 0`, the copter walk too :119]; the wall guns (facing 1) track Bond only. Copter
    /// distance ×0.25 [M :158-166]; cap 63 (the walk breaks once the count passes 0x3e)
    /// [M :146,170]; sorted by distance, the first two with LOS remembered, the current
    /// target returned while it is one of them, else the first LOS candidate, else none
    /// [M :246-300]. The current-target validity re-check (dead flags clear ext+0xbc,
    /// [M 0x17f9c8:66-88]) is the caller's alive-filter here: a target missing from `cands`
    /// is cleared.
    pub fn track(&mut self, cands: &[TrackCand], mp: bool) -> Option<u32> {
        let range2 = if mp { 500.0f32 } else { 2500.0f32 };
        if self.target != 0 && !cands.iter().any(|c| c.obj == self.target) {
            self.target = 0;
        }
        let mut list: Vec<(u32, f32, bool)> = Vec::new();
        for c in cands {
            if mp && c.obj == self.owner {
                continue; // the MP branch skips the owner [M 0x17f9c8:186]
            }
            // SP: a turret whose owner (ext+0xB8) IS `glb_players` — the Ronin Bond deployed —
            // never adds the player candidate (`if (*(int *)(param_1 + 0xb8) == glb_players)`
            // skips the whole player block) [M 0x17f9c8:62-64]. The port's player candidate is
            // obj u32::MAX and a player-deployed turret's owner is PLAYER_OWNER (= u32::MAX).
            if !mp && self.owner == PLAYER_OWNER && c.obj == PLAYER_OWNER {
                continue;
            }
            if !mp && self.facing != 0 && (c.obj != u32::MAX || c.copter) {
                continue; // SP: drones/copters only for facing-0 turrets [M 0x17f9c8:101,119]
            }
            let dx = c.pos[0] - self.pos[0];
            let dy = c.pos[1] - self.pos[1];
            let dz = c.pos[2] - self.pos[2];
            let mut d2 = dx * dx + dy * dy + dz * dz;
            if c.copter {
                d2 *= 0.25;
            }
            // The front-cone test [M 0x17f9c8:252-257]: the candidate's normalised direction
            // from the script base dotted with the create-time fire direction (ext+0x80,
            // `Mat_GetDir`/`Mat_GetUp`-negated of the placement matrix [M 0017ffa0:141-147])
            // must reach 0.01 — a candidate behind the turret's facing is not shootable.
            let m = d2.sqrt();
            if m > 0.0 {
                let dot = self.fire_dir[0] * (dx / m) + self.fire_dir[1] * (dy / m) + self.fire_dir[2] * (dz / m);
                if dot < 0.01 {
                    continue;
                }
            }
            if d2 < range2 && list.len() < 0x3f {
                list.push((c.obj, d2, c.los_ok));
            }
        }
        if list.is_empty() {
            self.target = 0;
            return None;
        }
        // CompTargetObj 0x1d3490 [M] — QuickSort comparator id 6, the GT_Track 0x17f9c8 sort at 0x17fe38 (key = f32 squared distance at +4, ascending); this sort replaces it.
        list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut first: Option<u32> = None;
        let mut second: Option<u32> = None;
        for &(obj, _, los) in &list {
            if los {
                if first.is_none() {
                    first = Some(obj);
                } else {
                    second = Some(obj);
                    break;
                }
            }
        }
        let cur = self.target;
        if cur != 0 && (first == Some(cur) || second == Some(cur)) {
            return Some(cur);
        }
        self.target = first.unwrap_or(0);
        first
    }

    /// The mounted aim step (`GT_Update` case 7): `slot += -input * 0.024543693`, x
    /// (pitch, ext+0x90) clamped ±0.5, y (yaw, ext+0x94) clamped ±1.5707964 [M case 7].
    pub fn mount_aim(&mut self, yaw_input: f32, pitch_input: f32) {
        self.aim[0] += -pitch_input * AIM_INPUT_SCALE;
        self.aim[1] += -yaw_input * AIM_INPUT_SCALE;
        if self.aim[0] < -0.5 { self.aim[0] = -0.5; }
        if self.aim[0] > 0.5 { self.aim[0] = 0.5; }
        let half_pi = f32::from_bits(0x3fc9_0fdb); // 1.5707964
        if self.aim[1] < -half_pi { self.aim[1] = -half_pi; }
        if self.aim[1] > half_pi { self.aim[1] = half_pi; }
    }

    /// The create-time yaw of the turret (`Vec_DirToRot` of the fire direction, the rot the
    /// creator built obj+0x90 from [M 00181560 → 0017ffa0:69-71]).
    pub fn base_yaw(&self) -> f32 {
        crate::trig::atan2f_libm(self.fire_dir[0], self.fire_dir[2])
    }

    /// The mounted aim frame [M 00180498 case 7]: obj+0x90 · RotY(ext+0x94) · RotX(ext+0x90)
    /// (`Concat(src, dst)` = dst = src·dst), at the turret position. Rows: side, up, forward —
    /// RotX(p) maps +Z to (0, −sin p, cos p), the gunimp barrel's convention.
    pub fn aim_matrix(&self) -> nf_formats::anim::Mat4 {
        let yaw = self.base_yaw() + self.aim[1];
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = self.aim[0].sin_cos();
        [
            [cy, 0.0, -sy, 0.0],
            [sy * sp, cp, cy * sp, 0.0],
            [sy * cp, -sp, cy * cp, 0.0],
            [self.pos[0], self.pos[1], self.pos[2], 1.0],
        ]
    }

    /// The gun's drawn pose (`Script_SetPosRot(ext+0xB0, …)` — case 2's slerp back to ext+0x40,
    /// case 4's slerp onto the look-at of the target, case 7's onto `aim_matrix`), as (yaw, pitch)
    /// eased by REC_FRAME_RATE·4 per tick toward `want` (`Quat_Slerp_Acc(REC_FRAME_RATE·4.0, …)`
    /// [M 00180498 cases 2/4/7]; the angle ease stands in for the quaternion slerp [S]).
    pub fn ease_pose(&mut self, want: (f32, f32)) {
        let k = crate::grenade::REC_FRAME_RATE * 4.0;
        let (py, pp) = self.pose.unwrap_or((self.base_yaw(), 0.0));
        let mut dy = want.0 - py;
        while dy > core::f32::consts::PI { dy -= 2.0 * core::f32::consts::PI; }
        while dy < -core::f32::consts::PI { dy += 2.0 * core::f32::consts::PI; }
        self.pose = Some((py + dy * k, pp + (want.1 - pp) * k));
    }

    /// The pose matrix script A is posed at (`ease_pose`'s angles, the turret position).
    pub fn pose_matrix(&self) -> nf_formats::anim::Mat4 {
        let (yaw, pitch) = self.pose.unwrap_or((self.base_yaw(), 0.0));
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        [
            [cy, 0.0, -sy, 0.0],
            [sy * sp, cp, cy * sp, 0.0],
            [sy * cp, -sp, cy * cp, 0.0],
            [self.pos[0], self.pos[1], self.pos[2], 1.0],
        ]
    }

    /// Each shot's flash write [M 00180498 case 4 / case 7, after `Bullet_init`]: when
    /// registered[0] is set, `reg0+0xF4 = 2` and `reg0+0xF0 &= ~0x10` (the obj+0xF6 toggle and
    /// its smoke puff are not modelled).
    /// The spin block [M 00180498:365-382, 538-556]: ext+0xD4 wrapped into (−π, π) with
    /// `fmodf(a, 2π)` (only when |a| ≥ π, and again shifted by 2π if still |a| ≥ π), the barrel drawn
    /// at that angle, then ext+0xD4 += `inc`.
    fn spin_advance(&mut self, inc: f32) {
        const PI: f32 = 3.141_592_7;
        const TAU: f32 = 6.283_185_5;
        let a0 = self.spin;
        let mut a = a0;
        if PI <= a0.abs() {
            a = a0 % TAU;
            if PI <= a.abs() { a = if 0.0 <= a0 { a - TAU } else { a + TAU }; }
        }
        self.spin_draw = Some(a);
        self.spin = a + inc;
    }

    fn show_flash(&mut self) {
        if self.registered[0] != 0 {
            self.flash = 2;
            self.flash_hidden = false;
        }
    }

    /// `GT_Update` 0x180498 — one tick (`dt` = `REC_FRAME_RATE`, 1/60 [M every timer step]).
    pub fn update(&mut self, ctx: &GTCtx, rng: &mut crate::drone::Rng, out: &mut Vec<GTEvent>) {
        self.spin_draw = None;
        // The registered[0] hide gate [M 00180498:110-122]: `s = reg0+0xF4; if s == 0 ||
        // (reg0+0xF4 = s − 1, s == 1) { reg0+0xF0 |= 0x10 }` — the muzzle flash shows on a
        // shot's tick and the next one, then hides.
        if self.registered[0] != 0 {
            let s = self.flash;
            if s != 0 { self.flash = s - 1; }
            if s == 0 || s == 1 {
                self.flash_hidden = true;
                out.push(GTEvent::Registered0Hidden);
            }
        }
        // The contact sums [M 00180498:128-176]: script A always, script B in COMBAT only. The
        // decompile's stun test (id 0x4A/0x4C → `GT_Disable`) reads the class byte of the SCRIPT
        // OBJECT, not of the contact's hitter (`iVar10 = lVar7` then `iVar10 + 0xff`), and a
        // script object is never class 5, so that arm does not fire on a bullet.
        let contact = std::mem::take(&mut self.contact);
        self.health -= contact[0];
        if self.state == gt_state::COMBAT { self.health -= contact[1]; }
        // The death gate [M 00180498:236-258]: health < 0 or the shot counter at 0 → 6.
        if (self.health < 0.0 || self.hp == 0) && self.state != gt_state::DESTROYED {
            if self.state == gt_state::MOUNTED {
                self.lose_control();
            }
            self.state = gt_state::DESTROYED;
            self.timer = 0.0;
            out.push(GTEvent::DestroyedBurst { pos: self.pos });
        }
        match self.state {
            gt_state::DORMANT => {
                if self.facing == 1 {
                    // enable channel (ext+0xe8) 0 or ON; disable (ext+0xea) 0 or OFF [M case 0].
                    if self.param4 != 0 && !ctx.channels.on(self.param4 as u32) {
                        return;
                    }
                    if self.param5 != 0 && ctx.channels.on(self.param5 as u32) {
                        return;
                    }
                    if self.track(ctx.cands, ctx.mp).is_none() {
                        return;
                    }
                }
                // both scripts' done flags reset (the +0x9c = 0 write — the looping deploy
                // replays), timer 0, → 1 [M case 0 tail: 0x180498:222-229].
                self.scripts_done = false;
                out.push(GTEvent::RewindScripts);
                self.timer = 0.0;
                self.state = gt_state::ARMING;
            }
            gt_state::ARMING => {
                // waits for both scripts' obj+0x9c [M case 1].
                if self.scripts_done {
                    self.state = gt_state::COMBAT;
                }
            }
            gt_state::RESTORING => {
                self.timer += crate::grenade::REC_FRAME_RATE;
                if self.timer > 1.2 {
                    self.timer = 0.0;
                    // both scripts' +0x9c = 0 — the fold anims replay [M 0x180498:256-261].
                    self.scripts_done = false;
                    out.push(GTEvent::RewindScripts);
                    self.state = gt_state::FINISHED;
                }
            }
            gt_state::FINISHED => {
                if self.scripts_done {
                    // [M 0x180498 case 3, disasm 0x180914-0x18098c]: a mounted turret loses
                    // control first (`lbu v0,242(s2)` / `jal 0x181408`), then the FACING test
                    // `lhu v0,236(s2)` + `bnel v0,zero` (delay slot `sh zero,244(s3)` = state 0,
                    // annulled when the branch is not taken): facing ≠ 0 → back to DORMANT;
                    // facing == 0 (the deployed Ronin, the MP floor turrets) → `Pickup_Create(
                    // obj+0xC0, 0, obj+0x90, celglist 0x2000366, kind 0, id 0x52 (t1 = 82),
                    // rounds 1 (t2), …, sound 0xF5 (sp+0 = 245))` + obj+0xFE |= 1 — the case
                    // goes back into the player's hands. (The branch had the two arms swapped:
                    // the facing-1 wall guns of 0a/0b deleted themselves after their first
                    // re-holster, and a folded Ronin re-armed in place.)
                    if self.controlled != 0 {
                        self.lose_control();
                    }
                    if self.facing == 0 {
                        out.push(GTEvent::DropPickup { pos: self.pos });
                        self.delete_flag = true;
                    } else {
                        self.state = gt_state::DORMANT;
                    }
                }
            }
            gt_state::COMBAT => {
                let had = self.track(ctx.cands, ctx.mp);
                if had.is_some() {
                    // A tracked frame zeroes the re-holster timer +0xDC [M 0x180498:293] (review
                    // 2026-10-21: the branch let it accumulate across target gaps).
                    self.timer = 0.0;
                    out.push(GTEvent::CombatSting);
                    // case 4's per-tick aim ease runs BEFORE the fire block: `Mat_LookAt(obj+0x30 →
                    // target+0x30)` slerped by `Quat_Slerp_Acc(REC_FRAME_RATE·4.0, …)` into obj+0x90
                    // and the barrel objects ext+0xc0/c4/c8 [M 0x180498:295-330] — and the fire
                    // block below reads THIS tick's eased barrel matrix.
                    if let Some(c) = ctx.cands.iter().find(|c| c.obj == self.target) {
                        let (dx, dy, dz) = (c.pos[0] - self.pos[0], c.pos[1] - self.pos[1], c.pos[2] - self.pos[2]);
                        let h = (dx * dx + dz * dz).sqrt();
                        self.ease_pose((crate::trig::atan2f_libm(dx, dz), -(dy.atan2(h))));
                    }
                    // fire cadence: ext+0xe0 += dt; > 0.1 → shoot [M case 4].
                    self.fire_timer += crate::grenade::REC_FRAME_RATE;
                    if self.fire_timer > 0.1 {
                        self.fire_timer = 0.0;
                        if self.hp > 0 {
                            self.hp -= 1;
                            // THE ROUND FLIES ALONG THE EASED BARREL, not at the target:
                            // `Mat_GetDir(iVar3 + 0x90)` — ext+0xc4's object matrix, just written
                            // from the slerp — feeds BOTH the muzzle offset and `Bullet_init`'s
                            // direction [M 0x180498:344-352]. The port's pose is that slerp (the
                            // same `Quat_Slerp_Acc(REC_FRAME_RATE·4.0)` ease; the (yaw, pitch)
                            // pair stands in for the quaternion [S]); its forward is
                            // `(sin yaw·cos pitch, −sin pitch, cos yaw·cos pitch)` [M pose_matrix
                            // row 2]. Firing dead-on at the target was the 0700000b corridor
                            // death: the game's lagging barrel (a 6.67%/tick first-order ease)
                            // sweeps behind a runner and misses — the wall guns landed 42/42
                            // rounds dead-on there ([L] the walkreal_0700000b_live real-damage
                            // log; every hit −2.40 at 3-5-tick gaps = the 2-turret 0.1 s
                            // cadence with a 100% hit rate).
                            let (yaw, pitch) = self.pose.unwrap_or((self.base_yaw(), 0.0));
                            let (sy, cy) = yaw.sin_cos();
                            let (sp, cp) = pitch.sin_cos();
                            let dir = [sy * cp, -sp, cy * cp];
                            let muzzle = [
                                self.pos[0] + dir[0] * self.muzzle_offset,
                                self.pos[1] + dir[1] * self.muzzle_offset,
                                self.pos[2] + dir[2] * self.muzzle_offset,
                            ];
                            let sound = if self.facing == 1 { SOUND_FIRE_WALL } else { SOUND_FIRE };
                            out.push(GTEvent::Fire { muzzle, dir, sound });
                            // The fire tail's barrel toggle + the alternate-shot smoke [M disasm
                            // 0x180bd0-0x180c3c]: `+0xf6 ^= 1` (with `+0xf4 = 2`, `+0xf0 &= ~0x10`
                            // — show_flash re-arms those), smoke when the toggled byte is 1.
                            self.barrel_alt = !self.barrel_alt;
                            if self.barrel_alt { out.push(GTEvent::ShotSmoke { muzzle }); }
                            self.show_flash();
                        }
                    }
                    // The tracked frame spins the barrel at a fixed 2 rev/s — ext+0xD4 += 2·2π·dt,
                    // ext+0xD8 untouched [M 00180498:360-382] (registered[1] = 0 → the case returns).
                    if self.registered[1] != 0 {
                        self.spin_advance(crate::grenade::REC_FRAME_RATE * 6.2831855 + crate::grenade::REC_FRAME_RATE * 6.2831855);
                    }
                } else if self.facing == 1 {
                    // re-holster path [M case 4 tail: the channel gate, 0.5 s → state 2].
                    // [M 0x180498:401-414]: ext+0xe8 == 0 → count; e8 ON → count only when ext+0xea is
                    // set AND on (ea == 0 or ea OFF → return); e8 OFF → count. (Review 2026-10-21: the
                    // branch opened the gate for e8 ON with ea == 0.)
                    let gate_open = if self.param4 == 0 {
                        true
                    } else if ctx.channels.on(self.param4 as u32) {
                        self.param5 != 0 && ctx.channels.on(self.param5 as u32)
                    } else {
                        true
                    };
                    if gate_open {
                        self.timer += crate::grenade::REC_FRAME_RATE;
                        if self.timer > 0.5 {
                            self.state = gt_state::RESTORING;
                            self.timer = 0.0;
                        }
                    }
                }
            }
            gt_state::DISABLED_SWEEP => {
                // `GT_RandMove` each tick: exactly two shared-RNG draws [M 00180348:44-45];
                // 3.0 s → obj+0xf6 [M case 5].
                self.timer += crate::grenade::REC_FRAME_RATE;
                let pitch = rng.frand_half(core::f32::consts::PI);
                let yaw = rng.frand_half(f32::from_bits(0x40c9_0fdb)); // 2π
                out.push(GTEvent::Sweep { yaw, pitch });
                if self.timer > 3.0 {
                    self.state = self.prev_state;
                }
            }
            gt_state::DESTROYED => {
                if self.timer >= 10.0 {
                    if ctx.mp && self.facing != 1 {
                        self.delete_flag = true;
                        out.push(GTEvent::Exploded { pos: self.pos });
                    }
                } else {
                    self.timer += crate::grenade::REC_FRAME_RATE;
                    if ctx.frame % 10 == 0 {
                        out.push(GTEvent::Smoke { pos: self.pos });
                    }
                }
            }
            gt_state::MOUNTED => {
                // The mounted aim + fire [M case 7]: aim step, the same 0.1 s cadence,
                // spin_vel refreshed on fire, exit action 14 → `GT_LoseControl`.
                if let Some((yaw_in, pitch_in, fire, exit)) = ctx.mount {
                    self.mount_aim(yaw_in, pitch_in);
                    let m = self.aim_matrix();
                    let h = (m[2][0] * m[2][0] + m[2][2] * m[2][2]).sqrt();
                    self.ease_pose((crate::trig::atan2f_libm(m[2][0], m[2][2]), -(m[2][1].atan2(h))));
                    self.fire_timer += crate::grenade::REC_FRAME_RATE;
                    // ext+0xD8 decays ×0.9 every mounted tick [M 00180498:496].
                    self.spin_vel *= 0.9;
                    if fire && self.fire_timer > 0.1 {
                        self.fire_timer = 0.0;
                        if self.hp > 0 {
                            self.hp -= 1;
                        }
                        // The shot runs whatever the counter reads (the decrement is the only
                        // `0 < ext+0xF0` gate) [M case 7]: `Mat_GetDir` of the barrel, muzzle =
                        // barrel + dir·ext+0xD0, `Sound_Play3D(0x10a)`, `Bullet_init(−1,
                        // ext+0xB8 (the owner), …, 0x2c55d0)`. The barrel follows script A, which
                        // case 7 poses at obj+0x90·RotY(ext+0x94)·RotX(ext+0x90) — the port's
                        // `aim_matrix` (the slerp onto it is the draw side's).
                        let m = self.aim_matrix();
                        let dir = [m[2][0], m[2][1], m[2][2]];
                        out.push(GTEvent::Fire {
                            muzzle: [self.pos[0] + dir[0] * self.muzzle_offset, self.pos[1] + dir[1] * self.muzzle_offset, self.pos[2] + dir[2] * self.muzzle_offset],
                            dir,
                            sound: SOUND_FIRE,
                        });
                        // The same barrel toggle + alternate-shot smoke as case 4 [M disasm
                        // 0x181304-0x181350 — the case-7 fire tail's own copy].
                        self.barrel_alt = !self.barrel_alt;
                        if self.barrel_alt {
                            out.push(GTEvent::ShotSmoke { muzzle: [self.pos[0] + dir[0] * self.muzzle_offset, self.pos[1] + dir[1] * self.muzzle_offset, self.pos[2] + dir[2] * self.muzzle_offset] });
                        }
                        self.show_flash();
                    }
                    // A held trigger re-arms the spin every tick, shot or not [M 00180498:536];
                    // the barrel turns while the velocity is non-zero [M :538-556].
                    if fire {
                        self.spin_vel = crate::grenade::REC_FRAME_RATE * 6.2831855 + crate::grenade::REC_FRAME_RATE * 6.2831855;
                    }
                    if self.registered[1] != 0 && self.spin_vel != 0.0 {
                        self.spin_advance(self.spin_vel);
                    }
                    if exit {
                        self.lose_control();
                    }
                }
            }
            _ => {}
        }
    }
}

impl Default for GTurret {
    fn default() -> Self {
        // A record the tests can mutate; `from_placement` is the real path (script ids 1/2
        // stand in for the loaded scripts).
        GTurret {
            placement: 0,
            facing: 0,
            param1: 0,
            script_a: 1,
            script_b: 2,
            param4: 0,
            param5: 0,
            hp: 200,
            state: 0,
            health: 100.0,
            contact: [0.0; 2],
            prev_state: 0,
            timer: 0.0,
            fire_timer: 0.0,
            target: 0,
            owner: 0,
            registered: [0; 3],
            flash: 0,
            barrel_alt: false,
            flash_hidden: false,
            controlled: 0,
            delete_flag: false,
            aim: [0.0; 3],
            spin: 0.0,
            spin_vel: 0.0,
            spin_draw: None,
            muzzle_offset: 0.0,
            scripts_done: true,
            pos: [0.0; 3],
            script_player_a: usize::MAX,
            script_player_b: usize::MAX,
            fire_dir: [0.0, 0.0, 1.0],
            pose: None,
            sp_key: usize::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 0017ffa0:69,83-96] the gate, the ext map, the HP selector.
    #[test]
    fn the_creator_param_map() {
        let ps: Vec<(u32, u32)> = vec![(0, 0), (1, 7), (2, 0x0600_0100), (3, 0x0600_0101), (4, 9), (5, 11)];
        let t = from_placement(2, &ps).expect("scripts present");
        assert_eq!(t.placement, 2);
        assert_eq!((t.facing, t.param1), (0, 7));
        assert_eq!((t.script_a, t.script_b), (0x0600_0100, 0x0600_0101));
        assert_eq!((t.param4, t.param5), (9, 11));
        assert_eq!(t.hp, 200, "param 0 == 0 → HP 200 [M 0017ffa0:92-96]");
        assert_eq!(t.state, 0);
        // Sister (can fail): a nonzero facing flips BOTH the facing mode and the HP rule.
        let ps1: Vec<(u32, u32)> = vec![(0, 3), (1, 7), (2, 0x60), (3, 0x61), (4, 9), (5, 11)];
        let t1 = from_placement(0, &ps1).expect("scripts present");
        assert_eq!(t1.facing, 3);
        assert_eq!(t1.hp, 0xffff, "param 0 != 0 → invulnerable 0xffff [M 0017ffa0:92-96]");
    }

    // [M 0017ffa0:69] either script param zero → the game never creates the object.
    #[test]
    fn the_script_gate() {
        assert!(from_placement(0, &[(2, 0x60), (3, 0)]).is_none());
        assert!(from_placement(0, &[(2, 0), (3, 0x61)]).is_none());
        assert!(from_placement(0, &[]).is_none());
        // Sister (can fail): the gate is BOTH scripts — one present is still None.
        assert!(from_placement(0, &[(2, 0x60)]).is_none());
    }

    // [M 00178f68:129-137] the deploy offset: x+sin(yaw), z+sin(yaw+π/2).
    #[test]
    fn the_deploy_position_offset() {
        let p = deploy_position([10.0, 2.0, 20.0], 0.0);
        assert_eq!(p[1], 2.0, "y untouched");
        assert!((p[0] - 10.0).abs() < 1e-4, "sin(0) = 0");
        assert!((p[2] - 21.0).abs() < 1e-4, "sin(π/2) = 1");
        // Sister (can fail): yaw π/2 moves x by 1, z by 0 — a swapped sin would flip them.
        let q = deploy_position([10.0, 2.0, 20.0], HALF_PI);
        assert!((q[0] - 11.0).abs() < 1e-4);
        assert!((q[2] - 20.0).abs() < 1e-4);
    }
    // --- GT_Update 0x180498 / the state machine ---

    fn ctx<'a>(ch: &'a crate::triggers::Channels, cands: &'a [TrackCand]) -> GTCtx<'a> {
        GTCtx { channels: ch, mp: false, frame: 1, cands, mount: None }
    }

    fn step(t: &mut GTurret, ch: &crate::triggers::Channels, cands: &[TrackCand], rng: &mut crate::drone::Rng) -> Vec<GTEvent> {
        let mut out = Vec::new();
        let c = ctx(ch, cands);
        t.update(&c, rng, &mut out);
        out
    }

    /// [M 00180498 case 0→1→4] a facing-0 turret: dormant arms in one tick (no channel gate),
    /// waits one more tick for the scripts, then combat.
    #[test]
    fn dormant_arms_into_combat() {
        let ch = crate::triggers::Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() };
        let mut rng = crate::drone::Rng::new(1);
        let mut t = GTurret::default();
        assert_eq!(t.state, 0);
        // The arming step carries the case-0 tail's SCRIPTINFO+0x9c = 0 clears as the rewind
        // event [M 00180498:222-227] — the caller zeroes the mounted players' delays with it.
        assert_eq!(step(&mut t, &ch, &[], &mut rng), vec![GTEvent::RewindScripts]);
        assert_eq!(t.state, 1, "case 0 tail: scripts reset, timer 0, -> 1 [M 00180498 case 0]");
        assert!(!t.scripts_done, "case 0 resets both scripts' obj+0x9c [M case 0 tail]");
        // The script VM (the tracked gap) plays both scripts and sets obj+0x9c on each —
        // the port's stand-in is the record's flag.
        t.scripts_done = true;
        step(&mut t, &ch, &[], &mut rng);
        assert_eq!(t.state, 4, "case 1: both scripts done -> 4 [M 00180498 case 1]");
        // Sister (can fail): with the scripts NOT done the turret must wait in 1.
        let mut t2 = GTurret::default();
        t2.scripts_done = false;
        step(&mut t2, &ch, &[], &mut rng);
        assert_eq!(t2.state, 1);
        step(&mut t2, &ch, &[], &mut rng);
        assert_eq!(t2.state, 1, "scripts unfinished → still arming");
    }

    /// [M 00180498 case 4] combat: a target in range with LOS fires on the 0.1 s cadence,
    /// spending one shot per bullet; no target → no fire.
    /// [M 0x180498:293, 401-414] the facing-1 re-holster gate (review 2026-10-21): with the enable
    /// channel ON and no disable channel the turret never re-holsters; with the enable channel OFF
    /// it re-holsters after 0.5 s of CONTINUOUS no-target time — a tracked frame zeroes the timer.
    #[test]
    fn reholster_gate_and_tracked_frame_reset() {
        let mut ch = crate::triggers::Channels::default();
        ch.set(18, 1, 0);
        let none: Vec<TrackCand> = Vec::new();
        let mut rng = crate::drone::Rng::new(1);
        let mut t = GTurret { state: gt_state::COMBAT, facing: 1, param4: 18, param5: 0, ..GTurret::default() };
        for f in 0..120u32 {
            let ctx = GTCtx { channels: &ch, mp: false, frame: f, cands: &none, mount: None };
            t.update(&ctx, &mut rng, &mut Vec::new());
        }
        assert_eq!(t.state, gt_state::COMBAT, "e8 ON with ea == 0 returns before the timer [M :406-409]");
        // Sister: the enable channel OFF → the 0.5 s count runs and the turret re-holsters.
        let ch_off = crate::triggers::Channels::default();
        let mut t2 = GTurret { state: gt_state::COMBAT, facing: 1, param4: 18, param5: 0, ..GTurret::default() };
        let mut moved = None;
        for f in 0..120u32 {
            let ctx = GTCtx { channels: &ch_off, mp: false, frame: f, cands: &none, mount: None };
            t2.update(&ctx, &mut rng, &mut Vec::new());
            if t2.state == gt_state::RESTORING { moved = Some(f); break; }
        }
        assert!(moved.is_some_and(|f| f >= 29), "e8 OFF → > 0.5 s → state 2 [M :415-421], got {moved:?}");
        // A tracked frame zeroes the timer: 20 idle frames, one tracked, 20 idle → still COMBAT.
        let bond = [TrackCand { obj: u32::MAX, pos: [0.0, 0.0, 5.0], copter: false, los_ok: true }];
        let mut t3 = GTurret { state: gt_state::COMBAT, facing: 1, param4: 18, param5: 0, ..GTurret::default() };
        for f in 0..41u32 {
            let c: &[TrackCand] = if f == 20 { &bond } else { &none };
            let ctx = GTCtx { channels: &ch_off, mp: false, frame: f, cands: c, mount: None };
            t3.update(&ctx, &mut rng, &mut Vec::new());
        }
        assert_eq!(t3.state, gt_state::COMBAT, "the tracked frame reset +0xDC — 20 + 20 idle frames stay under 0.5 s [M :293]");
        // Sister: 41 idle frames with no tracked frame DO cross 0.5 s.
        let mut t4 = GTurret { state: gt_state::COMBAT, facing: 1, param4: 18, param5: 0, ..GTurret::default() };
        for f in 0..41u32 {
            let ctx = GTCtx { channels: &ch_off, mp: false, frame: f, cands: &none, mount: None };
            t4.update(&ctx, &mut rng, &mut Vec::new());
        }
        assert_ne!(t4.state, gt_state::COMBAT, "41 continuous idle frames re-holster");
    }

    #[test]
    fn combat_fires_on_cadence_and_spends_shots() {
        let ch = crate::triggers::Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() };
        let mut rng = crate::drone::Rng::new(1);
        let mut t = GTurret::default();
        t.state = 4;
        t.pos = [0.0, 0.0, 0.0];
        // The candidates sit on +x: the create-time fire direction must point at them or the
        // front-cone test rejects everything [M 0x17f9c8:252-257].
        t.fire_dir = [1.0, 0.0, 0.0];
        let far = [60.0, 0.0, 0.0]; // beyond the SP range (dist² 3600 > 2500) [M 0x17f9c8:92]
        let cand_far = TrackCand { obj: 7, pos: far, copter: false, los_ok: true };
        let cand_near = TrackCand { obj: 7, pos: [10.0, 0.0, 0.0], copter: false, los_ok: true }; // inside the 50-unit SP range
        // out of range: no target, no fire, no sting
        let ev = step(&mut t, &ch, &[cand_far], &mut rng);
        assert!(ev.is_empty(), "out-of-range candidate → no events");
        assert_eq!(t.target, 0);
        assert_eq!(t.hp, 200);
        // in range: sting on acquire, fire at >0.1 s = 7 ticks (7/60 = 0.1167)
        let mut fired = 0;
        let mut stung = 0;
        for _ in 0..7 {
            let ev = step(&mut t, &ch, &[cand_near], &mut rng);
            fired += ev.iter().filter(|e| matches!(e, GTEvent::Fire { .. })).count();
            stung += ev.iter().filter(|e| matches!(e, GTEvent::CombatSting)).count();
        }
        assert_eq!(stung, 7, "Music_Event(2,1) each tick with a target [M case 4]");
        assert_eq!(fired, 1, "0.1 s cadence: the 7th tick at 1/60 crosses 0.1 [M case 4]");
        assert_eq!(t.hp, 199, "one shot spent [M case 4: ext+0xf0 decrement]");
        // Sister (can fail): the 6th tick must NOT have fired (the cadence gate is strict).
        let mut t2 = GTurret::default();
        t2.state = 4;
        t2.pos = [0.0, 0.0, 0.0];
        t2.fire_dir = [1.0, 0.0, 0.0]; // the +x candidate must sit in the cone [M 0x17f9c8:252-257]
        for _ in 0..6 { step(&mut t2, &ch, &[cand_near], &mut rng); }
        assert_eq!(t2.hp, 200, "6 ticks = 0.1 s exactly → not > 0.1 → no shot");
    }

    /// [M 0x180498:295-330 + :344-352] the round flies along the EASED BARREL — case 4 eases the
    /// pose first (`Mat_LookAt(obj+0x30 → target+0x30)` through `Quat_Slerp_Acc(REC_FRAME_RATE·4.0)`
    /// = a 6.67%/tick first-order ease) and the fire block reads `Mat_GetDir(ext+0xc4's obj+0x90)`,
    /// the just-slerped barrel matrix — not the dead-on line to the target. The old port fired
    /// dead-on: the 0700000b wall guns landed every round on a runner and the exec-floor return
    /// corridor cost 42 × −2.40 (walkreal_0700000b_live real-damage log) where the game's lagging
    /// barrel sweeps behind him. Sister: a still target is the ease's fixed point — the barrel
    /// converges onto the dead-on line, so a stationary Bond IS hit dead-on.
    #[test]
    fn combat_fire_rides_the_eased_barrel_not_the_dead_on_line() {
        let ch = crate::triggers::Channels::default();
        let mut rng = crate::drone::Rng::new(1);
        let mut t = GTurret { state: gt_state::COMBAT, pos: [0.0, 5.0, 0.0], fire_dir: [1.0, 0.0, 0.0], ..GTurret::default() };
        // The runner: +z every tick, 10 units out — the corridor sprint. The create facing
        // (+x) passes the front-cone gate [M 0x17f9c8:252-257] for the whole run.
        let mut fired: Option<([f32; 3], u32)> = None;
        for tick in 0..20u32 {
            let cand = TrackCand { obj: 7, pos: [10.0, 5.0, 3.0 + tick as f32], copter: false, los_ok: true };
            let mut out = Vec::new();
            let ctx = GTCtx { channels: &ch, mp: false, frame: tick, cands: &[cand], mount: None };
            t.update(&ctx, &mut rng, &mut out);
            if let Some(GTEvent::Fire { dir, .. }) = out.iter().find(|e| matches!(e, GTEvent::Fire { .. })) {
                fired = Some((*dir, tick));
                break;
            }
        }
        let (dir, at) = fired.expect("the 0.1 s cadence fires by the 7th tick [M case 4]");
        assert!(at >= 6, "first fire at tick {at}");
        // Dead-on at the fire tick's candidate would be normalise((10, 0, 3+at)); the eased
        // barrel still carries the create facing's +x bias — its yaw sits ABOVE the target's
        // (toward 90°), i.e. a smaller z and a larger x component: it points BEHIND the runner.
        let m = (10.0f32 * 10.0 + (3.0 + at as f32) * (3.0 + at as f32)).sqrt();
        let dead = [10.0 / m, 0.0, (3.0 + at as f32) / m];
        assert!(dir[2] < dead[2] - 0.05, "the barrel lags behind the runner: dir z {:.3} vs dead-on z {:.3}", dir[2], dead[2]);
        assert!(dir[0] > dead[0] + 0.05, "dir x {:.3} vs dead-on x {:.3}", dir[0], dead[0]);
        // And the fired dir IS this tick's eased pose forward (the ease ran before the fire).
        let (yaw, pitch) = t.pose.expect("a tracked tick eased the pose");
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let pose_fwd = [sy * cp, -sp, cy * cp];
        for k in 0..3 { assert!((dir[k] - pose_fwd[k]).abs() < 1e-5, "dir {dir:?} vs pose fwd {pose_fwd:?}"); }
        // Sister: a STILL target is the fixed point — the barrel converges onto the dead-on
        // line and the rounds land on him (the same law, no lag left).
        let mut t2 = GTurret { state: gt_state::COMBAT, pos: [0.0, 5.0, 0.0], fire_dir: [1.0, 0.0, 0.0], ..GTurret::default() };
        let cand = TrackCand { obj: 7, pos: [10.0, 5.0, 8.0], copter: false, los_ok: true };
        let mut last: Option<[f32; 3]> = None;
        for tick in 0..220u32 {
            let mut out = Vec::new();
            let ctx = GTCtx { channels: &ch, mp: false, frame: tick, cands: &[cand], mount: None };
            t2.update(&ctx, &mut rng, &mut out);
            if let Some(GTEvent::Fire { dir, .. }) = out.iter().find(|e| matches!(e, GTEvent::Fire { .. })) { last = Some(*dir); }
        }
        let d = last.expect("the still target keeps the cadence firing");
        let m = (10.0f32 * 10.0 + 8.0 * 8.0).sqrt();
        let dead = [10.0 / m, 0.0, 8.0 / m];
        let dot: f32 = d.iter().zip(dead).map(|(a, b)| a * b).sum();
        assert!(dot > 0.999, "a still target's barrel converges onto the dead-on line, dot {dot}");
    }

    /// [M 0x17f9c8:92,158-166,178] the range rules: 2500 SP / 500 MP, copter ×0.25,
    /// prefer-current among the first two LOS candidates.
    #[test]
    fn track_range_and_preference_rules() {
        let mut t = GTurret::default();
        t.state = 4;
        t.pos = [0.0, 0.0, 0.0];
        // Every candidate below sits on +x: the fire direction must face them or the
        // front-cone dot test [M 0x17f9c8:252-257] rejects the lot.
        t.fire_dir = [1.0, 0.0, 0.0];
        // The game compares SQUARED distance to 2500 (SP) → 50 units [M 0x17f9c8:92-99].
        // A copter at 80: 6400 × 0.25 = 1600 < 2500 → in; a ground candidate at 52 (2704) → out.
        let copter = TrackCand { obj: 3, pos: [80.0, 0.0, 0.0], copter: true, los_ok: true };
        let guard = TrackCand { obj: 5, pos: [52.0, 0.0, 0.0], copter: false, los_ok: true };
        assert_eq!(t.track(&[guard, copter], false), Some(3), "copter dist² ×0.25 inside 2500 [M :158-166]");
        // Sister: 48 units (2304) is in range, 52 (2704) is not.
        let g2 = TrackCand { obj: 6, pos: [48.0, 0.0, 0.0], copter: false, los_ok: true };
        t.target = 0;
        assert_eq!(t.track(&[guard, g2], false), Some(6), "48 in, 52 out — the 50-unit SP range");
        // MP: squared distance under 500 → 22.36 units; 25 (625) is out, 20 (400) in [M :178].
        t.target = 0;
        assert_eq!(t.track(&[TrackCand { obj: 9, pos: [25.0, 0.0, 0.0], copter: false, los_ok: true }], true), None, "MP 25 units out");
        assert_eq!(t.track(&[TrackCand { obj: 9, pos: [20.0, 0.0, 0.0], copter: false, los_ok: true }], true), Some(9), "MP 20 units in");
        // Facing-1 (wall) turrets track Bond only — drones are candidates for facing 0 alone [M :101].
        let mut w = GTurret { state: 4, facing: 1, fire_dir: [1.0, 0.0, 0.0], ..GTurret::default() };
        let near_guard = TrackCand { obj: 4, pos: [5.0, 0.0, 0.0], copter: false, los_ok: true };
        assert_eq!(w.track(&[near_guard], false), None, "a wall gun ignores guards");
        let bond = TrackCand { obj: u32::MAX, pos: [10.0, 0.0, 0.0], copter: false, los_ok: true };
        assert_eq!(w.track(&[near_guard, bond], false), Some(u32::MAX), "and takes Bond");
        // prefer-current [M :246-300]: the walk remembers the first TWO candidates WITH LOS
        // and keeps the current target while it is one of them — even a farther current.
        let a = TrackCand { obj: 1, pos: [10.0, 0.0, 0.0], copter: false, los_ok: true };
        let b = TrackCand { obj: 2, pos: [20.0, 0.0, 0.0], copter: false, los_ok: false };
        let c3 = TrackCand { obj: 3, pos: [30.0, 0.0, 0.0], copter: false, los_ok: true };
        t.target = 3;
        assert_eq!(t.track(&[a, b, c3], false), Some(3), "current is the second LOS candidate → kept");
        t.target = 2;
        assert_eq!(t.track(&[a, b, c3], false), Some(1), "current without LOS → the first LOS candidate wins");
        t.target = 1;
        assert_eq!(t.track(&[a, b, c3], false), Some(1), "current is the first LOS → kept");
        t.target = 0;
        assert_eq!(t.track(&[b, c3], false), Some(3), "a nearer candidate without LOS is skipped [M :271-278]");
    }

    /// [M 00180498:236-258] the death gate: health < 0 OR the shot counter at 0 → state 6,
    /// one DestroyedBurst, then smoke on the %10 gate; state 7 loses control first.
    #[test]
    fn death_gate_and_wreck_smoke() {
        let ch = crate::triggers::Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() };
        let mut rng = crate::drone::Rng::new(1);
        let mut t = GTurret::default();
        t.state = 4;
        t.apply_damage(100.5); // GT_ApplyDamage 0x1816d8: 100.0 - 100.5 < 0
        assert!((t.health - (-0.5)).abs() < 1e-4);
        let ev = step(&mut t, &ch, &[], &mut rng);
        assert_eq!(t.state, 6, "health < 0 → destroyed [M 00180498:236-244]");
        assert_eq!(ev.iter().filter(|e| matches!(e, GTEvent::DestroyedBurst { .. })).count(), 1);
        // mounted dies: LoseControl runs before state 6 [M :245-247]
        let mut t2 = GTurret::default();
        t2.state = 7;
        t2.controlled = 1;
        t2.apply_damage(101.0);
        let _ = step(&mut t2, &ch, &[], &mut rng);
        assert_eq!(t2.state, 6);
        assert_eq!(t2.controlled, 0, "state 7 → LoseControl before 6 [M :245-247]");
        // the shot-counter arm: hp == 0 kills too [M :236-244 *(short*)(ext+0xf0) == 0]
        let mut t3 = GTurret::default();
        t3.state = 4;
        t3.hp = 0;
        let ev = step(&mut t3, &ch, &[], &mut rng);
        assert_eq!(t3.state, 6);
        assert!(matches!(ev.first(), Some(GTEvent::DestroyedBurst { .. })));
        // wreck smoke only on frames %10 == 0, timer under 10 s [M case 6]
        let mut ch10 = crate::triggers::Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() };
        let _ = &mut ch10;
        let mut t4 = GTurret::default();
        t4.state = 6;
        t4.timer = 0.0;
        let mut c = ctx(&ch, &[]);
        c.frame = 10;
        let mut out = Vec::new();
        t4.update(&c, &mut rng, &mut out);
        assert!(matches!(out.last(), Some(GTEvent::Smoke { .. })), "frame 10 % 10 == 0 → smoke [M case 6]");
        let mut c11 = ctx(&ch, &[]);
        c11.frame = 11;
        let mut out = Vec::new();
        t4.update(&c11, &mut rng, &mut out);
        assert!(!out.iter().any(|e| matches!(e, GTEvent::Smoke { .. })), "frame 11 → no smoke");
    }

    /// [M 001816a8 / 00180348 / case 5] disable: 4 → 5 with prev 4 and timer 0; the sweep
    /// consumes exactly two RNG draws per tick; 3.0 s back to obj+0xf6.
    #[test]
    fn disable_sweeps_two_draws_per_tick_then_returns() {
        let ch = crate::triggers::Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() };
        let mut rng = crate::drone::Rng::new(1);
        let mut t = GTurret::default();
        t.state = 5; // direct, to isolate the sweep: disable() itself is covered below
        let seed = crate::drone::Rng::new(1);
        let mut expect = seed;
        let _ = expect.frand_half(core::f32::consts::PI);
        let _ = expect.frand_half(f32::from_bits(0x40c9_0fdb));
        let ev = step(&mut t, &ch, &[], &mut rng);
        assert!(matches!(ev.first(), Some(GTEvent::Sweep { .. })));
        assert_eq!(rng.x, expect.x, "exactly two draws per sweep tick [M 00180348:44-45]");
        // 3.0 s = 181 ticks at 1/60 → back to prev (4) [M case 5: 3.0 < ext+0xdc]
        let mut t2 = GTurret::default();
        t2.state = 4;
        t2.disable(); // GT_Disable 0x1816a8
        assert_eq!(t2.state, 5);
        assert_eq!(t2.prev_state, 4, "prev = 4 [M 001816a8]");
        assert_eq!(t2.timer, 0.0);
        for _ in 0..181 { step(&mut t2, &ch, &[], &mut rng); }
        assert_eq!(t2.state, 4, "181 ticks = 3.0167 s > 3.0 → back to prev [M case 5]");
        // Sister: 180 ticks = 3.0 s exactly → NOT > 3.0 → still sweeping.
        let mut t3 = GTurret::default();
        t3.state = 4;
        t3.disable();
        for _ in 0..180 { step(&mut t3, &ch, &[], &mut rng); }
        assert_eq!(t3.state, 5, "3.0 s exactly keeps sweeping (strict >)");
        // disable() outside state 4 does nothing [M 001816a8's `if (state == 4)`]
        let mut t4 = GTurret::default();
        t4.state = 6;
        t4.disable();
        assert_eq!(t4.state, 6);
    }

    /// [M 00180498 cases 2/3 + 001816f0] activate re-holsters: 4 → 2, 1.2 s → 3, scripts → 0;
    /// states 0/1/2/3/6 ignore it.
    #[test]
    fn activate_restores_through_2_3_0() {
        let ch = crate::triggers::Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() };
        let mut rng = crate::drone::Rng::new(1);
        let mut t = GTurret::default();
        t.state = 4;
        t.activate(); // GT_Activate 0x1816f0, facing 0
        assert_eq!(t.state, 2);
        assert_eq!(t.timer, 0.0);
        // 1.2 s = 73 ticks (73/60 = 1.2167 > 1.2) → state 3 [M case 2]
        for _ in 0..73 { step(&mut t, &ch, &[], &mut rng); }
        assert_eq!(t.state, 3);
        assert!(!t.scripts_done, "case 2's expiry reset the scripts for the return anim [M case 2]");
        t.scripts_done = true; // the script VM replays them (the port's stand-in)
        let ev = step(&mut t, &ch, &[], &mut rng); // scripts done → facing 0 → the case drops [M case 3]
        assert!(ev.iter().any(|e| matches!(e, GTEvent::DropPickup { .. })),
            "a folded facing-0 turret drops the 0x52 case [M 0x180498 case 3, disasm 0x18092c-0x18098c]");
        assert!(t.delete_flag, "and deletes itself (obj+0xFE |= 1) [M 0x180980-0x18098c]");
        // Sister: a facing-1 turret folding through state 3 goes back to DORMANT and stays [M bnel].
        let mut w = GTurret { state: 3, facing: 1, scripts_done: true, ..GTurret::default() };
        let ev = step(&mut w, &ch, &[], &mut rng);
        assert_eq!(w.state, 0, "facing 1 → state 0 (the bnel delay-slot store)");
        assert!(!w.delete_flag && !ev.iter().any(|e| matches!(e, GTEvent::DropPickup { .. })),
            "the wall gun neither drops a case nor deletes itself");
        // Sister: 72 ticks = 1.2 s exactly → still restoring.
        let mut t2 = GTurret::default();
        t2.state = 2;
        for _ in 0..72 { step(&mut t2, &ch, &[], &mut rng); }
        assert_eq!(t2.state, 2, "1.2 s exactly keeps restoring (strict >)");
        // the ignore set [M 001816f0 switch cases 0/1/2/3/6]
        for s in [0u16, 1, 2, 3, 6] {
            let mut t3 = GTurret::default();
            t3.state = s;
            t3.activate();
            assert_eq!(t3.state, s, "state {s} ignores activate [M 001816f0]");
        }
        // facing 1 turrets do not re-holster via activate [M 001816f0's facing check]
        let mut t4 = GTurret::default();
        t4.state = 4;
        t4.facing = 1;
        t4.activate();
        assert_eq!(t4.state, 4);
    }

    /// [M 00181740 / 00181408 / 00181808] the mount pair and the death hook.
    #[test]
    fn take_and_lose_control_and_player_death() {
        let mut t = GTurret::default();
        t.state = 2;
        assert!(!t.take_control(), "state 2 cannot mount [M 00181740's state==4 gate]");
        t.state = 4;
        assert!(t.take_control());
        assert_eq!(t.state, 7);
        assert_eq!(t.controlled, 1);
        t.lose_control(); // GT_LoseControl 0x181408
        assert_eq!(t.state, 4, "7 → 4 [M 00181408:67-73]");
        assert_eq!(t.controlled, 0);
        // GT_PlayerHasDied 0x181808: only the dead owner's mounted turret dismounts
        let mut a = GTurret::default();
        a.state = 7;
        a.controlled = 1;
        a.owner = 11;
        let mut b = GTurret::default();
        b.state = 7;
        b.controlled = 1;
        b.owner = 12;
        let mut c = GTurret::default();
        c.state = 4;
        c.owner = 11;
        GTurret::player_has_died(&mut [a, b, c], 11);
        // (re-fetch through a fresh list — the helper takes a slice, values above were moved;
        // redo with a vec to assert)
        let mut list = vec![GTurret::default(), GTurret::default(), GTurret::default()];
        list[0].state = 7; list[0].controlled = 1; list[0].owner = 11;
        list[1].state = 7; list[1].controlled = 1; list[1].owner = 12;
        list[2].state = 4; list[2].owner = 11;
        GTurret::player_has_died(&mut list, 11);
        assert_eq!(list[0].state, 4, "owner 11's mounted turret dismounts [M 00181808]");
        assert_eq!(list[0].controlled, 0);
        assert_eq!(list[1].state, 7, "another owner's mounted turret stays");
        assert_eq!(list[2].state, 4, "a non-mounted turret is untouched (and state 4 stays 4)");
        // Sister: a state-7 turret of the dead owner MUST change — flip the owner and it must not.
        let mut solo = vec![GTurret::default()];
        solo[0].state = 7; solo[0].owner = 99;
        GTurret::player_has_died(&mut solo, 11);
        assert_eq!(solo[0].state, 7);
    }

    /// [M 00181538 / 00181678] register's bounds and delete's script drop.
    #[test]
    fn register_bounds_and_delete() {
        let mut t = GTurret::default();
        assert!(t.register(0, 0x55));
        assert!(t.register(2, 0x77));
        assert_eq!(t.registered, [0x55, 0, 0x77]);
        assert!(!t.register(3, 0x99), "idx 3 rejected [M 00181538's < 3 gate]");
        assert!(!t.register(1, 0), "obj 0 rejected [M 00181538's != 0 gate]");
        assert_eq!(t.registered[1], 0, "the rejected slot stays empty");
        // Sister: idx 2 is the last legal slot — it must take.
        assert!(t.register(2, 0x13));
        assert_eq!(t.registered[2], 0x13);
        // GT_Delete 0x181678: Script_Free(script_a); Script_Free(script_b)
        let mut d = GTurret::default();
        d.script_a = 0x11;
        d.script_b = 0x22;
        d.delete();
        assert_eq!((d.script_a, d.script_b), (0, 0));
    }

    /// [M 00180498 case 7] the mounted aim clamps: pitch ±0.5, yaw ±1.5707964, scale 0.024543693.
    #[test]
    fn mount_aim_clamps() {
        let mut t = GTurret::default();
        t.mount_aim(-1000.0, -1000.0); // huge inputs
        assert_eq!(t.aim[0], 0.5, "pitch clamped +0.5 [M case 7]");
        assert!((t.aim[1] - 1.5707964).abs() < 1e-6, "yaw clamped +π/2 [M case 7]");
        let mut t2 = GTurret::default();
        t2.mount_aim(1000.0, 1000.0);
        assert_eq!(t2.aim[0], -0.5, "pitch clamped −0.5 (the input is negated) [M case 7]");
        assert!((t2.aim[1] + 1.5707964).abs() < 1e-6);
        // one unit of input moves the aim by exactly the scale [M 00180498 case 7 literal]
        let mut t3 = GTurret::default();
        t3.mount_aim(1.0, 0.0);
        assert!((t3.aim[1] - (-AIM_INPUT_SCALE)).abs() < 1e-9, "yaw step = 0.024543693");
    }

    /// [M 0x17f9c8:62-64] SP: a player-owned turret (the deployed Ronin) never takes Bond as a
    /// candidate; a guard in front of it is the target.
    /// [M 00180498:496, 536-556] the mounted spin: ×0.9 per tick, re-armed to 2·2π/60 by a held
    /// trigger, the barrel drawn at the wrapped angle before each increment. Against the live
    /// capture [L PCSX2 audits/render-1101-roninview2/live_ab.txt]: R1 held 8 frames to take control,
    /// 45 frames later ext+0xD4 = −2.7410 (and the first capture's barrel row0 = RotZ(−2.741)).
    #[test]
    fn mounted_spin_matches_the_live_angle() {
        let ch = crate::triggers::Channels::default();
        let mut rng = crate::drone::Rng::default();
        let mut t = GTurret::default();
        t.state = gt_state::MOUNTED;
        t.registered = [1, 2, 3];
        let run = |t: &mut GTurret, rng: &mut crate::drone::Rng, fire: bool| {
            let mut out = Vec::new();
            let c = GTCtx { channels: &ch, mp: false, frame: 1, cands: &[], mount: Some((0.0, 0.0, fire, false)) };
            t.update(&c, rng, &mut out);
        };
        run(&mut t, &mut rng, false);
        assert_eq!(t.spin_draw, None, "no spin before the trigger (ext+0xD8 = 0)");
        for _ in 0..8 { run(&mut t, &mut rng, true); }
        assert!(t.spin_draw.is_some(), "a held trigger turns the barrel");
        for _ in 0..45 { run(&mut t, &mut rng, false); }
        assert!((t.spin - (-2.741)).abs() < 0.01, "ext+0xD4 after the 8-frame press + 45 frames: {}", t.spin);
        assert!(t.spin_draw.is_some_and(|a| a.abs() < 3.1415927), "drawn angle wrapped into (−π, π)");
        // Sister: without the ×0.9 decay the angle would run on (45 more increments of 0.209).
        let undecayed = 8.0 * 0.209_44 + 45.0 * 0.209_44;
        assert!((undecayed % 6.283_185_5 - 3.141_592_7 - t.spin).abs() > 0.05);
        // Without a registered barrel the block never runs.
        let mut bare = GTurret::default();
        bare.state = gt_state::MOUNTED;
        for _ in 0..4 { run(&mut bare, &mut rng, true); }
        assert_eq!((bare.spin, bare.spin_draw), (0.0, None), "registered[1] = 0 → no spin");
    }

    /// [M 00180498:360-382] a tracked combat frame spins the barrel at a fixed 2·2π/60 per tick
    /// (ext+0xD8 untouched); an untracked frame leaves it at its script pose.
    #[test]
    fn combat_spin_is_fixed_while_tracking() {
        let ch = crate::triggers::Channels::default();
        let mut rng = crate::drone::Rng::default();
        let mut t = GTurret::default();
        t.state = gt_state::COMBAT;
        t.registered = [1, 2, 3];
        t.pos = [0.0, 0.0, 0.0];
        t.fire_dir = [0.0, 0.0, 1.0];
        step(&mut t, &ch, &[], &mut rng);
        assert_eq!(t.spin_draw, None, "nothing tracked → no spin block");
        let cand = [TrackCand { obj: 5, pos: [0.0, 0.0, 5.0], copter: false, los_ok: true }];
        step(&mut t, &ch, &cand, &mut rng);
        assert_eq!(t.target, 5, "tracked");
        assert_eq!(t.spin_draw, Some(0.0), "drawn at the pre-increment angle");
        step(&mut t, &ch, &cand, &mut rng);
        let inc = 2.0 * 6.283_185_5 / 60.0;
        assert!((t.spin - 2.0 * inc).abs() < 1e-5 && t.spin_vel == 0.0, "fixed step, D8 untouched: {} {}", t.spin, t.spin_vel);
    }

    #[test]
    fn player_owned_turret_skips_bond() {
        let mut t = from_deploy([0.0, 0.5, 0.0], [0.0, 0.0, 1.0]);
        let bond = TrackCand { obj: PLAYER_OWNER, pos: [0.0, 0.0, 3.0], copter: false, los_ok: true };
        let guard = TrackCand { obj: 4, pos: [0.0, 0.0, 6.0], copter: false, los_ok: true };
        assert_eq!(t.track(&[bond], false), None, "Bond in front of his own Ronin is not a target");
        assert_eq!(t.track(&[bond, guard], false), Some(4), "the guard behind him is");
        // Sister (can fail): a placed facing-0 turret (owner 0) DOES take Bond.
        let mut placed = GTurret { fire_dir: [0.0, 0.0, 1.0], ..GTurret::default() };
        assert_eq!(placed.track(&[bond], false), Some(PLAYER_OWNER));
    }

    /// [M 00181560 + 0017ffa0] the Ronin record: facing 0, HP 200, scripts 0x06000045/44,
    /// owner the player.
    #[test]
    fn the_deploy_record() {
        let t = from_deploy([1.0, 2.0, 3.0], [1.0, 0.0, 0.0]);
        assert_eq!((t.facing, t.param1, t.hp), (0, 2, 200));
        assert_eq!((t.script_a, t.script_b), (RONIN_SCRIPT_A, RONIN_SCRIPT_B));
        assert_eq!(t.owner, PLAYER_OWNER);
        assert_eq!(t.state, gt_state::DORMANT);
        assert!((t.base_yaw() - HALF_PI).abs() < 1e-5, "fire dir +x → yaw π/2");
        // Sister: a +z fire dir is yaw 0 — the base yaw follows the direction.
        assert!(from_deploy([0.0; 3], [0.0, 0.0, 1.0]).base_yaw().abs() < 1e-6);
    }

    /// [M 0x1ecfb8] `Collide_PlaceObject(0.5, 0.6)`: a wide floor seats the case 0.5 above it;
    /// no floor within 3.0, a wall within 0.37 or a ledge narrower than 0.6 refuses.
    #[test]
    fn place_object_rules() {
        let floor = |h: f32| vec![[[-5.0, 0.0, -5.0], [-5.0, 0.0, 5.0], [5.0, 0.0, 5.0]], [[-5.0, 0.0, -5.0], [5.0, 0.0, 5.0], [5.0, 0.0, -5.0]]]
            .into_iter().map(|t: [[f32; 3]; 3]| t.map(|v| [v[0] * h, v[1], v[2] * h])).collect::<Vec<_>>();
        let w = crate::world::CollisionWorld::from_triangles(floor(1.0));
        let p = place_object(&w, [0.0, 0.4, 0.0], [0.0, 0.0, 1.0]).expect("a wide floor seats");
        assert!((p[1] - 0.5).abs() < 1e-3, "seat = floor + normal·0.5 (got {p:?})");
        assert!(place_object(&w, [0.0, 4.0, 0.0], [0.0, 0.0, 1.0]).is_none(), "the floor 4.05 below is past the 3.0 ray");
        // a wall 0.2 from the seat (plane distance 0.2 − 0.5 < −0.13) refuses
        let mut tris = floor(1.0);
        tris.push([[0.2, -1.0, -2.0], [0.2, 3.0, -2.0], [0.2, -1.0, 2.0]]);
        tris.push([[0.2, 3.0, -2.0], [0.2, 3.0, 2.0], [0.2, -1.0, 2.0]]);
        let ww = crate::world::CollisionWorld::from_triangles(tris);
        assert!(place_object(&ww, [0.0, 0.4, 0.0], [0.0, 0.0, 1.0]).is_none(), "a wall within 0.37 refuses");
        // a 0.4-wide pillar top cannot support the 0.6 reach
        let narrow = crate::world::CollisionWorld::from_triangles(floor(0.04));
        assert!(place_object(&narrow, [0.0, 0.4, 0.0], [0.0, 0.0, 1.0]).is_none(), "a narrow ledge refuses");
        // Sister: the same wall 0.45 away (0.45 − 0.5 = −0.05 > −0.13) allows the seat.
        let mut tris = floor(1.0);
        tris.push([[0.45, -1.0, -2.0], [0.45, 3.0, -2.0], [0.45, -1.0, 2.0]]);
        tris.push([[0.45, 3.0, -2.0], [0.45, 3.0, 2.0], [0.45, -1.0, 2.0]]);
        let ok = crate::world::CollisionWorld::from_triangles(tris);
        assert!(place_object(&ok, [0.0, 0.4, 0.0], [0.0, 0.0, 1.0]).is_some(), "a wall 0.45 away still seats");
    }

    /// fx-muzzle-and-trail row 6: the fire tail's barrel byte (`obj+0xf6 ^= 1`, the port's
    /// `barrel_alt`) toggles on EVERY shot and the shot smoke fires only when the toggled value
    /// is 1 — smoke on the odd shots [M disasm 0x180bd0-0x180bf8, the case-4 fire tail; the
    /// case-7 copy 0x181304-0x181350]. Fresh objects read 0, so the FIRST shot smokes. The smoke's
    /// muzzle is the same vector the round spawned from [M 0x180c3c].
    #[test]
    fn shot_smoke_fires_on_every_other_shot() {
        let ch = crate::triggers::Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() };
        let mut rng = crate::drone::Rng::new(3);
        let mut t = GTurret::default();
        t.state = 4;
        t.pos = [0.0, 0.0, 0.0];
        t.fire_dir = [1.0, 0.0, 0.0];
        let cand = TrackCand { obj: 7, pos: [10.0, 0.0, 0.0], copter: false, los_ok: true };
        let mut seq = Vec::new();
        let mut muzzles = Vec::new();
        for _ in 0..60 {
            let ev = step(&mut t, &ch, &[cand], &mut rng);
            for e in ev {
                match e {
                    GTEvent::Fire { muzzle, .. } => { seq.push(b'F'); muzzles.push(muzzle); }
                    GTEvent::ShotSmoke { muzzle } => { seq.push(b'S'); assert_eq!(Some(&muzzle), muzzles.last(), "the smoke's muzzle is the round's"); }
                    _ => {}
                }
            }
            if seq.len() >= 8 { break; }
        }
        assert!(seq.len() >= 8, "the cadence fires (got {seq:?})");
        let pat: String = String::from_utf8(seq.clone()).unwrap();
        assert!(pat.starts_with("FSFFS"), "smoke on the odd shots (barrel_alt starts clear), got {pat}");
        // Sister: a both-shots or no-shots toggle breaks the count.
        assert_eq!(pat.matches('S').count(), pat.matches('F').count().div_ceil(2));
    }
}
    /// fx-gas-and-blasts row 10 [M GT_Create 0x17ffa0:150-166]: the same-owner chain — placing a
    /// second turret with the same ext+0xb8 owner self-deletes the first with the 5/5/50
    /// 0x6000052 blast at its own +0x30. The game runs the sweep INSIDE GT_Create, so the level
    /// placement walk fires it per placement too.
    #[test]
    fn create_chain_blows_the_same_owner_turrets() {
        let mut turrets = vec![
            GTurret { owner: 7, pos: [1.0, 2.0, 3.0], ..GTurret::default() },
            GTurret { owner: 9, pos: [4.0, 5.0, 6.0], ..GTurret::default() },
            GTurret { owner: 7, pos: [7.0, 8.0, 9.0], ..GTurret::default() },
        ];
        // The new one is index 2 (owner 7): the other owner-7 turret (index 0) blows.
        let blown = GTurret::create_chain(&mut turrets, 2);
        assert_eq!(blown, vec![[1.0, 2.0, 3.0]], "the same-owner sibling dies [M 0x17ffa0:157-163]");
        assert!(turrets[0].delete_flag, "obj+0xfe |= 1 on the blown one");
        assert!(!turrets[1].delete_flag && !turrets[2].delete_flag, "the other owner and the new one stay");
        // Sister: an already-requested delete is not blown twice.
        turrets[1].owner = 7;
        turrets[1].delete_flag = true;
        let blown = GTurret::create_chain(&mut turrets, 2);
        assert!(blown.is_empty(), "a delete-flagged sibling is skipped");
    }
